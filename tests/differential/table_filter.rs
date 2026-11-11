//! Table-scoped catalog loading.

use pg_diesel::{
    database::AclTableGrant,
    model_metadata::PgRuleEntry,
    models::{RoleColumnGrants, Table},
};

use pg_diesel::sql_traits::{
    errors::LookupError,
    traits::{ForeignKeyLike, TableLike},
};

use super::{
    AlreadyStored, BTreeSet, DatabaseLike, PgConnection, PgDieselDatabase, PgDieselDatabaseBuilder,
    SUPERUSER, Scope, SimpleConnection, compare::describe_table, establish_connection,
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

#[tokio::test]
async fn selected_tables_preserve_local_facts() {
    let docker = reference_docker("table_filter").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "table_filter").unwrap();
    connection
        .batch_execute(include_str!("corpus/relations/table_filter.sql"))
        .unwrap();
    connection
        .batch_execute("GRANT USAGE ON SEQUENCE app.skipped_seq TO PUBLIC")
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
    let whole_table = whole
        .tables()
        .find(|table| table.name() == "kept" && table.schema() == "app")
        .unwrap();
    let selected_table = selected.tables().next().unwrap();
    assert_eq!(
        describe_table(&selected, selected_table, &scope, &mut AlreadyStored).unwrap(),
        describe_table(&whole, whole_table, &scope, &mut AlreadyStored).unwrap()
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
    connection
        .batch_execute("GRANT USAGE ON SEQUENCE app.skipped_seq TO PUBLIC")
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
