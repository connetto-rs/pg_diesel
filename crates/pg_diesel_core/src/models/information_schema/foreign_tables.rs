//! Model struct for the `information_schema.foreign_tables` view.

use diesel::prelude::*;

/// Model struct for `information_schema.foreign_tables`.
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Queryable, QueryableByName, Selectable,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::information_schema::foreign_tables::foreign_tables)]
pub struct ForeignTables {
    /// Catalog (database) containing the foreign table.
    pub foreign_table_catalog: Option<String>,
    /// Schema containing the foreign table.
    pub foreign_table_schema: Option<String>,
    /// Name of the foreign table.
    pub foreign_table_name: Option<String>,
    /// Catalog (database) containing the foreign server.
    pub foreign_server_catalog: Option<String>,
    /// Name of the foreign server.
    pub foreign_server_name: Option<String>,
}
