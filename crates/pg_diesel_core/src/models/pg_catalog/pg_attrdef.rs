//! `PostgreSQL` column default values catalog model.

/// Represents a row from the `pg_catalog.pg_attrdef` table.
#[derive(diesel::Queryable, diesel::QueryableByName, diesel::Selectable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[diesel(table_name = crate::schema::pg_catalog::pg_attrdef::pg_attrdef)]
pub struct PgAttrdef {
    /// OID of this entry (primary key).
    pub oid: u32,
    /// OID of the table this default belongs to.
    pub adrelid: u32,
    /// Column number that this default is for.
    pub adnum: i16,
    /// Default expression (in nodeToString representation).
    pub adbin: String,
}
