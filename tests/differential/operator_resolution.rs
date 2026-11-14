//! Operator selection and coercion evidence from the server's rewrite trees.

use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use pg_diesel::{
    PgDieselDatabase,
    schema::pg_catalog::{pg_class::pg_class, pg_rewrite::pg_rewrite},
    sql_traits::{
        prelude::IdentifierCase,
        structs::{OperandType, OperatorResolution, ParseOptions, SelectedOperator, TargetName},
        traits::{DatabaseLike, OperatorLike},
    },
    sqlparser::dialect::PostgreSqlDialect,
};

use super::{
    PgConnection, PgDieselDatabaseBuilder, SimpleConnection, establish_connection,
    node_tree::{self, Tree},
    operator_coercion::Catalog,
    operator_fixtures::{SERVER_DDL, SHARED_DDL, catalog},
    reference_docker,
};

struct Operand {
    expression: &'static str,
    data_type: OperandType<'static>,
}

struct Case {
    name: &'static str,
    operator: TargetName<'static>,
    left: Option<Operand>,
    right: Operand,
    bindings: Option<[TargetName<'static>; 2]>,
}

/// Names a stored catalog type.
fn target(schema: &'static str, name: &'static str) -> TargetName<'static> {
    TargetName::new(name, true).with_schema(schema, true)
}

/// Describes an operand with a known stored type.
fn typed(expression: &'static str, schema: &'static str, name: &'static str) -> Operand {
    Operand {
        expression,
        data_type: OperandType::Known {
            type_name: target(schema, name),
            typmod: None,
        },
    }
}

/// Describes a built-in operand without a type modifier.
fn pg(expression: &'static str, name: &'static str) -> Operand {
    typed(expression, "pg_catalog", name)
}

/// Describes an operand of a fixture type.
fn public(expression: &'static str, name: &'static str) -> Operand {
    typed(expression, "public", name)
}

/// Describes a character operand with its stored type modifier.
fn varchar(expression: &'static str, typmod: i32) -> Operand {
    Operand {
        expression,
        data_type: OperandType::Known {
            type_name: target("pg_catalog", "varchar"),
            typmod: Some(typmod),
        },
    }
}

/// Describes an untyped literal.
fn literal(expression: &'static str) -> Operand {
    Operand {
        expression,
        data_type: OperandType::UntypedLiteral,
    }
}

/// Describes a binary operator invocation.
fn binary(name: &'static str, operator: &'static str, left: Operand, right: Operand) -> Case {
    Case {
        name,
        operator: TargetName::new(operator, false),
        left: Some(left),
        right,
        bindings: None,
    }
}

/// Reads a fixture view's stored query with the typed catalog schema.
fn stored_query(connection: &mut PgConnection, name: &str) -> Tree {
    let action: String = pg_rewrite::table
        .inner_join(pg_class::table.on(pg_class::oid.eq(pg_rewrite::ev_class)))
        .filter(pg_class::relname.eq(name))
        .filter(pg_rewrite::rulename.eq("_RETURN"))
        .select(pg_rewrite::ev_action)
        .first(connection)
        .unwrap();
    node_tree::parse(&action).unwrap()
}

/// Returns one result expression of a stored view query.
fn expression(query: &Tree, index: usize) -> &Tree {
    let Tree::List(queries) = query else {
        panic!("the rewrite action must contain queries");
    };
    &queries[0].list("targetList").unwrap()[index]
        .field("expr")
        .unwrap()[0]
}

/// Loads the live model under the connection's current search path.
fn load(connection: &mut PgConnection) -> PgDieselDatabase {
    PgDieselDatabaseBuilder::default()
        .connection(connection)
        .catalog("operator_resolution")
        .schema("public")
        .try_into()
        .unwrap()
}

/// Spells a qualified operator using PostgreSQL's operator syntax.
fn operator_sql(operator: &TargetName<'_>) -> String {
    operator.schema().map_or_else(
        || operator.name().to_owned(),
        |schema| format!("OPERATOR(\"{schema}\".{})", operator.name()),
    )
}

/// Builds the selected expression over the same source relation order.
fn selection_sql(case: &Case) -> String {
    let left = case.left.as_ref().map_or("", |operand| operand.expression);
    format!(
        "CREATE VIEW public.selected AS SELECT {left} {} {} AS result \
         FROM public.samples CROSS JOIN public.child",
        operator_sql(&case.operator),
        case.right.expression
    )
}

/// Preserves unknown literals inside null tests when recording source nodes.
fn source_query(connection: &mut PgConnection, case: &Case) -> Tree {
    let right = case.right.expression;
    let projections = case.left.as_ref().map_or_else(
        || format!("({right}) IS NULL AS r"),
        |left| format!("({}) IS NULL AS l, ({right}) IS NULL AS r", left.expression),
    );
    connection
        .batch_execute(&format!(
            "CREATE VIEW public.sources AS SELECT {projections} \
             FROM public.samples CROSS JOIN public.child"
        ))
        .unwrap();
    stored_query(connection, "sources")
}

/// Compares stored identifiers independently of their display quoting.
fn assert_target(actual: &TargetName<'_>, expected: &TargetName<'_>) {
    assert_eq!(actual.schema(), expected.schema());
    assert_eq!(actual.name(), expected.name());
}

/// Compares signatures independently of backend-specific operator identities.
fn assert_signature<A: DatabaseLike, B: DatabaseLike>(
    a: &A,
    left: &A::Operator,
    b: &B,
    right: &B::Operator,
) {
    assert_target(&left.target_name(), &right.target_name());
    for (left, right) in [
        (left.left_type(a), right.left_type(b)),
        (left.right_type(a), right.right_type(b)),
        (left.result_type(a), right.result_type(b)),
    ] {
        assert_eq!(left.is_some(), right.is_some());
        if let (Some(left), Some(right)) = (left, right) {
            assert_target(&left, &right);
        }
    }
}

/// Checks operand targets separately from the types exposed by their nodes.
fn assert_operands<DB: DatabaseLike>(
    database: &DB,
    selected: &SelectedOperator<'_, DB>,
    case: &Case,
    sources: &Tree,
    operator: &Tree,
    catalog: &Catalog,
) {
    let evidence = selected.left().into_iter().chain([selected.right()]);
    let inputs = case.left.iter().chain([&case.right]);
    let declarations = selected
        .operator()
        .left_type(database)
        .into_iter()
        .chain([selected.operator().right_type(database).unwrap()]);
    let arguments = operator.list("args").unwrap();
    assert_eq!(arguments.len(), if case.left.is_some() { 2 } else { 1 });
    for (index, ((evidence, input), declaration)) in
        evidence.zip(inputs).zip(declarations).enumerate()
    {
        let polymorphic = matches!(
            declaration.name(),
            "anyelement"
                | "anynonarray"
                | "anyarray"
                | "anyenum"
                | "anycompatible"
                | "anycompatiblearray"
        );
        let expected = if polymorphic {
            case.bindings.as_ref().expect("polymorphic fixture binding")[index].clone()
        } else if declaration.name() == "any" {
            match &input.data_type {
                OperandType::Known { type_name, .. } => type_name.clone(),
                OperandType::UntypedLiteral => target("pg_catalog", "unknown"),
                OperandType::Unknown => panic!("a selected operand has a known type"),
            }
        } else {
            declaration
        };
        assert_target(evidence.target(), &expected);
        assert_eq!(evidence.is_polymorphic_binding(), polymorphic);
        let source = &expression(sources, index).field("arg").unwrap()[0];
        catalog.assert_operand(database, evidence, source, &arguments[index]);
    }
}

/// Compares a successful invocation with the server and both metadata backends.
fn compare_case<DB: DatabaseLike>(
    connection: &mut PgConnection,
    live: &PgDieselDatabase,
    parsed: &DB,
    catalog: &Catalog,
    case: &Case,
) {
    eprintln!("operator case {}", case.name);
    connection.batch_execute(&selection_sql(case)).unwrap();
    let query = stored_query(connection, "selected");
    let operator = expression(&query, 0);
    assert_eq!(operator.name(), Some("OPEXPR"));
    let sources = source_query(connection, case);
    let left = || case.left.as_ref().map(|operand| operand.data_type.clone());
    let OperatorResolution::Selected(selected) = live.resolve_operator(
        case.operator.clone(),
        left(),
        case.right.data_type.clone(),
        IdentifierCase::AsWritten,
    ) else {
        panic!("live operator must resolve for {}", case.name);
    };
    assert_eq!(
        selected.operator().oid(),
        operator.atom("opno").unwrap().parse::<u32>().unwrap(),
        "{}",
        case.name
    );
    let OperatorResolution::Selected(source) = parsed.resolve_operator(
        case.operator.clone(),
        left(),
        case.right.data_type.clone(),
        IdentifierCase::AsWritten,
    ) else {
        panic!("parsed operator must resolve for {}", case.name);
    };
    assert_signature(live, selected.operator(), parsed, source.operator());
    assert_operands(live, &selected, case, &sources, operator, catalog);
    assert_operands(parsed, &source, case, &sources, operator, catalog);
    connection
        .batch_execute("DROP VIEW public.selected; DROP VIEW public.sources;")
        .unwrap();
}

/// Exercises concrete signatures, literals and domain conversions.
fn concrete_cases() -> Vec<Case> {
    vec![
        binary("cross_integer", ">", pg("i", "int4"), pg("b", "int8")),
        binary("numeric_function", ">", pg("i", "int4"), pg("n", "numeric")),
        binary("literal_input", "=", pg("t", "text"), literal("'x'")),
        binary("binary_relabel", "=", varchar("v3", 7), pg("t", "text")),
        binary("domain_literal", "=", public("dt", "dtext"), literal("'x'")),
        binary("domain_base", "=", public("di", "dint"), pg("i", "int4")),
        binary("domain_relabel", "=", public("dv", "d5"), pg("t", "text")),
        binary(
            "domain_function",
            "=",
            public("di", "dint"),
            pg("n", "numeric"),
        ),
        binary("two_literals", "=", literal("'x'"), literal("'y'")),
        binary("domain_check", "<%>", pg("s", "int2"), pg("i", "int4")),
        binary("domain_length", "<~>", pg("t", "text"), varchar("v5", 9)),
        binary("literal_length", "<~>", varchar("v3", 7), literal("'hi'")),
        binary(
            "record_rows",
            "<#>",
            pg("ROW(i)", "record"),
            public("p", "pair"),
        ),
        binary(
            "composite_row_constructor",
            "<#>",
            public("p", "pair"),
            public("ROW(i)::public.pair", "pair"),
        ),
        binary(
            "inherited_row",
            "<^>",
            public("child", "child"),
            public("pa", "_parent"),
        ),
        binary("any_bytea", "~~~", pg("i", "int4"), pg("bytes", "bytea")),
        binary("any_literal", "~~~", pg("i", "int4"), literal("'x'")),
        Case {
            name: "in_out",
            operator: TargetName::new("<!>", false),
            left: None,
            right: public("c", "celsius"),
            bindings: None,
        },
        Case {
            name: "domain_in_out",
            operator: TargetName::new("<!>", false),
            left: None,
            right: public("dc", "dcelsius"),
            bindings: None,
        },
    ]
}

/// Exercises classic and compatible polymorphic bindings.
fn polymorphic_cases() -> Vec<Case> {
    vec![
        Case {
            bindings: Some([target("pg_catalog", "_int4"), target("pg_catalog", "_int4")]),
            ..binary(
                "anyarray_literal",
                "<@",
                pg("samples.a", "_int4"),
                literal("'{1}'"),
            )
        },
        Case {
            bindings: Some([target("pg_catalog", "_int8"), target("pg_catalog", "int8")]),
            ..binary(
                "array_function",
                "||",
                pg("samples.a", "_int4"),
                pg("b", "int8"),
            )
        },
        Case {
            bindings: Some([target("pg_catalog", "_int8"), target("pg_catalog", "int8")]),
            ..binary(
                "domain_array",
                "||",
                public("da", "dintarray"),
                pg("b", "int8"),
            )
        },
        Case {
            bindings: Some([target("pg_catalog", "int4"), target("pg_catalog", "int4")]),
            ..binary("anyelement", "===", pg("i", "int4"), pg("i", "int4"))
        },
        Case {
            bindings: Some([target("public", "mood"), target("public", "mood")]),
            ..binary(
                "enum_anyelement",
                "===",
                public("e", "mood"),
                public("e", "mood"),
            )
        },
        Case {
            name: "anyenum",
            operator: TargetName::new("#?", false),
            left: None,
            right: public("e", "mood"),
            bindings: Some([target("public", "mood"), target("public", "mood")]),
        },
        Case {
            bindings: Some([target("public", "dint"), target("public", "dint")]),
            ..binary(
                "domain_anyelement",
                "===",
                public("di", "dint"),
                public("di", "dint"),
            )
        },
        Case {
            bindings: Some([target("pg_catalog", "int4"), target("pg_catalog", "_int4")]),
            ..binary("anyelement_array", "&&&", pg("i", "int4"), literal("'{1}'"))
        },
    ]
}

#[tokio::test]
async fn selected_operators_and_coercions_match_the_server_and_parser() {
    let docker = reference_docker("operator_resolution").await.unwrap();
    let mut connection: PgConnection =
        establish_connection(docker.port, "operator_resolution").unwrap();
    connection.batch_execute(SHARED_DDL).unwrap();
    connection.batch_execute(SERVER_DDL).unwrap();
    let live = load(&mut connection);
    let parsed = ParseOptions::default()
        .with_postgres_catalog(catalog())
        .parse::<PostgreSqlDialect>(SHARED_DDL)
        .unwrap();
    let catalog = Catalog::load(&mut connection, &live);
    for case in concrete_cases().into_iter().chain(polymorphic_cases()) {
        compare_case(&mut connection, &live, &parsed, &catalog, &case);
    }
}

#[derive(Clone, Copy)]
enum Refusal {
    NotFound,
    Ambiguous,
    Unbound,
}

/// Names a signature without backend-specific quoting or identity fields.
fn signature<DB: DatabaseLike>(database: &DB, operator: &DB::Operator) -> String {
    let name = |target: TargetName<'_>| format!("{}.{}", target.schema().unwrap(), target.name());
    format!(
        "{}({},{})",
        name(operator.target_name()),
        operator.left_type(database).map_or_else(String::new, name),
        name(operator.right_type(database).unwrap())
    )
}

/// Checks refusal kinds and returns the remaining candidate signatures.
fn refused<DB: DatabaseLike>(database: &DB, case: &Case, expected: Refusal) -> Vec<String> {
    let resolution = database.resolve_operator(
        case.operator.clone(),
        case.left.as_ref().map(|operand| operand.data_type.clone()),
        case.right.data_type.clone(),
        IdentifierCase::AsWritten,
    );
    match (expected, resolution) {
        (Refusal::NotFound, OperatorResolution::NotFound) => Vec::new(),
        (Refusal::Ambiguous, OperatorResolution::Ambiguous(operators)) => {
            assert!(operators.len() > 1);
            let mut candidates: Vec<_> = operators
                .into_iter()
                .map(|operator| signature(database, operator))
                .collect();
            candidates.sort_unstable();
            candidates
        }
        (Refusal::Unbound, OperatorResolution::UnboundPolymorphic(operator)) => {
            vec![signature(database, operator)]
        }
        _ => panic!("unexpected refusal for {}", case.name),
    }
}

#[tokio::test]
async fn refused_operators_match_server_errors_and_parser_candidates() {
    let docker = reference_docker("operator_resolution").await.unwrap();
    let mut connection: PgConnection =
        establish_connection(docker.port, "operator_resolution").unwrap();
    connection.batch_execute(SHARED_DDL).unwrap();
    connection.batch_execute(SERVER_DDL).unwrap();
    let live = load(&mut connection);
    let parsed = ParseOptions::default()
        .with_postgres_catalog(catalog())
        .parse::<PostgreSqlDialect>(SHARED_DDL)
        .unwrap();
    let cases = [
        (
            binary("not_found", "~~", pg("i", "int4"), pg("i", "int4")),
            Refusal::NotFound,
            "operator does not exist",
        ),
        (
            binary(
                "domain_base_anyelement",
                "===",
                public("di", "dint"),
                pg("i", "int4"),
            ),
            Refusal::NotFound,
            "operator does not exist",
        ),
        (
            Case {
                name: "ambiguous_prefix",
                operator: TargetName::new("~", false),
                left: None,
                right: literal("'1'"),
                bindings: None,
            },
            Refusal::Ambiguous,
            "operator is not unique",
        ),
        (
            binary("unbound_element", "===", literal("'1'"), literal("'2'")),
            Refusal::Unbound,
            "could not determine polymorphic type",
        ),
        (
            binary("unbound_range", "@@@", literal("'[1,2)'"), pg("i", "int4")),
            Refusal::Unbound,
            "could not determine polymorphic type",
        ),
    ];
    for (case, expected, message) in cases {
        let error = connection.batch_execute(&selection_sql(&case)).unwrap_err();
        assert!(error.to_string().contains(message), "{} {error}", case.name);
        assert_eq!(
            refused(&live, &case, expected),
            refused(&parsed, &case, expected),
            "{}",
            case.name
        );
    }
}

#[tokio::test]
async fn operator_search_path_and_qualification_match_the_server() {
    let docker = reference_docker("operator_resolution").await.unwrap();
    let mut connection: PgConnection =
        establish_connection(docker.port, "operator_resolution").unwrap();
    connection.batch_execute(SHARED_DDL).unwrap();
    connection.batch_execute(SERVER_DDL).unwrap();
    for path in ["app", "app, pg_catalog", "pg_catalog"] {
        let setting = format!("SET search_path = {path};");
        connection.batch_execute(&setting).unwrap();
        let live = load(&mut connection);
        let parsed = ParseOptions::default()
            .with_postgres_catalog(catalog())
            .parse::<PostgreSqlDialect>(&format!("{SHARED_DDL}\n{setting}"))
            .unwrap();
        let catalog = Catalog::load(&mut connection, &live);
        for operator in [
            TargetName::new("=", false),
            TargetName::new("=", false).with_schema("app", false),
            TargetName::new("=", false).with_schema("pg_catalog", false),
            TargetName::new("===", false).with_schema("app", false),
        ] {
            let case = Case {
                operator,
                ..binary("search_path", "=", pg("i", "int4"), pg("i", "int4"))
            };
            compare_case(&mut connection, &live, &parsed, &catalog, &case);
        }
        let case = binary("app_visibility", "===", pg("i", "int4"), pg("i", "int4"));
        if path == "pg_catalog" {
            let error = connection.batch_execute(&selection_sql(&case)).unwrap_err();
            assert!(error.to_string().contains("operator does not exist"));
            assert_eq!(
                refused(&live, &case, Refusal::NotFound),
                refused(&parsed, &case, Refusal::NotFound)
            );
        } else {
            compare_case(&mut connection, &live, &parsed, &catalog, &case);
        }
    }
}

/// Keeps unavailable selection facts distinct from a proven refusal.
fn assert_undetermined<DB: DatabaseLike>(database: &DB, case: &Case) {
    let resolution = database.resolve_operator(
        case.operator.clone(),
        case.left.as_ref().map(|operand| operand.data_type.clone()),
        case.right.data_type.clone(),
        IdentifierCase::AsWritten,
    );
    assert!(
        matches!(resolution, OperatorResolution::Undetermined),
        "{} {:?}",
        case.name,
        case.right.data_type
    );
}

#[tokio::test]
async fn unsupported_facts_stay_undetermined_for_valid_server_expressions() {
    let docker = reference_docker("operator_resolution").await.unwrap();
    let mut connection: PgConnection =
        establish_connection(docker.port, "operator_resolution").unwrap();
    connection.batch_execute(SHARED_DDL).unwrap();
    connection.batch_execute(SERVER_DDL).unwrap();
    connection
        .batch_execute("CREATE TABLE public.typed OF public.pair;")
        .unwrap();
    let live = load(&mut connection);
    let parsed = ParseOptions::default()
        .with_postgres_catalog(catalog())
        .parse::<PostgreSqlDialect>(SHARED_DDL)
        .unwrap();
    let cases = [
        Case {
            name: "missing_enum_kind",
            operator: TargetName::new("#?", false),
            left: None,
            right: public("e", "missing"),
            bindings: None,
        },
        binary("range_facts", "@>", pg("r", "int4range"), pg("i", "int4")),
        binary(
            "typed_table",
            "<#>",
            public("(SELECT typed FROM public.typed LIMIT 1)", "typed"),
            public("p", "pair"),
        ),
    ];
    for case in cases {
        connection.batch_execute(&selection_sql(&case)).unwrap();
        assert_eq!(
            expression(&stored_query(&mut connection, "selected"), 0).name(),
            Some("OPEXPR")
        );
        assert_undetermined(&live, &case);
        assert_undetermined(&parsed, &case);
        connection
            .batch_execute("DROP VIEW public.selected;")
            .unwrap();
    }
    for unknown in [
        OperandType::Unknown,
        OperandType::Known {
            type_name: target("pg_catalog", "anyelement"),
            typmod: None,
        },
    ] {
        let case = Case {
            left: Some(Operand {
                expression: "i",
                data_type: unknown.clone(),
            }),
            right: Operand {
                expression: "i",
                data_type: unknown,
            },
            ..binary("missing_facts", "===", pg("i", "int4"), pg("i", "int4"))
        };
        assert_undetermined(&live, &case);
        assert_undetermined(&parsed, &case);
    }
}

#[tokio::test]
async fn non_enum_operand_cannot_select_an_anyenum_operator() {
    let docker = reference_docker("operator_resolution").await.unwrap();
    let mut connection: PgConnection =
        establish_connection(docker.port, "operator_resolution").unwrap();
    connection.batch_execute(SHARED_DDL).unwrap();
    connection.batch_execute(SERVER_DDL).unwrap();
    let live = load(&mut connection);
    let parsed = ParseOptions::default()
        .with_postgres_catalog(catalog())
        .parse::<PostgreSqlDialect>(SHARED_DDL)
        .unwrap();
    for (name, right) in [
        ("integer", pg("i", "int4")),
        ("enum_domain", public("dc", "dcelsius")),
        ("enum_array", public("ARRAY[e]", "_mood")),
    ] {
        let case = Case {
            name,
            operator: TargetName::new("#?", false),
            left: None,
            right,
            bindings: None,
        };
        let error = connection.batch_execute(&selection_sql(&case)).unwrap_err();
        assert!(
            error.to_string().contains("operator does not exist"),
            "{name} {error}"
        );
        assert_eq!(
            refused(&live, &case, Refusal::NotFound),
            refused(&parsed, &case, Refusal::NotFound),
            "{name}"
        );
    }
}
