//! Implementation of [`SequenceLike`] for [`PgSequenceEntry`].

use sql_traits::{
    errors::{LookupError, ObjectKind},
    structs::SequenceParameters,
    traits::{
        DatabaseLike, Metadata, SequenceLike, SequenceOrigin, SequenceOwner, TableLike, ViewLike,
    },
};

use crate::{PgDieselDatabase, model_metadata::PgSequenceEntry};

impl Metadata for PgSequenceEntry {
    type Meta = ();
}

impl SequenceLike for PgSequenceEntry {
    type DB = PgDieselDatabase;

    fn sequence_name(&self) -> &str {
        PgSequenceEntry::name(self)
    }

    fn sequence_name_is_quoted(&self) -> bool {
        // Catalog names are stored names: folding again would rename them.
        true
    }

    fn sequence_schema(&self) -> Option<&str> {
        Some(PgSequenceEntry::schema(self))
    }

    fn sequence_schema_is_quoted(&self) -> bool {
        true
    }

    fn parameters(&self) -> SequenceParameters {
        PgSequenceEntry::parameters(self)
    }

    fn owner<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<Option<SequenceOwner<'db, Self::DB>>, LookupError>
    where
        Self: 'db,
    {
        let Some(owned) = self.owner() else {
            return Ok(None);
        };
        if owned.is_view {
            let ordinal =
                usize::try_from(owned.attnum - 1).expect("a stored column number is one-based");
            database
                .views()
                .find(|view| {
                    view.view_schema() == Some(&owned.schema) && view.view_name() == owned.name
                })
                .map(|view| Some(SequenceOwner::ViewColumn { view, ordinal }))
                .ok_or_else(|| LookupError::RelationNotFound {
                    object_name: format!("{}.{}", owned.schema, owned.name),
                })
        } else {
            let table = database
                .tables_and_foreign_tables()
                .find(|table| {
                    table.table_schema() == Some(&owned.schema) && table.table_name() == owned.name
                })
                .ok_or_else(|| LookupError::TableNotFound {
                    object_name: format!("{}.{}", owned.schema, owned.name),
                })?;
            let column =
                owned
                    .column
                    .as_deref()
                    .ok_or_else(|| LookupError::ObjectNotInDatabase {
                        object_kind: ObjectKind::Column,
                        object: format!("{}.{} column {}", owned.schema, owned.name, owned.attnum),
                    })?;
            table
                .columns(database)?
                .find(|candidate| candidate.column_name() == column)
                .map(|column| Some(SequenceOwner::Column(column)))
                .ok_or_else(|| LookupError::ColumnNotFound {
                    table_name: format!("{}.{}", owned.schema, owned.name),
                    column_name: column.to_owned(),
                })
        }
    }

    fn origin(&self) -> SequenceOrigin {
        PgSequenceEntry::origin(self)
    }
}
