//! Submodule providing the `PgForeignTable` struct representing a row of the
//! `pg_foreign_table` table in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_foreign_table` table.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_foreign_table::pg_foreign_table)]
pub struct PgForeignTable {
    /// OID of the `pg_class` entry for this foreign table.
    pub ftrelid: u32,
    /// OID of the foreign server.
    pub ftserver: u32,
    /// Foreign table options.
    pub ftoptions: Option<Vec<String>>,
}
