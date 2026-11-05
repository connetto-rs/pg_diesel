//! Submodule defining the cached queries methods used in the [`PgProc`] struct.

use diesel::{
    ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
    dsl::sql,
    sql_types::{Oid, Text},
};

use crate::models::{PgExtension, PgProc};

/// One row of a function's `pg_proc.proacl` as `aclexplode` reads it, or a
/// row without grantee for an ACL that grants nothing.
#[derive(diesel::QueryableByName, Debug, Clone, PartialEq, Eq)]
pub struct ProcAclEntry {
    /// The OID of the function the ACL belongs to.
    #[diesel(sql_type = diesel::sql_types::Oid)]
    pub function_oid: u32,
    /// The OID of the role holding `EXECUTE`, `0` for `PUBLIC`, or `None`
    /// when the ACL is empty.
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Oid>)]
    pub grantee: Option<u32>,
    /// Whether the grant was made `WITH GRANT OPTION`, `None` when the ACL
    /// is empty.
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Bool>)]
    pub grantable: Option<bool>,
}

/// Returns the extension that defines the procedure.
pub(super) fn extension(
    pg_proc: &PgProc,
    conn: &mut PgConnection,
) -> Result<PgExtension, diesel::result::Error> {
    use crate::schema::pg_catalog::{pg_depend::pg_depend, pg_extension::pg_extension};
    pg_extension::table
        .inner_join(pg_depend::table.on(pg_extension::oid.eq(pg_depend::refobjid)))
        .filter(pg_depend::objid.eq(pg_proc.oid))
        .select(PgExtension::as_select())
        .first::<PgExtension>(conn)
}

/// Loads every `pg_proc` row except the procedures, which `PostgreSQL` reads
/// as a different object kind and `sql-traits` has no kind for.
pub(super) fn load_all(conn: &mut PgConnection) -> Result<Vec<PgProc>, diesel::result::Error> {
    use crate::schema::pg_catalog::pg_proc::pg_proc;
    pg_proc::table
        .filter(pg_proc::prokind.ne("p")) // Exclude procedures
        .order_by(pg_proc::proname.asc())
        .then_order_by(pg_proc::oid.asc())
        .select(PgProc::as_select())
        .load::<PgProc>(conn)
}

/// Loads the `EXECUTE` grants of every function whose `pg_proc.proacl` is
/// set, at least one row per such function.
///
/// `proacl` is an `aclitem` array `diesel` cannot read, so each array goes
/// through `aclexplode` in a raw query, since the DSL cannot place a set
/// returning function in a `FROM`. A function whose `proacl` is `NULL` has
/// no row and holds the default the server starts it with, while an ACL
/// that grants nothing yields one row without grantee.
pub(super) fn load_all_execute_grants(
    conn: &mut PgConnection,
) -> Result<Vec<ProcAclEntry>, diesel::result::Error> {
    diesel::sql_query(
        r"
        SELECT
            pro.oid AS function_oid,
            acl.grantee AS grantee,
            acl.is_grantable AS grantable
        FROM pg_catalog.pg_proc AS pro
        LEFT JOIN LATERAL aclexplode(pro.proacl) AS acl (
            grantor,
            grantee,
            privilege_type,
            is_grantable
        ) ON true
        WHERE pro.prokind <> 'p' AND pro.proacl IS NOT NULL
        ORDER BY pro.oid, acl.grantee
        ",
    )
    .load::<ProcAclEntry>(conn)
}

/// Loads the definition `pg_get_functiondef` prints for every function whose
/// body `PostgreSQL` bound at creation, beside the function's OID.
///
/// The bound body is stored as a node tree in `prosqlbody`, which only the
/// server can print back as SQL.
pub(super) fn load_bound_definitions(
    conn: &mut PgConnection,
) -> Result<Vec<(u32, String)>, diesel::result::Error> {
    use crate::schema::pg_catalog::pg_proc::pg_proc;
    pg_proc::table
        .filter(pg_proc::prokind.ne("p"))
        .filter(pg_proc::prosqlbody.is_not_null())
        .select((pg_proc::oid, sql::<Text>("pg_get_functiondef(pg_proc.oid)")))
        .load(conn)
}

/// Loads the function each bound body reached, as `(function, reached)` OID
/// pairs in the order `pg_depend` lists them.
///
/// Binding a body records a normal dependency from the function on every
/// function its calls resolved to.
pub(super) fn load_bound_dependencies(
    conn: &mut PgConnection,
) -> Result<Vec<(u32, u32)>, diesel::result::Error> {
    use crate::schema::pg_catalog::{pg_depend::pg_depend, pg_proc::pg_proc};
    pg_depend::table
        .inner_join(pg_proc::table.on(pg_proc::oid.eq(pg_depend::objid)))
        .filter(pg_proc::prosqlbody.is_not_null())
        .filter(pg_depend::classid.eq(sql::<Oid>("'pg_proc'::regclass")))
        .filter(pg_depend::refclassid.eq(sql::<Oid>("'pg_proc'::regclass")))
        .filter(pg_depend::deptype.eq("n"))
        .select((pg_depend::objid, pg_depend::refobjid))
        .load(conn)
}
