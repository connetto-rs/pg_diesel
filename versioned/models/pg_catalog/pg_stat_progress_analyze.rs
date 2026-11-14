//! Submodule providing the `PgStatProgressAnalyze` struct representing a row of
//! the `pg_stat_progress_analyze` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_stat_progress_analyze` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_stat_progress_analyze::pg_stat_progress_analyze)]
pub struct PgStatProgressAnalyze {
    /// Process ID.
    pub pid: Option<i32>,
    /// Database OID.
    pub datid: Option<u32>,
    /// Database name.
    pub datname: Option<String>,
    /// Table OID.
    pub relid: Option<u32>,
    /// Current phase.
    pub phase: Option<String>,
    /// Sample blocks total.
    pub sample_blks_total: Option<i64>,
    /// Sample blocks scanned.
    pub sample_blks_scanned: Option<i64>,
    /// Extended stats total.
    pub ext_stats_total: Option<i64>,
    /// Extended stats computed.
    pub ext_stats_computed: Option<i64>,
    /// Child tables total.
    pub child_tables_total: Option<i64>,
    /// Child tables done.
    pub child_tables_done: Option<i64>,
    /// Current child table OID.
    pub current_child_table_relid: Option<u32>,
    /// Delay time (milliseconds).
    /// Added in `PostgreSQL` 18.
    #[cfg(feature = "postgres-18")]
    pub delay_time: Option<f64>,
}
