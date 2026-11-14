//! Submodule for the `pg_catalog.pg_database` table schema.

diesel::table! {
    /// `pg_catalog.pg_database` — system catalog containing one row per database.
    /// Stores metadata about databases including encoding, locale, connection limits,
    /// and access control lists.
    pg_catalog.pg_database (oid) {
        /// OID of the database.
        oid -> Oid,
        /// Name of the database.
        datname -> Text,
        /// OID of the role that owns the database.
        datdba -> Oid,
        /// Character encoding for this database (encoding ID from `pg_encoding`).
        encoding -> Integer,
        /// Locale provider: 'c' (libc), 'i' (icu), or 'd' (database default) (`PostgreSQL` 15+).
        #[cfg(not(feature = "postgres-14"))]
        datlocprovider -> Text,
        /// `true` if this is a template database (can be cloned with CREATE DATABASE).
        datistemplate -> Bool,
        /// `true` if connections to this database are allowed.
        datallowconn -> Bool,
        /// `true` if login events are logged for this database.
        /// Added in `PostgreSQL` 17.
        #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
        dathasloginevt -> Bool,
        /// Maximum number of concurrent connections (-1 = no limit).
        datconnlimit -> Integer,
        /// All transaction IDs before this one have been replaced with a permanent ID.
        datfrozenxid -> Oid,
        /// All multixact IDs before this one have been replaced with a permanent ID.
        datminmxid -> Oid,
        /// Highest OID of any system object in this database (only in `PostgreSQL` 14).
        #[cfg(feature = "postgres-14")]
        datlastsysoid -> Oid,
        /// Default tablespace for this database.
        dattablespace -> Oid,
        /// `LC_COLLATE` setting for this database.
        datcollate -> Text,
        /// `LC_CTYPE` setting for this database.
        datctype -> Text,
        /// ICU locale ID for this database (`PostgreSQL` 15-16 only).
        /// Renamed to `datlocale` in `PostgreSQL` 17.
        #[cfg(any(feature = "postgres-15", feature = "postgres-16"))]
        daticulocale -> Nullable<Text>,
        /// Locale name if using ICU provider (`PostgreSQL` 17+).
        /// Renamed from `daticulocale` in `PostgreSQL` 17.
        #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
        datlocale -> Nullable<Text>,
        /// ICU collation rules if using ICU provider (`PostgreSQL` 16+).
        #[cfg(not(any(feature = "postgres-14", feature = "postgres-15")))]
        daticurules -> Nullable<Text>,
        /// Version of the collation.
        #[cfg(not(feature = "postgres-14"))]
        datcollversion -> Nullable<Text>,
    }
}
