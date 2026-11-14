//! Submodule for the `pg_catalog.pg_stat_bgwriter` view schema.

diesel::table! {
    /// `pg_catalog.pg_stat_bgwriter` — view showing background writer statistics.
    /// Contains a single row showing cluster-wide background writer statistics.
    /// Uses `buffers_clean` as a nominal primary key for Diesel compatibility.
    pg_catalog.pg_stat_bgwriter (buffers_clean) {
        /// Number of scheduled checkpoints that have been performed.
        /// Only available in `PostgreSQL` 16 and earlier.
        #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
        checkpoints_timed -> Nullable<BigInt>,
        /// Number of requested checkpoints that have been performed.
        /// Only available in `PostgreSQL` 16 and earlier.
        #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
        checkpoints_req -> Nullable<BigInt>,
        /// Total amount of time that has been spent in the portion of checkpoint
        /// processing where files are written to disk, in milliseconds.
        /// Only available in `PostgreSQL` 16 and earlier.
        #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
        checkpoint_write_time -> Nullable<Double>,
        /// Total amount of time that has been spent in the portion of checkpoint
        /// processing where files are synchronized to disk, in milliseconds.
        /// Only available in `PostgreSQL` 16 and earlier.
        #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
        checkpoint_sync_time -> Nullable<Double>,
        /// Number of buffers written during checkpoints.
        /// Only available in `PostgreSQL` 16 and earlier.
        #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
        buffers_checkpoint -> Nullable<BigInt>,
        /// Number of buffers written by the background writer.
        buffers_clean -> Nullable<BigInt>,
        /// Number of times the background writer stopped a cleaning scan because it had written too many buffers.
        maxwritten_clean -> Nullable<BigInt>,
        /// Number of buffers written directly by a backend.
        /// Only available in `PostgreSQL` 16 and earlier.
        #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
        buffers_backend -> Nullable<BigInt>,
        /// Number of times a backend had to execute its own fsync call.
        /// Only available in `PostgreSQL` 16 and earlier.
        #[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
        buffers_backend_fsync -> Nullable<BigInt>,
        /// Number of buffers allocated.
        buffers_alloc -> Nullable<BigInt>,
        /// Time at which these statistics were last reset.
        stats_reset -> Nullable<Timestamp>,
    }
}
