//! Conversion facts read independently of the selected table schemas.

use std::collections::HashMap;

use diesel::{QueryDsl, RunQueryDsl, SelectableHelper, dsl::sql, sql_types::Oid};
use pg_diesel::{
    PgDieselDatabase,
    dialect::PostgresDialect,
    models::PgType,
    schema::pg_catalog::{pg_namespace::pg_namespace, pg_type::pg_type},
    sql_traits::{
        structs::{TargetName, TypeCategory, generic_db::GenericDBBuilder},
        traits::DatabaseLike,
    },
};

use super::{
    PgConnection, PgDieselDatabaseBuilder, SimpleConnection, establish_connection, reference_docker,
};

#[tokio::test]
async fn type_facts_preserve_catalog_categories_arrays_and_domain_bases() {
    let docker = reference_docker("type_facts").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "type_facts").unwrap();
    connection
        .batch_execute(
            r#"
            CREATE SCHEMA "Types";
            CREATE TYPE "Types"."Pair" AS (value integer);
            CREATE TYPE "Types"."Mood" AS ENUM ('calm', 'busy');
            CREATE DOMAIN "Types"."Label" AS varchar(5);
            CREATE DOMAIN "Types"."Nested" AS "Types"."Label";
            CREATE DOMAIN "Types"."Count" AS integer;
            CREATE DOMAIN "Types"."List" AS integer[];
            "#,
        )
        .unwrap();
    let types: HashMap<u32, PgType> = pg_type::table
        .select(PgType::as_select())
        .load::<PgType>(&mut connection)
        .unwrap()
        .into_iter()
        .map(|row| (row.oid, row))
        .collect();
    let namespaces: HashMap<u32, String> = pg_namespace::table
        .select((pg_namespace::oid, pg_namespace::nspname))
        .load(&mut connection)
        .unwrap()
        .into_iter()
        .collect();
    let array_handler: u32 = diesel::select(sql::<Oid>(
        "'pg_catalog.array_subscript_handler'::regproc::oid",
    ))
    .get_result(&mut connection)
    .unwrap();
    let database: PgDieselDatabase = PgDieselDatabaseBuilder::default()
        .connection(&mut connection)
        .catalog("type_facts")
        .schema("public")
        .try_into()
        .unwrap();
    let identity = |oid| {
        let row = &types[&oid];
        TargetName::new(row.typname.as_str(), true)
            .with_schema(namespaces[&row.typnamespace].as_str(), true)
    };
    for row in types.values() {
        let name = identity(row.oid);
        let facts = database
            .type_facts(name.clone())
            .unwrap_or_else(|| panic!("{name}"));
        assert_eq!(
            facts.category().code(),
            row.typcategory.as_bytes()[0],
            "{name}"
        );
        assert_eq!(facts.is_preferred(), row.typispreferred, "{name}");
        assert_eq!(
            facts.array_type(),
            (row.typarray != 0).then(|| identity(row.typarray)).as_ref(),
            "{name}"
        );
        assert_eq!(
            facts.array_element(),
            (row.typsubscript == array_handler)
                .then(|| identity(row.typelem))
                .as_ref(),
            "{name}"
        );
        assert_eq!(facts.domain().is_some(), row.typtype == "d", "{name}");
    }
    for (name, base, typmod) in [
        ("Label", "varchar", Some(9)),
        ("Nested", "varchar", Some(9)),
        ("Count", "int4", None),
        ("List", "_int4", None),
    ] {
        let facts = database
            .type_facts(TargetName::new(name, true).with_schema("Types", true))
            .unwrap();
        let domain = facts.domain().unwrap();
        assert_eq!(
            domain.base(),
            &TargetName::new(base, true).with_schema("pg_catalog", true)
        );
        assert_eq!(domain.base_typmod(), typmod);
    }
}

#[tokio::test]
async fn stored_type_names_preserve_quoting_and_search_path() {
    let docker = reference_docker("type_names").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "type_names").unwrap();
    connection
        .batch_execute(
            r#"
            CREATE SCHEMA "Types";
            CREATE DOMAIN "Types"."Label" AS varchar(5);
            CREATE DOMAIN "Types".int4 AS text;
            SET search_path TO "Types", pg_catalog;
            "#,
        )
        .unwrap();
    let database: PgDieselDatabase = PgDieselDatabaseBuilder::default()
        .connection(&mut connection)
        .catalog("type_names")
        .schema("public")
        .try_into()
        .unwrap();
    let unqualified = database.type_facts(TargetName::new("INT4", false)).unwrap();
    assert_eq!(unqualified.category(), TypeCategory::STRING);
    assert_eq!(unqualified.domain().unwrap().base().name(), "text");
    assert_eq!(
        database
            .type_facts(TargetName::new("INT4", false).with_schema("PG_CATALOG", false))
            .unwrap()
            .category(),
        TypeCategory::NUMERIC
    );
    for name in [
        TargetName::new("INT4", true).with_schema("pg_catalog", true),
        TargetName::new("Label", true).with_schema("types", true),
        TargetName::new("missing", true).with_schema("Types", true),
        TargetName::new("int4[]", true).with_schema("pg_catalog", true),
    ] {
        assert!(database.type_facts(name.clone()).is_none(), "{name}");
    }
}

#[tokio::test]
async fn unqualified_types_follow_implicit_and_explicit_catalog_paths() {
    let docker = reference_docker("type_paths").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "type_paths").unwrap();
    connection
        .batch_execute("CREATE DOMAIN public.int4 AS text;")
        .unwrap();
    for (statement, expected) in [
        ("SET search_path TO public", TypeCategory::NUMERIC),
        (
            "CREATE TEMP TABLE int4 (value boolean)",
            TypeCategory::COMPOSITE,
        ),
        (
            "SET search_path TO public, pg_catalog, pg_temp",
            TypeCategory::STRING,
        ),
        (
            "SET search_path TO pg_catalog, public, pg_temp",
            TypeCategory::NUMERIC,
        ),
        (
            "SET search_path TO pg_temp, pg_catalog, public",
            TypeCategory::COMPOSITE,
        ),
    ] {
        connection.batch_execute(statement).unwrap();
        let database: PgDieselDatabase = PgDieselDatabaseBuilder::default()
            .connection(&mut connection)
            .catalog("type_paths")
            .schema("public")
            .try_into()
            .unwrap();
        assert_eq!(
            database
                .type_facts(TargetName::new("int4", false))
                .unwrap()
                .category(),
            expected,
            "{statement}",
        );
        assert!(
            database
                .type_facts(TargetName::new("missing", false))
                .is_none()
        );
    }
}

#[test]
fn unavailable_type_catalog_does_not_invent_builtin_facts() {
    let database: PgDieselDatabase =
        GenericDBBuilder::new("unloaded".to_owned(), PostgresDialect).into();
    assert!(
        database
            .type_facts(TargetName::new("int4", true).with_schema("pg_catalog", true))
            .is_none()
    );
}

#[tokio::test]
async fn type_catalog_preserves_category_codes_and_rejects_broken_references() {
    use diesel::ExpressionMethods;
    use pg_diesel::database::PgDatabaseBuildError;

    let docker = reference_docker("invalid_types").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "invalid_types").unwrap();
    connection
        .batch_execute("CREATE DOMAIN public.checked AS integer")
        .unwrap();
    let row = pg_type::table
        .filter(pg_type::typname.eq("checked"))
        .select(PgType::as_select())
        .first::<PgType>(&mut connection)
        .unwrap();
    for (category, base, array, namespace, malformed) in [
        ("", row.typbasetype, row.typarray, row.typnamespace, true),
        ("N", row.oid, row.typarray, row.typnamespace, true),
        ("N", 0, row.typarray, row.typnamespace, false),
        ("N", row.typbasetype, u32::MAX, row.typnamespace, false),
        ("N", row.typbasetype, row.typarray, u32::MAX, false),
    ] {
        diesel::update(pg_type::table.filter(pg_type::oid.eq(row.oid)))
            .set((
                pg_type::typcategory.eq(category),
                pg_type::typbasetype.eq(base),
                pg_type::typarray.eq(array),
                pg_type::typnamespace.eq(namespace),
            ))
            .execute(&mut connection)
            .unwrap();
        let result: Result<PgDieselDatabase, PgDatabaseBuildError> =
            PgDieselDatabaseBuilder::default()
                .connection(&mut connection)
                .catalog("invalid_types")
                .schema("public")
                .try_into();
        if category.is_empty() {
            let database = result.unwrap();
            assert_eq!(
                database
                    .type_facts(TargetName::new("checked", true).with_schema("public", true))
                    .unwrap()
                    .category()
                    .code(),
                0
            );
            continue;
        }
        if malformed {
            assert!(
                matches!(
                    result,
                    Err(PgDatabaseBuildError::Diesel(
                        diesel::result::Error::DeserializationError(_)
                    ))
                ),
                "{result:?}"
            );
        } else {
            assert!(
                matches!(
                    result,
                    Err(PgDatabaseBuildError::Diesel(
                        diesel::result::Error::NotFound
                    ))
                ),
                "{result:?}"
            );
        }
    }
}
