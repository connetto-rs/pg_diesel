//! Submodule for the `pg_catalog.pg_stat_io` view schema.

diesel::table! {
    /// `pg_catalog.pg_stat_io` — view showing I/O statistics.
    /// Each row represents I/O statistics for a specific combination of backend type, object, and context.
    /// Uses `backend_type` as a nominal primary key for Diesel compatibility.
    pg_catalog.pg_stat_io (backend_type) {
        /// Type of backend (e.g., client backend, autovacuum worker).
        backend_type -> Nullable<Text>,
        /// Target object (e.g., relation, temp relation).
        object -> Nullable<Text>,
        /// I/O context (e.g., normal, vacuum, bulkread).
        context -> Nullable<Text>,
        /// Number of read operations.
        reads -> Nullable<BigInt>,
        /// Total number of bytes read.
        /// Added in `PostgreSQL` 18.
        #[cfg(feature = "postgres-18")]
        read_bytes -> Nullable<diesel::sql_types::Numeric>,
        /// Time spent reading, in milliseconds.
        read_time -> Nullable<Double>,
        /// Number of write operations.
        writes -> Nullable<BigInt>,
        /// Total number of bytes written.
        /// Added in `PostgreSQL` 18.
        #[cfg(feature = "postgres-18")]
        write_bytes -> Nullable<diesel::sql_types::Numeric>,
        /// Time spent writing, in milliseconds.
        write_time -> Nullable<Double>,
        /// Number of writeback operations.
        writebacks -> Nullable<BigInt>,
        /// Time spent in writeback, in milliseconds.
        writeback_time -> Nullable<Double>,
        /// Number of relation extend operations.
        extends -> Nullable<BigInt>,
        /// Total number of bytes extended.
        /// Added in `PostgreSQL` 18.
        #[cfg(feature = "postgres-18")]
        extend_bytes -> Nullable<diesel::sql_types::Numeric>,
        /// Time spent extending relations, in milliseconds.
        extend_time -> Nullable<Double>,
        /// Bytes per operation.
        /// Removed in `PostgreSQL` 18 in favor of `read_bytes`, `write_bytes`, `extend_bytes`.
        #[cfg(not(feature = "postgres-18"))]
        op_bytes -> Nullable<BigInt>,
        /// Number of buffer hits.
        hits -> Nullable<BigInt>,
        /// Number of evictions.
        evictions -> Nullable<BigInt>,
        /// Number of reuses.
        reuses -> Nullable<BigInt>,
        /// Number of fsync operations.
        fsyncs -> Nullable<BigInt>,
        /// Time spent in fsync, in milliseconds.
        fsync_time -> Nullable<Double>,
        /// Time at which these statistics were last reset.
        stats_reset -> Nullable<Timestamp>,
    }
}
