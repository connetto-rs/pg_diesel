//! `PostgreSQL` authorization identifiers (roles) catalog model.

use std::time::SystemTime;

/// Represents a row from the `pg_catalog.pg_authid` table.
#[derive(diesel::Queryable, diesel::QueryableByName, diesel::Selectable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[diesel(table_name = crate::schema::pg_catalog::pg_authid::pg_authid)]
#[allow(clippy::struct_excessive_bools)]
pub struct PgAuthid {
    /// OID of the role (primary key).
    pub oid: u32,
    /// Name of the role.
    pub rolname: String,
    /// Whether role has superuser privileges.
    pub rolsuper: bool,
    /// Whether role inherits privileges of roles it is a member of.
    pub rolinherit: bool,
    /// Whether role can create other roles.
    pub rolcreaterole: bool,
    /// Whether role can create databases.
    pub rolcreatedb: bool,
    /// Whether role can log in (has login privilege).
    pub rolcanlogin: bool,
    /// Whether role can initiate streaming replication.
    pub rolreplication: bool,
    /// Whether role can bypass row-level security policies.
    pub rolbypassrls: bool,
    /// Maximum number of concurrent connections for this role (-1 = no limit).
    pub rolconnlimit: i32,
    /// Encrypted password for the role.
    pub rolpassword: Option<String>,
    /// Password expiry time.
    pub rolvaliduntil: Option<SystemTime>,
}
