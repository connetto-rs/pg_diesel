//! Submodule providing the `PgRole` struct representing a row of the
//! `pg_roles` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

pub mod cached_queries;

/// Represents a row from the `pg_roles` view.
#[derive(
    Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_roles::pg_roles)]
pub struct PgRole {
    /// Role name.
    pub rolname: Option<String>,
    /// Whether superuser.
    pub rolsuper: Option<bool>,
    /// Whether inherits privileges.
    pub rolinherit: Option<bool>,
    /// Whether can create roles.
    pub rolcreaterole: Option<bool>,
    /// Whether can create databases.
    pub rolcreatedb: Option<bool>,
    /// Whether can login.
    pub rolcanlogin: Option<bool>,
    /// Whether can replicate.
    pub rolreplication: Option<bool>,
    /// Connection limit.
    pub rolconnlimit: Option<i32>,
    /// Hashed password.
    pub rolpassword: Option<String>,
    /// Password expiry time.
    pub rolvaliduntil: Option<std::time::SystemTime>,
    /// Whether can bypass RLS.
    pub rolbypassrls: Option<bool>,
    /// Role configuration.
    pub rolconfig: Option<Vec<String>>,
    /// OID of the role.
    pub oid: Option<u32>,
}

impl PgRole {
    /// Loads all roles from the `pg_roles` view.
    ///
    /// # Errors
    ///
    /// * If the database query fails.
    pub fn load_all(conn: &mut diesel::PgConnection) -> Result<Vec<Self>, diesel::result::Error> {
        cached_queries::load_all(conn)
    }

    /// Returns the name of the role.
    #[must_use]
    pub fn name(&self) -> &str {
        self.rolname.as_deref().unwrap_or("<unknown>")
    }
}
