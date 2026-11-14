//! Submodule providing the `PgEventTrigger` struct representing a row of the
//! `pg_event_trigger` table in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_event_trigger` table.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_event_trigger::pg_event_trigger)]
pub struct PgEventTrigger {
    /// OID of the event trigger.
    pub oid: u32,
    /// Name of the event trigger.
    pub evtname: String,
    /// Event that this trigger fires on.
    pub evtevent: String,
    /// OID of the role that owns the event trigger.
    pub evtowner: u32,
    /// OID of the function to be called.
    pub evtfoid: u32,
    /// Firing mode.
    pub evtenabled: String,
    /// Command tags for which this trigger fires.
    pub evttags: Option<Vec<String>>,
}
