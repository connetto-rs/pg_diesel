//! The relation kind a `pg_views` or `pg_matviews` row becomes.

use sqlparser::{ast::Query, dialect::PostgreSqlDialect, parser::Parser};

use crate::models::{PgMatview, PgView};

/// A view with the [`Query`] its `pg_get_viewdef` text parses into.
///
/// One struct stands for both kinds; the collection a view is added to, not its
/// type, keeps plain and materialized apart.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgViewDefinition {
    /// The schema the view lives in.
    schema: String,
    /// The name of the view.
    name: String,
    /// The role owning the view.
    owner: Option<String>,
    /// Whether the view holds a stored snapshot of its definition's output.
    materialized: bool,
    /// Whether the view is a security barrier, from its `pg_class.reloptions`.
    security_barrier: bool,
    /// Whether the view runs with the rights of its reader, from its
    /// `pg_class.reloptions`.
    security_invoker: bool,
    /// The parsed defining query, boxed to keep the struct small.
    definition: Box<Query>,
}

/// What stopped a catalog row from becoming a [`PgViewDefinition`].
#[derive(Debug, thiserror::Error)]
pub enum ViewDefinitionError {
    /// The row left one of the columns the view is identified by empty.
    #[error("the {column} column of a {catalog} row is empty")]
    MissingColumn {
        /// The catalog view the row came from.
        catalog: &'static str,
        /// The column that carried no value.
        column: &'static str,
    },
    /// `sqlparser` could not read the definition the catalog reconstructed.
    #[error("the definition of the view {schema}.{name} does not parse: {source}")]
    Unparsable {
        /// The schema of the view whose definition failed to parse.
        schema: String,
        /// The name of the view whose definition failed to parse.
        name: String,
        /// The failure `sqlparser` reported.
        #[source]
        source: sqlparser::parser::ParserError,
    },
}

/// Reads the single query a view definition consists of.
fn parse_definition(
    schema: &str,
    name: &str,
    definition: &str,
) -> Result<Box<Query>, ViewDefinitionError> {
    Parser::new(&PostgreSqlDialect {})
        .try_with_sql(definition)
        .and_then(|mut parser| parser.parse_query())
        .map_err(|source| ViewDefinitionError::Unparsable {
            schema: schema.to_owned(),
            name: name.to_owned(),
            source,
        })
}

/// Returns `value`, or reports the column of `catalog` that carried nothing.
fn required(
    value: Option<String>,
    catalog: &'static str,
    column: &'static str,
) -> Result<String, ViewDefinitionError> {
    value.ok_or(ViewDefinitionError::MissingColumn { catalog, column })
}

/// Returns whether `options` sets the relation option `name`, a bare name or
/// a `true` or `on` value counting as set.
fn relation_option_is_set(options: &[String], name: &str) -> bool {
    options.iter().any(|entry| match entry.split_once('=') {
        Some((option, value)) => {
            option == name
                && (value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("on"))
        }
        None => entry == name,
    })
}

impl PgViewDefinition {
    /// Reads a plain view from a `pg_catalog.pg_views` row, whose
    /// `pg_class.reloptions` are `options`.
    ///
    /// # Errors
    ///
    /// * [`ViewDefinitionError::MissingColumn`] when the row identifies no
    ///   schema, name or definition.
    /// * [`ViewDefinitionError::Unparsable`] when `sqlparser` refuses the
    ///   definition the catalog reconstructed.
    pub fn plain(view: PgView, options: &[String]) -> Result<Self, ViewDefinitionError> {
        let schema = required(view.schemaname, "pg_views", "schemaname")?;
        let name = required(view.viewname, "pg_views", "viewname")?;
        let definition = required(view.definition, "pg_views", "definition")?;
        let definition = parse_definition(&schema, &name, &definition)?;
        Ok(Self {
            schema,
            name,
            owner: view.viewowner,
            materialized: false,
            security_barrier: relation_option_is_set(options, "security_barrier"),
            security_invoker: relation_option_is_set(options, "security_invoker"),
            definition,
        })
    }

    /// Reads a materialized view from a `pg_catalog.pg_matviews` row, whose
    /// `pg_class.reloptions` are `options`.
    ///
    /// # Errors
    ///
    /// * [`ViewDefinitionError::MissingColumn`] when the row identifies no
    ///   schema, name or definition.
    /// * [`ViewDefinitionError::Unparsable`] when `sqlparser` refuses the
    ///   definition the catalog reconstructed.
    pub fn materialized(view: PgMatview, options: &[String]) -> Result<Self, ViewDefinitionError> {
        let schema = required(view.schemaname, "pg_matviews", "schemaname")?;
        let name = required(view.matviewname, "pg_matviews", "matviewname")?;
        let definition = required(view.definition, "pg_matviews", "definition")?;
        let definition = parse_definition(&schema, &name, &definition)?;
        Ok(Self {
            schema,
            name,
            owner: view.matviewowner,
            materialized: true,
            security_barrier: relation_option_is_set(options, "security_barrier"),
            security_invoker: relation_option_is_set(options, "security_invoker"),
            definition,
        })
    }

    /// Returns the schema the view lives in.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the name of the view.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the role owning the view.
    #[must_use]
    pub fn owner(&self) -> Option<&str> {
        self.owner.as_deref()
    }

    /// Returns whether the view holds a stored snapshot.
    #[must_use]
    pub fn is_materialized(&self) -> bool {
        self.materialized
    }

    /// Returns whether the view is a security barrier.
    #[must_use]
    pub fn is_security_barrier(&self) -> bool {
        self.security_barrier
    }

    /// Returns whether the view runs with the rights of its reader.
    #[must_use]
    pub fn is_security_invoker(&self) -> bool {
        self.security_invoker
    }

    /// Returns the query defining the view.
    #[must_use]
    pub fn definition(&self) -> &Query {
        &self.definition
    }
}
