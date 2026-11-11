//! Live operator signatures and exact implementing functions.

use pg_diesel::sql_traits::{
    structs::{TargetName, generic_db::GenericDBBuilder},
    traits::{DatabaseLike, OperatorKind, OperatorLike},
};
use pg_diesel::{PgDieselDatabase, dialect::PostgresDialect};

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

    let unavailable: PgDieselDatabase =
        GenericDBBuilder::new("unloaded".to_owned(), PostgresDialect).into();
    assert!(unavailable.operators().is_none());
}
