//! Live cast contexts, methods and exact function identities.

use std::sync::Arc;

use diesel::{RunQueryDsl, dsl::sql, sql_types::Oid};
use pg_diesel::{
    PgDieselDatabase,
    database::{PgCatalogMetadata, PgDatabaseBuildError},
    dialect::PostgresDialect,
    sql_traits::{
        structs::{TargetName, generic_db::GenericDBBuilder},
        traits::{CastContext, CastLike, CastMethod, DatabaseLike},
    },
};

use super::{
    PgConnection, PgDieselDatabaseBuilder, SimpleConnection, establish_connection, reference_docker,
};

/// Loads catalog facts independently of the selected table schema.
fn load(connection: &mut PgConnection) -> Result<PgDieselDatabase, PgDatabaseBuildError> {
    PgDieselDatabaseBuilder::default()
        .connection(connection)
        .catalog("casts")
        .schema("public")
        .try_into()
}

#[tokio::test]
async fn casts_preserve_context_method_types_and_function_identity() {
    let docker = reference_docker("casts").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "casts").unwrap();
    connection
        .batch_execute(
            r#"
            CREATE SCHEMA "Casts";
            CREATE TYPE "Casts"."Source" AS (value integer);
            CREATE TYPE "Casts"."FunctionTarget" AS (value integer);
            CREATE TYPE "Casts"."InOutTarget" AS (value integer);
            CREATE FUNCTION "Casts".convert("Casts"."Source") RETURNS "Casts"."FunctionTarget"
                LANGUAGE sql IMMUTABLE AS 'SELECT ROW(($1).value)::"Casts"."FunctionTarget"';
            CREATE FUNCTION "Casts".convert(integer) RETURNS "Casts"."FunctionTarget"
                LANGUAGE sql IMMUTABLE AS 'SELECT ROW($1)::"Casts"."FunctionTarget"';
            CREATE FUNCTION "Casts".convert("Casts"."Source"[]) RETURNS "Casts"."FunctionTarget"[]
                LANGUAGE sql IMMUTABLE AS 'SELECT array_agg("Casts".convert(element)) FROM unnest($1) AS element';
            CREATE CAST ("Casts"."Source" AS "Casts"."FunctionTarget")
                WITH FUNCTION "Casts".convert("Casts"."Source") AS IMPLICIT;
            CREATE CAST ("Casts"."Source" AS "Casts"."InOutTarget") WITH INOUT AS ASSIGNMENT;
            CREATE CAST ("Casts"."Source"[] AS "Casts"."FunctionTarget"[])
                WITH FUNCTION "Casts".convert("Casts"."Source"[]);
            "#,
        )
        .unwrap();
    let function_oid: u32 = diesel::select(sql::<Oid>(
        r#"'"Casts".convert("Casts"."Source")'::regprocedure::oid"#,
    ))
    .get_result(&mut connection)
    .unwrap();
    let database = load(&mut connection).unwrap();
    let find = |schema, source, target| {
        database
            .casts()
            .unwrap()
            .find(|cast| {
                cast.source_type(&database)
                    == TargetName::new(source, true).with_schema(schema, true)
                    && cast.target_type(&database)
                        == TargetName::new(target, true).with_schema(schema, true)
            })
            .unwrap()
    };
    let function_cast = find("Casts", "Source", "FunctionTarget");
    assert_eq!(function_cast.context(), CastContext::Implicit);
    assert_eq!(function_cast.method(), CastMethod::Function);
    let expected = database
        .functions()
        .find(|function| function.oid() == function_oid)
        .unwrap();
    assert!(std::ptr::eq(
        function_cast.function(&database).unwrap(),
        expected
    ));

    let in_out = find("Casts", "Source", "InOutTarget");
    assert_eq!(in_out.context(), CastContext::Assignment);
    assert_eq!(in_out.method(), CastMethod::InOut);
    assert!(in_out.function(&database).is_none());

    let binary = find("pg_catalog", "text", "varchar");
    assert_eq!(binary.context(), CastContext::Implicit);
    assert_eq!(binary.method(), CastMethod::Binary);
    assert!(binary.function(&database).is_none());

    let array = find("Casts", "_Source", "_FunctionTarget");
    assert_eq!(array.context(), CastContext::Explicit);
    assert_eq!(array.method(), CastMethod::Function);
    assert_ne!(array.function(&database).unwrap().oid(), function_oid);
}

#[test]
fn casts_distinguish_unavailable_and_loaded_empty_catalogs() {
    let unavailable: PgDieselDatabase =
        GenericDBBuilder::new("unloaded".to_owned(), PostgresDialect).into();
    assert!(unavailable.casts().is_none());
    assert!(unavailable.operators().is_none());
    let empty: PgDieselDatabase = GenericDBBuilder::new("empty".to_owned(), PostgresDialect)
        .into_database(Some(Arc::new(PgCatalogMetadata::default())));
    assert_eq!(empty.casts().unwrap().count(), 0);
    assert_eq!(empty.operators().unwrap().count(), 0);
}

#[tokio::test]
async fn casts_reject_invalid_codes_and_types_but_allow_unavailable_functions() {
    let docker = reference_docker("casts").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "casts").unwrap();
    connection
        .batch_execute(
            "UPDATE pg_catalog.pg_cast SET castcontext = '?' 
             WHERE castsource = 'integer'::regtype AND casttarget = 'bigint'::regtype;",
        )
        .unwrap();
    assert!(matches!(
        load(&mut connection),
        Err(PgDatabaseBuildError::Diesel(
            diesel::result::Error::DeserializationError(_)
        ))
    ));
    connection
        .batch_execute(
            "UPDATE pg_catalog.pg_cast SET castcontext = 'i', castmethod = '?'
             WHERE castsource = 'integer'::regtype AND casttarget = 'bigint'::regtype;",
        )
        .unwrap();
    assert!(matches!(
        load(&mut connection),
        Err(PgDatabaseBuildError::Diesel(
            diesel::result::Error::DeserializationError(_)
        ))
    ));
    connection
        .batch_execute(
            "UPDATE pg_catalog.pg_cast SET castmethod = 'f', castsource = 0
             WHERE castsource = 'integer'::regtype AND casttarget = 'bigint'::regtype;",
        )
        .unwrap();
    assert!(matches!(
        load(&mut connection),
        Err(PgDatabaseBuildError::Diesel(
            diesel::result::Error::NotFound
        ))
    ));
    connection
        .batch_execute(
            "UPDATE pg_catalog.pg_cast SET castsource = 'integer'::regtype, castfunc = 0
             WHERE castsource = 0 AND casttarget = 'bigint'::regtype;",
        )
        .unwrap();
    let database = load(&mut connection).unwrap();
    let cast = database
        .casts()
        .unwrap()
        .find(|cast| {
            cast.source_type(&database)
                == TargetName::new("int4", true).with_schema("pg_catalog", true)
                && cast.target_type(&database)
                    == TargetName::new("int8", true).with_schema("pg_catalog", true)
        })
        .unwrap();
    assert_eq!(cast.method(), CastMethod::Function);
    assert!(cast.function(&database).is_none());
}
