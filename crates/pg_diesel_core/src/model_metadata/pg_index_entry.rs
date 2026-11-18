//! Submodule providing [`PgIndexEntry`], the index a `PgDieselDatabase` holds.

use std::sync::Arc;

use diesel::PgConnection;
use sqlparser::ast::{Expr, Ident, IndexType};

use crate::{
    model_metadata::{IndexMetadata, PgTable},
    shared::models::PgIndex,
};

/// What an index is beyond its key and predicate, as `pg_class.relam` and
/// `pg_index` record it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct IndexShape {
    /// The access method of the index.
    access_method: IndexType,
    /// The names of the columns the `INCLUDE` list carries, in the order
    /// written.
    include_column_names: Vec<String>,
    /// Whether the index treats nulls as distinct from one another.
    nulls_distinct: bool,
}

impl IndexShape {
    /// Creates the shape of an index whose access method `pg_am.amname`
    /// names `access_method`.
    #[must_use]
    pub(crate) fn new(
        access_method: &str,
        include_column_names: Vec<String>,
        nulls_distinct: bool,
    ) -> Self {
        let access_method = match access_method {
            "btree" => IndexType::BTree,
            "hash" => IndexType::Hash,
            "gin" => IndexType::GIN,
            "gist" => IndexType::GiST,
            "spgist" => IndexType::SPGiST,
            "brin" => IndexType::BRIN,
            "bloom" => IndexType::Bloom,
            other => IndexType::Custom(Ident::new(other)),
        };
        Self {
            access_method,
            include_column_names,
            nulls_distinct,
        }
    }
}

/// A `pg_index` row paired with the name and schema its `indexrelid` names,
/// its access method, the columns of its `INCLUDE` list and the predicate of
/// a partial index.
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
    /// The access method, `INCLUDE` list and null treatment of the index.
    shape: IndexShape,
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
        shape: IndexShape,
        constraint_name: Option<String>,
        predicate: Option<Expr>,
    ) -> Self {
        Self {
            model,
            schema,
            name,
            shape,
            constraint_name,
            predicate,
        }
    }

    /// Returns the access method of the index.
    #[must_use]
    pub fn access_method(&self) -> &IndexType {
        &self.shape.access_method
    }

    /// Returns the names of the columns the `INCLUDE` list of the index
    /// carries, in the order written.
    #[must_use]
    pub fn include_column_names(&self) -> &[String] {
        &self.shape.include_column_names
    }

    /// Returns whether the index treats nulls as distinct from one another,
    /// `false` for an index created `NULLS NOT DISTINCT`.
    #[must_use]
    pub fn nulls_distinct(&self) -> bool {
        self.shape.nulls_distinct
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

#[cfg(test)]
mod tests {
    use sql_traits::{prelude::ParserDB, traits::DatabaseLike, traits::IndexLike};
    use sqlparser::dialect::PostgreSqlDialect;

    use super::IndexShape;

    #[test]
    fn an_access_method_answers_what_parser_db_answers_for_its_using_clause() {
        for method in [
            "btree", "hash", "gin", "gist", "spgist", "brin", "bloom", "hnsw",
        ] {
            let database = ParserDB::parse::<PostgreSqlDialect>(&format!(
                "CREATE TABLE docs (body text);
                 CREATE INDEX probe ON docs USING {method} (body);"
            ))
            .expect("the schema parses");
            let index = database
                .indexes()
                .next()
                .expect("the schema holds one index");
            assert_eq!(
                Some(&IndexShape::new(method, Vec::new(), true).access_method),
                index.access_method(),
                "{method}"
            );
        }
    }
}
