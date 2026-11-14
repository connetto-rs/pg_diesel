//! Submodule providing the `PgStatSubscription` struct representing a row of
//! the `pg_stat_subscription` view in `PostgreSQL`.

use std::time::SystemTime;

use diesel::{Queryable, QueryableByName, Selectable, data_types::PgLsn};

/// Represents a row from the `pg_stat_subscription` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[diesel(table_name = crate::schema::pg_catalog::pg_stat_subscription::pg_stat_subscription)]
pub struct PgStatSubscription {
    /// Subscription OID.
    pub subid: Option<u32>,
    /// Subscription name.
    pub subname: Option<String>,
    /// Worker type.
    /// Added in `PostgreSQL` 17.
    #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
    pub worker_type: Option<String>,
    /// Process ID.
    pub pid: Option<i32>,
    /// Leader process ID.
    #[cfg(not(any(feature = "postgres-15", feature = "postgres-14")))]
    pub leader_pid: Option<i32>,
    /// Relation OID.
    pub relid: Option<u32>,
    /// Received LSN.
    pub received_lsn: Option<PgLsn>,
    /// Last message send time.
    pub last_msg_send_time: Option<SystemTime>,
    /// Last message receipt time.
    pub last_msg_receipt_time: Option<SystemTime>,
    /// Latest end LSN.
    pub latest_end_lsn: Option<PgLsn>,
    /// Latest end time.
    pub latest_end_time: Option<SystemTime>,
}
