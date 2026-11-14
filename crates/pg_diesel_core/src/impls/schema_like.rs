//! Implementation of [`SchemaLike`] for [`PgSchema`].

use crate::PgDieselDatabase;
use crate::model_metadata::PgSchema;
use sql_traits::traits::{Metadata, SchemaLike};

impl Metadata for PgSchema {
    type Meta = ();
}

impl SchemaLike for PgSchema {
    type DB = PgDieselDatabase;

    fn name(&self) -> &str {
        &self.model().schema_name
    }

    fn authorization(&self) -> Option<&str> {
        Some(&self.model().schema_owner)
    }

    fn public_holds_usage(&self) -> bool {
        self.public_usage()
    }

    fn owner_holds_usage(&self) -> bool {
        self.owner_usage()
    }
}
