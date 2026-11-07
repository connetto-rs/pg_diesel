//! Implementation of `ColumnGrantLike` trait for `RoleColumnGrants`.

use std::sync::Arc;

use sql_traits::{
    errors::LookupError,
    structs::TargetName,
    traits::{
        ColumnGrantLike, DatabaseLike, GrantLike, Metadata, Relation, RoleLike, TableLike,
        ViewLike, grant::GrantTarget,
    },
};
use sqlparser::ast::{Action, Grantee};

use crate::{
    PgDieselDatabase,
    impls::table_grant_like::relation_target,
    model_metadata::{PgColumn, PgTable},
    models::RoleColumnGrants,
};

/// Metadata for `RoleColumnGrants`.
#[derive(Debug, Clone)]
pub struct RoleColumnGrantsMetadata {
    /// The parsed privilege action.
    pub privilege: Option<Action>,
    /// The parsed grantee.
    pub grantee: Option<Grantee>,
    /// The table this grant applies to.
    pub table: Option<Arc<PgTable>>,
    /// The column this grant applies to.
    pub column: Option<Arc<PgColumn>>,
}

impl RoleColumnGrantsMetadata {
    /// Creates a new `RoleColumnGrantsMetadata`.
    #[must_use]
    pub fn new(
        privilege: Option<Action>,
        grantee: Option<Grantee>,
        table: Option<Arc<PgTable>>,
        column: Option<Arc<PgColumn>>,
    ) -> Self {
        Self {
            privilege,
            grantee,
            table,
            column,
        }
    }

    /// Returns the privilege action.
    #[must_use]
    pub fn privilege(&self) -> Option<&Action> {
        self.privilege.as_ref()
    }

    /// Returns the grantee.
    #[must_use]
    pub fn grantee(&self) -> Option<&Grantee> {
        self.grantee.as_ref()
    }

    /// Returns the table.
    #[must_use]
    pub fn table(&self) -> Option<&PgTable> {
        self.table.as_deref()
    }

    /// Returns the column.
    #[must_use]
    pub fn column(&self) -> Option<&PgColumn> {
        self.column.as_deref()
    }
}

impl Metadata for RoleColumnGrants {
    type Meta = RoleColumnGrantsMetadata;
}

impl GrantLike for RoleColumnGrants {
    type DB = PgDieselDatabase;

    fn privileges<'db>(&'db self, database: &'db Self::DB) -> impl Iterator<Item = &'db Action>
    where
        Self: 'db,
    {
        database
            .column_grant_metadata(self)
            .and_then(RoleColumnGrantsMetadata::privilege)
            .into_iter()
    }

    fn is_all_privileges(&self) -> bool {
        self.privilege_type.as_deref().is_some_and(|p| {
            p.eq_ignore_ascii_case("ALL") || p.eq_ignore_ascii_case("ALL PRIVILEGES")
        })
    }

    fn grantees<'db>(&'db self, database: &'db Self::DB) -> impl Iterator<Item = &'db Grantee>
    where
        Self: 'db,
    {
        database
            .column_grant_metadata(self)
            .and_then(RoleColumnGrantsMetadata::grantee)
            .into_iter()
    }

    fn applies_to_public(&self) -> bool {
        self.grantee
            .as_deref()
            .is_some_and(|grantee| grantee.eq_ignore_ascii_case("PUBLIC"))
    }

    fn target(&self) -> GrantTarget<impl Iterator<Item = TargetName<'_>>> {
        relation_target(self.table_schema.as_deref(), self.table_name.as_deref())
    }

    fn with_grant_option(&self) -> bool {
        self.is_grantable
            .as_deref()
            .is_some_and(|g| g.eq_ignore_ascii_case("YES"))
    }

    fn granted_by<'a>(
        &'a self,
        database: &'a Self::DB,
    ) -> Option<&'a <Self::DB as DatabaseLike>::Role> {
        let grantor_name = self.grantor.as_deref()?;
        database.roles().find(|r| r.name() == grantor_name)
    }

    fn applies_to_role(&self, role: &<Self::DB as DatabaseLike>::Role) -> bool {
        self.applies_to_public() || self.grantee.as_deref().is_some_and(|g| g == role.name())
    }
}

impl ColumnGrantLike for RoleColumnGrants {
    fn columns<'a>(
        &'a self,
        table: &'a <Self::DB as DatabaseLike>::Table,
        database: &'a Self::DB,
    ) -> Result<impl Iterator<Item = &'a <Self::DB as DatabaseLike>::Column>, LookupError> {
        Ok(table
            .columns(database)?
            .filter(move |column| self.column_name.as_deref() == Some(column.column_name())))
    }

    fn table<'a>(
        &'a self,
        database: &'a Self::DB,
    ) -> Option<&'a <Self::DB as DatabaseLike>::Table> {
        database
            .column_grant_metadata(self)
            .and_then(RoleColumnGrantsMetadata::table)
    }

    fn relation<'a>(&'a self, database: &'a Self::DB) -> Option<Relation<'a, Self::DB>> {
        let names = |schema: Option<&str>, name: &str| {
            self.table_schema.as_deref() == schema && self.table_name.as_deref() == Some(name)
        };
        if let Some(table) = database
            .tables()
            .find(|t| names(t.table_schema(), t.table_name()))
        {
            return Some(Relation::Table(table));
        }
        if let Some(table) = database
            .foreign_tables()
            .find(|t| names(t.table_schema(), t.table_name()))
        {
            return Some(Relation::ForeignTable(table));
        }
        if let Some(view) = database
            .views()
            .find(|v| names(v.view_schema(), v.view_name()))
        {
            return Some(Relation::View(view));
        }
        database
            .materialized_views()
            .find(|v| names(v.view_schema(), v.view_name()))
            .map(Relation::MaterializedView)
    }
}
