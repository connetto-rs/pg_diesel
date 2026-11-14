//! Submodule providing the `PgInitPriv` struct representing a row of the
//! `pg_init_privs` table in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_init_privs` table.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_init_privs::pg_init_privs)]
pub struct PgInitPriv {
    /// OID of the specific object.
    pub objoid: u32,
    /// OID of the system catalog containing the object.
    pub classoid: u32,
    /// Object sub-ID.
    pub objsubid: i32,
    /// Type of privilege.
    pub privtype: String,
}
