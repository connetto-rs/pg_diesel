//! Implementation of [`TriggerLike`] for [`TriggerMetadata`].

use sql_traits::{
    errors::LookupError,
    structs::{TargetName, TriggerEnablement},
    traits::{DatabaseLike, Metadata, TableLike, TriggerLike},
};
use sqlparser::ast::{Expr, TriggerEvent, TriggerObjectKind, TriggerPeriod};

use crate::{PgDieselDatabase, model_metadata::TriggerMetadata};

impl Metadata for TriggerMetadata {
    type Meta = ();
}

impl TriggerLike for TriggerMetadata {
    type DB = PgDieselDatabase;

    fn name(&self) -> &str {
        &self.model.tgname
    }

    fn table<'db>(
        &'db self,
        _database: &'db Self::DB,
    ) -> Result<&'db <Self::DB as DatabaseLike>::Table, LookupError>
    where
        Self: 'db,
    {
        Ok(&self.table)
    }

    fn target_table_name(&self) -> TargetName<'_> {
        // Catalog names are stored names: folding again would rename them.
        TargetName::new(self.table.name(), true).with_schema(self.table.schema(), true)
    }

    fn events(&self) -> &[TriggerEvent] {
        &self.events
    }

    fn timing(&self) -> Option<TriggerPeriod> {
        self.timing
    }

    fn orientation(&self) -> Option<TriggerObjectKind> {
        self.orientation
    }

    fn condition(&self) -> Option<&Expr> {
        self.condition.as_ref()
    }

    fn parent_trigger<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<Option<&'db <Self::DB as DatabaseLike>::Trigger>, LookupError>
    where
        Self: 'db,
    {
        if self.model.tgparentid == 0 {
            return Ok(None);
        }
        TableLike::partition_root(self.table.as_ref(), database)?
            .map(|parent_table| TableLike::triggers(parent_table, database))
            .transpose()
            .map(|triggers| triggers?.find(|trigger| trigger.model.oid == self.model.tgparentid))
    }

    fn enablement(&self, _database: &Self::DB) -> Result<TriggerEnablement, LookupError> {
        Ok(self.enablement)
    }

    fn function<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Option<&'db <Self::DB as DatabaseLike>::Function>
    where
        Self: 'db,
    {
        database
            .functions()
            .find(|function| function.oid() == self.function_oid)
    }

    fn function_name(&self) -> Option<&str> {
        Some(&self.function_name)
    }
}
