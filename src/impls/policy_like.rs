//! `PolicyLike` implementation.

use sql_traits::{
    errors::{LookupError, ObjectKind},
    structs::TargetName,
    traits::{DatabaseLike, Metadata, PolicyLike},
};
use sqlparser::ast::{CreatePolicyCommand, CreatePolicyType, Expr, Owner};

use crate::{
    database::PgDieselDatabase,
    model_metadata::{PgPolicy, PolicyMetadata},
};

impl Metadata for PgPolicy {
    type Meta = PolicyMetadata;
}

/// Returns the metadata `database` holds for `policy`.
fn metadata<'db>(
    policy: &PgPolicy,
    database: &'db PgDieselDatabase,
) -> Result<&'db PolicyMetadata, LookupError> {
    database
        .policy_metadata(policy)
        .ok_or_else(|| LookupError::ObjectNotInDatabase {
            object_kind: ObjectKind::Policy,
            object: policy.model().polname.clone(),
        })
}

impl PolicyLike for PgPolicy {
    type DB = PgDieselDatabase;

    fn name(&self) -> &str {
        &self.model().polname
    }

    fn table<'db>(
        &'db self,
        _database: &'db Self::DB,
    ) -> Result<&'db <Self::DB as DatabaseLike>::Table, LookupError>
    where
        Self: 'db,
    {
        Ok(PgPolicy::table(self))
    }

    fn target_table_name(&self) -> TargetName<'_> {
        let table = PgPolicy::table(self);
        // Catalog names are stored names: folding again would rename them.
        TargetName::new(table.name(), true).with_schema(table.schema(), true)
    }

    fn command(&self) -> CreatePolicyCommand {
        match self.model().polcmd.as_str() {
            "r" => CreatePolicyCommand::Select,
            "a" => CreatePolicyCommand::Insert,
            "w" => CreatePolicyCommand::Update,
            "d" => CreatePolicyCommand::Delete,
            _ => CreatePolicyCommand::All,
        }
    }

    fn policy_type(&self) -> CreatePolicyType {
        if self.model().polpermissive {
            CreatePolicyType::Permissive
        } else {
            CreatePolicyType::Restrictive
        }
    }

    fn applies_to_public(&self) -> bool {
        // `polroles` holds OID zero for `TO PUBLIC`, and for an absent clause.
        self.model().polroles.contains(&0)
    }

    fn roles<'db>(&'db self, database: &'db Self::DB) -> impl Iterator<Item = &'db Owner>
    where
        Self: 'db,
    {
        metadata(self, database)
            .into_iter()
            .flat_map(PolicyMetadata::roles)
    }

    fn using_expression<'db>(&'db self, database: &'db Self::DB) -> Option<&'db Expr>
    where
        Self: 'db,
    {
        metadata(self, database).ok()?.using_expression()
    }

    fn check_expression<'db>(&'db self, database: &'db Self::DB) -> Option<&'db Expr>
    where
        Self: 'db,
    {
        metadata(self, database).ok()?.check_expression()
    }

    fn using_functions<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Function>, LookupError> {
        Ok(metadata(self, database)?
            .using_functions()
            .map(AsRef::as_ref))
    }

    fn check_functions<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Function>, LookupError> {
        Ok(metadata(self, database)?
            .check_functions()
            .map(AsRef::as_ref))
    }
}
