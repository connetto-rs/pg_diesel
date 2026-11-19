//! Submodule providing the kinds of relation a [`PgIndexEntry`] is built on.

use std::{fmt::Debug, hash::Hash};

use sql_traits::traits::Relation;

use crate::{
    PgDieselDatabase,
    model_metadata::{IndexMetadata, PgIndexEntry, PgTable, PgViewDefinition},
};

/// The kind of relation an index is built on, which names the relation its
/// metadata holds and the collection of the database that lists it.
pub trait IndexedRelation: Debug + Clone + Ord + Hash + Send + Sync + 'static {
    /// The relation the index is built on.
    type Relation: Debug + Clone + Send + Sync;

    /// Returns the metadata `database` holds for `index`, `None` when it does
    /// not hold the index.
    fn metadata<'db>(
        index: &PgIndexEntry<Self>,
        database: &'db PgDieselDatabase,
    ) -> Option<&'db IndexMetadata<Self>>;

    /// Returns `relation` as the relation an index answers it is built on.
    fn relation(relation: &Self::Relation) -> Relation<'_, PgDieselDatabase>;
}

/// Marks an index built on a table, which `pg_index.indrelid` names with
/// `relkind` `r` or `p`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OnTable {}

/// Marks an index built on a materialized view, which `pg_index.indrelid`
/// names with `relkind` `m`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OnMaterializedView {}

impl IndexedRelation for OnTable {
    type Relation = PgTable;

    /// Looks the index up among the plain indexes and then among the unique
    /// ones, since one entry type serves both collections.
    fn metadata<'db>(
        index: &PgIndexEntry<Self>,
        database: &'db PgDieselDatabase,
    ) -> Option<&'db IndexMetadata<Self>> {
        database
            .index_metadata(index)
            .or_else(|| database.unique_index_metadata(index))
    }

    fn relation(relation: &Self::Relation) -> Relation<'_, PgDieselDatabase> {
        Relation::Table(relation)
    }
}

impl IndexedRelation for OnMaterializedView {
    type Relation = PgViewDefinition;

    fn metadata<'db>(
        index: &PgIndexEntry<Self>,
        database: &'db PgDieselDatabase,
    ) -> Option<&'db IndexMetadata<Self>> {
        database.materialized_view_index_metadata(index)
    }

    fn relation(relation: &Self::Relation) -> Relation<'_, PgDieselDatabase> {
        Relation::MaterializedView(relation)
    }
}
