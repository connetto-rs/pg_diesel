//! Implementation of [`RuleLike`] for [`PgRuleEntry`].

use sql_traits::{
    errors::LookupError,
    structs::TargetName,
    traits::{Metadata, Relation, RuleLike},
};
use sqlparser::ast::{CreateRuleAction, CreateRuleEvent, Expr};

use crate::{PgDieselDatabase, model_metadata::PgRuleEntry};

impl Metadata for PgRuleEntry {
    type Meta = ();
}

impl RuleLike for PgRuleEntry {
    type DB = PgDieselDatabase;

    fn name(&self) -> &str {
        PgRuleEntry::name(self)
    }

    fn target_relation_name(&self) -> TargetName<'_> {
        self.target().target_name()
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
        self.target().resolve(database)
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
