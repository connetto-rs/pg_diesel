//! Submodule providing [`PgIndexEntry`], the index a `PgDieselDatabase` holds.

use std::sync::Arc;

use diesel::PgConnection;
use sqlparser::ast::Expr;

use crate::{
    model_metadata::{IndexMetadata, PgTable},
    shared::models::PgIndex,
};

/// A `pg_index` row paired with the name and schema its `indexrelid` names
/// and the predicate of a partial index.
///
/// `IndexLike` asks for these without handing over a database.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgIndexEntry {
    /// The `pg_catalog.pg_index` row describing the index.
    model: PgIndex,
    /// The schema the index lives in.
    schema: String,
    /// The name of the index.
    name: String,
    /// The name the primary key, unique or exclusion constraint the index
    /// enforces holds, when one does.
    constraint_name: Option<String>,
    /// The predicate `pg_get_expr` prints for a partial index.
    predicate: Option<Expr>,
}

impl PgIndexEntry {
    /// Pairs a `pg_index` row with the relation name its `indexrelid` names.
    #[must_use]
    pub(crate) fn new(
        model: PgIndex,
        schema: String,
        name: String,
        constraint_name: Option<String>,
        predicate: Option<Expr>,
    ) -> Self {
        Self {
            model,
            schema,
            name,
            constraint_name,
            predicate,
        }
    }

    /// Returns whether the index enforces uniqueness.
    #[must_use]
    pub fn is_unique(&self) -> bool {
        self.model.indisunique
    }

    /// Returns the schema the index lives in.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the name of the index.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the name the constraint the index enforces holds, when one
    /// does.
    #[must_use = "the index may enforce no constraint, so the caller decides"]
    pub fn constraint_name(&self) -> Option<&str> {
        self.constraint_name.as_deref()
    }

    /// Returns the predicate of a partial index, `None` for an index over
    /// every row.
    #[must_use]
    pub fn predicate(&self) -> Option<&Expr> {
        self.predicate.as_ref()
    }

    /// Returns the metadata of the index, defined on `table`, with no calls
    /// resolved.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if a catalog query fails.
    pub fn metadata(
        &self,
        table: Arc<PgTable>,
        conn: &mut PgConnection,
    ) -> Result<IndexMetadata, diesel::result::Error> {
        Ok(IndexMetadata::new(
            self.model.index_expression(conn)?,
            table,
        ))
    }
}
