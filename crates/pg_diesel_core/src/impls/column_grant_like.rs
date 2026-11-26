//! Implementation of `ColumnGrantLike` trait for `PgColumnGrant`.

use std::sync::Arc;

use sql_traits::{
    errors::LookupError,
    structs::TargetName,
    traits::{
        ColumnGrantLike, ColumnLike, DatabaseLike, GrantLike, Metadata, Relation, TableLike,
        ViewLike, grant::GrantTarget,
    },
};
use sqlparser::ast::{Action, Grantee, GranteesType};

use crate::{
    PgDieselDatabase,
    impls::table_grant_like::relation_target,
    model_metadata::{PgColumnGrant, PgTable},
};

impl Metadata for PgColumnGrant {
    /// The table the column belongs to, `None` for a view's column.
    type Meta = Option<Arc<PgTable>>;
}

impl GrantLike for PgColumnGrant {
    type DB = PgDieselDatabase;

    fn privileges<'db>(&'db self, _database: &'db Self::DB) -> impl Iterator<Item = &'db Action>
    where
        Self: 'db,
    {
        std::iter::once(self.privilege())
    }

    fn is_all_privileges(&self) -> bool {
        false
    }

    fn grantees<'db>(&'db self, _database: &'db Self::DB) -> impl Iterator<Item = &'db Grantee>
    where
        Self: 'db,
    {
        std::iter::once(self.grantee())
    }

    fn applies_to_public(&self) -> bool {
        self.grantee().grantee_type == GranteesType::Public
    }

    fn target(&self) -> GrantTarget<impl Iterator<Item = TargetName<'_>>> {
        relation_target(self.schema_name(), Some(self.relation_name()))
    }

    fn with_grant_option(&self) -> bool {
        self.grantable()
    }

    fn granted_by<'a>(&'a self, _database: &'a Self::DB) -> Option<&'a str> {
        Some(self.grantor())
    }

    fn applies_to_role(&self, role: &<Self::DB as DatabaseLike>::Role) -> bool {
        self.applies_to_public() || self.grantee_role() == Some(role.name())
    }
}

impl ColumnGrantLike for PgColumnGrant {
    fn columns<'a>(
        &'a self,
        table: &'a <Self::DB as DatabaseLike>::Table,
        database: &'a Self::DB,
    ) -> Result<impl Iterator<Item = &'a <Self::DB as DatabaseLike>::Column>, LookupError> {
        Ok(table
            .columns(database)?
            .filter(move |column| column.column_name() == self.column_name()))
    }

    fn column_names(&self) -> impl Iterator<Item = (&str, bool)> {
        // A stored name, quoted so that folding cannot rename it.
        std::iter::once((self.column_name(), true))
    }

    fn table<'a>(
        &'a self,
        database: &'a Self::DB,
    ) -> Option<&'a <Self::DB as DatabaseLike>::Table> {
        database
            .column_grant_metadata(self)
            .and_then(Option::as_deref)
    }

    fn relation<'a>(&'a self, database: &'a Self::DB) -> Option<Relation<'a, Self::DB>> {
        let names = |schema: Option<&str>, name: &str| {
            schema == Some(self.schema_name()) && name == self.relation_name()
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
