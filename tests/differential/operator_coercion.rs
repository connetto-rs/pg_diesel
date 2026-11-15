//! Checks operator operand coercions against stored expression trees.

use std::collections::HashMap;

use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use pg_diesel::{
    PgDieselDatabase,
    schema::pg_catalog::{pg_namespace::pg_namespace, pg_type::pg_type},
    sql_traits::{
        structs::{CoercionStep, OperandEvidence, TargetName},
        traits::{CastLike, DatabaseLike, FunctionLike},
        utils::normalize_postgres_type,
    },
};

use super::node_tree::Tree;

#[derive(Debug)]
struct Type {
    name: String,
    schema: String,
    element: u32,
    array: u32,
    base: u32,
    typmod: i32,
}

#[derive(Debug, PartialEq, Eq)]
struct Signature {
    schema: Option<String>,
    name: String,
    arguments: Vec<u32>,
}

pub(super) struct Catalog {
    types: HashMap<u32, Type>,
    names: HashMap<String, Vec<u32>>,
    functions: HashMap<u32, Signature>,
}

#[derive(Debug)]
enum Expected {
    Source(Tree),
    Node {
        name: &'static str,
        attributes: Vec<(&'static str, String)>,
        children: Vec<(&'static str, bool, Vec<Expected>)>,
    },
    Integer(i32),
    Boolean(bool),
}

impl Catalog {
    /// Loads type identities and overloaded function signatures.
    pub(super) fn load(connection: &mut diesel::PgConnection, live: &PgDieselDatabase) -> Self {
        let rows = pg_type::table
            .inner_join(pg_namespace::table.on(pg_namespace::oid.eq(pg_type::typnamespace)))
            .select((
                pg_type::oid,
                pg_type::typname,
                pg_namespace::nspname,
                pg_type::typelem,
                pg_type::typarray,
                pg_type::typbasetype,
                pg_type::typtypmod,
            ))
            .load::<(u32, String, String, u32, u32, u32, i32)>(connection)
            .unwrap();
        let mut catalog = Self {
            types: rows
                .into_iter()
                .map(|(oid, name, schema, element, array, base, typmod)| {
                    (
                        oid,
                        Type {
                            name,
                            schema,
                            element,
                            array,
                            base,
                            typmod,
                        },
                    )
                })
                .collect(),
            names: HashMap::new(),
            functions: HashMap::new(),
        };
        for (&oid, ty) in &catalog.types {
            for qualified in [false, true] {
                let target = TargetName::new(&ty.name, true);
                let target = if qualified {
                    target.with_schema(&ty.schema, true)
                } else {
                    target
                };
                let name = normalize_postgres_type(&target.to_string()).into_owned();
                let entries = catalog.names.entry(name).or_default();
                if !entries.contains(&oid) {
                    entries.push(oid);
                }
                if ty.element != 0 && catalog.types[&ty.element].array == oid {
                    let element = &catalog.types[&ty.element];
                    let name = TargetName::new(&element.name, true);
                    let name = if qualified {
                        name.with_schema(&element.schema, true)
                    } else {
                        name
                    };
                    let name = normalize_postgres_type(&format!("{name}[]")).into_owned();
                    let entries = catalog.names.entry(name).or_default();
                    if !entries.contains(&oid) {
                        entries.push(oid);
                    }
                }
            }
        }
        catalog.functions = live
            .functions()
            .map(|function| (function.oid(), catalog.signature(live, function)))
            .collect();
        catalog
    }

    /// Checks the normalized conversion and its exposed result type.
    pub(super) fn assert_operand<DB: DatabaseLike>(
        &self,
        db: &DB,
        evidence: &OperandEvidence<'_, DB>,
        source: &Tree,
        actual: &Tree,
    ) {
        let expected = self.convert(
            db,
            evidence.conversion(),
            self.type_oid(evidence.target()),
            Expected::Source(meaningful(source)),
            expression_type(source),
            expression_typmod(source),
        );
        self.assert_expression(&expected, actual);
    }

    /// Resolves a declared type spelling without consulting expression nodes.
    fn spelling_oid(&self, name: &str) -> u32 {
        let name = normalize_postgres_type(name);
        let candidates = self
            .names
            .get(name.as_ref())
            .expect("type spelling must be loaded");
        if let [oid] = candidates.as_slice() {
            return *oid;
        }
        let mut builtin = candidates
            .iter()
            .filter(|oid| self.types[*oid].schema == "pg_catalog");
        let oid = *builtin.next().expect("type spelling must be unambiguous");
        assert!(
            builtin.next().is_none(),
            "type spelling must identify one built-in"
        );
        oid
    }

    /// Resolves the catalog identity named by a conversion step.
    fn type_oid(&self, name: &TargetName<'_>) -> u32 {
        self.spelling_oid(&name.to_string())
    }

    /// Identifies a function by schema, name and input argument types.
    fn signature<DB: DatabaseLike>(&self, db: &DB, function: &DB::Function) -> Signature {
        let target = function.target_name();
        Signature {
            schema: target
                .schema()
                .map(|schema| stored(schema, target.schema_is_quoted())),
            name: stored(target.name(), target.name_is_quoted()),
            arguments: function
                .argument_type_names(db)
                .zip(function.argument_modes())
                .filter(|(_, mode)| *mode != sqlparser::ast::ArgMode::Out)
                .map(|(name, _)| self.spelling_oid(&name))
                .collect(),
        }
    }

    /// Finds the server function matching the cast's exact signature.
    fn function_oid<DB: DatabaseLike>(&self, db: &DB, cast: &DB::Cast) -> (u32, usize) {
        let function = cast.function(db).expect("cast function must be loaded");
        let expected = self.signature(db, function);
        let mut matches = self.functions.iter().filter(|(_, candidate)| {
            candidate.name == expected.name
                && candidate.arguments == expected.arguments
                && expected
                    .schema
                    .as_ref()
                    .is_none_or(|schema| candidate.schema.as_ref() == Some(schema))
        });
        let (&oid, signature) = matches.next().expect("cast function signature must exist");
        assert!(
            matches.next().is_none(),
            "cast function signature must be unambiguous"
        );
        (oid, signature.arguments.len())
    }

    /// Removes domain layers while retaining their effective modifier.
    fn base(&self, mut oid: u32, mut typmod: i32) -> (u32, i32) {
        for _ in 0..self.types.len() {
            let ty = &self.types[&oid];
            if ty.base == 0 {
                return (oid, typmod);
            }
            if typmod < 0 {
                typmod = ty.typmod;
            }
            oid = ty.base;
        }
        panic!("domain bases must be acyclic");
    }

    /// Builds every wrapper from conversion evidence and source catalog facts.
    fn convert<DB: DatabaseLike>(
        &self,
        db: &DB,
        steps: &[CoercionStep<'_, DB>],
        target: u32,
        mut expression: Expected,
        mut source_type: u32,
        mut source_typmod: i32,
    ) -> Expected {
        let mut binary_target = None;
        for step in steps {
            match step {
                CoercionStep::LiteralInput { type_name } => {
                    source_type = self.type_oid(type_name);
                    source_typmod = -1;
                    expression = literal_input(source_type, &expression);
                }
                CoercionStep::DomainToBase { base } => {
                    source_typmod = self.base(source_type, source_typmod).1;
                    source_type = self.type_oid(base);
                    binary_target = Some(source_type);
                }
                CoercionStep::Relabel(cast) => {
                    source_type = self.type_oid(&cast.target_type(db));
                    source_typmod = -1;
                    binary_target = Some(source_type);
                }
                CoercionStep::Function(cast) | CoercionStep::LengthCoercion { cast, .. } => {
                    binary_target = None;
                    let typmod = if let CoercionStep::LengthCoercion { typmod, .. } = step {
                        *typmod
                    } else {
                        -1
                    };
                    expression = self.cast_expression(db, cast, typmod, expression);
                    source_type = self.type_oid(&cast.target_type(db));
                    source_typmod = typmod;
                }
                CoercionStep::InOut(cast) => {
                    binary_target = None;
                    source_type = self.type_oid(&cast.target_type(db));
                    source_typmod = -1;
                    expression =
                        unary("COERCEVIAIO", "coerceformat", source_type, None, expression);
                }
                CoercionStep::Array { element } => {
                    binary_target = None;
                    let (array, _) = self.base(target, -1);
                    expression = self.array_expression(
                        db,
                        element,
                        array,
                        source_type,
                        source_typmod,
                        expression,
                    );
                    source_type = array;
                    source_typmod = -1;
                }
                CoercionStep::RecordToComposite => {
                    assert!(binary_target.is_none());
                    let Expected::Source(source) = &mut expression else {
                        panic!("record conversion must consume a row constructor");
                    };
                    assert_eq!(source.name(), Some("ROWEXPR"));
                    set_atom(source, "row_typeid", target.to_string());
                    set_atom(source, "row_format", "2".into());
                    source_type = target;
                    source_typmod = -1;
                }
                CoercionStep::CompositeToRecord => {}
                CoercionStep::Inherited => {
                    assert!(binary_target.is_none());
                    expression = unary(
                        "CONVERTROWTYPEEXPR",
                        "convertformat",
                        target,
                        None,
                        expression,
                    );
                    source_type = target;
                    source_typmod = -1;
                }
                CoercionStep::DomainCheck { domain } => {
                    binary_target = None;
                    source_type = self.type_oid(domain);
                    source_typmod = -1;
                    expression = unary(
                        "COERCETODOMAIN",
                        "coercionformat",
                        source_type,
                        Some(-1),
                        expression,
                    );
                }
            }
        }
        if let Some(target) = binary_target {
            expression = unary("RELABELTYPE", "relabelformat", target, Some(-1), expression);
        }
        expression
    }

    /// Applies a cast function with its modifier and explicitness arguments.
    fn cast_expression<DB: DatabaseLike>(
        &self,
        db: &DB,
        cast: &DB::Cast,
        typmod: i32,
        expression: Expected,
    ) -> Expected {
        let (function, arity) = self.function_oid(db, cast);
        assert!((1..=3).contains(&arity));
        let target = self.type_oid(&cast.target_type(db));
        let mut arguments = vec![expression];
        if arity >= 2 {
            arguments.push(Expected::Integer(typmod));
        }
        if arity == 3 {
            arguments.push(Expected::Boolean(false));
        }
        node(
            "FUNCEXPR",
            [
                ("funcid", function.to_string()),
                ("funcresulttype", target.to_string()),
                ("funcformat", "2".into()),
            ],
            vec![("args", true, arguments)],
        )
    }

    /// Converts each array element over the server's case-test placeholder.
    fn array_expression<DB: DatabaseLike>(
        &self,
        db: &DB,
        element: &[CoercionStep<'_, DB>],
        target: u32,
        source_type: u32,
        source_typmod: i32,
        expression: Expected,
    ) -> Expected {
        let target_element = self.types[&target].element;
        let (source_array, element_typmod) = self.base(source_type, source_typmod);
        let source_element = self.types[&source_array].element;
        assert_ne!(target_element, 0);
        assert_ne!(source_element, 0);
        let placeholder = node(
            "CASETESTEXPR",
            [
                ("typeId", source_element.to_string()),
                ("typeMod", element_typmod.to_string()),
            ],
            vec![],
        );
        let converted = self.convert(
            db,
            element,
            target_element,
            placeholder,
            source_element,
            element_typmod,
        );
        node(
            "ARRAYCOERCEEXPR",
            [
                ("resulttype", target.to_string()),
                ("resulttypmod", "-1".into()),
                ("coerceformat", "2".into()),
            ],
            vec![
                ("arg", false, vec![expression]),
                ("elemexpr", false, vec![converted]),
            ],
        )
    }

    /// Compares wrapper attributes, child expressions and cast auxiliary constants.
    fn assert_expression(&self, expected: &Expected, actual: &Tree) {
        match expected {
            Expected::Source(source) => assert_eq!(*source, meaningful(actual)),
            Expected::Node {
                name,
                attributes,
                children,
            } => {
                assert_eq!(actual.name(), Some(*name));
                for (field, value) in attributes {
                    assert_eq!(actual.atom(field), Some(value.as_str()), "{name}.{field}");
                }
                for (field, list, expected) in children {
                    let actual = if *list {
                        actual.list(field)
                    } else {
                        actual.field(field)
                    }
                    .unwrap();
                    assert_eq!(expected.len(), actual.len(), "{name}.{field}");
                    for (expected, actual) in expected.iter().zip(actual) {
                        self.assert_expression(expected, actual);
                    }
                }
            }
            Expected::Integer(value) => {
                self.assert_constant(actual, "int4");
                assert_eq!(datum::<4>(actual), value.to_ne_bytes());
            }
            Expected::Boolean(value) => {
                self.assert_constant(actual, "bool");
                assert_eq!(datum::<1>(actual), [u8::from(*value)]);
            }
        }
    }

    /// Checks the type and modifier of a non-null auxiliary constant.
    fn assert_constant(&self, actual: &Tree, type_name: &str) {
        assert_eq!(actual.name(), Some("CONST"));
        assert_eq!(
            expression_type(actual),
            self.type_oid(&TargetName::new(type_name, true).with_schema("pg_catalog", true))
        );
        assert_eq!(actual.atom("consttypmod"), Some("-1"));
        assert_eq!(actual.atom("constisnull"), Some("false"));
    }
}

/// Folds an unquoted catalog identifier.
fn stored(name: &str, quoted: bool) -> String {
    if quoted {
        name.to_owned()
    } else {
        name.to_ascii_lowercase()
    }
}

/// Builds an expected expression node.
fn node<const N: usize>(
    name: &'static str,
    attributes: [(&'static str, String); N],
    children: Vec<(&'static str, bool, Vec<Expected>)>,
) -> Expected {
    Expected::Node {
        name,
        attributes: Vec::from(attributes),
        children,
    }
}

/// Applies literal input while preserving the literal's nullness.
fn literal_input(target: u32, expression: &Expected) -> Expected {
    let Expected::Source(source) = expression else {
        panic!("literal input must consume a source constant");
    };
    assert_eq!(source.name(), Some("CONST"));
    node(
        "CONST",
        [
            ("consttype", target.to_string()),
            ("consttypmod", "-1".into()),
            ("constisnull", source.atom("constisnull").unwrap().into()),
        ],
        vec![],
    )
}

/// Builds an implicit unary conversion with an optional result modifier.
fn unary(
    name: &'static str,
    format: &'static str,
    target: u32,
    typmod: Option<i32>,
    expression: Expected,
) -> Expected {
    let mut attributes = Vec::with_capacity(2 + usize::from(typmod.is_some()));
    attributes.push(("resulttype", target.to_string()));
    attributes.push((format, "2".into()));
    if let Some(typmod) = typmod {
        attributes.push(("resulttypmod", typmod.to_string()));
    }
    Expected::Node {
        name,
        attributes,
        children: vec![("arg", false, vec![expression])],
    }
}

/// Removes source locations and row labels without changing row children.
fn meaningful(tree: &Tree) -> Tree {
    let mut tree = tree.clone();
    tree.strip(&["location", "colnames"]);
    tree
}

/// Replaces one required scalar field of a source expression.
fn set_atom(tree: &mut Tree, field: &str, value: String) {
    let Tree::Node { fields, .. } = tree else {
        panic!("expression must be a node")
    };
    let (_, values) = fields.iter_mut().find(|(name, _)| name == field).unwrap();
    *values = vec![Tree::Atom(value)];
}

/// Reads the exposed type of a source expression.
fn expression_type(tree: &Tree) -> u32 {
    let field = match tree.name().unwrap() {
        "VAR" => "vartype",
        "CONST" => "consttype",
        "PARAM" => "paramtype",
        "FUNCEXPR" => "funcresulttype",
        "OPEXPR" | "DISTINCTEXPR" | "NULLIFEXPR" => "opresulttype",
        "ROWEXPR" => "row_typeid",
        "ARRAYEXPR" => "array_typeid",
        "CASETESTEXPR" => "typeId",
        "RELABELTYPE" | "COERCEVIAIO" | "ARRAYCOERCEEXPR" | "CONVERTROWTYPEEXPR"
        | "COERCETODOMAIN" => "resulttype",
        name => panic!("unsupported source expression {name}"),
    };
    tree.atom(field).unwrap().parse().unwrap()
}

/// Reads an explicit source modifier, defaulting to an unconstrained type.
fn expression_typmod(tree: &Tree) -> i32 {
    [
        "vartypmod",
        "consttypmod",
        "paramtypmod",
        "typeMod",
        "resulttypmod",
    ]
    .into_iter()
    .find_map(|field| tree.atom(field))
    .map_or(-1, |value| value.parse().unwrap())
}

/// Reads native datum bytes from a stored by-value constant.
fn datum<const N: usize>(tree: &Tree) -> [u8; N] {
    let values = tree.field("constvalue").unwrap();
    let start = values
        .iter()
        .position(|value| matches!(value, Tree::Atom(atom) if atom == "["))
        .unwrap()
        + 1;
    let mut bytes = [0; N];
    for (byte, value) in bytes.iter_mut().zip(&values[start..start + N]) {
        let Tree::Atom(value) = value else {
            panic!("datum byte must be an atom")
        };
        *byte = value.parse::<i16>().unwrap().to_le_bytes()[0];
    }
    bytes
}
