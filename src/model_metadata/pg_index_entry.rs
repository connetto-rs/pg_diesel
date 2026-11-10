//! Submodule providing [`PgIndexEntry`], the index a `PgDieselDatabase` holds.

use crate::models::PgIndex;

/// A `pg_index` row paired with the name and schema its `indexrelid` names.
///
/// `IndexLike` asks for both without handing over a database.
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
}

impl PgIndexEntry {
    /// Pairs a `pg_index` row with the relation name its `indexrelid` names.
    #[must_use]
    pub fn new(
        model: PgIndex,
        schema: String,
        name: String,
        constraint_name: Option<String>,
    ) -> Self {
        Self {
            model,
            schema,
            name,
            constraint_name,
        }
    }

    /// Returns the `pg_catalog.pg_index` row describing the index.
    #[must_use]
    pub fn model(&self) -> &PgIndex {
        &self.model
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

    /// Returns the metadata for the index.
    ///
    /// # Arguments
    ///
    /// * `table` - A reference-counted pointer to the table this index belongs
    ///   to.
    /// * `conn` - A mutable reference to a `PgConnection`
    ///
    /// # Errors
    ///
    /// * If an error occurs while loading the metadata from the database
    pub fn metadata(
        &self,
        table: std::sync::Arc<crate::model_metadata::PgTable>,
        conn: &mut diesel::PgConnection,
    ) -> Result<sql_traits::structs::metadata::UniqueIndexMetadata<Self>, diesel::result::Error>
    {
        Ok(sql_traits::structs::metadata::UniqueIndexMetadata::new(
            self.model.index_expression(conn)?,
            table,
        ))
    }
}
