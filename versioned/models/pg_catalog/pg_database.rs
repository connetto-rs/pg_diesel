//! Submodule providing the `PgDatabase` struct representing a row of the
//! `pg_database` table in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_database` table.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_database::pg_database)]
pub struct PgDatabase {
    /// OID of the database.
    pub oid: u32,
    /// Name of the database.
    pub datname: String,
    /// OID of the role that owns the database.
    pub datdba: u32,
    /// Character encoding for this database.
    pub encoding: i32,
    /// Locale provider (`PostgreSQL` 15+).
    #[cfg(not(feature = "postgres-14"))]
    pub datlocprovider: String,
    /// Whether this is a template database.
    pub datistemplate: bool,
    /// Whether connections to this database are allowed.
    pub datallowconn: bool,
    /// Whether login events are logged for this database.
    /// Added in `PostgreSQL` 17.
    #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
    pub dathasloginevt: bool,
    /// Maximum number of concurrent connections.
    pub datconnlimit: i32,
    /// All transaction IDs before this one have been replaced.
    pub datfrozenxid: u32,
    /// All multixact IDs before this one have been replaced.
    pub datminmxid: u32,
    /// Highest OID of any system object in this database (only in `PostgreSQL` 14).
    #[cfg(feature = "postgres-14")]
    pub datlastsysoid: u32,
    /// Default tablespace for this database.
    pub dattablespace: u32,
    /// `LC_COLLATE` setting for this database.
    pub datcollate: String,
    /// `LC_CTYPE` setting for this database.
    pub datctype: String,
    /// ICU locale ID for this database (`PostgreSQL` 15-16 only).
    /// Renamed to `datlocale` in `PostgreSQL` 17.
    #[cfg(any(feature = "postgres-15", feature = "postgres-16"))]
    pub daticulocale: Option<String>,
    /// Locale name if using ICU provider (`PostgreSQL` 17+).
    /// Renamed from `daticulocale` in `PostgreSQL` 17.
    #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
    pub datlocale: Option<String>,
    /// ICU collation rules if using ICU provider (`PostgreSQL` 16+).
    #[cfg(not(any(feature = "postgres-14", feature = "postgres-15")))]
    pub daticurules: Option<String>,
    /// Version of the collation.
    #[cfg(not(feature = "postgres-14"))]
    pub datcollversion: Option<String>,
}
