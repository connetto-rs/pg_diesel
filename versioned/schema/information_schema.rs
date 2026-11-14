//! Diesel schema definitions for the SQL standard `information_schema` views.

pub use pg_diesel_core::schema::information_schema::*;

pub mod element_types;
pub mod table_constraints;
