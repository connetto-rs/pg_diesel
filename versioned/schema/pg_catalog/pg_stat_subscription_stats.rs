//! Submodule for the `pg_catalog.pg_stat_subscription_stats` view schema.

diesel::table! {
    /// `pg_catalog.pg_stat_subscription_stats` — view showing subscription error statistics.
    /// Each row represents one subscription showing error statistics for that subscription.
    /// Uses `subid` as a nominal primary key for Diesel compatibility.
    pg_catalog.pg_stat_subscription_stats (subid) {
        /// OID of the subscription.
        subid -> Nullable<Oid>,
        /// Name of the subscription.
        subname -> Nullable<Text>,
        /// Number of times an apply worker has errored.
        apply_error_count -> Nullable<BigInt>,
        /// Number of times a sync worker has errored.
        sync_error_count -> Nullable<BigInt>,
        #[cfg(feature = "postgres-18")]
        /// Number of times a row insertion violated a unique constraint.
        confl_insert_exists -> Nullable<BigInt>,
        #[cfg(feature = "postgres-18")]
        /// Number of times an update was applied to a row modified by another source.
        confl_update_origin_differs -> Nullable<BigInt>,
        #[cfg(feature = "postgres-18")]
        /// Number of times an update violated a unique constraint.
        confl_update_exists -> Nullable<BigInt>,
        #[cfg(feature = "postgres-18")]
        /// Number of times the tuple to be updated was not found.
        confl_update_missing -> Nullable<BigInt>,
        #[cfg(feature = "postgres-18")]
        /// Number of times a delete was applied to a row modified by another source.
        confl_delete_origin_differs -> Nullable<BigInt>,
        #[cfg(feature = "postgres-18")]
        /// Number of times the tuple to be deleted was not found.
        confl_delete_missing -> Nullable<BigInt>,
        #[cfg(feature = "postgres-18")]
        /// Number of times insertion/update violated multiple unique constraints.
        confl_multiple_unique_conflicts -> Nullable<BigInt>,
        /// Time at which these statistics were last reset.
        stats_reset -> Nullable<Timestamp>,
    }
}
