//! Stored cast types, contexts, methods and exact implementing functions.

use std::sync::Arc;

use diesel::{PgConnection, RunQueryDsl, SelectableHelper};
use sql_traits::{
    structs::TargetName,
    traits::{CastContext, CastLike, CastMethod, Metadata},
};

use crate::{PgDieselDatabase, models::PgCast};

use super::{
    PgFunction,
    catalog_identity::{CatalogIdentity, CatalogObjects},
};

/// A cast resolved by catalog type and function OIDs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgCastEntry {
    /// The qualified source type stored in `pg_type`.
    source: Arc<CatalogIdentity>,
    /// The qualified target type stored in `pg_type`.
    target: Arc<CatalogIdentity>,
    /// The contexts in which the cast applies.
    context: CastContext,
    /// The conversion method recorded in `pg_cast`.
    method: CastMethod,
    /// The exact loaded function referenced by `castfunc`, when applicable.
    function: Option<Arc<PgFunction>>,
}

impl PgCastEntry {
    /// Loads casts with their stored type identities and exact loaded functions.
    ///
    /// # Errors
    ///
    /// Returns an error if a query fails, a type is unavailable or a cast code is unsupported.
    pub(crate) fn load(
        connection: &mut PgConnection,
        objects: &mut CatalogObjects<'_>,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::schema::pg_catalog::pg_cast::pg_cast;
        use diesel::QueryDsl;

        pg_cast::table
            .select(PgCast::as_select())
            .load::<PgCast>(connection)?
            .into_iter()
            .map(|model| {
                let context = match model.castcontext.as_str() {
                    "e" => CastContext::Explicit,
                    "a" => CastContext::Assignment,
                    "i" => CastContext::Implicit,
                    context => {
                        return Err(diesel::result::Error::DeserializationError(
                            format!("unsupported cast context {context}").into(),
                        ));
                    }
                };
                let method = match model.castmethod.as_str() {
                    "f" => CastMethod::Function,
                    "i" => CastMethod::InOut,
                    "b" => CastMethod::Binary,
                    method => {
                        return Err(diesel::result::Error::DeserializationError(
                            format!("unsupported cast method {method}").into(),
                        ));
                    }
                };
                Ok(Self {
                    source: objects
                        .type_identity(model.castsource)
                        .ok_or(diesel::result::Error::NotFound)?,
                    target: objects
                        .type_identity(model.casttarget)
                        .ok_or(diesel::result::Error::NotFound)?,
                    context,
                    method,
                    function: if method == CastMethod::Function {
                        objects.function(model.castfunc)
                    } else {
                        None
                    },
                })
            })
            .collect()
    }
}

impl Metadata for PgCastEntry {
    type Meta = ();
}

impl CastLike for PgCastEntry {
    type DB = PgDieselDatabase;

    fn source_type<'db>(&'db self, _database: &'db Self::DB) -> TargetName<'db> {
        self.source.target()
    }

    fn target_type<'db>(&'db self, _database: &'db Self::DB) -> TargetName<'db> {
        self.target.target()
    }

    fn context(&self) -> CastContext {
        self.context
    }

    fn method(&self) -> CastMethod {
        self.method
    }

    fn function<'db>(&'db self, _database: &'db Self::DB) -> Option<&'db PgFunction> {
        self.function.as_deref()
    }
}
