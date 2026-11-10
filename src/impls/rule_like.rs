//! Implementation of [`RuleLike`] for [`PgRuleEntry`].

use sql_traits::{
    errors::LookupError,
    structs::{IdentifierCase, TargetName},
    traits::{DatabaseLike, Metadata, Relation, RuleLike},
};
use sqlparser::ast::{CreateRuleAction, CreateRuleEvent, Expr};

use crate::{
    PgDieselDatabase,
    model_metadata::{PgRuleEntry, RuleRelationKind},
};

impl Metadata for PgRuleEntry {
    type Meta = ();
}

impl RuleLike for PgRuleEntry {
    type DB = PgDieselDatabase;

    fn name(&self) -> &str {
        PgRuleEntry::name(self)
    }

    fn target_relation_name(&self) -> TargetName<'_> {
        // Catalog names are stored names: folding again would rename them.
        TargetName::new(self.relation_name(), true).with_schema(self.relation_schema(), true)
    }

    /// The relation is read by the stored name the catalog bound it to, the
    /// kind `pg_rewrite` records deciding the store it lives in.
    fn relation<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<Relation<'db, Self::DB>, LookupError>
    where
        Self: 'db,
    {
        let target = self.target_relation_name();
        let not_found = || LookupError::RelationNotFound {
            object_name: format!("{}.{}", self.relation_schema(), self.relation_name()),
        };
        match self.relation_kind() {
            RuleRelationKind::View => database
                .view_by_target(target, IdentifierCase::AsWritten)?
                .map(Relation::View),
            RuleRelationKind::ForeignTable => database
                .foreign_table_by_target(target, IdentifierCase::AsWritten)?
                .map(Relation::ForeignTable),
            RuleRelationKind::Table => database
                .table_by_target(target, IdentifierCase::AsWritten)?
                .map(Relation::Table),
        }
        .ok_or_else(not_found)
    }

    fn event(&self) -> CreateRuleEvent {
        PgRuleEntry::event(self)
    }

    fn is_instead(&self) -> bool {
        PgRuleEntry::is_instead(self)
    }

    fn condition(&self) -> Option<&Expr> {
        PgRuleEntry::condition(self)
    }

    fn action(&self) -> &CreateRuleAction {
        PgRuleEntry::action(self)
    }
}
