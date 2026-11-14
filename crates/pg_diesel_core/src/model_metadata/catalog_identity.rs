//! Shared stored identities and loaded functions indexed by catalog OID.

use std::{collections::HashMap, sync::Arc};

use sql_traits::structs::TargetName;

use crate::database::CatalogCache;

use super::PgFunction;

/// A stored catalog identity with a shared schema name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct CatalogIdentity {
    /// The schema name as stored in `pg_namespace`.
    pub(crate) schema: Arc<str>,
    /// The object name as stored in its catalog.
    pub(crate) name: String,
}

impl CatalogIdentity {
    /// Returns the qualified identity without identifier folding.
    pub(crate) fn target(&self) -> TargetName<'_> {
        TargetName::new(self.name.as_str(), true).with_schema(self.schema.as_ref(), true)
    }
}

/// Resolves shared identities against one catalog snapshot.
pub(crate) struct CatalogObjects<'a> {
    /// The catalog snapshot supplying type and namespace names.
    cache: &'a CatalogCache,
    /// The interned namespace names, by OID.
    namespaces: HashMap<u32, Arc<str>>,
    /// The interned type identities, by OID.
    types: HashMap<u32, Arc<CatalogIdentity>>,
    /// The exact loaded functions, by OID.
    functions: HashMap<u32, &'a Arc<PgFunction>>,
}

impl<'a> CatalogObjects<'a> {
    /// Indexes loaded functions for operator and cast resolution.
    pub(crate) fn new(cache: &'a CatalogCache, functions: &'a [Arc<PgFunction>]) -> Self {
        Self {
            cache,
            namespaces: HashMap::new(),
            types: HashMap::new(),
            functions: functions
                .iter()
                .map(|function| (function.oid(), function))
                .collect(),
        }
    }

    /// Shares a stored schema name across catalog identities.
    pub(crate) fn namespace(&mut self, oid: u32) -> Option<Arc<str>> {
        if let Some(name) = self.namespaces.get(&oid) {
            return Some(Arc::clone(name));
        }
        let name: Arc<str> = self.cache.namespace(oid)?.into();
        self.namespaces.insert(oid, Arc::clone(&name));
        Some(name)
    }

    /// Resolves a type OID without replacing its stored name with a display spelling.
    pub(crate) fn type_identity(&mut self, oid: u32) -> Option<Arc<CatalogIdentity>> {
        if let Some(identity) = self.types.get(&oid) {
            return Some(Arc::clone(identity));
        }
        let model = self.cache.pg_type(oid)?;
        let identity = Arc::new(CatalogIdentity {
            schema: self.namespace(model.typnamespace)?,
            name: model.typname.clone(),
        });
        self.types.insert(oid, Arc::clone(&identity));
        Some(identity)
    }

    /// Returns the exact loaded function referenced by an OID.
    pub(crate) fn function(&self, oid: u32) -> Option<Arc<PgFunction>> {
        self.functions
            .get(&oid)
            .map(|function| Arc::clone(function))
    }
}
