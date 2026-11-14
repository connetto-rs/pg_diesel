//! Submodule providing the struct `PgAttribute`.
use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a `PostgreSQL` attribute (column) in a table.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Hash, Clone)]
#[diesel(table_name = crate::schema::pg_catalog::pg_attribute::pg_attribute)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::struct_excessive_bools)]
pub struct PgAttribute {
    /// The OID of the table this column belongs to.
    pub attrelid: u32,
    /// The name of the column.
    pub attname: String,
    /// The OID of the data type of the column.
    pub atttypid: u32,
    /// The length of the column (if fixed), otherwise -1.
    pub attlen: i16,
    /// The number of the column (its position in the table, starting at 1).
    pub attnum: i16,
    /// The offset of the column in the tuple (if cached), otherwise -1.
    /// Removed in `PostgreSQL` 18.
    #[cfg(not(feature = "postgres-18"))]
    pub attcacheoff: i32,
    /// Type-specific data about the column (e.g., precision for numeric types).
    pub atttypmod: i32,
    /// The number of dimensions if the column is an array type.
    #[cfg(not(any(
        feature = "postgres-16",
        feature = "postgres-17",
        feature = "postgres-18"
    )))]
    pub attndims: i32,
    /// The number of dimensions if the column is an array type.
    #[cfg(any(
        feature = "postgres-16",
        feature = "postgres-17",
        feature = "postgres-18"
    ))]
    pub attndims: i16,
    /// Whether the column is passed by value.
    pub attbyval: bool,
    /// The alignment requirement of the column.
    pub attalign: String,
    /// The storage strategy for the column.
    pub attstorage: String,
    /// Whether the column is marked as NOT NULL.
    pub attnotnull: bool,
    /// Whether the column has a default value.
    pub atthasdef: bool,
    /// Whether the column has a missing value.
    pub atthasmissing: bool,
    /// The identity type of the column (e.g., 'a' for always, 'd' for by
    /// default).
    pub attidentity: String,
    /// The generation expression for the column (if any).
    pub attgenerated: String,
    /// Whether the column has been dropped.
    pub attisdropped: bool,
    /// Whether the column is a local definition.
    pub attislocal: bool,
    #[cfg(not(any(
        feature = "postgres-16",
        feature = "postgres-17",
        feature = "postgres-18"
    )))]
    /// The number of times the column is inherited.
    pub attinhcount: i32,
    #[cfg(any(
        feature = "postgres-16",
        feature = "postgres-17",
        feature = "postgres-18"
    ))]
    /// The number of times the column is inherited.
    pub attinhcount: i16,
    /// The collation of the column.
    pub attcollation: u32,
    /// The statistics target for the column.
    #[cfg(not(any(
        feature = "postgres-16",
        feature = "postgres-17",
        feature = "postgres-18"
    )))]
    pub attstattarget: i32,
    /// The statistics target for the column.
    #[cfg(all(
        feature = "postgres-16",
        not(any(feature = "postgres-17", feature = "postgres-18"))
    ))]
    pub attstattarget: i16,
    /// The statistics target for the column, if one is set.
    #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
    pub attstattarget: Option<i16>,
    /// The column options (if any).
    pub attoptions: Option<Vec<String>>,
    /// The foreign data wrapper options for the column (if any).
    pub attfdwoptions: Option<Vec<String>>,
    /// Compression method for the column
    pub attcompression: String,
}
