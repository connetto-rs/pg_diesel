//! Live operator signatures and exact implementing functions.

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use pg_diesel::PgDieselDatabase;
use pg_diesel::schema::pg_catalog::{pg_operator::pg_operator, pg_proc::pg_proc};
use pg_diesel::sql_traits::{
    structs::TargetName,
    traits::{DatabaseLike, OperatorKind, OperatorLike},
};

use super::{
    PgConnection, PgDieselDatabaseBuilder, SimpleConnection, establish_connection, reference_docker,
};

#[tokio::test]
async fn operators_preserve_signatures_shells_and_function_identity() {
    let docker = reference_docker("operators").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "operators").unwrap();
    connection
        .batch_execute(
            r#"
            CREATE SCHEMA "Ops";
            CREATE DOMAIN "Ops"."Operand" AS integer;
            CREATE FUNCTION "Ops".combine(integer, integer) RETURNS integer
                LANGUAGE sql IMMUTABLE AS 'SELECT $1 + $2';
            CREATE FUNCTION "Ops".combine(bigint, bigint) RETURNS bigint
                LANGUAGE sql IMMUTABLE AS 'SELECT $1 + $2';
            CREATE FUNCTION "Ops".negate(integer) RETURNS integer
                LANGUAGE sql IMMUTABLE AS 'SELECT -$1';
            CREATE FUNCTION "Ops".arrays("Ops"."Operand"[], "Ops"."Operand"[]) RETURNS boolean
                LANGUAGE sql IMMUTABLE AS 'SELECT $1 = $2';
            CREATE OPERATOR "Ops".## (
                LEFTARG = integer, RIGHTARG = integer, FUNCTION = "Ops".combine,
                COMMUTATOR = OPERATOR("Ops".#@)
            );
            CREATE OPERATOR "Ops".!! (RIGHTARG = integer, FUNCTION = "Ops".negate);
            CREATE OPERATOR "Ops".=== (
                LEFTARG = "Ops"."Operand"[], RIGHTARG = "Ops"."Operand"[],
                FUNCTION = "Ops".arrays
            );
            "#,
        )
        .unwrap();
    let database: PgDieselDatabase = PgDieselDatabaseBuilder::default()
        .connection(&mut connection)
        .catalog("operators")
        .schema("Ops")
        .try_into()
        .unwrap();
    let find = |name| {
        database
            .operators()
            .unwrap()
            .find(|operator| {
                let target = operator.target_name();
                target.schema() == Some("Ops") && target.name() == name
            })
            .unwrap()
    };
    let integer = TargetName::new("int4", true).with_schema("pg_catalog", true);
    let binary = find("##");
    assert_eq!(
        binary.target_name(),
        TargetName::new("##", true).with_schema("Ops", true)
    );
    assert_eq!(binary.kind(), OperatorKind::Binary);
    assert_eq!(binary.left_type(&database).as_ref(), Some(&integer));
    assert_eq!(binary.right_type(&database).as_ref(), Some(&integer));
    assert_eq!(binary.result_type(&database).as_ref(), Some(&integer));
    let function = binary.implementation(&database).unwrap();
    let expected = database
        .functions()
        .find(|candidate| {
            candidate.schema() == "Ops"
                && candidate.model().proname == "combine"
                && candidate.model().prorettype == 23
        })
        .unwrap();
    assert!(std::ptr::eq(function, expected));

    let prefix = find("!!");
    assert_eq!(prefix.kind(), OperatorKind::Prefix);
    assert_eq!(prefix.left_type(&database), None);
    assert_eq!(prefix.right_type(&database).as_ref(), Some(&integer));
    assert_eq!(prefix.result_type(&database).as_ref(), Some(&integer));
    assert_eq!(
        prefix.implementation(&database).unwrap().model().proname,
        "negate"
    );

    let shell = find("#@");
    assert_eq!(shell.kind(), OperatorKind::Binary);
    assert_eq!(shell.left_type(&database).as_ref(), Some(&integer));
    assert_eq!(shell.right_type(&database).as_ref(), Some(&integer));
    assert_eq!(shell.result_type(&database), None);
    assert!(shell.implementation(&database).is_none());
    assert!(std::ptr::eq(binary.commutator(&database).unwrap(), shell));
    assert!(std::ptr::eq(shell.commutator(&database).unwrap(), binary));

    let array = find("===");
    let operand = TargetName::new("_Operand", true).with_schema("Ops", true);
    assert_eq!(array.left_type(&database).as_ref(), Some(&operand));
    assert_eq!(array.right_type(&database).as_ref(), Some(&operand));
    assert_eq!(
        array.result_type(&database),
        Some(TargetName::new("bool", true).with_schema("pg_catalog", true))
    );
    assert_eq!(
        array.implementation(&database).unwrap().model().proname,
        "arrays"
    );

    assert!(prefix.commutator(&database).is_none());
    assert!(prefix.negator(&database).is_none());
}

#[tokio::test]
async fn operators_preserve_links_join_flags_and_estimator_identity() {
    let docker = reference_docker("operator_links").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "operator_links").unwrap();
    connection
        .batch_execute(
            r#"
            CREATE SCHEMA "Ops";
            CREATE OPERATOR "Ops".#= (
                LEFTARG = integer, RIGHTARG = integer, FUNCTION = pg_catalog.int4eq,
                COMMUTATOR = OPERATOR("Ops".#=), NEGATOR = OPERATOR("Ops".#<>),
                HASHES, RESTRICT = pg_catalog.eqsel, JOIN = pg_catalog.eqjoinsel
            );
            CREATE OPERATOR "Ops".#<> (
                LEFTARG = integer, RIGHTARG = integer, FUNCTION = pg_catalog.int4ne,
                NEGATOR = OPERATOR("Ops".#=)
            );
            CREATE OPERATOR "Ops".~= (
                LEFTARG = integer, RIGHTARG = integer, FUNCTION = pg_catalog.int4eq,
                MERGES
            );
            CREATE OPERATOR "Ops".<# (
                LEFTARG = integer, RIGHTARG = integer, FUNCTION = pg_catalog.int4lt,
                COMMUTATOR = OPERATOR("Ops".#>)
            );
            CREATE OPERATOR "Ops".#> (
                LEFTARG = integer, RIGHTARG = integer, FUNCTION = pg_catalog.int4gt,
                COMMUTATOR = OPERATOR("Ops".<#)
            );
            "#,
        )
        .unwrap();
    let database: PgDieselDatabase = PgDieselDatabaseBuilder::default()
        .connection(&mut connection)
        .catalog("operator_links")
        .schema("Ops")
        .try_into()
        .unwrap();
    let find = |name| {
        database
            .operators()
            .unwrap()
            .find(|operator| {
                let target = operator.target_name();
                target.schema() == Some("Ops") && target.name() == name
            })
            .unwrap()
    };
    let equality = find("#=");
    let inequality = find("#<>");
    assert!(std::ptr::eq(
        equality.commutator(&database).unwrap(),
        equality
    ));
    assert!(std::ptr::eq(
        equality.negator(&database).unwrap(),
        inequality
    ));
    assert!(std::ptr::eq(
        inequality.negator(&database).unwrap(),
        equality
    ));
    let less = find("<#");
    let greater = find("#>");
    assert!(std::ptr::eq(less.commutator(&database).unwrap(), greater));
    assert!(std::ptr::eq(greater.commutator(&database).unwrap(), less));
    assert!(equality.can_hash());
    assert!(!equality.can_merge());
    let merge = find("~=");
    assert!(!merge.can_hash());
    assert!(merge.can_merge());
    assert!(!inequality.can_hash());
    assert!(!inequality.can_merge());
    for (estimator, name) in [
        (equality.restriction_estimator(&database), "eqsel"),
        (equality.join_estimator(&database), "eqjoinsel"),
    ] {
        let expected = database
            .functions()
            .find(|function| function.schema() == "pg_catalog" && function.model().proname == name)
            .unwrap();
        assert!(std::ptr::eq(estimator.unwrap(), expected));
    }
    assert!(inequality.commutator(&database).is_none());
    assert!(merge.negator(&database).is_none());
    assert!(merge.restriction_estimator(&database).is_none());
    assert!(merge.join_estimator(&database).is_none());
}

#[tokio::test]
async fn operators_with_unavailable_links_and_estimators_return_none() {
    let docker = reference_docker("operator_missing").await.unwrap();
    let mut connection: PgConnection =
        establish_connection(docker.port, "operator_missing").unwrap();
    connection
        .batch_execute(
            r#"
            CREATE SCHEMA "Ops";
            CREATE OPERATOR "Ops".#= (
                LEFTARG = integer, RIGHTARG = integer, FUNCTION = pg_catalog.int4eq
            );
            "#,
        )
        .unwrap();
    let missing_operator = pg_operator::table
        .select(pg_operator::oid)
        .order(pg_operator::oid.desc())
        .first::<u32>(&mut connection)
        .unwrap()
        .checked_add(1)
        .unwrap();
    let missing_function = pg_proc::table
        .select(pg_proc::oid)
        .order(pg_proc::oid.desc())
        .first::<u32>(&mut connection)
        .unwrap()
        .checked_add(1)
        .unwrap();
    diesel::update(pg_operator::table.filter(pg_operator::oprname.eq("#=")))
        .set((
            pg_operator::oprcom.eq(missing_operator),
            pg_operator::oprnegate.eq(missing_operator),
            pg_operator::oprrest.eq(missing_function),
            pg_operator::oprjoin.eq(missing_function),
        ))
        .execute(&mut connection)
        .unwrap();
    let database: PgDieselDatabase = PgDieselDatabaseBuilder::default()
        .connection(&mut connection)
        .catalog("operator_missing")
        .schema("Ops")
        .try_into()
        .unwrap();
    let operator = database
        .operators()
        .unwrap()
        .find(|operator| operator.target_name().name() == "#=")
        .unwrap();
    assert!(operator.commutator(&database).is_none());
    assert!(operator.negator(&database).is_none());
    assert!(operator.restriction_estimator(&database).is_none());
    assert!(operator.join_estimator(&database).is_none());
}

#[tokio::test]
async fn catalog_errors_reject_unsupported_operator_kind() {
    let docker = reference_docker("operators").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "operators").unwrap();
    connection
        .batch_execute(
            "CREATE OPERATOR public.!! (
                 FUNCTION = pg_catalog.int4abs, RIGHTARG = integer
             );
             UPDATE pg_catalog.pg_operator SET oprkind = 'r'
             WHERE oprnamespace = 'public'::regnamespace AND oprname = '!!';",
        )
        .unwrap();
    let result = PgDieselDatabase::try_from(
        PgDieselDatabaseBuilder::default()
            .connection(&mut connection)
            .catalog("operators")
            .schema("public"),
    );
    assert!(matches!(
        result,
        Err(pg_diesel::database::PgDatabaseBuildError::Diesel(
            diesel::result::Error::DeserializationError(_)
        ))
    ));
}
