//! Submodule for the `pg_catalog.pg_attrdef` table schema.

diesel::table! {
    /// `pg_catalog.pg_attrdef` — catalog of column default values.
    /// Contains default expressions for table columns that have defaults.
    pg_catalog.pg_attrdef (oid) {
        /// OID of this entry (primary key).
        oid -> diesel::sql_types::Oid,
        /// OID of the table this default belongs to.
        adrelid -> diesel::sql_types::Oid,
        /// Column number that this default is for.
        adnum -> SmallInt,
        /// Default expression (in nodeToString representation).
        adbin -> Text,
    }
}

use super::pg_attribute::pg_attribute;
diesel::allow_tables_to_appear_in_same_query!(pg_attrdef, pg_attribute);

use super::pg_class::pg_class;
diesel::allow_tables_to_appear_in_same_query!(pg_attrdef, pg_class);

use super::pg_namespace::pg_namespace;
diesel::allow_tables_to_appear_in_same_query!(pg_attrdef, pg_namespace);
