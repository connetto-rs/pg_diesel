//! `RoleMetadata` struct.

use crate::{model_metadata::PgPolicy, models::PgRole};
use std::sync::Arc;

#[derive(Debug, Clone)]
/// Metadata for a role.
pub struct RoleMetadata {
    /// Roles that this role is a member of (roles granted to this role).
    pub member_of: Vec<Arc<PgRole>>,
    /// Policies that reference this role.
    pub policies: Vec<Arc<PgPolicy>>,
}

impl RoleMetadata {
    /// Creates a new `RoleMetadata`.
    #[must_use]
    pub fn new(member_of: Vec<Arc<PgRole>>, policies: Vec<Arc<PgPolicy>>) -> Self {
        Self {
            member_of,
            policies,
        }
    }

    /// Returns the roles that this role is a member of.
    pub fn member_of(&self) -> impl Iterator<Item = &Arc<PgRole>> {
        self.member_of.iter()
    }

    /// Returns the policies that reference this role.
    pub fn policies(&self) -> impl Iterator<Item = &Arc<PgPolicy>> {
        self.policies.iter()
    }
}
