//! Submodule providing the `PgLargeobjectMetadatum` struct representing a row
//! of the `pg_largeobject_metadata` table in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_largeobject_metadata` table.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_largeobject_metadata::pg_largeobject_metadata)]
pub struct PgLargeobjectMetadatum {
    /// OID of the large object.
    pub oid: u32,
    /// OID of the owner.
    pub lomowner: u32,
}
