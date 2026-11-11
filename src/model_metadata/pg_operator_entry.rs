//! Catalog operators with qualified signatures and exact implementations.

use std::{collections::HashMap, sync::Arc};

use diesel::{PgConnection, RunQueryDsl, SelectableHelper, connection::DefaultLoadingMode};
use sql_traits::{structs::TargetName, traits::OperatorKind};

use crate::{database::CatalogCache, model_metadata::PgFunction, models::PgOperator};

/// A qualified catalog type shared by every operator that names its OID.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct OperatorType {
    /// The exact catalog type name.
    name: String,
    /// The exact catalog schema name.
    schema: Arc<str>,
}

impl OperatorType {
    /// Returns the stored type name without identifier folding.
    fn target_name(&self) -> TargetName<'_> {
        TargetName::new(&self.name, true).with_schema(&self.schema, true)
    }
}

/// An operator with shared type names and its exact implementing function.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgOperatorEntry {
    /// The catalog OID of the operator.
    oid: u32,
    /// The exact catalog operator name.
    name: String,
    /// The exact catalog schema name.
    schema: Arc<str>,
    /// The declared operand positions.
    kind: OperatorKind,
    /// The qualified left operand type, if present and known.
    left_type: Option<Arc<OperatorType>>,
    /// The qualified right operand type, if known.
    right_type: Option<Arc<OperatorType>>,
    /// The qualified result type, if known.
    result_type: Option<Arc<OperatorType>>,
    /// The function whose OID the operator records in `oprcode`.
    implementation: Option<Arc<PgFunction>>,
}

/// Resolves each namespace once while sharing its exact catalog spelling.
fn namespace(
    oid: u32,
    cache: &CatalogCache,
    namespaces: &mut HashMap<u32, Option<Arc<str>>>,
) -> Option<Arc<str>> {
    namespaces
        .entry(oid)
        .or_insert_with(|| cache.namespace(oid).map(Arc::from))
        .as_ref()
        .map(Arc::clone)
}

/// Resolves each type once while retaining absent or unknown types.
fn operator_type(
    oid: u32,
    cache: &CatalogCache,
    types: &mut HashMap<u32, Option<Arc<OperatorType>>>,
    namespaces: &mut HashMap<u32, Option<Arc<str>>>,
) -> Option<Arc<OperatorType>> {
    if oid == 0 {
        return None;
    }
    types
        .entry(oid)
        .or_insert_with(|| {
            let pg_type = cache.pg_type(oid)?;
            let schema = namespace(pg_type.typnamespace, cache, namespaces)?;
            Some(Arc::new(OperatorType {
                name: pg_type.typname.clone(),
                schema,
            }))
        })
        .as_ref()
        .map(Arc::clone)
}

impl PgOperatorEntry {
    /// Returns the catalog OID of the operator.
    #[must_use]
    pub fn oid(&self) -> u32 {
        self.oid
    }

    /// Returns the exact catalog operator name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the exact catalog schema name.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the qualified operator name without identifier folding.
    #[must_use]
    pub fn target_name(&self) -> TargetName<'_> {
        TargetName::new(&self.name, true).with_schema(&self.schema, true)
    }

    /// Returns the declared operand positions.
    #[must_use]
    pub fn kind(&self) -> OperatorKind {
        self.kind
    }

    /// Returns the qualified left operand type, if present and known.
    #[must_use]
    pub(crate) fn declared_left_type(&self) -> Option<TargetName<'_>> {
        self.left_type.as_deref().map(OperatorType::target_name)
    }

    /// Returns the qualified right operand type, if known.
    #[must_use]
    pub(crate) fn declared_right_type(&self) -> Option<TargetName<'_>> {
        self.right_type.as_deref().map(OperatorType::target_name)
    }

    /// Returns the qualified result type, if known.
    #[must_use]
    pub(crate) fn declared_result_type(&self) -> Option<TargetName<'_>> {
        self.result_type.as_deref().map(OperatorType::target_name)
    }

    /// Returns the exact implementing function, if known.
    #[must_use]
    pub(crate) fn implementing_function(&self) -> Option<&PgFunction> {
        self.implementation.as_deref()
    }

    /// Loads every operator independently of selected tables or schemas.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails, an operator namespace is missing, or an operand position is unsupported.
    pub fn load_all(
        cache: &CatalogCache,
        functions: &[Arc<PgFunction>],
        connection: &mut PgConnection,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use diesel::QueryDsl;

        use crate::schema::pg_catalog::pg_operator::pg_operator;

        let rows = pg_operator::table
            .select(PgOperator::as_select())
            .load_iter::<PgOperator, DefaultLoadingMode>(connection)?;
        let functions: HashMap<_, _> = functions
            .iter()
            .map(|function| (function.oid(), function))
            .collect();
        let mut types = HashMap::new();
        let mut namespaces = HashMap::new();

        rows.map(|row| {
            let row = row?;
            let kind = match row.oprkind.as_str() {
                "l" => OperatorKind::Prefix,
                "b" => OperatorKind::Binary,
                kind => {
                    return Err(diesel::result::Error::DeserializationError(
                        format!("unsupported operator kind {kind:?} for OID {}", row.oid).into(),
                    ));
                }
            };
            let schema = namespace(row.oprnamespace, cache, &mut namespaces).ok_or_else(|| {
                diesel::result::Error::DeserializationError(
                    format!("missing namespace for operator OID {}", row.oid).into(),
                )
            })?;
            let left_type = operator_type(row.oprleft, cache, &mut types, &mut namespaces);
            let right_type = operator_type(row.oprright, cache, &mut types, &mut namespaces);
            let result_type = operator_type(row.oprresult, cache, &mut types, &mut namespaces);
            let implementation = if row.oprcode == 0 {
                None
            } else {
                functions
                    .get(&row.oprcode)
                    .map(|function| Arc::clone(function))
            };
            Ok(Self {
                oid: row.oid,
                name: row.oprname,
                schema,
                kind,
                left_type,
                right_type,
                result_type,
                implementation,
            })
        })
        .collect()
    }
}
