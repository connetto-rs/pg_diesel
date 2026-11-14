//! Submodule providing the `PgIdentFileMapping` struct representing a row of
//! the `pg_ident_file_mappings` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_ident_file_mappings` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_ident_file_mappings::pg_ident_file_mappings)]
#[cfg(not(any(feature = "postgres-15", feature = "postgres-14")))]
pub struct PgIdentFileMapping {
    /// Mapping number.
    pub map_number: Option<i32>,
    /// Name of the file containing this mapping.
    pub file_name: Option<String>,
    /// Line number of this mapping within the file.
    pub line_number: Option<i32>,
    /// Name of the map.
    pub map_name: Option<String>,
    /// System user name.
    pub sys_name: Option<String>,
    /// `PostgreSQL` user name.
    pub pg_username: Option<String>,
    /// Error message if the mapping is invalid.
    pub error: Option<String>,
}

/// Represents a row from the `pg_ident_file_mappings` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_ident_file_mappings::pg_ident_file_mappings)]
#[cfg(any(feature = "postgres-15", feature = "postgres-14"))]
pub struct PgIdentFileMapping {
    /// Line number of this mapping within the file.
    pub line_number: Option<i32>,
    /// Name of the map.
    pub map_name: Option<String>,
    /// System user name.
    pub sys_name: Option<String>,
    /// `PostgreSQL` user name.
    pub pg_username: Option<String>,
    /// Error message if the mapping is invalid.
    pub error: Option<String>,
}
