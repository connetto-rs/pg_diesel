//! Implementation of [`IndexLike`] and [`TableIndexLike`] for
//! [`PgIndexEntry`].

use std::borrow::Cow;

use sql_traits::{
    errors::{LookupError, ObjectKind},
    structs::{ConstraintDeferral, IndexKey, WrittenCall},
    traits::{DatabaseLike, IndexLike, Metadata, Relation, TableIndexLike, TableLike},
};
use sqlparser::ast::{Expr, IndexType};

use crate::{
    PgDieselDatabase,
    model_metadata::{IndexMetadata, IndexedRelation, OnTable, PgColumn, PgIndexEntry},
};

impl<On: IndexedRelation> Metadata for PgIndexEntry<On> {
    type Meta = IndexMetadata<On>;
}

impl<On: IndexedRelation> IndexLike for PgIndexEntry<On> {
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

    fn relation<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<Relation<'db, Self::DB>, LookupError>
    where
        Self: 'db,
    {
        Ok(On::relation(metadata(self, database)?.relation()))
    }

    fn access_method(&self) -> Option<&IndexType> {
        Some(PgIndexEntry::access_method(self))
    }

    fn include_column_names(&self) -> impl Iterator<Item = (&str, bool)> {
        // Catalog names are stored names: folding again would rename them.
        PgIndexEntry::include_column_names(self)
            .iter()
            .map(|name| (name.as_str(), true))
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

impl TableIndexLike for PgIndexEntry<OnTable> {
    fn table<'db>(&'db self, database: &'db Self::DB) -> &'db <Self::DB as DatabaseLike>::Table
    where
        Self: 'db,
    {
        metadata(self, database)
            .expect("an index the database lists carries its metadata")
            .relation()
    }

    fn deferral(&self, _database: &Self::DB) -> ConstraintDeferral {
        PgIndexEntry::constraint_deferral(self)
    }

    fn include_columns<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Column>, LookupError>
    where
        Self: 'db,
    {
        let columns: Vec<&PgColumn> = metadata(self, database)?
            .relation()
            .columns(database)?
            .collect();
        Ok(PgIndexEntry::include_column_names(self)
            .iter()
            .filter_map(move |name| {
                columns
                    .iter()
                    .copied()
                    .find(|column| column.column_name() == name)
            }))
    }
}

/// Returns the metadata `database` holds for `index`.
fn metadata<'db, On: IndexedRelation>(
    index: &PgIndexEntry<On>,
    database: &'db PgDieselDatabase,
) -> Result<&'db IndexMetadata<On>, LookupError> {
    On::metadata(index, database).ok_or_else(|| LookupError::ObjectNotInDatabase {
        object_kind: ObjectKind::Index,
        object: format!(
            "{}.{}",
            PgIndexEntry::schema(index),
            PgIndexEntry::name(index)
        ),
    })
}
