//! End-to-end test of [`PgDieselDatabase`] built from a live `PostgreSQL`.

mod test_utils;

use std::collections::BTreeSet;

use diesel::{PgConnection, connection::SimpleConnection};
use pg_diesel::sqlparser::ast::{
    ArgMode, FunctionCalledOnNull, FunctionSecurity, ReferentialAction, TriggerEvent,
    TriggerObject, TriggerObjectKind, TriggerPeriod,
};
use pg_diesel::{
    PgDieselDatabase,
    database::PgDieselDatabaseBuilder,
    model_metadata::PgTable,
    sql_traits::{
        structs::FunctionKind,
        traits::{
            CheckConstraintLike, ColumnCollation, ColumnGrantLike, ColumnLike, DatabaseLike,
            ForeignKeyLike, FunctionLike, GrantLike, IndexLike, PartitionStrategy, PolicyLike,
            Relation, RoleLike, SchemaLike, TableGrantLike, TableLike, TriggerLike, ViewLike,
            grant::GrantTarget,
        },
    },
};
use test_utils::{establish_connection, reference_docker};

/// The schema the test introspects, as raw SQL since the typed DSL has no DDL.
const FIXTURE: &str = r#"
CREATE ROLE app_owner;
CREATE SCHEMA app;

CREATE TABLE app.parents (
    id INTEGER PRIMARY KEY,
    name TEXT COLLATE "C" NOT NULL
);
COMMENT ON TABLE app.parents IS 'every parent';
COMMENT ON COLUMN app.parents.name IS 'the parent name';
ALTER TABLE app.parents OWNER TO app_owner;
ALTER TABLE app.parents ENABLE ROW LEVEL SECURITY;
CREATE POLICY parents_are_visible ON app.parents FOR SELECT USING (true);

CREATE TABLE app.children (
    id INTEGER PRIMARY KEY,
    parent_id INTEGER NOT NULL REFERENCES app.parents(id) ON DELETE CASCADE,
    CONSTRAINT children_id_is_positive CHECK (id > 0)
);
CREATE INDEX children_by_parent ON app.children (parent_id);
GRANT SELECT ON app.children TO app_owner;
GRANT UPDATE (parent_id) ON app.children TO app_owner;

CREATE FUNCTION app.doubled(value INTEGER) RETURNS INTEGER
    LANGUAGE sql STRICT SECURITY DEFINER
    SET search_path = app
    AS 'SELECT value * 2';
ALTER FUNCTION app.doubled(INTEGER) OWNER TO app_owner;
CREATE POLICY parents_are_doubled ON app.parents FOR UPDATE TO app_owner
    USING (app.doubled(id) > 0)
    WITH CHECK (app.doubled(id) > 0);

CREATE FUNCTION app.keep_row() RETURNS trigger
    LANGUAGE plpgsql STRICT
    AS $$ BEGIN RETURN NEW; END; $$;
CREATE TRIGGER children_are_kept BEFORE INSERT ON app.children
    FOR EACH ROW EXECUTE FUNCTION app.keep_row();

CREATE TABLE app.notes (id INTEGER PRIMARY KEY);
CREATE TABLE app.secret_notes (classification TEXT) INHERITS (app.notes);

CREATE TABLE app.events (id INTEGER, region TEXT) PARTITION BY LIST (region);
CREATE TABLE app.events_north PARTITION OF app.events FOR VALUES IN ('north');

CREATE VIEW app.parent_names AS SELECT id, name FROM app.parents;
CREATE MATERIALIZED VIEW app.parent_ids AS SELECT id FROM app.parents;
"#;

/// Returns the table recorded under `name`, failing the test when absent.
fn table<'db>(database: &'db PgDieselDatabase, name: &str) -> &'db PgTable {
    database
        .tables()
        .find(|table| table.name() == name)
        .unwrap_or_else(|| panic!("the database holds the table {name}"))
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_database_build() {
    let database_name = "test_database_build";
    let port = 35436;
    let _docker = reference_docker(port, database_name)
        .await
        .expect("Failed to start docker");
    let mut conn: PgConnection = establish_connection(port, database_name)
        .expect("Failed to establish connection to database");

    conn.batch_execute(FIXTURE).expect("the fixture applies");

    let database: PgDieselDatabase = PgDieselDatabaseBuilder::default()
        .connection(&mut conn)
        .catalog(database_name)
        .schema("app")
        .try_into()
        .expect("the app schema is introspected");

    // Only base tables land in the table collection.
    let table_names: BTreeSet<&str> = database.tables().map(PgTable::name).collect();
    assert_eq!(
        table_names,
        BTreeSet::from([
            "children",
            "events",
            "events_north",
            "notes",
            "parents",
            "secret_notes"
        ]),
        "views must not be recorded as tables"
    );

    // Views and materialized views land in their own collections.
    let views: Vec<&str> = database.views().map(ViewLike::view_name).collect();
    assert_eq!(views, vec!["parent_names"]);
    let view = database.views().next().expect("the plain view is recorded");
    assert!(!view.is_materialized());
    assert!(
        view.definition().to_string().contains("parents"),
        "the definition names the relation it selects from: {}",
        view.definition()
    );
    let materialized: Vec<&str> = database
        .materialized_views()
        .map(ViewLike::view_name)
        .collect();
    assert_eq!(materialized, vec!["parent_ids"]);
    assert!(
        database
            .materialized_views()
            .next()
            .expect("the snapshot is recorded")
            .is_materialized()
    );

    // The schema itself is recorded, so unqualified names resolve.
    assert!(DatabaseLike::schemas(&database).any(|schema| SchemaLike::name(schema) == "app"));

    // Partitioning.
    assert_eq!(
        table(&database, "events").partition_strategy(),
        Some(PartitionStrategy::List)
    );
    assert_eq!(table(&database, "parents").partition_strategy(), None);
    let partition = table(&database, "events_north");
    assert_eq!(
        partition
            .partition_root(&database)
            .expect("the partition is in the database")
            .map(PgTable::name),
        Some("events")
    );
    assert_eq!(
        partition
            .inherits_from(&database)
            .expect("in database")
            .count(),
        0
    );

    // Inheritance.
    let child = table(&database, "secret_notes");
    let parents: Vec<&str> = child
        .inherits_from(&database)
        .expect("in database")
        .map(PgTable::name)
        .collect();
    assert_eq!(parents, vec!["notes"]);
    let local: Vec<&str> = child
        .local_columns(&database)
        .expect("in database")
        .map(ColumnLike::column_name)
        .collect();
    assert_eq!(
        local,
        vec!["classification"],
        "an inherited column is not local"
    );
    assert!(
        child.columns(&database).expect("in database").count() > local.len(),
        "the inherited column is still a column"
    );

    // Ownership and row level security.
    let owned = table(&database, "parents");
    assert_eq!(
        owned.owner(&database).expect("in database"),
        Some("app_owner")
    );
    assert!(
        owned
            .has_row_level_security(&database)
            .expect("in database")
    );
    assert!(
        !owned
            .has_forced_row_level_security(&database)
            .expect("in database")
    );

    // Comments, on the table and on one of its columns.
    assert_eq!(
        owned.table_doc(&database).expect("in database"),
        Some("every parent")
    );
    assert_eq!(
        table(&database, "children")
            .table_doc(&database)
            .expect("in database"),
        None
    );

    // Collation.
    let name_column = owned
        .columns(&database)
        .expect("in database")
        .find(|column| column.column_name() == "name")
        .expect("the collated column is recorded");
    assert_eq!(
        name_column.column_doc(&database).expect("in database"),
        Some("the parent name")
    );
    match name_column.collation(&database).expect("in database") {
        ColumnCollation::Named(collation) => {
            assert_eq!(collation.name().name(), "C");
            assert_eq!(collation.postgres_deterministic(), Some(true));
        }
        other => panic!("a COLLATE clause is reported as a named collation, got {other:?}"),
    }
    let id_column = owned
        .columns(&database)
        .expect("in database")
        .find(|column| column.column_name() == "id")
        .expect("the primary key column is recorded");
    assert_eq!(
        id_column.collation(&database).expect("in database"),
        ColumnCollation::DatabaseDefault
    );

    // Foreign keys name their target as the catalog stores it.
    let foreign_key = table(&database, "children")
        .foreign_keys(&database)
        .expect("in database")
        .next()
        .expect("the reference is recorded");
    assert_eq!(foreign_key.referenced_table_name().name(), "parents");
    assert_eq!(foreign_key.referenced_table_name().schema(), Some("app"));
    assert_eq!(
        foreign_key
            .referenced_table(&database)
            .expect("the target is in the database")
            .name(),
        "parents"
    );
    assert_eq!(foreign_key.on_delete(&database), ReferentialAction::Cascade);

    // Check constraints resolve their columns.
    let check = table(&database, "children")
        .check_constraints(&database)
        .expect("in database")
        .find(|constraint| constraint.name() == "children_id_is_positive")
        .expect("the check constraint is recorded");
    let checked: Vec<&str> = CheckConstraintLike::columns(check, &database)
        .expect("in database")
        .map(ColumnLike::column_name)
        .collect();
    assert_eq!(checked, vec!["id"]);

    // A unique index is not an index and the other way around.
    let indexed = table(&database, "children");
    let indices: Vec<&str> = indexed
        .indices(&database)
        .expect("in database")
        .filter_map(IndexLike::name)
        .collect();
    assert_eq!(indices, vec!["children_by_parent"]);
    let unique: Vec<&str> = indexed
        .unique_indices(&database)
        .expect("in database")
        .filter_map(IndexLike::name)
        .collect();
    assert_eq!(unique, vec!["children_pkey"]);
    let index = indexed
        .indices(&database)
        .expect("in database")
        .next()
        .expect("the index is recorded");
    assert_eq!(IndexLike::table(index, &database).name(), "children");
    let indexed_columns: Vec<&str> = index
        .columns(&database)
        .expect("in database")
        .map(ColumnLike::column_name)
        .collect();
    assert_eq!(indexed_columns, vec!["parent_id"]);

    // Policies name the table they were declared on.
    let policy = database
        .policies()
        .find(|policy| PolicyLike::name(policy) == "parents_are_visible")
        .expect("the policy is recorded");
    assert_eq!(policy.target_table_name().name(), "parents");
    assert_eq!(policy.target_table_name().schema(), Some("app"));
    assert!(
        policy.applies_to_public(),
        "a policy without a TO clause applies to PUBLIC"
    );

    // A policy resolves the functions its expressions call, and the role it
    // names holds it back.
    let calling = database
        .policies()
        .find(|policy| PolicyLike::name(policy) == "parents_are_doubled")
        .expect("the policy is recorded");
    assert!(!calling.applies_to_public());
    assert_eq!(
        calling.using_expression(&database).map(ToString::to_string),
        Some("(app.doubled(id) > 0)".to_string())
    );
    assert_eq!(
        calling.check_expression(&database).map(ToString::to_string),
        Some("(app.doubled(id) > 0)".to_string())
    );
    let using: Vec<&str> = calling
        .using_functions(&database)
        .expect("in database")
        .map(FunctionLike::name)
        .collect();
    assert_eq!(using, vec!["doubled"]);
    let checking: Vec<&str> = calling
        .check_functions(&database)
        .expect("in database")
        .map(FunctionLike::name)
        .collect();
    assert_eq!(checking, vec!["doubled"]);

    let role = database
        .roles()
        .find(|role| RoleLike::name(role) == "app_owner")
        .expect("the role is recorded");
    let role_policies: Vec<&str> = role.policies(&database).map(PolicyLike::name).collect();
    assert_eq!(role_policies, vec!["parents_are_doubled"]);

    // Grants name their target relation and resolve to it.
    let grant = database
        .table_grants()
        .find(|grant| {
            grant.grantee.as_deref() == Some("app_owner")
                && grant.table_name.as_deref() == Some("children")
        })
        .expect("the grant is recorded");
    let GrantTarget::Tables(names) = grant.target() else {
        panic!("a relation grant names its table");
    };
    let targets: Vec<String> = names.map(|name| name.to_string()).collect();
    // A catalog name is reported quoted, the spelling that resolves back.
    assert_eq!(targets, vec![r#""app"."children""#.to_string()]);
    assert_eq!(grant.relations(&database).count(), 1);
    assert!(!grant.applies_to_public());

    // A column grant names the relation and resolves to the granted column.
    let column_grant = database
        .column_grants()
        .find(|grant| {
            grant.grantee.as_deref() == Some("app_owner")
                && grant.column_name.as_deref() == Some("parent_id")
        })
        .expect("the column grant is recorded");
    let granted_table = column_grant
        .table(&database)
        .expect("the granted relation is in the database");
    assert_eq!(granted_table.name(), "children");
    let granted: Vec<&str> = column_grant
        .columns(granted_table, &database)
        .expect("in database")
        .map(ColumnLike::column_name)
        .collect();
    assert_eq!(granted, vec!["parent_id"]);
    assert!(matches!(
        column_grant.relation(&database),
        Some(Relation::Table(_))
    ));

    // Functions carry the facts `pg_proc` records only as OIDs or raw text.
    let function = database
        .functions()
        .find(|function| function.name() == "doubled")
        .expect("the function is recorded");
    assert_eq!(function.target_name().schema(), Some("app"));
    assert_eq!(function.language(), Some("sql"));
    assert_eq!(
        function.owner(&database).expect("in database"),
        Some("app_owner")
    );
    assert_eq!(function.security_mode(), FunctionSecurity::Definer);
    assert_eq!(function.null_input_behavior(), FunctionCalledOnNull::Strict);
    assert!(!function.returns_set());
    assert_eq!(
        function.kind(&database).expect("in database"),
        FunctionKind::Normal
    );
    let modes: Vec<ArgMode> = function.argument_modes().collect();
    assert_eq!(modes, vec![ArgMode::In]);
    assert_eq!(function.argument_default_count(), 0);
    assert!(
        function
            .is_executable_by_public(&database)
            .expect("in database"),
        "no `REVOKE` took `EXECUTE` from `PUBLIC`"
    );
    let app_owner = database.role("app_owner").expect("the role is recorded");
    assert!(
        function
            .can_execute(app_owner, &database)
            .expect("in database")
    );
    assert!(
        function
            .can_grant_execute(app_owner, &database)
            .expect("in database"),
        "the owner may grant whatever its own entry says"
    );
    // A body written as a string literal is parsed only when the function runs.
    assert!(
        !function.body_is_bound(),
        "a string-literal SQL body leaves `prosqlbody` empty"
    );
    assert_eq!(
        function.search_path(&database).expect("in database"),
        Some(vec![("app".into(), false)])
    );
    let argument_names: Vec<String> = function
        .argument_names(&database)
        .map(|name| name.expect("the argument is named").name().to_string())
        .collect();
    assert_eq!(argument_names, vec!["value".to_string()]);
    let argument_types: Vec<String> = function
        .argument_type_names(&database)
        .map(std::borrow::Cow::into_owned)
        .collect();
    assert_eq!(argument_types, vec!["integer".to_string()]);
    assert_eq!(
        function.return_type_name(&database).as_deref(),
        Some("integer")
    );
    assert!(
        function
            .body()
            .is_some_and(|body| body.contains("value * 2"))
    );
    let configuration: Vec<String> = function
        .configuration_parameters()
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(configuration, vec!["SET search_path = app".to_string()]);

    // Triggers name their table and resolve the function they execute.
    let trigger = database
        .triggers()
        .find(|trigger| TriggerLike::name(trigger) == "children_are_kept")
        .expect("the trigger is recorded");
    assert_eq!(trigger.target_table_name().name(), "children");
    assert_eq!(trigger.target_table_name().schema(), Some("app"));
    assert_eq!(
        TriggerLike::table(trigger, &database)
            .expect("in database")
            .name(),
        "children"
    );
    assert_eq!(trigger.timing(), Some(TriggerPeriod::Before));
    assert_eq!(
        trigger.orientation(),
        Some(TriggerObjectKind::ForEach(TriggerObject::Row))
    );
    assert_eq!(trigger.events(), &[TriggerEvent::Insert]);
    assert_eq!(trigger.function_name(), Some("keep_row"));
    assert_eq!(
        trigger.function(&database).map(FunctionLike::name),
        Some("keep_row")
    );
}
