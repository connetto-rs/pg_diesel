#![doc = include_str!("../README.md")]

/// The trait vocabulary this crate implements, re-exported so a consumer
/// speaks the version `pg_diesel_core` was built against.
pub use sql_traits;
/// The AST the trait readers hand back, re-exported for the same reason.
pub use sqlparser;
pub mod database;
pub mod dialect;
pub mod impls;
pub mod model_metadata;
pub mod models;
pub mod schema;
mod shared;
pub mod traits;
pub use database::PgDieselDatabase;
