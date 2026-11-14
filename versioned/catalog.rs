//! The [`VersionCatalog`] of this `PostgreSQL` version.

use diesel::PgConnection;
use pg_diesel_core::{
    database::{ServerVersion, VersionCatalog},
    model_metadata::MembershipGrant,
};

/// The `PostgreSQL` major version this catalog reads.
#[cfg(feature = "postgres-14")]
pub const MAJOR: u32 = 14;
/// The `PostgreSQL` major version this catalog reads.
#[cfg(feature = "postgres-15")]
pub const MAJOR: u32 = 15;
/// The `PostgreSQL` major version this catalog reads.
#[cfg(feature = "postgres-16")]
pub const MAJOR: u32 = 16;
/// The `PostgreSQL` major version this catalog reads.
#[cfg(feature = "postgres-17")]
pub const MAJOR: u32 = 17;
/// The `PostgreSQL` major version this catalog reads.
#[cfg(feature = "postgres-18")]
pub const MAJOR: u32 = 18;

/// The catalog reads whose queries depend on the server's version, for a
/// server of major version [`MAJOR`].
#[derive(Debug, Clone, Copy, Default)]
pub struct Catalog;

impl VersionCatalog for Catalog {
    fn majors(&self) -> &'static [u32] {
        &[MAJOR]
    }

    #[cfg(not(any(feature = "postgres-15", feature = "postgres-14")))]
    fn membership_grants(
        &self,
        _version: ServerVersion,
        conn: &mut PgConnection,
    ) -> Result<Vec<MembershipGrant>, diesel::result::Error> {
        use crate::schema::pg_catalog::{pg_auth_members::pg_auth_members, pg_authid::pg_authid};
        use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};

        pg_auth_members::table
            .inner_join(pg_authid::table.on(pg_auth_members::member.eq(pg_authid::oid)))
            .select((
                pg_auth_members::roleid,
                pg_auth_members::member,
                pg_authid::rolname,
                pg_auth_members::admin_option,
                pg_auth_members::inherit_option,
                pg_auth_members::set_option,
            ))
            .load(conn)
    }

    #[cfg(any(feature = "postgres-15", feature = "postgres-14"))]
    fn membership_grants(
        &self,
        _version: ServerVersion,
        conn: &mut PgConnection,
    ) -> Result<Vec<MembershipGrant>, diesel::result::Error> {
        MembershipGrant::load_before_16(conn)
    }
}
