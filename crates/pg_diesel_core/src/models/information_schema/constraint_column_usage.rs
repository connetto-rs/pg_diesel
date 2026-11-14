//! Constraint column usage model.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row in the `constraint_column_usage` table in `PostgreSQL`
#[derive(
    Queryable, QueryableByName, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Selectable,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::information_schema::constraint_column_usage::constraint_column_usage)]
pub struct ConstraintColumnUsage {
    /// The name of the database containing the constraint
    pub constraint_catalog: String,
    /// The name of the schema containing the constraint
    pub constraint_schema: String,
    /// The name of the constraint
    pub constraint_name: String,
    /// The name of the database containing the table with the column
    pub table_catalog: String,
    /// The name of the schema containing the table with the column
    pub table_schema: String,
    /// The name of the table containing the column
    pub table_name: String,
    /// The name of the column used in the constraint
    pub column_name: String,
}
