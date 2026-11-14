//! Cached queries for `PgPolicyTable`.

use diesel::{PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::models::PgPolicyTable;

/// Returns the roles associated with the policy.
///
/// # Errors
///
/// Returns a `diesel::result::Error` if the query fails.
pub fn roles(
    policy: &PgPolicyTable,
    conn: &mut PgConnection,
) -> Result<Vec<crate::models::PgAuthid>, diesel::result::Error> {
    use crate::schema::pg_catalog::pg_authid::pg_authid;
    use diesel::ExpressionMethods;

    pg_authid::table
        .filter(pg_authid::oid.eq_any(&policy.polroles))
        .select(crate::models::PgAuthid::as_select())
        .load(conn)
}

/// Returns the `USING` and `WITH CHECK` expressions of the policy, as SQL.
///
/// `polqual` and `polwithcheck` hold the node tree `nodeToString` writes, which
/// is not SQL; `pg_get_expr` is the only way back to the text, and the DSL has
/// no term for it.
///
/// # Errors
///
/// Returns a `diesel::result::Error` if the query fails.
pub fn expressions(
    policy: &PgPolicyTable,
    conn: &mut PgConnection,
) -> Result<(Option<String>, Option<String>), diesel::result::Error> {
    use crate::schema::pg_catalog::pg_policy::pg_policy;
    use diesel::{
        ExpressionMethods,
        dsl::sql,
        sql_types::{Nullable, Text},
    };

    pg_policy::table
        .filter(pg_policy::oid.eq(policy.oid))
        .select((
            sql::<Nullable<Text>>("pg_get_expr(polqual, polrelid)"),
            sql::<Nullable<Text>>("pg_get_expr(polwithcheck, polrelid)"),
        ))
        .first::<(Option<String>, Option<String>)>(conn)
}
