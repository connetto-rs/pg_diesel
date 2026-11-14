//! `PolicyMetadata` struct.

use sql_traits::structs::metadata::ExpressionCalls;
use sqlparser::ast::{Expr, Owner};

use crate::model_metadata::PgFunction;

#[derive(Debug, Clone)]
/// Metadata for a policy.
pub struct PolicyMetadata {
    /// The functions each call of the USING expression may reach, resolved
    /// under the search path in force where the policy was written.
    pub using_calls: ExpressionCalls<PgFunction>,
    /// The functions each call of the WITH CHECK expression may reach.
    pub check_calls: ExpressionCalls<PgFunction>,
    /// Parsed USING expression.
    pub using_expression: Option<Expr>,
    /// Parsed WITH CHECK expression.
    pub check_expression: Option<Expr>,
    /// Roles the policy applies to.
    pub roles: Vec<Owner>,
}

impl PolicyMetadata {
    /// Creates a new `PolicyMetadata`.
    #[must_use]
    pub fn new(
        using_calls: ExpressionCalls<PgFunction>,
        check_calls: ExpressionCalls<PgFunction>,
        using_expression: Option<Expr>,
        check_expression: Option<Expr>,
        roles: Vec<Owner>,
    ) -> Self {
        Self {
            using_calls,
            check_calls,
            using_expression,
            check_expression,
            roles,
        }
    }

    /// Returns the call record of the USING expression.
    #[must_use]
    pub fn using_calls(&self) -> &ExpressionCalls<PgFunction> {
        &self.using_calls
    }

    /// Returns the call record of the WITH CHECK expression.
    #[must_use]
    pub fn check_calls(&self) -> &ExpressionCalls<PgFunction> {
        &self.check_calls
    }

    /// Returns the functions the USING expression may reach, each once.
    pub fn using_functions(&self) -> impl Iterator<Item = &PgFunction> {
        self.using_calls.functions()
    }

    /// Returns the functions the WITH CHECK expression may reach, each once.
    pub fn check_functions(&self) -> impl Iterator<Item = &PgFunction> {
        self.check_calls.functions()
    }

    /// Returns the using expression.
    #[must_use]
    pub fn using_expression(&self) -> Option<&Expr> {
        self.using_expression.as_ref()
    }

    /// Returns the check expression.
    #[must_use]
    pub fn check_expression(&self) -> Option<&Expr> {
        self.check_expression.as_ref()
    }

    /// Returns the roles.
    pub fn roles(&self) -> impl Iterator<Item = &Owner> {
        self.roles.iter()
    }
}
