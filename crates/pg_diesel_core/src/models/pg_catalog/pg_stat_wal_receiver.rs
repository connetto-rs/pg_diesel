//! Submodule providing the `PgStatWalReceiver` struct representing a row of the
//! `pg_stat_wal_receiver` view in `PostgreSQL`.

use std::time::SystemTime;

use diesel::{Queryable, QueryableByName, Selectable, data_types::PgLsn};

/// Represents a row from the `pg_stat_wal_receiver` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[diesel(table_name = crate::schema::pg_catalog::pg_stat_wal_receiver::pg_stat_wal_receiver)]
pub struct PgStatWalReceiver {
    /// Process ID.
    pub pid: Option<i32>,
    /// Receiver status.
    pub status: Option<String>,
    /// Receive start LSN.
    pub receive_start_lsn: Option<PgLsn>,
    /// Receive start timeline.
    pub receive_start_tli: Option<i32>,
    /// Written LSN.
    pub written_lsn: Option<PgLsn>,
    /// Flushed LSN.
    pub flushed_lsn: Option<PgLsn>,
    /// Received timeline.
    pub received_tli: Option<i32>,
    /// Last message send time.
    pub last_msg_send_time: Option<SystemTime>,
    /// Last message receipt time.
    pub last_msg_receipt_time: Option<SystemTime>,
    /// Latest end LSN.
    pub latest_end_lsn: Option<PgLsn>,
    /// Latest end time.
    pub latest_end_time: Option<SystemTime>,
    /// Replication slot name.
    pub slot_name: Option<String>,
    /// Sender host.
    pub sender_host: Option<String>,
    /// Sender port.
    pub sender_port: Option<i32>,
    /// Connection info.
    pub conninfo: Option<String>,
}
