//! Submodule providing the `PgRule` struct representing a row of the
//! `pg_rules` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_rules` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_rules::pg_rules)]
pub struct PgRule {
    /// Schema name.
    pub schemaname: Option<String>,
    /// Table name.
    pub tablename: Option<String>,
    /// Rule name.
    pub rulename: Option<String>,
    /// SQL definition of the rule.
    pub definition: Option<String>,
}
