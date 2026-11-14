//! The `DatabaseLike` catalog a `PostgreSQL` connection is read into, its
//! version-dependent reads taken from the catalog of the server's version
//! among the ones the `postgres-N` features compiled in.

use diesel::PgConnection;
pub use pg_diesel_core::database::*;

use crate::model_metadata::MembershipGrant;

/// Builder for constructing a [`PgDieselDatabase`] instance from
/// `PostgreSQL` metadata of any server version this build compiled in.
pub type PgDieselDatabaseBuilder<'conn> =
    pg_diesel_core::database::PgDieselDatabaseBuilder<'conn, CompiledVersions>;

/// The `PostgreSQL` major versions this build compiled a catalog for.
const COMPILED: &[u32] = &[
    #[cfg(feature = "postgres-14")]
    pg_diesel_pg14::MAJOR,
    #[cfg(feature = "postgres-15")]
    pg_diesel_pg15::MAJOR,
    #[cfg(feature = "postgres-16")]
    pg_diesel_pg16::MAJOR,
    #[cfg(feature = "postgres-17")]
    pg_diesel_pg17::MAJOR,
    #[cfg(feature = "postgres-18")]
    pg_diesel_pg18::MAJOR,
];

/// The [`VersionCatalog`] that hands each read to the catalog of the
/// server's major version, among the ones the `postgres-N` features compiled
/// in.
#[derive(Debug, Clone, Copy, Default)]
pub struct CompiledVersions;

impl VersionCatalog for CompiledVersions {
    fn majors(&self) -> &'static [u32] {
        COMPILED
    }

    fn membership_grants(
        &self,
        version: ServerVersion,
        conn: &mut PgConnection,
    ) -> Result<Vec<MembershipGrant>, diesel::result::Error> {
        match version.major() {
            #[cfg(feature = "postgres-14")]
            pg_diesel_pg14::MAJOR => pg_diesel_pg14::Catalog.membership_grants(version, conn),
            #[cfg(feature = "postgres-15")]
            pg_diesel_pg15::MAJOR => pg_diesel_pg15::Catalog.membership_grants(version, conn),
            #[cfg(feature = "postgres-16")]
            pg_diesel_pg16::MAJOR => pg_diesel_pg16::Catalog.membership_grants(version, conn),
            #[cfg(feature = "postgres-17")]
            pg_diesel_pg17::MAJOR => pg_diesel_pg17::Catalog.membership_grants(version, conn),
            #[cfg(feature = "postgres-18")]
            pg_diesel_pg18::MAJOR => pg_diesel_pg18::Catalog.membership_grants(version, conn),
            major => unreachable!(
                "the builder refuses PostgreSQL {major}, which no catalog was compiled for"
            ),
        }
    }
}
