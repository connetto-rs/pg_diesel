//! Implementation of [`TableLike`] for [`PgTable`].

use std::borrow::Cow;

use sql_traits::{
    errors::LookupError,
    structs::{
        ActingRole, ForeignTableSource, PartitionBound, ReplicaIdentity, ReplicaIdentityIndex,
        TablePrivileges,
    },
    traits::{DatabaseLike, Metadata, PartitionStrategy, TableLike},
};
use sqlparser::ast::Action;

use crate::{
    PgDieselDatabase,
    model_metadata::{PgTable, ReplicaIdentitySetting, TableMetadata},
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
        // `pg_inherits.inhseqno` keeps the order the `INHERITS` list wrote.
        let parents = metadata(self, database)?;
        let parents: Result<Vec<_>, _> = parents
            .parents()
            .map(|(schema, name)| {
                database
                    .tables()
                    .find(|candidate| candidate.schema() == schema && candidate.name() == name)
                    .ok_or_else(|| LookupError::TableNotFound {
                        object_name: format!("{schema}.{name}"),
                    })
            })
            .collect();
        Ok(parents?.into_iter())
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
        database
            .tables()
            .find(|candidate| candidate.schema() == schema && candidate.name() == name)
            .map(Some)
            .ok_or_else(|| LookupError::TableNotFound {
                object_name: format!("{schema}.{name}"),
            })
    }

    fn of_type<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<Option<&'db <Self::DB as DatabaseLike>::UserDefinedType>, LookupError>
    where
        Self: 'db,
    {
        metadata(self, database)?;
        let Some((schema, name)) = self.of_type_name() else {
            return Ok(None);
        };
        database
            .user_defined_types()
            .find(|candidate| candidate.schema() == schema && candidate.name() == name)
            .map(Some)
            .ok_or_else(|| LookupError::ObjectNotInDatabase {
                object_kind: sql_traits::errors::ObjectKind::Type,
                object: format!("{schema}.{name}"),
            })
    }

    fn partition_strategy(&self) -> Option<PartitionStrategy> {
        PgTable::partition_strategy(self)
    }

    fn partition_bound(&self) -> Option<PartitionBound> {
        PgTable::partition_bound(self).cloned()
    }

    fn is_unlogged(&self) -> bool {
        PgTable::unlogged(self)
    }

    fn storage_parameters(&self) -> impl Iterator<Item = (Cow<'_, str>, Cow<'_, str>)> {
        PgTable::storage_parameters(self)
            .iter()
            .map(|(name, value)| (Cow::Borrowed(name.as_str()), Cow::Borrowed(value.as_str())))
    }

    fn toast_storage_parameters(&self) -> impl Iterator<Item = (Cow<'_, str>, Cow<'_, str>)> {
        PgTable::toast_storage_parameters(self)
            .iter()
            .map(|(name, value)| (Cow::Borrowed(name.as_str()), Cow::Borrowed(value.as_str())))
    }

    fn foreign_source<'db>(
        &self,
        database: &'db Self::DB,
    ) -> Result<Option<&'db ForeignTableSource>, LookupError> {
        Ok(metadata(self, database)?.foreign_source())
    }

    fn owner_privileges<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db Action>, LookupError> {
        Ok(metadata(self, database)?.owner_privileges().iter())
    }

    fn possible_privileges<'db>(
        &'db self,
        role: impl Into<ActingRole<'db, <Self::DB as DatabaseLike>::Role>>,
        database: &'db Self::DB,
    ) -> Result<TablePrivileges<&'db <Self::DB as DatabaseLike>::Column>, LookupError>
    where
        Self: 'db,
    {
        // The catalog's access control list is complete, so every privilege
        // it leaves out is one the role lacks.
        self.privileges(role, database)
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

    fn exclusion_indices<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::ExclusionIndex>, LookupError>
    where
        Self: 'db,
    {
        Ok(metadata(self, database)?.exclusion_indices())
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

    fn replica_identity<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<ReplicaIdentity<'db, Self::DB>, LookupError>
    where
        Self: 'db,
    {
        let metadata = metadata(self, database)?;
        Ok(match metadata.replica_identity() {
            ReplicaIdentitySetting::Default => ReplicaIdentity::Default,
            ReplicaIdentitySetting::Nothing => ReplicaIdentity::Nothing,
            ReplicaIdentitySetting::Full => ReplicaIdentity::Full,
            // A dropped index leaves `relreplident` at `i` with no index
            // holding `indisreplident`.
            ReplicaIdentitySetting::Index => ReplicaIdentity::Index(
                metadata
                    .unique_indices()
                    .find(|index| index.is_replica_identity())
                    .map(ReplicaIdentityIndex::UniqueIndex)
                    .or_else(|| {
                        metadata
                            .indices()
                            .find(|index| index.is_replica_identity())
                            .map(ReplicaIdentityIndex::Index)
                    }),
            ),
        })
    }
}
