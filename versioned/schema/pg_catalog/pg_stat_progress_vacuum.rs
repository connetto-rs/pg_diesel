//! Submodule for the `pg_catalog.pg_stat_progress_vacuum` view schema.

diesel::table! {
    /// `pg_catalog.pg_stat_progress_vacuum` — view showing VACUUM command progress.
    /// Each row represents one backend running VACUUM showing progress information.
    /// Uses `pid` as a nominal primary key for Diesel compatibility.
    pg_catalog.pg_stat_progress_vacuum (pid) {
        /// Process ID of the backend running VACUUM.
        pid -> Nullable<Integer>,
        /// OID of the database this backend is connected to.
        datid -> Nullable<Oid>,
        /// Name of the database this backend is connected to.
        datname -> Nullable<Text>,
        /// OID of the table being vacuumed.
        relid -> Nullable<Oid>,
        /// Current processing phase.
        phase -> Nullable<Text>,
        /// Total number of heap blocks in the table.
        heap_blks_total -> Nullable<BigInt>,
        /// Number of heap blocks scanned.
        heap_blks_scanned -> Nullable<BigInt>,
        /// Number of heap blocks vacuumed.
        heap_blks_vacuumed -> Nullable<BigInt>,
        /// Number of completed index vacuum cycles.
        index_vacuum_count -> Nullable<BigInt>,
        /// Maximum bytes allowed for dead tuples before index vacuum.
        /// Renamed from `max_dead_tuples` in `PostgreSQL` 17.
        #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
        max_dead_tuple_bytes -> Nullable<BigInt>,
        /// Maximum dead tuples before index vacuum.
        /// Renamed to `max_dead_tuple_bytes` in `PostgreSQL` 17.
        #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
        max_dead_tuples -> Nullable<BigInt>,
        /// Current bytes of dead tuples.
        /// Added in `PostgreSQL` 17 (renamed from non-existent `dead_tuples`).
        #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
        dead_tuple_bytes -> Nullable<BigInt>,
        /// Number of dead tuples.
        /// Renamed to `num_dead_item_ids` in `PostgreSQL` 17.
        #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
        num_dead_tuples -> Nullable<BigInt>,
        /// Number of dead item identifiers.
        /// Renamed from `num_dead_tuples` in `PostgreSQL` 17.
        #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
        num_dead_item_ids -> Nullable<BigInt>,
        /// Total number of indexes on the table.
        /// Added in `PostgreSQL` 17.
        #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
        indexes_total -> Nullable<BigInt>,
        /// Number of indexes processed.
        /// Added in `PostgreSQL` 17.
        #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
        indexes_processed -> Nullable<BigInt>,
        /// Total time spent sleeping due to cost-based delay, in milliseconds.
        /// Added in `PostgreSQL` 18.
        #[cfg(feature = "postgres-18")]
        delay_time -> Nullable<Double>,
    }
}
