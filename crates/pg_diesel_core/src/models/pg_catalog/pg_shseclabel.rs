//! Submodule providing the `PgShseclabel` struct representing a row of the
//! `pg_shseclabel` table in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_shseclabel` table.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_shseclabel::pg_shseclabel)]
pub struct PgShseclabel {
    /// Object OID.
    pub objoid: u32,
    /// System catalog OID.
    pub classoid: u32,
    /// Provider name.
    pub provider: String,
    /// Security label.
    pub label: String,
}
