//! Shared Docker container setup for tests.

use diesel::Connection;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt, TestcontainersError};

/// The `PostgreSQL` major version the tests start a server of, the highest
/// one whose catalog the features compiled in.
pub const TESTED_MAJOR: u32 = catalog::MAJOR;
#[cfg(not(any(
    feature = "postgres-14",
    feature = "postgres-15",
    feature = "postgres-16",
    feature = "postgres-17",
    feature = "postgres-18"
)))]
compile_error!(
    "No postgres version feature enabled. Enable one of: postgres-14, postgres-15, postgres-16, postgres-17, postgres-18"
);

/// The catalog of the server version the tests start.
#[cfg(all(
    feature = "postgres-14",
    not(any(
        feature = "postgres-15",
        feature = "postgres-16",
        feature = "postgres-17",
        feature = "postgres-18"
    ))
))]
pub use pg_diesel::v14 as catalog;
/// The catalog of the server version the tests start.
#[cfg(all(
    feature = "postgres-15",
    not(any(
        feature = "postgres-16",
        feature = "postgres-17",
        feature = "postgres-18"
    ))
))]
pub use pg_diesel::v15 as catalog;
/// The catalog of the server version the tests start.
#[cfg(all(
    feature = "postgres-16",
    not(any(feature = "postgres-17", feature = "postgres-18"))
))]
pub use pg_diesel::v16 as catalog;
/// The catalog of the server version the tests start.
#[cfg(all(feature = "postgres-17", not(feature = "postgres-18")))]
pub use pg_diesel::v17 as catalog;
/// The catalog of the server version the tests start.
#[cfg(feature = "postgres-18")]
pub use pg_diesel::v18 as catalog;

/// The `postgres-N` feature naming [`TESTED_MAJOR`].
pub const TESTED_FEATURE: &str = match TESTED_MAJOR {
    14 => "postgres-14",
    15 => "postgres-15",
    16 => "postgres-16",
    17 => "postgres-17",
    _ => "postgres-18",
};

/// The `postgis/postgis` image tag of the `PostgreSQL` `major` version.
pub fn postgres_version_tag(major: u32) -> &'static str {
    match major {
        14 => "14-3.5",
        15 => "15-3.5",
        16 => "16-3.5",
        17 => "17-3.5",
        _ => "18-3.6",
    }
}

/// The port `PostgreSQL` listens on inside the container.
const CONTAINER_PORT: u16 = 5432;

/// A running reference database and the host port Docker published it on.
/// Dropping it stops and removes the container.
pub struct ReferenceDocker {
    /// The container, held only so that dropping it removes the container.
    _container: ContainerAsync<GenericImage>,
    /// The free host port Docker chose for the container's `PostgreSQL`.
    pub port: u16,
}

/// Starts a `PostgreSQL` [`TESTED_MAJOR`] container holding the database
/// `database_name`, published on a host port Docker picks, so concurrent
/// runs never contend for one.
///
/// # Errors
///
/// Returns the [`TestcontainersError`] when the container does not start or
/// Docker reports no published port.
pub async fn reference_docker(database_name: &str) -> Result<ReferenceDocker, TestcontainersError> {
    reference_docker_of(TESTED_MAJOR, database_name).await
}

/// Starts a `PostgreSQL` `major` container holding the database
/// `database_name`, the way [`reference_docker`] starts one of the tested
/// version.
///
/// # Errors
///
/// Returns the [`TestcontainersError`] when the container does not start or
/// Docker reports no published port.
pub async fn reference_docker_of(
    major: u32,
    database_name: &str,
) -> Result<ReferenceDocker, TestcontainersError> {
    let tag = postgres_version_tag(major);

    let container = GenericImage::new("postgis/postgis", tag)
        .with_exposed_port(CONTAINER_PORT.tcp())
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_network("bridge")
        .with_env_var("DEBUG", "1")
        .with_env_var("POSTGRES_USER", "user")
        .with_env_var("POSTGRES_PASSWORD", "password")
        .with_env_var("POSTGRES_DB", database_name)
        .start()
        .await?;
    let port = container.get_host_port_ipv4(CONTAINER_PORT).await?;
    Ok(ReferenceDocker {
        _container: container,
        port,
    })
}

/// Returns the URL of the database `database_name` on the container
/// listening on `database_port`.
pub fn database_url(database_port: u16, database_name: &str) -> String {
    format!("postgres://user:password@localhost:{database_port}/{database_name}")
}

/// Establish a connection to a postgres database.
pub fn establish_connection<C: Connection>(
    database_port: u16,
    database_name: &str,
) -> Result<C, diesel::ConnectionError> {
    let database_url = database_url(database_port, database_name);

    let mut number_of_attempts = 0;

    while let Err(e) = C::establish(&database_url) {
        eprintln!("Failed to establish connection: {e:?}");
        std::thread::sleep(std::time::Duration::from_secs(1));
        if number_of_attempts > 10 {
            eprintln!("Failed to establish connection after 10 attempts");
            std::process::exit(1);
        }
        number_of_attempts += 1;
    }

    C::establish(&database_url)
}
