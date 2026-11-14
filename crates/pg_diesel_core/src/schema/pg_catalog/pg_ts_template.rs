//! Schema for `pg_catalog.pg_ts_template` table.

diesel::table! {
    use diesel::sql_types::{Oid, Text};

    /// Text search templates
    pg_catalog.pg_ts_template (oid) {
        /// Row identifier
        oid -> Oid,
        /// Text search template name
        tmplname -> Text,
        /// The OID of the namespace that contains this template
        tmplnamespace -> Oid,
        /// OID of the template's initialization function
        tmplinit -> Oid,
        /// OID of the template's lexize function
        tmpllexize -> Oid,
    }
}
