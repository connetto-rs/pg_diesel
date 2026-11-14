//! Submodule providing [`PgForeignKey`], the foreign key a database holds.

use crate::models::KeyColumnUsage;

/// A `key_column_usage` row paired with the relation it references.
///
/// `ForeignKeyLike` asks for that name without handing over a database, and the
/// row describes only the referencing side.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgForeignKey {
    /// The `information_schema.key_column_usage` row describing the key.
    model: KeyColumnUsage,
    /// The schema of the referenced relation.
    referenced_schema: String,
    /// The name of the referenced relation.
    referenced_name: String,
}

impl PgForeignKey {
    /// Pairs a `key_column_usage` row with the relation it references.
    #[must_use]
    pub fn new(model: KeyColumnUsage, referenced_schema: String, referenced_name: String) -> Self {
        Self {
            model,
            referenced_schema,
            referenced_name,
        }
    }

    /// Returns the `information_schema.key_column_usage` row.
    #[must_use]
    pub fn model(&self) -> &KeyColumnUsage {
        &self.model
    }

    /// Returns the schema of the referenced relation.
    #[must_use]
    pub fn referenced_schema(&self) -> &str {
        &self.referenced_schema
    }

    /// Returns the name of the referenced relation.
    #[must_use]
    pub fn referenced_name(&self) -> &str {
        &self.referenced_name
    }
}
