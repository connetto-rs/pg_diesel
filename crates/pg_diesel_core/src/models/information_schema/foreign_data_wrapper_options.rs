//! Model struct for the `information_schema.foreign_data_wrapper_options` view.

use diesel::prelude::*;

/// Model struct for `information_schema.foreign_data_wrapper_options`.
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Queryable, QueryableByName, Selectable,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::information_schema::foreign_data_wrapper_options::foreign_data_wrapper_options)]
pub struct ForeignDataWrapperOptions {
    /// Catalog (database) containing the foreign data wrapper.
    pub foreign_data_wrapper_catalog: Option<String>,
    /// Name of the foreign data wrapper.
    pub foreign_data_wrapper_name: Option<String>,
    /// Name of the option.
    pub option_name: Option<String>,
    /// Value of the option.
    pub option_value: Option<String>,
}
