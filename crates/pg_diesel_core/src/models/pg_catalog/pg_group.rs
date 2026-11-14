//! Submodule providing the `PgGroup` struct representing a row of the
//! `pg_group` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_group` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_group::pg_group)]
pub struct PgGroup {
    /// Name of the group role.
    pub groname: Option<String>,
    /// OID of the group role.
    pub grosysid: Option<u32>,
    /// Array of member role OIDs.
    pub grolist: Option<Vec<u32>>,
}
