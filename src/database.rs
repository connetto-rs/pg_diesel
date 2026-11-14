//! The `DatabaseLike` catalog a `PostgreSQL` connection is read into.

use std::sync::Arc;

use sql_traits::structs::{GenericDB, SchemaProfile};

use crate::dialect::PostgresDialect;
use crate::model_metadata::{
    PgCastEntry, PgCheckConstraint, PgColumn, PgDataType, PgDomainEntry, PgExtensionEntry,
    PgForeignKey, PgFunction, PgIndexEntry, PgOperatorEntry, PgPolicy, PgRoleMembership,
    PgRuleEntry, PgSchema, PgSequenceEntry, PgTable, PgUserDefinedType, PgViewDefinition,
    TableMetadata, TriggerMetadata,
};
use crate::models::{PgRole, RoleColumnGrants, RoleTableGrants};
mod key_column_usage_metadata;
pub use key_column_usage_metadata::KeyColumnUsageMetadata;
mod pg_proc_metadata;
pub use pg_proc_metadata::{ExecuteHolder, PgProcMetadata};
mod acl_schema_grant;
pub use acl_schema_grant::AclSchemaGrant;
mod acl_table_grant;
pub use acl_table_grant::AclTableGrant;
mod catalog_cache;
pub use catalog_cache::{CatalogCache, CompositeAttribute, FIRST_NORMAL_OBJECT_ID};
mod server_version;
mod type_facts;
pub use server_version::ServerVersion;

mod builder;
pub use builder::{PgDatabaseBuildError, PgDieselDatabaseBuilder};

/// Loaded operator, cast and type catalogs, including an explicitly loaded empty catalog.
#[derive(Debug, Default)]
pub struct PgCatalogMetadata {
    /// The operator signatures loaded from `pg_operator`.
    operators: Box<[PgOperatorEntry]>,
    /// The casts loaded from `pg_cast`.
    casts: Box<[PgCastEntry]>,
    /// The conversion facts loaded from `pg_type`.
    types: type_facts::PgTypeCatalog,
}

/// The kinds of schema object a database read through `pg_catalog` holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgDieselProfile;

impl SchemaProfile for PgDieselProfile {
    type Cast = PgCastEntry;
    type CheckConstraint = PgCheckConstraint;
    type Column = PgColumn;
    type ColumnGrant = RoleColumnGrants;
    type DataType = PgDataType;
    type Dialect = PostgresDialect;
    type Domain = PgDomainEntry;
    type Extension = PgExtensionEntry;
    type ForeignKey = PgForeignKey;
    type Function = PgFunction;
    type Index = PgIndexEntry;
    type Ingestion = Option<Arc<PgCatalogMetadata>>;
    type MaterializedView = PgViewDefinition;
    type Operator = PgOperatorEntry;
    type Policy = PgPolicy;
    type Role = PgRole;
    type RoleMembership = PgRoleMembership;
    type Rule = PgRuleEntry;
    type Schema = PgSchema;
    type Sequence = PgSequenceEntry;
    type Table = PgTable;
    type TableGrant = RoleTableGrants;
    type Trigger = TriggerMetadata;
    type UniqueIndex = PgIndexEntry;
    type UserDefinedType = PgUserDefinedType;
    type View = PgViewDefinition;

    fn default_ingestion(_dialect: &Self::Dialect) -> Self::Ingestion {
        None
    }

    fn operators<'a>(
        _dialect: &'a Self::Dialect,
        ingestion: &'a Self::Ingestion,
    ) -> Option<impl Iterator<Item = &'a Self::Operator>>
    where
        Self::Operator: 'a,
    {
        ingestion.as_ref().map(|catalog| catalog.operators.iter())
    }

    fn casts<'a>(
        _dialect: &'a Self::Dialect,
        ingestion: &'a Self::Ingestion,
    ) -> Option<impl Iterator<Item = &'a Self::Cast>>
    where
        Self::Cast: 'a,
    {
        ingestion.as_ref().map(|catalog| catalog.casts.iter())
    }

    fn type_facts<'a>(
        database: &'a GenericDB<Self>,
        ingestion: &'a Self::Ingestion,
        type_name: sql_traits::structs::TargetName<'_>,
    ) -> Option<sql_traits::structs::TypeFacts<'a>> {
        ingestion.as_ref()?.types.find(database, &type_name)
    }

    /// The schemas a fresh read of this profile starts with, none, since the
    /// catalog read adds every schema `pg_namespace` holds.
    fn initial_schemas(
        _dialect: &Self::Dialect,
    ) -> impl IntoIterator<Item = (Arc<Self::Schema>, ())> {
        core::iter::empty()
    }

    fn is_foreign_table(metadata: &TableMetadata) -> bool {
        metadata.foreign_source().is_some()
    }
}

/// Type alias representing a `PostgreSQL` database with loaded metadata.
pub type PgDieselDatabase = GenericDB<PgDieselProfile>;
