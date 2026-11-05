//! Implementation of [`FunctionLike`] for [`PgFunction`].

use std::borrow::Cow;

use sql_traits::{
    errors::{LookupError, ObjectKind},
    structs::{FunctionBody, FunctionKind, TargetName},
    traits::{DatabaseLike, FunctionLike, Metadata, RoleLike},
    utils::identifier_resolution::SearchPath,
};
use sqlparser::ast::{
    ArgMode, FunctionBehavior, FunctionCalledOnNull, FunctionDefinitionSetParam, FunctionSecurity,
};

use crate::{PgDieselDatabase, database::PgProcMetadata, model_metadata::PgFunction};

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
        // `proargnames` holds an empty string for an unnamed argument.
        self.model()
            .proargnames
            .iter()
            .flatten()
            .map(|name| (!name.is_empty()).then(|| TargetName::new(name, true)))
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

    fn returns_null_on_null_argument(&self, _database: &Self::DB, index: usize) -> bool {
        if !self.model().proisstrict {
            return false;
        }
        // A `VARIADIC` argument collects into a set, so strictness does not settle what it yields.
        let Some(mode) = self
            .argument_modes()
            .filter(|mode| !matches!(mode, ArgMode::Out))
            .nth(index)
        else {
            return false;
        };
        !matches!(mode, ArgMode::Variadic)
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
}
