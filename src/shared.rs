//! Shared-column views of the catalog relations whose columns differ between
//! `PostgreSQL` versions, so the loaders that read only columns every
//! supported version has run unchanged on any server.

pub(crate) mod models;
pub(crate) mod schema;
