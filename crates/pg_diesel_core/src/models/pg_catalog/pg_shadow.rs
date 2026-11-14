//! Submodule providing the `PgShadow` struct representing a row of the
//! `pg_shadow` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_shadow` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_shadow::pg_shadow)]
pub struct PgShadow {
    /// User name.
    pub usename: Option<String>,
    /// User ID.
    pub usesysid: Option<u32>,
    /// Can create databases.
    pub usecreatedb: Option<bool>,
    /// Is superuser.
    pub usesuper: Option<bool>,
    /// Can replicate.
    pub userepl: Option<bool>,
    /// Can bypass RLS.
    pub usebypassrls: Option<bool>,
    /// Encrypted password.
    pub passwd: Option<String>,
    /// Password expiry.
    pub valuntil: Option<std::time::SystemTime>,
    /// Configuration settings.
    pub useconfig: Option<Vec<String>>,
}
