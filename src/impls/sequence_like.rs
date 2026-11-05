//! Implementation of [`SequenceLike`] for [`PgSequenceEntry`].

use sql_traits::{
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

    fn owner<'db>(&'db self, database: &'db Self::DB) -> Option<SequenceOwner<'db, Self::DB>>
    where
        Self: 'db,
    {
        let owned = self.owner()?;
        let ordinal =
            usize::try_from(owned.attnum - 1).expect("a stored column number is one-based");
        if owned.is_view {
            database
                .views()
                .find(|view| {
                    view.view_schema() == Some(&owned.schema) && view.view_name() == owned.name
                })
                .map(|view| SequenceOwner::ViewColumn { view, ordinal })
        } else {
            let table = database.tables_and_foreign_tables().find(|table| {
                table.table_schema() == Some(&owned.schema) && table.table_name() == owned.name
            })?;
            let column = owned.column.as_deref()?;
            table
                .columns(database)
                .ok()?
                .find(|candidate| candidate.column_name() == column)
                .map(SequenceOwner::Column)
        }
    }

    fn origin(&self) -> SequenceOrigin {
        PgSequenceEntry::origin(self)
    }
}
