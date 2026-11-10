//! Implementation of [`IndexLike`] for [`PgIndexEntry`].

use std::borrow::Cow;

use sql_traits::{
    errors::{LookupError, ObjectKind},
    structs::metadata::UniqueIndexMetadata,
    traits::{DatabaseLike, IndexLike, Metadata},
};
use sqlparser::ast::Expr;

use crate::{PgDieselDatabase, model_metadata::PgIndexEntry};

impl Metadata for PgIndexEntry {
    type Meta = UniqueIndexMetadata<Self>;
}

impl IndexLike for PgIndexEntry {
    type DB = PgDieselDatabase;

    fn name(&self) -> Option<&str> {
        Some(PgIndexEntry::name(self))
    }

    fn is_unique(&self) -> bool {
        PgIndexEntry::model(self).indisunique
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
        Ok(metadata(self, database)?.expression())
    }
}

/// Returns the metadata `database` holds for `index`, which one entry type
/// serves for both the index and the unique index collections.
fn metadata<'db>(
    index: &PgIndexEntry,
    database: &'db PgDieselDatabase,
) -> Result<&'db UniqueIndexMetadata<PgIndexEntry>, LookupError> {
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
    use super::*;
    use crate::{PgDieselDatabase, dialect::PostgresDialect, models::PgIndex};

    #[test]
    fn an_index_the_database_does_not_hold_is_reported_not_in_the_database() {
        let index = PgIndexEntry::new(
            PgIndex {
                indexrelid: 1,
                indrelid: 2,
                indnatts: 1,
                indnkeyatts: 1,
                indisunique: false,
                indkey: vec![1],
                #[cfg(any(
                    feature = "postgres-15",
                    feature = "postgres-16",
                    feature = "postgres-17",
                    feature = "postgres-18"
                ))]
                indnullsnotdistinct: false,
                indisprimary: false,
                indisexclusion: false,
                indimmediate: true,
                indisclustered: false,
                indisvalid: true,
                indcheckxmin: false,
                indisready: true,
                indislive: true,
                indisreplident: false,
                indcollation: Vec::new(),
                indclass: Vec::new(),
                indoption: Vec::new(),
                indexprs: None,
                indpred: None,
            },
            "app".to_string(),
            "absent".to_string(),
            None,
        );
        let database = PgDieselDatabase::new("catalog".to_string(), PostgresDialect).into();
        let error =
            IndexLike::constraint_name(&index, &database).expect_err("the database holds no index");
        assert!(matches!(error, LookupError::ObjectNotInDatabase { .. }));
    }
}
