//! Submodule providing the `DomainMetadata` struct for a
//! [`PgDomainEntry`](crate::model_metadata::PgDomainEntry).

use sql_traits::structs::metadata::ExpressionCalls;

use crate::model_metadata::PgFunction;

/// The functions the calls of a domain's default and checks may reach,
/// resolved under the search path `pg_get_expr` printed them under.
#[derive(Debug, Clone)]
pub struct DomainMetadata {
    /// The call record of the default expression.
    default_calls: ExpressionCalls<PgFunction>,
    /// The call record of each check expression, in the order the domain
    /// holds the checks.
    check_calls: Vec<ExpressionCalls<PgFunction>>,
}

impl DomainMetadata {
    /// Creates the metadata from the call records of the default and of each
    /// check, in the order the domain holds the checks.
    #[must_use]
    pub fn new(
        default_calls: ExpressionCalls<PgFunction>,
        check_calls: Vec<ExpressionCalls<PgFunction>>,
    ) -> Self {
        Self {
            default_calls,
            check_calls,
        }
    }

    /// Returns the call record of the default expression.
    #[must_use]
    pub fn default_calls(&self) -> &ExpressionCalls<PgFunction> {
        &self.default_calls
    }

    /// Returns the call record of each check expression, in the order the
    /// domain holds the checks.
    #[must_use]
    pub fn check_calls(&self) -> &[ExpressionCalls<PgFunction>] {
        &self.check_calls
    }
}
