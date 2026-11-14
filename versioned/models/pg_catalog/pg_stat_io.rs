//! Submodule providing the `PgStatIo` struct representing a row of the
//! `pg_stat_io` view in `PostgreSQL`.

use std::time::SystemTime;

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_stat_io` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_stat_io::pg_stat_io)]
pub struct PgStatIo {
    /// Backend type.
    pub backend_type: Option<String>,
    /// Target object.
    pub object: Option<String>,
    /// I/O context.
    pub context: Option<String>,
    /// Read operations.
    pub reads: Option<i64>,
    /// Total number of bytes read.
    /// Added in `PostgreSQL` 18.
    #[cfg(feature = "postgres-18")]
    pub read_bytes: Option<bigdecimal::BigDecimal>,
    /// Read time (milliseconds).
    pub read_time: Option<f64>,
    /// Write operations.
    pub writes: Option<i64>,
    /// Total number of bytes written.
    /// Added in `PostgreSQL` 18.
    #[cfg(feature = "postgres-18")]
    pub write_bytes: Option<bigdecimal::BigDecimal>,
    /// Write time (milliseconds).
    pub write_time: Option<f64>,
    /// Writeback operations.
    pub writebacks: Option<i64>,
    /// Writeback time (milliseconds).
    pub writeback_time: Option<f64>,
    /// Extend operations.
    pub extends: Option<i64>,
    /// Total number of bytes extended.
    /// Added in `PostgreSQL` 18.
    #[cfg(feature = "postgres-18")]
    pub extend_bytes: Option<bigdecimal::BigDecimal>,
    /// Extend time (milliseconds).
    pub extend_time: Option<f64>,
    /// Bytes per operation.
    /// Removed in `PostgreSQL` 18 in favor of `read_bytes`, `write_bytes`, `extend_bytes`.
    #[cfg(not(feature = "postgres-18"))]
    pub op_bytes: Option<i64>,
    /// Buffer hits.
    pub hits: Option<i64>,
    /// Evictions.
    pub evictions: Option<i64>,
    /// Reuses.
    pub reuses: Option<i64>,
    /// Fsync operations.
    pub fsyncs: Option<i64>,
    /// Fsync time (milliseconds).
    pub fsync_time: Option<f64>,
    /// Statistics reset time.
    pub stats_reset: Option<SystemTime>,
}
