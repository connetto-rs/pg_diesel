//! Submodule providing the `PgShdescription` struct representing a row of the
//! `pg_shdescription` table in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_shdescription` table.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_shdescription::pg_shdescription)]
#[diesel(primary_key(objoid, classoid))]
pub struct PgShdescription {
    /// OID of the object.
    pub objoid: u32,
    /// OID of the system catalog.
    pub classoid: u32,
    /// Description text.
    pub description: String,
}
