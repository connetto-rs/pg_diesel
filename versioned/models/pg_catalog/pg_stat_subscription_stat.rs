//! Submodule providing the `PgStatSubscriptionStat` struct representing a row
//! of the `pg_stat_subscription_stats` view in `PostgreSQL`.

use std::time::SystemTime;

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_stat_subscription_stats` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_stat_subscription_stats::pg_stat_subscription_stats)]
pub struct PgStatSubscriptionStat {
    /// Subscription OID.
    pub subid: Option<u32>,
    /// Subscription name.
    pub subname: Option<String>,
    /// Apply error count.
    pub apply_error_count: Option<i64>,
    /// Sync error count.
    pub sync_error_count: Option<i64>,
    /// Statistics reset time.
    pub stats_reset: Option<SystemTime>,
}
