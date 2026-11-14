//! Submodule
use diesel::{
    Selectable,
    prelude::{Queryable, QueryableByName},
};

/// Represents a row in the `pg_stat_statements` view, which contains
/// performance statistics for all SQL statements executed by `PostgreSQL`.
#[derive(Queryable, QueryableByName, Selectable, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_stat_statements::pg_stat_statements)]
pub struct PgStatStatement {
    /// OID of the user who executed the statement.
    pub userid: u32,
    /// OID of the database in which the statement was executed.
    pub dbid: u32,
    /// True if the query was executed as a top-level statement
    /// (always true if `pg_stat_statements.track` is set to `top`).
    pub toplevel: bool,
    /// Hash code to identify identical normalized queries.
    pub queryid: i64,
    /// Text of a representative normalized SQL statement.
    pub query: String,
    /// Number of times the statement was executed.
    pub calls: i64,
    /// Number of times the statement was planned.
    /// (Zero if `pg_stat_statements.track_planning` is disabled.)
    pub plans: i64,
    /// Total time spent planning the statement (in milliseconds).
    /// Zero if planning statistics are disabled.
    pub total_plan_time: f64,
    /// Minimum time spent planning the statement (in milliseconds).
    /// May be zero if planning stats were reset or disabled.
    pub min_plan_time: f64,
    /// Maximum time spent planning the statement (in milliseconds).
    /// May be zero if planning stats were reset or disabled.
    pub max_plan_time: f64,
    /// Mean time spent planning the statement (in milliseconds).
    /// Zero if planning statistics are disabled.
    pub mean_plan_time: f64,
    /// Population standard deviation of planning time (in milliseconds).
    pub stddev_plan_time: f64,
    /// Total time spent executing the statement (in milliseconds).
    pub total_exec_time: f64,
    /// Minimum execution time for the statement (in milliseconds).
    /// May be zero until the statement is first executed after a reset.
    pub min_exec_time: f64,
    /// Maximum execution time for the statement (in milliseconds).
    /// May be zero until the statement is first executed after a reset.
    pub max_exec_time: f64,
    /// Mean execution time of the statement (in milliseconds).
    pub mean_exec_time: f64,
    /// Population standard deviation of execution time (in milliseconds).
    pub stddev_exec_time: f64,
    /// Total number of rows retrieved or affected by the statement.
    pub rows: i64,
    /// Total number of shared block cache hits by the statement.
    pub shared_blks_hit: i64,
    /// Total number of shared blocks read by the statement.
    pub shared_blks_read: i64,
    /// Total number of shared blocks dirtied by the statement.
    pub shared_blks_dirtied: i64,
    /// Total number of shared blocks written by the statement.
    pub shared_blks_written: i64,
    /// Total number of local block cache hits by the statement.
    pub local_blks_hit: i64,
    /// Total number of local blocks read by the statement.
    pub local_blks_read: i64,
    /// Total number of local blocks dirtied by the statement.
    pub local_blks_dirtied: i64,
    /// Total number of local blocks written by the statement.
    pub local_blks_written: i64,
    /// Total number of temp blocks read by the statement.
    pub temp_blks_read: i64,
    /// Total number of temp blocks written by the statement.
    pub temp_blks_written: i64,
    /// Total time spent reading shared blocks (in milliseconds).
    /// Requires `track_io_timing` to be enabled.
    pub shared_blk_read_time: f64,
    /// Total time spent writing shared blocks (in milliseconds).
    /// Requires `track_io_timing` to be enabled.
    pub shared_blk_write_time: f64,
    /// Total time spent reading local blocks (in milliseconds).
    /// Requires `track_io_timing` to be enabled.
    pub local_blk_read_time: f64,
    /// Total time spent writing local blocks (in milliseconds).
    /// Requires `track_io_timing` to be enabled.
    pub local_blk_write_time: f64,
}
