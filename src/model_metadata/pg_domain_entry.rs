//! The domain a `pg_type` row of type `d` becomes.
//!
//! A domain's declared type is the base type the domain was declared over,
//! one step: the base may itself be a domain, and it is the base type's own
//! `base_type` that reaches the final non-domain type. The catalog holds the
//! base by OID, and the typmod the declaration wrote is the domain's own
//! `typtypmod`.

use std::collections::HashMap;

use diesel::{
    ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl,
    dsl::sql,
    sql_types::{Nullable, Text},
};
use sqlparser::{ast::Expr, parser::ParserError};

use crate::{
    database::CatalogCache,
    model_metadata::{PgDataType, parse_printed_expression},
    schema::pg_catalog::{
        pg_constraint::pg_constraint, pg_namespace::pg_namespace, pg_type::pg_type,
    },
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
    /// The default or a check expression of the domain, as `pg_get_expr`
    /// prints it, does not parse.
    #[error("the expression `{expression}` of the domain {schema}.{name} does not parse: {source}")]
    UnparsableExpression {
        /// The name of the domain.
        name: String,
        /// The schema of the domain.
        schema: String,
        /// The expression as `PostgreSQL` printed it.
        expression: String,
        /// Why the parser rejected it.
        source: ParserError,
    },
}

/// A domain read from `pg_type`, beside the type it declares over and the
/// expressions it holds.
#[derive(Debug, Clone, PartialEq)]
pub struct PgDomainEntry {
    /// The stored name of the domain, in exact case.
    name: String,
    /// The stored name of the schema the domain lives in, in exact case.
    schema: String,
    /// The type the domain declares over, with the typmod the declaration
    /// wrote and the base it reaches.
    data_type: PgDataType,
    /// The expression the domain defaults to, as `pg_get_expr` prints it.
    default_expression: Option<Expr>,
    /// The expression of each `CHECK` constraint of the domain, as
    /// `pg_get_expr` prints it, in constraint name order.
    check_expressions: Vec<Expr>,
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

    /// Returns the expression the domain defaults to, if any.
    #[must_use]
    pub fn default_expression(&self) -> Option<&Expr> {
        self.default_expression.as_ref()
    }

    /// Returns the expression of each `CHECK` constraint of the domain, in
    /// constraint name order.
    #[must_use]
    pub fn check_expressions(&self) -> &[Expr] {
        &self.check_expressions
    }

    /// Loads every domain of the schemas `schemas` names, with the default
    /// and checks `pg_get_expr` prints under the reading session's search
    /// path.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if a query fails, and
    /// [`DomainDefinitionError`] if a domain's base type cannot be read from
    /// the cache or one of its expressions does not parse.
    pub fn load_all(
        schemas: &[String],
        cache: &CatalogCache,
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, DomainDefinitionError> {
        let rows = pg_type::table
            .inner_join(pg_namespace::table.on(pg_type::typnamespace.eq(pg_namespace::oid)))
            .filter(pg_type::typtype.eq("d"))
            .filter(pg_namespace::nspname.eq_any(schemas))
            .select((
                pg_type::oid,
                pg_type::typname,
                pg_namespace::nspname,
                pg_type::typbasetype,
                pg_type::typtypmod,
                pg_type::typndims,
                sql::<Nullable<Text>>("pg_get_expr(typdefaultbin, 0)"),
            ))
            .load::<(u32, String, String, u32, i32, i32, Option<String>)>(conn)?;
        let mut checks: HashMap<u32, Vec<String>> = HashMap::new();
        for (domain, printed) in pg_constraint::table
            .filter(pg_constraint::contypid.eq_any(rows.iter().map(|row| row.0)))
            .filter(pg_constraint::contype.eq("c"))
            .order_by(pg_constraint::conname)
            .select((
                pg_constraint::contypid,
                sql::<Text>("pg_get_expr(conbin, 0)"),
            ))
            .load::<(u32, String)>(conn)?
        {
            checks.entry(domain).or_default().push(printed);
        }
        rows.into_iter()
            .map(|(oid, name, schema, base, typmod, ndims, default)| {
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
                let parse = |printed: String| {
                    parse_printed_expression(&printed).map_err(|source| {
                        DomainDefinitionError::UnparsableExpression {
                            name: name.clone(),
                            schema: schema.clone(),
                            expression: printed,
                            source,
                        }
                    })
                };
                let default_expression = default.map(parse).transpose()?;
                let check_expressions = checks
                    .remove(&oid)
                    .unwrap_or_default()
                    .into_iter()
                    .map(parse)
                    .collect::<Result<_, _>>()?;
                Ok(Self {
                    name,
                    schema,
                    data_type,
                    default_expression,
                    check_expressions,
                })
            })
            .collect()
    }
}
