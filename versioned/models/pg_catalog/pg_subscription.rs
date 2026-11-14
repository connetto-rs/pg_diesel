//! Submodule providing the `PgSubscription` struct representing a row of the
//! `pg_subscription` table in `PostgreSQL`.

use diesel::{Identifiable, Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_subscription` table.
#[derive(
    Identifiable, Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash,
)]
#[diesel(table_name = crate::schema::pg_catalog::pg_subscription::pg_subscription)]
#[diesel(primary_key(oid))]
#[allow(clippy::struct_excessive_bools)]
pub struct PgSubscription {
    /// Object identifier.
    pub oid: u32,
    /// Database OID.
    pub subdbid: u32,
    /// Skip LSN (`PostgreSQL` 15+).
    #[cfg(not(feature = "postgres-14"))]
    pub subskiplsn: diesel::data_types::PgLsn,
    /// Subscription name.
    pub subname: String,
    /// Owner OID.
    pub subowner: u32,
    /// Enabled flag.
    pub subenabled: bool,
    /// Binary format flag.
    pub subbinary: bool,
    /// Stream mode.
    pub substream: String,
    /// Two-phase state (`PostgreSQL` 15+).
    #[cfg(not(feature = "postgres-14"))]
    pub subtwophasestate: String,
    /// Disable on error flag (`PostgreSQL` 15+).
    #[cfg(not(feature = "postgres-14"))]
    pub subdisableonerr: bool,
    /// Password required flag.
    #[cfg(not(any(feature = "postgres-15", feature = "postgres-14")))]
    pub subpasswordrequired: bool,
    /// Run as owner flag.
    #[cfg(not(any(feature = "postgres-15", feature = "postgres-14")))]
    pub subrunasowner: bool,
    /// Failover flag.
    /// Added in `PostgreSQL` 17.
    #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
    pub subfailover: bool,
    /// Connection info.
    pub subconninfo: String,
    /// Slot name.
    pub subslotname: Option<String>,
    /// Sync commit setting.
    pub subsynccommit: String,
    /// Publications.
    pub subpublications: Vec<String>,
    /// Origin name.
    #[cfg(not(any(feature = "postgres-15", feature = "postgres-14")))]
    pub suborigin: Option<String>,
}
