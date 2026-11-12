//! Stored operator signatures and exact implementing functions.

use std::{collections::HashMap, sync::Arc};

use diesel::{PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use sql_traits::{
    structs::TargetName,
    traits::{Metadata, OperatorKind, OperatorLike},
};

use crate::{
    PgDieselDatabase, database::CatalogCache, model_metadata::PgFunction, models::PgOperator,
};

/// A stored catalog identity with a shared schema name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CatalogIdentity {
    /// The schema name as stored in `pg_namespace`.
    schema: Arc<str>,
    /// The object name as stored in its catalog.
    name: String,
}

impl CatalogIdentity {
    /// Returns the qualified identity without identifier folding.
    fn target(&self) -> TargetName<'_> {
        TargetName::new(self.name.as_str(), true).with_schema(self.schema.as_ref(), true)
    }
}

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

/// Shares a stored schema name across catalog identities.
fn namespace(
    oid: u32,
    cache: &CatalogCache,
    namespaces: &mut HashMap<u32, Arc<str>>,
) -> Option<Arc<str>> {
    if let Some(name) = namespaces.get(&oid) {
        return Some(Arc::clone(name));
    }
    let name: Arc<str> = cache.namespace(oid)?.into();
    namespaces.insert(oid, Arc::clone(&name));
    Some(name)
}

/// Resolves a type OID without replacing its stored name with a display spelling.
fn operand_type(
    oid: u32,
    cache: &CatalogCache,
    types: &mut HashMap<u32, Arc<CatalogIdentity>>,
    namespaces: &mut HashMap<u32, Arc<str>>,
) -> Option<Arc<CatalogIdentity>> {
    if let Some(identity) = types.get(&oid) {
        return Some(Arc::clone(identity));
    }
    let model = cache.pg_type(oid)?;
    let identity = Arc::new(CatalogIdentity {
        schema: namespace(model.typnamespace, cache, namespaces)?,
        name: model.typname.clone(),
    });
    types.insert(oid, Arc::clone(&identity));
    Some(identity)
}

impl PgOperatorEntry {
    /// Loads operator signatures with their exact loaded function objects.
    ///
    /// # Errors
    ///
    /// Returns an error if a catalog query fails or an operator kind is unsupported.
    pub(crate) fn load(
        connection: &mut PgConnection,
        cache: &CatalogCache,
        functions: &[Arc<PgFunction>],
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::schema::pg_catalog::pg_operator::pg_operator;

        let functions: HashMap<_, _> = functions
            .iter()
            .map(|function| (function.oid(), function))
            .collect();
        let mut types = HashMap::new();
        let mut namespaces = HashMap::new();
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
                        schema: namespace(model.oprnamespace, cache, &mut namespaces)
                            .ok_or(diesel::result::Error::NotFound)?,
                        name: model.oprname,
                    },
                    kind,
                    left: operand_type(model.oprleft, cache, &mut types, &mut namespaces),
                    right: operand_type(model.oprright, cache, &mut types, &mut namespaces),
                    result: operand_type(model.oprresult, cache, &mut types, &mut namespaces),
                    implementation: functions
                        .get(&model.oprcode)
                        .map(|function| Arc::clone(function)),
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
