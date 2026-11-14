//! Submodule providing [`PgPolicy`], the row security policy a database holds.

use std::sync::Arc;

use crate::{model_metadata::PgTable, models::PgPolicyTable};

/// A `pg_policy` row paired with the table its `polrelid` names.
///
/// `PolicyLike` asks for that table without handing over a database.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgPolicy {
    /// The `pg_catalog.pg_policy` row describing the policy.
    model: PgPolicyTable,
    /// The table the policy is declared on.
    table: Arc<PgTable>,
}

impl PgPolicy {
    /// Pairs a `pg_policy` row with the table its `polrelid` names.
    #[must_use]
    pub fn new(model: PgPolicyTable, table: Arc<PgTable>) -> Self {
        Self { model, table }
    }

    /// Returns the `pg_catalog.pg_policy` row describing the policy.
    #[must_use]
    pub fn model(&self) -> &PgPolicyTable {
        &self.model
    }

    /// Returns the table the policy is declared on.
    #[must_use]
    pub fn table(&self) -> &PgTable {
        &self.table
    }
}
