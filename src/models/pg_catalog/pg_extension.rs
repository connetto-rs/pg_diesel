//! Submodule providing a struct [`PgExtension`] representing the `pg_extension`
//! table.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents the `pg_extension` system catalog table in `PostgreSQL`.
/// This table stores information about extensions.
#[derive(Queryable, QueryableByName, Selectable, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_extension::pg_extension)]
pub struct PgExtension {
    /// The OID of the extension.
    pub oid: u32,
    /// The name of the extension.
    pub extname: String,
    /// The OID of the owner of the extension.
    pub extowner: u32,
    /// The OID of the namespace that contains this extension.
    pub extnamespace: u32,
    /// Whether the extension is relocatable.
    pub extrelocatable: bool,
    /// The version of the extension.
    pub extversion: String,
    /// The schema of the extension.
    pub extconfig: Option<Vec<u32>>,
    /// The configuration settings for the extension.
    pub extcondition: Option<Vec<String>>,
}
