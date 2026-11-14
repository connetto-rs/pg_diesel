//! The `DatabaseLike` catalog a `PostgreSQL` connection is read into, its
//! version-dependent reads taken from the `PostgreSQL` version the
//! `postgres-N` features select.

use diesel::PgConnection;
pub use pg_diesel_core::database::*;

use crate::model_metadata::MembershipGrant;

/// Builder for constructing a [`PgDieselDatabase`] instance from
/// `PostgreSQL` metadata.
pub type PgDieselDatabaseBuilder<'conn> =
    pg_diesel_core::database::PgDieselDatabaseBuilder<'conn, CompiledCatalog>;

/// The [`VersionCatalog`] of the `PostgreSQL` version the `postgres-N`
/// features select.
#[derive(Debug, Clone, Copy, Default)]
pub struct CompiledCatalog;

impl VersionCatalog for CompiledCatalog {
    fn membership_grants(
        &self,
        conn: &mut PgConnection,
    ) -> Result<Vec<MembershipGrant>, diesel::result::Error> {
        #[cfg(not(any(feature = "postgres-15", feature = "postgres-14")))]
        {
            use crate::schema::pg_catalog::{
                pg_auth_members::pg_auth_members, pg_authid::pg_authid,
            };
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
        MembershipGrant::load_before_16(conn)
    }
}
