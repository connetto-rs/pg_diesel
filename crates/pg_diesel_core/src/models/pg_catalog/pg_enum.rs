//! Submodule providing the `PgEnum` struct, which represents a `PostgreSQL`
//! enum type.
use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a `PostgreSQL` enum type.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_enum::pg_enum)]
pub struct PgEnum {
    /// The OID of the enum value.
    pub oid: u32,
    /// The OID of the enum type.
    pub enumtypid: u32,
    /// The sort order of the enum value.
    pub enumsortorder: f32,
    /// The label of the enum value.
    pub enumlabel: String,
}
