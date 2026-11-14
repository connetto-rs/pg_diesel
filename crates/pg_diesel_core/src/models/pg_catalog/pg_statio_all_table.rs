//! Submodule providing the `PgStatioAllTable` struct representing a row of the
//! `pg_statio_all_tables` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_statio_all_tables` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_statio_all_tables::pg_statio_all_tables)]
pub struct PgStatioAllTable {
    /// Table OID.
    pub relid: Option<u32>,
    /// Schema name.
    pub schemaname: Option<String>,
    /// Table name.
    pub relname: Option<String>,
    /// Heap disk blocks read.
    pub heap_blks_read: Option<i64>,
    /// Heap buffer hits.
    pub heap_blks_hit: Option<i64>,
    /// Index disk blocks read.
    pub idx_blks_read: Option<i64>,
    /// Index buffer hits.
    pub idx_blks_hit: Option<i64>,
    /// TOAST disk blocks read.
    pub toast_blks_read: Option<i64>,
    /// TOAST buffer hits.
    pub toast_blks_hit: Option<i64>,
    /// TOAST index disk blocks read.
    pub tidx_blks_read: Option<i64>,
    /// TOAST index buffer hits.
    pub tidx_blks_hit: Option<i64>,
}
