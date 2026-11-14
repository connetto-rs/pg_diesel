//! `PostgreSQL` authorization identifier membership catalog model.

/// Represents a row from the `pg_catalog.pg_auth_members` table.
#[derive(diesel::Queryable, diesel::QueryableByName, diesel::Selectable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[diesel(table_name = crate::schema::pg_catalog::pg_auth_members::pg_auth_members)]
#[cfg(not(any(feature = "postgres-15", feature = "postgres-14")))]
pub struct PgAuthMembers {
    /// OID of this membership relationship (primary key).
    pub oid: u32,
    /// OID of the role that has members.
    pub roleid: u32,
    /// OID of the role that is a member.
    pub member: u32,
    /// OID of the role that granted this membership.
    pub grantor: u32,
    /// Whether the member can grant this role to others.
    pub admin_option: bool,
    /// Whether the member inherits privileges of this role.
    pub inherit_option: bool,
    /// Whether the member can SET ROLE to this role.
    pub set_option: bool,
}

/// Represents a row from the `pg_catalog.pg_auth_members` table.
#[derive(diesel::Queryable, diesel::QueryableByName, diesel::Selectable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[diesel(table_name = crate::schema::pg_catalog::pg_auth_members::pg_auth_members)]
#[cfg(any(feature = "postgres-15", feature = "postgres-14"))]
pub struct PgAuthMembers {
    /// OID of the role that has members.
    pub roleid: u32,
    /// OID of the role that is a member.
    pub member: u32,
    /// OID of the role that granted this membership.
    pub grantor: u32,
    /// Whether the member can grant this role to others.
    pub admin_option: bool,
}
