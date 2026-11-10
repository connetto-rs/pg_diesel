//! Builder pattern for constructing a [`PgDieselDatabase`] instance.

use std::{collections::HashMap, sync::Arc};

use diesel::{
    ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl,
    dsl::sql,
    sql_types::{Array, Text},
};
use sql_traits::{
    errors::LookupError,
    structs::{IdentifierCase, generic_db::GenericDBBuilder, metadata::ExpressionCalls},
};
use sqlparser::{
    ast::{
        Action, Expr, Grantee, GranteeName, GranteesType, Ident, ObjectName, ObjectNamePart, Owner,
    },
    dialect::PostgreSqlDialect,
    parser::Parser,
};

use crate::{
    PgDieselDatabase,
    database::{AclSchemaGrant, AclTableGrant, CatalogCache, FIRST_NORMAL_OBJECT_ID},
    dialect::PostgresDialect,
    impls::{RoleColumnGrantsMetadata, RoleTableGrantsMetadata, string_to_action},
    model_metadata::{
        DomainDefinitionError, PgColumn, PgDomainEntry, PgExtensionEntry, PgFunction, PgPolicy,
        PgRoleMembership, PgRuleEntry, PgSchema, PgSequenceEntry, PgTable, PgTableError,
        PgUserDefinedType, PgViewDefinition, RuleDefinitionError, TriggerMetadata,
        ViewDefinitionError,
    },
    models::{
        PgDbRoleSetting, PgMatview, PgProc, PgRole, PgView, RoleColumnGrants, RoleSettingError,
        RoleTableGrants, Table, TableMetadataError,
    },
};

/// Returns the schemas the reading session resolves unqualified names in,
/// which is the path `pg_get_expr` leaves names unqualified under.
///
/// # Errors
///
/// Returns a `diesel::result::Error` if the query fails.
fn session_search_path(
    connection: &mut PgConnection,
) -> Result<Vec<(String, bool)>, diesel::result::Error> {
    let schemas: Vec<String> = diesel::select(sql::<Array<Text>>("current_schemas(false)::text[]"))
        .get_result(connection)?;
    Ok(schemas.into_iter().map(|schema| (schema, true)).collect())
}

/// The privileges `GRANT ALL` on a plain or partitioned table hands over,
/// the full set the server materializes for it.
#[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
const TABLE_ALL_PRIVILEGES: [&str; 7] = [
    "SELECT",
    "INSERT",
    "UPDATE",
    "DELETE",
    "TRUNCATE",
    "REFERENCES",
    "TRIGGER",
];

/// The privileges `GRANT ALL` on a plain or partitioned table hands over,
/// the full set the server materializes for it.
#[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
const TABLE_ALL_PRIVILEGES: [&str; 8] = [
    "SELECT",
    "INSERT",
    "UPDATE",
    "DELETE",
    "TRUNCATE",
    "REFERENCES",
    "TRIGGER",
    "MAINTAIN",
];

/// The privileges `GRANT ALL` on a view, materialized view or foreign table
/// hands over, the full set the server materializes for it.
const VIEW_ALL_PRIVILEGES: [&str; 4] = ["SELECT", "INSERT", "UPDATE", "DELETE"];

/// The privileges `GRANT ALL` on a sequence hands over, the full set the
/// server materializes for it.
const SEQUENCE_ALL_PRIVILEGES: [&str; 3] = ["USAGE", "SELECT", "UPDATE"];

/// The full privilege set a `GRANT ALL` on a relation of kind `relkind`
/// materializes, empty when the kind is no relation the grant reaches.
fn all_privileges_for(relkind: &str) -> &'static [&'static str] {
    match relkind {
        "r" | "p" => &TABLE_ALL_PRIVILEGES,
        "v" | "m" | "f" => &VIEW_ALL_PRIVILEGES,
        "S" => &SEQUENCE_ALL_PRIVILEGES,
        _ => &[],
    }
}

/// The grantee one `aclexplode` entry names, `PUBLIC` for the pseudo-role.
fn acl_grantee(name: &str) -> Grantee {
    if name.eq_ignore_ascii_case("PUBLIC") {
        Grantee {
            grantee_type: GranteesType::Public,
            name: None,
        }
    } else {
        Grantee {
            grantee_type: GranteesType::None,
            name: Some(GranteeName::ObjectName(ObjectName(vec![
                ObjectNamePart::Identifier(Ident::new(name)),
            ]))),
        }
    }
}

/// The action a schema privilege `aclexplode` lists names, or `None` for one
/// the AST has no action for.
fn schema_action(privilege: &str) -> Option<Action> {
    match privilege {
        "USAGE" => Some(Action::Usage),
        "CREATE" => Some(Action::Create { obj_type: None }),
        _ => None,
    }
}

/// Returns `expression`'s calls resolved against `functions` under
/// `search_path`, or no calls for an absent expression.
fn expression_calls(
    expression: Option<&Expr>,
    functions: &[Arc<PgFunction>],
    search_path: &[(&str, bool)],
) -> ExpressionCalls<PgFunction> {
    expression.map_or_else(ExpressionCalls::default, |expression| {
        ExpressionCalls::resolve(
            expression,
            functions,
            search_path,
            IdentifierCase::AsWritten,
        )
    })
}

/// Returns the pinned functions the calls of the bound body `definition`
/// reach, resolved against `functions` under `search_path`, the path
/// `pg_get_functiondef` printed the definition under.
///
/// # Errors
///
/// Returns the parser's complaint when `definition` does not parse.
fn bound_calls(
    definition: &str,
    functions: &[Arc<PgFunction>],
    search_path: &[(&str, bool)],
) -> Result<Vec<u32>, String> {
    let statements =
        Parser::parse_sql(&PostgreSqlDialect {}, definition).map_err(|error| error.to_string())?;
    let calls = ExpressionCalls::resolve(
        &statements,
        functions,
        search_path,
        IdentifierCase::AsWritten,
    );
    let mut pinned: Vec<u32> = Vec::new();
    for function in calls.functions() {
        if function.oid() < FIRST_NORMAL_OBJECT_ID && !pinned.contains(&function.oid()) {
            pinned.push(function.oid());
        }
    }
    Ok(pinned)
}

/// Returns the `pg_class.reloptions` of every view and materialized view in
/// the schemas, keyed by the schema and name `pg_class` records the relation
/// under.
///
/// # Errors
///
/// Returns a `diesel::result::Error` if the query fails.
fn relation_options(
    connection: &mut PgConnection,
    schemas: &[String],
) -> Result<HashMap<(String, String), Vec<String>>, diesel::result::Error> {
    use crate::schema::pg_catalog::{pg_class::pg_class, pg_namespace::pg_namespace};

    let rows = pg_class::table
        .inner_join(pg_namespace::table.on(pg_namespace::oid.eq(pg_class::relnamespace)))
        .filter(pg_namespace::nspname.eq_any(schemas))
        .filter(pg_class::relkind.eq_any(&["v", "m"]))
        .select((
            pg_namespace::nspname,
            pg_class::relname,
            pg_class::reloptions,
        ))
        .load::<(String, String, Option<Vec<String>>)>(connection)?;
    Ok(rows
        .into_iter()
        .map(|(schema, name, options)| ((schema, name), options.unwrap_or_default()))
        .collect())
}

/// Returns the `pg_class.reloptions` the relation of the given schema and
/// name carries, empty when the relation left none.
fn relation_options_for<'a>(
    options: &'a HashMap<(String, String), Vec<String>>,
    schema: Option<&str>,
    name: Option<&str>,
) -> &'a [String] {
    schema
        .zip(name)
        .and_then(|(schema, name)| options.get(&(schema.to_owned(), name.to_owned())))
        .map_or_default(Vec::as_slice)
}

#[derive(Default)]
/// Builder for constructing a [`PgDieselDatabase`] instance from `PostgreSQL` metadata.
pub struct PgDieselDatabaseBuilder<'conn> {
    /// Connection to the `PostgreSQL` database.
    connection: Option<&'conn mut PgConnection>,
    /// The catalog (database) name to filter by.
    catalog: Option<String>,
    /// The schema names to include.
    schemas: Vec<String>,
    /// Types denylist.
    denylist_types: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
/// Errors that can occur when building a [`PgDieselDatabase`] instance.
pub enum PgDatabaseBuildError {
    #[error("Missing required builder attribute: {0}")]
    /// An attribute was missing.
    MissingAttribute(&'static str),
    #[error("Diesel error: {0}")]
    /// An error occurred while querying the database schema.
    Diesel(#[from] diesel::result::Error),
    #[error("Duplicate denylisted type: {0}")]
    /// A deny-listed type was inserted multiple times.
    DuplicateDenylistedType(String),
    #[error("Table definition: {0}")]
    /// A table could not be read from the catalogs.
    TableDefinition(#[from] PgTableError),
    #[error("View definition: {0}")]
    /// A view could not be read from the catalogs.
    ViewDefinition(#[from] ViewDefinitionError),
    #[error("Rule definition: {0}")]
    /// A rule's definition could not be read from the catalogs.
    RuleDefinition(#[from] RuleDefinitionError),
    #[error("Domain definition: {0}")]
    /// A domain could not be read from the catalogs.
    DomainDefinition(#[from] DomainDefinitionError),
    #[error("Lookup error: {0}")]
    /// Adding an object left the database unable to resolve a name.
    Lookup(#[from] LookupError),
    #[error("Role setting: {0}")]
    /// A role setting could not be read from `pg_db_role_setting`.
    RoleSetting(#[from] RoleSettingError),
    #[error("Table metadata: {0}")]
    /// A table's columns, keys or indexes could not be read.
    TableMetadata(#[from] TableMetadataError),
    #[error("The privilege {privilege} on {object} has no sqlparser action")]
    /// The catalog grants a privilege `sqlparser::ast::Action` cannot spell.
    UnrepresentablePrivilege {
        /// The relation or column the privilege is granted on.
        object: String,
        /// The privilege as `PostgreSQL` names it.
        privilege: String,
    },
}

impl<'conn> PgDieselDatabaseBuilder<'conn> {
    /// Sets the `PostgreSQL` connection to use for building the `PgDieselDatabase`.
    #[must_use]
    pub fn connection(mut self, connection: &'conn mut PgConnection) -> Self {
        self.connection = Some(connection);
        self
    }

    /// Sets the catalog (database) name to filter by.
    #[must_use]
    pub fn catalog<S: AsRef<str>>(mut self, catalog: S) -> Self {
        self.catalog = Some(catalog.as_ref().to_string());
        self
    }

    /// Adds a schema name to include.
    #[must_use]
    pub fn schema<S: AsRef<str>>(mut self, schema: S) -> Self {
        self.schemas.push(schema.as_ref().to_string());
        self
    }

    /// Sets the schema names to include.
    #[must_use]
    pub fn schemas<I, S>(mut self, schemas: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: ToString + AsRef<str>,
    {
        for schema in schemas {
            if !self.schemas.contains(&schema.as_ref().to_string()) {
                self = self.schema(schema);
            }
        }
        self
    }

    /// Adds a type to the denylist.
    ///
    /// # Errors
    ///
    /// Returns [`PgDatabaseBuildError::DuplicateDenylistedType`] if the type is already in the denylist.
    pub fn denylist_type<S: AsRef<str>>(mut self, ty: S) -> Result<Self, PgDatabaseBuildError> {
        let ty_str = ty.as_ref().to_string();
        if self.denylist_types.contains(&ty_str) {
            return Err(PgDatabaseBuildError::DuplicateDenylistedType(ty_str));
        }
        self.denylist_types.push(ty_str);
        Ok(self)
    }

    /// Adds multiple types to the denylist.
    ///
    /// # Errors
    ///
    /// Returns [`PgDatabaseBuildError::DuplicateDenylistedType`] if any type is already in the denylist.
    pub fn denylist_types<I, S>(mut self, types: I) -> Result<Self, PgDatabaseBuildError>
    where
        I: IntoIterator<Item = S>,
        S: ToString + AsRef<str>,
    {
        for ty in types {
            self = self.denylist_type(ty)?;
        }
        Ok(self)
    }
}

impl<'a> TryFrom<PgDieselDatabaseBuilder<'a>> for PgDieselDatabase {
    type Error = PgDatabaseBuildError;

    #[allow(clippy::needless_borrow)]
    #[allow(clippy::too_many_lines)]
    fn try_from(value: PgDieselDatabaseBuilder<'a>) -> Result<Self, Self::Error> {
        let connection = value
            .connection
            .ok_or(PgDatabaseBuildError::MissingAttribute("connection"))?;

        let table_catalog = value
            .catalog
            .ok_or(PgDatabaseBuildError::MissingAttribute("catalog"))?;

        let table_schemas = {
            if value.schemas.is_empty() {
                return Err(PgDatabaseBuildError::MissingAttribute("schemas"));
            }
            value.schemas
        };

        let mut generic_builder = GenericDBBuilder::new(table_catalog.clone(), PostgresDialect);

        for schema in PgSchema::load_all(&table_catalog, &table_schemas, connection)? {
            generic_builder = generic_builder.add_schema(Arc::new(schema), ());
        }

        let cache = CatalogCache::load(connection)?;
        let search_path = session_search_path(connection)?;
        let search_path: Vec<(&str, bool)> = search_path
            .iter()
            .map(|(schema, quoted)| (schema.as_str(), *quoted))
            .collect();

        let roles: Vec<Arc<PgRole>> = PgRole::load_all(connection)?
            .into_iter()
            .map(Arc::new)
            .collect();

        // Functions come first: other objects reference them.
        let mut function_acls = PgProc::load_all_execute_grants(connection)?
            .into_iter()
            .fold(
                HashMap::<u32, Vec<crate::models::ProcAclEntry>>::new(),
                |mut acls, entry| {
                    acls.entry(entry.function_oid).or_default().push(entry);
                    acls
                },
            );
        let mut bound_definitions: HashMap<u32, String> =
            PgProc::load_bound_definitions(connection)?
                .into_iter()
                .collect();
        let mut bound_dependencies = PgProc::load_bound_dependencies(connection)?
            .into_iter()
            .fold(
                HashMap::<u32, Vec<u32>>::new(),
                |mut reached, (function, called)| {
                    reached.entry(function).or_default().push(called);
                    reached
                },
            );
        let mut loaded = Vec::new();
        for function in PgProc::load_all(connection)? {
            let acls = function_acls.remove(&function.oid);
            let metadata = crate::database::PgProcMetadata::new(&function, &cache, acls.as_deref());
            let schema = cache
                .namespace(function.pronamespace)
                .unwrap_or_default()
                .to_owned();
            let language = cache.language(function.prolang).map(ToOwned::to_owned);
            let definition = bound_definitions.remove(&function.oid);
            let mut pg_function = PgFunction::new(function, schema, language);
            if let Some(definition) = definition {
                pg_function = pg_function.with_bound_definition(definition);
            }
            loaded.push((Arc::new(pg_function), metadata));
        }
        let functions: Vec<Arc<PgFunction>> = loaded
            .iter()
            .map(|(function, _)| Arc::clone(function))
            .collect();
        for (function, metadata) in loaded {
            let recorded = bound_dependencies
                .remove(&function.oid())
                .unwrap_or_default();
            let metadata = match function.bound_definition() {
                Some(definition) => metadata.with_bound_functions(
                    bound_calls(definition, &functions, &search_path).map(|pinned| {
                        let mut reached = recorded;
                        for oid in pinned {
                            if !reached.contains(&oid) {
                                reached.push(oid);
                            }
                        }
                        reached
                    }),
                ),
                None => metadata,
            };
            generic_builder = generic_builder.add_function(function, metadata);
        }

        let mut tables = Vec::new();
        for table_schema in &table_schemas {
            for row in Table::load_all(connection, &table_catalog, table_schema)? {
                let facts = row.table_facts(connection)?;
                let partition_strategy = row.partition_strategy(connection)?;
                tables.push(Arc::new(PgTable::new(
                    row,
                    partition_strategy.as_deref(),
                    &facts,
                )?));
            }
        }

        tables.sort_by(|left, right| {
            (left.schema(), left.name()).cmp(&(right.schema(), right.name()))
        });

        let mut tables_by_schema_name: HashMap<(String, String), Arc<PgTable>> = HashMap::new();
        let mut columns_by_table_column: HashMap<(String, String, String), Arc<PgColumn>> =
            HashMap::new();
        let mut policies_by_role: HashMap<String, Vec<Arc<PgPolicy>>> = HashMap::new();

        for table in tables {
            tables_by_schema_name.insert(
                (table.schema().to_string(), table.name().to_string()),
                Arc::clone(&table),
            );

            let table_metadata = table.metadata(connection, &value.denylist_types, &cache)?;

            for column in table_metadata.column_arcs() {
                columns_by_table_column.insert(
                    (
                        table.schema().to_string(),
                        table.name().to_string(),
                        column.column_name().to_string(),
                    ),
                    Arc::clone(column),
                );

                let metadata = table_metadata
                    .column_metadata(column.column_name())
                    .expect("a column of the table carries the metadata read with it")
                    .clone();
                generic_builder = generic_builder.add_column(Arc::clone(column), metadata);
            }

            for check_constraint in table_metadata.check_constraint_arcs() {
                let metadata = check_constraint.metadata(
                    &table_metadata,
                    generic_builder.function_arc_vec().as_slice(),
                    &search_path,
                );
                generic_builder =
                    generic_builder.add_check_constraint(Arc::clone(check_constraint), metadata);
            }

            for fk in table_metadata.foreign_key_arcs() {
                let metadata = fk.model().metadata(Arc::clone(&table), connection)?;
                generic_builder = generic_builder.add_foreign_key(Arc::clone(fk), metadata);
            }

            for index in table_metadata.unique_index_arcs() {
                generic_builder = generic_builder.add_unique_index(
                    Arc::clone(index),
                    index.metadata(Arc::clone(&table), connection)?,
                );
            }

            for index in table_metadata.index_arcs() {
                generic_builder = generic_builder.add_index(
                    Arc::clone(index),
                    index.metadata(Arc::clone(&table), connection)?,
                );
            }

            for (trigger, function_name, update_columns) in table_metadata.triggers() {
                let metadata = TriggerMetadata::new(
                    (**trigger).clone(),
                    Arc::clone(&table),
                    function_name.clone(),
                    update_columns.clone(),
                );
                generic_builder = generic_builder.add_trigger(Arc::new(metadata), ());
            }

            for policy in table_metadata.policies() {
                let parse_expr = |sql: Option<String>| -> Option<Expr> {
                    Parser::new(&PostgreSqlDialect {})
                        .try_with_sql(&sql?)
                        .ok()?
                        .parse_expr()
                        .ok()
                };

                let (using, check) = crate::models::pg_policy_table::cached_queries::expressions(
                    policy.model(),
                    connection,
                )?;
                let using_expression = parse_expr(using);
                let check_expression = parse_expr(check);

                let mut roles = Vec::new();
                for role in crate::models::pg_policy_table::cached_queries::roles(
                    policy.model(),
                    connection,
                )? {
                    policies_by_role
                        .entry(role.rolname.clone())
                        .or_default()
                        .push(Arc::clone(policy));
                    roles.push(Owner::Ident(Ident::new(role.rolname)));
                }

                let functions = generic_builder.function_arc_vec();
                let metadata = crate::model_metadata::PolicyMetadata::new(
                    expression_calls(using_expression.as_ref(), &functions, &search_path),
                    expression_calls(check_expression.as_ref(), &functions, &search_path),
                    using_expression,
                    check_expression,
                    roles,
                );

                generic_builder = generic_builder.add_policy(Arc::clone(policy), metadata);
            }

            generic_builder = generic_builder.add_table(table, table_metadata)?;
        }

        let view_options = relation_options(connection, &table_schemas)?;
        for view in PgView::load_all(&table_schemas, connection)? {
            let options = relation_options_for(
                &view_options,
                view.schemaname.as_deref(),
                view.viewname.as_deref(),
            );
            generic_builder =
                generic_builder.add_view(Arc::new(PgViewDefinition::plain(view, options)?), ());
        }
        for view in PgMatview::load_all(&table_schemas, connection)? {
            let options = relation_options_for(
                &view_options,
                view.schemaname.as_deref(),
                view.matviewname.as_deref(),
            );
            generic_builder = generic_builder.add_materialized_view(
                Arc::new(PgViewDefinition::materialized(view, options)?),
                (),
            );
        }

        for sequence in PgSequenceEntry::load_all(&table_schemas, connection)? {
            generic_builder = generic_builder.add_sequence(Arc::new(sequence), ());
        }

        for row in PgRuleEntry::load_rows(&table_schemas, connection)? {
            generic_builder = generic_builder.add_rule(Arc::new(PgRuleEntry::new(row)?), ());
        }

        for extension in PgExtensionEntry::load_all(connection)? {
            generic_builder = generic_builder.add_extension(Arc::new(extension), ());
        }

        for domain in PgDomainEntry::load_all(&table_schemas, &cache, connection)? {
            generic_builder = generic_builder.add_domain(Arc::new(domain), ());
        }

        for user_defined_type in PgUserDefinedType::load_all(&table_schemas, connection)? {
            generic_builder =
                generic_builder.add_user_defined_type(Arc::new(user_defined_type), ());
        }

        let mut role_memberships: std::collections::HashMap<u32, Vec<u32>> =
            std::collections::HashMap::new();
        let mut roles_map: std::collections::HashMap<u32, Arc<PgRole>> =
            std::collections::HashMap::new();

        for role in roles {
            let role_rc = Arc::clone(&role);

            if let Some(role_oid) = role.oid {
                roles_map.insert(role_oid, Arc::clone(&role_rc));

                let member_of_oids =
                    crate::models::pg_role::cached_queries::member_of(&role, connection)?;

                role_memberships.insert(role_oid, member_of_oids);
            }
        }

        for (role_oid, role_rc) in &roles_map {
            let Some(member_of_oids) = role_memberships.get(role_oid) else {
                continue;
            };

            let member_of: Vec<Arc<PgRole>> = member_of_oids
                .iter()
                .filter_map(|oid| roles_map.get(oid).cloned())
                .collect();

            let policies = role_rc
                .rolname
                .as_ref()
                .and_then(|name| policies_by_role.remove(name))
                .unwrap_or_default();

            let metadata = crate::model_metadata::RoleMetadata::new(member_of, policies);
            generic_builder = generic_builder.add_role(Arc::clone(role_rc), metadata);
        }

        for membership in PgRoleMembership::load_all(&cache, connection)? {
            generic_builder = generic_builder.add_role_membership(Arc::new(membership), ());
        }
        generic_builder = generic_builder.add_role_settings(PgDbRoleSetting::load_settings(
            &table_catalog,
            &cache,
            connection,
        )?);

        // The server expands `GRANT ALL` into one `aclexplode` row per
        // privilege, so the rows of one relation and grantee group back into
        // the grant the DDL wrote.
        let mut grants_by_relation: std::collections::BTreeMap<
            (String, String, String),
            Vec<AclTableGrant>,
        > = std::collections::BTreeMap::new();
        for grant in AclTableGrant::load(&table_schemas, connection)? {
            grants_by_relation
                .entry((
                    grant.table_schema.clone(),
                    grant.table_name.clone(),
                    grant.grantee.clone(),
                ))
                .or_default()
                .push(grant);
        }
        for ((table_schema, table_name, grantee), rows) in grants_by_relation {
            let held: std::collections::BTreeSet<String> = rows
                .iter()
                .map(|row| row.privilege_type.to_uppercase())
                .collect();
            let full: &'static [&'static str] = all_privileges_for(&rows[0].relkind);
            let full_set: std::collections::BTreeSet<String> =
                full.iter().copied().map(ToString::to_string).collect();
            if !full_set.is_empty() && held == full_set {
                let grantable = rows.iter().all(|row| row.is_grantable);
                let table_rc = tables_by_schema_name
                    .get(&(table_schema.clone(), table_name.clone()))
                    .cloned();
                let metadata =
                    RoleTableGrantsMetadata::new(None, Some(acl_grantee(&grantee)), table_rc);
                let model = RoleTableGrants {
                    grantor: Some(rows[0].grantor.clone()),
                    grantee: Some(grantee),
                    table_catalog: Some(table_catalog.clone()),
                    table_schema: Some(table_schema),
                    table_name: Some(table_name),
                    privilege_type: Some("ALL".to_owned()),
                    is_grantable: Some(if grantable { "YES" } else { "NO" }.to_owned()),
                    with_hierarchy: None,
                };
                generic_builder = generic_builder.add_table_grant(Arc::new(model), metadata);
                continue;
            }
            for grant in rows {
                let action = string_to_action(&grant.privilege_type).ok_or_else(|| {
                    PgDatabaseBuildError::UnrepresentablePrivilege {
                        object: format!("{}.{}", grant.table_schema, grant.table_name),
                        privilege: grant.privilege_type.clone(),
                    }
                })?;
                let table_rc = tables_by_schema_name
                    .get(&(grant.table_schema.clone(), grant.table_name.clone()))
                    .cloned();
                let metadata = RoleTableGrantsMetadata::new(
                    Some(action),
                    Some(acl_grantee(&grant.grantee)),
                    table_rc,
                );
                let model = RoleTableGrants {
                    grantor: Some(grant.grantor),
                    grantee: Some(grant.grantee),
                    table_catalog: Some(table_catalog.clone()),
                    table_schema: Some(grant.table_schema),
                    table_name: Some(grant.table_name),
                    privilege_type: Some(grant.privilege_type),
                    is_grantable: Some(if grant.is_grantable { "YES" } else { "NO" }.to_owned()),
                    with_hierarchy: None,
                };
                generic_builder = generic_builder.add_table_grant(Arc::new(model), metadata);
            }
        }

        // A schema grant goes through the table grants as in `ParserDB`,
        // a grant naming its schema and no relation.
        for grant in AclSchemaGrant::load(&table_schemas, connection)? {
            let action = schema_action(&grant.privilege_type).ok_or_else(|| {
                PgDatabaseBuildError::UnrepresentablePrivilege {
                    object: grant.schema.clone(),
                    privilege: grant.privilege_type.clone(),
                }
            })?;
            let metadata =
                RoleTableGrantsMetadata::new(Some(action), Some(acl_grantee(&grant.grantee)), None);
            let model = RoleTableGrants {
                grantor: Some(grant.grantor),
                grantee: Some(grant.grantee),
                table_catalog: Some(table_catalog.clone()),
                table_schema: Some(grant.schema),
                table_name: None,
                privilege_type: Some(grant.privilege_type),
                is_grantable: Some(if grant.is_grantable { "YES" } else { "NO" }.to_owned()),
                with_hierarchy: None,
            };
            generic_builder = generic_builder.add_table_grant(Arc::new(model), metadata);
        }

        for grant in RoleColumnGrants::load_acl(&table_schemas, connection)? {
            let action = string_to_action(&grant.privilege_type).ok_or_else(|| {
                PgDatabaseBuildError::UnrepresentablePrivilege {
                    object: format!(
                        "{}.{}.{}",
                        grant.table_schema, grant.table_name, grant.column_name
                    ),
                    privilege: grant.privilege_type.clone(),
                }
            })?;
            let table_rc = tables_by_schema_name
                .get(&(grant.table_schema.clone(), grant.table_name.clone()))
                .cloned();
            let column_rc = columns_by_table_column
                .get(&(
                    grant.table_schema.clone(),
                    grant.table_name.clone(),
                    grant.column_name.clone(),
                ))
                .cloned();
            let metadata = RoleColumnGrantsMetadata::new(
                Some(action),
                Some(acl_grantee(&grant.grantee)),
                table_rc,
                column_rc,
            );
            let model = RoleColumnGrants {
                grantor: Some(grant.grantor),
                grantee: Some(grant.grantee),
                table_catalog: Some(table_catalog.clone()),
                table_schema: Some(grant.table_schema),
                table_name: Some(grant.table_name),
                column_name: Some(grant.column_name),
                privilege_type: Some(grant.privilege_type),
                is_grantable: Some(if grant.is_grantable { "YES" } else { "NO" }.to_owned()),
            };
            generic_builder = generic_builder.add_column_grant(Arc::new(model), metadata);
        }

        Ok(generic_builder.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_denylist_type() {
        let builder = PgDieselDatabaseBuilder::default();
        let builder = builder.denylist_type("foo").unwrap();
        assert_eq!(builder.denylist_types, vec!["foo"]);
    }

    #[test]
    fn test_denylist_type_duplicate() {
        let builder = PgDieselDatabaseBuilder::default();
        let builder = builder.denylist_type("foo").unwrap();
        match builder.denylist_type("foo") {
            Ok(_) => panic!("Expected error"),
            Err(e) => match e {
                PgDatabaseBuildError::DuplicateDenylistedType(t) => assert_eq!(t, "foo"),
                _ => panic!("Unexpected error: {e:?}"),
            },
        }
    }

    #[test]
    fn test_denylist_types() {
        let builder = PgDieselDatabaseBuilder::default();
        let builder = builder.denylist_types(["foo", "bar"]).unwrap();
        assert_eq!(builder.denylist_types, vec!["foo", "bar"]);
    }

    #[test]
    fn test_denylist_types_duplicate() {
        let builder = PgDieselDatabaseBuilder::default();
        let builder = builder.denylist_type("foo").unwrap();
        match builder.denylist_types(["bar", "foo"]) {
            Ok(_) => panic!("Expected error"),
            Err(e) => match e {
                PgDatabaseBuildError::DuplicateDenylistedType(t) => assert_eq!(t, "foo"),
                _ => panic!("Unexpected error: {e:?}"),
            },
        }
    }

    #[test]
    fn test_schema() {
        let builder = PgDieselDatabaseBuilder::default();
        let builder = builder.schema("public");
        assert_eq!(builder.schemas, vec!["public"]);
    }

    #[test]
    fn test_schemas() {
        let builder = PgDieselDatabaseBuilder::default();
        let builder = builder.schemas(["public", "private"]);
        assert_eq!(builder.schemas, vec!["public", "private"]);
    }

    #[test]
    fn test_schemas_deduplication() {
        let builder = PgDieselDatabaseBuilder::default();
        let builder = builder.schema("public");
        let builder = builder.schemas(["public", "private"]);
        // "public" is already there, so it should not be added again by schemas()
        assert_eq!(builder.schemas, vec!["public", "private"]);
    }

    #[test]
    fn test_schema_no_deduplication() {
        let builder = PgDieselDatabaseBuilder::default();
        let builder = builder.schema("public");
        let builder = builder.schema("public");
        // schema() does not check for duplicates
        assert_eq!(builder.schemas, vec!["public", "public"]);
    }

    #[test]
    fn test_try_from_missing_connection() {
        let builder = PgDieselDatabaseBuilder::default();
        let result: Result<PgDieselDatabase, _> = builder.try_into();
        match result {
            Err(PgDatabaseBuildError::MissingAttribute(attr)) => assert_eq!(attr, "connection"),
            _ => panic!("Unexpected result: {result:?}"),
        }
    }
}
