//! Submodule providing [`PgForeignKey`], the foreign key a database holds.

use crate::models::KeyColumnUsage;

/// A `key_column_usage` row paired with the relation and the columns it
/// references.
///
/// `ForeignKeyLike` asks for those names without handing over a database, and
/// the row describes only the referencing side.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgForeignKey {
    /// The `information_schema.key_column_usage` row describing the key.
    model: KeyColumnUsage,
    /// The schema of the referenced relation.
    referenced_schema: String,
    /// The name of the referenced relation.
    referenced_name: String,
    /// The `attname` of each referenced column, in the order of `confkey`.
    referenced_attnames: Vec<String>,
}

impl PgForeignKey {
    /// Pairs a `key_column_usage` row with the relation and the columns it
    /// references.
    #[must_use]
    pub fn new(
        model: KeyColumnUsage,
        referenced_schema: String,
        referenced_name: String,
        referenced_attnames: Vec<String>,
    ) -> Self {
        Self {
            model,
            referenced_schema,
            referenced_name,
            referenced_attnames,
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

    /// Returns the `attname` of each referenced column, in the order the key
    /// pairs them with its host columns.
    #[must_use]
    pub fn referenced_attnames(&self) -> &[String] {
        &self.referenced_attnames
    }
}
