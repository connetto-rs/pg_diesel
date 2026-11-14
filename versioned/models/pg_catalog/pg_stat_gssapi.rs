//! Submodule providing the `PgStatGssapi` struct representing a row of the
//! `pg_stat_gssapi` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_stat_gssapi` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_stat_gssapi::pg_stat_gssapi)]
pub struct PgStatGssapi {
    /// Process ID.
    pub pid: Option<i32>,
    /// GSSAPI authenticated.
    pub gss_authenticated: Option<bool>,
    /// Principal name.
    pub principal: Option<String>,
    /// Connection encrypted.
    pub encrypted: Option<bool>,
    /// Credentials delegated.
    #[cfg(not(any(feature = "postgres-15", feature = "postgres-14")))]
    pub credentials_delegated: Option<bool>,
}
