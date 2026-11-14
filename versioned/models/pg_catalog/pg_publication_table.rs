//! Submodule providing the `PgPublicationTable` struct representing a row of
//! the `pg_publication_tables` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_publication_tables` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_publication_tables::pg_publication_tables)]
pub struct PgPublicationTable {
    /// Publication name.
    pub pubname: Option<String>,
    /// Schema name.
    pub schemaname: Option<String>,
    /// Table name.
    pub tablename: Option<String>,
    /// Column names included (`PostgreSQL` 15+).
    #[cfg(not(feature = "postgres-14"))]
    pub attnames: Option<Vec<String>>,
    /// Row filter expression (`PostgreSQL` 15+).
    #[cfg(not(feature = "postgres-14"))]
    pub rowfilter: Option<String>,
}
