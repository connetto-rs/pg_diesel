//! Submodule providing the `PgPublicationRel` struct representing a row of the
//! `pg_publication_rel` table in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_publication_rel` table.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_publication_rel::pg_publication_rel)]
pub struct PgPublicationRel {
    /// OID of the mapping entry.
    pub oid: u32,
    /// OID of the publication.
    pub prpubid: u32,
    /// OID of the relation.
    pub prrelid: u32,
    /// Array of attribute numbers for column-level replication; `NULL` if all columns (`PostgreSQL` 15+).
    #[cfg(not(feature = "postgres-14"))]
    pub prattrs: Option<Vec<i16>>,
    /// Optional WHERE clause for row filtering (`PostgreSQL` 15+).
    #[cfg(not(feature = "postgres-14"))]
    pub prqual: Option<String>,
}
