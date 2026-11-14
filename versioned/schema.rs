//! Diesel table! macro definitions for `PostgreSQL` system schemas.

pub mod information_schema;
pub mod pg_catalog;
pub use pg_diesel_core::schema::public;
