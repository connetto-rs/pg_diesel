//! Cached queries for `PgRole`.

use diesel::{PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::models::PgRole;

/// Loads all roles from the `pg_roles` view.
///
/// # Errors
///
/// Returns a `diesel::result::Error` if the query fails.
pub fn load_all(conn: &mut PgConnection) -> Result<Vec<PgRole>, diesel::result::Error> {
    use crate::schema::pg_catalog::pg_roles::pg_roles;

    pg_roles::table.select(PgRole::as_select()).load(conn)
}

/// Returns the role memberships for a given role.
///
/// # Errors
///
/// Returns a `diesel::result::Error` if the query fails.
pub fn member_of(
    role: &PgRole,
    conn: &mut PgConnection,
) -> Result<Vec<u32>, diesel::result::Error> {
    use crate::shared::schema::pg_catalog::pg_auth_members;
    use diesel::ExpressionMethods;

    // The role's OID might be None, in which case we can't find memberships
    let role_oid = role.oid.ok_or_else(|| diesel::result::Error::NotFound)?;

    pg_auth_members::table
        .filter(pg_auth_members::member.eq(role_oid))
        .select(pg_auth_members::roleid)
        .load(conn)
}
