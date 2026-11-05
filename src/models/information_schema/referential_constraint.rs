//! Referential constraint model.

use diesel::{Queryable, QueryableByName, Selectable};
use sqlparser::ast::ReferentialAction;

/// Reads the rule `PostgreSQL` stores for a referential action.
fn referential_action(rule: &str) -> ReferentialAction {
    match rule.to_uppercase().as_str() {
        "NO ACTION" => ReferentialAction::NoAction,
        "RESTRICT" => ReferentialAction::Restrict,
        "CASCADE" => ReferentialAction::Cascade,
        "SET NULL" => ReferentialAction::SetNull,
        "SET DEFAULT" => ReferentialAction::SetDefault,
        other => unreachable!("unexpected referential action: {other}"),
    }
}

#[derive(Queryable, QueryableByName, Selectable, PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::information_schema::referential_constraints::referential_constraints)]
/// A struct representing a referential constraint
pub struct ReferentialConstraint {
    /// The name of the constraint catalog
    pub constraint_catalog: String,
    /// The name of the constraint schema
    pub constraint_schema: String,
    /// The name of the constraint
    pub constraint_name: String,
    /// The name of the table catalog the constraint is associated with
    pub unique_constraint_catalog: Option<String>,
    /// The name of the table schema the constraint is associated with
    pub unique_constraint_schema: Option<String>,
    /// The name of the table the constraint is associated with
    pub unique_constraint_name: Option<String>,
    /// Match options
    pub match_option: String,
    /// Update rule
    pub update_rule: String,
    /// Delete rule
    pub delete_rule: String,
}

impl ReferentialConstraint {
    /// Returns the action the constraint takes on its referencing rows when a
    /// referenced row is deleted.
    #[must_use]
    pub fn on_delete(&self) -> sqlparser::ast::ReferentialAction {
        referential_action(&self.delete_rule)
    }

    /// Returns the action the constraint takes on its referencing rows when a
    /// referenced key is updated.
    #[must_use]
    pub fn on_update(&self) -> sqlparser::ast::ReferentialAction {
        referential_action(&self.update_rule)
    }

    /// Returns the match kind of the referential constraint
    #[must_use]
    pub fn match_kind(&self) -> sqlparser::ast::ConstraintReferenceMatchKind {
        match self.match_option.to_uppercase().as_str() {
            "FULL" => sqlparser::ast::ConstraintReferenceMatchKind::Full,
            "PARTIAL" => sqlparser::ast::ConstraintReferenceMatchKind::Partial,
            "SIMPLE" | "NONE" => sqlparser::ast::ConstraintReferenceMatchKind::Simple,
            other => unreachable!("Unexpected match option: {other}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_referential_constraint() -> ReferentialConstraint {
        ReferentialConstraint {
            constraint_catalog: "db".to_string(),
            constraint_schema: "schema".to_string(),
            constraint_name: "constraint".to_string(),
            unique_constraint_catalog: None,
            unique_constraint_schema: None,
            unique_constraint_name: None,
            match_option: "SIMPLE".to_string(),
            update_rule: "NO ACTION".to_string(),
            delete_rule: "NO ACTION".to_string(),
        }
    }

    #[test]
    fn test_referential_actions() {
        let mut constraint = dummy_referential_constraint();
        assert_eq!(constraint.on_delete(), ReferentialAction::NoAction);
        assert_eq!(constraint.on_update(), ReferentialAction::NoAction);

        for (rule, action) in [
            ("NO ACTION", ReferentialAction::NoAction),
            ("RESTRICT", ReferentialAction::Restrict),
            ("CASCADE", ReferentialAction::Cascade),
            ("SET NULL", ReferentialAction::SetNull),
            ("SET DEFAULT", ReferentialAction::SetDefault),
        ] {
            constraint.delete_rule = rule.to_string();
            constraint.update_rule = rule.to_string();
            assert_eq!(constraint.on_delete(), action);
            assert_eq!(constraint.on_update(), action);
        }

        constraint.delete_rule = "cascade".to_string();
        assert_eq!(constraint.on_delete(), ReferentialAction::Cascade);
    }

    #[test]
    fn test_match_kind() {
        let mut constraint = dummy_referential_constraint();

        constraint.match_option = "FULL".to_string();
        assert!(matches!(
            constraint.match_kind(),
            sqlparser::ast::ConstraintReferenceMatchKind::Full
        ));

        constraint.match_option = "PARTIAL".to_string();
        assert!(matches!(
            constraint.match_kind(),
            sqlparser::ast::ConstraintReferenceMatchKind::Partial
        ));

        constraint.match_option = "SIMPLE".to_string();
        assert!(matches!(
            constraint.match_kind(),
            sqlparser::ast::ConstraintReferenceMatchKind::Simple
        ));
    }

    #[test]
    #[should_panic(expected = "Unexpected match option: INVALID")]
    fn test_match_kind_invalid() {
        let mut constraint = dummy_referential_constraint();
        constraint.match_option = "INVALID".to_string();
        let _ = constraint.match_kind();
    }
}
