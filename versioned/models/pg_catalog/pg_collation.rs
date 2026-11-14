//! `PostgreSQL` collation catalog model.

/// Represents a row from the `pg_catalog.pg_collation` table.
#[derive(diesel::Queryable, diesel::QueryableByName, diesel::Selectable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[diesel(table_name = crate::schema::pg_catalog::pg_collation::pg_collation)]
pub struct PgCollation {
    /// OID of the collation (primary key).
    pub oid: u32,
    /// Name of the collation.
    pub collname: String,
    /// OID of the namespace containing this collation.
    pub collnamespace: u32,
    /// OID of the owner of this collation.
    pub collowner: u32,
    /// Provider of the collation (c=libc, i=icu, d=database default).
    pub collprovider: String,
    /// Whether the collation is deterministic.
    pub collisdeterministic: bool,
    /// Encoding for which this collation is applicable (-1 for any).
    pub collencoding: i32,
    /// `LC_COLLATE` setting for this collation.
    pub collcollate: Option<String>,
    /// `LC_CTYPE` setting for this collation.
    pub collctype: Option<String>,
    /// ICU locale ID for this collation object (`PostgreSQL` 15-16).
    /// Renamed to `colllocale` in `PostgreSQL` 17.
    #[cfg(any(feature = "postgres-15", feature = "postgres-16"))]
    pub colliculocale: Option<String>,
    /// Collation provider locale name for this collation object (`PostgreSQL` 17+).
    /// Renamed from `colliculocale` in `PostgreSQL` 17.
    #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
    pub colllocale: Option<String>,
    /// ICU rules for this collation.
    #[cfg(not(any(feature = "postgres-15", feature = "postgres-14")))]
    pub collicurules: Option<String>,
    /// Provider-specific version string.
    pub collversion: Option<String>,
}
