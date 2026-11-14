//! Submodule providing the `PgStatArchiver` struct representing a row of the
//! `pg_stat_archiver` view in `PostgreSQL`.

use std::time::SystemTime;

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_stat_archiver` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_stat_archiver::pg_stat_archiver)]
pub struct PgStatArchiver {
    /// Archived WAL file count.
    pub archived_count: Option<i64>,
    /// Last archived WAL filename.
    pub last_archived_wal: Option<String>,
    /// Last successful archive time.
    pub last_archived_time: Option<SystemTime>,
    /// Failed archive attempts.
    pub failed_count: Option<i64>,
    /// Last failed WAL filename.
    pub last_failed_wal: Option<String>,
    /// Last failed archive time.
    pub last_failed_time: Option<SystemTime>,
    /// Statistics reset time.
    pub stats_reset: Option<SystemTime>,
}
