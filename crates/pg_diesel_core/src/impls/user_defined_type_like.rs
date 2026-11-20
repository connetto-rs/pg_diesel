//! Implementation of [`UserDefinedTypeLike`] for [`PgUserDefinedType`].

use sql_traits::traits::{
    DataTypeLike, Enumeration, EnumerationKind, EnumerationMember, Metadata, StructuredField,
    UserDefinedTypeKind, UserDefinedTypeLike,
};

use crate::{
    PgDieselDatabase,
    model_metadata::{PgDataType, PgUserDefinedType},
};

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

    fn enumeration(&self) -> Option<Enumeration<impl Iterator<Item = EnumerationMember<'_>>>> {
        self.enum_labels().map(|labels| {
            Enumeration::new(
                EnumerationKind::Enum,
                None,
                labels
                    .iter()
                    .map(|label| EnumerationMember::new(label, None)),
            )
        })
    }

    fn attributes<'db>(
        &'db self,
        database: &'db PgDieselDatabase,
    ) -> Option<impl Iterator<Item = StructuredField<'db, PgDataType>>> {
        self.composite()?.structured_fields(database)
    }
}
