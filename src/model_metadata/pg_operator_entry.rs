//! Stored operator signatures, relations, and planner metadata.

use std::sync::Arc;

use diesel::{PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use sql_traits::{
    structs::TargetName,
    traits::{DatabaseLike, Metadata, OperatorKind, OperatorLike},
};

use crate::{PgDieselDatabase, model_metadata::PgFunction, models::PgOperator};

use super::catalog_identity::{CatalogIdentity, CatalogObjects};

/// An operator signature resolved by catalog OIDs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgOperatorEntry {
    /// The operator's catalog OID.
    oid: u32,
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
    /// The commutator's position in the catalog operator slice.
    commutator: Option<usize>,
    /// The negator's position in the catalog operator slice.
    negator: Option<usize>,
    /// Whether the operator supports hash joins.
    can_hash: bool,
    /// Whether the operator supports merge joins.
    can_merge: bool,
    /// The exact loaded restriction selectivity function.
    restriction_estimator: Option<Arc<PgFunction>>,
    /// The exact loaded join selectivity function.
    join_estimator: Option<Arc<PgFunction>>,
}

impl PgOperatorEntry {
    /// Returns the operator's catalog OID.
    #[must_use]
    pub const fn oid(&self) -> u32 {
        self.oid
    }

    /// Loads operator signatures and relations in catalog slice order.
    ///
    /// # Errors
    ///
    /// Returns an error if a catalog query fails or an operator kind is unsupported.
    pub(crate) fn load(
        connection: &mut PgConnection,
        objects: &mut CatalogObjects<'_>,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::schema::pg_catalog::pg_operator::pg_operator;

        let mut models = pg_operator::table
            .order(pg_operator::oid)
            .select(PgOperator::as_select())
            .load::<PgOperator>(connection)?;
        // The catalog operator slice retains this OID order for positional links.
        (0..models.len())
            .map(|index| {
                let index_of = |oid| {
                    if oid == 0 {
                        None
                    } else {
                        models.binary_search_by_key(&oid, |model| model.oid).ok()
                    }
                };
                let commutator = index_of(models[index].oprcom);
                let negator = index_of(models[index].oprnegate);
                let model = &mut models[index];
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
                    oid: model.oid,
                    target: CatalogIdentity {
                        schema: objects
                            .namespace(model.oprnamespace)
                            .ok_or(diesel::result::Error::NotFound)?,
                        name: std::mem::take(&mut model.oprname),
                    },
                    kind,
                    left: objects.type_identity(model.oprleft),
                    right: objects.type_identity(model.oprright),
                    result: objects.type_identity(model.oprresult),
                    implementation: objects.function(model.oprcode),
                    commutator,
                    negator,
                    can_hash: model.oprcanhash,
                    can_merge: model.oprcanmerge,
                    restriction_estimator: objects.function(model.oprrest),
                    join_estimator: objects.function(model.oprjoin),
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

    fn commutator<'db>(&'db self, database: &'db Self::DB) -> Option<&'db Self> {
        database.operators()?.nth(self.commutator?)
    }

    fn negator<'db>(&'db self, database: &'db Self::DB) -> Option<&'db Self> {
        database.operators()?.nth(self.negator?)
    }

    fn can_hash(&self) -> bool {
        self.can_hash
    }

    fn can_merge(&self) -> bool {
        self.can_merge
    }

    fn restriction_estimator<'db>(&'db self, _database: &'db Self::DB) -> Option<&'db PgFunction> {
        self.restriction_estimator.as_deref()
    }

    fn join_estimator<'db>(&'db self, _database: &'db Self::DB) -> Option<&'db PgFunction> {
        self.join_estimator.as_deref()
    }
}
