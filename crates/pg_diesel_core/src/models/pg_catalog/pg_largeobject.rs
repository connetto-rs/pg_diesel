//! Submodule providing the `PgLargeobject` struct representing a row of the
//! `pg_largeobject` table in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_largeobject` table.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_largeobject::pg_largeobject)]
#[diesel(primary_key(loid, pageno))]
pub struct PgLargeobject {
    /// OID of the large object.
    pub loid: u32,
    /// Page number within the large object.
    pub pageno: i32,
    /// Data stored in this page.
    pub data: Vec<u8>,
}
