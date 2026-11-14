//! Model struct for the `information_schema.schemata` view.

use diesel::prelude::*;

/// Represents a row from the `information_schema.schemata` view.
/// Contains metadata about database schemas including catalog, schema name,
/// owner, and default character set information.
#[derive(
    Queryable, QueryableByName, Selectable, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::information_schema::schemata::schemata)]
pub struct Schemata {
    /// Name of the database (catalog) containing the schema.
    pub catalog_name: String,
    /// Name of the schema.
    pub schema_name: String,
    /// Name of the user who owns the schema.
    pub schema_owner: String,
    /// Name of the default character set for the schema.
    pub default_character_set_catalog: Option<String>,
    /// Schema containing the default character set.
    pub default_character_set_schema: Option<String>,
    /// Name of the default character set.
    pub default_character_set_name: Option<String>,
    /// SQL path for the schema (typically NULL in `PostgreSQL`).
    pub sql_path: Option<String>,
}

impl Schemata {
    /// Loads the named schemas of the given catalog.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn load_all(
        catalog: &str,
        schemas: &[String],
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::schema::information_schema::schemata::schemata;

        schemata::table
            .filter(schemata::catalog_name.eq(catalog))
            .filter(schemata::schema_name.eq_any(schemas))
            .order_by(schemata::schema_name)
            .select(Self::as_select())
            .load::<Self>(conn)
    }
}
