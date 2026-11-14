//! The columns of the catalog relations whose shape differs between
//! `PostgreSQL` versions that the differential helpers read, each one present
//! in every supported version, so one definition serves every server.

use pg_diesel::schema::pg_catalog::{pg_depend::pg_depend, pg_namespace::pg_namespace};

diesel::table! {
    /// The `pg_catalog.pg_attribute` columns the helpers read.
    pg_catalog.pg_attribute (attrelid, attnum) {
        /// The relation the column belongs to.
        attrelid -> Oid,
        /// The column name.
        attname -> Text,
        /// The column number.
        attnum -> SmallInt,
        /// Whether the column was dropped.
        attisdropped -> Bool,
    }
}

diesel::table! {
    /// The `pg_catalog.pg_class` columns the helpers read.
    pg_catalog.pg_class (oid) {
        /// The relation OID.
        oid -> Oid,
        /// The relation name.
        relname -> Text,
        /// The schema the relation lives in.
        relnamespace -> Oid,
    }
}

diesel::table! {
    /// The `pg_catalog.pg_constraint` columns the helpers read.
    pg_catalog.pg_constraint (oid) {
        /// The constraint OID.
        oid -> Oid,
        /// The constraint name.
        conname -> Text,
    }
}

diesel::table! {
    /// The `pg_catalog.pg_index` columns the helpers read.
    pg_catalog.pg_index (indexrelid) {
        /// The index OID.
        indexrelid -> Oid,
        /// The table the index is on.
        indrelid -> Oid,
        /// Whether the index enforces uniqueness.
        indisunique -> Bool,
        /// The indexed column numbers.
        indkey -> Array<SmallInt>,
        /// The expression tree of an expression index.
        indexprs -> Nullable<Text>,
        /// The predicate of a partial index.
        indpred -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(pg_class, pg_depend);
diesel::allow_tables_to_appear_in_same_query!(pg_class, pg_namespace);
