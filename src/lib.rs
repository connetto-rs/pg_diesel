#![doc = include_str!("../README.md")]

pub use pg_diesel_core::{
    dialect, impls, model_metadata, models, schema, sql_traits, sqlparser, traits,
};
pub mod database;
pub use database::PgDieselDatabase;
/// The `PostgreSQL` 14 catalog, the version-independent tables and models
/// beside the ones `PostgreSQL` 14 shapes its own way.
#[cfg(feature = "postgres-14")]
pub use pg_diesel_pg14 as v14;
/// The `PostgreSQL` 15 catalog, the version-independent tables and models
/// beside the ones `PostgreSQL` 15 shapes its own way.
#[cfg(feature = "postgres-15")]
pub use pg_diesel_pg15 as v15;
/// The `PostgreSQL` 16 catalog, the version-independent tables and models
/// beside the ones `PostgreSQL` 16 shapes its own way.
#[cfg(feature = "postgres-16")]
pub use pg_diesel_pg16 as v16;
/// The `PostgreSQL` 17 catalog, the version-independent tables and models
/// beside the ones `PostgreSQL` 17 shapes its own way.
#[cfg(feature = "postgres-17")]
pub use pg_diesel_pg17 as v17;
/// The `PostgreSQL` 18 catalog, the version-independent tables and models
/// beside the ones `PostgreSQL` 18 shapes its own way.
#[cfg(feature = "postgres-18")]
pub use pg_diesel_pg18 as v18;
