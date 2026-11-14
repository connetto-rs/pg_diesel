//! Implementation of [`CheckConstraintLike`] for [`PgCheckConstraint`].

use std::borrow::Cow;

use sql_traits::{
    errors::{LookupError, ObjectKind},
    structs::metadata::CheckMetadata,
    traits::{CheckConstraintLike, DatabaseLike, Metadata},
};
use sqlparser::ast::Function;

use crate::{PgDieselDatabase, model_metadata::PgCheckConstraint};

impl Metadata for PgCheckConstraint {
    type Meta = CheckMetadata<Self>;
}

/// Returns the metadata `database` holds for `constraint`.
fn metadata<'db>(
    constraint: &PgCheckConstraint,
    database: &'db PgDieselDatabase,
) -> Result<&'db CheckMetadata<PgCheckConstraint>, LookupError> {
    database
        .check_constraint_metadata(constraint)
        .ok_or_else(|| LookupError::ObjectNotInDatabase {
            object_kind: ObjectKind::CheckConstraint,
            object: constraint.name().to_owned(),
        })
}

impl CheckConstraintLike for PgCheckConstraint {
    type DB = PgDieselDatabase;

    fn expression<'db>(&'db self, _database: &'db Self::DB) -> &'db sqlparser::ast::Expr {
        PgCheckConstraint::expression(self)
    }

    fn table<'db>(
        &'db self,
        _database: &'db Self::DB,
    ) -> Result<&'db <Self::DB as DatabaseLike>::Table, LookupError> {
        Ok(PgCheckConstraint::table(self))
    }

    fn columns<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Column>, LookupError> {
        Ok(metadata(self, database)?.columns())
    }

    fn functions<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Function> + 'db, LookupError>
    {
        Ok(metadata(self, database)?.functions())
    }

    fn functions_reached_by<'db>(
        &'db self,
        call: &Function,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Function> + 'db, LookupError>
    {
        metadata(self, database)?
            .calls()
            .reached_by(CheckConstraintLike::expression(self, database), call)
            .ok_or_else(|| LookupError::CallNotInExpression {
                call: call.to_string(),
            })
    }

    fn constraint_name<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<Option<Cow<'db, str>>, LookupError> {
        metadata(self, database)?;
        Ok(Some(Cow::Borrowed(self.name())))
    }

    fn is_validated(&self, _database: &Self::DB) -> Result<bool, LookupError> {
        Ok(PgCheckConstraint::is_validated(self))
    }

    fn no_inherit(&self) -> bool {
        PgCheckConstraint::no_inherit(self)
    }
}
