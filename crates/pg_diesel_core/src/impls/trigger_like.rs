//! Implementation of [`TriggerLike`] for [`TriggerMetadata`].

use sql_traits::{
    errors::LookupError,
    structs::TargetName,
    traits::{DatabaseLike, Metadata, TriggerLike},
};
use sqlparser::ast::{TriggerEvent, TriggerObjectKind, TriggerPeriod};

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
