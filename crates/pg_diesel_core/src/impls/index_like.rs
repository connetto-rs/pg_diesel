//! Implementation of [`IndexLike`] for [`PgIndexEntry`].

use std::borrow::Cow;

use sql_traits::{
    errors::{LookupError, ObjectKind},
    structs::{IndexKey, WrittenCall},
    traits::{DatabaseLike, IndexLike, Metadata, TableLike},
};
use sqlparser::ast::{Expr, IndexType};

use crate::{
    PgDieselDatabase,
    model_metadata::{IndexMetadata, PgColumn, PgIndexEntry},
};

impl Metadata for PgIndexEntry {
    type Meta = IndexMetadata;
}

impl IndexLike for PgIndexEntry {
    type DB = PgDieselDatabase;

    fn name(&self) -> Option<&str> {
        Some(PgIndexEntry::name(self))
    }

    fn is_unique(&self) -> bool {
        PgIndexEntry::is_unique(self)
    }

    fn name_is_quoted(&self) -> bool {
        // Catalog names are stored names: folding again would rename them.
        true
    }

    fn schema(&self) -> Option<&str> {
        Some(PgIndexEntry::schema(self))
    }

    fn schema_is_quoted(&self) -> bool {
        true
    }

    fn constraint_name<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<Option<Cow<'db, str>>, LookupError> {
        let _ = metadata(self, database)?;
        Ok(PgIndexEntry::constraint_name(self).map(Cow::Borrowed))
    }

    fn table<'db>(&'db self, database: &'db Self::DB) -> &'db <Self::DB as DatabaseLike>::Table
    where
        Self: 'db,
    {
        metadata(self, database)
            .expect("an index the database lists carries its metadata")
            .table()
    }

    fn expression<'db>(&'db self, database: &'db Self::DB) -> Result<&'db Expr, LookupError>
    where
        Self: 'db,
    {
        let _ = metadata(self, database)?;
        Ok(PgIndexEntry::expression(self))
    }

    fn keys<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = IndexKey<'db>>, LookupError>
    where
        Self: 'db,
    {
        let _ = metadata(self, database)?;
        Ok(PgIndexEntry::keys(self))
    }

    fn access_method(&self) -> Option<&IndexType> {
        Some(PgIndexEntry::access_method(self))
    }

    fn include_columns<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Column>, LookupError>
    where
        Self: 'db,
    {
        let columns: Vec<&PgColumn> = metadata(self, database)?
            .table()
            .columns(database)?
            .collect();
        Ok(self.include_column_names().iter().filter_map(move |name| {
            columns
                .iter()
                .copied()
                .find(|column| column.column_name() == name)
        }))
    }

    fn predicate(&self) -> Option<&Expr> {
        PgIndexEntry::predicate(self)
    }

    fn nulls_distinct(&self) -> bool {
        PgIndexEntry::nulls_distinct(self)
    }

    fn functions_reached_by<'db, 'call>(
        &'db self,
        call: impl Into<WrittenCall<'call>>,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Function> + 'db, LookupError>
    {
        let call = call.into();
        let metadata = metadata(self, database)?;
        metadata
            .expression_calls()
            .reached_by(PgIndexEntry::expression(self), call)
            .or_else(|| {
                PgIndexEntry::predicate(self)
                    .and_then(|predicate| metadata.predicate_calls().reached_by(predicate, call))
            })
            .ok_or_else(|| LookupError::CallNotInExpression {
                call: call.to_string(),
            })
    }
}

/// Returns the metadata `database` holds for `index`, which one entry type
/// serves for both the index and the unique index collections.
fn metadata<'db>(
    index: &PgIndexEntry,
    database: &'db PgDieselDatabase,
) -> Result<&'db IndexMetadata, LookupError> {
    database
        .index_metadata(index)
        .or_else(|| database.unique_index_metadata(index))
        .ok_or_else(|| LookupError::ObjectNotInDatabase {
            object_kind: ObjectKind::Index,
            object: format!(
                "{}.{}",
                PgIndexEntry::schema(index),
                PgIndexEntry::name(index)
            ),
        })
}

#[cfg(test)]
mod tests {
    use sqlparser::ast::{Ident, IndexColumn, OrderByExpr, OrderByOptions};

    use super::*;
    use crate::{
        PgDieselDatabase, dialect::PostgresDialect, model_metadata::IndexShape,
        shared::models::PgIndex,
    };

    #[test]
    fn an_index_the_database_does_not_hold_is_reported_not_in_the_database() {
        let index = PgIndexEntry::new(
            PgIndex {
                indexrelid: 1,
                indrelid: 2,
                indisunique: false,
                indkey: vec![1],
                indnkeyatts: 1,
            },
            "app".to_string(),
            "absent".to_string(),
            IndexShape::new("btree", Vec::new(), true),
            None,
            vec![IndexColumn {
                column: OrderByExpr {
                    expr: Expr::Identifier(Ident::new("body")),
                    options: OrderByOptions::default(),
                    with_fill: None,
                },
                operator_class: None,
            }],
            None,
        );
        let database = PgDieselDatabase::new("catalog".to_string(), PostgresDialect).into();
        let error =
            IndexLike::constraint_name(&index, &database).expect_err("the database holds no index");
        assert!(matches!(error, LookupError::ObjectNotInDatabase { .. }));
    }
}
