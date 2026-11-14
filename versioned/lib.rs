#![doc = include_str!("../README.md")]

mod catalog;
pub mod models;
mod oid;
pub mod schema;
pub use catalog::{Catalog, MAJOR};
