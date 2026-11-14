//! Filtering `information_schema.table_constraints` by [`ConstraintType`],
//! which writes each type in the spelling the server reports.

mod test_utils;

use diesel::{
    ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, connection::SimpleConnection, dsl::sql,
    sql_types::Text,
};
use pg_diesel::models::ConstraintType;
use test_utils::{COMPILED_MAJORS, establish_connection, reference_docker};

/// One constraint of every type, each under a name the test looks up.
const FIXTURE: &str = "
    CREATE SCHEMA app;
    CREATE TABLE app.parents (
        id INTEGER CONSTRAINT parents_pkey PRIMARY KEY,
        code TEXT CONSTRAINT parents_code_key UNIQUE
    );
    CREATE TABLE app.children (
        parent_id INTEGER CONSTRAINT children_parent_fkey REFERENCES app.parents (id),
        age INTEGER CONSTRAINT children_age_check CHECK (age >= 0)
    );
";

/// Each constraint type with the one fixture constraint of that type.
const CONSTRAINTS: [(ConstraintType, &str); 4] = [
    (ConstraintType::PrimaryKey, "parents_pkey"),
    (ConstraintType::Unique, "parents_code_key"),
    (ConstraintType::ForeignKey, "children_parent_fkey"),
    (ConstraintType::Check, "children_age_check"),
];

/// Loads the fixture constraints of type `kind` through the
/// `table_constraints` table of the catalog of `major`.
fn fixture_constraints_of_type(
    major: u32,
    kind: &ConstraintType,
    conn: &mut PgConnection,
) -> Vec<String> {
    let names = CONSTRAINTS.map(|(_, name)| name);
    macro_rules! load_from {
        ($catalog:ident) => {{
            use pg_diesel::$catalog::schema::information_schema::table_constraints::table_constraints;
            table_constraints::table
                .filter(table_constraints::constraint_schema.eq("app"))
                .filter(table_constraints::constraint_name.eq_any(names))
                .filter(table_constraints::constraint_type.eq(kind.clone()))
                .select(table_constraints::constraint_name)
                .load(conn)
                .expect("the constraints load")
        }};
    }
    match major {
        #[cfg(feature = "postgres-14")]
        14 => load_from!(v14),
        #[cfg(feature = "postgres-15")]
        15 => load_from!(v15),
        #[cfg(feature = "postgres-16")]
        16 => load_from!(v16),
        #[cfg(feature = "postgres-17")]
        17 => load_from!(v17),
        #[cfg(feature = "postgres-18")]
        18 => load_from!(v18),
        _ => unreachable!("PostgreSQL {major} is not a compiled major"),
    }
}

#[tokio::test]
async fn table_constraints_filter_by_each_constraint_type() {
    for &major in COMPILED_MAJORS {
        let database_name = "constraint_type";
        let docker = reference_docker(major, database_name)
            .await
            .expect("the database starts");
        let mut conn: PgConnection =
            establish_connection(docker.port, database_name).expect("the connection opens");
        conn.batch_execute(FIXTURE).expect("the fixture applies");

        for (kind, expected) in CONSTRAINTS {
            assert_eq!(
                fixture_constraints_of_type(major, &kind, &mut conn),
                [expected],
                "constraints of type {kind} on PostgreSQL {major}"
            );
        }

        let unknown =
            diesel::select(sql::<Text>("'EXCLUDE'")).get_result::<ConstraintType>(&mut conn);
        assert!(
            unknown.is_err(),
            "a spelling outside the four types is refused on PostgreSQL {major}, got {unknown:?}"
        );
    }
}
