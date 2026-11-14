//! Implementation of [`ExtensionLike`] for [`PgExtensionEntry`].

use sql_traits::traits::{ExtensionLike, Metadata};

use crate::{PgDieselDatabase, model_metadata::PgExtensionEntry};

impl Metadata for PgExtensionEntry {
    type Meta = ();
}

impl ExtensionLike for PgExtensionEntry {
    type DB = PgDieselDatabase;

    fn name(&self) -> &str {
        PgExtensionEntry::name(self)
    }

    fn name_is_quoted(&self) -> bool {
        // Catalog names are stored names: folding again would rename them.
        true
    }

    fn schema(&self) -> Option<&str> {
        Some(PgExtensionEntry::schema(self))
    }

    fn schema_is_quoted(&self) -> bool {
        true
    }

    fn version(&self) -> Option<&str> {
        Some(PgExtensionEntry::version(self))
    }
}
