//! Submodule providing [`VersionCatalog`], the catalog reads whose queries
//! depend on the server's `PostgreSQL` version.

use diesel::PgConnection;

use crate::model_metadata::MembershipGrant;

/// The catalog reads whose queries depend on the server's `PostgreSQL`
/// version, which a [`PgDieselDatabaseBuilder`](super::PgDieselDatabaseBuilder)
/// takes from the catalog of the version it reads.
pub trait VersionCatalog {
    /// Loads every grant `pg_auth_members` records, one per grantor.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the query fails.
    fn membership_grants(
        &self,
        conn: &mut PgConnection,
    ) -> Result<Vec<MembershipGrant>, diesel::result::Error>;
}
