//! The catalog lookups a build resolves in one query instead of many.

use std::collections::HashMap;

use diesel::dsl::sql;
use diesel::sql_types::Text;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::models::{PgAttribute, PgType};

/// The first OID `PostgreSQL` hands to an object created after `initdb`.
///
/// Objects below it are built in and pinned, so the server records no
/// `pg_depend` row on one and no user-defined type lives under it.
pub const FIRST_NORMAL_OBJECT_ID: u32 = 16384;

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
    /// The subtype a range or multirange type holds, by the OID
    /// `pg_range` records the range or the multirange under.
    range_subtypes: HashMap<u32, u32>,
    /// The live attributes of every composite type's relation, by the OID
    /// `pg_type` records the relation under, in `attnum` order, each as the
    /// OID of its type, the typmod it holds and the array dimensions it
    /// declares.
    composite_attributes: HashMap<u32, Vec<(u32, i32, i32)>>,
}

impl CatalogCache {
    /// Reads every lookup table this cache answers from.
    ///
    /// # Errors
    ///
    /// * If one of the catalogs cannot be read.
    pub fn load(conn: &mut PgConnection) -> Result<Self, diesel::result::Error> {
        use crate::schema::pg_catalog::{
            pg_attribute::pg_attribute, pg_collation::pg_collation, pg_language::pg_language,
            pg_namespace::pg_namespace, pg_range::pg_range, pg_roles::pg_roles, pg_type::pg_type,
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

        // A range type and the multirange it brings both hold the subtype.
        let range_subtypes: HashMap<u32, u32> = pg_range::table
            .select((
                pg_range::rngtypid,
                pg_range::rngsubtype,
                pg_range::rngmultitypid,
            ))
            .load::<(u32, u32, u32)>(conn)?
            .into_iter()
            .flat_map(|(range, subtype, multirange)| [(range, subtype), (multirange, subtype)])
            .collect();

        // A composite type sends each live attribute of its relation, in
        // `attnum` order.
        let composite_attributes: HashMap<u32, Vec<(u32, i32, i32)>> = pg_attribute::table
            .filter(pg_attribute::attisdropped.eq(false))
            .filter(pg_attribute::attnum.gt(0))
            .filter(
                pg_attribute::attrelid.eq_any(
                    pg_type::table
                        .filter(pg_type::typtype.eq("c"))
                        .filter(pg_type::typrelid.ne(0))
                        .select(pg_type::typrelid),
                ),
            )
            .order(pg_attribute::attnum.asc())
            .select(PgAttribute::as_select())
            .load::<PgAttribute>(conn)?
            .into_iter()
            .map(|attribute| {
                #[cfg(not(any(
                    feature = "postgres-16",
                    feature = "postgres-17",
                    feature = "postgres-18"
                )))]
                let ndims = attribute.attndims;
                #[cfg(any(
                    feature = "postgres-16",
                    feature = "postgres-17",
                    feature = "postgres-18"
                ))]
                let ndims = i32::from(attribute.attndims);
                (
                    attribute.attrelid,
                    (attribute.atttypid, attribute.atttypmod, ndims),
                )
            })
            .fold(HashMap::new(), |mut attributes, (relation, field)| {
                attributes.entry(relation).or_default().push(field);
                attributes
            });

        Ok(Self {
            types,
            type_names,
            roles,
            namespaces,
            languages,
            collations,
            range_subtypes,
            composite_attributes,
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

    /// Returns the OID `pg_range` records the range or multirange type under
    /// `oid` holds as its subtype.
    #[must_use]
    pub fn range_subtype(&self, oid: u32) -> Option<u32> {
        self.range_subtypes.get(&oid).copied()
    }

    /// Returns the live attributes of the relation `pg_type` records under
    /// `oid`, in `attnum` order, each as the OID of its type, the typmod it
    /// holds and the array dimensions it declares.
    #[must_use]
    pub fn composite_attributes(&self, oid: u32) -> Option<&[(u32, i32, i32)]> {
        self.composite_attributes.get(&oid).map(Vec::as_slice)
    }
}
