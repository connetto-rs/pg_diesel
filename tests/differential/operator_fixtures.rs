//! Shared declarations and matching PostgreSQL operator catalogs.

use pg_diesel::sql_traits::prelude::{
    CastContext, PostgresCatalog, PostgresCatalogCast, PostgresCatalogOperator, TargetName,
};

/// Declares the same operand types and implementation functions in both databases.
pub(super) const SHARED_DDL: &str = r#"
CREATE SCHEMA app;
CREATE DOMAIN public.dtext AS text;
CREATE DOMAIN public.dint AS int4;
CREATE DOMAIN public.d3 AS varchar(3);
CREATE DOMAIN public.d5 AS varchar(5);
CREATE DOMAIN public.dintarray AS int4[];
CREATE TYPE public.celsius AS ENUM ('x');
CREATE TYPE public.fahrenheit AS ENUM ('x');
CREATE DOMAIN public.dcelsius AS public.celsius;
CREATE TYPE public.mood AS ENUM ('x');
CREATE TYPE public.pair AS (a int4);
CREATE TABLE public.parent (a int4);
CREATE TABLE public.child () INHERITS (public.parent);
CREATE TABLE public.samples (
    s int2,
    i int4,
    b int8,
    n numeric,
    t text,
    v3 varchar(3),
    v5 varchar(5),
    dt public.dtext,
    di public.dint,
    dv public.d5,
    a int4[],
    da public.dintarray,
    c public.celsius,
    dc public.dcelsius,
    p public.pair,
    pa public.parent[],
    e public.mood,
    r int4range,
    bytes bytea
);
CREATE FUNCTION public.element_equal(anyelement, anyelement) RETURNS bool
    LANGUAGE sql IMMUTABLE STRICT AS 'SELECT $1 = $2';
CREATE FUNCTION public.element_in_array(anyelement, anyarray) RETURNS bool
    LANGUAGE sql IMMUTABLE STRICT AS 'SELECT $1 = ANY($2)';
CREATE FUNCTION public.range_contains(anyrange, anyelement) RETURNS bool
    LANGUAGE sql IMMUTABLE STRICT AS 'SELECT $1 @> $2';
CREATE FUNCTION public.any_values(int4, "any") RETURNS jsonb
    LANGUAGE internal STABLE AS 'jsonb_build_array';
CREATE FUNCTION public.dint_equal(public.dint, public.dint) RETURNS bool
    LANGUAGE sql IMMUTABLE STRICT AS 'SELECT $1::int4 = $2::int4';
CREATE FUNCTION public.d3_equal(public.d3, public.d3) RETURNS bool
    LANGUAGE sql IMMUTABLE STRICT AS 'SELECT $1::text = $2::text';
CREATE FUNCTION public.pair_record_equal(public.pair, record) RETURNS bool
    LANGUAGE plpgsql IMMUTABLE STRICT AS 'BEGIN RETURN $1 = $2; END;';
CREATE FUNCTION public.parent_record_values(public.parent, record[]) RETURNS jsonb
    LANGUAGE internal STABLE AS 'jsonb_build_array';
CREATE FUNCTION public.fahrenheit_label(public.fahrenheit) RETURNS text
    LANGUAGE sql STABLE STRICT AS 'SELECT $1::text';
CREATE FUNCTION public.enum_label(anyenum) RETURNS text
    LANGUAGE sql STABLE STRICT AS 'SELECT $1::text';
CREATE FUNCTION app.int4_equal(int4, int4) RETURNS bool
    LANGUAGE sql IMMUTABLE STRICT AS 'SELECT pg_catalog.int4eq($1, $2)';
"#;

/// Installs the server declarations represented by the configured catalog.
pub(super) const SERVER_DDL: &str = r#"
CREATE OPERATOR public.=== (
    LEFTARG = anyelement, RIGHTARG = anyelement, FUNCTION = public.element_equal
);
CREATE OPERATOR public.&&& (
    LEFTARG = anyelement, RIGHTARG = anyarray, FUNCTION = public.element_in_array
);
CREATE OPERATOR public.@@@ (
    LEFTARG = anyrange, RIGHTARG = anyelement, FUNCTION = public.range_contains
);
CREATE OPERATOR public.~~~ (
    LEFTARG = int4, RIGHTARG = "any", FUNCTION = public.any_values
);
CREATE OPERATOR public.<%> (
    LEFTARG = public.dint, RIGHTARG = public.dint, FUNCTION = public.dint_equal
);
CREATE OPERATOR public.<~> (
    LEFTARG = public.d3, RIGHTARG = public.d3, FUNCTION = public.d3_equal
);
CREATE OPERATOR public.<#> (
    LEFTARG = public.pair, RIGHTARG = record, FUNCTION = public.pair_record_equal
);
CREATE OPERATOR public.<^> (
    LEFTARG = public.parent, RIGHTARG = record[], FUNCTION = public.parent_record_values
);
CREATE OPERATOR public.<!> (
    RIGHTARG = public.fahrenheit, FUNCTION = public.fahrenheit_label
);
CREATE OPERATOR public.#? (
    RIGHTARG = anyenum, FUNCTION = public.enum_label
);
CREATE OPERATOR app.= (
    LEFTARG = int4, RIGHTARG = int4, FUNCTION = app.int4_equal
);
CREATE OPERATOR app.=== (
    LEFTARG = int4, RIGHTARG = int4, FUNCTION = app.int4_equal
);
CREATE CAST (public.celsius AS public.fahrenheit) WITH INOUT AS IMPLICIT;
"#;

/// Mirrors the server's custom signatures, implementations and implicit cast.
pub(super) fn catalog() -> PostgresCatalog {
    let pg = |name: &'static str| TargetName::new(name, false).with_schema("pg_catalog", false);
    let public = |name: &'static str| TargetName::new(name, false).with_schema("public", false);
    let mut catalog = PostgresCatalog::default().with_cast(PostgresCatalogCast::through_io(
        public("celsius"),
        public("fahrenheit"),
        CastContext::Implicit,
    ));
    for (name, left, right, function, result) in [
        (
            "===",
            Some(pg("anyelement")),
            pg("anyelement"),
            "element_equal",
            "bool",
        ),
        (
            "&&&",
            Some(pg("anyelement")),
            pg("anyarray"),
            "element_in_array",
            "bool",
        ),
        (
            "@@@",
            Some(pg("anyrange")),
            pg("anyelement"),
            "range_contains",
            "bool",
        ),
        ("~~~", Some(pg("int4")), pg("any"), "any_values", "jsonb"),
        (
            "<%>",
            Some(public("dint")),
            public("dint"),
            "dint_equal",
            "bool",
        ),
        ("<~>", Some(public("d3")), public("d3"), "d3_equal", "bool"),
        (
            "<#>",
            Some(public("pair")),
            pg("record"),
            "pair_record_equal",
            "bool",
        ),
        (
            "<^>",
            Some(public("parent")),
            pg("_record"),
            "parent_record_values",
            "jsonb",
        ),
        (
            "<!>",
            None,
            public("fahrenheit"),
            "fahrenheit_label",
            "text",
        ),
        ("#?", None, pg("anyenum"), "enum_label", "text"),
    ] {
        catalog = catalog.with_operator(
            PostgresCatalogOperator::new(name, left, right)
                .with_schema("public", false)
                .with_implementation(public(function), pg(result)),
        );
    }
    for name in ["=", "==="] {
        catalog = catalog.with_operator(
            PostgresCatalogOperator::new(name, Some(pg("int4")), pg("int4"))
                .with_schema("app", false)
                .with_implementation(
                    TargetName::new("int4_equal", false).with_schema("app", false),
                    pg("bool"),
                ),
        );
    }
    catalog
}
