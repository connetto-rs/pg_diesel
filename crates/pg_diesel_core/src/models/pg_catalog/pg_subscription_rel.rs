//! Submodule providing the `PgSubscriptionRel` struct representing a row of the
//! `pg_subscription_rel` table in `PostgreSQL`.

use diesel::{Identifiable, Queryable, QueryableByName, Selectable, data_types::PgLsn};

/// Represents a row from the `pg_subscription_rel` table.
#[derive(
    Identifiable, Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash,
)]
#[diesel(table_name = crate::schema::pg_catalog::pg_subscription_rel::pg_subscription_rel)]
#[diesel(primary_key(srrelid, srsubid))]
pub struct PgSubscriptionRel {
    /// Subscription OID.
    pub srsubid: u32,
    /// Relation OID.
    pub srrelid: u32,
    /// Subscription state.
    pub srsubstate: String,
    /// Subscription LSN.
    pub srsublsn: Option<PgLsn>,
}
