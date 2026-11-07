//! Compares two [`DatabaseLike`] models of one schema, fact by fact.
//!
//! Each model is projected onto the same set of named facts, so the
//! comparison is the symmetric difference of two maps. A fact names the
//! object it belongs to and the property it states, for example
//! `table public.posts / column body / nullable`. Identifiers enter the names
//! in the form PostgreSQL stores them, which is what
//! [`identifiers_match`](pg_diesel::sql_traits::utils::identifier_resolution::identifiers_match)
//! compares.
//!
//! Expressions are compared as parsed [`Expr`] trees, after both sides pass
//! through the print PostgreSQL stores. The live model already holds that
//! print. The source model's expressions go through a [`StoredForm`], which
//! writes them into a scratch object and reads PostgreSQL's print back.
//!
//! Nothing here depends on a test framework, so the routine can serve a
//! caller that compares a parsed schema with a live database at run time.

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use pg_diesel::{
    sql_traits::{
        errors::LookupError,
        structs::{
            ColumnPrivilege, GateFailure, GateTerm, PolicyClause, RowSecurity, RowStatement,
            RowVersion,
        },
        traits::{
            BinarySend, CheckConstraintLike, ColumnGrantLike, ColumnLike, DataTypeLike,
            DatabaseLike, DomainLike, ExtensionLike, ForeignKeyLike, FunctionLike, GrantLike,
            IndexLike, PolicyLike, Relation, RoleLike, RolePrivilege, RuleLike, SchemaLike,
            SequenceLike, SequenceOwner, TableGrantLike, TableLike, TriggerLike, UniqueIndexLike,
            UserDefinedTypeLike, ViewLike,
        },
        utils::identifier_resolution::normalize_identifier,
    },
    sqlparser::{
        ast::{Action, CreateRule, Expr, Query, Statement},
        dialect::PostgreSqlDialect,
        parser::{Parser, ParserError},
    },
};

/// Where an expression is written, which decides how `PostgreSQL` prints it
/// back.
#[derive(Debug, Clone, Copy)]
pub enum Slot<'a> {
    /// The `DEFAULT` of the named column.
    ColumnDefault {
        /// The column the default belongs to.
        column: &'a str,
    },
    /// A table `CHECK` constraint.
    Check,
    /// The `USING` clause of a policy.
    PolicyUsing,
    /// The `WITH CHECK` clause of a policy.
    PolicyCheck,
    /// The `WHERE` predicate of a partial index.
    IndexPredicate,
}

impl fmt::Display for Slot<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ColumnDefault { column } => write!(formatter, "default of {column}"),
            Self::Check => formatter.write_str("check"),
            Self::PolicyUsing => formatter.write_str("policy using"),
            Self::PolicyCheck => formatter.write_str("policy with check"),
            Self::IndexPredicate => formatter.write_str("index predicate"),
        }
    }
}

/// Why an expression could not be brought into its stored form.
#[derive(Debug, thiserror::Error)]
pub enum StoredFormError {
    /// `PostgreSQL` refused the scratch statement holding the expression.
    #[error("PostgreSQL refused the {slot} `{sql}` on {table}: {message}")]
    Refused {
        /// The table the expression was written on.
        table: String,
        /// The slot it was written into.
        slot: String,
        /// The expression text.
        sql: String,
        /// The server's message.
        message: String,
    },
    /// The print could not be parsed back.
    #[error("the print `{print}` does not parse")]
    Unparsable {
        /// The text `PostgreSQL` printed.
        print: String,
        /// The parser's complaint.
        #[source]
        source: ParserError,
    },
    /// The print is not one `CREATE RULE` statement.
    #[error("the print `{print}` is not a CREATE RULE statement")]
    NotRule {
        /// The text `PostgreSQL` printed.
        print: String,
    },
}

/// Turns expression text into the tree `PostgreSQL` stores for it.
pub trait StoredForm {
    /// Returns `sql`, written into `slot` of the table `schema.table`, as the
    /// expression `PostgreSQL` prints back.
    ///
    /// # Errors
    ///
    /// Returns a [`StoredFormError`] when `PostgreSQL` refuses the expression
    /// or its print does not parse.
    fn expression(
        &mut self,
        schema: &str,
        table: &str,
        slot: Slot<'_>,
        sql: &str,
    ) -> Result<Expr, StoredFormError>;

    /// Returns the view query `sql` as `PostgreSQL` stores it.
    ///
    /// # Errors
    ///
    /// Returns a [`StoredFormError`] when `PostgreSQL` refuses the query or
    /// its print does not parse.
    fn view_query(&mut self, sql: &str) -> Result<Query, StoredFormError>;

    /// Returns the `CREATE RULE` statement `sql` as `PostgreSQL` stores it.
    ///
    /// # Errors
    ///
    /// Returns a [`StoredFormError`] when `PostgreSQL` refuses the rule or
    /// its print does not parse.
    fn rule(&mut self, sql: &str) -> Result<CreateRule, StoredFormError>;
}

/// Reads text that already is `PostgreSQL`'s print, as the live model holds.
#[derive(Debug, Default, Clone, Copy)]
pub struct AlreadyStored;

/// Parses `print` as one expression in the `PostgreSQL` dialect.
///
/// # Errors
///
/// Returns [`StoredFormError::Unparsable`] when `print` is not one expression.
pub fn parse_expression(print: &str) -> Result<Expr, StoredFormError> {
    Parser::new(&PostgreSqlDialect {})
        .try_with_sql(print)
        .and_then(|mut parser| parser.parse_expr())
        .map_err(|source| StoredFormError::Unparsable {
            print: print.to_owned(),
            source,
        })
}

/// Parses `print` as one query in the `PostgreSQL` dialect.
///
/// # Errors
///
/// Returns [`StoredFormError::Unparsable`] when `print` is not one query.
pub fn parse_query(print: &str) -> Result<Query, StoredFormError> {
    Parser::new(&PostgreSqlDialect {})
        .try_with_sql(print.trim().trim_end_matches(';'))
        .and_then(|mut parser| parser.parse_query())
        .map(|query| *query)
        .map_err(|source| StoredFormError::Unparsable {
            print: print.to_owned(),
            source,
        })
}

/// Parses `print` as one `CREATE RULE` statement in the `PostgreSQL` dialect.
///
/// # Errors
///
/// Returns [`StoredFormError::Unparsable`] when `print` does not parse and
/// [`StoredFormError::NotRule`] when it is another statement.
pub fn parse_rule(print: &str) -> Result<CreateRule, StoredFormError> {
    let statements = Parser::parse_sql(&PostgreSqlDialect {}, print).map_err(|source| {
        StoredFormError::Unparsable {
            print: print.to_owned(),
            source,
        }
    })?;
    match statements.into_iter().next() {
        Some(Statement::CreateRule(rule)) => Ok(rule),
        _ => Err(StoredFormError::NotRule {
            print: print.to_owned(),
        }),
    }
}

impl StoredForm for AlreadyStored {
    fn expression(
        &mut self,
        _schema: &str,
        _table: &str,
        _slot: Slot<'_>,
        sql: &str,
    ) -> Result<Expr, StoredFormError> {
        parse_expression(sql)
    }

    fn view_query(&mut self, sql: &str) -> Result<Query, StoredFormError> {
        parse_query(sql)
    }

    fn rule(&mut self, sql: &str) -> Result<CreateRule, StoredFormError> {
        parse_rule(sql)
    }
}

/// The value a fact states.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A value with a canonical text form.
    Text(String),
    /// An expression in its stored form, compared as a tree.
    Expr(Box<Expr>),
    /// A query in its stored form, compared as a tree.
    Query(Box<Query>),
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(text) => formatter.write_str(text),
            Self::Expr(expr) => write!(formatter, "{expr}"),
            Self::Query(query) => write!(formatter, "{query}"),
        }
    }
}

/// Every fact a model states, by name.
pub type Facts = BTreeMap<String, Value>;

/// Why a model could not be described.
#[derive(Debug, thiserror::Error)]
pub enum DescribeError {
    /// A trait reader failed to resolve an object.
    #[error(transparent)]
    Lookup(#[from] LookupError),
    /// An expression could not be brought into its stored form.
    #[error(transparent)]
    StoredForm(#[from] StoredFormError),
}

/// One fact the two models state differently.
#[derive(Debug, Clone, PartialEq)]
pub struct Difference {
    /// The fact's name.
    pub fact: String,
    /// What the source model states, if it states the fact.
    pub source: Option<Value>,
    /// What the live model states, if it states the fact.
    pub live: Option<Value>,
}

impl Difference {
    /// Returns the difference printed with `source` and `live` naming its two
    /// sides.
    pub fn labelled<'a>(&'a self, source: &'a str, live: &'a str) -> impl fmt::Display + 'a {
        fmt::from_fn(move |formatter| {
            let side = |value: &Option<Value>| {
                value
                    .as_ref()
                    .map_or_else(|| "absent".to_owned(), ToString::to_string)
            };
            write!(
                formatter,
                "{}: {source} `{}`, {live} `{}`",
                self.fact,
                side(&self.source),
                side(&self.live)
            )
        })
    }
}

impl fmt::Display for Difference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.labelled("source", "live").fmt(formatter)
    }
}

/// Returns the facts the two descriptions state differently, ordered by name.
#[must_use]
pub fn differences(source: &Facts, live: &Facts) -> Vec<Difference> {
    let mut found = Vec::new();
    for (fact, value) in source {
        if live.get(fact) != Some(value) {
            found.push(Difference {
                fact: fact.clone(),
                source: Some(value.clone()),
                live: live.get(fact).cloned(),
            });
        }
    }
    for (fact, value) in live {
        if !source.contains_key(fact) {
            found.push(Difference {
                fact: fact.clone(),
                source: None,
                live: Some(value.clone()),
            });
        }
    }
    found.sort_by(|left, right| left.fact.cmp(&right.fact));
    found
}

/// Returns the facts `reference` states that `live` states differently or
/// not at all, ordered by name.
#[must_use]
pub fn contradictions(reference: &Facts, live: &Facts) -> Vec<Difference> {
    reference
        .iter()
        .filter(|(fact, value)| live.get(*fact) != Some(*value))
        .map(|(fact, value)| Difference {
            fact: fact.clone(),
            source: Some(value.clone()),
            live: live.get(fact).cloned(),
        })
        .collect()
}

/// Every privilege one role can hold on another, with the keyword
/// `pg_has_role` takes for it.
pub const ROLE_PRIVILEGES: [(RolePrivilege, &str); 4] = [
    (RolePrivilege::Member, "MEMBER"),
    (RolePrivilege::Usage, "USAGE"),
    (RolePrivilege::Set, "SET"),
    (RolePrivilege::Admin, "MEMBER WITH ADMIN OPTION"),
];

/// Returns the name of the fact stating whether the role `member` holds the
/// privilege `keyword` on the role `target`.
#[must_use]
pub fn role_privilege_fact(member: &str, keyword: &str, target: &str) -> String {
    format!("role {member} / {keyword} on role {target}")
}

/// The table privileges that can be granted per column, by keyword, in the
/// order of the fields of `TablePrivileges`.
pub const COLUMN_PRIVILEGES: [&str; 4] = ["SELECT", "INSERT", "UPDATE", "REFERENCES"];

/// The table privileges that exist on the whole table only, by keyword, in
/// the order of the fields of `TablePrivileges`.
pub const WHOLE_TABLE_PRIVILEGES: [&str; 4] = ["DELETE", "TRUNCATE", "TRIGGER", "MAINTAIN"];

/// Returns the name of the fact stating where the role `role` holds the
/// privilege `keyword` on the table `schema.table`.
#[must_use]
pub fn table_privilege_fact(schema: &str, table: &str, keyword: &str, role: &str) -> String {
    format!("table {schema}.{table} / privilege {keyword} / role {role}")
}

/// Returns where a column privilege holds, `table` for the table-level
/// privilege, else the `columns` that carry it or `none`.
#[must_use]
pub fn column_privilege_value(on_table: bool, columns: &[String]) -> String {
    if on_table {
        "table".to_owned()
    } else if columns.is_empty() {
        "none".to_owned()
    } else {
        format!("columns {}", columns.join(", "))
    }
}

/// Every statement shape row-level security tells apart, with the label the
/// facts about it carry.
pub const ROW_STATEMENTS: [(RowStatement, &str); 20] = [
    (RowStatement::Select, "SELECT"),
    (RowStatement::SelectLocking, "SELECT FOR UPDATE"),
    (RowStatement::Insert { returning: false }, "INSERT"),
    (RowStatement::Insert { returning: true }, "INSERT RETURNING"),
    (RowStatement::Update { reads: false }, "UPDATE"),
    (
        RowStatement::Update { reads: true },
        "UPDATE reading the table",
    ),
    (RowStatement::Delete { reads: false }, "DELETE"),
    (
        RowStatement::Delete { reads: true },
        "DELETE reading the table",
    ),
    (
        RowStatement::InsertOnConflictDoNothing { arbiter: false },
        "INSERT ON CONFLICT DO NOTHING",
    ),
    (
        RowStatement::InsertOnConflictDoNothing { arbiter: true },
        "INSERT ON CONFLICT (arbiter) DO NOTHING",
    ),
    (
        RowStatement::InsertOnConflictDoUpdate,
        "INSERT ON CONFLICT DO UPDATE",
    ),
    (
        RowStatement::MergeInsert {
            reads: false,
            returning: false,
        },
        "MERGE INSERT",
    ),
    (
        RowStatement::MergeInsert {
            reads: false,
            returning: true,
        },
        "MERGE INSERT RETURNING",
    ),
    (
        RowStatement::MergeInsert {
            reads: true,
            returning: false,
        },
        "MERGE INSERT reading the target",
    ),
    (
        RowStatement::MergeInsert {
            reads: true,
            returning: true,
        },
        "MERGE INSERT RETURNING reading the target",
    ),
    (RowStatement::MergeUpdate { reads: false }, "MERGE UPDATE"),
    (
        RowStatement::MergeUpdate { reads: true },
        "MERGE UPDATE reading the target",
    ),
    (RowStatement::MergeDelete { reads: false }, "MERGE DELETE"),
    (
        RowStatement::MergeDelete { reads: true },
        "MERGE DELETE reading the target",
    ),
    (RowStatement::MergeDoNothing, "MERGE DO NOTHING"),
];

/// Returns the name of the fact stating the row-level security the statement
/// `statement` meets on the table `schema.table` when `role` runs it, in the
/// terms of the query `PostgreSQL`'s rewriter produces.
#[must_use]
pub fn rewritten_fact(schema: &str, table: &str, role: &str, statement: &str) -> String {
    format!("table {schema}.{table} / rewritten for role {role} on {statement}")
}

/// Returns one term of a rewritten qualification, the clause `clause` of the
/// policy `policy`.
#[must_use]
pub fn rewritten_term(policy: &str, clause: PolicyClause) -> String {
    match clause {
        PolicyClause::Using => format!("{policy} USING"),
        PolicyClause::WithCheck => format!("{policy} WITH CHECK"),
    }
}

/// Returns one qualification of a rewritten query, kept in `list`, the row
/// filter or the kind of `WithCheckOption`, holding when any of `terms` holds
/// and so `false` without terms.
#[must_use]
pub fn rewritten_item(list: &str, mut terms: Vec<String>) -> String {
    terms.sort();
    if terms.is_empty() {
        format!("{list} false")
    } else {
        format!("{list} any of {}", terms.join(", "))
    }
}

/// Returns the qualifications of a rewritten query as one value, `none`
/// when it carries none.
#[must_use]
pub fn rewritten(items: BTreeSet<String>) -> String {
    if items.is_empty() {
        "none".to_owned()
    } else {
        items.into_iter().collect::<Vec<_>>().join("; ")
    }
}

/// Returns the message `PostgreSQL` refuses a statement with when expanding
/// its policies loops back into `relation`.
#[must_use]
pub fn recursion(relation: &str) -> String {
    format!("infinite recursion detected in policy for relation \"{relation}\"")
}

/// Returns `security`, the answer for `statement`, in the terms of the query
/// `PostgreSQL`'s rewriter produces.
///
/// A filter gate is a qualification on the table's range table entry, and a
/// raising gate a `WithCheckOption` of the kind the executor checks for the
/// row it judges. A gate with no permissive term admits nothing, which
/// `PostgreSQL` writes as `false` without the restrictive terms. The
/// qualifications are a set, as the rewriter adds each only once.
fn rewritten_form<P: PolicyLike>(security: &RowSecurity<'_, P>, statement: RowStatement) -> String {
    let gates = match security {
        RowSecurity::Exempt(_) => return rewritten(BTreeSet::new()),
        RowSecurity::InfiniteRecursion { relation } => return recursion(relation),
        RowSecurity::Gated(gates) => gates,
    };
    let term = |term: &GateTerm<'_, P>| rewritten_term(term.policy().name(), term.clause());
    let mut items = BTreeSet::new();
    for gate in gates {
        let list = match (gate.failure(), gate.row()) {
            (GateFailure::Filter, _) => "filter",
            (GateFailure::Raise, RowVersion::Inserted) => "WCO_RLS_INSERT_CHECK",
            (GateFailure::Raise, RowVersion::Updated) => "WCO_RLS_UPDATE_CHECK",
            (GateFailure::Raise, RowVersion::Existing) => match statement {
                RowStatement::InsertOnConflictDoUpdate => "WCO_RLS_CONFLICT_CHECK",
                RowStatement::MergeUpdate { .. } => "WCO_RLS_MERGE_UPDATE_CHECK",
                RowStatement::MergeDelete { .. } => "WCO_RLS_MERGE_DELETE_CHECK",
                _ => "check of the existing row",
            },
        };
        items.insert(rewritten_item(
            list,
            gate.permissive().iter().map(term).collect(),
        ));
        if !gate.permissive().is_empty() {
            for restrictive in gate.restrictive() {
                items.insert(rewritten_item(list, vec![term(restrictive)]));
            }
        }
    }
    rewritten(items)
}

/// Returns `security` with every detail the trait answers, the gates in
/// their order and the reason for an exemption.
fn row_security_detail<P: PolicyLike>(security: &RowSecurity<'_, P>) -> String {
    let gates = match security {
        RowSecurity::Exempt(reason) => return format!("exempt, {reason:?}"),
        RowSecurity::InfiniteRecursion { relation } => return recursion(relation),
        RowSecurity::Gated(gates) => gates,
    };
    let terms = |terms: &[GateTerm<'_, P>]| {
        terms
            .iter()
            .map(|term| rewritten_term(term.policy().name(), term.clause()))
            .collect::<Vec<_>>()
            .join(", ")
    };
    gates
        .iter()
        .map(|gate| {
            format!(
                "{:?} {:?} {:?}: any of [{}] and all of [{}]",
                gate.command(),
                gate.row(),
                gate.failure(),
                terms(gate.permissive()),
                terms(gate.restrictive())
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// The objects a description covers.
#[derive(Debug, Clone)]
pub struct Scope {
    /// The schema an unqualified name lands in, `current_schema()` of the
    /// session that applied the DDL.
    pub default_schema: String,
    /// The schemas whose objects are described.
    pub schemas: Vec<String>,
    /// Roles left out of the description, such as the superuser applying the
    /// DDL.
    pub ignored_roles: Vec<String>,
    /// The `(schema, name)` of every function and relation an extension
    /// script created, which DDL never spells out.
    pub extension_members: BTreeSet<(String, String)>,
}

impl Scope {
    /// Returns the stored schema name, with an unqualified name resolved to
    /// the default schema.
    fn schema<'a>(&'a self, schema: Option<Cow<'a, str>>) -> Cow<'a, str> {
        schema.unwrap_or(Cow::Borrowed(self.default_schema.as_str()))
    }

    /// Returns whether objects in `schema` are described.
    fn covers(&self, schema: &str) -> bool {
        self.schemas.iter().any(|covered| covered == schema)
    }

    /// Returns whether `schema.name` is described, which an object an
    /// extension created is not.
    fn covers_object(&self, schema: &str, name: &str) -> bool {
        self.covers(schema)
            && !self
                .extension_members
                .contains(&(schema.to_owned(), name.to_owned()))
    }

    /// Returns whether the role is described.
    fn covers_role(&self, role: &str) -> bool {
        !role.starts_with("pg_") && !self.ignored_roles.iter().any(|ignored| ignored == role)
    }

    /// Returns the stored names of the roles of `database` that are described.
    pub fn roles<DB: DatabaseLike>(&self, database: &DB) -> Vec<String> {
        database
            .roles()
            .map(|role| role.stored_name().into_owned())
            .filter(|name| self.covers_role(name))
            .collect()
    }

    /// Returns the stored schema and name of every table of `database` that
    /// is described, foreign tables left out.
    pub fn tables<DB: DatabaseLike>(&self, database: &DB) -> Vec<(String, String)> {
        database
            .tables()
            .map(|table| {
                (
                    self.schema(table.stored_table_schema()).into_owned(),
                    table.stored_table_name().into_owned(),
                )
            })
            .filter(|(schema, name)| self.covers_object(schema, name))
            .collect()
    }
}

/// Collects the facts of one model.
struct Describer<'a, DB: DatabaseLike> {
    /// The model described.
    database: &'a DB,
    /// The objects covered.
    scope: &'a Scope,
    /// The route expressions take to their stored form.
    stored: &'a mut dyn StoredForm,
    /// The facts collected so far.
    facts: Facts,
}

/// Returns `name` as a quoted identifier.
pub fn quoted(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// Returns the privilege keyword `action` renders with, without the column
/// list `Action` prints behind it, which the fact already names.
fn privilege_keyword(action: &Action) -> String {
    action
        .to_string()
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_owned()
}

/// Returns `values` sorted and joined, so a set compares independently of
/// order.
fn set<I: IntoIterator<Item = String>>(values: I) -> String {
    let mut values: Vec<String> = values.into_iter().collect();
    values.sort();
    values.dedup();
    values.join(", ")
}

impl<'a, DB: DatabaseLike> Describer<'a, DB> {
    /// Returns the roles in scope, each with its stored name.
    fn described_roles(&self) -> Vec<(String, &'a DB::Role)> {
        self.database
            .roles()
            .map(|role| (role.stored_name().into_owned(), role))
            .filter(|(name, _)| self.scope.covers_role(name))
            .collect()
    }
}

impl<DB: DatabaseLike> Describer<'_, DB> {
    /// Records one fact with a textual value.
    fn text(&mut self, fact: String, value: impl Into<String>) {
        self.facts.insert(fact, Value::Text(value.into()));
    }

    /// Returns the qualified stored name of a table.
    fn table_name(&self, table: &DB::Table) -> (String, String) {
        let schema = self.scope.schema(table.stored_table_schema()).into_owned();
        (schema, table.stored_table_name().into_owned())
    }

    /// Returns the qualified stored name of a plain or materialized view.
    fn view_name<V: ViewLike>(&self, view: &V) -> (String, String) {
        (
            self.scope.schema(view.stored_view_schema()).into_owned(),
            view.stored_view_name().into_owned(),
        )
    }

    /// Returns the stored schema and name of any relation.
    fn relation_parts(&self, relation: &Relation<'_, DB>) -> (String, String) {
        match relation {
            Relation::Table(table) | Relation::ForeignTable(table) => self.table_name(table),
            Relation::View(view) => self.view_name(*view),
            Relation::MaterializedView(view) => self.view_name(*view),
            Relation::Sequence(sequence) => (
                self.scope
                    .schema(sequence.stored_sequence_schema())
                    .into_owned(),
                sequence.stored_sequence_name().into_owned(),
            ),
        }
    }

    /// Returns the qualified stored name of any relation.
    fn relation_name(&self, relation: &Relation<'_, DB>) -> String {
        let (schema, name) = self.relation_parts(relation);
        format!("{schema}.{name}")
    }

    /// Records every fact the model states.
    fn describe(&mut self) -> Result<(), DescribeError> {
        self.schemas();
        let database = self.database;
        let tables = database
            .tables()
            .map(|table| (table, false))
            .chain(database.foreign_tables().map(|table| (table, true)));
        for (table, foreign) in tables {
            let (schema, name) = self.table_name(table);
            if self.scope.covers_object(&schema, &name) {
                self.table(table, foreign)?;
            }
        }
        self.views()?;
        self.functions()?;
        self.roles();
        self.memberships();
        self.role_privileges();
        self.grants()?;
        self.sequences();
        self.rules()?;
        self.extensions();
        self.domains();
        self.user_defined_types();
        Ok(())
    }

    /// Records the schemas in scope, with the `USAGE` each holds before any
    /// `GRANT`.
    fn schemas(&mut self) {
        for schema in self.database.schemas() {
            let name = schema.stored_name().into_owned();
            if self.scope.covers(&name) {
                self.text(format!("schema {name}"), "present");
                self.text(
                    format!("schema {name} / public holds usage"),
                    schema.public_holds_usage().to_string(),
                );
                self.text(
                    format!("schema {name} / owner holds usage"),
                    schema.owner_holds_usage().to_string(),
                );
            }
        }
    }

    /// Records a table, its columns, keys, constraints, indexes, triggers and
    /// policies.
    fn table(&mut self, table: &DB::Table, foreign: bool) -> Result<(), DescribeError> {
        let database = self.database;
        let (schema, name) = self.table_name(table);
        let subject = format!("table {schema}.{name}");
        self.text(
            format!("{subject} / kind"),
            if foreign { "foreign table" } else { "table" },
        );
        self.text(
            format!("{subject} / row level security"),
            table.has_row_level_security(database)?.to_string(),
        );
        self.text(
            format!("{subject} / forced row level security"),
            table.has_forced_row_level_security(database)?.to_string(),
        );
        self.text(
            format!("{subject} / unlogged"),
            table.is_unlogged().to_string(),
        );
        self.text(
            format!("{subject} / storage parameters"),
            set(table
                .storage_parameters()
                .map(|(key, value)| format!("{key}={value}"))),
        );
        if let Some(bound) = table.partition_bound() {
            self.text(format!("{subject} / partition bound"), format!("{bound:?}"));
        }
        self.text(
            format!("{subject} / owner privileges"),
            set(table.owner_privileges(database)?.map(ToString::to_string)),
        );
        if let Some(source) = table.foreign_source(database)? {
            self.text(format!("{subject} / foreign source"), format!("{source:?}"));
        }
        let parents: Vec<String> = table
            .extended_tables(database)?
            .map(|parent| {
                let (schema, name) = self.table_name(parent);
                format!("{schema}.{name}")
            })
            .collect();
        self.text(format!("{subject} / parents"), parents.join(", "));

        let columns: Vec<&DB::Column> = table.columns(database)?.collect();
        self.text(
            format!("{subject} / column order"),
            columns
                .iter()
                .map(|column| column.stored_column_name())
                .collect::<Vec<_>>()
                .join(", "),
        );
        for column in columns {
            self.column(&schema, &name, &subject, column)?;
        }
        self.text(
            format!("{subject} / primary key"),
            table
                .primary_key_columns(database)?
                .map(|column| column.stored_column_name().into_owned())
                .collect::<Vec<_>>()
                .join(", "),
        );
        self.checks(&schema, &name, &subject, table)?;
        self.foreign_keys(&subject, table)?;
        self.indexes(&schema, &name, &subject, table)?;
        self.triggers(&subject, table)?;
        self.policies(&schema, &name, &subject, table)?;
        if !foreign {
            self.table_privileges(&schema, &name, table)?;
            self.row_security(&schema, &name, table);
        }
        Ok(())
    }

    /// Records a column's type, nullability, default and generation.
    fn column(
        &mut self,
        schema: &str,
        table: &str,
        subject: &str,
        column: &DB::Column,
    ) -> Result<(), DescribeError> {
        let database = self.database;
        let column_name = column.stored_column_name().into_owned();
        let subject = format!("{subject} / column {column_name}");
        self.data_type(&format!("{subject} / type"), column.data_type(database));
        self.text(
            format!("{subject} / nullable"),
            column.is_nullable(database)?.to_string(),
        );
        self.text(
            format!("{subject} / not null constraint name"),
            format!("{:?}", column.not_null_constraint_name(database)?),
        );
        if let Some(default) = column.default_value() {
            let stored = self.stored.expression(
                schema,
                table,
                Slot::ColumnDefault {
                    column: &column_name,
                },
                &default,
            )?;
            self.facts.insert(
                format!("{subject} / default"),
                Value::Expr(Box::new(stored)),
            );
        }
        self.text(
            format!("{subject} / generated"),
            column.is_generated(database).to_string(),
        );
        self.text(
            format!("{subject} / identity"),
            format!("{:?}", column.identity_generation(database)),
        );
        self.text(
            format!("{subject} / serial"),
            column.is_serial(database).to_string(),
        );
        self.text(
            format!("{subject} / collation"),
            format!("{:?}", column.collation(database)?),
        );
        self.text(
            format!("{subject} / foreign options"),
            set(column
                .foreign_options()
                .map(|(key, value)| format!("{key}={value}"))),
        );
        Ok(())
    }

    /// Records the facts of a data type, recursing into array elements and
    /// domain base types.
    fn data_type(&mut self, subject: &str, data_type: &DB::DataType) {
        let database = self.database;
        self.text(
            format!("{subject} / name"),
            data_type.normalized_name(database),
        );
        self.text(
            format!("{subject} / length"),
            format!("{:?}", data_type.max_character_length(database)),
        );
        self.text(
            format!("{subject} / scale"),
            format!("{:?}", data_type.numeric_scale(database)),
        );
        self.text(
            format!("{subject} / fractional seconds"),
            format!("{:?}", data_type.fractional_seconds_precision(database)),
        );
        self.text(
            format!("{subject} / interval fields"),
            format!("{:?}", data_type.interval_fields(database)),
        );
        self.text(
            format!("{subject} / array dimensions"),
            format!("{:?}", data_type.array_dimensions(database)),
        );
        self.text(
            format!("{subject} / family"),
            format!("{:?}", data_type.scalar_family(database)),
        );
        self.text(
            format!("{subject} / vector"),
            format!("{:?}", data_type.vector(database)),
        );
        self.text(
            format!("{subject} / binary send"),
            match data_type.binary_send(database) {
                BinarySend::Binary => "Binary".to_owned(),
                BinarySend::Text => "Text".to_owned(),
                BinarySend::Fails(type_) => format!("Fails({})", type_.normalized_name(database)),
                BinarySend::Unknown(type_) => {
                    format!("Unknown({})", type_.normalized_name(database))
                }
            },
        );
        self.text(
            format!("{subject} / binary length"),
            format!("{:?}", data_type.fixed_binary_length(database)),
        );
        if let Some(element) = data_type.array_element(database) {
            self.data_type(&format!("{subject} / element"), element);
        }
        if let Some(base) = data_type.base_type(database) {
            self.data_type(&format!("{subject} / base"), base);
        }
    }

    /// Records a table's check constraints, keyed by their stored expression.
    fn checks(
        &mut self,
        schema: &str,
        table_name: &str,
        subject: &str,
        table: &DB::Table,
    ) -> Result<(), DescribeError> {
        let database = self.database;
        for check in table.check_constraints(database)? {
            let stored = self.stored.expression(
                schema,
                table_name,
                Slot::Check,
                &check.expression(database).to_string(),
            )?;
            let check_subject = format!("{subject} / check {stored}");
            self.text(
                format!("{check_subject} / constraint name"),
                format!("{:?}", check.constraint_name(database)?),
            );
            self.text(
                format!("{check_subject} / validated"),
                check.is_validated(database)?.to_string(),
            );
            self.text(
                format!("{check_subject} / no inherit"),
                check.no_inherit().to_string(),
            );
            self.facts.insert(
                format!("{check_subject} / expression"),
                Value::Expr(Box::new(stored)),
            );
        }
        Ok(())
    }

    /// Records a table's foreign keys, keyed by the name the constraint
    /// holds.
    fn foreign_keys(&mut self, subject: &str, table: &DB::Table) -> Result<(), DescribeError> {
        let database = self.database;
        for key in table.foreign_keys(database)? {
            let constraint_name = key.constraint_name(database)?.map(Cow::into_owned);
            let name = constraint_name
                .as_deref()
                .unwrap_or_else(|| key.foreign_key_name().unwrap_or("<unnamed>"));
            let key_subject = format!("{subject} / foreign key {name}");
            self.text(
                format!("{key_subject} / constraint name"),
                format!("{constraint_name:?}"),
            );
            let referenced = key.referenced_table(database)?;
            let (schema, referenced_name) = self.table_name(referenced);
            let columns = |columns: Vec<&DB::Column>| {
                columns
                    .iter()
                    .map(|column| column.stored_column_name().into_owned())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            self.text(
                format!("{key_subject} / columns"),
                columns(key.host_columns(database)?.collect()),
            );
            self.text(
                format!("{key_subject} / references"),
                format!("{schema}.{referenced_name}"),
            );
            self.text(
                format!("{key_subject} / referenced columns"),
                columns(key.referenced_columns(database)?.collect()),
            );
            self.text(
                format!("{key_subject} / on delete"),
                key.on_delete(database).to_string(),
            );
            self.text(
                format!("{key_subject} / on update"),
                key.on_update(database).to_string(),
            );
            self.text(
                format!("{key_subject} / match"),
                format!("{:?}", key.match_kind(database)),
            );
            self.text(
                format!("{key_subject} / validated"),
                key.is_validated(database)?.to_string(),
            );
        }
        Ok(())
    }

    /// Records a table's indexes and unique indexes, the unique ones keyed
    /// by the name their constraint holds.
    fn indexes(
        &mut self,
        schema: &str,
        table_name: &str,
        subject: &str,
        table: &DB::Table,
    ) -> Result<(), DescribeError> {
        let database = self.database;
        for index in table.unique_indices(database)? {
            let constraint_name = index.constraint_name(database)?.map(Cow::into_owned);
            let name = constraint_name
                .as_deref()
                .unwrap_or_else(|| index.name().unwrap_or("<unnamed>"));
            let index_subject = format!("{subject} / index {name}");
            self.text(
                format!("{index_subject} / constraint name"),
                format!("{constraint_name:?}"),
            );
            self.text(
                format!("{index_subject} / primary key"),
                index.is_primary_key(database)?.to_string(),
            );
        }
        for index in table.indices(database)? {
            let name = index.name().unwrap_or("<unnamed>").to_owned();
            let index_subject = format!("{subject} / index {name}");
            self.text(
                format!("{index_subject} / unique"),
                index.is_unique().to_string(),
            );
            self.text(
                format!("{index_subject} / nulls distinct"),
                index.nulls_distinct().to_string(),
            );
            self.text(
                format!("{index_subject} / method"),
                index
                    .access_method()
                    .map_or_else(|| "btree".to_owned(), ToString::to_string),
            );
            self.text(
                format!("{index_subject} / columns"),
                index
                    .columns(database)?
                    .map(|column| column.stored_column_name().into_owned())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            self.text(
                format!("{index_subject} / include"),
                index
                    .include_columns(database)?
                    .map(|column| column.stored_column_name().into_owned())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            if let Some(predicate) = index.predicate() {
                let stored = self.stored.expression(
                    schema,
                    table_name,
                    Slot::IndexPredicate,
                    &predicate.to_string(),
                )?;
                self.facts.insert(
                    format!("{index_subject} / predicate"),
                    Value::Expr(Box::new(stored)),
                );
            }
        }
        Ok(())
    }

    /// Records a table's triggers, keyed by name.
    fn triggers(&mut self, subject: &str, table: &DB::Table) -> Result<(), DescribeError> {
        let database = self.database;
        for trigger in table.triggers(database)? {
            let trigger_subject = format!("{subject} / trigger {}", trigger.name());
            self.text(
                format!("{trigger_subject} / timing"),
                format!("{:?}", trigger.timing()),
            );
            self.text(
                format!("{trigger_subject} / events"),
                set(trigger.events().iter().map(ToString::to_string)),
            );
            self.text(
                format!("{trigger_subject} / orientation"),
                format!("{:?}", trigger.orientation()),
            );
            self.text(
                format!("{trigger_subject} / function"),
                format!("{:?}", trigger.function_name()),
            );
        }
        Ok(())
    }

    /// Records a table's policies, keyed by name.
    fn policies(
        &mut self,
        schema: &str,
        table_name: &str,
        subject: &str,
        table: &DB::Table,
    ) -> Result<(), DescribeError> {
        let database = self.database;
        for policy in table.policies(database)? {
            let policy_subject = format!("{subject} / policy {}", policy.name());
            self.text(
                format!("{policy_subject} / command"),
                format!("{:?}", policy.command()),
            );
            self.text(
                format!("{policy_subject} / type"),
                format!("{:?}", policy.policy_type()),
            );
            self.text(
                format!("{policy_subject} / public"),
                policy.applies_to_public().to_string(),
            );
            self.text(
                format!("{policy_subject} / roles"),
                set(policy.roles(database).map(ToString::to_string)),
            );
            for (slot, label, expression) in [
                (
                    Slot::PolicyUsing,
                    "using",
                    policy.using_expression(database),
                ),
                (
                    Slot::PolicyCheck,
                    "with check",
                    policy.check_expression(database),
                ),
            ] {
                if let Some(expression) = expression {
                    let stored = self.stored.expression(
                        schema,
                        table_name,
                        slot,
                        &expression.to_string(),
                    )?;
                    self.facts.insert(
                        format!("{policy_subject} / {label}"),
                        Value::Expr(Box::new(stored)),
                    );
                }
            }
        }
        Ok(())
    }

    /// Records plain and materialized views.
    fn views(&mut self) -> Result<(), DescribeError> {
        let database = self.database;
        for view in database.views() {
            self.view("view", view)?;
        }
        for view in database.materialized_views() {
            self.view("materialized view", view)?;
        }
        Ok(())
    }

    /// Records one plain or materialized view.
    fn view<V: ViewLike<DB = DB>>(&mut self, kind: &str, view: &V) -> Result<(), DescribeError> {
        let (schema, name) = self.view_name(view);
        if !self.scope.covers_object(&schema, &name) {
            return Ok(());
        }
        let subject = format!("{kind} {schema}.{name}");
        let query = self.stored.view_query(&view.definition().to_string())?;
        self.facts.insert(
            format!("{subject} / definition"),
            Value::Query(Box::new(query)),
        );
        self.text(
            format!("{subject} / security barrier"),
            view.is_security_barrier().to_string(),
        );
        self.text(
            format!("{subject} / security invoker"),
            view.is_security_invoker().to_string(),
        );
        self.text(
            format!("{subject} / owner"),
            format!("{:?}", view.owner(self.database)?),
        );
        Ok(())
    }

    /// Records the functions in scope, keyed by name and argument types.
    fn functions(&mut self) -> Result<(), DescribeError> {
        let database = self.database;
        for function in database.functions() {
            let schema = function
                .target_name()
                .schema()
                .map_or_else(|| self.scope.default_schema.clone(), str::to_owned);
            if !self.scope.covers_object(&schema, &function.stored_name()) {
                continue;
            }
            let arguments = function.normalized_argument_type_names(database);
            let subject = format!(
                "function {schema}.{}({})",
                function.stored_name(),
                arguments.join(", ")
            );
            self.text(
                format!("{subject} / language"),
                format!("{:?}", function.stored_language()),
            );
            self.text(
                format!("{subject} / returns"),
                format!("{:?}", function.normalized_return_type_name(database)),
            );
            self.text(
                format!("{subject} / returns set"),
                function.returns_set().to_string(),
            );
            self.text(
                format!("{subject} / argument names"),
                format!(
                    "{:?}",
                    function.stored_argument_names(database).collect::<Vec<_>>()
                ),
            );
            self.text(
                format!("{subject} / argument modes"),
                format!("{:?}", function.argument_modes().collect::<Vec<_>>()),
            );
            self.text(
                format!("{subject} / argument defaults"),
                function.argument_default_count().to_string(),
            );
            self.text(
                format!("{subject} / kind"),
                format!("{:?}", function.kind(database)?),
            );
            self.text(
                format!("{subject} / volatility"),
                format!("{:?}", function.volatility()),
            );
            self.text(
                format!("{subject} / security"),
                format!("{:?}", function.security_mode()),
            );
            self.text(
                format!("{subject} / null input"),
                format!("{:?}", function.null_input_behavior()),
            );
            self.text(
                format!("{subject} / search path"),
                format!(
                    "{:?}",
                    function.search_path(database)?.map(|path| {
                        path.iter()
                            .map(|(schema, quoted)| {
                                normalize_identifier(schema, *quoted).into_owned()
                            })
                            .collect::<Vec<_>>()
                    })
                ),
            );
            self.text(
                format!("{subject} / body bound"),
                function.body_is_bound().to_string(),
            );
            if !function.body_is_bound() {
                self.text(
                    format!("{subject} / body"),
                    format!("{:?}", function.body()),
                );
            }
            self.function_references(&subject, function);
            self.text(
                format!("{subject} / executable by public"),
                function.is_executable_by_public(database)?.to_string(),
            );
            self.text(
                format!("{subject} / owner"),
                format!("{:?}", function.owner(database)?),
            );
        }
        Ok(())
    }

    /// Records the functions and tables a function's body reaches, or the
    /// kind of error a reader answers instead.
    fn function_references(&mut self, subject: &str, function: &DB::Function) {
        let database = self.database;
        let reached = |names: Result<Vec<String>, LookupError>| match names {
            Ok(names) => set(names),
            Err(error) => {
                let debug = format!("{error:?}");
                let variant = debug.split([' ', '(', '{']).next().unwrap_or_default();
                format!("unanswered: {variant}")
            }
        };
        let functions = reached(function.referenced_functions(database).map(|functions| {
            functions
                .into_iter()
                .map(|called| {
                    let target = called.target_name();
                    let schema = target.schema().map_or_else(
                        || self.scope.default_schema.clone(),
                        |schema| {
                            normalize_identifier(schema, target.schema_is_quoted()).into_owned()
                        },
                    );
                    format!("{schema}.{}", called.stored_name())
                })
                .collect()
        }));
        let tables = reached(function.referenced_tables(database).map(|tables| {
            tables
                .into_iter()
                .map(|table| {
                    let (schema, name) = self.table_name(table);
                    format!("{schema}.{name}")
                })
                .collect()
        }));
        self.text(format!("{subject} / referenced functions"), functions);
        self.text(format!("{subject} / referenced tables"), tables);
    }

    /// Records the roles in scope and their attributes.
    fn roles(&mut self) {
        for role in self.database.roles() {
            let name = role.stored_name().into_owned();
            if !self.scope.covers_role(&name) {
                continue;
            }
            let subject = format!("role {name}");
            self.text(
                format!("{subject} / attributes"),
                format!(
                    "login={} superuser={} createdb={} createrole={} bypassrls={} replication={} inherit={} connection limit={:?}",
                    role.can_login(),
                    role.is_superuser(),
                    role.can_create_db(),
                    role.can_create_role(),
                    role.can_bypass_rls(),
                    role.is_replication(),
                    role.inherits(),
                    role.connection_limit(),
                ),
            );
        }
    }

    /// Records whether each role in scope holds each privilege on each role
    /// in scope, itself included.
    fn role_privileges(&mut self) {
        let database = self.database;
        let roles = self.described_roles();
        for (member_name, member) in &roles {
            for (target_name, target) in &roles {
                for (privilege, keyword) in ROLE_PRIVILEGES {
                    self.text(
                        role_privilege_fact(member_name, keyword, target_name),
                        member.has_role(target, privilege, database).to_string(),
                    );
                }
            }
        }
    }

    /// Records where each role in scope holds each privilege on `table`.
    fn table_privileges(
        &mut self,
        schema: &str,
        name: &str,
        table: &DB::Table,
    ) -> Result<(), DescribeError> {
        let database = self.database;
        for (role_name, role) in self.described_roles() {
            let held = table.privileges(role, database)?;
            let by_column = [&held.select, &held.insert, &held.update, &held.references];
            for (keyword, privilege) in COLUMN_PRIVILEGES.into_iter().zip(by_column) {
                let value = match privilege {
                    ColumnPrivilege::Table => column_privilege_value(true, &[]),
                    ColumnPrivilege::Columns(columns) => column_privilege_value(
                        false,
                        &columns
                            .iter()
                            .map(|column| column.stored_column_name().into_owned())
                            .collect::<Vec<_>>(),
                    ),
                };
                self.text(
                    table_privilege_fact(schema, name, keyword, &role_name),
                    value,
                );
            }
            let whole = [held.delete, held.truncate, held.trigger, held.maintain];
            for (keyword, holds) in WHOLE_TABLE_PRIVILEGES.into_iter().zip(whole) {
                self.text(
                    table_privilege_fact(schema, name, keyword, &role_name),
                    holds.to_string(),
                );
            }
        }
        Ok(())
    }

    /// Records the row-level security each statement shape meets on `table`
    /// for each role in scope, in full and in the terms of the rewritten
    /// query.
    fn row_security(&mut self, schema: &str, name: &str, table: &DB::Table) {
        let database = self.database;
        for (role_name, role) in self.described_roles() {
            for (statement, label) in ROW_STATEMENTS {
                let (detail, rewritten) = match table.row_security(statement, role, database) {
                    Ok(security) => (
                        row_security_detail(&security),
                        rewritten_form(&security, statement),
                    ),
                    Err(error) => (format!("error: {error}"), format!("error: {error}")),
                };
                self.text(
                    format!("table {schema}.{name} / row security for role {role_name} on {label}"),
                    detail,
                );
                self.text(rewritten_fact(schema, name, &role_name, label), rewritten);
            }
        }
    }

    /// Records role memberships.
    fn memberships(&mut self) {
        use pg_diesel::sql_traits::traits::RoleMembershipLike;
        for membership in self.database.role_memberships() {
            let member = membership.stored_member_name().into_owned();
            let role = membership.stored_role_name().into_owned();
            if !self.scope.covers_role(&member) || !self.scope.covers_role(&role) {
                continue;
            }
            self.text(
                format!("membership {member} in {role}"),
                format!(
                    "admin={} inherit={} set={}",
                    membership.admin_option(),
                    membership.inherit_option(),
                    membership.set_option()
                ),
            );
        }
    }

    /// Records table and column grants, one fact per relation, grantee and
    /// privilege.
    fn grants(&mut self) -> Result<(), DescribeError> {
        let database = self.database;
        for grant in database.table_grants() {
            let grantees: Vec<String> = grant.grantees(database).map(ToString::to_string).collect();
            let privileges: Vec<String> = if grant.is_all_privileges() {
                vec!["ALL".to_owned()]
            } else {
                grant.privileges(database).map(privilege_keyword).collect()
            };
            for relation in grant.relations(database) {
                let relation_name = self.relation_name(&relation);
                if !self
                    .scope
                    .covers(relation_name.split('.').next().unwrap_or_default())
                {
                    continue;
                }
                for grantee in &grantees {
                    for privilege in &privileges {
                        self.text(
                            format!("grant {privilege} on {relation_name} to {grantee}"),
                            format!("grant option={}", grant.with_grant_option()),
                        );
                    }
                }
            }
        }
        for grant in database.column_grants() {
            let Some(relation) = grant.relation(database) else {
                continue;
            };
            let relation_name = self.relation_name(&relation);
            if !self
                .scope
                .covers(relation_name.split('.').next().unwrap_or_default())
            {
                continue;
            }
            let columns: Vec<String> = match grant.table(database) {
                Some(table) => grant
                    .columns(table, database)?
                    .map(|column| column.stored_column_name().into_owned())
                    .collect(),
                None => Vec::new(),
            };
            let grantees: Vec<String> = grant.grantees(database).map(ToString::to_string).collect();
            let privileges: Vec<String> =
                grant.privileges(database).map(privilege_keyword).collect();
            for column in &columns {
                for grantee in &grantees {
                    for privilege in &privileges {
                        self.text(
                            format!("grant {privilege} on {relation_name}.{column} to {grantee}"),
                            format!("grant option={}", grant.with_grant_option()),
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// Records the sequences in scope.
    fn sequences(&mut self) {
        let database = self.database;
        for sequence in database.sequences() {
            let schema = self
                .scope
                .schema(sequence.stored_sequence_schema())
                .into_owned();
            if !self
                .scope
                .covers_object(&schema, &sequence.stored_sequence_name())
            {
                continue;
            }
            let subject = format!("sequence {schema}.{}", sequence.stored_sequence_name());
            self.text(
                format!("{subject} / origin"),
                format!("{:?}", sequence.origin()),
            );
            let owner = sequence.owner(database).map(|owner| match owner {
                SequenceOwner::Column(column) => {
                    let (schema, name) = self.table_name(column.table(database));
                    format!("{schema}.{name}.{}", column.stored_column_name())
                }
                SequenceOwner::ViewColumn { view, ordinal } => {
                    let (schema, name) = self.view_name(view);
                    format!("{schema}.{name} column {ordinal}")
                }
            });
            self.text(format!("{subject} / owner"), format!("{owner:?}"));
        }
    }

    /// Records the rewrite rules on relations in scope, each brought into
    /// the form `PostgreSQL` stores.
    fn rules(&mut self) -> Result<(), DescribeError> {
        let database = self.database;
        for rule in database.rules() {
            let (schema, relation) = self.relation_parts(&rule.relation(database)?);
            if !self.scope.covers_object(&schema, &relation) {
                continue;
            }
            let subject = format!("rule {} on {schema}.{relation}", rule.name());
            let written = format!(
                "CREATE RULE pgdiff_rule AS ON {} TO {}.{}{} DO {} {}",
                rule.event(),
                quoted(&schema),
                quoted(&relation),
                rule.condition()
                    .map_or_else(String::new, |condition| format!(" WHERE {condition}")),
                if rule.is_instead() { "INSTEAD" } else { "ALSO" },
                rule.action(),
            );
            let stored = self.stored.rule(&written)?;
            self.text(format!("{subject} / event"), stored.event.to_string());
            self.text(
                format!("{subject} / instead"),
                rule.is_instead().to_string(),
            );
            if let Some(condition) = stored.condition {
                self.facts.insert(
                    format!("{subject} / condition"),
                    Value::Expr(Box::new(condition)),
                );
            }
            self.text(format!("{subject} / action"), stored.action.to_string());
        }
        Ok(())
    }

    /// Records the extensions.
    fn extensions(&mut self) {
        for extension in self.database.extensions() {
            let schema = self.scope.schema(extension.stored_schema()).into_owned();
            self.text(
                format!("extension {}", extension.stored_name()),
                format!("schema={schema}"),
            );
        }
    }

    /// Records the domains in scope.
    fn domains(&mut self) {
        let database = self.database;
        for domain in database.domains() {
            let schema = self.scope.schema(domain.stored_schema()).into_owned();
            if !self.scope.covers_object(&schema, &domain.stored_name()) {
                continue;
            }
            let subject = format!("domain {schema}.{}", domain.stored_name());
            self.data_type(&format!("{subject} / type"), domain.data_type(database));
        }
    }

    /// Records the user-defined types in scope and the form each takes.
    fn user_defined_types(&mut self) {
        let database = self.database;
        for type_ in database.user_defined_types() {
            let schema = self.scope.schema(type_.stored_schema()).into_owned();
            let name = type_.stored_name().into_owned();
            if !self.scope.covers_object(&schema, &name) {
                continue;
            }
            self.text(
                format!("user-defined type {schema}.{name} / kind"),
                format!("{:?}", type_.kind()),
            );
        }
    }
}

/// Describes `database` within `scope`, bringing every expression into its
/// stored form through `stored`.
///
/// # Errors
///
/// Returns a [`DescribeError`] when a trait reader fails or an expression
/// cannot be brought into its stored form.
pub fn describe<DB: DatabaseLike>(
    database: &DB,
    scope: &Scope,
    stored: &mut dyn StoredForm,
) -> Result<Facts, DescribeError> {
    let mut describer = Describer {
        database,
        scope,
        stored,
        facts: Facts::new(),
    };
    describer.describe()?;
    Ok(describer.facts)
}
