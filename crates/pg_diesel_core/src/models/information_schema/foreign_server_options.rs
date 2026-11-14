//! Model struct for the `information_schema.foreign_server_options` view.

use diesel::prelude::*;

/// Model struct for `information_schema.foreign_server_options`.
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Queryable, QueryableByName, Selectable,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::information_schema::foreign_server_options::foreign_server_options)]
pub struct ForeignServerOptions {
    /// Catalog (database) containing the foreign server.
    pub foreign_server_catalog: Option<String>,
    /// Name of the foreign server.
    pub foreign_server_name: Option<String>,
    /// Name of the option.
    pub option_name: Option<String>,
    /// Value of the option.
    pub option_value: Option<String>,
}
