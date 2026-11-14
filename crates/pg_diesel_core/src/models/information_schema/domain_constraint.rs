//! Domain constraint model.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a domain constraint in the database.
#[derive(
    Queryable, QueryableByName, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Selectable,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::information_schema::domain_constraints::domain_constraints)]
pub struct DomainConstraint {
    /// The name of the database containing the constraint (always the current
    /// database).
    pub constraint_catalog: String,
    /// The name of the schema containing the constraint.
    pub constraint_schema: String,
    /// The name of the constraint.
    pub constraint_name: String,
    /// The name of the database containing the domain (always the current
    /// database).
    pub domain_catalog: Option<String>,
    /// The name of the schema containing the domain.
    pub domain_schema: Option<String>,
    /// The name of the domain.
    pub domain_name: Option<String>,
    /// Indicates if the constraint is deferrable (YES or NO).
    pub is_deferrable: String,
    /// Indicates if the constraint is initially deferred (YES or NO).
    pub initially_deferred: String,
}
