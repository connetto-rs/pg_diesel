//! Implementation of [`OperatorLike`] for [`PgOperatorEntry`].

use sql_traits::{
    structs::TargetName,
    traits::{Metadata, OperatorKind, OperatorLike},
};

use crate::{
    PgDieselDatabase,
    model_metadata::{PgFunction, PgOperatorEntry},
};

impl Metadata for PgOperatorEntry {
    type Meta = ();
}

impl OperatorLike for PgOperatorEntry {
    type DB = PgDieselDatabase;

    fn target_name(&self) -> TargetName<'_> {
        PgOperatorEntry::target_name(self)
    }

    fn kind(&self) -> OperatorKind {
        PgOperatorEntry::kind(self)
    }

    fn left_type<'db>(&'db self, _database: &'db Self::DB) -> Option<TargetName<'db>> {
        self.declared_left_type()
    }

    fn right_type<'db>(&'db self, _database: &'db Self::DB) -> Option<TargetName<'db>> {
        self.declared_right_type()
    }

    fn result_type<'db>(&'db self, _database: &'db Self::DB) -> Option<TargetName<'db>> {
        self.declared_result_type()
    }

    fn implementation<'db>(&'db self, _database: &'db Self::DB) -> Option<&'db PgFunction> {
        self.implementing_function()
    }
}
