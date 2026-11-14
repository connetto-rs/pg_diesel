//! Implementation of [`DomainLike`] for [`PgDomainEntry`].

use sql_traits::{
    errors::{LookupError, ObjectKind},
    traits::{DatabaseLike, DomainLike, Metadata},
};
use sqlparser::ast::{Expr, Function};

use crate::{
    PgDieselDatabase,
    model_metadata::{DomainMetadata, PgDomainEntry},
};

impl Metadata for PgDomainEntry {
    type Meta = DomainMetadata;
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

    fn default_expression(&self) -> Option<&Expr> {
        PgDomainEntry::default_expression(self)
    }

    fn check_expressions(&self) -> impl Iterator<Item = &Expr> {
        PgDomainEntry::check_expressions(self).iter()
    }

    fn functions_reached_by<'db>(
        &'db self,
        call: &Function,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Function> + 'db, LookupError>
    {
        let metadata = database
            .domain_metadata(self)
            .ok_or_else(|| ObjectKind::Domain.not_in_database(PgDomainEntry::name(self)))?;
        PgDomainEntry::default_expression(self)
            .and_then(|default| metadata.default_calls().reached_by(default, call))
            .or_else(|| {
                PgDomainEntry::check_expressions(self)
                    .iter()
                    .zip(metadata.check_calls())
                    .find_map(|(check, calls)| calls.reached_by(check, call))
            })
            .ok_or_else(|| LookupError::CallNotInExpression {
                call: call.to_string(),
            })
    }
}
