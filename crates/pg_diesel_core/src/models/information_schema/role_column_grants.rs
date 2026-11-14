//! `RoleColumnGrants` model representing the column-level grants a column's
//! `pg_attribute.attacl` records.

use diesel::{PgConnection, Queryable, QueryableByName, Selectable};

/// Model struct for one column-level grant, the way its `privilege_type`
/// names the privilege.
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Queryable, QueryableByName, Selectable,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::information_schema::role_column_grants::role_column_grants)]
pub struct RoleColumnGrants {
    /// Role that granted the privilege.
    pub grantor: Option<String>,
    /// Role that received the privilege.
    pub grantee: Option<String>,
    /// Catalog (database) containing the table.
    pub table_catalog: Option<String>,
    /// Schema containing the table.
    pub table_schema: Option<String>,
    /// Name of the table.
    pub table_name: Option<String>,
    /// Name of the column.
    pub column_name: Option<String>,
    /// Type of privilege granted.
    pub privilege_type: Option<String>,
    /// Whether the privilege is grantable.
    pub is_grantable: Option<String>,
}

/// One privilege a column's `pg_attribute.attacl` grants, as `aclexplode`
/// lists it.
#[derive(Debug, Clone, diesel::QueryableByName)]
pub struct AclColumnGrant {
    /// The role that granted the privilege, its OID when the role is gone.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub grantor: String,
    /// The role that received it, `PUBLIC` for the pseudo-role.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub grantee: String,
    /// The schema the column's table lives in.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub table_schema: String,
    /// The name of the table the column belongs to.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub table_name: String,
    /// The name of the column the privilege reaches.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub column_name: String,
    /// The privilege, spelled as `PostgreSQL` names it.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub privilege_type: String,
    /// Whether the privilege was granted `WITH GRANT OPTION`.
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub is_grantable: bool,
}

impl RoleColumnGrants {
    /// Loads every privilege `pg_attribute.attacl` grants on the columns of
    /// the relations in `table_schemas`.
    ///
    /// Raw `sql_query`, since the DSL cannot place `aclexplode` in a `FROM`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn load_acl(
        table_schemas: &[String],
        conn: &mut PgConnection,
    ) -> Result<Vec<AclColumnGrant>, diesel::result::Error> {
        Self::load_acl_filtered(table_schemas, None, conn)
    }

    /// Loads column grants, restricting table grants to the selected OIDs.
    pub(crate) fn load_acl_filtered(
        table_schemas: &[String],
        table_oids: Option<&[u32]>,
        conn: &mut PgConnection,
    ) -> Result<Vec<AclColumnGrant>, diesel::result::Error> {
        use diesel::RunQueryDsl;

        diesel::sql_query(
            "SELECT coalesce(grantor.rolname, entry.grantor::text) AS grantor, \
             CASE WHEN entry.grantee = 0 THEN 'PUBLIC' \
                  ELSE coalesce(grantee.rolname, entry.grantee::text) END AS grantee, \
             namespace.nspname AS table_schema, class.relname AS table_name, \
             attribute.attname AS column_name, \
             entry.privilege_type AS privilege_type, entry.is_grantable AS is_grantable \
             FROM pg_catalog.pg_attribute AS attribute \
             JOIN pg_catalog.pg_class AS class ON class.oid = attribute.attrelid \
             JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = class.relnamespace \
             CROSS JOIN LATERAL aclexplode(attribute.attacl) AS entry \
             LEFT JOIN pg_catalog.pg_roles AS grantor ON grantor.oid = entry.grantor \
             LEFT JOIN pg_catalog.pg_roles AS grantee ON grantee.oid = entry.grantee \
             WHERE class.relkind IN ('r', 'p', 'v', 'm', 'f') \
               AND attribute.attnum > 0 AND NOT attribute.attisdropped \
               AND namespace.nspname = ANY($1) \
               AND ($2::oid[] IS NULL OR class.relkind NOT IN ('r', 'p', 'f') \
                    OR class.oid = ANY($2)) \
             ORDER BY namespace.nspname, class.relname, attribute.attnum, grantee, privilege_type",
        )
        .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(table_schemas)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Oid>>, _>(
            table_oids,
        )
        .load::<AclColumnGrant>(conn)
    }
}
