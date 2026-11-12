//! The catalog lookups a build resolves in one query instead of many.

use std::{collections::HashMap, sync::Arc};

use diesel::dsl::sql;
use diesel::sql_types::{Integer, Text};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, PgConnection,
    QueryDsl, RunQueryDsl, SelectableHelper,
};

use crate::models::PgType;

/// The first OID `PostgreSQL` hands to an object created after `initdb`.
///
/// Objects below it are built in and pinned, so the server records no
/// `pg_depend` row on one and no user-defined type lives under it.
pub const FIRST_NORMAL_OBJECT_ID: u32 = 16384;

/// A live composite attribute with its declared type and collation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, diesel::Queryable)]
pub struct CompositeAttribute {
    /// The stored field name.
    pub name: String,
    /// The declared type OID.
    pub type_oid: u32,
    /// The declared type modifier.
    pub typmod: i32,
    /// The declared array dimension count.
    pub dimensions: i32,
    /// Whether the field carries a `NOT NULL` constraint.
    pub not_null: bool,
    /// The effective collation OID.
    pub collation_oid: u32,
    /// The nondefault collation's schema, name and determinism.
    pub collation: Option<(String, String, bool)>,
}

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
    /// Every composite relation's live attributes in `attnum` order.
    composite_attributes: HashMap<u32, Vec<Arc<CompositeAttribute>>>,
    /// Every enum's labels in catalog sort order.
    enum_labels: HashMap<u32, Arc<[String]>>,
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
            pg_range::pg_range, pg_roles::pg_roles, pg_type::pg_type,
        };

        let types: HashMap<u32, PgType> = pg_type::table
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

        let composite_attributes = Self::load_composite_attributes(conn, &types)?;
        let enum_labels = Self::load_enum_labels(conn, &types)?;

        Ok(Self {
            types,
            type_names,
            roles,
            namespaces,
            languages,
            collations,
            range_subtypes,
            composite_attributes,
            enum_labels,
        })
    }

    /// Loads enum labels in sort order, retaining empty enum types.
    fn load_enum_labels(
        conn: &mut PgConnection,
        types: &HashMap<u32, PgType>,
    ) -> Result<HashMap<u32, Arc<[String]>>, diesel::result::Error> {
        use crate::schema::pg_catalog::pg_enum::pg_enum;

        let mut rows = pg_enum::table
            .order((pg_enum::enumtypid.asc(), pg_enum::enumsortorder.asc()))
            .select((pg_enum::enumtypid, pg_enum::enumlabel))
            .load::<(u32, String)>(conn)?
            .into_iter()
            .peekable();
        let mut enums = HashMap::new();
        while let Some((oid, label)) = rows.next() {
            let mut labels = vec![label];
            while let Some((_, label)) = rows.next_if(|(next, _)| *next == oid) {
                labels.push(label);
            }
            enums.insert(oid, Arc::from(labels));
        }
        for row in types.values().filter(|row| row.typtype == "e") {
            enums.entry(row.oid).or_insert_with(|| Arc::from([]));
        }
        Ok(enums)
    }

    /// Loads live composite declarations, retaining empty relations.
    fn load_composite_attributes(
        conn: &mut PgConnection,
        types: &HashMap<u32, PgType>,
    ) -> Result<HashMap<u32, Vec<Arc<CompositeAttribute>>>, diesel::result::Error> {
        use crate::schema::pg_catalog::{
            pg_attribute::pg_attribute, pg_collation::pg_collation, pg_namespace::pg_namespace,
            pg_type::pg_type,
        };

        let mut composite_attributes: HashMap<u32, Vec<Arc<CompositeAttribute>>> = types
            .values()
            .filter(|data_type| data_type.typtype == "c" && data_type.typrelid != 0)
            .map(|data_type| (data_type.typrelid, Vec::new()))
            .collect();
        let attributes = pg_attribute::table
            .left_join(pg_type::table.on(pg_type::oid.eq(pg_attribute::atttypid)))
            .left_join(
                pg_collation::table.on(pg_collation::oid
                    .eq(pg_attribute::attcollation)
                    .and(pg_collation::oid.ne(pg_type::typcollation))),
            )
            .left_join(pg_namespace::table.on(pg_namespace::oid.eq(pg_collation::collnamespace)))
            .filter(pg_attribute::attisdropped.eq(false))
            .filter(pg_attribute::attnum.gt(0))
            .filter(pg_attribute::attrelid.eq_any(composite_attributes.keys().copied()))
            .order(pg_attribute::attnum.asc())
            .select((
                pg_attribute::attrelid,
                (
                    pg_attribute::attname,
                    pg_attribute::atttypid,
                    pg_attribute::atttypmod,
                    sql::<Integer>("attndims::integer"),
                    pg_attribute::attnotnull,
                    pg_attribute::attcollation,
                    (
                        pg_namespace::nspname,
                        pg_collation::collname,
                        pg_collation::collisdeterministic,
                    )
                        .nullable(),
                ),
            ))
            .load::<(u32, CompositeAttribute)>(conn)?;
        for (relation, attribute) in attributes {
            composite_attributes
                .entry(relation)
                .or_default()
                .push(Arc::new(attribute));
        }
        Ok(composite_attributes)
    }

    /// Returns the type recorded under `oid`.
    #[must_use]
    pub fn pg_type(&self, oid: u32) -> Option<&PgType> {
        self.types.get(&oid)
    }

    /// Iterates every cached type row.
    pub(super) fn types(&self) -> impl ExactSizeIterator<Item = &PgType> {
        self.types.values()
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

    /// Returns a composite relation's live attributes in `attnum` order.
    #[must_use]
    pub fn composite_attributes(&self, oid: u32) -> Option<&[Arc<CompositeAttribute>]> {
        self.composite_attributes.get(&oid).map(Vec::as_slice)
    }

    /// Returns shared enum labels, including an empty slice for an empty enum.
    pub(crate) fn enum_labels(&self, oid: u32) -> Option<&Arc<[String]>> {
        self.enum_labels.get(&oid)
    }
}
