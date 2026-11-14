//! Metadata for foreign key relationships.

use std::sync::Arc;

use crate::{model_metadata::PgTable, models::ReferentialConstraint};

#[derive(Debug, Clone)]
/// Struct collecting metadata about a foreign key represented by a
/// [`PgForeignKey`](crate::model_metadata::PgForeignKey) entry.
pub struct KeyColumnUsageMetadata {
    /// The names of the columns in the referenced table the foreign key
    /// points to.
    referenced_columns: Vec<String>,
    /// The table that contains the foreign key.
    host_table: Arc<PgTable>,
    /// The names of the columns in the host table that are part of the
    /// foreign key.
    host_columns: Vec<String>,
    /// The referential constraint associated with the foreign key.
    referential_constraint: ReferentialConstraint,
}

impl KeyColumnUsageMetadata {
    /// Creates a new `KeyColumnUsageMetadata` instance.
    pub(crate) fn new(
        referenced_columns: Vec<String>,
        host_table: Arc<PgTable>,
        host_columns: Vec<String>,
        referential_constraint: ReferentialConstraint,
    ) -> Self {
        Self {
            referenced_columns,
            host_table,
            host_columns,
            referential_constraint,
        }
    }

    /// Returns a reference to the table that contains the foreign key.
    #[must_use]
    pub fn host_table(&self) -> &PgTable {
        &self.host_table
    }

    /// Returns the names of the columns in the referenced table that the
    /// foreign key points to.
    #[must_use]
    pub fn referenced_columns(&self) -> &[String] {
        &self.referenced_columns
    }

    /// Returns the names of the columns in the host table that are part of
    /// the foreign key.
    #[must_use]
    pub fn host_columns(&self) -> &[String] {
        &self.host_columns
    }

    /// Returns a reference to the referential constraint associated with the
    /// foreign key.
    #[must_use]
    pub fn referential_constraint(&self) -> &ReferentialConstraint {
        &self.referential_constraint
    }

    /// Returns the action the foreign key takes when a referenced row is
    /// deleted.
    #[must_use]
    pub fn on_delete(&self) -> sqlparser::ast::ReferentialAction {
        self.referential_constraint.on_delete()
    }

    /// Returns the action the foreign key takes when a referenced key is
    /// updated.
    #[must_use]
    pub fn on_update(&self) -> sqlparser::ast::ReferentialAction {
        self.referential_constraint.on_update()
    }

    /// Returns the match kind of the foreign key.
    #[must_use]
    pub fn match_kind(&self) -> sqlparser::ast::ConstraintReferenceMatchKind {
        self.referential_constraint.match_kind()
    }
}
