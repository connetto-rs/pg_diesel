//! Implementation of [`ViewLike`] for [`PgViewDefinition`].

use sql_traits::{
    errors::{LookupError, ObjectKind},
    traits::{DatabaseLike, Metadata, ViewLike},
};
use sqlparser::ast::{Action, Query};

use crate::{PgDieselDatabase, model_metadata::PgViewDefinition};

impl Metadata for PgViewDefinition {
    type Meta = ();
}

impl ViewLike for PgViewDefinition {
    type DB = PgDieselDatabase;

    fn view_name(&self) -> &str {
        self.name()
    }

    fn view_name_is_quoted(&self) -> bool {
        // Catalog names are stored names: folding again would rename them.
        true
    }

    fn view_schema(&self) -> Option<&str> {
        Some(self.schema())
    }

    fn view_schema_is_quoted(&self) -> bool {
        true
    }

    fn is_materialized(&self) -> bool {
        PgViewDefinition::is_materialized(self)
    }

    fn is_security_invoker(&self) -> bool {
        PgViewDefinition::is_security_invoker(self)
    }

    fn is_security_barrier(&self) -> bool {
        PgViewDefinition::is_security_barrier(self)
    }

    fn owner<'db>(&self, database: &'db Self::DB) -> Result<Option<&'db str>, LookupError> {
        Ok(held(self, database)?.owner())
    }

    fn owner_privileges<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db Action>, LookupError> {
        Ok(held(self, database)?.owner_privileges().iter())
    }

    fn definition(&self) -> &Query {
        PgViewDefinition::definition(self)
    }

    fn declared_column_names(&self) -> &[(String, bool)] {
        // `pg_get_viewdef` does not reconstruct the declared column list.
        &[]
    }
}

/// Returns the view `database` holds as `view`.
fn held<'db>(
    view: &PgViewDefinition,
    database: &'db PgDieselDatabase,
) -> Result<&'db PgViewDefinition, LookupError> {
    let held = if view.is_materialized() {
        database.materialized_views().find(|held| *held == view)
    } else {
        database.views().find(|held| *held == view)
    };
    held.ok_or_else(|| LookupError::ObjectNotInDatabase {
        object_kind: ObjectKind::View,
        object: format!("{}.{}", view.schema(), view.name()),
    })
}
