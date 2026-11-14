//! `PostgreSQL` access method support procedures catalog model.

/// Represents a row from the `pg_catalog.pg_amproc` table.
#[derive(diesel::Queryable, diesel::QueryableByName, diesel::Selectable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[diesel(table_name = crate::schema::pg_catalog::pg_amproc::pg_amproc)]
pub struct PgAmproc {
    /// OID of this entry.
    pub oid: u32,
    /// OID of operator family this procedure belongs to.
    pub amprocfamily: u32,
    /// Left-hand input data type, or zero if not type-specific.
    pub amproclefttype: u32,
    /// Right-hand input data type, or zero if not type-specific.
    pub amprocrighttype: u32,
    /// Support procedure number.
    pub amprocnum: i16,
    /// OID of the procedure.
    pub amproc: u32,
}
