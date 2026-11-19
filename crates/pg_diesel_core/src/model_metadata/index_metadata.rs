//! Submodule providing the `IndexMetadata` struct for a
//! [`PgIndexEntry`](crate::model_metadata::PgIndexEntry).

use std::sync::Arc;

use sql_traits::structs::metadata::ExpressionCalls;

use crate::model_metadata::{IndexedRelation, OnTable, PgFunction};

/// The relation an index is built on, and the functions the calls of its key
/// and predicate may reach.
#[derive(Debug, Clone)]
pub struct IndexMetadata<On: IndexedRelation = OnTable> {
    /// The relation the index is built on.
    relation: Arc<On::Relation>,
    /// The call record of the key expression.
    expression_calls: ExpressionCalls<PgFunction>,
    /// The call record of the predicate of a partial index.
    predicate_calls: ExpressionCalls<PgFunction>,
}

impl<On: IndexedRelation> IndexMetadata<On> {
    /// Creates the metadata of an index on `relation`, with no calls
    /// resolved.
    #[must_use]
    pub fn new(relation: Arc<On::Relation>) -> Self {
        Self {
            relation,
            expression_calls: ExpressionCalls::default(),
            predicate_calls: ExpressionCalls::default(),
        }
    }

    /// Sets the functions the calls of the key expression and of the
    /// predicate may reach, resolved under the search path in force where
    /// `PostgreSQL` printed them.
    #[must_use]
    pub fn with_calls(
        mut self,
        expression_calls: ExpressionCalls<PgFunction>,
        predicate_calls: ExpressionCalls<PgFunction>,
    ) -> Self {
        self.expression_calls = expression_calls;
        self.predicate_calls = predicate_calls;
        self
    }

    /// Returns the relation the index is built on.
    #[must_use]
    pub fn relation(&self) -> &On::Relation {
        &self.relation
    }

    /// Returns the call record of the key expression.
    #[must_use]
    pub fn expression_calls(&self) -> &ExpressionCalls<PgFunction> {
        &self.expression_calls
    }

    /// Returns the call record of the predicate.
    #[must_use]
    pub fn predicate_calls(&self) -> &ExpressionCalls<PgFunction> {
        &self.predicate_calls
    }
}
