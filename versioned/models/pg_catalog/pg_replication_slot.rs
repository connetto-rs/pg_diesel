//! Submodule providing the `PgReplicationSlot` struct representing a row of the
//! `pg_replication_slots` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable, data_types::PgLsn};

/// Represents a row from the `pg_replication_slots` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[diesel(table_name = crate::schema::pg_catalog::pg_replication_slots::pg_replication_slots)]
pub struct PgReplicationSlot {
    /// Slot name.
    pub slot_name: Option<String>,
    /// Output plugin name.
    pub plugin: Option<String>,
    /// Slot type.
    pub slot_type: Option<String>,
    /// Database OID.
    pub datoid: Option<u32>,
    /// Database name.
    pub database: Option<String>,
    /// Whether temporary.
    pub temporary: Option<bool>,
    /// Whether active.
    pub active: Option<bool>,
    /// Active session PID.
    pub active_pid: Option<i32>,
    /// Oldest transaction to retain.
    pub xmin: Option<u32>,
    /// Oldest catalog transaction to retain.
    pub catalog_xmin: Option<u32>,
    /// Restart LSN position.
    pub restart_lsn: Option<PgLsn>,
    /// Confirmed flush LSN position.
    pub confirmed_flush_lsn: Option<PgLsn>,
    /// WAL status.
    pub wal_status: Option<String>,
    /// Safe WAL size in bytes.
    pub safe_wal_size: Option<i64>,
    /// Whether two-phase commit enabled.
    pub two_phase: Option<bool>,
    /// Time since inactive.
    /// Added in `PostgreSQL` 17.
    #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
    pub inactive_since: Option<std::time::SystemTime>,
    /// Whether conflicting.
    #[cfg(not(any(feature = "postgres-15", feature = "postgres-14")))]
    pub conflicting: Option<bool>,
    /// Invalidation reason.
    /// Added in `PostgreSQL` 17.
    #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
    pub invalidation_reason: Option<String>,
    /// Whether failover enabled.
    /// Added in `PostgreSQL` 17.
    #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
    pub failover: Option<bool>,
    /// Whether synced from primary.
    /// Added in `PostgreSQL` 17.
    #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
    pub synced: Option<bool>,
    /// LSN at which two-phase commit decoding is enabled.
    /// Added in `PostgreSQL` 18.
    #[cfg(feature = "postgres-18")]
    pub two_phase_at: Option<PgLsn>,
}
