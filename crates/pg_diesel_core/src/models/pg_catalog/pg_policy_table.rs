//! Submodule providing the `PgPolicyTable` struct representing a row of the
//! `pg_policy` table in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

pub mod cached_queries;

/// Represents a row from the `pg_policy` table.
#[derive(
    Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_policy::pg_policy)]
pub struct PgPolicyTable {
    /// OID of the policy.
    pub oid: u32,
    /// Name of the policy.
    pub polname: String,
    /// OID of the table.
    pub polrelid: u32,
    /// Command type.
    pub polcmd: String,
    /// Whether the policy is permissive.
    pub polpermissive: bool,
    /// Role OIDs to which the policy applies.
    pub polroles: Vec<u32>,
    /// USING clause expression.
    pub polqual: Option<String>,
    /// WITH CHECK clause expression.
    pub polwithcheck: Option<String>,
}
