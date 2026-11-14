//! Schema for `pg_catalog.pg_ts_config_map` table.

diesel::table! {
    use diesel::sql_types::{Oid, Integer};

    /// Text search configuration mappings
    pg_catalog.pg_ts_config_map (mapcfg, maptokentype, mapseqno) {
        /// OID of the `pg_ts_config` entry owning this map entry
        mapcfg -> Oid,
        /// Token type that this entry is for
        maptokentype -> Integer,
        /// Order in which to consult this entry (lower numbers first)
        mapseqno -> Integer,
        /// OID of the text search dictionary to consult
        mapdict -> Oid,
    }
}
