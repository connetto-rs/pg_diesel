//! `PostgreSQL` index catalog model.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents the `pg_index` system catalog table in `PostgreSQL`.
/// This table stores information about indexes on tables.
#[derive(
    Clone, Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Hash, Ord, PartialOrd,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_index::pg_index)]
#[allow(clippy::struct_excessive_bools)]
pub struct PgIndex {
    /// The OID of the index.
    pub indexrelid: u32,
    /// The OID of the table this index belongs to.
    pub indrelid: u32,
    /// The number of columns in the index (including expression columns).
    pub indnatts: i16,
    /// The number of key columns in the index (excluding non-key expressions).
    pub indnkeyatts: i16,
    /// `true` if the index enforces uniqueness.
    pub indisunique: bool,
    /// `true` if nulls are considered distinct in a unique index.
    /// Added in `PostgreSQL` 15.
    #[cfg(any(
        feature = "postgres-15",
        feature = "postgres-16",
        feature = "postgres-17",
        feature = "postgres-18"
    ))]
    pub indnullsnotdistinct: bool,
    /// `true` if this is the primary key index for the table.
    pub indisprimary: bool,
    /// `true` if this index enforces an exclusion constraint.
    pub indisexclusion: bool,
    /// `true` if this index enforces immediate constraints (not deferrable).
    pub indimmediate: bool,
    /// `true` if the index is the clustering index for the table.
    pub indisclustered: bool,
    /// `true` if the index is valid for use by queries.
    pub indisvalid: bool,
    /// `true` if the index's validity depends on the transaction snapshot.
    pub indcheckxmin: bool,
    /// `true` if the index is ready for inserts and queries.
    pub indisready: bool,
    /// `true` if the index is marked as live (not pending deletion).
    pub indislive: bool,
    /// `true` if the index is the replication identity index.
    pub indisreplident: bool,
    /// The column numbers of the indexed columns (0-based for expressions).
    pub indkey: Vec<i16>,
    /// The collation OIDs for indexed columns (0 if default collation is used).
    pub indcollation: Vec<u32>,
    /// The operator class OIDs for indexed columns.
    pub indclass: Vec<u32>,
    /// Per-column index options.
    pub indoption: Vec<i16>,
    /// Expression tree for the index (if it's an expression index).
    pub indexprs: Option<String>,
    /// Partial index predicate (if it's a partial index).
    pub indpred: Option<String>,
}
