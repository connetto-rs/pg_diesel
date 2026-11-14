//! Schema for `pg_catalog.pg_user_mappings` view.

diesel::table! {
    use diesel::sql_types::{Nullable, Oid, Text, Array};

    /// User mappings view
    pg_catalog.pg_user_mappings (umid) {
        /// OID of the user mapping
        umid -> Nullable<Oid>,
        /// OID of the foreign server
        srvid -> Nullable<Oid>,
        /// Name of the foreign server
        srvname -> Nullable<Text>,
        /// OID of the local user being mapped, or 0 if the mapping is public
        umuser -> Nullable<Oid>,
        /// Name of the local user
        usename -> Nullable<Text>,
        /// User mapping specific options
        umoptions -> Nullable<Array<Text>>,
    }
}
