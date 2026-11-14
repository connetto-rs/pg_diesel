//! Schema for `pg_catalog.pg_ts_config` table.

diesel::table! {
    use diesel::sql_types::{Oid, Text};

    /// Text search configurations
    pg_catalog.pg_ts_config (oid) {
        /// Row identifier
        oid -> Oid,
        /// Text search configuration name
        cfgname -> Text,
        /// The OID of the namespace that contains this configuration
        cfgnamespace -> Oid,
        /// Owner of the configuration
        cfgowner -> Oid,
        /// The OID of the text search parser for this configuration
        cfgparser -> Oid,
    }
}
