//! `PostgreSQL` backend memory contexts view model.

/// Represents a row from the `pg_catalog.pg_backend_memory_contexts` view.
#[derive(diesel::Queryable, diesel::QueryableByName, diesel::Selectable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[diesel(table_name = crate::schema::pg_catalog::pg_backend_memory_contexts::pg_backend_memory_contexts)]
pub struct PgBackendMemoryContexts {
    /// Name of the memory context (part of composite primary key).
    pub name: Option<String>,
    /// Identification string of the memory context (part of composite primary
    /// key).
    pub ident: Option<String>,
    /// Name of the parent memory context.
    /// Removed in `PostgreSQL` 18 in favor of `path`.
    #[cfg(not(feature = "postgres-18"))]
    pub parent: Option<String>,
    /// Nesting level of the memory context.
    pub level: Option<i32>,
    /// Total number of bytes allocated in this context.
    pub total_bytes: Option<i64>,
    /// Total number of blocks allocated in this context.
    pub total_nblocks: Option<i64>,
    /// Number of free bytes in this context.
    pub free_bytes: Option<i64>,
    /// Number of free chunks in this context.
    pub free_chunks: Option<i64>,
    /// Number of used bytes in this context.
    pub used_bytes: Option<i64>,
    /// Path of the memory context (replaces `parent` in `PostgreSQL` 18+).
    #[cfg(feature = "postgres-18")]
    pub path: Option<String>,
    /// Type of the memory context.
    /// Added in `PostgreSQL` 18.
    #[cfg(feature = "postgres-18")]
    pub r#type: Option<String>,
}
