//! The catalog lookups a build resolves in one query instead of many.

use std::collections::HashMap;

use diesel::dsl::sql;
use diesel::sql_types::Text;
use diesel::{PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::models::PgType;

/// The catalog rows a build resolves by OID, loaded once.
///
/// `pg_proc` alone names a type per argument and a role per function, and a
/// PostgreSQL database carries thousands of functions, so resolving each of
/// those by its own query dominates the cost of building a database.
#[derive(Debug, Clone, Default)]
pub struct CatalogCache {
    /// Every type, by the OID `pg_type` records it under.
    types: HashMap<u32, PgType>,
    /// The name `format_type` resolves each type to in the reading session,
    /// by OID.
    type_names: HashMap<u32, String>,
    /// Every role name, by the OID `pg_roles` records it under.
    roles: HashMap<u32, String>,
    /// Every schema name, by the OID `pg_namespace` records it under.
    namespaces: HashMap<u32, String>,
    /// Every language name, by the OID `pg_language` records it under.
    languages: HashMap<u32, String>,
    /// Whether a collation compares deterministically, by schema and name.
    collations: HashMap<(String, String), bool>,
}

impl CatalogCache {
    /// Reads every lookup table this cache answers from.
    ///
    /// # Errors
    ///
    /// * If one of the catalogs cannot be read.
    pub fn load(conn: &mut PgConnection) -> Result<Self, diesel::result::Error> {
        use crate::schema::pg_catalog::{
            pg_collation::pg_collation, pg_language::pg_language, pg_namespace::pg_namespace,
            pg_roles::pg_roles, pg_type::pg_type,
        };

        let types = pg_type::table
            .select(PgType::as_select())
            .load::<PgType>(conn)?
            .into_iter()
            .map(|pg_type| (pg_type.oid, pg_type))
            .collect();

        // The name a type reads as is the session's own print of it, no type
        // modifier.
        let type_names = pg_type::table
            .select((pg_type::oid, sql::<Text>("format_type(oid, NULL)")))
            .load::<(u32, String)>(conn)?
            .into_iter()
            .collect();

        let roles = pg_roles::table
            .select((pg_roles::oid, pg_roles::rolname))
            .load::<(Option<u32>, Option<String>)>(conn)?
            .into_iter()
            .filter_map(|(oid, name)| Some((oid?, name?)))
            .collect();

        let namespaces: HashMap<u32, String> = pg_namespace::table
            .select((pg_namespace::oid, pg_namespace::nspname))
            .load::<(u32, String)>(conn)?
            .into_iter()
            .collect();

        let languages = pg_language::table
            .select((pg_language::oid, pg_language::lanname))
            .load::<(u32, String)>(conn)?
            .into_iter()
            .collect();

        let collations = pg_collation::table
            .select((
                pg_collation::collnamespace,
                pg_collation::collname,
                pg_collation::collisdeterministic,
            ))
            .load::<(u32, String, bool)>(conn)?
            .into_iter()
            .filter_map(|(namespace, name, deterministic)| {
                Some(((namespaces.get(&namespace)?.clone(), name), deterministic))
            })
            .collect();

        Ok(Self {
            types,
            type_names,
            roles,
            namespaces,
            languages,
            collations,
        })
    }

    /// Returns the type recorded under `oid`.
    #[must_use]
    pub fn pg_type(&self, oid: u32) -> Option<&PgType> {
        self.types.get(&oid)
    }

    /// Returns the name `format_type` resolves the type under `oid` to in
    /// the reading session.
    #[must_use]
    pub fn type_name(&self, oid: u32) -> Option<&str> {
        self.type_names.get(&oid).map(String::as_str)
    }

    /// Returns the role recorded under `oid`.
    #[must_use]
    pub fn role(&self, oid: u32) -> Option<&str> {
        self.roles.get(&oid).map(String::as_str)
    }

    /// Returns the schema recorded under `oid`.
    #[must_use]
    pub fn namespace(&self, oid: u32) -> Option<&str> {
        self.namespaces.get(&oid).map(String::as_str)
    }

    /// Returns the language recorded under `oid`.
    #[must_use]
    pub fn language(&self, oid: u32) -> Option<&str> {
        self.languages.get(&oid).map(String::as_str)
    }

    /// Returns whether the named collation compares deterministically.
    #[must_use]
    pub fn collation_is_deterministic(&self, schema: &str, name: &str) -> Option<bool> {
        self.collations
            .get(&(schema.to_owned(), name.to_owned()))
            .copied()
    }
}
