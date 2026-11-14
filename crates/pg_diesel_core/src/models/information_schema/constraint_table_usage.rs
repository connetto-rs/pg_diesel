//! Constraint table usage model.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row in the `constraint_table_usage` table in the `PostgreSQL`
/// database.
#[derive(
    Queryable, QueryableByName, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Selectable,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::information_schema::constraint_table_usage::constraint_table_usage)]
pub struct ConstraintTableUsage {
    /// The catalog of the constraint.
    pub constraint_catalog: String,
    /// The schema of the constraint.
    pub constraint_schema: String,
    /// The name of the constraint.
    pub constraint_name: String,
    /// The catalog of the table.
    pub table_catalog: String,
    /// The schema of the table.
    pub table_schema: String,
    /// The name of the table.
    pub table_name: String,
}
