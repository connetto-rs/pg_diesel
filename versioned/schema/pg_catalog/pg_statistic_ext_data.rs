//! Submodule for the `pg_catalog.pg_statistic_ext_data` table schema.

#[cfg(feature = "postgres-14")]
diesel::table! {
    /// `pg_catalog.pg_statistic_ext_data` — table storing data for extended statistics objects (`PostgreSQL` 14).
    /// Each row represents statistics data for an extended statistics object.
    pg_catalog.pg_statistic_ext_data (stxoid) {
        /// The extended statistics object containing the definition for this data.
        stxoid -> Oid,
        /// N-distinct counts derived from the custom statistics.
        stxdndistinct -> Nullable<Binary>,
        /// Functional dependency statistics derived from the custom statistics.
        stxddependencies -> Nullable<Binary>,
        /// MCV (Most Common Values) list statistics derived from the custom statistics.
        stxdmcv -> Nullable<Binary>,
        /// Statistics for expressions.
        stxdexpr -> Nullable<Binary>,
    }
}

#[cfg(not(feature = "postgres-14"))]
diesel::table! {
    /// `pg_catalog.pg_statistic_ext_data` — table storing data for extended statistics objects (`PostgreSQL` 15+).
    /// Each row represents statistics data for an extended statistics object.
    /// Uses composite primary key (stxoid, stxdinherit).
    pg_catalog.pg_statistic_ext_data (stxoid, stxdinherit) {
        /// The extended statistics object containing the definition for this data.
        stxoid -> Oid,
        /// If true, the stats include values only from child tables, not the named table.
        stxdinherit -> Bool,
        /// N-distinct counts derived from the custom statistics.
        stxdndistinct -> Nullable<Binary>,
        /// Functional dependency statistics derived from the custom statistics.
        stxddependencies -> Nullable<Binary>,
        /// MCV (Most Common Values) list statistics derived from the custom statistics.
        stxdmcv -> Nullable<Binary>,
        /// Statistics for expressions.
        stxdexpr -> Nullable<Binary>,
    }
}
