//! Submodule providing the `PgPublication` struct representing a row of the
//! `pg_publication` table in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_publication` table.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_publication::pg_publication)]
#[allow(clippy::struct_excessive_bools)]
pub struct PgPublication {
    /// OID of the publication.
    pub oid: u32,
    /// Name of the publication.
    pub pubname: String,
    /// OID of the owner.
    pub pubowner: u32,
    /// Whether all tables are included.
    pub puballtables: bool,
    /// Whether INSERT is published.
    pub pubinsert: bool,
    /// Whether UPDATE is published.
    pub pubupdate: bool,
    /// Whether DELETE is published.
    pub pubdelete: bool,
    /// Whether TRUNCATE is published.
    pub pubtruncate: bool,
    /// Whether publishing via root table.
    pub pubviaroot: bool,
    /// Whether generated columns are published.
    /// Added in `PostgreSQL` 18.
    #[cfg(feature = "postgres-18")]
    pub pubgencols: bool,
}
