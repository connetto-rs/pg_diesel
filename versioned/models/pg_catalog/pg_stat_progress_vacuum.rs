//! Submodule providing the `PgStatProgressVacuum` struct representing a row of
//! the `pg_stat_progress_vacuum` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_stat_progress_vacuum` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_stat_progress_vacuum::pg_stat_progress_vacuum)]
pub struct PgStatProgressVacuum {
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
    /// Total heap blocks.
    pub heap_blks_total: Option<i64>,
    /// Heap blocks scanned.
    pub heap_blks_scanned: Option<i64>,
    /// Heap blocks vacuumed.
    pub heap_blks_vacuumed: Option<i64>,
    /// Index vacuum cycles.
    pub index_vacuum_count: Option<i64>,
    /// Max dead tuple bytes.
    /// Renamed from `max_dead_tuples` in `PostgreSQL` 17.
    #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
    pub max_dead_tuple_bytes: Option<i64>,
    /// Max dead tuples.
    /// Renamed to `max_dead_tuple_bytes` in `PostgreSQL` 17.
    #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
    pub max_dead_tuples: Option<i64>,
    /// Current dead tuple bytes.
    /// Added in `PostgreSQL` 17.
    #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
    pub dead_tuple_bytes: Option<i64>,
    /// Dead tuples count.
    /// Renamed to `num_dead_item_ids` in `PostgreSQL` 17.
    #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
    pub num_dead_tuples: Option<i64>,
    /// Dead item identifiers.
    /// Renamed from `num_dead_tuples` in `PostgreSQL` 17.
    #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
    pub num_dead_item_ids: Option<i64>,
    /// Total indexes.
    /// Added in `PostgreSQL` 17.
    #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
    pub indexes_total: Option<i64>,
    /// Indexes processed.
    /// Added in `PostgreSQL` 17.
    #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
    pub indexes_processed: Option<i64>,
    /// Delay time (milliseconds).
    /// Added in `PostgreSQL` 18.
    #[cfg(feature = "postgres-18")]
    pub delay_time: Option<f64>,
}
