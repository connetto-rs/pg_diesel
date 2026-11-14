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
        let missing_column = || {
            let name = owned.column.as_deref().map_or_else(
                || format!("{}.{} column {}", owned.schema, owned.name, owned.attnum),
                |column| format!("{}.{}.{column}", owned.schema, owned.name),
            );
            ObjectKind::Column.not_in_database(&name)
        };
        if owned.is_view {
            let view = database
                .views()
                .find(|view| {
                    view.view_schema() == Some(&owned.schema) && view.view_name() == owned.name
                })
                .ok_or_else(|| {
                    ObjectKind::View.not_in_database(&format!("{}.{}", owned.schema, owned.name))
                })?;
            owned.column.as_deref().ok_or_else(missing_column)?;
            let ordinal = owned
                .attnum
                .checked_sub(1)
                .and_then(|ordinal| usize::try_from(ordinal).ok())
                .ok_or_else(missing_column)?;
            Ok(Some(SequenceOwner::ViewColumn { view, ordinal }))
        } else {
            let table = database
                .tables_and_foreign_tables()
                .find(|table| {
                    table.table_schema() == Some(&owned.schema) && table.table_name() == owned.name
                })
                .ok_or_else(|| {
                    ObjectKind::Table.not_in_database(&format!("{}.{}", owned.schema, owned.name))
                })?;
            let column = owned.column.as_deref().ok_or_else(missing_column)?;
            table
                .columns(database)?
                .find(|candidate| {
                    candidate.column_name() == column
                        && candidate.model().ordinal_position == owned.attnum
                })
                .map(|column| Some(SequenceOwner::Column(column)))
                .ok_or_else(missing_column)
        }
    }

    fn origin(&self) -> SequenceOrigin {
        PgSequenceEntry::origin(self)
    }
}
