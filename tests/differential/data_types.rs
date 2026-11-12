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
        traits::{ColumnCollation, DataTypeLike, StructureKind},
    },
};

use super::{PgConnection, SimpleConnection, establish_connection, reference_docker};

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
    let docker = reference_docker("data_types").await.unwrap();
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

#[tokio::test]
async fn incomplete_or_cyclic_composites_fail_type_construction() {
    let docker = reference_docker("invalid_types").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "invalid_types").unwrap();
    connection
        .batch_execute("CREATE TYPE broken AS (value text);")
        .unwrap();
    for assignment in [
        "atttypid = 0",
        "atttypid = 'broken'::regtype::oid",
        "atttypid = 'broken[]'::regtype::oid",
        "attcollation = 4294967295",
    ] {
        connection.batch_execute(&format!(
            "BEGIN; UPDATE pg_catalog.pg_attribute SET {assignment}
             WHERE attrelid = (SELECT typrelid FROM pg_catalog.pg_type WHERE oid = 'broken'::regtype)
             AND attnum = 1;"
        )).unwrap();
        let cache = CatalogCache::load(&mut connection).unwrap();
        assert!(
            declared_type(&mut connection, &cache, "broken").is_none(),
            "{assignment}"
        );
        connection.batch_execute("ROLLBACK").unwrap();
    }
}

#[tokio::test]
async fn enumeration_preserves_catalog_order_labels_and_type_boundaries() {
    use pg_diesel::sql_traits::traits::EnumerationKind;

    let docker = reference_docker("enumeration").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "enumeration").unwrap();
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

#[tokio::test]
async fn enum_without_cached_members_fails_type_construction() {
    use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
    use pg_diesel::{models::PgType, schema::pg_catalog::pg_type::pg_type};

    let docker = reference_docker("uncached_enum").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "uncached_enum").unwrap();
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

#[tokio::test]
async fn lengths_and_units_preserve_declared_type_boundaries() {
    use pg_diesel::sql_traits::traits::CharacterLengthUnit;

    let docker = reference_docker("type_lengths").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "type_lengths").unwrap();
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
