//! Submodule providing the `PgMatview` struct representing a row of the
//! `pg_matviews` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_matviews` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_matviews::pg_matviews)]
pub struct PgMatview {
    /// Schema name.
    pub schemaname: Option<String>,
    /// Materialized view name.
    pub matviewname: Option<String>,
    /// Owner name.
    pub matviewowner: Option<String>,
    /// Tablespace name.
    pub tablespace: Option<String>,
    /// Whether the materialized view has indexes.
    pub hasindexes: Option<bool>,
    /// Whether the materialized view is populated.
    pub ispopulated: Option<bool>,
    /// SQL definition of the materialized view.
    pub definition: Option<String>,
}

impl PgMatview {
    /// Loads every materialized view declared in the given schemas.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn load_all(
        schemas: &[String],
        conn: &mut diesel::PgConnection,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::schema::pg_catalog::pg_matviews::pg_matviews;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};

        pg_matviews::table
            .filter(pg_matviews::schemaname.eq_any(schemas))
            .order_by(pg_matviews::matviewname)
            .select(Self::as_select())
            .load::<Self>(conn)
    }
}
