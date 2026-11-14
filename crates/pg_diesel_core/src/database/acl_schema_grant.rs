//! One row the raw read of a schema's `pg_namespace.nspacl` yields, through
//! `aclexplode`, for the builder's schema grants.

use diesel::{PgConnection, QueryableByName};

/// One privilege a schema's `pg_namespace.nspacl` grants, as `aclexplode`
/// lists it.
#[derive(Debug, Clone, QueryableByName)]
pub struct AclSchemaGrant {
    /// The role that granted the privilege, its OID when the role is gone.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub grantor: String,
    /// The role that received it, `PUBLIC` for the pseudo-role.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub grantee: String,
    /// The schema the privilege reaches.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub schema: String,
    /// The privilege, `USAGE` or `CREATE`.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub privilege_type: String,
    /// Whether the privilege was granted `WITH GRANT OPTION`.
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub is_grantable: bool,
}

impl AclSchemaGrant {
    /// Loads every privilege `pg_namespace.nspacl` grants on `schemas`,
    /// leaving out the owner's own entry and the `USAGE` the built-in
    /// `public` schema gives `PUBLIC` from creation, which
    /// [`SchemaLike::owner_holds_usage`](sql_traits::traits::SchemaLike::owner_holds_usage)
    /// and
    /// [`SchemaLike::public_holds_usage`](sql_traits::traits::SchemaLike::public_holds_usage)
    /// answer instead.
    ///
    /// Raw `sql_query`, since the DSL cannot place `aclexplode` in a `FROM`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn load(
        schemas: &[String],
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use diesel::RunQueryDsl;

        diesel::sql_query(
            "SELECT coalesce(grantor.rolname, entry.grantor::text) AS grantor, \
             CASE WHEN entry.grantee = 0 THEN 'PUBLIC' \
                  ELSE coalesce(grantee.rolname, entry.grantee::text) END AS grantee, \
             namespace.nspname AS schema, \
             entry.privilege_type AS privilege_type, entry.is_grantable AS is_grantable \
             FROM pg_catalog.pg_namespace AS namespace \
             CROSS JOIN LATERAL aclexplode(namespace.nspacl) AS entry \
             LEFT JOIN pg_catalog.pg_roles AS grantor ON grantor.oid = entry.grantor \
             LEFT JOIN pg_catalog.pg_roles AS grantee ON grantee.oid = entry.grantee \
             WHERE entry.grantee <> namespace.nspowner \
               AND NOT (namespace.oid = 2200 AND entry.grantee = 0 \
                        AND entry.privilege_type = 'USAGE') \
               AND namespace.nspname = ANY($1) \
             ORDER BY namespace.nspname, grantee, privilege_type",
        )
        .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(schemas)
        .load::<Self>(conn)
    }
}
