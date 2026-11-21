//! Rows read through the shared-column relations of [`super::schema`].

use diesel::{Queryable, Selectable};

/// The columns of a `pg_index` row the catalog loaders read.
#[derive(Clone, Queryable, Selectable, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[diesel(table_name = super::schema::pg_catalog::pg_index)]
pub(crate) struct PgIndex {
    /// The OID of the index.
    pub(crate) indexrelid: u32,
    /// The OID of the table this index belongs to.
    pub(crate) indrelid: u32,
    /// `true` if the index enforces uniqueness.
    pub(crate) indisunique: bool,
    /// `true` if the index is the table's replica identity, which
    /// `ALTER TABLE ... REPLICA IDENTITY USING INDEX` chose.
    pub(crate) indisreplident: bool,
    /// The column numbers of the indexed columns, `0` for an expression,
    /// the key first and the `INCLUDE` list after it.
    pub(crate) indkey: Vec<i16>,
    /// The number of key columns, those `indkey` lists before the `INCLUDE`
    /// list.
    pub(crate) indnkeyatts: i16,
}

impl PgIndex {
    /// Returns the column numbers of the `INCLUDE` list, in the order
    /// written.
    pub(crate) fn included_attributes(&self) -> &[i16] {
        self.indkey.split_at(self.key_count()).1
    }

    /// Returns how many entries of `indkey` belong to the key.
    fn key_count(&self) -> usize {
        usize::try_from(self.indnkeyatts)
            .expect("pg_index.indnkeyatts counts columns")
            .min(self.indkey.len())
    }
}
