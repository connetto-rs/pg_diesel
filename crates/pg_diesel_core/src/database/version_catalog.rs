//! Submodule providing [`VersionCatalog`], the catalog reads whose queries
//! depend on the server's `PostgreSQL` version.

use diesel::PgConnection;

use crate::{database::ServerVersion, model_metadata::MembershipGrant};

/// The catalog reads whose queries depend on the server's `PostgreSQL`
/// version, which a [`PgDieselDatabaseBuilder`](super::PgDieselDatabaseBuilder)
/// takes from the catalogs of the versions it was compiled with.
pub trait VersionCatalog {
    /// Returns the `PostgreSQL` major versions this catalog reads.
    fn majors(&self) -> &'static [u32];

    /// Loads every grant `pg_auth_members` records on a server of
    /// `version`, one per grantor.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the query fails.
    fn membership_grants(
        &self,
        version: ServerVersion,
        conn: &mut PgConnection,
    ) -> Result<Vec<MembershipGrant>, diesel::result::Error>;
}
