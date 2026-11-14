//! Implementation of [`DomainLike`] for [`PgDomainEntry`].

use sql_traits::traits::{DatabaseLike, DomainLike, Metadata};

use crate::{PgDieselDatabase, model_metadata::PgDomainEntry};

impl Metadata for PgDomainEntry {
    type Meta = ();
}

impl DomainLike for PgDomainEntry {
    type DB = PgDieselDatabase;

    fn name(&self) -> &str {
        PgDomainEntry::name(self)
    }

    fn name_is_quoted(&self) -> bool {
        // Catalog names are stored names: folding again would rename them.
        true
    }

    fn schema(&self) -> Option<&str> {
        Some(PgDomainEntry::schema(self))
    }

    fn schema_is_quoted(&self) -> bool {
        true
    }

    /// The type the domain declares over, held beside it since the build
    /// resolved it from the catalog.
    fn data_type<'db>(
        &'db self,
        _database: &'db Self::DB,
    ) -> &'db <Self::DB as DatabaseLike>::DataType {
        PgDomainEntry::data_type(self)
    }
}
