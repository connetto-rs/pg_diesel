//! Differential test of [`PgDieselDatabase`] against `ParserDB`.
//!
//! Every DDL document under `tests/differential/corpus` is applied to a fresh
//! `PostgreSQL` database, read back through [`PgDieselDatabaseBuilder`], and
//! parsed by `ParserDB`. The two models are described through the
//! `sql-traits` traits and compared fact by fact. A difference fails the test
//! unless [`expected::EXPECTED`] names it. The live model's answers on role
//! membership, table privileges and row-level security are also checked
//! against what the same server answers through `pg_has_role`,
//! `has_table_privilege`, `has_column_privilege` and the query its rewriter
//! produces.

mod test_utils;

#[path = "differential/compare.rs"]
mod compare;
#[path = "differential/deparse.rs"]
mod deparse;
#[path = "differential/expected.rs"]
mod expected;
#[path = "differential/node_tree.rs"]
mod node_tree;
#[path = "differential/operators.rs"]
mod operators;
#[path = "differential/server.rs"]
mod server;
#[path = "differential/table_filter.rs"]
mod table_filter;

use std::{
    collections::BTreeSet,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

use diesel::{
    ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl, TextExpressionMethods,
    connection::SimpleConnection,
    dsl::sql,
    sql_types::{Oid, Text},
};
use pg_diesel::{
    PgDieselDatabase,
    database::PgDieselDatabaseBuilder,
    schema::pg_catalog::{
        pg_authid::pg_authid, pg_class::pg_class, pg_depend::pg_depend, pg_namespace::pg_namespace,
        pg_proc::pg_proc, pg_range::pg_range, pg_roles::pg_roles, pg_type::pg_type,
    },
    sql_traits::{structs::ParseOptions, traits::DatabaseLike},
    sqlparser::dialect::PostgreSqlDialect,
};
use test_utils::{establish_connection, reference_docker};

use crate::{
    compare::{AlreadyStored, Facts, Scope, contradictions, describe, differences, function_key},
    deparse::ThroughPostgres,
    server::{
        RowSecurityProbeError, server_function_privileges, server_role_privileges,
        server_row_security, server_schema_usage, server_table_privileges,
    },
};

/// The maintenance database documents are created from.
const MAINTENANCE: &str = "differential";
/// The superuser the container creates, left out of role comparisons.
const SUPERUSER: &str = "user";

/// Returns every corpus document, sorted by path.
fn corpus() -> Vec<PathBuf> {
    fn walk(directory: &Path, documents: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(directory).expect("the corpus directory is readable") {
            let path = entry.expect("the corpus entry is readable").path();
            if path.is_dir() {
                walk(&path, documents);
            } else if path.extension().is_some_and(|extension| extension == "sql") {
                documents.push(path);
            }
        }
    }
    let mut documents = Vec::new();
    walk(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/differential/corpus"),
        &mut documents,
    );
    documents.sort();
    documents
}

/// Returns `name` as a quoted identifier.
fn quoted(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// Drops every role a document created, so the next document starts from the
/// cluster's own roles.
fn drop_document_roles(maintenance: &mut PgConnection) {
    let roles: Vec<Option<String>> = pg_roles::table
        .filter(pg_roles::rolname.not_like("pg\\_%"))
        .filter(pg_roles::rolname.ne(SUPERUSER))
        .select(pg_roles::rolname)
        .load(maintenance)
        .expect("the roles are listed");
    for role in roles.into_iter().flatten() {
        maintenance
            .batch_execute(&format!("DROP ROLE {};", quoted(&role)))
            .unwrap_or_else(|error| panic!("the role {role} drops: {error}"));
    }
}

/// What one document produced.
enum Outcome {
    /// Both models were described, with these unexplained differences, each
    /// printed as one line.
    Compared(Vec<String>),
    /// `ParserDB` refused a document that names the sqlparser gap it waits
    /// on.
    ParserGap {
        /// The gap the document names.
        gap: String,
        /// The parser's complaint.
        error: String,
    },
    /// The document could not be compared, for this reason.
    Failed(String),
}

/// Returns the `(schema, name)` of every function, relation and type an
/// extension script created, which `pg_depend` ties to the extension with
/// deptype `e`.
fn extension_members(connection: &mut PgConnection) -> BTreeSet<(String, String)> {
    let functions: Vec<(String, String)> = pg_depend::table
        .inner_join(pg_proc::table.on(pg_proc::oid.eq(pg_depend::objid)))
        .inner_join(pg_namespace::table.on(pg_namespace::oid.eq(pg_proc::pronamespace)))
        .filter(pg_depend::deptype.eq("e"))
        .filter(pg_depend::classid.eq(sql::<Oid>("'pg_proc'::regclass")))
        .select((pg_namespace::nspname, pg_proc::proname))
        .load(connection)
        .expect("the extension functions are listed");
    let relations: Vec<(String, String)> = pg_depend::table
        .inner_join(pg_class::table.on(pg_class::oid.eq(pg_depend::objid)))
        .inner_join(pg_namespace::table.on(pg_namespace::oid.eq(pg_class::relnamespace)))
        .filter(pg_depend::deptype.eq("e"))
        .filter(pg_depend::classid.eq(sql::<Oid>("'pg_class'::regclass")))
        .select((pg_namespace::nspname, pg_class::relname))
        .load(connection)
        .expect("the extension relations are listed");
    let types: Vec<(String, String)> = pg_depend::table
        .inner_join(pg_type::table.on(pg_type::oid.eq(pg_depend::objid)))
        .inner_join(pg_namespace::table.on(pg_namespace::oid.eq(pg_type::typnamespace)))
        .filter(pg_depend::deptype.eq("e"))
        .filter(pg_depend::classid.eq(sql::<Oid>("'pg_type'::regclass")))
        .select((pg_namespace::nspname, pg_type::typname))
        .load(connection)
        .expect("the extension types are listed");
    functions
        .into_iter()
        .chain(relations)
        .chain(types)
        .collect()
}

/// Why the server could not answer what it knows about a document.
#[derive(Debug, thiserror::Error)]
enum ServerFactsError {
    /// Asking `pg_has_role` failed.
    #[error("pg_has_role failed: {0}")]
    RolePrivileges(#[source] diesel::result::Error),
    /// Asking `has_table_privilege` failed.
    #[error("has_table_privilege failed: {0}")]
    TablePrivileges(#[source] diesel::result::Error),
    /// Asking `has_function_privilege` failed.
    #[error("has_function_privilege failed: {0}")]
    FunctionPrivileges(#[source] diesel::result::Error),
    /// Asking `has_schema_privilege` failed.
    #[error("has_schema_privilege failed: {0}")]
    SchemaUsage(#[source] diesel::result::Error),
    /// Probing row-level security failed.
    #[error("probing row-level security failed: {0}")]
    RowSecurity(#[source] RowSecurityProbeError),
}

/// Returns what the server itself answers about the roles, tables, functions
/// and schemas of `scope` in the database `database`, a server of major
/// version `server_major` published on the host port `port`, in the facts
/// the describer names.
///
/// # Errors
///
/// Returns a [`ServerFactsError`] naming the server read that failed.
fn server_facts(
    connection: &mut PgConnection,
    port: u16,
    database: &str,
    live: &PgDieselDatabase,
    scope: &Scope,
    server_major: u32,
) -> Result<Facts, ServerFactsError> {
    let roles = scope.roles(live);
    let tables = scope.tables(live);
    let functions: Vec<(String, u32)> = scope
        .functions(live)
        .into_iter()
        .map(|function| (function_key(scope, live, function), function.oid()))
        .collect();
    let mut facts = server_role_privileges(connection, &roles, server_major)
        .map_err(ServerFactsError::RolePrivileges)?;
    facts.extend(
        server_table_privileges(connection, &tables, &roles, server_major)
            .map_err(ServerFactsError::TablePrivileges)?,
    );
    facts.extend(
        server_function_privileges(connection, &functions, &roles)
            .map_err(ServerFactsError::FunctionPrivileges)?,
    );
    facts.extend(
        server_schema_usage(connection, &scope.schemas, &roles)
            .map_err(ServerFactsError::SchemaUsage)?,
    );
    facts.extend(
        server_row_security(connection, port, database, &tables, &roles, server_major)
            .map_err(ServerFactsError::RowSecurity)?,
    );
    Ok(facts)
}

/// Returns the bootstrap superuser of the database `live` was read from,
/// with the keys of its range and multirange constructor functions, which
/// PostgreSQL names after the type they return.
fn expected_context(
    connection: &mut PgConnection,
    live: &PgDieselDatabase,
    scope: &Scope,
) -> expected::Context {
    let bootstrap_superuser: String = pg_authid::table
        .filter(pg_authid::oid.eq(10))
        .select(pg_authid::rolname)
        .first(connection)
        .expect("the bootstrap superuser is listed");
    let range_types: Vec<u32> = pg_range::table
        .select((pg_range::rngtypid, pg_range::rngmultitypid))
        .load::<(u32, u32)>(connection)
        .expect("the range types are listed")
        .into_iter()
        .flat_map(|(range, multirange)| [range, multirange])
        .collect();
    let constructors: BTreeSet<u32> = pg_proc::table
        .inner_join(pg_type::table.on(pg_type::oid.eq(pg_proc::prorettype)))
        .filter(pg_proc::proname.eq(pg_type::typname))
        .filter(pg_proc::pronamespace.eq(pg_type::typnamespace))
        .filter(pg_type::oid.eq_any(range_types))
        .select(pg_proc::oid)
        .load::<u32>(connection)
        .expect("the range constructors are listed")
        .into_iter()
        .collect();
    expected::Context {
        bootstrap_superuser,
        range_constructors: live
            .functions()
            .filter(|function| constructors.contains(&function.oid()))
            .map(|function| function_key(scope, live, function))
            .collect(),
    }
}

/// Applies `ddl` to the database `database` on a server of major version
/// `server_major` published on the host port `port`, then describes and
/// compares the two models.
fn compare_document(port: u16, database: &str, ddl: &str, server_major: u32) -> Outcome {
    let mut connection: PgConnection =
        establish_connection(port, database).expect("the document database accepts connections");
    if let Err(error) = connection.batch_execute(ddl) {
        return Outcome::Failed(format!("PostgreSQL refused the document: {error}"));
    }
    // The document runs as the container's superuser, which owns what it
    // creates.
    let source = match ParseOptions::default()
        .with_session_role(SUPERUSER)
        .parse::<PostgreSqlDialect>(ddl)
    {
        Ok(source) => {
            if let Some(gap) = parser_gap(ddl) {
                eprintln!(
                    "{database}: ParserDB parses the document now, so its parser gap line ({gap}) can go"
                );
            }
            source
        }
        Err(error) => {
            return match parser_gap(ddl) {
                Some(gap) => Outcome::ParserGap {
                    gap: gap.to_owned(),
                    error: error.to_string(),
                },
                None => Outcome::Failed(format!("ParserDB refused the document: {error}")),
            };
        }
    };
    let default_schema: String = diesel::select(sql::<Text>("current_schema()"))
        .get_result(&mut connection)
        .expect("the session has a current schema");
    let schemas: Vec<String> = pg_namespace::table
        .filter(pg_namespace::nspname.not_like("pg\\_%"))
        .filter(pg_namespace::nspname.ne("information_schema"))
        .select(pg_namespace::nspname)
        .load(&mut connection)
        .expect("the schemas are listed");
    let scope = Scope {
        default_schema,
        schemas,
        ignored_roles: vec![SUPERUSER.to_owned()],
        extension_members: extension_members(&mut connection),
    };
    let live: PgDieselDatabase = match PgDieselDatabaseBuilder::default()
        .connection(&mut connection)
        .catalog(database)
        .schemas(&scope.schemas)
        .try_into()
    {
        Ok(live) => live,
        Err(error) => {
            return Outcome::Failed(format!("pg_diesel could not read the database: {error}"));
        }
    };
    let live_facts = match describe(&live, &scope, &mut AlreadyStored) {
        Ok(facts) => facts,
        Err(error) => return Outcome::Failed(format!("describing the live model failed: {error}")),
    };
    let source_facts = match describe(&source, &scope, &mut ThroughPostgres::new(&mut connection)) {
        Ok(facts) => facts,
        Err(error) => {
            return Outcome::Failed(format!("describing the source model failed: {error}"));
        }
    };
    let server_facts =
        match server_facts(&mut connection, port, database, &live, &scope, server_major) {
            Ok(facts) => facts,
            Err(error) => return Outcome::Failed(error.to_string()),
        };
    let context = expected_context(&mut connection, &live, &scope);
    let mut found: Vec<String> = differences(&source_facts, &live_facts)
        .into_iter()
        .filter(
            |difference| match expected::accepted(difference, &context) {
                Some(accepted) => {
                    eprintln!(
                        "{database}: {difference} accepted as {}, {}",
                        accepted.name, accepted.reason
                    );
                    false
                }
                None => true,
            },
        )
        .map(|difference| difference.to_string())
        .collect();
    found.extend(
        contradictions(&server_facts, &live_facts)
            .iter()
            .map(|difference| difference.labelled("PostgreSQL", "live").to_string()),
    );
    Outcome::Compared(found)
}

/// Returns the oldest `PostgreSQL` major version a document's DDL needs, from
/// a `-- minimum server: N` line among its leading comments.
fn minimum_server(ddl: &str) -> Option<u32> {
    leading_comment(ddl, "-- minimum server:").map(|version| {
        version
            .parse()
            .expect("a minimum server line names a major version")
    })
}

/// Returns the sqlparser gap a document waits on, from a
/// `-- parser gap: <what sqlparser lacks and where it is fixed>` line among
/// its leading comments.
///
/// A document `ParserDB` refuses is skipped when it names one, and compared
/// as any other once sqlparser parses it.
fn parser_gap(ddl: &str) -> Option<&str> {
    leading_comment(ddl, "-- parser gap:")
}

/// Returns the rest of the leading comment line starting with `prefix`,
/// trimmed.
fn leading_comment<'a>(ddl: &'a str, prefix: &str) -> Option<&'a str> {
    ddl.lines()
        .take_while(|line| line.starts_with("--"))
        .find_map(|line| line.strip_prefix(prefix))
        .map(str::trim)
}

#[tokio::test]
async fn parser_and_catalog_agree_on_every_corpus_document() {
    let docker = reference_docker(MAINTENANCE)
        .await
        .expect("the container starts");
    let mut maintenance: PgConnection = establish_connection(docker.port, MAINTENANCE)
        .expect("the maintenance database accepts connections");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/differential/corpus");
    let server_major = diesel::select(sql::<Text>("current_setting('server_version_num')"))
        .get_result::<String>(&mut maintenance)
        .expect("the server reports its version")
        .parse::<u32>()
        .expect("server_version_num is a number")
        / 10_000;
    let mut report = String::new();
    let documents = corpus();
    assert!(!documents.is_empty(), "the corpus holds documents");
    for (number, document) in documents.iter().enumerate() {
        let ddl = fs::read_to_string(document).expect("the document is readable");
        let name = document.strip_prefix(&root).unwrap_or(document).display();
        if let Some(minimum) = minimum_server(&ddl)
            && server_major < minimum
        {
            eprintln!(
                "{name}: skipped, it needs PostgreSQL {minimum} and the server is {server_major}"
            );
            continue;
        }
        let database = format!("document_{number}");
        maintenance
            .batch_execute(&format!("CREATE DATABASE {};", quoted(&database)))
            .expect("the document database is created");
        let outcome = compare_document(docker.port, &database, &ddl, server_major);
        maintenance
            .batch_execute(&format!(
                "DROP DATABASE {} WITH (FORCE);",
                quoted(&database)
            ))
            .expect("the document database drops");
        drop_document_roles(&mut maintenance);
        match outcome {
            Outcome::Compared(found) if found.is_empty() => {}
            Outcome::Compared(found) => {
                writeln!(report, "{name}:").expect("writing to a string succeeds");
                for difference in found {
                    writeln!(report, "  {difference}").expect("writing to a string succeeds");
                }
            }
            Outcome::ParserGap { gap, error } => {
                eprintln!("{name}: skipped, sqlparser cannot parse it yet ({gap}): {error}");
            }
            Outcome::Failed(reason) => {
                writeln!(report, "{name}: {reason}").expect("writing to a string succeeds");
            }
        }
    }
    assert!(report.is_empty(), "the models disagree:\n{report}");
}
