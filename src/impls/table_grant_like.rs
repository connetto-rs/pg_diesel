//! Implementation of `TableGrantLike` trait for `RoleTableGrants`.

use std::sync::Arc;

use sql_traits::{
    structs::TargetName,
    traits::{
        DatabaseLike, GrantLike, Metadata, Relation, RoleLike, SequenceLike, TableGrantLike,
        TableLike, ViewLike, grant::GrantTarget,
    },
};
use sqlparser::ast::{Action, Grantee};

use crate::{PgDieselDatabase, model_metadata::PgTable, models::RoleTableGrants};

/// Metadata for `RoleTableGrants`.
#[derive(Debug, Clone)]
pub struct RoleTableGrantsMetadata {
    /// The parsed privilege action.
    pub privilege: Option<Action>,
    /// The parsed grantee.
    pub grantee: Option<Grantee>,
    /// The table this grant applies to.
    pub table: Option<Arc<PgTable>>,
}

impl RoleTableGrantsMetadata {
    /// Creates a new `RoleTableGrantsMetadata`.
    #[must_use]
    pub fn new(
        privilege: Option<Action>,
        grantee: Option<Grantee>,
        table: Option<Arc<PgTable>>,
    ) -> Self {
        Self {
            privilege,
            grantee,
            table,
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
}

impl Metadata for RoleTableGrants {
    type Meta = RoleTableGrantsMetadata;
}

/// Returns whether a grant naming `schema` and `name` matches a relation
/// recorded under `relation_schema` and `relation_name`.
fn names_relation(
    schema: Option<&str>,
    name: Option<&str>,
    relation_schema: Option<&str>,
    relation_name: &str,
) -> bool {
    schema == relation_schema && name == Some(relation_name)
}

/// Returns the grant target of a privilege `aclexplode` read from one
/// relation, the relation recorded under `schema` and `name`.
///
/// The server expands `ON ALL TABLES IN SCHEMA` into an entry per relation,
/// so a grant read from the catalog names the one relation it reached.
pub(crate) fn relation_target<'a>(
    schema: Option<&'a str>,
    name: Option<&'a str>,
) -> GrantTarget<std::option::IntoIter<TargetName<'a>>> {
    let target = name.map(|name| {
        let target = TargetName::new(name, true);
        match schema {
            Some(schema) => target.with_schema(schema, true),
            None => target,
        }
    });
    GrantTarget::Tables(target.into_iter())
}

impl GrantLike for RoleTableGrants {
    type DB = PgDieselDatabase;

    fn privileges<'db>(&'db self, database: &'db Self::DB) -> impl Iterator<Item = &'db Action>
    where
        Self: 'db,
    {
        database
            .table_grant_metadata(self)
            .and_then(RoleTableGrantsMetadata::privilege)
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
            .table_grant_metadata(self)
            .and_then(RoleTableGrantsMetadata::grantee)
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

impl TableGrantLike for RoleTableGrants {
    fn tables<'a>(
        &'a self,
        database: &'a Self::DB,
    ) -> impl Iterator<Item = &'a <Self::DB as DatabaseLike>::Table> {
        database.tables().filter(move |table| {
            names_relation(
                self.table_schema.as_deref(),
                self.table_name.as_deref(),
                table.table_schema(),
                table.table_name(),
            )
        })
    }

    fn relations<'a>(
        &'a self,
        database: &'a Self::DB,
    ) -> impl Iterator<Item = Relation<'a, Self::DB>> {
        database
            .tables()
            .filter(move |table| {
                names_relation(
                    self.table_schema.as_deref(),
                    self.table_name.as_deref(),
                    table.table_schema(),
                    table.table_name(),
                )
            })
            .map(Relation::Table)
            .chain(
                database
                    .foreign_tables()
                    .filter(move |table| {
                        names_relation(
                            self.table_schema.as_deref(),
                            self.table_name.as_deref(),
                            table.table_schema(),
                            table.table_name(),
                        )
                    })
                    .map(Relation::ForeignTable),
            )
            .chain(
                database
                    .views()
                    .filter(move |view| {
                        names_relation(
                            self.table_schema.as_deref(),
                            self.table_name.as_deref(),
                            view.view_schema(),
                            view.view_name(),
                        )
                    })
                    .map(Relation::View),
            )
            .chain(
                database
                    .materialized_views()
                    .filter(move |view| {
                        names_relation(
                            self.table_schema.as_deref(),
                            self.table_name.as_deref(),
                            view.view_schema(),
                            view.view_name(),
                        )
                    })
                    .map(Relation::MaterializedView),
            )
            .chain(
                database
                    .sequences()
                    .filter(move |sequence| {
                        names_relation(
                            self.table_schema.as_deref(),
                            self.table_name.as_deref(),
                            sequence.sequence_schema(),
                            sequence.sequence_name(),
                        )
                    })
                    .map(Relation::Sequence),
            )
    }

    fn applies_to_table(
        &self,
        table: &<Self::DB as DatabaseLike>::Table,
        _database: &Self::DB,
    ) -> bool {
        names_relation(
            self.table_schema.as_deref(),
            self.table_name.as_deref(),
            table.table_schema(),
            table.table_name(),
        )
    }
}

/// Returns the action `PostgreSQL`'s spelling of a relation or column
/// privilege names, or `None` for a privilege the AST has no action for.
#[must_use]
pub fn string_to_action(privilege: &str) -> Option<Action> {
    Some(match privilege.to_uppercase().as_str() {
        "SELECT" => Action::Select { columns: None },
        "INSERT" => Action::Insert { columns: None },
        "UPDATE" => Action::Update { columns: None },
        "DELETE" => Action::Delete,
        "TRUNCATE" => Action::Truncate,
        "REFERENCES" => Action::References { columns: None },
        "TRIGGER" => Action::Trigger,
        "USAGE" => Action::Usage,
        "MAINTAIN" => Action::Maintain,
        _ => return None,
    })
}
