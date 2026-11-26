//! Implementation of [`TriggerLike`] for [`TriggerMetadata`].

use sql_traits::{
    errors::LookupError,
    structs::{ConstraintDeferral, TargetName, TransitionTable, TriggerEnablement, TriggerRow},
    traits::{DatabaseLike, Metadata, Relation, TableLike, TriggerLike},
};
use sqlparser::ast::{Expr, Statement, TriggerEvent, TriggerObjectKind, TriggerPeriod};

use crate::{
    PgDieselDatabase,
    model_metadata::{TriggerMetadata, decode_deferral},
};

impl Metadata for TriggerMetadata {
    type Meta = ();
}

impl TriggerLike for TriggerMetadata {
    type DB = PgDieselDatabase;

    fn name(&self) -> &str {
        &self.model.tgname
    }

    fn name_is_quoted(&self) -> bool {
        true
    }

    /// The relation is read by the stored name the catalog bound it to, the
    /// kind `pg_class` records deciding the store it lives in.
    fn relation<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<Relation<'db, Self::DB>, LookupError>
    where
        Self: 'db,
    {
        self.target.resolve(database)
    }

    fn target_relation_name(&self) -> TargetName<'_> {
        self.target.target_name()
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

    /// The catalog keeps no written order, so the old table comes first as
    /// `pg_get_triggerdef` prints it, each name the stored one.
    fn transition_tables(&self) -> impl Iterator<Item = TransitionTable<'_>> {
        [
            (TriggerRow::Old, &self.model.tgoldtable),
            (TriggerRow::New, &self.model.tgnewtable),
        ]
        .into_iter()
        .filter_map(|(rows, name)| Some(TransitionTable::new(rows, name.as_deref()?, true)))
    }

    /// Only a partition's copy of a trigger records a parent, and a partition
    /// is a table or a foreign table.
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
        let (Relation::Table(table) | Relation::ForeignTable(table)) = self.relation(database)?
        else {
            return Ok(None);
        };
        TableLike::partition_root(table, database)?
            .map(|parent_table| TableLike::triggers(parent_table, database))
            .transpose()
            .map(|triggers| triggers?.find(|trigger| trigger.model.oid == self.model.tgparentid))
    }

    fn enablement(&self, _database: &Self::DB) -> Result<TriggerEnablement, LookupError> {
        Ok(self.enablement)
    }

    /// A partition's copy of a constraint trigger carries the deferral flags
    /// of the trigger it copies.
    fn deferral(&self) -> Option<ConstraintDeferral> {
        (self.model.tgconstraint != 0)
            .then(|| decode_deferral(self.model.tgdeferrable, self.model.tginitdeferred))
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

    /// A PostgreSQL trigger always executes a function and runs no
    /// statement inline.
    fn statements(&self) -> &[Statement] {
        &[]
    }
}
