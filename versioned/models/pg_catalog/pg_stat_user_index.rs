//! Submodule providing the `PgStatUserIndex` struct representing a row of the
//! `pg_stat_user_indexes` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_stat_user_indexes` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_stat_user_indexes::pg_stat_user_indexes)]
pub struct PgStatUserIndex {
    /// Table OID.
    pub relid: Option<u32>,
    /// Index OID.
    pub indexrelid: Option<u32>,
    /// Schema name.
    pub schemaname: Option<String>,
    /// Table name.
    pub relname: Option<String>,
    /// Index name.
    pub indexrelname: Option<String>,
    /// Index scans count.
    pub idx_scan: Option<i64>,
    /// Last index scan time.
    #[cfg(not(any(feature = "postgres-15", feature = "postgres-14")))]
    pub last_idx_scan: Option<std::time::SystemTime>,
    /// Index entries read.
    pub idx_tup_read: Option<i64>,
    /// Tuples fetched.
    pub idx_tup_fetch: Option<i64>,
}
