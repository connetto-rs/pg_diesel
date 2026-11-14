//! Schema for `pg_catalog.pg_ts_dict` table.

diesel::table! {
    use diesel::sql_types::{Oid, Text, Nullable};

    /// Text search dictionaries
    pg_catalog.pg_ts_dict (oid) {
        /// Row identifier
        oid -> Oid,
        /// Text search dictionary name
        dictname -> Text,
        /// The OID of the namespace that contains this dictionary
        dictnamespace -> Oid,
        /// Owner of the dictionary
        dictowner -> Oid,
        /// OID of the text search template for this dictionary
        dicttemplate -> Oid,
        /// Initialization option string for the dictionary
        dictinitoption -> Nullable<Text>,
    }
}
