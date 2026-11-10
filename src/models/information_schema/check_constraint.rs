//! Check constraint model.

use std::fmt::{Debug, Display};

use diesel::{Queryable, QueryableByName, Selectable};

#[derive(
    Queryable, QueryableByName, Debug, Clone, Selectable, Ord, PartialEq, Eq, Hash, PartialOrd,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::information_schema::check_constraints::check_constraints)]
/// A struct representing a check constraint
pub struct CheckConstraint {
    /// The name of the constraint catalog
    pub constraint_catalog: String,
    /// The name of the constraint schema
    pub constraint_schema: String,
    /// The name of the constraint
    pub constraint_name: String,
    /// The check clause of the constraint
    pub check_clause: String,
}

impl Display for CheckConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            self.constraint_catalog, self.constraint_schema, self.constraint_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_check_constraint() -> CheckConstraint {
        CheckConstraint {
            constraint_catalog: "db".to_string(),
            constraint_schema: "schema".to_string(),
            constraint_name: "constraint".to_string(),
            check_clause: "CHECK (id > 0)".to_string(),
        }
    }

    #[test]
    fn test_display() {
        let constraint = dummy_check_constraint();
        assert_eq!(format!("{constraint}"), "db.schema.constraint");
    }
}
