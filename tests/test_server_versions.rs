//! One build reading the catalogs of servers of different `PostgreSQL`
//! versions, each through the catalog of its own version.

mod test_utils;

use diesel::PgConnection;
use pg_diesel::{
    PgDieselDatabase,
    database::{PgDatabaseBuildError, ServerVersion, VersionCatalog},
    model_metadata::MembershipGrant,
};
use test_utils::{docker_helpers::TESTED_MAJOR, establish_connection, reference_docker};

/// The schema both servers hold, a column whose `NOT NULL` only
/// `PostgreSQL` 18 records as a named constraint.
#[cfg(all(feature = "postgres-17", feature = "postgres-18"))]
const FIXTURE: &str = "
    CREATE SCHEMA app;
    CREATE TABLE app.notes (id INTEGER NOT NULL);
";

/// Reads the `app` schema of a fresh `PostgreSQL` `major` server through
/// the builder every consumer uses, returning the name of the not-null
/// constraint on `app.notes.id`.
#[cfg(all(feature = "postgres-17", feature = "postgres-18"))]
async fn not_null_constraint_name(major: u32) -> Option<String> {
    use crate::test_utils::docker_helpers::reference_docker_of;
    use diesel::connection::SimpleConnection;
    use pg_diesel::{
        database::PgDieselDatabaseBuilder,
        sql_traits::traits::{ColumnLike, DatabaseLike, TableLike},
    };

    let database_name = format!("server_versions_{major}");
    let docker = reference_docker_of(major, &database_name)
        .await
        .expect("the database starts");
    let mut conn: PgConnection =
        establish_connection(docker.port, &database_name).expect("the connection opens");
    conn.batch_execute(FIXTURE).expect("the fixture applies");
    let database: PgDieselDatabase = PgDieselDatabaseBuilder::default()
        .connection(&mut conn)
        .catalog(&database_name)
        .schema("app")
        .try_into()
        .expect("the app schema loads");
    let notes = database
        .tables()
        .find(|table| table.table_name() == "notes")
        .expect("the table is loaded");
    let id = notes
        .columns(&database)
        .expect("in database")
        .next()
        .expect("the column is loaded");
    ColumnLike::not_null_constraint_name(id, &database)
        .expect("in database")
        .map(std::borrow::Cow::into_owned)
}

/// One process reads a `PostgreSQL` 17 and a `PostgreSQL` 18 catalog, the
/// 18 one with the named not-null constraint only that version records.
#[cfg(all(feature = "postgres-17", feature = "postgres-18"))]
#[tokio::test]
async fn one_build_reads_postgres_17_and_18() {
    assert_eq!(not_null_constraint_name(17).await, None);
    assert_eq!(
        not_null_constraint_name(18).await.as_deref(),
        Some("notes_id_not_null")
    );
}

/// A catalog compiled for no server version.
#[derive(Default)]
struct NoVersions;

impl VersionCatalog for NoVersions {
    fn majors(&self) -> &'static [u32] {
        &[]
    }

    fn membership_grants(
        &self,
        _version: ServerVersion,
        _conn: &mut PgConnection,
    ) -> Result<Vec<MembershipGrant>, diesel::result::Error> {
        panic!("the builder reads no catalog of a version it refused")
    }
}

/// A server whose version no catalog was compiled for is refused, naming
/// its version, before any catalog query.
#[tokio::test]
async fn an_uncompiled_server_version_is_refused() {
    let database_name = "uncompiled_server_version";
    let docker = reference_docker(database_name)
        .await
        .expect("the database starts");
    let mut conn: PgConnection =
        establish_connection(docker.port, database_name).expect("the connection opens");
    let result: Result<PgDieselDatabase, _> =
        pg_diesel_core::database::PgDieselDatabaseBuilder::<NoVersions>::default()
            .connection(&mut conn)
            .catalog(database_name)
            .schema("public")
            .try_into();
    assert!(matches!(
        result,
        Err(PgDatabaseBuildError::UnsupportedServerVersion { server, compiled: [] })
            if server == TESTED_MAJOR
    ));
}
