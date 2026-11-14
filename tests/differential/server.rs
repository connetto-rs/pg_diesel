//! Reads what `PostgreSQL` itself answers about roles, privileges and
//! row-level security, in the facts the describer names, so the live model
//! can be checked against the server it was read from.

use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex, PoisonError},
    time::Duration,
};

use diesel::{
    ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl,
    sql_types::{Bool, Oid, SmallInt, Text},
};
use pg_diesel::sql_traits::{
    structs::{PolicyClause, RowStatement},
    traits::RolePrivilege,
};

use crate::test_utils::docker_helpers::catalog::schema::pg_catalog::{
    pg_attribute::pg_attribute, pg_class::pg_class, pg_index::pg_index, pg_namespace::pg_namespace,
    pg_policy::pg_policy,
};

use crate::{
    compare::{
        COLUMN_PRIVILEGES, Facts, ROLE_PRIVILEGES, ROW_STATEMENTS, Value, WHOLE_TABLE_PRIVILEGES,
        column_privilege_value, execute_fact, execute_value, quoted, rewritten, rewritten_fact,
        rewritten_item, rewritten_term, role_privilege_fact, schema_usage_fact,
        table_privilege_fact,
    },
    node_tree::{self, NodeTreeError, Tree},
    test_utils::docker_helpers::database_url,
};

diesel::define_sql_function! {
    /// `pg_has_role(member, target, privilege)`.
    fn pg_has_role(member: Text, target: Text, privilege: Text) -> Bool;
}

diesel::define_sql_function! {
    /// `has_table_privilege(role, table, privilege)`.
    fn has_table_privilege(role: Text, table: Oid, privilege: Text) -> Bool;
}

diesel::define_sql_function! {
    /// `has_column_privilege(role, table, column, privilege)`.
    fn has_column_privilege(role: Text, table: Oid, column: SmallInt, privilege: Text) -> Bool;
}

diesel::define_sql_function! {
    /// `has_function_privilege(role, function, privilege)`.
    fn has_function_privilege(role: Text, function: Oid, privilege: Text) -> Bool;
}

diesel::define_sql_function! {
    /// `has_schema_privilege(role, schema, privilege)`.
    fn has_schema_privilege(role: Text, schema: Text, privilege: Text) -> Bool;
}

/// Returns, for every ordered pair of `roles` and every privilege the server
/// knows, whether `pg_has_role` grants the first role the privilege on the
/// second, as the facts the describer names.
///
/// `SET` is a `pg_has_role` privilege from PostgreSQL 16 on, so a server of
/// an older `server_major` is asked about the others only.
///
/// # Errors
///
/// Returns a `diesel::result::Error` when a query fails.
pub fn server_role_privileges(
    connection: &mut PgConnection,
    roles: &[String],
    server_major: u32,
) -> Result<Facts, diesel::result::Error> {
    let mut facts = Facts::new();
    let privileges = ROLE_PRIVILEGES
        .iter()
        .filter(|(privilege, _)| server_major >= 16 || *privilege != RolePrivilege::Set);
    for member in roles {
        for target in roles {
            for (_, keyword) in privileges.clone() {
                let holds: bool =
                    diesel::select(pg_has_role(member, target, *keyword)).get_result(connection)?;
                facts.insert(
                    role_privilege_fact(member, keyword, target),
                    Value::Text(holds.to_string()),
                );
            }
        }
    }
    Ok(facts)
}

/// Returns the oid of the relation `schema.name`.
fn relation_oid(
    connection: &mut PgConnection,
    schema: &str,
    name: &str,
) -> Result<u32, diesel::result::Error> {
    pg_class::table
        .inner_join(pg_namespace::table.on(pg_namespace::oid.eq(pg_class::relnamespace)))
        .filter(pg_namespace::nspname.eq(schema))
        .filter(pg_class::relname.eq(name))
        .select(pg_class::oid)
        .first(connection)
}

/// Returns the number and name of every column of the relation `oid`, in
/// table order.
fn columns(
    connection: &mut PgConnection,
    oid: u32,
) -> Result<Vec<(i16, String)>, diesel::result::Error> {
    pg_attribute::table
        .filter(pg_attribute::attrelid.eq(oid))
        .filter(pg_attribute::attnum.gt(0))
        .filter(pg_attribute::attisdropped.eq(false))
        .order(pg_attribute::attnum)
        .select((pg_attribute::attnum, pg_attribute::attname))
        .load(connection)
}

/// Returns, for every table of `tables`, every role of `roles` and every
/// table privilege the server knows, where `has_table_privilege` and
/// `has_column_privilege` grant the role the privilege, as the facts the
/// describer names.
///
/// `MAINTAIN` is a table privilege from PostgreSQL 17 on, so a server of an
/// older `server_major` is asked about the others only.
///
/// # Errors
///
/// Returns a `diesel::result::Error` when a query fails.
pub fn server_table_privileges(
    connection: &mut PgConnection,
    tables: &[(String, String)],
    roles: &[String],
    server_major: u32,
) -> Result<Facts, diesel::result::Error> {
    let mut facts = Facts::new();
    for (schema, name) in tables {
        let oid = relation_oid(connection, schema, name)?;
        let columns = columns(connection, oid)?;
        for role in roles {
            for keyword in COLUMN_PRIVILEGES {
                let on_table: bool = diesel::select(has_table_privilege(role, oid, keyword))
                    .get_result(connection)?;
                let mut holding = Vec::new();
                if !on_table {
                    for (number, column) in &columns {
                        if diesel::select(has_column_privilege(role, oid, *number, keyword))
                            .get_result(connection)?
                        {
                            holding.push(column.clone());
                        }
                    }
                }
                facts.insert(
                    table_privilege_fact(schema, name, keyword, role),
                    Value::Text(column_privilege_value(on_table, &holding)),
                );
            }
            for keyword in WHOLE_TABLE_PRIVILEGES {
                if keyword == "MAINTAIN" && server_major < 17 {
                    continue;
                }
                let holds: bool = diesel::select(has_table_privilege(role, oid, keyword))
                    .get_result(connection)?;
                facts.insert(
                    table_privilege_fact(schema, name, keyword, role),
                    Value::Text(holds.to_string()),
                );
            }
        }
    }
    Ok(facts)
}

/// Returns, for every function of `functions`, keyed as the describer keys
/// it, and every role of `roles`, whether `has_function_privilege` grants the
/// role `EXECUTE` and `EXECUTE WITH GRANT OPTION`, as the facts the describer
/// names.
///
/// # Errors
///
/// Returns a `diesel::result::Error` when a query fails.
pub fn server_function_privileges(
    connection: &mut PgConnection,
    functions: &[(String, u32)],
    roles: &[String],
) -> Result<Facts, diesel::result::Error> {
    let mut facts = Facts::new();
    for (key, oid) in functions {
        for role in roles {
            let can_execute: bool = diesel::select(has_function_privilege(role, *oid, "EXECUTE"))
                .get_result(connection)?;
            let can_grant: bool = diesel::select(has_function_privilege(
                role,
                *oid,
                "EXECUTE WITH GRANT OPTION",
            ))
            .get_result(connection)?;
            facts.insert(
                execute_fact(key, role),
                Value::Text(execute_value(can_execute, can_grant)),
            );
        }
    }
    Ok(facts)
}

/// Returns, for every schema of `schemas` and every role of `roles`, whether
/// `has_schema_privilege` grants the role `USAGE`, as the facts the describer
/// names.
///
/// # Errors
///
/// Returns a `diesel::result::Error` when a query fails.
pub fn server_schema_usage(
    connection: &mut PgConnection,
    schemas: &[String],
    roles: &[String],
) -> Result<Facts, diesel::result::Error> {
    let mut facts = Facts::new();
    for schema in schemas {
        for role in roles {
            let holds: bool = diesel::select(has_schema_privilege(role, schema, "USAGE"))
                .get_result(connection)?;
            facts.insert(
                schema_usage_fact(schema, role),
                Value::Text(holds.to_string()),
            );
        }
    }
    Ok(facts)
}

/// Why `PostgreSQL`'s answer about row-level security could not be read.
#[derive(Debug, thiserror::Error)]
pub enum RowSecurityProbeError {
    /// A catalog query failed.
    #[error(transparent)]
    Catalog(#[from] diesel::result::Error),
    /// The probing session failed.
    #[error(transparent)]
    Session(#[from] postgres::Error),
    /// A stored policy clause is no node tree.
    #[error(transparent)]
    NodeTree(#[from] NodeTreeError),
}

/// One policy clause of a table, as `pg_policy` stores it.
struct StoredClause {
    /// The term the describer names the clause by.
    term: String,
    /// The stored expression, without the fields expansion changes.
    tree: Tree,
}

/// What the probing session needs about one table.
struct ProbedTable {
    /// The table's stored schema.
    schema: String,
    /// The table's stored name.
    name: String,
    /// The first column, quoted, which the probe statements write and read.
    column: Option<String>,
    /// The columns of a unique index, quoted, which the probes name as the
    /// `ON CONFLICT` arbiter.
    arbiter: Option<String>,
    /// Every policy clause of the table.
    clauses: Vec<StoredClause>,
}

/// The fields left out when matching an expression with the policy clause it
/// came from. A stored clause keeps the positions in the policy's source
/// text, which the copy in a rewritten query lacks on some server versions,
/// and the rewriter sets the others as it expands the policies of the tables
/// the expression reads in turn.
const IGNORED_FIELDS: [&str; 6] = [
    "location",
    "stmt_location",
    "stmt_len",
    "securityQuals",
    "hasRowSecurity",
    "hasSubLinks",
];

/// The kinds of `WithCheckOption`, by the number `WCOKind` gives each.
const WITH_CHECK_KINDS: [&str; 6] = [
    "WCO_VIEW_CHECK",
    "WCO_RLS_INSERT_CHECK",
    "WCO_RLS_UPDATE_CHECK",
    "WCO_RLS_CONFLICT_CHECK",
    "WCO_RLS_MERGE_UPDATE_CHECK",
    "WCO_RLS_MERGE_DELETE_CHECK",
];

/// Returns the kinds of `WithCheckOption` the executor checks for
/// `statement`.
///
/// The rewriter gives a `MERGE` the checks of every action whatever actions
/// it has, and the executor runs the checks of the action it takes, so a
/// check of another action never judges a row.
fn checked_kinds(statement: RowStatement) -> &'static [&'static str] {
    match statement {
        RowStatement::Insert { .. }
        | RowStatement::InsertOnConflictDoNothing { .. }
        | RowStatement::MergeInsert { .. } => &["WCO_RLS_INSERT_CHECK"],
        RowStatement::Update { .. } => &["WCO_RLS_UPDATE_CHECK"],
        RowStatement::InsertOnConflictDoUpdate => &[
            "WCO_RLS_INSERT_CHECK",
            "WCO_RLS_CONFLICT_CHECK",
            "WCO_RLS_UPDATE_CHECK",
        ],
        RowStatement::MergeUpdate { .. } => &["WCO_RLS_MERGE_UPDATE_CHECK", "WCO_RLS_UPDATE_CHECK"],
        RowStatement::MergeDelete { .. } => &["WCO_RLS_MERGE_DELETE_CHECK"],
        RowStatement::Select
        | RowStatement::SelectLocking
        | RowStatement::Delete { .. }
        | RowStatement::MergeDoNothing => &[],
    }
}

/// Returns the statement of shape `statement` the probe runs on `table`, or
/// `None` where the table or a server of `server_major` cannot take one.
///
/// The statements change only the first column, set to its default, and
/// read it where the shape reads the table. A `MERGE` that reads its target
/// joins on that column and one that does not joins on `true`, and its
/// `RETURNING` names no column.
fn probe_statement(
    statement: RowStatement,
    table: &ProbedTable,
    server_major: u32,
) -> Option<String> {
    let target = format!("{}.{}", quoted(&table.schema), quoted(&table.name));
    let column = table.column.as_deref();
    let insert = format!("INSERT INTO {target} DEFAULT VALUES");
    let merge = |reads: bool, action: &str| {
        let join = if reads {
            format!("{} IS NULL", column?)
        } else {
            "true".to_owned()
        };
        (server_major >= 15)
            .then(|| format!("MERGE INTO {target} USING (SELECT) AS source ON {join} {action}"))
    };
    match statement {
        RowStatement::Select => Some(format!("SELECT FROM {target}")),
        RowStatement::SelectLocking => Some(format!("SELECT FROM {target} FOR UPDATE")),
        RowStatement::Insert { returning } => Some(if returning {
            format!("{insert} RETURNING *")
        } else {
            insert
        }),
        RowStatement::Update { reads } => column.map(|column| {
            let set = format!("UPDATE {target} SET {column} = DEFAULT");
            if reads {
                format!("{set} WHERE {column} IS NULL")
            } else {
                set
            }
        }),
        RowStatement::Delete { reads: false } => Some(format!("DELETE FROM {target}")),
        RowStatement::Delete { reads: true } => {
            column.map(|column| format!("DELETE FROM {target} WHERE {column} IS NULL"))
        }
        RowStatement::InsertOnConflictDoNothing { arbiter: false } => {
            Some(format!("{insert} ON CONFLICT DO NOTHING"))
        }
        RowStatement::InsertOnConflictDoNothing { arbiter: true } => table
            .arbiter
            .as_deref()
            .map(|arbiter| format!("{insert} ON CONFLICT ({arbiter}) DO NOTHING")),
        RowStatement::InsertOnConflictDoUpdate => {
            let (arbiter, column) = (table.arbiter.as_deref()?, column?);
            Some(format!(
                "{insert} ON CONFLICT ({arbiter}) DO UPDATE SET {column} = DEFAULT"
            ))
        }
        RowStatement::MergeInsert { reads, returning } => {
            let action = "WHEN NOT MATCHED THEN INSERT DEFAULT VALUES";
            if !returning {
                return merge(reads, action);
            }
            if server_major < 17 {
                return None;
            }
            merge(reads, &format!("{action} RETURNING merge_action()"))
        }
        RowStatement::MergeUpdate { reads } => merge(
            reads,
            &format!("WHEN MATCHED THEN UPDATE SET {} = DEFAULT", column?),
        ),
        RowStatement::MergeDelete { reads } => merge(reads, "WHEN MATCHED THEN DELETE"),
        RowStatement::MergeDoNothing => merge(true, "WHEN MATCHED THEN DO NOTHING"),
    }
}

/// Reads what the probes need about `schema.name` from the catalog.
fn probed_table(
    connection: &mut PgConnection,
    schema: &str,
    name: &str,
) -> Result<ProbedTable, RowSecurityProbeError> {
    let oid = relation_oid(connection, schema, name)?;
    let columns = columns(connection, oid)?;
    let unique_keys: Vec<Vec<i16>> = pg_index::table
        .filter(pg_index::indrelid.eq(oid))
        .filter(pg_index::indisunique.eq(true))
        .filter(pg_index::indpred.is_null())
        .filter(pg_index::indexprs.is_null())
        .order(pg_index::indexrelid)
        .select(pg_index::indkey)
        .load(connection)?;
    let arbiter = unique_keys.first().map(|key| {
        key.iter()
            .filter_map(|number| columns.iter().find(|(column, _)| column == number))
            .map(|(_, column)| quoted(column))
            .collect::<Vec<_>>()
            .join(", ")
    });
    let stored: Vec<(String, Option<String>, Option<String>)> = pg_policy::table
        .filter(pg_policy::polrelid.eq(oid))
        .select((
            pg_policy::polname,
            pg_policy::polqual,
            pg_policy::polwithcheck,
        ))
        .load(connection)?;
    let mut clauses = Vec::new();
    for (policy, using, check) in stored {
        for (clause, text) in [
            (PolicyClause::Using, using),
            (PolicyClause::WithCheck, check),
        ] {
            if let Some(text) = text {
                let mut tree = node_tree::parse(&text)?;
                tree.strip(&IGNORED_FIELDS);
                clauses.push(StoredClause {
                    term: rewritten_term(&policy, clause),
                    tree,
                });
            }
        }
    }
    Ok(ProbedTable {
        schema: schema.to_owned(),
        name: name.to_owned(),
        column: columns.first().map(|(_, column)| quoted(column)),
        arbiter,
        clauses,
    })
}

/// Returns the term of the policy clause `qualification` is, or `None` when
/// it is none of the table's clauses.
fn clause_term(qualification: &Tree, clauses: &[StoredClause]) -> Result<Option<String>, String> {
    let matching: Vec<&str> = clauses
        .iter()
        .filter(|clause| clause.tree == *qualification)
        .map(|clause| clause.term.as_str())
        .collect();
    match matching.as_slice() {
        [] => Ok(None),
        [term] => Ok(Some((*term).to_owned())),
        terms => Err(format!(
            "the policy clauses {} are one expression, which the rewritten query cannot tell apart",
            terms.join(", ")
        )),
    }
}

/// Returns whether `tree` is the constant `false`.
fn is_false(tree: &Tree) -> bool {
    tree.name() == Some("CONST")
        && tree.atom("consttype") == Some("16")
        && tree.atom("constisnull") == Some("false")
        && tree.field("constvalue").is_some_and(|value| {
            value
                .iter()
                .skip(2)
                .all(|byte| matches!(byte, Tree::Atom(byte) if byte == "0" || byte == "]"))
        })
}

/// Returns the policy clauses a qualification holds on any of, empty for
/// `false`.
fn qualification_terms(
    qualification: &Tree,
    clauses: &[StoredClause],
) -> Result<Vec<String>, String> {
    let mut qualification = qualification.clone();
    qualification.strip(&IGNORED_FIELDS);
    if let Some(term) = clause_term(&qualification, clauses)? {
        return Ok(vec![term]);
    }
    if is_false(&qualification) {
        return Ok(Vec::new());
    }
    if qualification.name() == Some("BOOLEXPR") && qualification.atom("boolop") == Some("or") {
        let arguments = qualification.list("args").unwrap_or_default();
        let terms: Vec<Option<String>> = arguments
            .iter()
            .map(|argument| clause_term(argument, clauses))
            .collect::<Result<_, _>>()?;
        if let Some(terms) = terms.into_iter().collect::<Option<Vec<String>>>() {
            return Ok(terms);
        }
    }
    Err(format!(
        "a {} qualification matches no policy clause",
        qualification.name().unwrap_or("non-node")
    ))
}

/// Returns the row-level security the rewritten query `tree` of `statement`
/// carries, in the terms the describer uses.
fn rewritten_answer(
    tree: &Tree,
    statement: RowStatement,
    clauses: &[StoredClause],
) -> Result<String, String> {
    let Tree::List(queries) = tree else {
        return Err("the rewritten tree is no list of queries".to_owned());
    };
    let [query] = queries.as_slice() else {
        return Err(format!(
            "the statement rewrote into {} queries",
            queries.len()
        ));
    };
    let rtable = query.list("rtable").ok_or("the query has no range table")?;
    let result: usize = query
        .atom("resultRelation")
        .and_then(|number| number.parse().ok())
        .ok_or("the query has no result relation")?;
    let target = rtable
        .get(result.saturating_sub(1))
        .ok_or("the range table lacks the probed table")?;
    let mut items = BTreeSet::new();
    for qualification in target.list("securityQuals").unwrap_or_default() {
        items.insert(rewritten_item(
            "filter",
            qualification_terms(qualification, clauses)?,
        ));
    }
    let checked = checked_kinds(statement);
    for option in query.list("withCheckOptions").unwrap_or_default() {
        let kind = option
            .atom("kind")
            .and_then(|kind| kind.parse::<usize>().ok())
            .and_then(|kind| WITH_CHECK_KINDS.get(kind))
            .ok_or("a WithCheckOption of unknown kind")?;
        if !checked.contains(kind) {
            continue;
        }
        let [qualification] = option.field("qual").unwrap_or_default() else {
            return Err(format!("a {kind} without one qualification"));
        };
        items.insert(rewritten_item(
            kind,
            qualification_terms(qualification, clauses)?,
        ));
    }
    Ok(rewritten(items))
}

/// Runs `sql` on `table` as `role` in a transaction it rolls back, with the
/// rewritten query logged into `log`, and returns the answer it reads off
/// the log.
///
/// Looking a table up by name takes `USAGE` on its schema, which the
/// transaction grants `role` first. The rewriter logs the query before the
/// executor runs it, so a statement the executor refuses, for want of a
/// privilege or for a row a check rejects, still leaves its rewritten query.
/// A policy expansion looping back into a table fails in the rewriter, and
/// its message is the answer.
fn probe(
    client: &mut postgres::Client,
    log: &Mutex<Vec<String>>,
    table: &ProbedTable,
    role: &str,
    sql: &str,
    statement: RowStatement,
) -> Result<String, postgres::Error> {
    client.batch_execute(&format!(
        "BEGIN; GRANT USAGE ON SCHEMA {} TO {role}; SET LOCAL statement_timeout = '10s'; \
         SET LOCAL client_min_messages = log; SET LOCAL debug_pretty_print = off; \
         SET LOCAL debug_print_rewritten = on; SET LOCAL ROLE {role};",
        quoted(&table.schema),
        role = quoted(role)
    ))?;
    log.lock().unwrap_or_else(PoisonError::into_inner).clear();
    let outcome = client.batch_execute(sql);
    let logged = std::mem::take(&mut *log.lock().unwrap_or_else(PoisonError::into_inner));
    client.batch_execute("ROLLBACK")?;
    let answer = match (logged.as_slice(), outcome) {
        ([tree], _) => node_tree::parse(tree)
            .map_err(|error| error.to_string())
            .and_then(|tree| rewritten_answer(&tree, statement, &table.clauses)),
        ([], Err(error)) => match error.as_db_error() {
            Some(refusal) if refusal.code().code() == "42P17" => Ok(refusal.message().to_owned()),
            _ => Err(format!(
                "PostgreSQL refused `{sql}`: {}",
                error
                    .as_db_error()
                    .map_or_else(|| error.to_string(), |refusal| refusal.message().to_owned())
            )),
        },
        (trees, _) => Err(format!("`{sql}` logged {} rewritten queries", trees.len())),
    };
    Ok(answer.unwrap_or_else(|reason| format!("unreadable: {reason}")))
}

/// Returns, for every table of `tables`, every role of `roles` and every
/// statement shape a server of `server_major` takes on the table, the
/// row-level security `PostgreSQL`'s rewriter applies when the role runs it,
/// as the facts the describer names.
///
/// `diesel` drops the log messages that carry the rewritten query, so the
/// probes run on a `postgres` session to the database `database` on `port`,
/// its own thread driving it, as that client starts a runtime of its own,
/// which the thread of the test's runtime cannot.
///
/// # Errors
///
/// Returns a [`RowSecurityProbeError`] when a catalog query or the probing
/// session fails.
pub fn server_row_security(
    connection: &mut PgConnection,
    port: u16,
    database: &str,
    tables: &[(String, String)],
    roles: &[String],
    server_major: u32,
) -> Result<Facts, RowSecurityProbeError> {
    let probed: Vec<ProbedTable> = tables
        .iter()
        .map(|(schema, name)| probed_table(connection, schema, name))
        .collect::<Result<_, _>>()?;
    let url = database_url(port, database);
    std::thread::scope(|scope| {
        scope
            .spawn(|| -> Result<Facts, RowSecurityProbeError> {
                let log = Arc::new(Mutex::new(Vec::new()));
                let sink = Arc::clone(&log);
                let mut client = url
                    .parse::<postgres::Config>()?
                    .connect_timeout(Duration::from_secs(10))
                    .notice_callback(move |notice| {
                        if notice.message() == "rewritten parse tree:"
                            && let Some(tree) = notice.detail()
                        {
                            sink.lock()
                                .unwrap_or_else(PoisonError::into_inner)
                                .push(tree.to_owned());
                        }
                    })
                    .connect(postgres::NoTls)?;
                let mut facts = Facts::new();
                for table in &probed {
                    for role in roles {
                        for (statement, label) in ROW_STATEMENTS {
                            let Some(sql) = probe_statement(statement, table, server_major) else {
                                continue;
                            };
                            let answer = probe(&mut client, &log, table, role, &sql, statement)?;
                            facts.insert(
                                rewritten_fact(&table.schema, &table.name, role, label),
                                Value::Text(answer),
                            );
                        }
                    }
                }
                Ok(facts)
            })
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })
}
