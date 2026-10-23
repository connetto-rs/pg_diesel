//! Implementation of [`TableLike`] for [`PgTable`].

use sql_traits::{
    errors::LookupError,
    traits::{DatabaseLike, Metadata, PartitionStrategy, TableLike},
};

use crate::{
    PgDieselDatabase,
    model_metadata::{PgTable, TableMetadata},
};

impl Metadata for PgTable {
    type Meta = TableMetadata;
}

/// Returns the metadata `database` holds for `table`.
fn metadata<'db>(
    table: &PgTable,
    database: &'db PgDieselDatabase,
) -> Result<&'db TableMetadata, LookupError> {
    database
        .table_metadata(table)
        .ok_or_else(|| LookupError::ObjectNotInDatabase {
            object_kind: sql_traits::errors::ObjectKind::Table,
            object: format!("{}.{}", table.schema(), table.name()),
        })
}

impl TableLike for PgTable {
    type DB = PgDieselDatabase;

    fn table_name(&self) -> &str {
        self.name()
    }

    fn table_name_is_quoted(&self) -> bool {
        // Catalog names are stored names: folding again would rename them.
        true
    }

    fn table_schema(&self) -> Option<&str> {
        Some(self.schema())
    }

    fn table_schema_is_quoted(&self) -> bool {
        true
    }

    fn table_id(&self, database: &Self::DB) -> Option<usize> {
        database.table_id(self)
    }

    fn table_doc<'db>(&'db self, database: &'db Self::DB) -> Result<Option<&'db str>, LookupError>
    where
        Self: 'db,
    {
        Ok(metadata(self, database)?
            .description()
            .map(|desc| desc.description.as_str()))
    }

    fn owner<'db>(&self, database: &'db Self::DB) -> Result<Option<&'db str>, LookupError> {
        Ok(metadata(self, database)?.owner())
    }

    fn columns<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Column>, LookupError>
    where
        Self: 'db,
    {
        Ok(metadata(self, database)?.columns())
    }

    fn local_columns<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Column>, LookupError>
    where
        Self: 'db,
    {
        Ok(metadata(self, database)?.local_columns())
    }

    fn inherits_from<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Table>, LookupError>
    where
        Self: 'db,
    {
        let parents = metadata(self, database)?;
        Ok(database.tables().filter(move |candidate| {
            parents
                .parents()
                .any(|(schema, name)| candidate.schema() == schema && candidate.name() == name)
        }))
    }

    fn partition_root<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<Option<&'db <Self::DB as DatabaseLike>::Table>, LookupError>
    where
        Self: 'db,
    {
        let Some((schema, name)) = metadata(self, database)?.partition_root() else {
            return Ok(None);
        };
        Ok(database
            .tables()
            .find(|candidate| candidate.schema() == schema && candidate.name() == name))
    }

    fn partition_strategy(&self) -> Option<PartitionStrategy> {
        PgTable::partition_strategy(self)
    }

    fn indices<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Index>, LookupError>
    where
        Self: 'db,
    {
        Ok(metadata(self, database)?.indices())
    }

    fn primary_key_columns<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Column>, LookupError>
    where
        Self: 'db,
    {
        Ok(metadata(self, database)?.primary_key_columns())
    }

    fn foreign_keys<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::ForeignKey>, LookupError>
    where
        Self: 'db,
    {
        Ok(metadata(self, database)?.foreign_keys())
    }

    fn check_constraints<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::CheckConstraint>, LookupError>
    where
        Self: 'db,
    {
        Ok(metadata(self, database)?.check_constraints())
    }

    fn unique_indices<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::UniqueIndex>, LookupError>
    where
        Self: 'db,
    {
        Ok(metadata(self, database)?.unique_indices())
    }

    fn policies<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Policy>, LookupError>
    where
        Self: 'db,
    {
        Ok(metadata(self, database)?.policies().map(AsRef::as_ref))
    }

    fn has_row_level_security(&self, database: &Self::DB) -> Result<bool, LookupError> {
        Ok(metadata(self, database)?.row_security())
    }

    fn has_forced_row_level_security(&self, database: &Self::DB) -> Result<bool, LookupError> {
        Ok(metadata(self, database)?.forced_row_security())
    }
}
