//! Submodule providing the `PgDefaultAcl` struct representing a row of the
//! `pg_default_acl` table in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_default_acl` table.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_default_acl::pg_default_acl)]
pub struct PgDefaultAcl {
    /// OID of this default ACL entry.
    pub oid: u32,
    /// OID of the role that these default privileges are for.
    pub defaclrole: u32,
    /// OID of the namespace these defaults apply to.
    pub defaclnamespace: u32,
    /// Type of object this entry is for.
    pub defaclobjtype: String,
}
