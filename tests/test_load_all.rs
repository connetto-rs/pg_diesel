//! Test loading all supported tables

mod test_utils;

use diesel::{PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use test_utils::{COMPILED_MAJORS, establish_connection, reference_docker};

/// Expands `$body` once per compiled `PostgreSQL` version, with `$catalog`
/// naming that version's `pg_diesel::vN`.
macro_rules! for_each_catalog {
    ($catalog:ident => $body:block) => {
        #[cfg(feature = "postgres-14")]
        {
            use pg_diesel::v14 as $catalog;
            $body
        }
        #[cfg(feature = "postgres-15")]
        {
            use pg_diesel::v15 as $catalog;
            $body
        }
        #[cfg(feature = "postgres-16")]
        {
            use pg_diesel::v16 as $catalog;
            $body
        }
        #[cfg(feature = "postgres-17")]
        {
            use pg_diesel::v17 as $catalog;
            $body
        }
        #[cfg(feature = "postgres-18")]
        {
            use pg_diesel::v18 as $catalog;
            $body
        }
    };
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_load_all_models() {
    let mut loaded = Vec::new();
    for_each_catalog!(catalog => {
    let database_name = "test_load_all";
    let docker = reference_docker(catalog::MAJOR, database_name)
        .await
        .expect("Failed to start docker");
    let mut conn: PgConnection = establish_connection(docker.port, database_name)
        .expect("Failed to establish connection to database");
    loaded.push(catalog::MAJOR);

    // information_schema
    {
        use catalog::schema::information_schema::administrable_role_authorizations::administrable_role_authorizations::dsl::*;
        use catalog::models::AdministrableRoleAuthorizations;
        let _ = administrable_role_authorizations
            .select(AdministrableRoleAuthorizations::as_select())
            .load::<AdministrableRoleAuthorizations>(&mut conn);
    }
    {
        use catalog::models::ApplicableRoles;
        use catalog::schema::information_schema::applicable_roles::applicable_roles::dsl::*;
        let _ = applicable_roles
            .select(ApplicableRoles::as_select())
            .load::<ApplicableRoles>(&mut conn);
    }
    {
        use catalog::models::Attribute;
        use catalog::schema::information_schema::attributes::attributes::dsl::*;
        let _ = attributes
            .select(Attribute::as_select())
            .load::<Attribute>(&mut conn);
    }
    {
        use catalog::models::CharacterSet;
        use catalog::schema::information_schema::character_sets::character_sets::dsl::*;
        let _ = character_sets
            .select(CharacterSet::as_select())
            .load::<CharacterSet>(&mut conn);
    }
    {
        use catalog::models::CheckConstraint;
        use catalog::schema::information_schema::check_constraints::check_constraints::dsl::*;
        let _ = check_constraints
            .select(CheckConstraint::as_select())
            .load::<CheckConstraint>(&mut conn);
    }
    {
        use catalog::schema::information_schema::check_constraint_routine_usage::check_constraint_routine_usage::dsl::*;
        use catalog::models::CheckConstraintRoutineUsage;
        let _ = check_constraint_routine_usage
            .select(CheckConstraintRoutineUsage::as_select())
            .load::<CheckConstraintRoutineUsage>(&mut conn);
    }
    {
        use catalog::models::Collation;
        use catalog::schema::information_schema::collations::collations::dsl::*;
        let _ = collations
            .select(Collation::as_select())
            .load::<Collation>(&mut conn);
    }
    {
        use catalog::schema::information_schema::collation_character_set_applicability::collation_character_set_applicability::dsl::*;
        use catalog::models::CollationCharacterSetApplicability;
        let _ = collation_character_set_applicability
            .select(CollationCharacterSetApplicability::as_select())
            .load::<CollationCharacterSetApplicability>(&mut conn);
    }
    {
        use catalog::models::Column;
        use catalog::schema::information_schema::columns::columns::dsl::*;
        let _ = columns
            .select(Column::as_select())
            .load::<Column>(&mut conn);
    }
    {
        use catalog::models::ColumnColumnUsage;
        use catalog::schema::information_schema::column_column_usage::column_column_usage::dsl::*;
        let _ = column_column_usage
            .select(ColumnColumnUsage::as_select())
            .load::<ColumnColumnUsage>(&mut conn);
    }
    {
        use catalog::models::ColumnDomainUsage;
        use catalog::schema::information_schema::column_domain_usage::column_domain_usage::dsl::*;
        let _ = column_domain_usage
            .select(ColumnDomainUsage::as_select())
            .load::<ColumnDomainUsage>(&mut conn);
    }
    {
        use catalog::models::ColumnOptions;
        use catalog::schema::information_schema::column_options::column_options::dsl::*;
        let _ = column_options
            .select(ColumnOptions::as_select())
            .load::<ColumnOptions>(&mut conn);
    }
    {
        use catalog::models::ColumnPrivilege;
        use catalog::schema::information_schema::column_privileges::column_privileges::dsl::*;
        let _ = column_privileges
            .select(ColumnPrivilege::as_select())
            .load::<ColumnPrivilege>(&mut conn);
    }
    {
        use catalog::models::ColumnUdtUsage;
        use catalog::schema::information_schema::column_udt_usage::column_udt_usage::dsl::*;
        let _ = column_udt_usage
            .select(ColumnUdtUsage::as_select())
            .load::<ColumnUdtUsage>(&mut conn);
    }
    {
        use catalog::schema::information_schema::constraint_column_usage::constraint_column_usage::dsl::*;
        use catalog::models::ConstraintColumnUsage;
        let _ = constraint_column_usage
            .select(ConstraintColumnUsage::as_select())
            .load::<ConstraintColumnUsage>(&mut conn);
    }
    {
        use catalog::schema::information_schema::constraint_table_usage::constraint_table_usage::dsl::*;
        use catalog::models::ConstraintTableUsage;
        let _ = constraint_table_usage
            .select(ConstraintTableUsage::as_select())
            .load::<ConstraintTableUsage>(&mut conn);
    }
    {
        use catalog::models::DataTypePrivilege;
        use catalog::schema::information_schema::data_type_privileges::data_type_privileges::dsl::*;
        let _ = data_type_privileges
            .select(DataTypePrivilege::as_select())
            .load::<DataTypePrivilege>(&mut conn);
    }
    {
        use catalog::models::Domain;
        use catalog::schema::information_schema::domains::domains::dsl::*;
        let _ = domains
            .select(Domain::as_select())
            .load::<Domain>(&mut conn);
    }
    {
        use catalog::models::DomainConstraint;
        use catalog::schema::information_schema::domain_constraints::domain_constraints::dsl::*;
        let _ = domain_constraints
            .select(DomainConstraint::as_select())
            .load::<DomainConstraint>(&mut conn);
    }
    {
        use catalog::models::DomainUdtUsage;
        use catalog::schema::information_schema::domain_udt_usage::domain_udt_usage::dsl::*;
        let _ = domain_udt_usage
            .select(DomainUdtUsage::as_select())
            .load::<DomainUdtUsage>(&mut conn);
    }
    {
        use catalog::models::ElementTypes;
        use catalog::schema::information_schema::element_types::element_types::dsl::*;
        let _ = element_types
            .select(ElementTypes::as_select())
            .load::<ElementTypes>(&mut conn);
    }
    {
        use catalog::models::EnabledRoles;
        use catalog::schema::information_schema::enabled_roles::enabled_roles::dsl::*;
        let _ = enabled_roles
            .select(EnabledRoles::as_select())
            .load::<EnabledRoles>(&mut conn);
    }
    {
        use catalog::schema::information_schema::foreign_data_wrapper_options::foreign_data_wrapper_options::dsl::*;
        use catalog::models::ForeignDataWrapperOptions;
        let _ = foreign_data_wrapper_options
            .select(ForeignDataWrapperOptions::as_select())
            .load::<ForeignDataWrapperOptions>(&mut conn);
    }
    {
        use catalog::schema::information_schema::foreign_data_wrappers::foreign_data_wrappers::dsl::*;
        use catalog::models::ForeignDataWrappers;
        let _ = foreign_data_wrappers
            .select(ForeignDataWrappers::as_select())
            .load::<ForeignDataWrappers>(&mut conn);
    }
    {
        use catalog::schema::information_schema::foreign_server_options::foreign_server_options::dsl::*;
        use catalog::models::ForeignServerOptions;
        let _ = foreign_server_options
            .select(ForeignServerOptions::as_select())
            .load::<ForeignServerOptions>(&mut conn);
    }
    {
        use catalog::models::ForeignServers;
        use catalog::schema::information_schema::foreign_servers::foreign_servers::dsl::*;
        let _ = foreign_servers
            .select(ForeignServers::as_select())
            .load::<ForeignServers>(&mut conn);
    }
    {
        use catalog::schema::information_schema::foreign_table_options::foreign_table_options::dsl::*;
        use catalog::models::ForeignTableOptions;
        let _ = foreign_table_options
            .select(ForeignTableOptions::as_select())
            .load::<ForeignTableOptions>(&mut conn);
    }
    {
        use catalog::models::ForeignTables;
        use catalog::schema::information_schema::foreign_tables::foreign_tables::dsl::*;
        let _ = foreign_tables
            .select(ForeignTables::as_select())
            .load::<ForeignTables>(&mut conn);
    }
    {
        use catalog::schema::information_schema::information_schema_catalog_name::information_schema_catalog_name::dsl::*;
        use catalog::models::InformationSchemaCatalogName;
        let _ = information_schema_catalog_name
            .select(InformationSchemaCatalogName::as_select())
            .load::<InformationSchemaCatalogName>(&mut conn);
    }
    {
        use catalog::models::KeyColumnUsage;
        use catalog::schema::information_schema::key_column_usage::key_column_usage::dsl::*;
        let _ = key_column_usage
            .select(KeyColumnUsage::as_select())
            .load::<KeyColumnUsage>(&mut conn);
    }
    {
        use catalog::models::Parameters;
        use catalog::schema::information_schema::parameters::parameters::dsl::*;
        let _ = parameters
            .select(Parameters::as_select())
            .load::<Parameters>(&mut conn);
    }
    {
        use catalog::schema::information_schema::referential_constraints::referential_constraints::dsl::*;
        use catalog::models::ReferentialConstraint;
        let _ = referential_constraints
            .select(ReferentialConstraint::as_select())
            .load::<ReferentialConstraint>(&mut conn);
    }
    {
        use catalog::models::RoleColumnGrants;
        use catalog::schema::information_schema::role_column_grants::role_column_grants::dsl::*;
        let _ = role_column_grants
            .select(RoleColumnGrants::as_select())
            .load::<RoleColumnGrants>(&mut conn);
    }
    {
        use catalog::models::RoleRoutineGrants;
        use catalog::schema::information_schema::role_routine_grants::role_routine_grants::dsl::*;
        let _ = role_routine_grants
            .select(RoleRoutineGrants::as_select())
            .load::<RoleRoutineGrants>(&mut conn);
    }
    {
        use catalog::models::RoleTableGrants;
        use catalog::schema::information_schema::role_table_grants::role_table_grants::dsl::*;
        let _ = role_table_grants
            .select(RoleTableGrants::as_select())
            .load::<RoleTableGrants>(&mut conn);
    }
    {
        use catalog::models::RoleUdtGrants;
        use catalog::schema::information_schema::role_udt_grants::role_udt_grants::dsl::*;
        let _ = role_udt_grants
            .select(RoleUdtGrants::as_select())
            .load::<RoleUdtGrants>(&mut conn);
    }
    {
        use catalog::models::RoleUsageGrants;
        use catalog::schema::information_schema::role_usage_grants::role_usage_grants::dsl::*;
        let _ = role_usage_grants
            .select(RoleUsageGrants::as_select())
            .load::<RoleUsageGrants>(&mut conn);
    }
    {
        use catalog::models::RoutineColumnUsage;
        use catalog::schema::information_schema::routine_column_usage::routine_column_usage::dsl::*;
        let _ = routine_column_usage
            .select(RoutineColumnUsage::as_select())
            .load::<RoutineColumnUsage>(&mut conn);
    }
    {
        use catalog::models::RoutinePrivileges;
        use catalog::schema::information_schema::routine_privileges::routine_privileges::dsl::*;
        let _ = routine_privileges
            .select(RoutinePrivileges::as_select())
            .load::<RoutinePrivileges>(&mut conn);
    }
    {
        use catalog::schema::information_schema::routine_routine_usage::routine_routine_usage::dsl::*;
        use catalog::models::RoutineRoutineUsage;
        let _ = routine_routine_usage
            .select(RoutineRoutineUsage::as_select())
            .load::<RoutineRoutineUsage>(&mut conn);
    }
    {
        use catalog::schema::information_schema::routine_sequence_usage::routine_sequence_usage::dsl::*;
        use catalog::models::RoutineSequenceUsage;
        let _ = routine_sequence_usage
            .select(RoutineSequenceUsage::as_select())
            .load::<RoutineSequenceUsage>(&mut conn);
    }
    {
        use catalog::models::RoutineTableUsage;
        use catalog::schema::information_schema::routine_table_usage::routine_table_usage::dsl::*;
        let _ = routine_table_usage
            .select(RoutineTableUsage::as_select())
            .load::<RoutineTableUsage>(&mut conn);
    }
    #[cfg(feature = "routines")]
    {
        use catalog::models::Routines;
        use catalog::schema::information_schema::routines::routines::dsl::*;
        let _ = routines
            .select(Routines::as_select())
            .load::<Routines>(&mut conn);
    }
    {
        use catalog::models::Schemata;
        use catalog::schema::information_schema::schemata::schemata::dsl::*;
        let _ = schemata
            .select(Schemata::as_select())
            .load::<Schemata>(&mut conn);
    }
    {
        use catalog::models::Sequences;
        use catalog::schema::information_schema::sequences::sequences::dsl::*;
        let _ = sequences
            .select(Sequences::as_select())
            .load::<Sequences>(&mut conn);
    }
    {
        use catalog::models::SqlFeatures;
        use catalog::schema::information_schema::sql_features::sql_features::dsl::*;
        let _ = sql_features
            .select(SqlFeatures::as_select())
            .load::<SqlFeatures>(&mut conn);
    }
    {
        use catalog::schema::information_schema::sql_implementation_info::sql_implementation_info::dsl::*;
        use catalog::models::SqlImplementationInfo;
        let _ = sql_implementation_info
            .select(SqlImplementationInfo::as_select())
            .load::<SqlImplementationInfo>(&mut conn);
    }
    {
        use catalog::models::SqlParts;
        use catalog::schema::information_schema::sql_parts::sql_parts::dsl::*;
        let _ = sql_parts
            .select(SqlParts::as_select())
            .load::<SqlParts>(&mut conn);
    }
    {
        use catalog::models::SqlSizing;
        use catalog::schema::information_schema::sql_sizing::sql_sizing::dsl::*;
        let _ = sql_sizing
            .select(SqlSizing::as_select())
            .load::<SqlSizing>(&mut conn);
    }
    {
        use catalog::models::Table;
        use catalog::schema::information_schema::tables::tables::dsl::*;
        let _ = tables.select(Table::as_select()).load::<Table>(&mut conn);
    }
    {
        use catalog::models::TableConstraint;
        use catalog::schema::information_schema::table_constraints::table_constraints::dsl::*;
        let _ = table_constraints
            .select(TableConstraint::as_select())
            .load::<TableConstraint>(&mut conn);
    }
    {
        use catalog::models::TablePrivileges;
        use catalog::schema::information_schema::table_privileges::table_privileges::dsl::*;
        let _ = table_privileges
            .select(TablePrivileges::as_select())
            .load::<TablePrivileges>(&mut conn);
    }
    {
        use catalog::models::Transforms;
        use catalog::schema::information_schema::transforms::transforms::dsl::*;
        let _ = transforms
            .select(Transforms::as_select())
            .load::<Transforms>(&mut conn);
    }
    {
        use catalog::schema::information_schema::triggered_update_columns::triggered_update_columns::dsl::*;
        use catalog::models::TriggeredUpdateColumns;
        let _ = triggered_update_columns
            .select(TriggeredUpdateColumns::as_select())
            .load::<TriggeredUpdateColumns>(&mut conn);
    }
    {
        use catalog::models::Triggers;
        use catalog::schema::information_schema::triggers::triggers::dsl::*;
        let _ = triggers
            .select(Triggers::as_select())
            .load::<Triggers>(&mut conn);
    }
    {
        use catalog::models::UdtPrivileges;
        use catalog::schema::information_schema::udt_privileges::udt_privileges::dsl::*;
        let _ = udt_privileges
            .select(UdtPrivileges::as_select())
            .load::<UdtPrivileges>(&mut conn);
    }
    {
        use catalog::models::UsagePrivileges;
        use catalog::schema::information_schema::usage_privileges::usage_privileges::dsl::*;
        let _ = usage_privileges
            .select(UsagePrivileges::as_select())
            .load::<UsagePrivileges>(&mut conn);
    }
    {
        use catalog::models::UserDefinedTypes;
        use catalog::schema::information_schema::user_defined_types::user_defined_types::dsl::*;
        let _ = user_defined_types
            .select(UserDefinedTypes::as_select())
            .load::<UserDefinedTypes>(&mut conn);
    }
    {
        use catalog::models::UserMappingOptions;
        use catalog::schema::information_schema::user_mapping_options::user_mapping_options::dsl::*;
        let _ = user_mapping_options
            .select(UserMappingOptions::as_select())
            .load::<UserMappingOptions>(&mut conn);
    }
    {
        use catalog::models::UserMappings;
        use catalog::schema::information_schema::user_mappings::user_mappings::dsl::*;
        let _ = user_mappings
            .select(UserMappings::as_select())
            .load::<UserMappings>(&mut conn);
    }
    {
        use catalog::models::ViewColumnUsage;
        use catalog::schema::information_schema::view_column_usage::view_column_usage::dsl::*;
        let _ = view_column_usage
            .select(ViewColumnUsage::as_select())
            .load::<ViewColumnUsage>(&mut conn);
    }
    {
        use catalog::models::ViewRoutineUsage;
        use catalog::schema::information_schema::view_routine_usage::view_routine_usage::dsl::*;
        let _ = view_routine_usage
            .select(ViewRoutineUsage::as_select())
            .load::<ViewRoutineUsage>(&mut conn);
    }
    {
        use catalog::models::ViewTableUsage;
        use catalog::schema::information_schema::view_table_usage::view_table_usage::dsl::*;
        let _ = view_table_usage
            .select(ViewTableUsage::as_select())
            .load::<ViewTableUsage>(&mut conn);
    }
    {
        use catalog::models::Views;
        use catalog::schema::information_schema::views::views::dsl::*;
        let _ = views.select(Views::as_select()).load::<Views>(&mut conn);
    }
    // pg_catalog
    {
        use catalog::models::PgAggregate;
        use catalog::schema::pg_catalog::pg_aggregate::pg_aggregate::dsl::*;
        let _ = pg_aggregate
            .select(PgAggregate::as_select())
            .load::<PgAggregate>(&mut conn);
    }
    {
        use catalog::models::PgAm;
        use catalog::schema::pg_catalog::pg_am::pg_am::dsl::*;
        let _ = pg_am.select(PgAm::as_select()).load::<PgAm>(&mut conn);
    }
    {
        use catalog::models::PgAmop;
        use catalog::schema::pg_catalog::pg_amop::pg_amop::dsl::*;
        let _ = pg_amop
            .select(PgAmop::as_select())
            .load::<PgAmop>(&mut conn);
    }
    {
        use catalog::models::PgAmproc;
        use catalog::schema::pg_catalog::pg_amproc::pg_amproc::dsl::*;
        let _ = pg_amproc
            .select(PgAmproc::as_select())
            .load::<PgAmproc>(&mut conn);
    }
    {
        use catalog::models::PgAttrdef;
        use catalog::schema::pg_catalog::pg_attrdef::pg_attrdef::dsl::*;
        let _ = pg_attrdef
            .select(PgAttrdef::as_select())
            .load::<PgAttrdef>(&mut conn);
    }
    {
        use catalog::models::PgAttribute;
        use catalog::schema::pg_catalog::pg_attribute::pg_attribute::dsl::*;
        let _ = pg_attribute
            .select(PgAttribute::as_select())
            .load::<PgAttribute>(&mut conn);
    }
    {
        use catalog::models::PgAuthMembers;
        use catalog::schema::pg_catalog::pg_auth_members::pg_auth_members::dsl::*;
        let _ = pg_auth_members
            .select(PgAuthMembers::as_select())
            .load::<PgAuthMembers>(&mut conn);
    }
    {
        use catalog::models::PgAuthid;
        use catalog::schema::pg_catalog::pg_authid::pg_authid::dsl::*;
        let _ = pg_authid
            .select(PgAuthid::as_select())
            .load::<PgAuthid>(&mut conn);
    }
    {
        use catalog::schema::pg_catalog::pg_available_extension_versions::pg_available_extension_versions::dsl::*;
        use catalog::models::PgAvailableExtensionVersions;
        let _ = pg_available_extension_versions
            .select(PgAvailableExtensionVersions::as_select())
            .load::<PgAvailableExtensionVersions>(&mut conn);
    }
    {
        use catalog::models::PgAvailableExtensions;
        use catalog::schema::pg_catalog::pg_available_extensions::pg_available_extensions::dsl::*;
        let _ = pg_available_extensions
            .select(PgAvailableExtensions::as_select())
            .load::<PgAvailableExtensions>(&mut conn);
    }
    {
        use catalog::schema::pg_catalog::pg_backend_memory_contexts::pg_backend_memory_contexts::dsl::*;
        use catalog::models::PgBackendMemoryContexts;
        let _ = pg_backend_memory_contexts
            .select(PgBackendMemoryContexts::as_select())
            .load::<PgBackendMemoryContexts>(&mut conn);
    }
    {
        use catalog::models::PgCast;
        use catalog::schema::pg_catalog::pg_cast::pg_cast::dsl::*;
        let _ = pg_cast
            .select(PgCast::as_select())
            .load::<PgCast>(&mut conn);
    }
    {
        use catalog::models::PGClass;
        use catalog::schema::pg_catalog::pg_class::pg_class::dsl::*;
        let _ = pg_class
            .select(PGClass::as_select())
            .load::<PGClass>(&mut conn);
    }
    {
        use catalog::models::PgCollation;
        use catalog::schema::pg_catalog::pg_collation::pg_collation::dsl::*;
        let _ = pg_collation
            .select(PgCollation::as_select())
            .load::<PgCollation>(&mut conn);
    }
    {
        use catalog::models::PgConfig;
        use catalog::schema::pg_catalog::pg_config::pg_config::dsl::*;
        let _ = pg_config
            .select(PgConfig::as_select())
            .load::<PgConfig>(&mut conn);
    }
    {
        use catalog::models::PgConstraint;
        use catalog::schema::pg_catalog::pg_constraint::pg_constraint::dsl::*;
        let _ = pg_constraint
            .select(PgConstraint::as_select())
            .load::<PgConstraint>(&mut conn);
    }
    {
        use catalog::models::PgConversion;
        use catalog::schema::pg_catalog::pg_conversion::pg_conversion::dsl::*;
        let _ = pg_conversion
            .select(PgConversion::as_select())
            .load::<PgConversion>(&mut conn);
    }
    {
        use catalog::models::PgCursor;
        use catalog::schema::pg_catalog::pg_cursors::pg_cursors::dsl::*;
        let _ = pg_cursors
            .select(PgCursor::as_select())
            .load::<PgCursor>(&mut conn);
    }
    {
        use catalog::models::PgDatabase;
        use catalog::schema::pg_catalog::pg_database::pg_database::dsl::*;
        let _ = pg_database
            .select(PgDatabase::as_select())
            .load::<PgDatabase>(&mut conn);
    }
    {
        use catalog::models::PgDbRoleSetting;
        use catalog::schema::pg_catalog::pg_db_role_setting::pg_db_role_setting::dsl::*;
        let _ = pg_db_role_setting
            .select(PgDbRoleSetting::as_select())
            .load::<PgDbRoleSetting>(&mut conn);
    }
    {
        use catalog::models::PgDefaultAcl;
        use catalog::schema::pg_catalog::pg_default_acl::pg_default_acl::dsl::*;
        let _ = pg_default_acl
            .select(PgDefaultAcl::as_select())
            .load::<PgDefaultAcl>(&mut conn);
    }
    {
        use catalog::models::PgDepend;
        use catalog::schema::pg_catalog::pg_depend::pg_depend::dsl::*;
        let _ = pg_depend
            .select(PgDepend::as_select())
            .load::<PgDepend>(&mut conn);
    }
    {
        use catalog::models::PgDescription;
        use catalog::schema::pg_catalog::pg_description::pg_description::dsl::*;
        let _ = pg_description
            .select(PgDescription::as_select())
            .load::<PgDescription>(&mut conn);
    }
    {
        use catalog::models::PgEnum;
        use catalog::schema::pg_catalog::pg_enum::pg_enum::dsl::*;
        let _ = pg_enum
            .select(PgEnum::as_select())
            .load::<PgEnum>(&mut conn);
    }
    {
        use catalog::models::PgEventTrigger;
        use catalog::schema::pg_catalog::pg_event_trigger::pg_event_trigger::dsl::*;
        let _ = pg_event_trigger
            .select(PgEventTrigger::as_select())
            .load::<PgEventTrigger>(&mut conn);
    }
    {
        use catalog::models::PgExtension;
        use catalog::schema::pg_catalog::pg_extension::pg_extension::dsl::*;
        let _ = pg_extension
            .select(PgExtension::as_select())
            .load::<PgExtension>(&mut conn);
    }
    {
        use catalog::models::PgFileSetting;
        use catalog::schema::pg_catalog::pg_file_settings::pg_file_settings::dsl::*;
        let _ = pg_file_settings
            .select(PgFileSetting::as_select())
            .load::<PgFileSetting>(&mut conn);
    }
    {
        use catalog::models::PgForeignDataWrapper;
        use catalog::schema::pg_catalog::pg_foreign_data_wrapper::pg_foreign_data_wrapper::dsl::*;
        let _ = pg_foreign_data_wrapper
            .select(PgForeignDataWrapper::as_select())
            .load::<PgForeignDataWrapper>(&mut conn);
    }
    {
        use catalog::models::PgForeignServer;
        use catalog::schema::pg_catalog::pg_foreign_server::pg_foreign_server::dsl::*;
        let _ = pg_foreign_server
            .select(PgForeignServer::as_select())
            .load::<PgForeignServer>(&mut conn);
    }
    {
        use catalog::models::PgForeignTable;
        use catalog::schema::pg_catalog::pg_foreign_table::pg_foreign_table::dsl::*;
        let _ = pg_foreign_table
            .select(PgForeignTable::as_select())
            .load::<PgForeignTable>(&mut conn);
    }
    {
        use catalog::models::PgGroup;
        use catalog::schema::pg_catalog::pg_group::pg_group::dsl::*;
        let _ = pg_group
            .select(PgGroup::as_select())
            .load::<PgGroup>(&mut conn);
    }
    {
        use catalog::models::PgHbaFileRule;
        use catalog::schema::pg_catalog::pg_hba_file_rules::pg_hba_file_rules::dsl::*;
        let _ = pg_hba_file_rules
            .select(PgHbaFileRule::as_select())
            .load::<PgHbaFileRule>(&mut conn);
    }
    {
        use catalog::models::PgIdentFileMapping;
        use catalog::schema::pg_catalog::pg_ident_file_mappings::pg_ident_file_mappings::dsl::*;
        let _ = pg_ident_file_mappings
            .select(PgIdentFileMapping::as_select())
            .load::<PgIdentFileMapping>(&mut conn);
    }
    {
        use catalog::models::PgIndex;
        use catalog::schema::pg_catalog::pg_index::pg_index::dsl::*;
        let _ = pg_index
            .select(PgIndex::as_select())
            .load::<PgIndex>(&mut conn);
    }
    {
        use catalog::models::PgInherit;
        use catalog::schema::pg_catalog::pg_inherits::pg_inherits::dsl::*;
        let _ = pg_inherits
            .select(PgInherit::as_select())
            .load::<PgInherit>(&mut conn);
    }
    {
        use catalog::models::PgInitPriv;
        use catalog::schema::pg_catalog::pg_init_privs::pg_init_privs::dsl::*;
        let _ = pg_init_privs
            .select(PgInitPriv::as_select())
            .load::<PgInitPriv>(&mut conn);
    }
    {
        use catalog::models::PgLanguage;
        use catalog::schema::pg_catalog::pg_language::pg_language::dsl::*;
        let _ = pg_language
            .select(PgLanguage::as_select())
            .load::<PgLanguage>(&mut conn);
    }
    {
        use catalog::models::PgLargeobject;
        use catalog::schema::pg_catalog::pg_largeobject::pg_largeobject::dsl::*;
        let _ = pg_largeobject
            .select(PgLargeobject::as_select())
            .load::<PgLargeobject>(&mut conn);
    }
    {
        use catalog::models::PgLargeobjectMetadatum;
        use catalog::schema::pg_catalog::pg_largeobject_metadata::pg_largeobject_metadata::dsl::*;
        let _ = pg_largeobject_metadata
            .select(PgLargeobjectMetadatum::as_select())
            .load::<PgLargeobjectMetadatum>(&mut conn);
    }
    {
        use catalog::models::PgLock;
        use catalog::schema::pg_catalog::pg_locks::pg_locks::dsl::*;
        let _ = pg_locks
            .select(PgLock::as_select())
            .load::<PgLock>(&mut conn);
    }
    {
        use catalog::models::PgMatview;
        use catalog::schema::pg_catalog::pg_matviews::pg_matviews::dsl::*;
        let _ = pg_matviews
            .select(PgMatview::as_select())
            .load::<PgMatview>(&mut conn);
    }
    {
        use catalog::models::PgOpclass;
        use catalog::schema::pg_catalog::pg_opclass::pg_opclass::dsl::*;
        let _ = pg_opclass
            .select(PgOpclass::as_select())
            .load::<PgOpclass>(&mut conn);
    }
    {
        use catalog::models::PgOperator;
        use catalog::schema::pg_catalog::pg_operator::pg_operator::dsl::*;
        let _ = pg_operator
            .select(PgOperator::as_select())
            .load::<PgOperator>(&mut conn);
    }
    {
        use catalog::models::PgOpfamily;
        use catalog::schema::pg_catalog::pg_opfamily::pg_opfamily::dsl::*;
        let _ = pg_opfamily
            .select(PgOpfamily::as_select())
            .load::<PgOpfamily>(&mut conn);
    }
    {
        use catalog::models::PgParameterAcl;
        use catalog::schema::pg_catalog::pg_parameter_acl::pg_parameter_acl::dsl::*;
        let _ = pg_parameter_acl
            .select(PgParameterAcl::as_select())
            .load::<PgParameterAcl>(&mut conn);
    }
    {
        use catalog::models::PgPartitionedTable;
        use catalog::schema::pg_catalog::pg_partitioned_table::pg_partitioned_table::dsl::*;
        let _ = pg_partitioned_table
            .select(PgPartitionedTable::as_select())
            .load::<PgPartitionedTable>(&mut conn);
    }
    {
        use catalog::models::PgPolicy;
        use catalog::schema::pg_catalog::pg_policies::pg_policies::dsl::*;
        let _ = pg_policies
            .select(PgPolicy::as_select())
            .load::<PgPolicy>(&mut conn);
    }
    {
        use catalog::models::PgPolicyTable;
        use catalog::schema::pg_catalog::pg_policy::pg_policy::dsl::*;
        let _ = pg_policy
            .select(PgPolicyTable::as_select())
            .load::<PgPolicyTable>(&mut conn);
    }
    {
        use catalog::models::PgPreparedStatement;
        use catalog::schema::pg_catalog::pg_prepared_statements::pg_prepared_statements::dsl::*;
        let _ = pg_prepared_statements
            .select(PgPreparedStatement::as_select())
            .load::<PgPreparedStatement>(&mut conn);
    }
    {
        use catalog::models::PgPreparedXact;
        use catalog::schema::pg_catalog::pg_prepared_xacts::pg_prepared_xacts::dsl::*;
        let _ = pg_prepared_xacts
            .select(PgPreparedXact::as_select())
            .load::<PgPreparedXact>(&mut conn);
    }
    {
        use catalog::models::PgProc;
        use catalog::schema::pg_catalog::pg_proc::pg_proc::dsl::*;
        let _ = pg_proc
            .select(PgProc::as_select())
            .load::<PgProc>(&mut conn);
    }
    {
        use catalog::models::PgPublication;
        use catalog::schema::pg_catalog::pg_publication::pg_publication::dsl::*;
        let _ = pg_publication
            .select(PgPublication::as_select())
            .load::<PgPublication>(&mut conn);
    }
    {
        use catalog::models::PgPublicationNamespace;
        use catalog::schema::pg_catalog::pg_publication_namespace::pg_publication_namespace::dsl::*;
        let _ = pg_publication_namespace
            .select(PgPublicationNamespace::as_select())
            .load::<PgPublicationNamespace>(&mut conn);
    }
    {
        use catalog::models::PgPublicationRel;
        use catalog::schema::pg_catalog::pg_publication_rel::pg_publication_rel::dsl::*;
        let _ = pg_publication_rel
            .select(PgPublicationRel::as_select())
            .load::<PgPublicationRel>(&mut conn);
    }
    {
        use catalog::models::PgPublicationTable;
        use catalog::schema::pg_catalog::pg_publication_tables::pg_publication_tables::dsl::*;
        let _ = pg_publication_tables
            .select(PgPublicationTable::as_select())
            .load::<PgPublicationTable>(&mut conn);
    }
    {
        use catalog::models::PgRange;
        use catalog::schema::pg_catalog::pg_range::pg_range::dsl::*;
        let _ = pg_range
            .select(PgRange::as_select())
            .load::<PgRange>(&mut conn);
    }
    {
        use catalog::models::PgReplicationOrigin;
        use catalog::schema::pg_catalog::pg_replication_origin::pg_replication_origin::dsl::*;
        let _ = pg_replication_origin
            .select(PgReplicationOrigin::as_select())
            .load::<PgReplicationOrigin>(&mut conn);
    }
    {
        use catalog::schema::pg_catalog::pg_replication_origin_status::pg_replication_origin_status::dsl::*;
        use catalog::models::PgReplicationOriginStatus;
        let _ = pg_replication_origin_status
            .select(PgReplicationOriginStatus::as_select())
            .load::<PgReplicationOriginStatus>(&mut conn);
    }
    {
        use catalog::models::PgReplicationSlot;
        use catalog::schema::pg_catalog::pg_replication_slots::pg_replication_slots::dsl::*;
        let _ = pg_replication_slots
            .select(PgReplicationSlot::as_select())
            .load::<PgReplicationSlot>(&mut conn);
    }
    {
        use catalog::models::PgRewrite;
        use catalog::schema::pg_catalog::pg_rewrite::pg_rewrite::dsl::*;
        let _ = pg_rewrite
            .select(PgRewrite::as_select())
            .load::<PgRewrite>(&mut conn);
    }
    {
        use catalog::models::PgRole;
        use catalog::schema::pg_catalog::pg_roles::pg_roles::dsl::*;
        let _ = pg_roles
            .select(PgRole::as_select())
            .load::<PgRole>(&mut conn);
    }
    {
        use catalog::models::PgRule;
        use catalog::schema::pg_catalog::pg_rules::pg_rules::dsl::*;
        let _ = pg_rules
            .select(PgRule::as_select())
            .load::<PgRule>(&mut conn);
    }
    {
        use catalog::models::PgSeclabel;
        use catalog::schema::pg_catalog::pg_seclabel::pg_seclabel::dsl::*;
        let _ = pg_seclabel
            .select(PgSeclabel::as_select())
            .load::<PgSeclabel>(&mut conn);
    }
    {
        use catalog::models::PgSeclabelView;
        use catalog::schema::pg_catalog::pg_seclabels::pg_seclabels::dsl::*;
        let _ = pg_seclabels
            .select(PgSeclabelView::as_select())
            .load::<PgSeclabelView>(&mut conn);
    }
    {
        use catalog::models::PgSequence;
        use catalog::schema::pg_catalog::pg_sequence::pg_sequence::dsl::*;
        let _ = pg_sequence
            .select(PgSequence::as_select())
            .load::<PgSequence>(&mut conn);
    }
    {
        use catalog::models::PgSequenceView;
        use catalog::schema::pg_catalog::pg_sequences::pg_sequences::dsl::*;
        let _ = pg_sequences
            .select(PgSequenceView::as_select())
            .load::<PgSequenceView>(&mut conn);
    }
    {
        use catalog::models::PgSetting;
        use catalog::schema::pg_catalog::pg_settings::pg_settings::dsl::*;
        let _ = pg_settings
            .select(PgSetting::as_select())
            .load::<PgSetting>(&mut conn);
    }
    {
        use catalog::models::PgShadow;
        use catalog::schema::pg_catalog::pg_shadow::pg_shadow::dsl::*;
        let _ = pg_shadow
            .select(PgShadow::as_select())
            .load::<PgShadow>(&mut conn);
    }
    {
        use catalog::models::PgShdepend;
        use catalog::schema::pg_catalog::pg_shdepend::pg_shdepend::dsl::*;
        let _ = pg_shdepend
            .select(PgShdepend::as_select())
            .load::<PgShdepend>(&mut conn);
    }
    {
        use catalog::models::PgShdescription;
        use catalog::schema::pg_catalog::pg_shdescription::pg_shdescription::dsl::*;
        let _ = pg_shdescription
            .select(PgShdescription::as_select())
            .load::<PgShdescription>(&mut conn);
    }
    {
        use catalog::models::PgShmemAllocation;
        use catalog::schema::pg_catalog::pg_shmem_allocations::pg_shmem_allocations::dsl::*;
        let _ = pg_shmem_allocations
            .select(PgShmemAllocation::as_select())
            .load::<PgShmemAllocation>(&mut conn);
    }
    {
        use catalog::models::PgShseclabel;
        use catalog::schema::pg_catalog::pg_shseclabel::pg_shseclabel::dsl::*;
        let _ = pg_shseclabel
            .select(PgShseclabel::as_select())
            .load::<PgShseclabel>(&mut conn);
    }
    {
        use catalog::models::PgStat;
        use catalog::schema::pg_catalog::pg_stats::pg_stats::dsl::*;
        let _ = pg_stats
            .select(PgStat::as_select())
            .load::<PgStat>(&mut conn);
    }
    {
        use catalog::models::PgStatActivity;
        use catalog::schema::pg_catalog::pg_stat_activity::pg_stat_activity::dsl::*;
        let _ = pg_stat_activity
            .select(PgStatActivity::as_select())
            .load::<PgStatActivity>(&mut conn);
    }
    {
        use catalog::models::PgStatAllIndex;
        use catalog::schema::pg_catalog::pg_stat_all_indexes::pg_stat_all_indexes::dsl::*;
        let _ = pg_stat_all_indexes
            .select(PgStatAllIndex::as_select())
            .load::<PgStatAllIndex>(&mut conn);
    }
    {
        use catalog::models::PgStatAllTable;
        use catalog::schema::pg_catalog::pg_stat_all_tables::pg_stat_all_tables::dsl::*;
        let _ = pg_stat_all_tables
            .select(PgStatAllTable::as_select())
            .load::<PgStatAllTable>(&mut conn);
    }
    {
        use catalog::models::PgStatArchiver;
        use catalog::schema::pg_catalog::pg_stat_archiver::pg_stat_archiver::dsl::*;
        let _ = pg_stat_archiver
            .select(PgStatArchiver::as_select())
            .load::<PgStatArchiver>(&mut conn);
    }
    {
        use catalog::models::PgStatBgwriter;
        use catalog::schema::pg_catalog::pg_stat_bgwriter::pg_stat_bgwriter::dsl::*;
        let _ = pg_stat_bgwriter
            .select(PgStatBgwriter::as_select())
            .load::<PgStatBgwriter>(&mut conn);
    }
    {
        use catalog::models::PgStatCheckpointer;
        use catalog::schema::pg_catalog::pg_stat_checkpointer::pg_stat_checkpointer::dsl::*;
        let _ = pg_stat_checkpointer
            .select(PgStatCheckpointer::as_select())
            .load::<PgStatCheckpointer>(&mut conn);
    }
    {
        use catalog::models::PgStatDatabase;
        use catalog::schema::pg_catalog::pg_stat_database::pg_stat_database::dsl::*;
        let _ = pg_stat_database
            .select(PgStatDatabase::as_select())
            .load::<PgStatDatabase>(&mut conn);
    }
    {
        use catalog::schema::pg_catalog::pg_stat_database_conflicts::pg_stat_database_conflicts::dsl::*;
        use catalog::models::PgStatDatabaseConflict;
        let _ = pg_stat_database_conflicts
            .select(PgStatDatabaseConflict::as_select())
            .load::<PgStatDatabaseConflict>(&mut conn);
    }
    {
        use catalog::models::PgStatGssapi;
        use catalog::schema::pg_catalog::pg_stat_gssapi::pg_stat_gssapi::dsl::*;
        let _ = pg_stat_gssapi
            .select(PgStatGssapi::as_select())
            .load::<PgStatGssapi>(&mut conn);
    }
    {
        use catalog::models::PgStatIo;
        use catalog::schema::pg_catalog::pg_stat_io::pg_stat_io::dsl::*;
        let _ = pg_stat_io
            .select(PgStatIo::as_select())
            .load::<PgStatIo>(&mut conn);
    }
    {
        use catalog::models::PgStatProgressAnalyze;
        use catalog::schema::pg_catalog::pg_stat_progress_analyze::pg_stat_progress_analyze::dsl::*;
        let _ = pg_stat_progress_analyze
            .select(PgStatProgressAnalyze::as_select())
            .load::<PgStatProgressAnalyze>(&mut conn);
    }
    {
        use catalog::schema::pg_catalog::pg_stat_progress_basebackup::pg_stat_progress_basebackup::dsl::*;
        use catalog::models::PgStatProgressBasebackup;
        let _ = pg_stat_progress_basebackup
            .select(PgStatProgressBasebackup::as_select())
            .load::<PgStatProgressBasebackup>(&mut conn);
    }
    {
        use catalog::models::PgStatProgressCluster;
        use catalog::schema::pg_catalog::pg_stat_progress_cluster::pg_stat_progress_cluster::dsl::*;
        let _ = pg_stat_progress_cluster
            .select(PgStatProgressCluster::as_select())
            .load::<PgStatProgressCluster>(&mut conn);
    }
    {
        use catalog::models::PgStatProgressCopy;
        use catalog::schema::pg_catalog::pg_stat_progress_copy::pg_stat_progress_copy::dsl::*;
        let _ = pg_stat_progress_copy
            .select(PgStatProgressCopy::as_select())
            .load::<PgStatProgressCopy>(&mut conn);
    }
    {
        use catalog::schema::pg_catalog::pg_stat_progress_create_index::pg_stat_progress_create_index::dsl::*;
        use catalog::models::PgStatProgressCreateIndex;
        let _ = pg_stat_progress_create_index
            .select(PgStatProgressCreateIndex::as_select())
            .load::<PgStatProgressCreateIndex>(&mut conn);
    }
    {
        use catalog::models::PgStatProgressVacuum;
        use catalog::schema::pg_catalog::pg_stat_progress_vacuum::pg_stat_progress_vacuum::dsl::*;
        let _ = pg_stat_progress_vacuum
            .select(PgStatProgressVacuum::as_select())
            .load::<PgStatProgressVacuum>(&mut conn);
    }
    {
        use catalog::schema::pg_catalog::pg_stat_recovery_prefetch::pg_stat_recovery_prefetch::dsl::*;
        use catalog::models::PgStatRecoveryPrefetch;
        let _ = pg_stat_recovery_prefetch
            .select(PgStatRecoveryPrefetch::as_select())
            .load::<PgStatRecoveryPrefetch>(&mut conn);
    }
    {
        use catalog::models::PgStatReplication;
        use catalog::schema::pg_catalog::pg_stat_replication::pg_stat_replication::dsl::*;
        let _ = pg_stat_replication
            .select(PgStatReplication::as_select())
            .load::<PgStatReplication>(&mut conn);
    }
    {
        use catalog::schema::pg_catalog::pg_stat_replication_slots::pg_stat_replication_slots::dsl::*;
        use catalog::models::PgStatReplicationSlot;
        let _ = pg_stat_replication_slots
            .select(PgStatReplicationSlot::as_select())
            .load::<PgStatReplicationSlot>(&mut conn);
    }
    {
        use catalog::models::PgStatSlru;
        use catalog::schema::pg_catalog::pg_stat_slru::pg_stat_slru::dsl::*;
        let _ = pg_stat_slru
            .select(PgStatSlru::as_select())
            .load::<PgStatSlru>(&mut conn);
    }
    {
        use catalog::models::PgStatSsl;
        use catalog::schema::pg_catalog::pg_stat_ssl::pg_stat_ssl::dsl::*;
        let _ = pg_stat_ssl
            .select(PgStatSsl::as_select())
            .load::<PgStatSsl>(&mut conn);
    }
    {
        use catalog::models::PgStatStatement;
        use catalog::schema::pg_catalog::pg_stat_statements::pg_stat_statements::dsl::*;
        let _ = pg_stat_statements
            .select(PgStatStatement::as_select())
            .load::<PgStatStatement>(&mut conn);
    }
    {
        use catalog::models::PgStatSubscription;
        use catalog::schema::pg_catalog::pg_stat_subscription::pg_stat_subscription::dsl::*;
        let _ = pg_stat_subscription
            .select(PgStatSubscription::as_select())
            .load::<PgStatSubscription>(&mut conn);
    }
    {
        use catalog::schema::pg_catalog::pg_stat_subscription_stats::pg_stat_subscription_stats::dsl::*;
        use catalog::models::PgStatSubscriptionStat;
        let _ = pg_stat_subscription_stats
            .select(PgStatSubscriptionStat::as_select())
            .load::<PgStatSubscriptionStat>(&mut conn);
    }
    {
        use catalog::models::PgStatSysIndex;
        use catalog::schema::pg_catalog::pg_stat_sys_indexes::pg_stat_sys_indexes::dsl::*;
        let _ = pg_stat_sys_indexes
            .select(PgStatSysIndex::as_select())
            .load::<PgStatSysIndex>(&mut conn);
    }
    {
        use catalog::models::PgStatSysTable;
        use catalog::schema::pg_catalog::pg_stat_sys_tables::pg_stat_sys_tables::dsl::*;
        let _ = pg_stat_sys_tables
            .select(PgStatSysTable::as_select())
            .load::<PgStatSysTable>(&mut conn);
    }
    {
        use catalog::models::PgStatUserFunction;
        use catalog::schema::pg_catalog::pg_stat_user_functions::pg_stat_user_functions::dsl::*;
        let _ = pg_stat_user_functions
            .select(PgStatUserFunction::as_select())
            .load::<PgStatUserFunction>(&mut conn);
    }
    {
        use catalog::models::PgStatUserIndex;
        use catalog::schema::pg_catalog::pg_stat_user_indexes::pg_stat_user_indexes::dsl::*;
        let _ = pg_stat_user_indexes
            .select(PgStatUserIndex::as_select())
            .load::<PgStatUserIndex>(&mut conn);
    }
    {
        use catalog::models::PgStatUserTable;
        use catalog::schema::pg_catalog::pg_stat_user_tables::pg_stat_user_tables::dsl::*;
        let _ = pg_stat_user_tables
            .select(PgStatUserTable::as_select())
            .load::<PgStatUserTable>(&mut conn);
    }
    {
        use catalog::models::PgStatWal;
        use catalog::schema::pg_catalog::pg_stat_wal::pg_stat_wal::dsl::*;
        let _ = pg_stat_wal
            .select(PgStatWal::as_select())
            .load::<PgStatWal>(&mut conn);
    }
    {
        use catalog::models::PgStatWalReceiver;
        use catalog::schema::pg_catalog::pg_stat_wal_receiver::pg_stat_wal_receiver::dsl::*;
        let _ = pg_stat_wal_receiver
            .select(PgStatWalReceiver::as_select())
            .load::<PgStatWalReceiver>(&mut conn);
    }
    {
        use catalog::models::PgStatXactAllTable;
        use catalog::schema::pg_catalog::pg_stat_xact_all_tables::pg_stat_xact_all_tables::dsl::*;
        let _ = pg_stat_xact_all_tables
            .select(PgStatXactAllTable::as_select())
            .load::<PgStatXactAllTable>(&mut conn);
    }
    {
        use catalog::models::PgStatXactSysTable;
        use catalog::schema::pg_catalog::pg_stat_xact_sys_tables::pg_stat_xact_sys_tables::dsl::*;
        let _ = pg_stat_xact_sys_tables
            .select(PgStatXactSysTable::as_select())
            .load::<PgStatXactSysTable>(&mut conn);
    }
    {
        use catalog::schema::pg_catalog::pg_stat_xact_user_functions::pg_stat_xact_user_functions::dsl::*;
        use catalog::models::PgStatXactUserFunction;
        let _ = pg_stat_xact_user_functions
            .select(PgStatXactUserFunction::as_select())
            .load::<PgStatXactUserFunction>(&mut conn);
    }
    {
        use catalog::models::PgStatXactUserTable;
        use catalog::schema::pg_catalog::pg_stat_xact_user_tables::pg_stat_xact_user_tables::dsl::*;
        let _ = pg_stat_xact_user_tables
            .select(PgStatXactUserTable::as_select())
            .load::<PgStatXactUserTable>(&mut conn);
    }
    {
        use catalog::models::PgStatioAllIndex;
        use catalog::schema::pg_catalog::pg_statio_all_indexes::pg_statio_all_indexes::dsl::*;
        let _ = pg_statio_all_indexes
            .select(PgStatioAllIndex::as_select())
            .load::<PgStatioAllIndex>(&mut conn);
    }
    {
        use catalog::models::PgStatioAllSequence;
        use catalog::schema::pg_catalog::pg_statio_all_sequences::pg_statio_all_sequences::dsl::*;
        let _ = pg_statio_all_sequences
            .select(PgStatioAllSequence::as_select())
            .load::<PgStatioAllSequence>(&mut conn);
    }
    {
        use catalog::models::PgStatioAllTable;
        use catalog::schema::pg_catalog::pg_statio_all_tables::pg_statio_all_tables::dsl::*;
        let _ = pg_statio_all_tables
            .select(PgStatioAllTable::as_select())
            .load::<PgStatioAllTable>(&mut conn);
    }
    {
        use catalog::models::PgStatioSysIndex;
        use catalog::schema::pg_catalog::pg_statio_sys_indexes::pg_statio_sys_indexes::dsl::*;
        let _ = pg_statio_sys_indexes
            .select(PgStatioSysIndex::as_select())
            .load::<PgStatioSysIndex>(&mut conn);
    }
    {
        use catalog::models::PgStatioSysSequence;
        use catalog::schema::pg_catalog::pg_statio_sys_sequences::pg_statio_sys_sequences::dsl::*;
        let _ = pg_statio_sys_sequences
            .select(PgStatioSysSequence::as_select())
            .load::<PgStatioSysSequence>(&mut conn);
    }
    {
        use catalog::models::PgStatioSysTable;
        use catalog::schema::pg_catalog::pg_statio_sys_tables::pg_statio_sys_tables::dsl::*;
        let _ = pg_statio_sys_tables
            .select(PgStatioSysTable::as_select())
            .load::<PgStatioSysTable>(&mut conn);
    }
    {
        use catalog::models::PgStatioUserIndex;
        use catalog::schema::pg_catalog::pg_statio_user_indexes::pg_statio_user_indexes::dsl::*;
        let _ = pg_statio_user_indexes
            .select(PgStatioUserIndex::as_select())
            .load::<PgStatioUserIndex>(&mut conn);
    }
    {
        use catalog::models::PgStatioUserSequence;
        use catalog::schema::pg_catalog::pg_statio_user_sequences::pg_statio_user_sequences::dsl::*;
        let _ = pg_statio_user_sequences
            .select(PgStatioUserSequence::as_select())
            .load::<PgStatioUserSequence>(&mut conn);
    }
    {
        use catalog::models::PgStatioUserTable;
        use catalog::schema::pg_catalog::pg_statio_user_tables::pg_statio_user_tables::dsl::*;
        let _ = pg_statio_user_tables
            .select(PgStatioUserTable::as_select())
            .load::<PgStatioUserTable>(&mut conn);
    }
    {
        use catalog::models::PgStatistic;
        use catalog::schema::pg_catalog::pg_statistic::pg_statistic::dsl::*;
        let _ = pg_statistic
            .select(PgStatistic::as_select())
            .load::<PgStatistic>(&mut conn);
    }
    {
        use catalog::models::PgStatisticExt;
        use catalog::schema::pg_catalog::pg_statistic_ext::pg_statistic_ext::dsl::*;
        let _ = pg_statistic_ext
            .select(PgStatisticExt::as_select())
            .load::<PgStatisticExt>(&mut conn);
    }
    {
        use catalog::models::PgStatisticExtDatum;
        use catalog::schema::pg_catalog::pg_statistic_ext_data::pg_statistic_ext_data::dsl::*;
        let _ = pg_statistic_ext_data
            .select(PgStatisticExtDatum::as_select())
            .load::<PgStatisticExtDatum>(&mut conn);
    }
    {
        use catalog::models::PgStatsExt;
        use catalog::schema::pg_catalog::pg_stats_ext::pg_stats_ext::dsl::*;
        let _ = pg_stats_ext
            .select(PgStatsExt::as_select())
            .load::<PgStatsExt>(&mut conn);
    }
    {
        use catalog::models::PgStatsExtExpr;
        use catalog::schema::pg_catalog::pg_stats_ext_exprs::pg_stats_ext_exprs::dsl::*;
        let _ = pg_stats_ext_exprs
            .select(PgStatsExtExpr::as_select())
            .load::<PgStatsExtExpr>(&mut conn);
    }
    {
        use catalog::models::PgSubscription;
        use catalog::schema::pg_catalog::pg_subscription::pg_subscription::dsl::*;
        let _ = pg_subscription
            .select(PgSubscription::as_select())
            .load::<PgSubscription>(&mut conn);
    }
    {
        use catalog::models::PgSubscriptionRel;
        use catalog::schema::pg_catalog::pg_subscription_rel::pg_subscription_rel::dsl::*;
        let _ = pg_subscription_rel
            .select(PgSubscriptionRel::as_select())
            .load::<PgSubscriptionRel>(&mut conn);
    }
    {
        use catalog::models::PgTable;
        use catalog::schema::pg_catalog::pg_tables::pg_tables::dsl::*;
        let _ = pg_tables
            .select(PgTable::as_select())
            .load::<PgTable>(&mut conn);
    }
    {
        use catalog::models::PgTablespace;
        use catalog::schema::pg_catalog::pg_tablespace::pg_tablespace::dsl::*;
        let _ = pg_tablespace
            .select(PgTablespace::as_select())
            .load::<PgTablespace>(&mut conn);
    }
    {
        use catalog::models::PgTimezoneAbbrev;
        use catalog::schema::pg_catalog::pg_timezone_abbrevs::pg_timezone_abbrevs::dsl::*;
        let _ = pg_timezone_abbrevs
            .select(PgTimezoneAbbrev::as_select())
            .load::<PgTimezoneAbbrev>(&mut conn);
    }
    {
        use catalog::models::PgTimezoneName;
        use catalog::schema::pg_catalog::pg_timezone_names::pg_timezone_names::dsl::*;
        let _ = pg_timezone_names
            .select(PgTimezoneName::as_select())
            .load::<PgTimezoneName>(&mut conn);
    }
    {
        use catalog::models::PgTransform;
        use catalog::schema::pg_catalog::pg_transform::pg_transform::dsl::*;
        let _ = pg_transform
            .select(PgTransform::as_select())
            .load::<PgTransform>(&mut conn);
    }
    {
        use catalog::models::PgTrigger;
        use catalog::schema::pg_catalog::pg_trigger::pg_trigger::dsl::*;
        let _ = pg_trigger
            .select(PgTrigger::as_select())
            .load::<PgTrigger>(&mut conn);
    }
    {
        use catalog::models::PgTsConfig;
        use catalog::schema::pg_catalog::pg_ts_config::pg_ts_config::dsl::*;
        let _ = pg_ts_config
            .select(PgTsConfig::as_select())
            .load::<PgTsConfig>(&mut conn);
    }
    {
        use catalog::models::PgTsConfigMap;
        use catalog::schema::pg_catalog::pg_ts_config_map::pg_ts_config_map::dsl::*;
        let _ = pg_ts_config_map
            .select(PgTsConfigMap::as_select())
            .load::<PgTsConfigMap>(&mut conn);
    }
    {
        use catalog::models::PgTsDict;
        use catalog::schema::pg_catalog::pg_ts_dict::pg_ts_dict::dsl::*;
        let _ = pg_ts_dict
            .select(PgTsDict::as_select())
            .load::<PgTsDict>(&mut conn);
    }
    {
        use catalog::models::PgTsParser;
        use catalog::schema::pg_catalog::pg_ts_parser::pg_ts_parser::dsl::*;
        let _ = pg_ts_parser
            .select(PgTsParser::as_select())
            .load::<PgTsParser>(&mut conn);
    }
    {
        use catalog::models::PgTsTemplate;
        use catalog::schema::pg_catalog::pg_ts_template::pg_ts_template::dsl::*;
        let _ = pg_ts_template
            .select(PgTsTemplate::as_select())
            .load::<PgTsTemplate>(&mut conn);
    }
    {
        use catalog::models::PgType;
        use catalog::schema::pg_catalog::pg_type::pg_type::dsl::*;
        let _ = pg_type
            .select(PgType::as_select())
            .load::<PgType>(&mut conn);
    }
    {
        use catalog::models::PgUser;
        use catalog::schema::pg_catalog::pg_user::pg_user::dsl::*;
        let _ = pg_user
            .select(PgUser::as_select())
            .load::<PgUser>(&mut conn);
    }
    {
        use catalog::models::PgUserMapping;
        use catalog::schema::pg_catalog::pg_user_mapping::pg_user_mapping::dsl::*;
        let _ = pg_user_mapping
            .select(PgUserMapping::as_select())
            .load::<PgUserMapping>(&mut conn);
    }
    {
        use catalog::models::PgUserMappings;
        use catalog::schema::pg_catalog::pg_user_mappings::pg_user_mappings::dsl::*;
        let _ = pg_user_mappings
            .select(PgUserMappings::as_select())
            .load::<PgUserMappings>(&mut conn);
    }
    {
        use catalog::models::PgView;
        use catalog::schema::pg_catalog::pg_views::pg_views::dsl::*;
        let _ = pg_views
            .select(PgView::as_select())
            .load::<PgView>(&mut conn);
    }
    {
        use catalog::models::PgWaitEvent;
        use catalog::schema::pg_catalog::pg_wait_events::pg_wait_events::dsl::*;
        let _ = pg_wait_events
            .select(PgWaitEvent::as_select())
            .load::<PgWaitEvent>(&mut conn);
    }
    // public
    {
        use catalog::models::GeographyColumn;
        use catalog::schema::public::geography_columns::geography_columns::dsl::*;
        let _ = geography_columns
            .select(GeographyColumn::as_select())
            .load::<GeographyColumn>(&mut conn);
    }
    {
        use catalog::models::GeometryColumn;
        use catalog::schema::public::geometry_columns::geometry_columns::dsl::*;
        let _ = geometry_columns
            .select(GeometryColumn::as_select())
            .load::<GeometryColumn>(&mut conn);
    }
    });
    assert_eq!(loaded, COMPILED_MAJORS);
}

/// `pg_aios` exists from `PostgreSQL` 18 on.
#[cfg(feature = "postgres-18")]
#[tokio::test]
async fn test_load_aios() {
    use pg_diesel::v18 as catalog;

    let database_name = "test_load_aios";
    let docker = reference_docker(catalog::MAJOR, database_name)
        .await
        .expect("Failed to start docker");
    let mut conn: PgConnection = establish_connection(docker.port, database_name)
        .expect("Failed to establish connection to database");
    {
        use catalog::models::PgAios;
        use catalog::schema::pg_catalog::pg_aios::pg_aios::dsl::*;
        let _ = pg_aios
            .select(PgAios::as_select())
            .load::<PgAios>(&mut conn);
    }
}
