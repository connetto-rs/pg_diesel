//! Submodule for the `pg_catalog.pg_namespace` table schema.

diesel::table! {
    /// `pg_catalog.pg_namespace` — system catalog containing one row per schema (namespace) in the database.
    /// Provides metadata about schema names and their owners.
    pg_catalog.pg_namespace (oid) {
        /// OID of the schema.
        oid -> Oid,
        /// Name of the schema.
        nspname -> Text,
        /// OID of the user who owns the schema.
        nspowner -> Oid,
    }
}
