//! Implementation of [`ForeignKeyLike`] for [`PgForeignKey`].

use std::borrow::Cow;

use sql_traits::{
    errors::{LookupError, ObjectKind},
    structs::{IdentifierCase, TargetName},
    traits::{DatabaseLike, ForeignKeyLike, Metadata, TableLike},
};

use crate::{PgDieselDatabase, database::KeyColumnUsageMetadata, model_metadata::PgForeignKey};

impl Metadata for PgForeignKey {
    type Meta = KeyColumnUsageMetadata;
}

/// Returns the metadata `database` holds for `key`.
fn metadata<'db>(
    key: &PgForeignKey,
    database: &'db PgDieselDatabase,
) -> Result<&'db KeyColumnUsageMetadata, LookupError> {
    database
        .foreign_key_metadata(key)
        .ok_or_else(|| LookupError::ObjectNotInDatabase {
            object_kind: ObjectKind::Table,
            object: key.model().constraint_name.clone(),
        })
}

impl ForeignKeyLike for PgForeignKey {
    type DB = PgDieselDatabase;

    fn foreign_key_name(&self) -> Option<&str> {
        Some(&self.model().constraint_name)
    }

    fn constraint_name<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<Option<Cow<'db, str>>, LookupError> {
        metadata(self, database)?;
        Ok(Some(Cow::Borrowed(self.model().constraint_name.as_str())))
    }

    fn referenced_table_name(&self) -> TargetName<'_> {
        TargetName::new(self.referenced_name(), true).with_schema(self.referenced_schema(), true)
    }

    fn referenced_table<'db>(
        &self,
        database: &'db Self::DB,
    ) -> Result<&'db <Self::DB as DatabaseLike>::Table, LookupError> {
        database
            .tables()
            .find(|table| {
                table.schema() == self.referenced_schema() && table.name() == self.referenced_name()
            })
            .ok_or_else(|| LookupError::TableNotFound {
                object_name: format!("{}.{}", self.referenced_schema(), self.referenced_name()),
            })
    }

    fn host_table<'db>(&'db self, database: &'db Self::DB) -> &'db <Self::DB as DatabaseLike>::Table
    where
        Self: 'db,
    {
        database
            .foreign_key_metadata(self)
            .expect("Foreign key must exist in database")
            .host_table()
    }

    fn on_delete(&self, database: &Self::DB) -> sqlparser::ast::ReferentialAction {
        database
            .foreign_key_metadata(self)
            .expect("Foreign key must exist in database")
            .on_delete()
    }

    fn on_update(&self, database: &Self::DB) -> sqlparser::ast::ReferentialAction {
        database
            .foreign_key_metadata(self)
            .expect("Foreign key must exist in database")
            .on_update()
    }

    fn match_kind(&self, database: &Self::DB) -> sqlparser::ast::ConstraintReferenceMatchKind {
        database
            .foreign_key_metadata(self)
            .expect("Foreign key must exist in database")
            .match_kind()
    }

    fn host_columns<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Column>, LookupError>
    where
        Self: 'db,
    {
        let names = metadata(self, database)?.host_columns();
        let table = self.host_table(database);
        Ok(names.iter().filter_map(|name| {
            table
                .column(name, database, IdentifierCase::Exact)
                .ok()
                .flatten()
        }))
    }

    fn referenced_columns<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Column>, LookupError>
    where
        Self: 'db,
    {
        let names = metadata(self, database)?.referenced_columns();
        let table = self.referenced_table(database)?;
        Ok(names.iter().filter_map(|name| {
            table
                .column(name, database, IdentifierCase::Exact)
                .ok()
                .flatten()
        }))
    }
}
