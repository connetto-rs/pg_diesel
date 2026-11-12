//! Borrowed conversion facts indexed by qualified stored type names.

use std::sync::Arc;

use diesel::{PgConnection, RunQueryDsl, dsl::sql, result::Error, sql_types::Oid};
use sql_traits::{
    structs::{DomainFacts, TargetName, TypeCategory, TypeFacts},
    traits::DatabaseLike,
    utils::identifier_resolution::stored_identifier,
};

use crate::{
    PgDieselDatabase,
    model_metadata::catalog_identity::{CatalogIdentity, CatalogObjects},
    models::PgType,
};

use super::CatalogCache;

/// Conversion facts sorted by schema and stored type name.
#[derive(Debug, Default)]
pub(super) struct PgTypeCatalog {
    /// Entries sharing their identities with operator and cast signatures.
    entries: Box<[PgTypeFacts]>,
    /// The reading session's temporary schema, when allocated.
    temporary_schema: Option<Arc<str>>,
}

/// A domain's flattened base and inherited type modifier.
#[derive(Debug)]
struct PgDomainFacts {
    /// The first base that is not a domain.
    base: Arc<CatalogIdentity>,
    /// The first declared type modifier along the domain chain.
    typmod: Option<i32>,
}

/// Owned identities and scalar conversion facts for one catalog row.
#[derive(Debug)]
struct PgTypeFacts {
    /// The qualified stored type name.
    identity: Arc<CatalogIdentity>,
    /// The catalog's type category.
    category: TypeCategory,
    /// Whether this type is preferred within its category.
    preferred: bool,
    /// The flattened domain base and modifier.
    domain: Option<PgDomainFacts>,
    /// The element of a type using the array subscript handler.
    array_element: Option<Arc<CatalogIdentity>>,
    /// The catalog's array type over this type.
    array_type: Option<Arc<CatalogIdentity>>,
}

impl PgTypeCatalog {
    /// Loads all type facts using shared catalog identities.
    pub(super) fn load(
        connection: &mut PgConnection,
        cache: &CatalogCache,
        objects: &mut CatalogObjects<'_>,
    ) -> Result<Self, Error> {
        let (array_handler, temporary_namespace): (u32, u32) = diesel::select((
            sql::<Oid>("'pg_catalog.array_subscript_handler'::regproc::oid"),
            sql::<Oid>("pg_my_temp_schema()"),
        ))
        .get_result(connection)?;
        let mut entries = Vec::with_capacity(cache.types().len());
        for row in cache.types() {
            let [category] = row.typcategory.as_bytes() else {
                return Err(Error::DeserializationError("invalid type category".into()));
            };
            entries.push(PgTypeFacts {
                identity: objects.type_identity(row.oid).ok_or(Error::NotFound)?,
                category: TypeCategory::new(*category),
                preferred: row.typispreferred,
                domain: (row.typtype == "d")
                    .then(|| PgDomainFacts::load(row, cache, objects))
                    .transpose()?,
                array_element: (row.typelem != 0 && row.typsubscript == array_handler)
                    .then(|| objects.type_identity(row.typelem).ok_or(Error::NotFound))
                    .transpose()?,
                array_type: (row.typarray != 0)
                    .then(|| objects.type_identity(row.typarray).ok_or(Error::NotFound))
                    .transpose()?,
            });
        }
        entries.sort_unstable_by(|left, right| left.identity.cmp(&right.identity));
        Ok(Self {
            entries: entries.into_boxed_slice(),
            temporary_schema: objects.namespace(temporary_namespace),
        })
    }

    /// Resolves qualified names or the database's effective type search path.
    pub(super) fn find(
        &self,
        database: &PgDieselDatabase,
        name: &TargetName<'_>,
    ) -> Option<TypeFacts<'_>> {
        let written = stored_identifier(name.name(), name.name_is_quoted());
        let qualified = |schema: &str, quoted| {
            let schema = stored_identifier(schema, quoted);
            let index = self
                .entries
                .binary_search_by(|entry| {
                    entry
                        .identity
                        .schema
                        .as_ref()
                        .cmp(schema.as_ref())
                        .then_with(|| entry.identity.name.as_str().cmp(written.as_ref()))
                })
                .ok()?;
            Some(self.entries[index].borrowed())
        };
        if let Some(schema) = name.schema() {
            return qualified(schema, name.schema_is_quoted());
        }
        let mut catalog_explicit = false;
        let mut temporary_explicit = false;
        for (schema, quoted) in database.search_path() {
            let schema = stored_identifier(schema, quoted);
            catalog_explicit |= schema == "pg_catalog";
            temporary_explicit |= self.temporary_schema.as_deref() == Some(schema.as_ref());
        }
        if !temporary_explicit
            && let Some(facts) = self
                .temporary_schema
                .as_deref()
                .and_then(|schema| qualified(schema, true))
        {
            return Some(facts);
        }
        if !catalog_explicit && let Some(facts) = qualified("pg_catalog", true) {
            return Some(facts);
        }
        database
            .search_path()
            .find_map(|(schema, quoted)| qualified(schema, quoted))
    }
}

impl PgDomainFacts {
    /// Follows domain bases with a bound that rejects cycles.
    fn load<'a>(
        mut row: &'a PgType,
        cache: &'a CatalogCache,
        objects: &mut CatalogObjects<'_>,
    ) -> Result<Self, Error> {
        let mut typmod = None;
        for _ in 0..cache.types().len() {
            if row.typtype != "d" {
                return Ok(Self {
                    base: objects.type_identity(row.oid).ok_or(Error::NotFound)?,
                    typmod,
                });
            }
            if typmod.is_none() && row.typtypmod != -1 {
                typmod = Some(row.typtypmod);
            }
            row = cache.pg_type(row.typbasetype).ok_or(Error::NotFound)?;
        }
        Err(Error::DeserializationError(
            "cyclic domain base types".into(),
        ))
    }
}

impl PgTypeFacts {
    /// Borrows all names from the loaded catalog without allocating.
    fn borrowed(&self) -> TypeFacts<'_> {
        let mut facts = TypeFacts::new(self.category, self.preferred);
        if let Some(domain) = &self.domain {
            facts = facts.with_domain(DomainFacts::new(domain.base.target(), domain.typmod));
        }
        if let Some(element) = &self.array_element {
            facts = facts.with_array_element(element.target());
        }
        if let Some(array) = &self.array_type {
            facts = facts.with_array_type(array.target());
        }
        facts
    }
}
