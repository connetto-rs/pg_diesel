//! One row the raw read of a relation's `pg_class.relacl` yields, through
//! `aclexplode`, for the builder's table grants.

use diesel::{PgConnection, QueryableByName};

/// One privilege a relation's `pg_class.relacl` grants, as `aclexplode`
/// lists it.
#[derive(Debug, Clone, QueryableByName)]
pub struct AclTableGrant {
    /// The role that granted the privilege, its OID when the role is gone.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub grantor: String,
    /// The role that received it, `PUBLIC` for the pseudo-role.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub grantee: String,
    /// The schema the relation the privilege reaches lives in.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub table_schema: String,
    /// The name of the relation the privilege reaches.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub table_name: String,
    /// The kind of relation the privilege reaches, the `pg_class.relkind`
    /// `PostgreSQL` records it under.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub relkind: String,
    /// The privilege, spelled as `PostgreSQL` names it.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub privilege_type: String,
    /// Whether the privilege was granted `WITH GRANT OPTION`.
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub is_grantable: bool,
}

impl AclTableGrant {
    /// Loads every privilege `pg_class.relacl` grants on the relations in
    /// `table_schemas`, leaving out the owner's own entry, which
    /// [`TableLike::owner_privileges`](sql_traits::traits::TableLike::owner_privileges)
    /// answers instead.
    ///
    /// Raw `sql_query`, since the DSL cannot place `aclexplode` in a `FROM`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn load(
        table_schemas: &[String],
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use diesel::RunQueryDsl;

        diesel::sql_query(
            "SELECT coalesce(grantor.rolname, entry.grantor::text) AS grantor, \
             CASE WHEN entry.grantee = 0 THEN 'PUBLIC' \
                  ELSE coalesce(grantee.rolname, entry.grantee::text) END AS grantee, \
             namespace.nspname AS table_schema, class.relname AS table_name, \
             class.relkind AS relkind, \
             entry.privilege_type AS privilege_type, entry.is_grantable AS is_grantable \
             FROM pg_catalog.pg_class AS class \
             JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = class.relnamespace \
             CROSS JOIN LATERAL aclexplode(class.relacl) AS entry \
             LEFT JOIN pg_catalog.pg_roles AS grantor ON grantor.oid = entry.grantor \
             LEFT JOIN pg_catalog.pg_roles AS grantee ON grantee.oid = entry.grantee \
             WHERE class.relkind IN ('r', 'p', 'v', 'm', 'f', 'S') \
               AND entry.grantee <> class.relowner \
               AND namespace.nspname = ANY($1) \
             ORDER BY namespace.nspname, class.relname, grantee, privilege_type",
        )
        .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(table_schemas)
        .load::<Self>(conn)
    }
}
