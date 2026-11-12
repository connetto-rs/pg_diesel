//! Stored operator signatures and exact implementing functions.

use std::sync::Arc;

use diesel::{PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use sql_traits::{
    structs::TargetName,
    traits::{Metadata, OperatorKind, OperatorLike},
};

use crate::{PgDieselDatabase, model_metadata::PgFunction, models::PgOperator};

use super::catalog_identity::{CatalogIdentity, CatalogObjects};

/// An operator signature resolved by catalog OIDs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgOperatorEntry {
    /// The operator's stored identity.
    target: CatalogIdentity,
    /// The operand positions recorded independently of type availability.
    kind: OperatorKind,
    /// The left operand type, absent for prefix operators.
    left: Option<Arc<CatalogIdentity>>,
    /// The right operand type, when available.
    right: Option<Arc<CatalogIdentity>>,
    /// The result type, absent for shells.
    result: Option<Arc<CatalogIdentity>>,
    /// The exact loaded function referenced by `oprcode`.
    implementation: Option<Arc<PgFunction>>,
}

impl PgOperatorEntry {
    /// Loads operator signatures with their exact loaded function objects.
    ///
    /// # Errors
    ///
    /// Returns an error if a catalog query fails or an operator kind is unsupported.
    pub(crate) fn load(
        connection: &mut PgConnection,
        objects: &mut CatalogObjects<'_>,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::schema::pg_catalog::pg_operator::pg_operator;

        pg_operator::table
            .select(PgOperator::as_select())
            .load::<PgOperator>(connection)?
            .into_iter()
            .map(|model| {
                let kind = match model.oprkind.as_str() {
                    "b" => OperatorKind::Binary,
                    "l" => OperatorKind::Prefix,
                    kind => {
                        return Err(diesel::result::Error::DeserializationError(
                            format!("unsupported operator kind {kind}").into(),
                        ));
                    }
                };
                Ok(Self {
                    target: CatalogIdentity {
                        schema: objects
                            .namespace(model.oprnamespace)
                            .ok_or(diesel::result::Error::NotFound)?,
                        name: model.oprname,
                    },
                    kind,
                    left: objects.type_identity(model.oprleft),
                    right: objects.type_identity(model.oprright),
                    result: objects.type_identity(model.oprresult),
                    implementation: objects.function(model.oprcode),
                })
            })
            .collect()
    }
}

impl Metadata for PgOperatorEntry {
    type Meta = ();
}

impl OperatorLike for PgOperatorEntry {
    type DB = PgDieselDatabase;

    fn target_name(&self) -> TargetName<'_> {
        self.target.target()
    }

    fn kind(&self) -> OperatorKind {
        self.kind
    }

    fn left_type<'db>(&'db self, _database: &'db Self::DB) -> Option<TargetName<'db>> {
        self.left.as_deref().map(CatalogIdentity::target)
    }

    fn right_type<'db>(&'db self, _database: &'db Self::DB) -> Option<TargetName<'db>> {
        self.right.as_deref().map(CatalogIdentity::target)
    }

    fn result_type<'db>(&'db self, _database: &'db Self::DB) -> Option<TargetName<'db>> {
        self.result.as_deref().map(CatalogIdentity::target)
    }

    fn implementation<'db>(&'db self, _database: &'db Self::DB) -> Option<&'db PgFunction> {
        self.implementation.as_deref()
    }
}
