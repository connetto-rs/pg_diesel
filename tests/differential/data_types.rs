//! Declared type facts loaded from PostgreSQL catalogs.

use diesel::{
    RunQueryDsl,
    dsl::sql,
    sql_types::{Oid, Text},
};
use pg_diesel::{
    PgDieselDatabase,
    database::CatalogCache,
    dialect::PostgresDialect,
    model_metadata::PgDataType,
    sql_traits::{
        structs::{TargetName, generic_db::GenericDBBuilder},
        traits::{ArrayDimension, ColumnCollation, DataTypeLike, StructureKind},
    },
};

use super::{
    COMPILED_MAJORS, PgConnection, SimpleConnection, establish_connection, reference_docker,
};

/// Resolves a declared type through the cached catalog.
fn declared_type(
    connection: &mut PgConnection,
    cache: &CatalogCache,
    name: &str,
) -> Option<PgDataType> {
    let oid: u32 = diesel::select(
        sql::<Oid>("CAST(")
            .bind::<Text, _>(name)
            .sql(" AS regtype)::oid"),
    )
    .get_result(connection)
    .unwrap();
    PgDataType::from_type(cache.pg_type(oid).unwrap(), -1, cache, 0)
}

#[tokio::test]
async fn structured_fields_preserve_nested_types_names_constraints_and_collations() {
    for &major in COMPILED_MAJORS {
        let docker = reference_docker(major, "data_types").await.unwrap();
        let mut connection: PgConnection = establish_connection(docker.port, "data_types").unwrap();
        connection
            .batch_execute(
                r#"
    CREATE TYPE leaf AS ("Label" varchar(7) COLLATE "C", discarded text, amount numeric(8,2));
    ALTER TYPE leaf DROP ATTRIBUTE discarded;
    CREATE DOMAIN leaf_domain AS leaf;
    CREATE TYPE empty_record AS ();
    CREATE TABLE row_type (required integer NOT NULL, optional text);
    CREATE TYPE outer_record AS (
        item leaf_domain, items leaf[], row_value row_type, vacant empty_record
    );
"#,
            )
            .unwrap();
        let cache = CatalogCache::load(&mut connection).unwrap();
        let database: PgDieselDatabase =
            GenericDBBuilder::new("data_types".to_owned(), PostgresDialect).into();
        let outer = declared_type(&mut connection, &cache, "outer_record").unwrap();
        assert_eq!(outer.structure_kind(&database), Some(StructureKind::Struct));
        let fields: Vec<_> = outer.structured_fields(&database).unwrap().collect();
        assert_eq!(
            fields
                .iter()
                .map(|field| field.name().unwrap())
                .collect::<Vec<_>>(),
            ["item", "items", "row_value", "vacant"]
        );
        assert!(
            fields
                .iter()
                .all(|field| field.name_is_quoted() && field.is_nullable())
        );
        let domain = fields[0].data_type();
        assert!(domain.structure_kind(&database).is_none());
        assert!(domain.structured_fields(&database).is_none());
        assert!(domain.admits_null(&database));
        let leaf = domain.base_type(&database).unwrap();
        let leaf_fields: Vec<_> = leaf.structured_fields(&database).unwrap().collect();
        assert_eq!(
            leaf_fields
                .iter()
                .map(|field| field.name().unwrap())
                .collect::<Vec<_>>(),
            ["Label", "amount"]
        );
        assert_eq!(
            leaf_fields[0].data_type().max_character_length(&database),
            Some(7)
        );
        let ColumnCollation::Named(collation) = leaf_fields[0].collation() else {
            panic!("the field declares a named collation");
        };
        assert_eq!(
            collation.name(),
            &TargetName::new("C", true).with_schema("pg_catalog", true)
        );
        assert_eq!(collation.postgres_deterministic(), Some(true));
        assert_eq!(
            leaf_fields[1].data_type().numeric_precision(&database),
            Some(8)
        );
        assert_eq!(leaf_fields[1].data_type().numeric_scale(&database), Some(2));
        assert_eq!(
            leaf_fields[1].data_type().is_unsigned(&database),
            Some(false)
        );
        assert_eq!(fields[1].data_type().array_element(&database), Some(leaf));
        let row_fields: Vec<_> = fields[2]
            .data_type()
            .structured_fields(&database)
            .unwrap()
            .collect();
        assert!(!row_fields[0].is_nullable());
        assert!(row_fields[0].data_type().admits_null(&database));
        assert!(row_fields[1].is_nullable());
        assert_eq!(row_fields[1].collation(), &ColumnCollation::DatabaseDefault);
        let empty = fields[3].data_type();
        assert_eq!(empty.structure_kind(&database), Some(StructureKind::Struct));
        assert_eq!(empty.structured_fields(&database).unwrap().count(), 0);
        for (name, unsigned) in [
            ("int4", Some(false)),
            ("float8", Some(false)),
            ("text", None),
            ("leaf_domain", None),
        ] {
            let data_type = declared_type(&mut connection, &cache, name).unwrap();
            assert_eq!(data_type.is_unsigned(&database), unsigned, "{name}");
            assert!(data_type.admits_null(&database), "{name}");
            assert!(data_type.map_entry(&database).is_none(), "{name}");
        }
    }
}

#[tokio::test]
async fn incomplete_or_cyclic_composites_fail_type_construction() {
    for &major in COMPILED_MAJORS {
        let docker = reference_docker(major, "invalid_types").await.unwrap();
        let mut connection: PgConnection =
            establish_connection(docker.port, "invalid_types").unwrap();
        connection
            .batch_execute("CREATE TYPE broken AS (value text);")
            .unwrap();
        for assignment in [
            "atttypid = 0",
            "atttypid = 'broken'::regtype::oid",
            "atttypid = 'broken[]'::regtype::oid",
            "attcollation = 4294967295",
        ] {
            connection
                .batch_execute(&format!(
                    "BEGIN; UPDATE pg_catalog.pg_attribute SET {assignment}
         WHERE attrelid = (SELECT typrelid FROM pg_catalog.pg_type WHERE oid = 'broken'::regtype)
         AND attnum = 1;"
                ))
                .unwrap();
            let cache = CatalogCache::load(&mut connection).unwrap();
            assert!(
                declared_type(&mut connection, &cache, "broken").is_none(),
                "{assignment}"
            );
            connection.batch_execute("ROLLBACK").unwrap();
        }
    }
}

#[tokio::test]
async fn enumeration_preserves_catalog_order_labels_and_type_boundaries() {
    for &major in COMPILED_MAJORS {
        use pg_diesel::sql_traits::traits::EnumerationKind;

        let docker = reference_docker(major, "enumeration").await.unwrap();
        let mut connection: PgConnection =
            establish_connection(docker.port, "enumeration").unwrap();
        connection
            .batch_execute(
                r#"
        CREATE SCHEMA "Enums";
        CREATE TYPE "Enums"."Mood" AS ENUM ('z', 'a');
        ALTER TYPE "Enums"."Mood" ADD VALUE ' middle ' BEFORE 'a';
        ALTER TYPE "Enums"."Mood" ADD VALUE 'first' BEFORE 'z';
        ALTER TYPE "Enums"."Mood" ADD VALUE 'quote''label' AFTER 'z';
        ALTER TYPE "Enums"."Mood" RENAME VALUE 'a' TO 'end';
        CREATE TYPE public."Mood" AS ENUM ('other');
        CREATE TYPE "Enums"."Empty" AS ENUM ();
        CREATE DOMAIN "Enums"."Domain" AS "Enums"."Mood";
        "#,
            )
            .unwrap();
        let cache = CatalogCache::load(&mut connection).unwrap();
        let database: PgDieselDatabase =
            GenericDBBuilder::new("enumeration".to_owned(), PostgresDialect).into();
        let check = |data_type: &PgDataType, expected: &[&str]| {
            let enumeration = data_type.enumeration(&database).unwrap();
            assert_eq!(enumeration.kind(), EnumerationKind::Enum);
            assert_eq!(enumeration.width(), None);
            let labels: Vec<_> = enumeration
                .into_members()
                .map(|member| {
                    assert_eq!(member.value(), None);
                    member.label()
                })
                .collect();
            assert_eq!(labels, expected);
        };
        let expected = ["first", "z", "quote'label", " middle ", "end"];
        let mood = declared_type(&mut connection, &cache, r#""Enums"."Mood""#).unwrap();
        check(&mood, &expected);
        let other = declared_type(&mut connection, &cache, r#"public."Mood""#).unwrap();
        check(&other, &["other"]);
        let empty = declared_type(&mut connection, &cache, r#""Enums"."Empty""#).unwrap();
        check(&empty, &[]);
        let domain = declared_type(&mut connection, &cache, r#""Enums"."Domain""#).unwrap();
        assert!(domain.enumeration(&database).is_none());
        check(domain.base_type(&database).unwrap(), &expected);
        let array = declared_type(&mut connection, &cache, r#""Enums"."Mood"[]"#).unwrap();
        assert!(array.enumeration(&database).is_none());
        check(array.array_element(&database).unwrap(), &expected);
        let integer = declared_type(&mut connection, &cache, "integer").unwrap();
        assert!(integer.enumeration(&database).is_none());
    }
}

#[tokio::test]
async fn enum_without_cached_members_fails_type_construction() {
    for &major in COMPILED_MAJORS {
        use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
        use pg_diesel::{models::PgType, schema::pg_catalog::pg_type::pg_type};

        let docker = reference_docker(major, "uncached_enum").await.unwrap();
        let mut connection: PgConnection =
            establish_connection(docker.port, "uncached_enum").unwrap();
        let cache = CatalogCache::load(&mut connection).unwrap();
        connection
            .batch_execute("CREATE TYPE later AS ENUM ('ready')")
            .unwrap();
        let row = pg_type::table
            .filter(pg_type::typname.eq("later"))
            .select(PgType::as_select())
            .first::<PgType>(&mut connection)
            .unwrap();
        assert!(PgDataType::from_type(&row, -1, &cache, 0).is_none());
    }
}

#[tokio::test]
async fn lengths_and_units_preserve_declared_type_boundaries() {
    for &major in COMPILED_MAJORS {
        use pg_diesel::sql_traits::traits::CharacterLengthUnit;

        let docker = reference_docker(major, "type_lengths").await.unwrap();
        let mut connection: PgConnection =
            establish_connection(docker.port, "type_lengths").unwrap();
        connection
            .batch_execute(
                r"
        CREATE DOMAIN bit_domain AS bit varying(13);
        CREATE DOMAIN text_domain AS varchar(5);
        CREATE TYPE lengths AS (
            flag bit, mask bit(8), bits bit varying(16), any_bits varbit,
            letter char, code char(3), title varchar(7), any_title varchar,
            body text, payload bytea, number integer,
            zoned_timestamp timestamp with time zone, zoned_time time with time zone,
            plain_timestamp timestamp without time zone, plain_time time without time zone
        );
        CREATE TYPE wrapped_lengths AS (
            domain_bits bit_domain, array_bits bit(9)[],
            domain_text text_domain, array_text varchar(11)[]
        );
        ",
            )
            .unwrap();
        let cache = CatalogCache::load(&mut connection).unwrap();
        let database: PgDieselDatabase =
            GenericDBBuilder::new("type_lengths".to_owned(), PostgresDialect).into();
        let lengths = declared_type(&mut connection, &cache, "lengths").unwrap();
        let facts: Vec<_> = lengths
            .structured_fields(&database)
            .unwrap()
            .map(|field| {
                let data_type = field.data_type();
                assert_eq!(data_type.max_binary_length(&database), None);
                assert_eq!(data_type.time_zone(&database), None);
                (
                    field.name().unwrap(),
                    data_type.max_bit_length(&database),
                    data_type.max_character_length(&database),
                    data_type.character_length_unit(&database),
                )
            })
            .collect();
        let characters = Some(CharacterLengthUnit::Characters);
        assert_eq!(
            facts,
            [
                ("flag", Some(1), None, None),
                ("mask", Some(8), None, None),
                ("bits", Some(16), None, None),
                ("any_bits", None, None, None),
                ("letter", None, Some(1), characters),
                ("code", None, Some(3), characters),
                ("title", None, Some(7), characters),
                ("any_title", None, None, None),
                ("body", None, None, None),
                ("payload", None, None, None),
                ("number", None, None, None),
                ("zoned_timestamp", None, None, None),
                ("zoned_time", None, None, None),
                ("plain_timestamp", None, None, None),
                ("plain_time", None, None, None),
            ]
        );
        let wrapped = declared_type(&mut connection, &cache, "wrapped_lengths").unwrap();
        let nested_facts: Vec<_> = wrapped
            .structured_fields(&database)
            .unwrap()
            .map(|field| {
                let data_type = field.data_type();
                assert_eq!(data_type.max_bit_length(&database), None);
                assert_eq!(data_type.max_character_length(&database), None);
                assert_eq!(data_type.character_length_unit(&database), None);
                let nested = data_type
                    .base_type(&database)
                    .or_else(|| data_type.array_element(&database))
                    .unwrap();
                (
                    field.name().unwrap(),
                    nested.max_bit_length(&database),
                    nested.max_character_length(&database),
                    nested.character_length_unit(&database),
                )
            })
            .collect();
        assert_eq!(
            nested_facts,
            [
                ("domain_bits", Some(13), None, None),
                ("array_bits", Some(9), None, None),
                ("domain_text", None, Some(5), characters),
                ("array_text", None, Some(11), characters),
            ]
        );
    }
}

#[tokio::test]
async fn arrays_preserve_declared_dimensions_and_domain_boundaries() {
    for &major in COMPILED_MAJORS {
        let docker = reference_docker(major, "array_dimensions").await.unwrap();
        let mut connection: PgConnection =
            establish_connection(docker.port, "array_dimensions").unwrap();
        connection
            .batch_execute(
                r"
        CREATE DOMAIN array_domain AS integer[2][3];
        CREATE DOMAIN nested_array_domain AS array_domain;
        CREATE DOMAIN scalar_domain AS integer;
        CREATE TABLE array_values (
            id integer PRIMARY KEY,
            ordinary integer[2] NOT NULL,
            matrix integer[3][4] NOT NULL,
            cube integer[][][],
            scalar integer,
            wrapped nested_array_domain,
            members scalar_domain[]
        );
        CREATE TABLE copied_arrays (LIKE array_values);
        CREATE TABLE selected_arrays AS SELECT * FROM array_values;
        ",
            )
            .unwrap();
        let cache = CatalogCache::load(&mut connection).unwrap();
        let database: PgDieselDatabase =
            GenericDBBuilder::new("array_dimensions".to_owned(), PostgresDialect).into();
        let row = declared_type(&mut connection, &cache, "array_values").unwrap();
        let fields: Vec<_> = row.structured_fields(&database).unwrap().collect();
        assert_eq!(fields.len(), 7);
        let nullable = ArrayDimension::new(None, true);
        let rectangular = ArrayDimension::new(None, false);
        for (field, expected) in fields.iter().zip([
            None,
            Some(vec![nullable]),
            Some(vec![rectangular, nullable]),
            Some(vec![rectangular, rectangular, nullable]),
            None,
            None,
            Some(vec![nullable]),
        ]) {
            let data_type = field.data_type();
            assert_eq!(
                data_type.enforces_array_dimensions(&database),
                expected.as_ref().map(|_| false),
                "{:?}",
                field.name()
            );
            assert_eq!(
                data_type
                    .array_dimensions(&database)
                    .map(core::num::NonZeroU32::get),
                expected
                    .as_ref()
                    .map(|dimensions| u32::try_from(dimensions.len()).unwrap())
            );
            assert_eq!(
                data_type
                    .array_dimension_declarations(&database)
                    .map(Iterator::collect::<Vec<_>>),
                expected
            );
        }
        let domain = fields[5].data_type();
        assert!(domain.array_element(&database).is_none());
        let base = domain.base_type(&database).unwrap();
        assert!(base.base_type(&database).is_none());
        assert_eq!(base.enforces_array_dimensions(&database), Some(false));
        assert_eq!(
            base.array_dimensions(&database)
                .map(core::num::NonZeroU32::get),
            Some(2)
        );
        assert_eq!(
            base.array_dimension_declarations(&database)
                .unwrap()
                .collect::<Vec<_>>(),
            [rectangular, nullable]
        );
        let member = fields[6].data_type().array_element(&database).unwrap();
        assert!(member.base_type(&database).is_some());
        assert!(member.admits_null(&database));
        assert!(member.array_dimension_declarations(&database).is_none());

        for name in ["copied_arrays", "selected_arrays"] {
            let copied = declared_type(&mut connection, &cache, name).unwrap();
            let mut array_names = Vec::new();
            for field in copied.structured_fields(&database).unwrap() {
                let data_type = field.data_type();
                if data_type.array_element(&database).is_some() {
                    array_names.push(field.name().unwrap());
                    assert_eq!(data_type.array_dimensions(&database), None);
                    assert_eq!(data_type.enforces_array_dimensions(&database), Some(false));
                    let mut declarations =
                        data_type.array_dimension_declarations(&database).unwrap();
                    assert_eq!(declarations.next(), None);
                }
            }
            assert_eq!(array_names, ["ordinary", "matrix", "cube", "members"]);
        }
    }
}

#[tokio::test]
async fn array_values_ignore_declared_shape_and_require_rectangular_elements() {
    for &major in COMPILED_MAJORS {
        use diesel::{
            ExpressionMethods, QueryDsl,
            dsl::{array, array_fill, array_length, array_ndims},
            sql_types::{Array, Integer, Nullable},
        };

        diesel::table! {
            array_values (id) {
                id -> Integer,
                ordinary -> Array<Nullable<Integer>>,
                matrix -> Array<Nullable<Integer>>,
            }
        }

        let docker = reference_docker(major, "array_values").await.unwrap();
        let mut connection: PgConnection =
            establish_connection(docker.port, "array_values").unwrap();
        connection
            .batch_execute(
                r"
        CREATE TABLE array_values (
            id integer PRIMARY KEY,
            ordinary integer[2] NOT NULL,
            matrix integer[3][4] NOT NULL
        );
        ",
            )
            .unwrap();
        diesel::insert_into(array_values::table)
            .values((
                array_values::id.eq(1),
                array_values::ordinary
                    .eq(array_fill::<Nullable<Integer>, _, _>(Some(7), vec![2, 3])),
                array_values::matrix.eq(vec![Some(1), None]),
            ))
            .execute(&mut connection)
            .unwrap();
        let shape = array_values::table
            .select((
                array_ndims(array_values::ordinary),
                array_length(array_values::ordinary, 1),
                array_length(array_values::ordinary, 2),
                array_ndims(array_values::matrix),
                array_values::matrix,
            ))
            .get_result::<(i32, Option<i32>, Option<i32>, i32, Vec<Option<i32>>)>(&mut connection)
            .unwrap();
        assert_eq!(shape, (2, Some(2), Some(3), 1, vec![Some(1), None]));

        let rectangular_nulls =
            array::<Array<Nullable<Integer>>, _>((vec![Some(1), None], vec![None, Some(2)]));
        assert_eq!(
            diesel::select(array_length(rectangular_nulls, 2))
                .get_result::<Option<i32>>(&mut connection)
                .unwrap(),
            Some(2)
        );
        let ragged = array::<Array<Nullable<Integer>>, _>((vec![Some(1), None], vec![Some(2)]));
        let error = diesel::select(array_length(ragged, 2))
            .get_result::<Option<i32>>(&mut connection)
            .unwrap_err();
        assert!(error.to_string().contains(
            "multidimensional arrays must have array expressions with matching dimensions"
        ));
        let null_subarray = array::<Nullable<Array<Nullable<Integer>>>, _>((
            Some(vec![Some(1), None]),
            None::<Vec<Option<i32>>>,
        ));
        let error = diesel::select(array_length(null_subarray, 2))
            .get_result::<Option<i32>>(&mut connection)
            .unwrap_err();
        assert!(error.to_string().contains(
            "multidimensional arrays must have array expressions with matching dimensions"
        ));
    }
}
