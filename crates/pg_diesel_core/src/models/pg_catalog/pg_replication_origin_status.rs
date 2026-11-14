//! Submodule providing the `PgReplicationOriginStatus` struct representing a
//! row of the `pg_replication_origin_status` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable, data_types::PgLsn};

/// Represents a row from the `pg_replication_origin_status` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[diesel(table_name = crate::schema::pg_catalog::pg_replication_origin_status::pg_replication_origin_status)]
pub struct PgReplicationOriginStatus {
    /// Internal OID of the replication origin.
    pub local_id: Option<u32>,
    /// External name of the origin.
    pub external_id: Option<String>,
    /// Remote LSN position.
    pub remote_lsn: Option<PgLsn>,
    /// Local LSN position.
    pub local_lsn: Option<PgLsn>,
}
