//! Model structs representing rows from `PostgreSQL` system catalogs.

mod information_schema;
mod pg_catalog;
pub use information_schema::*;
pub use pg_catalog::*;
pub use pg_diesel_core::models::*;
