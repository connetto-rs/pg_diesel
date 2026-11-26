//! Implementation of `TableGrantLike` trait for `PgTableGrant`.

use sql_traits::{
    structs::TargetName,
    traits::{
        DatabaseLike, GrantLike, Metadata, Relation, SequenceLike, TableGrantLike, TableLike,
        ViewLike, grant::GrantTarget,
    },
};
use sqlparser::ast::{Action, Grantee, GranteesType};

use crate::{PgDieselDatabase, model_metadata::PgTableGrant};

impl Metadata for PgTableGrant {
    type Meta = ();
}

/// Returns whether a grant naming `schema` and `name` matches a relation
/// recorded under `relation_schema` and `relation_name`.
fn names_relation(
    schema: &str,
    name: Option<&str>,
    relation_schema: Option<&str>,
    relation_name: &str,
) -> bool {
    relation_schema == Some(schema) && name == Some(relation_name)
}

/// Returns the grant target of a privilege `aclexplode` read from one
/// relation, the relation `name` of `schema`, or from `schema` itself when
/// `name` is `None`.
///
/// The server expands `ON ALL TABLES IN SCHEMA` into an entry per relation,
/// so a grant read from the catalog names the one relation it reached.
pub(crate) fn relation_target<'a>(
    schema: &'a str,
    name: Option<&'a str>,
) -> GrantTarget<std::option::IntoIter<TargetName<'a>>> {
    match name {
        None => GrantTarget::Schemas(Some(TargetName::new(schema, true)).into_iter()),
        Some(name) => GrantTarget::Tables(
            Some(TargetName::new(name, true).with_schema(schema, true)).into_iter(),
        ),
    }
}

impl GrantLike for PgTableGrant {
    type DB = PgDieselDatabase;

    fn privileges<'db>(&'db self, _database: &'db Self::DB) -> impl Iterator<Item = &'db Action>
    where
        Self: 'db,
    {
        self.privilege().into_iter()
    }

    fn is_all_privileges(&self) -> bool {
        self.privilege().is_none()
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
        relation_target(self.schema_name(), self.relation_name())
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

impl TableGrantLike for PgTableGrant {
    fn tables<'a>(
        &'a self,
        database: &'a Self::DB,
    ) -> impl Iterator<Item = &'a <Self::DB as DatabaseLike>::Table> {
        database.tables().filter(move |table| {
            names_relation(
                self.schema_name(),
                self.relation_name(),
                table.table_schema(),
                table.table_name(),
            )
        })
    }

    fn relations<'a>(
        &'a self,
        database: &'a Self::DB,
    ) -> impl Iterator<Item = Relation<'a, Self::DB>> {
        let names = move |relation_schema: Option<&str>, relation_name: &str| {
            names_relation(
                self.schema_name(),
                self.relation_name(),
                relation_schema,
                relation_name,
            )
        };
        database
            .tables()
            .filter(move |table| names(table.table_schema(), table.table_name()))
            .map(Relation::Table)
            .chain(
                database
                    .foreign_tables()
                    .filter(move |table| names(table.table_schema(), table.table_name()))
                    .map(Relation::ForeignTable),
            )
            .chain(
                database
                    .views()
                    .filter(move |view| names(view.view_schema(), view.view_name()))
                    .map(Relation::View),
            )
            .chain(
                database
                    .materialized_views()
                    .filter(move |view| names(view.view_schema(), view.view_name()))
                    .map(Relation::MaterializedView),
            )
            .chain(
                database
                    .sequences()
                    .filter(move |sequence| {
                        names(sequence.sequence_schema(), sequence.sequence_name())
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
            self.schema_name(),
            self.relation_name(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_privilege_the_ast_has_no_action_for_is_none() {
        assert_eq!(
            string_to_action("SELECT"),
            Some(Action::Select { columns: None })
        );
        assert_eq!(string_to_action("SOMETP"), None);
    }
}
