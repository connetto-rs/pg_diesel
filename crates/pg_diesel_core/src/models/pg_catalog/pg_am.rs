//! `PostgreSQL` access methods catalog model.

/// Represents a row from the `pg_catalog.pg_am` table.
#[derive(diesel::Queryable, diesel::QueryableByName, diesel::Selectable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[diesel(table_name = crate::schema::pg_catalog::pg_am::pg_am)]
pub struct PgAm {
    /// OID of the access method.
    pub oid: u32,
    /// Name of the access method.
    pub amname: String,
    /// OID of handler function for this access method.
    pub amhandler: u32,
    /// Type of access method (i=index, t=table).
    pub amtype: String,
}
