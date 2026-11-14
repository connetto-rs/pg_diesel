//! Submodule providing the `PgStatBgwriter` struct representing a row of the
//! `pg_stat_bgwriter` view in `PostgreSQL`.

use std::time::SystemTime;

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_stat_bgwriter` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_stat_bgwriter::pg_stat_bgwriter)]
pub struct PgStatBgwriter {
    /// Number of scheduled checkpoints that have been performed.
    /// Only available in `PostgreSQL` 16 and earlier.
    #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
    pub checkpoints_timed: Option<i64>,
    /// Number of requested checkpoints that have been performed.
    /// Only available in `PostgreSQL` 16 and earlier.
    #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
    pub checkpoints_req: Option<i64>,
    /// Total amount of time that has been spent in the portion of checkpoint
    /// processing where files are written to disk, in milliseconds.
    /// Only available in `PostgreSQL` 16 and earlier.
    #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
    pub checkpoint_write_time: Option<f64>,
    /// Total amount of time that has been spent in the portion of checkpoint
    /// processing where files are synchronized to disk, in milliseconds.
    /// Only available in `PostgreSQL` 16 and earlier.
    #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
    pub checkpoint_sync_time: Option<f64>,
    /// Number of buffers written during checkpoints.
    /// Only available in `PostgreSQL` 16 and earlier.
    #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
    pub buffers_checkpoint: Option<i64>,
    /// Buffers written.
    pub buffers_clean: Option<i64>,
    /// Maxwritten clean stops.
    pub maxwritten_clean: Option<i64>,
    /// Number of buffers written directly by a backend.
    /// Only available in `PostgreSQL` 16 and earlier.
    #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
    pub buffers_backend: Option<i64>,
    /// Number of times a backend had to execute its own fsync call.
    /// Only available in `PostgreSQL` 16 and earlier.
    #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
    pub buffers_backend_fsync: Option<i64>,
    /// Buffers allocated.
    pub buffers_alloc: Option<i64>,
    /// Statistics reset time.
    pub stats_reset: Option<SystemTime>,
}
