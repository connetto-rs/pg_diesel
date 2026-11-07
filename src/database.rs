//! The `DatabaseLike` catalog a `PostgreSQL` connection is read into.

use std::sync::Arc;

use sql_traits::structs::{GenericDB, SchemaProfile};

use crate::dialect::PostgresDialect;
use crate::model_metadata::{
    PgCheckConstraint, PgColumn, PgDataType, PgDomainEntry, PgExtensionEntry, PgForeignKey,
    PgFunction, PgIndexEntry, PgPolicy, PgRoleMembership, PgRuleEntry, PgSchema, PgSequenceEntry,
    PgTable, PgUserDefinedType, PgViewDefinition, TableMetadata, TriggerMetadata,
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
pub use catalog_cache::{CatalogCache, FIRST_NORMAL_OBJECT_ID};

mod builder;
pub use builder::{PgDatabaseBuildError, PgDieselDatabaseBuilder};

/// The kinds of schema object a database read through `pg_catalog` holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgDieselProfile;

impl SchemaProfile for PgDieselProfile {
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
    /// A catalog read holds no statement-by-statement state: the whole
    /// database is loaded in one pass and never resumed.
    type Ingestion = ();
    type MaterializedView = PgViewDefinition;
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

    fn default_ingestion(_dialect: &Self::Dialect) -> Self::Ingestion {}

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
