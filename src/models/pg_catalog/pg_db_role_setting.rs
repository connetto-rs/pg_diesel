//! Submodule providing the `PgDbRoleSetting` struct representing a row of the
//! `pg_db_role_setting` table in `PostgreSQL`.

use diesel::{
    ExpressionMethods, PgConnection, QueryDsl, Queryable, QueryableByName, RunQueryDsl, Selectable,
    SelectableHelper,
};
use sql_traits::structs::RoleSetting;

use crate::{
    database::CatalogCache,
    schema::pg_catalog::{pg_database::pg_database, pg_db_role_setting::pg_db_role_setting},
};

/// Represents a row from the `pg_db_role_setting` table.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_db_role_setting::pg_db_role_setting)]
pub struct PgDbRoleSetting {
    /// OID of the database this setting applies to (0 = all databases).
    pub setdatabase: u32,
    /// OID of the role this setting applies to (0 = all roles).
    pub setrole: u32,
    /// Array of configuration parameter settings.
    pub setconfig: Option<Vec<String>>,
}

/// Why the role settings of a catalog could not be read.
#[derive(Debug, thiserror::Error)]
pub enum RoleSettingError {
    /// A catalog query failed.
    #[error(transparent)]
    Query(#[from] diesel::result::Error),
    /// A setting holds a value sql-traits refuses, such as a time zone
    /// `PostgreSQL` would not accept.
    #[error(transparent)]
    Setting(#[from] sql_traits::errors::Error),
}

impl PgDbRoleSetting {
    /// Loads the `ALTER ROLE ... SET` entries that reach the database
    /// `catalog`, those for every database and those for `catalog` itself, one
    /// [`RoleSetting`] per `name=value` entry in the order `setconfig` keeps.
    ///
    /// A setting for a role no longer in `cache` is left out.
    ///
    /// # Errors
    ///
    /// Returns a [`RoleSettingError`] when a query fails or sql-traits
    /// refuses a setting.
    pub fn load_settings(
        catalog: &str,
        cache: &CatalogCache,
        conn: &mut PgConnection,
    ) -> Result<Vec<RoleSetting>, RoleSettingError> {
        let database: u32 = pg_database::table
            .filter(pg_database::datname.eq(catalog))
            .select(pg_database::oid)
            .first(conn)?;
        let rows: Vec<Self> = pg_db_role_setting::table
            .filter(pg_db_role_setting::setdatabase.eq_any([0, database]))
            .order((pg_db_role_setting::setdatabase, pg_db_role_setting::setrole))
            .select(Self::as_select())
            .load(conn)?;
        let mut settings = Vec::new();
        for row in rows {
            let role = match row.setrole {
                0 => None,
                oid => match cache.role(oid) {
                    Some(name) => Some(name.to_owned()),
                    None => continue,
                },
            };
            let database = (row.setdatabase != 0).then(|| catalog.to_owned());
            for entry in row.setconfig.unwrap_or_default() {
                let Some((name, value)) = entry.split_once('=') else {
                    continue;
                };
                settings.push(RoleSetting::new(
                    role.clone(),
                    database.clone(),
                    name,
                    value,
                )?);
            }
        }
        Ok(settings)
    }
}
