//! The rule a `pg_rewrite` row becomes, with its definition parsed.
//!
//! A rule's condition and action live in `pg_rewrite` only as node trees,
//! which no catalog column spells as SQL. The engine's own rendering of the
//! rule, `pg_get_ruledef`, is a `CREATE RULE` statement, so it parses back
//! into the `CreateRule` the trait answers from. The statement renders
//! identifiers in canonical form, so an `ALSO` action comes back as a bare
//! `DO`, which reports `is_instead` as `false`, as `ALSO` asks.

use diesel::{
    ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl, dsl::sql, sql_types::Text,
};
use sqlparser::{
    ast::{CreateRule, CreateRuleAction, CreateRuleDoKind, CreateRuleEvent, Expr, Statement},
    dialect::PostgreSqlDialect,
    parser::Parser,
};

use crate::schema::pg_catalog::{
    pg_class::pg_class, pg_namespace::pg_namespace, pg_rewrite::pg_rewrite,
};

/// The relation a rule's `pg_rewrite` row points at, by its kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleRelationKind {
    /// A table, including a partitioned one.
    Table,
    /// A foreign table.
    ForeignTable,
    /// A plain view.
    View,
}

/// The catalog facts a rule row loads, the definition rendered by
/// `pg_get_ruledef`.
#[derive(Debug, diesel::Queryable)]
pub struct RuleRow {
    /// The OID of the rule.
    oid: u32,
    /// The stored name of the rule.
    rulename: String,
    /// The stored name of the relation the rule is defined on.
    relname: String,
    /// The stored name of the schema the relation lives in.
    nspname: String,
    /// The kind of the relation the rule is defined on.
    relkind: String,
    /// The rule's definition, rendered by `pg_get_ruledef`.
    definition: String,
}

/// What stopped a rule definition from parsing back into a `CreateRule`.
#[derive(Debug, thiserror::Error)]
pub enum RuleDefinitionError {
    /// `sqlparser` could not read the definition the catalog rendered.
    #[error("the definition of the rule {name} does not parse: {source}")]
    Unparsable {
        /// The name of the rule.
        name: String,
        /// The parse error.
        source: sqlparser::parser::ParserError,
    },
    /// The definition is not one `CREATE RULE` statement.
    #[error("the definition of the rule {name} is not one CREATE RULE statement")]
    UnexpectedShape {
        /// The name of the rule.
        name: String,
    },
}

/// A rewrite rule read from `pg_rewrite`, beside the parsed definition.
#[derive(Debug, Clone, PartialEq)]
pub struct PgRuleEntry {
    /// The OID of the rule.
    oid: u32,
    /// The stored name of the rule, in exact case.
    name: String,
    /// The stored name of the schema the rule's relation lives in.
    relation_schema: String,
    /// The stored name of the relation the rule is defined on.
    relation_name: String,
    /// The kind of the relation the rule is defined on.
    relation_kind: RuleRelationKind,
    /// The parsed definition the trait answers from.
    rule: CreateRule,
}

impl PgRuleEntry {
    /// Returns the name of the rule.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the schema the rule's relation lives in.
    #[must_use]
    pub fn relation_schema(&self) -> &str {
        &self.relation_schema
    }

    /// Returns the name of the relation the rule is defined on.
    #[must_use]
    pub fn relation_name(&self) -> &str {
        &self.relation_name
    }

    /// Returns the kind of the relation the rule is defined on.
    #[must_use]
    pub fn relation_kind(&self) -> RuleRelationKind {
        self.relation_kind
    }

    /// Returns the command the rule fires on.
    #[must_use]
    pub fn event(&self) -> CreateRuleEvent {
        self.rule.event
    }

    /// Returns whether the action replaces the original command.
    #[must_use]
    pub fn is_instead(&self) -> bool {
        matches!(self.rule.do_kind, Some(CreateRuleDoKind::Instead))
    }

    /// Returns the `WHERE` condition the rule fires under, if it has one.
    #[must_use]
    pub fn condition(&self) -> Option<&Expr> {
        self.rule.condition.as_ref()
    }

    /// Returns the action the rule performs.
    #[must_use]
    pub fn action(&self) -> &CreateRuleAction {
        &self.rule.action
    }

    /// Loads the rule rows of the relations the schemas `schemas` hold.
    ///
    /// A view's `_RETURN` rule is how the view stores its query and is not a
    /// rule a statement wrote, so it is left out.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the query fails.
    pub fn load_rows(
        schemas: &[String],
        conn: &mut PgConnection,
    ) -> Result<Vec<RuleRow>, diesel::result::Error> {
        pg_rewrite::table
            .inner_join(pg_class::table.on(pg_rewrite::ev_class.eq(pg_class::oid)))
            .inner_join(pg_namespace::table.on(pg_class::relnamespace.eq(pg_namespace::oid)))
            .filter(pg_rewrite::rulename.ne("_RETURN"))
            .filter(pg_class::relkind.eq_any(&["r", "v", "f", "p"]))
            .filter(pg_namespace::nspname.eq_any(schemas))
            .select((
                pg_rewrite::oid,
                pg_rewrite::rulename,
                pg_class::relname,
                pg_namespace::nspname,
                pg_class::relkind,
                sql::<Text>("pg_get_ruledef(pg_rewrite.oid)"),
            ))
            .load(conn)
    }

    /// Reads one rule row into an entry, parsing the definition it carries.
    ///
    /// # Errors
    ///
    /// Returns [`RuleDefinitionError::Unparsable`] when `sqlparser` cannot
    /// read the definition, and [`RuleDefinitionError::UnexpectedShape`]
    /// when it is not one `CREATE RULE` statement.
    pub fn new(row: RuleRow) -> Result<Self, RuleDefinitionError> {
        let mut statements =
            Parser::parse_sql(&PostgreSqlDialect {}, &row.definition).map_err(|source| {
                RuleDefinitionError::Unparsable {
                    name: row.rulename.clone(),
                    source,
                }
            })?;
        let rule = match statements.pop() {
            Some(Statement::CreateRule(rule)) if statements.is_empty() => rule,
            _ => return Err(RuleDefinitionError::UnexpectedShape { name: row.rulename }),
        };
        let kind = match row.relkind.as_str() {
            "v" => RuleRelationKind::View,
            "f" => RuleRelationKind::ForeignTable,
            _ => RuleRelationKind::Table,
        };
        Ok(Self {
            oid: row.oid,
            name: row.rulename,
            relation_schema: row.nspname,
            relation_name: row.relname,
            relation_kind: kind,
            rule,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(definition: &str) -> RuleRow {
        RuleRow {
            oid: 1,
            rulename: "rule".to_string(),
            relname: "relation".to_string(),
            nspname: "app".to_string(),
            relkind: "v".to_string(),
            definition: definition.to_string(),
        }
    }

    #[test]
    fn a_definition_the_parser_refuses_is_reported_with_the_complaint() {
        let error = PgRuleEntry::new(row("not a statement")).expect_err("the text does not parse");
        assert!(matches!(error, RuleDefinitionError::Unparsable { .. }));
    }

    #[test]
    fn a_statement_that_is_no_rule_is_reported_as_the_wrong_shape() {
        let error = PgRuleEntry::new(row("SELECT 1")).expect_err("the statement is no rule");
        assert!(matches!(error, RuleDefinitionError::UnexpectedShape { .. }));
    }

    fn entry_of(relkind: &str) -> PgRuleEntry {
        let row = RuleRow {
            relkind: relkind.to_string(),
            ..row("CREATE RULE keep AS ON SELECT TO app.relation DO INSTEAD NOTHING")
        };
        PgRuleEntry::new(row).expect("the definition parses back")
    }

    #[test]
    fn a_relation_the_database_does_not_hold_is_reported_not_found() {
        use sql_traits::{errors::LookupError, traits::RuleLike};

        use crate::PgDieselDatabase;
        use crate::dialect::PostgresDialect;

        for relkind in ["v", "f", "r"] {
            let entry = entry_of(relkind);
            let database = PgDieselDatabase::new("catalog".to_string(), PostgresDialect).into();
            let error =
                RuleLike::relation(&entry, &database).expect_err("the database holds no relation");
            assert!(
                matches!(error, LookupError::RelationNotFound { .. }),
                "relkind {relkind} reports the missing relation"
            );
        }
    }
}
