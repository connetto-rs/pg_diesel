//! Submodule providing the `PgInherit` struct representing a row of the
//! `pg_inherits` table in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_inherits` table.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_inherits::pg_inherits)]
pub struct PgInherit {
    /// OID of the child table.
    pub inhrelid: u32,
    /// OID of the parent table.
    pub inhparent: u32,
    /// Sequence number for multiple parents.
    pub inhseqno: i32,
    /// Whether the inheritance relationship is being detached.
    pub inhdetachpending: bool,
}
