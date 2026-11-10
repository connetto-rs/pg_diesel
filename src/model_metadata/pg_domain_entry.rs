//! The domain a `pg_type` row of type `d` becomes.
//!
//! A domain's declared type is the base type the domain was declared over,
//! one step: the base may itself be a domain, and it is the base type's own
//! `base_type` that reaches the final non-domain type. The catalog holds the
//! base by OID, and the typmod the declaration wrote is the domain's own
//! `typtypmod`.

use diesel::{ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl};

use crate::{
    database::CatalogCache,
    model_metadata::PgDataType,
    schema::pg_catalog::{pg_namespace::pg_namespace, pg_type::pg_type},
};

/// What stopped a domain row from becoming an entry.
#[derive(Debug, thiserror::Error)]
pub enum DomainDefinitionError {
    /// The base type the domain declares over is not in the catalog cache.
    #[error("the base type of the domain {schema}.{name} is missing from the catalog cache")]
    BaseTypeMissing {
        /// The name of the domain.
        name: String,
        /// The schema of the domain.
        schema: String,
    },
    /// The base type the domain declares over could not be read.
    #[error("the base type of the domain {schema}.{name} could not be read")]
    BaseTypeUnreadable {
        /// The name of the domain.
        name: String,
        /// The schema of the domain.
        schema: String,
    },
    /// A query failed.
    #[error("Diesel error: {0}")]
    Query(#[from] diesel::result::Error),
}

/// A domain read from `pg_type`, beside the type it declares over.
#[derive(Debug, Clone, PartialEq)]
pub struct PgDomainEntry {
    /// The stored name of the domain, in exact case.
    name: String,
    /// The stored name of the schema the domain lives in, in exact case.
    schema: String,
    /// The type the domain declares over, with the typmod the declaration
    /// wrote and the base it reaches.
    data_type: PgDataType,
}

impl PgDomainEntry {
    /// Returns the name of the domain.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the schema the domain lives in.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the type the domain declares over.
    #[must_use]
    pub fn data_type(&self) -> &PgDataType {
        &self.data_type
    }

    /// Loads every domain of the schemas `schemas` names.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if a query fails, and
    /// [`DomainDefinitionError`] if a domain's base type cannot be read from
    /// the cache.
    pub fn load_all(
        schemas: &[String],
        cache: &CatalogCache,
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, DomainDefinitionError> {
        pg_type::table
            .inner_join(pg_namespace::table.on(pg_type::typnamespace.eq(pg_namespace::oid)))
            .filter(pg_type::typtype.eq("d"))
            .filter(pg_namespace::nspname.eq_any(schemas))
            .select((
                pg_type::typname,
                pg_namespace::nspname,
                pg_type::typbasetype,
                pg_type::typtypmod,
                pg_type::typndims,
            ))
            .load::<(String, String, u32, i32, i32)>(conn)?
            .into_iter()
            .map(|(name, schema, base, typmod, ndims)| {
                let base_type =
                    cache
                        .pg_type(base)
                        .ok_or_else(|| DomainDefinitionError::BaseTypeMissing {
                            name: name.clone(),
                            schema: schema.clone(),
                        })?;
                let data_type =
                    PgDataType::from_type(base_type, typmod, cache, ndims).ok_or_else(|| {
                        DomainDefinitionError::BaseTypeUnreadable {
                            name: name.clone(),
                            schema: schema.clone(),
                        }
                    })?;
                Ok(Self {
                    name,
                    schema,
                    data_type,
                })
            })
            .collect()
    }
}
