//! Submodule providing the `PgPreparedStatement` struct representing a row of
//! the `pg_prepared_statements` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_prepared_statements` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_prepared_statements::pg_prepared_statements)]
pub struct PgPreparedStatement {
    /// Name of the prepared statement.
    pub name: Option<String>,
    /// SQL text of the statement.
    pub statement: Option<String>,
    /// Time when the statement was prepared.
    pub prepare_time: Option<std::time::SystemTime>,
    /// Parameter type OIDs.
    pub parameter_types: Option<Vec<u32>>,
    /// Result column type OIDs.
    #[cfg(not(any(feature = "postgres-15", feature = "postgres-14")))]
    pub result_types: Option<Vec<u32>>,
    /// Whether created via SQL PREPARE.
    pub from_sql: Option<bool>,
    /// Number of generic plan executions.
    pub generic_plans: Option<i64>,
    /// Number of custom plan executions.
    pub custom_plans: Option<i64>,
}
