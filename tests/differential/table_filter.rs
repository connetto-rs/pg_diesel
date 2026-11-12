//! Table-scoped catalog loading.

use pg_diesel::{
    database::AclTableGrant,
    model_metadata::PgRuleEntry,
    models::{RoleColumnGrants, Table},
};

use pg_diesel::sql_traits::{
    errors::{LookupError, ObjectKind},
    traits::{ForeignKeyLike, SequenceLike, SequenceOwner, TableLike, ViewLike},
};

use super::{
    AlreadyStored, BTreeSet, DatabaseLike, Facts, PgConnection, PgDieselDatabase,
    PgDieselDatabaseBuilder, SUPERUSER, Scope, SimpleConnection, describe, establish_connection,
    reference_docker,
};

/// Loads the selected tables within both fixture schemas.
fn load(connection: &mut PgConnection, tables: &[(&str, &str)]) -> PgDieselDatabase {
    PgDieselDatabaseBuilder::default()
        .connection(connection)
        .catalog("table_filter")
        .schemas(["app", "other"])
        .tables(tables.iter().copied())
        .try_into()
        .expect("the selected tables load")
}

/// Selects facts owned by the named table.
fn table_facts(facts: Facts, table: &str) -> Facts {
    let table_prefix = format!("table {table} / ");
    let grant_target = format!(" on {table} to ");
    let column_target = format!(" on {table}.");
    let rule_target = format!(" on {table} / ");
    facts
        .into_iter()
        .filter(|(key, _)| {
            key.starts_with(&table_prefix)
                || (key.starts_with("grant ")
                    && (key.contains(&grant_target) || key.contains(&column_target)))
                || (key.starts_with("rule ") && key.contains(&rule_target))
        })
        .collect()
}

#[tokio::test]
async fn selected_tables_preserve_local_facts() {
    let docker = reference_docker("table_filter").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "table_filter").unwrap();
    connection
        .batch_execute(include_str!("corpus/relations/table_filter.sql"))
        .unwrap();
    connection
        .batch_execute("ALTER SEQUENCE app.skipped_seq OWNED BY NONE")
        .unwrap();
    let whole: PgDieselDatabase = PgDieselDatabaseBuilder::default()
        .connection(&mut connection)
        .catalog("table_filter")
        .schemas(["app", "other"])
        .try_into()
        .unwrap();
    let scope = Scope {
        default_schema: "public".to_owned(),
        schemas: vec!["app".to_owned(), "other".to_owned()],
        ignored_roles: vec![SUPERUSER.to_owned()],
        extension_members: BTreeSet::new(),
    };
    let selected = load(&mut connection, &[("app", "kept"), ("app", "kept")]);
    assert_eq!(scope.tables(&selected), [("app".into(), "kept".into())]);
    let whole_facts = describe(&whole, &scope, &mut AlreadyStored).unwrap();
    let selected_facts = describe(&selected, &scope, &mut AlreadyStored).unwrap();
    assert_eq!(
        table_facts(selected_facts, "app.kept"),
        table_facts(whole_facts, "app.kept")
    );
    assert_eq!(selected.table_grants().count(), 3);
    assert_eq!(selected.column_grants().count(), 1);
    assert_eq!(selected.rules().count(), 1);
    assert_eq!(selected.views().count(), 1);
    assert_eq!(selected.sequences().count(), 1);

    let paired = load(&mut connection, &[("app", "kept"), ("other", "skipped")]);
    assert_eq!(
        scope.tables(&paired),
        [
            ("app".into(), "kept".into()),
            ("other".into(), "skipped".into())
        ]
    );
    let empty = load(&mut connection, &[]);
    assert_eq!(empty.tables().count(), 0);
    assert_eq!(empty.column_grants().count(), 0);
    assert_eq!(empty.rules().count(), 0);
    assert_eq!(empty.table_grants().count(), 2);
    let missing = load(&mut connection, &[("app", "missing")]);
    assert_eq!(missing.tables().count(), 0);

    connection
        .batch_execute("ALTER TABLE app.kept ADD FOREIGN KEY (value) REFERENCES app.skipped(id)")
        .unwrap();
    let with_external_key = load(&mut connection, &[("app", "kept")]);
    let table = with_external_key.tables().next().unwrap();
    let key = table
        .foreign_keys(&with_external_key)
        .unwrap()
        .find(|key| key.referenced_name() == "skipped")
        .unwrap();
    assert!(matches!(
        key.referenced_table(&with_external_key),
        Err(LookupError::TableNotFound { .. })
    ));
    assert_eq!(
        with_external_key
            .foreign_key_metadata(key)
            .unwrap()
            .referenced_columns(),
        ["id"]
    );
    connection
        .batch_execute("DROP TABLE app.kept CASCADE")
        .unwrap();
    assert_eq!(
        load(&mut connection, &[("app", "kept")]).tables().count(),
        0
    );
}

#[tokio::test]
async fn sequences_require_loaded_owning_tables() {
    let docker = reference_docker("table_filter").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "table_filter").unwrap();
    connection
        .batch_execute(include_str!("corpus/relations/table_filter.sql"))
        .unwrap();
    let whole = load(&mut connection, &[("app", "kept"), ("app", "skipped")]);
    let sequence = whole.sequences().next().unwrap();
    assert!(matches!(
        SequenceLike::owner(sequence, &whole).unwrap(),
        Some(SequenceOwner::Column(column))
            if column.column_name() == "id" && column.model().table_name == "skipped"
    ));

    let selected = load(&mut connection, &[("app", "kept")]);
    let sequence = selected.sequences().next().unwrap();
    assert_eq!(
        SequenceLike::owner(sequence, &selected).unwrap_err(),
        ObjectKind::Table.not_in_database("app.skipped")
    );
    assert_eq!(selected.sequences().count(), 1);
    assert_eq!(selected.table_grants().count(), 3);
}

#[tokio::test]
async fn sequence_owners_follow_catalog_column_positions() {
    let docker = reference_docker("table_filter").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "table_filter").unwrap();
    connection
        .batch_execute(
            "CREATE SCHEMA app;
             CREATE SCHEMA other;
             CREATE TABLE app.gapped (removed integer, kept integer, owned integer);
             ALTER TABLE app.gapped DROP COLUMN removed;
             CREATE SEQUENCE app.table_seq OWNED BY app.gapped.owned;
             CREATE VIEW app.owner_view AS SELECT 1 AS first, 2 AS owned;
             CREATE SEQUENCE app.view_seq OWNED BY app.owner_view.owned;
             CREATE SEQUENCE app.unowned;",
        )
        .unwrap();
    let database = load(&mut connection, &[("app", "gapped")]);
    let table_sequence = database
        .sequences()
        .find(|sequence| sequence.sequence_name() == "table_seq")
        .unwrap();
    assert!(matches!(
        SequenceLike::owner(table_sequence, &database).unwrap(),
        Some(SequenceOwner::Column(column))
            if column.column_name() == "owned" && column.model().ordinal_position == 3
    ));
    let view_sequence = database
        .sequences()
        .find(|sequence| sequence.sequence_name() == "view_seq")
        .unwrap();
    assert!(matches!(
        SequenceLike::owner(view_sequence, &database).unwrap(),
        Some(SequenceOwner::ViewColumn { view, ordinal: 1 })
            if view.view_name() == "owner_view"
    ));
    let unowned = database
        .sequences()
        .find(|sequence| sequence.sequence_name() == "unowned")
        .unwrap();
    assert!(SequenceLike::owner(unowned, &database).unwrap().is_none());

    let without_view: PgDieselDatabase = PgDieselDatabaseBuilder::default()
        .connection(&mut connection)
        .catalog("table_filter")
        .schema("other")
        .try_into()
        .unwrap();
    assert_eq!(
        SequenceLike::owner(view_sequence, &without_view).unwrap_err(),
        ObjectKind::View.not_in_database("app.owner_view")
    );

    connection
        .batch_execute("ALTER TABLE app.gapped DROP COLUMN owned")
        .unwrap();
    let without_column = load(&mut connection, &[("app", "gapped")]);
    assert_eq!(
        SequenceLike::owner(table_sequence, &without_column).unwrap_err(),
        ObjectKind::Column.not_in_database("app.gapped.owned")
    );
}

#[tokio::test]
async fn catalog_errors_report_unnamed_sequence_owner() {
    let docker = reference_docker("table_filter").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "table_filter").unwrap();
    connection
        .batch_execute(
            "CREATE SCHEMA app;
             CREATE SCHEMA other;
             CREATE TABLE app.owner (id integer);
             CREATE SEQUENCE app.owned_seq OWNED BY app.owner.id;
             UPDATE pg_catalog.pg_depend SET refobjsubid = 999
             WHERE classid = 'pg_class'::regclass
               AND objid = 'app.owned_seq'::regclass
               AND refclassid = 'pg_class'::regclass
               AND refobjid = 'app.owner'::regclass
               AND deptype = 'a';",
        )
        .unwrap();
    let database = load(&mut connection, &[("app", "owner")]);
    let sequence = database.sequences().next().unwrap();
    assert_eq!(
        SequenceLike::owner(sequence, &database).unwrap_err(),
        ObjectKind::Column.not_in_database("app.owner column 999")
    );
}

#[tokio::test]
async fn table_names_are_exact_and_intersect_schemas() {
    let docker = reference_docker("table_filter_names").await.unwrap();
    let mut connection: PgConnection =
        establish_connection(docker.port, "table_filter_names").unwrap();
    connection
        .batch_execute(
            r#"CREATE SCHEMA "odd.schema";
               CREATE SCHEMA other;
               CREATE TABLE "odd.schema"."odd.table" (id integer);
               CREATE TABLE "odd.schema"."Odd.Table" (id integer);
               CREATE TABLE other."odd.table" (id integer);
               CREATE EXTENSION file_fdw;
               CREATE SERVER files FOREIGN DATA WRAPPER file_fdw;
               CREATE FOREIGN TABLE "odd.schema".remote (id integer)
                   SERVER files OPTIONS (filename '/dev/null');"#,
        )
        .unwrap();
    let database: PgDieselDatabase = PgDieselDatabaseBuilder::default()
        .connection(&mut connection)
        .catalog("table_filter_names")
        .schema("odd.schema")
        .tables([("odd.schema", "Odd.Table")])
        .tables([
            ("odd.schema", "odd.table"),
            ("other", "odd.table"),
            ("odd.schema", "remote"),
            ("odd.schema", "' OR true --"),
        ])
        .try_into()
        .unwrap();
    assert_eq!(
        database
            .tables()
            .map(TableLike::table_name)
            .collect::<Vec<_>>(),
        ["odd.table"]
    );
    assert_eq!(
        database
            .foreign_tables()
            .map(TableLike::table_name)
            .collect::<Vec<_>>(),
        ["remote"]
    );
}

#[tokio::test]
async fn unfiltered_readers_return_all_schema_metadata() {
    let docker = reference_docker("unfiltered_readers").await.unwrap();
    let mut connection: PgConnection =
        establish_connection(docker.port, "unfiltered_readers").unwrap();
    connection
        .batch_execute(include_str!("corpus/relations/table_filter.sql"))
        .unwrap();
    let schemas = ["app".to_owned()];

    let tables = Table::load_all(&mut connection, "unfiltered_readers", "app").unwrap();
    assert_eq!(
        tables
            .iter()
            .map(|table| (table.table_schema.as_str(), table.table_name.as_str()))
            .collect::<Vec<_>>(),
        [("app", "kept"), ("app", "skipped")]
    );

    let grants = AclTableGrant::load(&schemas, &mut connection).unwrap();
    assert_eq!(
        grants
            .iter()
            .map(|grant| (
                grant.table_name.as_str(),
                grant.privilege_type.as_str(),
                grant.grantee.as_str(),
            ))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            ("kept", "SELECT", "PUBLIC"),
            ("kept_view", "SELECT", "PUBLIC"),
            ("skipped", "SELECT", "PUBLIC"),
            ("skipped_seq", "USAGE", "PUBLIC"),
        ])
    );

    let grants = RoleColumnGrants::load_acl(&schemas, &mut connection).unwrap();
    assert_eq!(
        grants
            .iter()
            .map(|grant| (
                grant.table_name.as_str(),
                grant.column_name.as_str(),
                grant.privilege_type.as_str(),
                grant.grantee.as_str(),
            ))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            ("kept", "value", "UPDATE", "PUBLIC"),
            ("skipped", "id", "UPDATE", "PUBLIC"),
        ])
    );

    let rules: Vec<_> = PgRuleEntry::load_rows(&schemas, &mut connection)
        .unwrap()
        .into_iter()
        .map(|row| PgRuleEntry::new(row).unwrap())
        .collect();
    assert_eq!(
        rules
            .iter()
            .map(|rule| (rule.relation_schema(), rule.relation_name(), rule.name()))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            ("app", "kept", "kept_no_delete"),
            ("app", "skipped", "skipped_no_delete"),
        ])
    );
}
