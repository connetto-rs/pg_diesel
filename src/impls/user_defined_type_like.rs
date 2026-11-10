//! Implementation of [`UserDefinedTypeLike`] for [`PgUserDefinedType`].

use sql_traits::traits::{Metadata, UserDefinedTypeKind, UserDefinedTypeLike};

use crate::{PgDieselDatabase, model_metadata::PgUserDefinedType};

impl Metadata for PgUserDefinedType {
    type Meta = ();
}

impl UserDefinedTypeLike for PgUserDefinedType {
    type DB = PgDieselDatabase;

    fn name(&self) -> &str {
        PgUserDefinedType::name(self)
    }

    fn name_is_quoted(&self) -> bool {
        // Catalog names are stored names: folding again would rename them.
        true
    }

    fn schema(&self) -> Option<&str> {
        Some(PgUserDefinedType::schema(self))
    }

    fn schema_is_quoted(&self) -> bool {
        true
    }

    fn kind(&self) -> UserDefinedTypeKind {
        PgUserDefinedType::kind(self)
    }
}
