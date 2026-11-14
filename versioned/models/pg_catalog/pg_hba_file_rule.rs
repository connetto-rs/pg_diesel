//! Submodule providing the `PgHbaFileRule` struct representing a row of the
//! `pg_hba_file_rules` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_hba_file_rules` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_hba_file_rules::pg_hba_file_rules)]
#[cfg(not(any(feature = "postgres-15", feature = "postgres-14")))]
pub struct PgHbaFileRule {
    /// Rule number.
    pub rule_number: Option<i32>,
    /// Name of the file containing this rule.
    pub file_name: Option<String>,
    /// Line number of this rule within the file.
    pub line_number: Option<i32>,
    /// Connection type.
    pub r#type: Option<String>,
    /// Array of database names this rule applies to.
    pub database: Option<Vec<String>>,
    /// Array of user names this rule applies to.
    pub user_name: Option<Vec<String>>,
    /// IP address or hostname.
    pub address: Option<String>,
    /// Network mask.
    pub netmask: Option<String>,
    /// Authentication method.
    pub auth_method: Option<String>,
    /// Authentication method options.
    pub options: Option<Vec<String>>,
    /// Error message if the rule is invalid.
    pub error: Option<String>,
}

/// Represents a row from the `pg_hba_file_rules` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_hba_file_rules::pg_hba_file_rules)]
#[cfg(any(feature = "postgres-15", feature = "postgres-14"))]
pub struct PgHbaFileRule {
    /// Line number of this rule within the file.
    pub line_number: Option<i32>,
    /// Connection type.
    pub r#type: Option<String>,
    /// Array of database names this rule applies to.
    pub database: Option<Vec<String>>,
    /// Array of user names this rule applies to.
    pub user_name: Option<Vec<String>>,
    /// IP address or hostname.
    pub address: Option<String>,
    /// Network mask.
    pub netmask: Option<String>,
    /// Authentication method.
    pub auth_method: Option<String>,
    /// Authentication method options.
    pub options: Option<Vec<String>>,
    /// Error message if the rule is invalid.
    pub error: Option<String>,
}
