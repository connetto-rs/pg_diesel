//! Submodule providing the `PgStatUserFunction` struct representing a row of
//! the `pg_stat_user_functions` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_stat_user_functions` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_stat_user_functions::pg_stat_user_functions)]
pub struct PgStatUserFunction {
    /// Function OID.
    pub funcid: Option<u32>,
    /// Schema name.
    pub schemaname: Option<String>,
    /// Function name.
    pub funcname: Option<String>,
    /// Number of calls.
    pub calls: Option<i64>,
    /// Total time including called functions (milliseconds).
    pub total_time: Option<f64>,
    /// Self time excluding called functions (milliseconds).
    pub self_time: Option<f64>,
}
