#![doc = include_str!("../README.md")]

pub use pg_diesel_core::{dialect, model_metadata, sql_traits, sqlparser, traits};
pub mod database;
pub mod impls;
pub mod models;
pub mod schema;
pub use database::PgDieselDatabase;
