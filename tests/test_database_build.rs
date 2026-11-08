//! End-to-end test of [`PgDieselDatabase`] built from a live `PostgreSQL`.

mod test_utils;

use std::collections::BTreeSet;

use diesel::{PgConnection, connection::SimpleConnection};
use pg_diesel::dialect::PostgresDialect;
use pg_diesel::sqlparser::{
    ast::{
        ArgMode, Expr, Function, FunctionCalledOnNull, FunctionSecurity, ReferentialAction,
        SelectItem, SetExpr, Statement, TriggerEvent, TriggerObject, TriggerObjectKind,
        TriggerPeriod,
    },
    dialect::PostgreSqlDialect,
    parser::Parser,
};
use pg_diesel::{
    PgDieselDatabase,
    database::PgDieselDatabaseBuilder,
    model_metadata::PgTable,
    sql_traits::{
        errors::LookupError,
        structs::{FunctionKind, PrivilegeHolder, SequenceType},
        traits::{
            CheckConstraintLike, ColumnCollation, ColumnGrantLike, ColumnLike, DatabaseLike,
            DialectLike, ExtensionLike, ForeignKeyLike, FunctionLike, GrantLike, IndexLike,
            PartitionStrategy, PolicyLike, Relation, RoleLike, SchemaLike, SequenceLike,
            TableGrantLike, TableLike, TriggerLike, TypeMatch, UserDefinedTypeKind, ViewLike,
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

CREATE ROLE app_worker;

CREATE TYPE app.placeholder;

CREATE SEQUENCE app.counts;
CREATE SEQUENCE app.odd INCREMENT 2 MINVALUE 1 MAXVALUE 100 START 3 CACHE 5 CYCLE;

GRANT USAGE ON SCHEMA app TO app_owner;
GRANT SELECT ON app.events TO PUBLIC;
GRANT SELECT ON app.events TO app_owner WITH GRANT OPTION;
SET ROLE app_owner;
GRANT SELECT ON app.events TO app_worker;
RESET ROLE;
GRANT ALL ON SEQUENCE app.counts TO app_owner;
GRANT USAGE ON SEQUENCE app.odd TO app_worker;

CREATE FUNCTION app.guarded(value INTEGER) RETURNS INTEGER
    LANGUAGE plpgsql
    AS $$ BEGIN RETURN value; END; $$;
ALTER FUNCTION app.guarded(INTEGER) OWNER TO app_owner;
REVOKE EXECUTE ON FUNCTION app.guarded(INTEGER) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION app.guarded(INTEGER) TO app_owner;

CREATE FUNCTION app.summed(numbers VARIADIC INTEGER[]) RETURNS INTEGER
    LANGUAGE sql STRICT
    AS 'SELECT 0';

CREATE FUNCTION app.nullish(value INTEGER) RETURNS INTEGER
    LANGUAGE sql
    AS 'SELECT value';

CREATE FUNCTION app.bind_total(lo INTEGER, hi INTEGER) RETURNS INTEGER
    LANGUAGE sql IMMUTABLE
    RETURN lo + hi;

CREATE FUNCTION app.splits(value INTEGER) RETURNS TABLE (lo INTEGER, hi INTEGER)
    LANGUAGE sql
    AS 'SELECT value / 2, value % 2';
CREATE FUNCTION app.out_first(OUT x RECORD, value INTEGER)
    LANGUAGE sql
    AS 'SELECT NULL::record';

CREATE FUNCTION app.admitted(value INTEGER) RETURNS boolean
    LANGUAGE sql IMMUTABLE
    AS 'SELECT value % 7 <> 0';
ALTER TABLE app.children
    ADD CONSTRAINT children_parent_admitted CHECK (app.admitted(parent_id));

CREATE TRIGGER events_truncated BEFORE TRUNCATE ON app.events
    FOR EACH STATEMENT EXECUTE FUNCTION app.keep_row();
CREATE TRIGGER children_parent_kept BEFORE UPDATE OF parent_id ON app.children
    FOR EACH ROW EXECUTE FUNCTION app.keep_row();

CREATE TABLE app.docs (id INTEGER, body TEXT)
    WITH (autovacuum_enabled = true, toast.autovacuum_enabled = false);
ALTER TABLE app.docs SET (toast.vacuum_truncate);

CREATE EXTENSION file_fdw;
CREATE SERVER app_files FOREIGN DATA WRAPPER file_fdw;
CREATE FOREIGN TABLE app.remote_lines (id INTEGER, body TEXT) SERVER app_files
    OPTIONS (filename '/dev/null', format 'csv');
GRANT SELECT (id) ON app.remote_lines TO app_owner;
GRANT SELECT (id) ON app.parent_names TO app_owner;
GRANT SELECT (id) ON app.parent_ids TO app_owner;

ALTER FOREIGN TABLE app.remote_lines ALTER COLUMN body OPTIONS (force_not_null 'true');

ALTER TABLE app.notes ADD COLUMN flag BOOLEAN, ADD COLUMN stamp UUID;

ALTER SCHEMA app OWNER TO app_owner;
"#;

/// Returns the table recorded under `name`, failing the test when absent.
fn table<'db>(database: &'db PgDieselDatabase, name: &str) -> &'db PgTable {
    database
        .tables()
        .find(|table| table.name() == name)
        .unwrap_or_else(|| panic!("the database holds the table {name}"))
}

/// The function a call of `name` holds at the top of the expression,
/// unwrapping the parentheses and comparisons the clause stores, for a call
/// the test feeds to `functions_reached_by`.
fn first_call<'db>(expression: &'db Expr, name: &str) -> &'db Function {
    match expression {
        Expr::Function(function) => {
            if function
                .name
                .0
                .iter()
                .any(|part| part.as_ident().is_some_and(|ident| ident.value == name))
            {
                function
            } else {
                panic!("the expression calls another function: {function:?}")
            }
        }
        Expr::Nested(inner) => first_call(inner, name),
        Expr::BinaryOp { left, .. } => first_call(left, name),
        other => panic!("no call of `{name}` at the top, got {other:?}"),
    }
}

/// The single expression a one-projection `SELECT` spells, for a call the
/// test feeds to `functions_reached_by`.
fn select_expr(sql: &str) -> Expr {
    let statements = Parser::parse_sql(&PostgreSqlDialect {}, sql).expect("the query parses");
    let [Statement::Query(query)] = statements.as_slice() else {
        panic!("one statement, got two or more")
    };
    let SetExpr::Select(select) = &*query.body else {
        panic!("a plain `SELECT`")
    };
    let [SelectItem::UnnamedExpr(expr)] = select.projection.as_slice() else {
        panic!("one expression, got something else")
    };
    expr.clone()
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
            "docs",
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
        calling
            .using_expression(&database)
            .expect("in database")
            .map(ToString::to_string),
        Some("(app.doubled(id) > 0)".to_string())
    );
    assert_eq!(
        calling
            .check_expression(&database)
            .expect("in database")
            .map(ToString::to_string),
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
    assert_eq!(
        calling.written_using(&database).expect("in database"),
        None,
        "a catalog stores the typed constants, never the clause as written"
    );
    assert_eq!(calling.written_check(&database).expect("in database"), None);

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

    // A function names the type its call yields, and settles per-argument
    // null behaviour the way the catalog records it.
    assert_eq!(
        function.result_type_name(&database).as_deref(),
        Some("integer"),
        "the concrete return type names the call's result"
    );
    assert!(
        function.returns_null_on_null_argument(&database, 0),
        "a strict function settles its `IN` argument"
    );
    assert!(!function.returns_null_on_null_argument(&database, 1));
    let summed = database
        .functions()
        .find(|function| function.name() == "summed")
        .expect("the variadic function is recorded");
    assert!(
        !summed.returns_null_on_null_argument(&database, 0),
        "strictness does not settle a `VARIADIC` argument"
    );
    let nullish = database
        .functions()
        .find(|function| function.name() == "nullish")
        .expect("the non-strict function is recorded");
    assert!(
        !nullish.returns_null_on_null_argument(&database, 0),
        "the catalog records no per-argument behaviour for a non-strict user function"
    );
    let splits = database
        .functions()
        .find(|function| function.name() == "splits")
        .expect("the output-argument function is recorded");
    assert_eq!(
        splits.result_type_name(&database).as_deref(),
        Some("record"),
        "several output arguments leave the result the `record` type"
    );
    let out_first = database
        .functions()
        .find(|function| function.name() == "out_first")
        .expect("the output-first function is recorded");
    assert_eq!(
        out_first.result_type_name(&database).as_deref(),
        Some("record"),
        "a single output argument resolves to the type it declares"
    );

    // A bound body resolves the functions it calls, or reports the refusal.
    let bound = database
        .functions()
        .find(|function| function.name() == "bind_total")
        .expect("the bound function is recorded");
    assert_eq!(
        bound
            .bound_body_functions(&database)
            .expect("in database")
            .count(),
        0
    );

    // `EXECUTE` holders, `PUBLIC` first while it holds the privilege, then
    // the owner, then the grantees in the order first granted.
    let holders: Vec<PrivilegeHolder> = function
        .execute_holders(&database)
        .expect("in database")
        .collect();
    assert_eq!(
        holders,
        vec![
            PrivilegeHolder::Public,
            PrivilegeHolder::Role("app_owner".into()),
        ]
    );
    let guarded = database
        .functions()
        .find(|function| function.name() == "guarded")
        .expect("the guarded function is recorded");
    let holders: Vec<PrivilegeHolder> = guarded
        .execute_holders(&database)
        .expect("in database")
        .collect();
    assert_eq!(holders, vec![PrivilegeHolder::Role("app_owner".into())]);
    assert!(
        !guarded
            .is_executable_by_public(&database)
            .expect("in database"),
        "the `REVOKE` took `EXECUTE` from `PUBLIC`"
    );
    assert!(
        guarded
            .can_execute(app_owner, &database)
            .expect("in database")
    );

    // A check constraint resolves its table and the functions its expression
    // calls, and reports a call the expression does not hold.
    let admitted = table(&database, "children")
        .check_constraints(&database)
        .expect("in database")
        .find(|constraint| constraint.name() == "children_parent_admitted")
        .expect("the constraint is recorded");
    assert_eq!(
        CheckConstraintLike::table(admitted, &database)
            .expect("in database")
            .name(),
        "children"
    );
    let call = first_call(admitted.expression(), "admitted");
    let reached: Vec<&str> = CheckConstraintLike::functions_reached_by(admitted, call, &database)
        .expect("the call is a node of the expression")
        .map(FunctionLike::name)
        .collect();
    assert_eq!(reached, vec!["admitted"]);
    let absent = match select_expr("SELECT app.doubled(parent_id)") {
        Expr::Function(function) => function,
        other => panic!("the projection is the call itself, got {other:?}"),
    };
    assert!(
        CheckConstraintLike::functions_reached_by(admitted, &absent, &database)
            .is_err_and(|error| matches!(error, LookupError::CallNotInExpression { .. }))
    );

    // A policy resolves the functions its expressions call, the same way the
    // check constraint does.
    let using = calling
        .using_expression(&database)
        .expect("in database")
        .expect("the policy carries a `USING` expression");
    let call = first_call(using, "doubled");
    let reached: Vec<&str> = calling
        .functions_reached_by(call, &database)
        .expect("the call is a node of the expression")
        .map(FunctionLike::name)
        .collect();
    assert_eq!(reached, vec!["doubled"]);
    assert!(
        calling
            .functions_reached_by(&absent, &database)
            .is_err_and(|error| matches!(
                error,
                pg_diesel::sql_traits::errors::LookupError::CallNotInExpression { .. }
            ))
    );

    // A column grant on a relation that is no table resolves to that
    // relation, and the grant names the role that granted it.
    let foreign_grant = database
        .column_grants()
        .find(|grant| grant.table_name.as_deref() == Some("remote_lines"))
        .expect("the foreign table column grant is recorded");
    assert!(matches!(
        foreign_grant.relation(&database),
        Some(Relation::ForeignTable(_))
    ));
    assert_eq!(foreign_grant.granted_by(&database), Some("user"));
    let worker = database.role("app_worker").expect("the role is recorded");
    assert!(foreign_grant.applies_to_role(app_owner));
    assert!(!foreign_grant.applies_to_role(worker));
    let GrantTarget::Tables(names) = foreign_grant.target() else {
        panic!("a relation grant names its table");
    };
    let targets: Vec<String> = names.map(|name| name.to_string()).collect();
    assert_eq!(targets, vec![r#""app"."remote_lines""#.to_string()]);
    let body = database
        .foreign_tables()
        .find(|foreign| foreign.name() == "remote_lines")
        .expect("the foreign table is recorded")
        .columns(&database)
        .expect("in database")
        .find(|column| column.column_name() == "body")
        .expect("the column is recorded");
    let options: Vec<(String, String)> = ColumnLike::foreign_options(body)
        .map(|(key, value)| (key.into_owned(), value.to_string()))
        .collect();
    assert_eq!(
        options,
        vec![("force_not_null".to_string(), "true".to_string())]
    );
    let view_grant = database
        .column_grants()
        .find(|grant| grant.table_name.as_deref() == Some("parent_names"))
        .expect("the view column grant is recorded");
    assert!(matches!(
        view_grant.relation(&database),
        Some(Relation::View(_))
    ));
    let materialized_grant = database
        .column_grants()
        .find(|grant| grant.table_name.as_deref() == Some("parent_ids"))
        .expect("the materialized view column grant is recorded");
    assert!(matches!(
        materialized_grant.relation(&database),
        Some(Relation::MaterializedView(_))
    ));

    // The grant one role recorded names its grantor, and the sequence grants
    // reach the sequences they name.
    let delegated = database
        .table_grants()
        .find(|grant| {
            grant.grantee.as_deref() == Some("app_worker")
                && grant.table_name.as_deref() == Some("events")
        })
        .expect("the delegated grant is recorded");
    assert_eq!(delegated.granted_by(&database), Some("app_owner"));
    assert!(delegated.applies_to_role(worker));
    assert!(!delegated.applies_to_role(app_owner));
    let sequence_grant = database
        .table_grants()
        .find(|grant| {
            grant.grantee.as_deref() == Some("app_worker")
                && grant.table_name.as_deref() == Some("odd")
        })
        .expect("the sequence grant is recorded");
    let relations: Vec<Relation<'_, PgDieselDatabase>> =
        sequence_grant.relations(&database).collect();
    assert_eq!(relations.len(), 1);
    assert!(matches!(
        relations
            .into_iter()
            .next()
            .expect("the grant reaches a relation"),
        Relation::Sequence(_)
    ));
    let all = database
        .table_grants()
        .find(|grant| {
            grant.grantee.as_deref() == Some("app_owner")
                && grant.table_name.as_deref() == Some("counts")
        })
        .expect("the `GRANT ALL` on the sequence is recorded");
    assert!(
        all.is_all_privileges(),
        "`GRANT ALL` reaches the sequence as the spellings the server materializes"
    );

    // A sequence answers the parameters `pg_sequence` records, and the
    // extension the database holds spells the version it was created with.
    let odd = database
        .sequences()
        .find(|sequence| sequence.name() == "odd")
        .expect("the sequence is recorded");
    assert_eq!(SequenceLike::parameters(odd), odd.parameters());
    let parameters = odd.parameters();
    assert_eq!(parameters.data_type(), SequenceType::BigInt);
    assert_eq!(parameters.increment(), 2);
    assert_eq!(parameters.min_value(), 1);
    assert_eq!(parameters.max_value(), 100);
    assert_eq!(parameters.start(), 3);
    assert_eq!(parameters.cache(), 5);
    assert!(parameters.cycles());
    let counts = database
        .sequences()
        .find(|sequence| sequence.name() == "counts")
        .expect("the sequence is recorded");
    assert_eq!(counts.parameters().cache(), 1);
    assert!(!counts.parameters().cycles());
    let plpgsql = database
        .extensions()
        .find(|extension| extension.name() == "plpgsql")
        .expect("every database holds `plpgsql`");
    assert_eq!(ExtensionLike::version(plpgsql), Some("1.0"));
    assert_eq!(plpgsql.version(), "1.0");

    // The schema names the role that owns it.
    let app_schema = DatabaseLike::schemas(&database)
        .find(|schema| SchemaLike::name(schema) == "app")
        .expect("the schema is recorded");
    assert_eq!(SchemaLike::authorization(app_schema), Some("app_owner"));

    // The table's TOAST relation holds its own storage parameters, and a role
    // that holds no privilege at all names none the table could carry.
    let docs = table(&database, "docs");
    assert_eq!(
        docs.toast_storage_parameters(),
        &[
            ("autovacuum_enabled".to_string(), "false".to_string()),
            ("vacuum_truncate".to_string(), "true".to_string()),
        ]
    );
    let toast: Vec<(String, String)> = TableLike::toast_storage_parameters(docs)
        .map(|(name, value)| (name.into_owned(), value.into_owned()))
        .collect();
    assert_eq!(toast.len(), 2);
    let privileges = table(&database, "events")
        .possible_privileges(worker, &database)
        .expect("in database");
    assert!(
        privileges.select.on_table(),
        "`GRANT SELECT` reaches the table and so every column"
    );
    assert!(!privileges.delete);

    // The trigger the catalog stores for `TRUNCATE` answers the event the
    // mask records.
    let truncated = database
        .triggers()
        .find(|trigger| TriggerLike::name(trigger) == "events_truncated")
        .expect("the trigger is recorded");
    assert_eq!(truncated.events(), &[TriggerEvent::Truncate]);

    // The dialect predicates ask about the types the catalog resolved.
    let flag = table(&database, "notes")
        .columns(&database)
        .expect("in database")
        .find(|column| column.column_name() == "flag")
        .expect("the column is recorded");
    assert_eq!(PostgresDialect.is_bool(&database, flag), TypeMatch::No);
    assert_eq!(PostgresDialect.is_uuid(&database, flag), TypeMatch::No);
    let stamp = table(&database, "notes")
        .columns(&database)
        .expect("in database")
        .find(|column| column.column_name() == "stamp")
        .expect("the column is recorded");
    assert_eq!(PostgresDialect.is_uuid(&database, stamp), TypeMatch::No);
    assert!(
        flag.created_from_query(&database).is_none(),
        "`PostgreSQL` records a type for every column"
    );

    // The model methods read the table's rows from the live catalogs.
    let children = table(&database, "children");
    let indices = children
        .model()
        .indices(&mut conn)
        .expect("the indices load");
    let mut names: Vec<Option<&str>> = indices.iter().map(IndexLike::name).collect();
    names.sort();
    assert_eq!(
        names,
        vec![Some("children_by_parent"), Some("children_pkey")]
    );
    let unique = children
        .model()
        .unique_indices(&mut conn)
        .expect("the unique indices load");
    assert_eq!(unique.len(), 1);
    let triggers = children
        .model()
        .triggers(&mut conn)
        .expect("the triggers load");
    assert_eq!(triggers.len(), 2);
    let parent_id = children
        .columns(&database)
        .expect("in database")
        .find(|column| column.column_name() == "parent_id")
        .expect("the column is recorded");
    let checks = parent_id
        .model()
        .check_constraints(&mut conn)
        .expect("the constraints load");
    let names: Vec<&str> = checks
        .iter()
        .map(|check| check.constraint_name.as_str())
        .collect();
    assert_eq!(names, vec!["children_parent_admitted"]);

    // The bare `CREATE TYPE` holds a name the later definition takes over.
    let placeholder = database
        .user_defined_types()
        .find(|user_type| user_type.name() == "placeholder")
        .expect("the shell type is recorded");
    assert_eq!(placeholder.kind(), UserDefinedTypeKind::Shell);
}
