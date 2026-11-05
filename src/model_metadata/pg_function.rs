//! Submodule providing [`PgFunction`], the function a `PgDieselDatabase` holds.

use std::borrow::Cow;

use sqlparser::ast::{
    ArgMode, Expr, FunctionDefinitionSetParam, FunctionSetValue, Ident, ObjectName, ObjectNamePart,
};

use crate::models::PgProc;

/// A `pg_proc` row paired with the schema, language and `SET` clauses it
/// records only as OIDs or raw text.
///
/// `FunctionLike` asks for all three without handing over a database.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgFunction {
    /// The `pg_catalog.pg_proc` row describing the function.
    model: PgProc,
    /// The schema the function lives in, from `pg_namespace`.
    schema: String,
    /// The language the function is written in, from `pg_language`.
    language: Option<String>,
    /// The `SET` clauses attached to the function, from `proconfig`.
    configuration: Vec<FunctionDefinitionSetParam>,
    /// The `CREATE FUNCTION` statement `pg_get_functiondef` prints for a body
    /// `PostgreSQL` bound at creation, a `RETURN` expression or a
    /// `BEGIN ATOMIC` block.
    bound_definition: Option<String>,
}

/// Reads one `name=value` `proconfig` entry, each comma-separated part
/// unquoted by the server's quoting so it renders back to the name the
/// catalog stored.
fn configuration_parameter(entry: &str) -> Option<FunctionDefinitionSetParam> {
    let (name, value) = entry.split_once('=')?;
    let values = value
        .split(',')
        .map(|part| Expr::Identifier(proconfig_identifier(part.trim())))
        .collect::<Vec<Expr>>();
    Some(FunctionDefinitionSetParam {
        name: ObjectName(vec![ObjectNamePart::Identifier(Ident::new(name.trim()))]),
        value: FunctionSetValue::Values(values),
    })
}

/// Reads one value entry of a `proconfig` setting as the identifier the
/// server stored.
///
/// The server quotes a value the way it quotes an identifier, doubling the
/// embedded quotes, so `"App"` stores the one schema `App` and `""` the
/// zero-length name a `SET search_path = ''` records.
fn proconfig_identifier(part: &str) -> Ident {
    match (part.chars().next(), part.chars().next_back()) {
        (Some('"'), Some('"')) if part.len() >= 2 => {
            let inner = &part[1..part.len() - 1];
            Ident::with_quote('"', inner.replace("\"\"", "\""))
        }
        _ => Ident::new(part),
    }
}

impl PgFunction {
    /// Pairs a `pg_proc` row with the names its OIDs point at.
    #[must_use]
    pub fn new(model: PgProc, schema: String, language: Option<String>) -> Self {
        let configuration = model
            .proconfig
            .iter()
            .flatten()
            .filter_map(|entry| configuration_parameter(entry))
            .collect();
        Self {
            model,
            schema,
            language,
            configuration,
            bound_definition: None,
        }
    }

    /// Attaches the definition `pg_get_functiondef` prints for a body
    /// `PostgreSQL` bound at creation.
    #[must_use]
    pub fn with_bound_definition(mut self, definition: String) -> Self {
        self.bound_definition = Some(definition);
        self
    }

    /// Returns the definition printed for a body `PostgreSQL` bound at
    /// creation, if it bound one.
    #[must_use]
    pub fn bound_definition(&self) -> Option<&str> {
        self.bound_definition.as_deref()
    }

    /// Returns the `pg_catalog.pg_proc` row describing the function.
    #[must_use]
    pub fn model(&self) -> &PgProc {
        &self.model
    }

    /// Returns the schema the function lives in.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the language the function is written in.
    #[must_use]
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    /// Returns the configuration settings stored on the function.
    #[must_use]
    pub fn configuration(&self) -> &[FunctionDefinitionSetParam] {
        &self.configuration
    }

    /// Returns the OID the catalog records the function under.
    #[must_use]
    pub fn oid(&self) -> u32 {
        self.model.oid
    }

    /// Returns the mode of each argument, in declaration order, `OUT`
    /// included.
    ///
    /// `proargmodes` stores one code per argument and is `NULL` for a
    /// function every argument of which is `IN`, whose arguments
    /// `proargtypes` alone lists.
    pub fn argument_modes(&self) -> impl Iterator<Item = ArgMode> {
        let modes = self.model.proargmodes.as_deref().unwrap_or_default();
        let declared = self.model.proargtypes.len();
        modes
            .iter()
            .map(|mode| argument_mode(mode))
            .chain((modes.len()..declared).map(|_| ArgMode::In))
    }

    /// Returns the schemas the function's `search_path` setting names, in
    /// order, or `None` when the function sets no `search_path` of its own
    /// and its body runs under the caller's path.
    ///
    /// Each schema is the name the catalog stored, unquoted from the server's
    /// quoting, with whether the server stored it quoted, which includes the
    /// one zero-length entry a `SET search_path = ''` records.
    #[must_use]
    pub fn own_search_path(&self) -> Option<Vec<(Cow<'_, str>, bool)>> {
        let parameter = self
            .configuration()
            .iter()
            .rev()
            .find(|parameter| is_search_path_parameter(&parameter.name))?;
        let FunctionSetValue::Values(values) = &parameter.value else {
            return None;
        };

        Some(
            values
                .iter()
                .filter_map(|value| match value {
                    Expr::Identifier(ident) => Some((
                        Cow::Borrowed(ident.value.as_str()),
                        ident.quote_style.is_some(),
                    )),
                    _ => None,
                })
                .collect(),
        )
    }
}

/// The mode one `proargmodes` code names.
fn argument_mode(code: &str) -> ArgMode {
    match code {
        "o" => ArgMode::Out,
        "b" => ArgMode::InOut,
        "v" => ArgMode::Variadic,
        _ => ArgMode::In,
    }
}

/// Whether a configuration parameter name is `search_path`, which
/// `PostgreSQL` matches without regard to case.
fn is_search_path_parameter(name: &ObjectName) -> bool {
    matches!(
        name.0.as_slice(),
        [ObjectNamePart::Identifier(ident)] if ident.value.eq_ignore_ascii_case("search_path")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_pg_proc() -> PgProc {
        PgProc {
            oid: 1,
            proname: "func".to_string(),
            pronamespace: 1,
            proowner: 1,
            prolang: 1,
            procost: 1.0,
            prorows: 1.0,
            provariadic: 0,
            prosupport: 0,
            prokind: "f".to_string(),
            prosecdef: false,
            proleakproof: false,
            proisstrict: true,
            proretset: false,
            provolatile: "i".to_string(),
            proparallel: "s".to_string(),
            pronargs: 0,
            pronargdefaults: 0,
            prorettype: 0,
            proargtypes: vec![],
            proallargtypes: None,
            proargmodes: None,
            proargnames: None,
            proargdefaults: None,
            protrftypes: None,
            prosrc: "src".to_string(),
            probin: None,
            #[cfg(any(
                feature = "postgres-14",
                feature = "postgres-15",
                feature = "postgres-16",
                feature = "postgres-17",
                feature = "postgres-18"
            ))]
            prosqlbody: None,
            proconfig: None,
        }
    }

    fn function(proconfig: &[&str]) -> PgFunction {
        let mut model = dummy_pg_proc();
        model.proconfig = Some(proconfig.iter().map(ToString::to_string).collect());
        PgFunction::new(model, "app".to_string(), Some("sql".to_string()))
    }

    fn entries(function: &PgFunction) -> Option<Vec<(String, bool)>> {
        function.own_search_path().map(|path| {
            path.into_iter()
                .map(|(schema, quoted)| (schema.into_owned(), quoted))
                .collect()
        })
    }

    #[test]
    fn test_own_search_path_reads_the_setting() {
        let function = function(&["search_path=app, pg_temp"]);
        assert_eq!(
            entries(&function),
            Some(vec![
                ("app".to_string(), false),
                ("pg_temp".to_string(), false),
            ])
        );
    }

    #[test]
    fn test_own_search_path_keeps_the_empty_entry() {
        // `SET search_path = ''` stores the one zero-length quoted entry.
        let function = function(&["search_path=\"\""]);
        assert_eq!(entries(&function), Some(vec![(String::new(), true)]));
    }

    #[test]
    fn test_own_search_path_unquotes_stored_names() {
        let function = function(&["search_path=\"My Schema\",app"]);
        assert_eq!(
            entries(&function),
            Some(vec![
                ("My Schema".to_string(), true),
                ("app".to_string(), false)
            ])
        );
    }

    #[test]
    fn test_own_search_path_unescapes_doubled_quotes() {
        let function = function(&["search_path=\"My\"\"Schema\""]);
        assert_eq!(
            entries(&function),
            Some(vec![("My\"Schema".to_string(), true)])
        );
    }

    #[test]
    fn test_own_search_path_without_the_setting() {
        let function = function(&["timezone=UTC"]);
        assert_eq!(function.own_search_path(), None);
    }

    #[test]
    fn test_own_search_path_reads_the_last_setting() {
        let function = function(&["search_path=app", "search_path=admin"]);
        assert_eq!(entries(&function), Some(vec![("admin".to_string(), false)]));
    }

    #[test]
    fn test_argument_modes() {
        let mut model = dummy_pg_proc();
        model.proargtypes = vec![1, 2, 3, 4];
        model.proargmodes = Some(vec![
            "i".to_string(),
            "o".to_string(),
            "b".to_string(),
            "v".to_string(),
        ]);
        let function = PgFunction::new(model, "app".to_string(), Some("sql".to_string()));
        let collected: Vec<ArgMode> = function.argument_modes().collect();
        assert_eq!(
            collected,
            vec![ArgMode::In, ArgMode::Out, ArgMode::InOut, ArgMode::Variadic]
        );

        // `NULL` stores a function every argument of which is `IN`.
        let mut unnamed = dummy_pg_proc();
        unnamed.proargtypes = vec![1, 2];
        let unnamed = PgFunction::new(unnamed, "app".to_string(), Some("sql".to_string()));
        let collected: Vec<ArgMode> = unnamed.argument_modes().collect();
        assert_eq!(collected, vec![ArgMode::In, ArgMode::In]);
    }
}
