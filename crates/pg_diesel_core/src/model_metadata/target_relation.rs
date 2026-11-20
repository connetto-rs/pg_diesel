//! The relation a rule or trigger is defined on, as `pg_class` records it.

use sql_traits::{
    errors::LookupError,
    structs::{IdentifierCase, TargetName},
    traits::{DatabaseLike, Relation},
};

use crate::PgDieselDatabase;

/// The kind of relation a rule or trigger is defined on, by the
/// `pg_class.relkind` of its target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetRelationKind {
    /// A table, including a partitioned one.
    Table,
    /// A foreign table.
    ForeignTable,
    /// A plain view.
    View,
}

impl TargetRelationKind {
    /// Returns the kind a `pg_class.relkind` names, a table for any kind
    /// other than a view or a foreign table.
    #[must_use]
    pub fn from_relkind(relkind: &str) -> Self {
        match relkind {
            "v" => Self::View,
            "f" => Self::ForeignTable,
            _ => Self::Table,
        }
    }
}

/// The relation a rule or trigger is defined on, by the stored names and the
/// kind `pg_class` records for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetRelation {
    /// The stored name of the schema the relation lives in.
    schema: String,
    /// The stored name of the relation.
    name: String,
    /// The kind of the relation.
    kind: TargetRelationKind,
}

impl TargetRelation {
    /// Creates the target of the stored `schema` and `name`, of the kind the
    /// `pg_class.relkind` `relkind` names.
    #[must_use]
    pub fn new(schema: String, name: String, relkind: &str) -> Self {
        Self {
            schema,
            name,
            kind: TargetRelationKind::from_relkind(relkind),
        }
    }

    /// Returns the stored name of the schema the relation lives in.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the stored name of the relation.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the kind of the relation.
    #[must_use]
    pub fn kind(&self) -> TargetRelationKind {
        self.kind
    }

    /// Returns the schema-qualified name of the relation, both parts marked
    /// quoted since catalog names are stored names and folding them again
    /// would rename them.
    #[must_use]
    pub fn target_name(&self) -> TargetName<'_> {
        TargetName::new(&self.name, true).with_schema(&self.schema, true)
    }

    /// Returns the relation `database` holds under the stored name, looked
    /// up in the store the kind decides.
    ///
    /// # Errors
    ///
    /// Returns [`LookupError::RelationNotFound`] when `database` holds no
    /// relation of the kind under the name, and the lookup's error when the
    /// name denotes no single relation.
    pub fn resolve<'db>(
        &self,
        database: &'db PgDieselDatabase,
    ) -> Result<Relation<'db, PgDieselDatabase>, LookupError> {
        let target = self.target_name();
        match self.kind {
            TargetRelationKind::View => database
                .view_by_target(target, IdentifierCase::AsWritten)?
                .map(Relation::View),
            TargetRelationKind::ForeignTable => database
                .foreign_table_by_target(target, IdentifierCase::AsWritten)?
                .map(Relation::ForeignTable),
            TargetRelationKind::Table => database
                .table_by_target(target, IdentifierCase::AsWritten)?
                .map(Relation::Table),
        }
        .ok_or_else(|| LookupError::RelationNotFound {
            object_name: format!("{}.{}", self.schema, self.name),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::PostgresDialect;

    #[test]
    fn a_relation_the_database_does_not_hold_is_reported_not_found() {
        let database = PgDieselDatabase::new("catalog".to_string(), PostgresDialect).into();
        for relkind in ["v", "f", "r"] {
            let target = TargetRelation::new("app".to_string(), "relation".to_string(), relkind);
            let error = target
                .resolve(&database)
                .expect_err("the database holds no relation");
            assert!(
                matches!(error, LookupError::RelationNotFound { .. }),
                "relkind {relkind} reports the missing relation"
            );
        }
    }
}
