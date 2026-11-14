//! Model struct for the `information_schema.routine_routine_usage` view.

use diesel::prelude::*;

/// Model struct for `information_schema.routine_routine_usage`.
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Queryable, QueryableByName, Selectable,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::information_schema::routine_routine_usage::routine_routine_usage)]
pub struct RoutineRoutineUsage {
    /// Catalog (database) containing the specific routine.
    pub specific_catalog: Option<String>,
    /// Schema containing the specific routine.
    pub specific_schema: Option<String>,
    /// Specific name of the routine.
    pub specific_name: Option<String>,
    /// Catalog (database) containing the used routine.
    pub routine_catalog: Option<String>,
    /// Schema containing the used routine.
    pub routine_schema: Option<String>,
    /// Name of the used routine.
    pub routine_name: Option<String>,
}
