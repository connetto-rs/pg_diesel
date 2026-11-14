//! Implementation of [`FunctionLike`] for [`PgFunction`].

use std::borrow::Cow;

use sql_traits::{
    errors::{LookupError, ObjectKind},
    structs::{AggregateKind, FunctionBody, FunctionKind, PrivilegeHolder, TargetName},
    traits::{DatabaseLike, FunctionLike, Metadata, RoleLike},
    utils::{identifier_resolution::SearchPath, postgres_null_returning_arguments},
};
use sqlparser::ast::{
    ArgMode, FunctionBehavior, FunctionCalledOnNull, FunctionDefinitionSetParam, FunctionSecurity,
};

use crate::{
    PgDieselDatabase,
    database::{ExecuteHolder, PgProcMetadata},
    model_metadata::PgFunction,
};

impl Metadata for PgFunction {
    type Meta = PgProcMetadata;
}

/// Returns the metadata `database` holds for `function`.
fn metadata<'db>(
    function: &PgFunction,
    database: &'db PgDieselDatabase,
) -> Result<&'db PgProcMetadata, LookupError> {
    database
        .function_metadata(function)
        .ok_or_else(|| ObjectKind::Function.not_in_database(function.name()))
}

impl FunctionLike for PgFunction {
    type DB = PgDieselDatabase;

    fn name(&self) -> &str {
        &self.model().proname
    }

    fn name_is_quoted(&self) -> bool {
        // Catalog names are stored names: folding again would rename them.
        true
    }

    fn target_name(&self) -> TargetName<'_> {
        TargetName::new(self.name(), true).with_schema(self.schema(), true)
    }

    fn written_body(&self) -> Option<FunctionBody<'_>> {
        if let Some(definition) = self.bound_definition() {
            return Some(FunctionBody::BoundDefinition(definition));
        }
        // `AS 'library', 'symbol'` stores the library in `probin` and the
        // symbol in `prosrc`, and the written body is the library.
        match self.model().probin.as_deref() {
            Some(library) => std::str::from_utf8(library).ok().map(FunctionBody::Text),
            None => Some(FunctionBody::Text(&self.model().prosrc)),
        }
    }

    fn bound_body_functions<'db>(
        &self,
        database: &'db Self::DB,
    ) -> Result<
        impl Iterator<Item = &'db <Self::DB as DatabaseLike>::Function> + use<'db>,
        LookupError,
    >
    where
        Self: 'db,
    {
        let reached = metadata(self, database)?
            .bound_functions()
            .map_err(|reason| LookupError::FunctionBodyUnparsable {
                function: self.name().to_owned(),
                reason: reason.to_owned(),
            })?;
        Ok(database
            .functions()
            .filter(move |function| reached.contains(&function.oid())))
    }

    fn returns_set(&self) -> bool {
        self.model().proretset
    }

    fn argument_names<'db>(
        &'db self,
        _database: &'db Self::DB,
    ) -> impl Iterator<Item = Option<TargetName<'db>>> {
        // `proargnames` holds an empty string for an unnamed argument, and is
        // `NULL` when no argument has a name.
        let names = self.model().proargnames.as_deref();
        let unnamed = if names.is_some() {
            0
        } else {
            self.argument_modes().count()
        };
        names
            .into_iter()
            .flatten()
            .map(|name| (!name.is_empty()).then(|| TargetName::new(name, true)))
            .chain(std::iter::repeat_n(None, unnamed))
    }

    fn argument_type_names<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> impl Iterator<Item = Cow<'db, str>> {
        metadata(self, database)
            .into_iter()
            .flat_map(PgProcMetadata::argument_type_names)
            .map(|name| Cow::Borrowed(name.as_str()))
    }

    fn argument_modes(&self) -> impl Iterator<Item = ArgMode> {
        PgFunction::argument_modes(self)
    }

    fn argument_default_count(&self) -> usize {
        let defaults = self.model().pronargdefaults.max(0);
        usize::try_from(defaults).unwrap_or_default()
    }

    fn return_type_name<'db>(&'db self, database: &'db Self::DB) -> Option<Cow<'db, str>> {
        metadata(self, database)
            .ok()?
            .return_type_name()
            .map(Cow::Borrowed)
    }

    fn result_type_name<'db>(&'db self, database: &'db Self::DB) -> Option<Cow<'db, str>> {
        metadata(self, database)
            .ok()?
            .result_type_name()
            .map(Cow::Borrowed)
    }

    fn kind(&self, database: &Self::DB) -> Result<FunctionKind, LookupError> {
        // The lookup reports a function `database` does not hold.
        metadata(self, database)?;
        Ok(match self.model().prokind.as_str() {
            "a" => FunctionKind::Aggregate,
            "w" => FunctionKind::Window,
            _ => FunctionKind::Normal,
        })
    }

    fn aggregate_kind(&self, database: &Self::DB) -> Result<Option<AggregateKind>, LookupError> {
        Ok(metadata(self, database)?.aggregate_kind())
    }

    fn language(&self) -> Option<&str> {
        PgFunction::language(self)
    }

    fn language_is_quoted(&self) -> bool {
        true
    }

    fn volatility(&self) -> FunctionBehavior {
        match self.model().provolatile.as_str() {
            "i" => FunctionBehavior::Immutable,
            "s" => FunctionBehavior::Stable,
            _ => FunctionBehavior::Volatile,
        }
    }

    fn security_mode(&self) -> FunctionSecurity {
        if self.model().prosecdef {
            FunctionSecurity::Definer
        } else {
            FunctionSecurity::Invoker
        }
    }

    fn null_input_behavior(&self) -> FunctionCalledOnNull {
        if self.model().proisstrict {
            FunctionCalledOnNull::Strict
        } else {
            FunctionCalledOnNull::CalledOnNullInput
        }
    }

    fn configuration_parameters(&self) -> &[FunctionDefinitionSetParam] {
        self.configuration()
    }

    fn search_path<'db>(
        &'db self,
        _database: &'db Self::DB,
    ) -> Result<Option<SearchPath<'db>>, LookupError> {
        // Each entry carries the quote state the catalog recorded for it.
        Ok(PgFunction::own_search_path(self).map(|schemas| schemas.into_iter().collect()))
    }

    fn owner<'db>(&self, database: &'db Self::DB) -> Result<Option<&'db str>, LookupError> {
        Ok(metadata(self, database)?.owner())
    }

    fn returns_null_on_null_argument(&self, database: &Self::DB, index: usize) -> bool {
        let inputs: Vec<(ArgMode, Cow<'_, str>)> = self
            .argument_modes()
            .zip(self.argument_type_names(database))
            .filter(|(mode, _)| !matches!(mode, ArgMode::Out))
            .collect();
        let Some((mode, _)) = inputs.get(index) else {
            return false;
        };
        if self.model().proisstrict {
            // A `VARIADIC` argument collects into a set, so strictness does not settle what it yields.
            return !matches!(mode, ArgMode::Variadic);
        }
        // `pg_proc` records no per-argument behaviour for a non-strict function,
        // which sql-traits keeps for the PostgreSQL builtins that have one.
        let types: Vec<&str> = inputs.iter().map(|(_, name)| name.as_ref()).collect();
        postgres_null_returning_arguments(self.schema(), self.name(), &types).contains(&index)
    }

    fn is_executable_by_public(&self, database: &Self::DB) -> Result<bool, LookupError> {
        Ok(metadata(self, database)?.is_executable_by_public())
    }

    fn can_execute(
        &self,
        role: &<Self::DB as DatabaseLike>::Role,
        database: &Self::DB,
    ) -> Result<bool, LookupError> {
        let execute = metadata(self, database)?;
        let mut holders = std::iter::once(role).chain(role.inherited_roles(database));
        Ok(holders.any(|holder| execute.can_execute(holder.oid)))
    }

    fn can_grant_execute(
        &self,
        role: &<Self::DB as DatabaseLike>::Role,
        database: &Self::DB,
    ) -> Result<bool, LookupError> {
        let execute = metadata(self, database)?;
        let mut holders = std::iter::once(role).chain(role.inherited_roles(database));
        Ok(holders.any(|holder| execute.can_grant_execute(holder.oid)))
    }

    fn execute_holders<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<impl Iterator<Item = PrivilegeHolder<'db>>, LookupError> {
        let holders = metadata(self, database)?.execute_holders();
        Ok(holders.filter_map(move |holder| match holder {
            ExecuteHolder::Public => Some(PrivilegeHolder::Public),
            ExecuteHolder::Role(oid) => database
                .roles()
                .find(|role| role.oid == Some(oid))
                .and_then(|role| role.rolname.as_deref())
                .map(|name| PrivilegeHolder::Role(Cow::Borrowed(name))),
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{
        database::CatalogCache,
        dialect::PostgresDialect,
        model_metadata::RoleMetadata,
        models::{PgProc, PgRole, ProcAclEntry},
    };

    fn role(oid: u32, name: &str) -> PgRole {
        PgRole {
            rolname: Some(name.to_string()),
            rolsuper: None,
            rolinherit: None,
            rolcreaterole: None,
            rolcreatedb: None,
            rolcanlogin: None,
            rolreplication: None,
            rolconnlimit: None,
            rolpassword: None,
            rolvaliduntil: None,
            rolbypassrls: None,
            rolconfig: None,
            oid: Some(oid),
        }
    }

    #[test]
    fn a_refused_bound_body_reports_the_refusal() {
        let model = PgProc::fixture(1);
        let function = Arc::new(PgFunction::new(
            model,
            "app".to_string(),
            Some("sql".to_string()),
        ));
        let metadata = PgProcMetadata::new(function.model(), &CatalogCache::default(), None)
            .with_bound_functions(Err("the definition does not parse".to_owned()));
        let database = PgDieselDatabase::new("catalog".to_string(), PostgresDialect)
            .add_function(Arc::clone(&function), metadata)
            .into();
        let error = function
            .bound_body_functions(&database)
            .err()
            .expect("the parser refused the bound body");
        assert!(
            matches!(error, LookupError::FunctionBodyUnparsable { .. }),
            "the refusal names the function and the reason"
        );
    }

    #[test]
    fn the_execute_holders_map_the_roles_the_access_list_names() {
        let mut model = PgProc::fixture(1);
        model.proowner = 500;
        let function = Arc::new(PgFunction::new(
            model,
            "app".to_string(),
            Some("sql".to_string()),
        ));
        let grants = vec![
            ProcAclEntry {
                function_oid: 1,
                grantee: Some(0),
                grantable: Some(false),
            },
            ProcAclEntry {
                function_oid: 1,
                grantee: Some(500),
                grantable: Some(false),
            },
            ProcAclEntry {
                function_oid: 1,
                grantee: Some(601),
                grantable: Some(true),
            },
            ProcAclEntry {
                function_oid: 1,
                grantee: Some(601),
                grantable: Some(false),
            },
            ProcAclEntry {
                function_oid: 1,
                grantee: Some(602),
                grantable: Some(false),
            },
        ];
        let metadata =
            PgProcMetadata::new(function.model(), &CatalogCache::default(), Some(&grants));
        let database = PgDieselDatabase::new("catalog".to_string(), PostgresDialect)
            .add_function(Arc::clone(&function), metadata)
            .add_role(
                Arc::new(role(500, "owner")),
                RoleMetadata::new(Vec::new(), Vec::new()),
            )
            .add_role(
                Arc::new(role(601, "grantee_a")),
                RoleMetadata::new(Vec::new(), Vec::new()),
            )
            .add_role(
                Arc::new(role(602, "grantee_b")),
                RoleMetadata::new(Vec::new(), Vec::new()),
            )
            .into();
        let holders: Vec<PrivilegeHolder> = function
            .execute_holders(&database)
            .expect("in database")
            .collect();
        assert_eq!(
            holders,
            vec![
                PrivilegeHolder::Public,
                PrivilegeHolder::Role("owner".into()),
                PrivilegeHolder::Role("grantee_a".into()),
                PrivilegeHolder::Role("grantee_b".into()),
            ]
        );
    }
}
