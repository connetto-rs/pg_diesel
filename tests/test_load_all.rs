//! Test loading all supported tables

mod test_utils;

use diesel::{PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use test_utils::{establish_connection, reference_docker};

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_load_all_models() {
    let database_name = "test_load_all";
    let docker = reference_docker(database_name)
        .await
        .expect("Failed to start docker");
    let mut conn: PgConnection = establish_connection(docker.port, database_name)
        .expect("Failed to establish connection to database");

    // information_schema
    {
        use pg_diesel::schema::information_schema::administrable_role_authorizations::administrable_role_authorizations::dsl::*;
        use pg_diesel::models::AdministrableRoleAuthorizations;
        let _ = administrable_role_authorizations
            .select(AdministrableRoleAuthorizations::as_select())
            .load::<AdministrableRoleAuthorizations>(&mut conn);
    }
    {
        use pg_diesel::models::ApplicableRoles;
        use pg_diesel::schema::information_schema::applicable_roles::applicable_roles::dsl::*;
        let _ = applicable_roles
            .select(ApplicableRoles::as_select())
            .load::<ApplicableRoles>(&mut conn);
    }
    {
        use pg_diesel::models::Attribute;
        use pg_diesel::schema::information_schema::attributes::attributes::dsl::*;
        let _ = attributes
            .select(Attribute::as_select())
            .load::<Attribute>(&mut conn);
    }
    {
        use pg_diesel::models::CharacterSet;
        use pg_diesel::schema::information_schema::character_sets::character_sets::dsl::*;
        let _ = character_sets
            .select(CharacterSet::as_select())
            .load::<CharacterSet>(&mut conn);
    }
    {
        use pg_diesel::models::CheckConstraint;
        use pg_diesel::schema::information_schema::check_constraints::check_constraints::dsl::*;
        let _ = check_constraints
            .select(CheckConstraint::as_select())
            .load::<CheckConstraint>(&mut conn);
    }
    {
        use pg_diesel::schema::information_schema::check_constraint_routine_usage::check_constraint_routine_usage::dsl::*;
        use pg_diesel::models::CheckConstraintRoutineUsage;
        let _ = check_constraint_routine_usage
            .select(CheckConstraintRoutineUsage::as_select())
            .load::<CheckConstraintRoutineUsage>(&mut conn);
    }
    {
        use pg_diesel::models::Collation;
        use pg_diesel::schema::information_schema::collations::collations::dsl::*;
        let _ = collations
            .select(Collation::as_select())
            .load::<Collation>(&mut conn);
    }
    {
        use pg_diesel::schema::information_schema::collation_character_set_applicability::collation_character_set_applicability::dsl::*;
        use pg_diesel::models::CollationCharacterSetApplicability;
        let _ = collation_character_set_applicability
            .select(CollationCharacterSetApplicability::as_select())
            .load::<CollationCharacterSetApplicability>(&mut conn);
    }
    {
        use pg_diesel::models::Column;
        use pg_diesel::schema::information_schema::columns::columns::dsl::*;
        let _ = columns
            .select(Column::as_select())
            .load::<Column>(&mut conn);
    }
    {
        use pg_diesel::models::ColumnColumnUsage;
        use pg_diesel::schema::information_schema::column_column_usage::column_column_usage::dsl::*;
        let _ = column_column_usage
            .select(ColumnColumnUsage::as_select())
            .load::<ColumnColumnUsage>(&mut conn);
    }
    {
        use pg_diesel::models::ColumnDomainUsage;
        use pg_diesel::schema::information_schema::column_domain_usage::column_domain_usage::dsl::*;
        let _ = column_domain_usage
            .select(ColumnDomainUsage::as_select())
            .load::<ColumnDomainUsage>(&mut conn);
    }
    {
        use pg_diesel::models::ColumnOptions;
        use pg_diesel::schema::information_schema::column_options::column_options::dsl::*;
        let _ = column_options
            .select(ColumnOptions::as_select())
            .load::<ColumnOptions>(&mut conn);
    }
    {
        use pg_diesel::models::ColumnPrivilege;
        use pg_diesel::schema::information_schema::column_privileges::column_privileges::dsl::*;
        let _ = column_privileges
            .select(ColumnPrivilege::as_select())
            .load::<ColumnPrivilege>(&mut conn);
    }
    {
        use pg_diesel::models::ColumnUdtUsage;
        use pg_diesel::schema::information_schema::column_udt_usage::column_udt_usage::dsl::*;
        let _ = column_udt_usage
            .select(ColumnUdtUsage::as_select())
            .load::<ColumnUdtUsage>(&mut conn);
    }
    {
        use pg_diesel::schema::information_schema::constraint_column_usage::constraint_column_usage::dsl::*;
        use pg_diesel::models::ConstraintColumnUsage;
        let _ = constraint_column_usage
            .select(ConstraintColumnUsage::as_select())
            .load::<ConstraintColumnUsage>(&mut conn);
    }
    {
        use pg_diesel::schema::information_schema::constraint_table_usage::constraint_table_usage::dsl::*;
        use pg_diesel::models::ConstraintTableUsage;
        let _ = constraint_table_usage
            .select(ConstraintTableUsage::as_select())
            .load::<ConstraintTableUsage>(&mut conn);
    }
    {
        use pg_diesel::schema::information_schema::data_type_privileges::data_type_privileges::dsl::*;
        use pg_diesel::models::DataTypePrivilege;
        let _ = data_type_privileges
            .select(DataTypePrivilege::as_select())
            .load::<DataTypePrivilege>(&mut conn);
    }
    {
        use pg_diesel::models::Domain;
        use pg_diesel::schema::information_schema::domains::domains::dsl::*;
        let _ = domains
            .select(Domain::as_select())
            .load::<Domain>(&mut conn);
    }
    {
        use pg_diesel::models::DomainConstraint;
        use pg_diesel::schema::information_schema::domain_constraints::domain_constraints::dsl::*;
        let _ = domain_constraints
            .select(DomainConstraint::as_select())
            .load::<DomainConstraint>(&mut conn);
    }
    {
        use pg_diesel::models::DomainUdtUsage;
        use pg_diesel::schema::information_schema::domain_udt_usage::domain_udt_usage::dsl::*;
        let _ = domain_udt_usage
            .select(DomainUdtUsage::as_select())
            .load::<DomainUdtUsage>(&mut conn);
    }
    {
        use pg_diesel::models::ElementTypes;
        use pg_diesel::schema::information_schema::element_types::element_types::dsl::*;
        let _ = element_types
            .select(ElementTypes::as_select())
            .load::<ElementTypes>(&mut conn);
    }
    {
        use pg_diesel::models::EnabledRoles;
        use pg_diesel::schema::information_schema::enabled_roles::enabled_roles::dsl::*;
        let _ = enabled_roles
            .select(EnabledRoles::as_select())
            .load::<EnabledRoles>(&mut conn);
    }
    {
        use pg_diesel::schema::information_schema::foreign_data_wrapper_options::foreign_data_wrapper_options::dsl::*;
        use pg_diesel::models::ForeignDataWrapperOptions;
        let _ = foreign_data_wrapper_options
            .select(ForeignDataWrapperOptions::as_select())
            .load::<ForeignDataWrapperOptions>(&mut conn);
    }
    {
        use pg_diesel::schema::information_schema::foreign_data_wrappers::foreign_data_wrappers::dsl::*;
        use pg_diesel::models::ForeignDataWrappers;
        let _ = foreign_data_wrappers
            .select(ForeignDataWrappers::as_select())
            .load::<ForeignDataWrappers>(&mut conn);
    }
    {
        use pg_diesel::schema::information_schema::foreign_server_options::foreign_server_options::dsl::*;
        use pg_diesel::models::ForeignServerOptions;
        let _ = foreign_server_options
            .select(ForeignServerOptions::as_select())
            .load::<ForeignServerOptions>(&mut conn);
    }
    {
        use pg_diesel::models::ForeignServers;
        use pg_diesel::schema::information_schema::foreign_servers::foreign_servers::dsl::*;
        let _ = foreign_servers
            .select(ForeignServers::as_select())
            .load::<ForeignServers>(&mut conn);
    }
    {
        use pg_diesel::schema::information_schema::foreign_table_options::foreign_table_options::dsl::*;
        use pg_diesel::models::ForeignTableOptions;
        let _ = foreign_table_options
            .select(ForeignTableOptions::as_select())
            .load::<ForeignTableOptions>(&mut conn);
    }
    {
        use pg_diesel::models::ForeignTables;
        use pg_diesel::schema::information_schema::foreign_tables::foreign_tables::dsl::*;
        let _ = foreign_tables
            .select(ForeignTables::as_select())
            .load::<ForeignTables>(&mut conn);
    }
    {
        use pg_diesel::schema::information_schema::information_schema_catalog_name::information_schema_catalog_name::dsl::*;
        use pg_diesel::models::InformationSchemaCatalogName;
        let _ = information_schema_catalog_name
            .select(InformationSchemaCatalogName::as_select())
            .load::<InformationSchemaCatalogName>(&mut conn);
    }
    {
        use pg_diesel::models::KeyColumnUsage;
        use pg_diesel::schema::information_schema::key_column_usage::key_column_usage::dsl::*;
        let _ = key_column_usage
            .select(KeyColumnUsage::as_select())
            .load::<KeyColumnUsage>(&mut conn);
    }
    {
        use pg_diesel::models::Parameters;
        use pg_diesel::schema::information_schema::parameters::parameters::dsl::*;
        let _ = parameters
            .select(Parameters::as_select())
            .load::<Parameters>(&mut conn);
    }
    {
        use pg_diesel::schema::information_schema::referential_constraints::referential_constraints::dsl::*;
        use pg_diesel::models::ReferentialConstraint;
        let _ = referential_constraints
            .select(ReferentialConstraint::as_select())
            .load::<ReferentialConstraint>(&mut conn);
    }
    {
        use pg_diesel::models::RoleColumnGrants;
        use pg_diesel::schema::information_schema::role_column_grants::role_column_grants::dsl::*;
        let _ = role_column_grants
            .select(RoleColumnGrants::as_select())
            .load::<RoleColumnGrants>(&mut conn);
    }
    {
        use pg_diesel::models::RoleRoutineGrants;
        use pg_diesel::schema::information_schema::role_routine_grants::role_routine_grants::dsl::*;
        let _ = role_routine_grants
            .select(RoleRoutineGrants::as_select())
            .load::<RoleRoutineGrants>(&mut conn);
    }
    {
        use pg_diesel::models::RoleTableGrants;
        use pg_diesel::schema::information_schema::role_table_grants::role_table_grants::dsl::*;
        let _ = role_table_grants
            .select(RoleTableGrants::as_select())
            .load::<RoleTableGrants>(&mut conn);
    }
    {
        use pg_diesel::models::RoleUdtGrants;
        use pg_diesel::schema::information_schema::role_udt_grants::role_udt_grants::dsl::*;
        let _ = role_udt_grants
            .select(RoleUdtGrants::as_select())
            .load::<RoleUdtGrants>(&mut conn);
    }
    {
        use pg_diesel::models::RoleUsageGrants;
        use pg_diesel::schema::information_schema::role_usage_grants::role_usage_grants::dsl::*;
        let _ = role_usage_grants
            .select(RoleUsageGrants::as_select())
            .load::<RoleUsageGrants>(&mut conn);
    }
    {
        use pg_diesel::schema::information_schema::routine_column_usage::routine_column_usage::dsl::*;
        use pg_diesel::models::RoutineColumnUsage;
        let _ = routine_column_usage
            .select(RoutineColumnUsage::as_select())
            .load::<RoutineColumnUsage>(&mut conn);
    }
    {
        use pg_diesel::models::RoutinePrivileges;
        use pg_diesel::schema::information_schema::routine_privileges::routine_privileges::dsl::*;
        let _ = routine_privileges
            .select(RoutinePrivileges::as_select())
            .load::<RoutinePrivileges>(&mut conn);
    }
    {
        use pg_diesel::schema::information_schema::routine_routine_usage::routine_routine_usage::dsl::*;
        use pg_diesel::models::RoutineRoutineUsage;
        let _ = routine_routine_usage
            .select(RoutineRoutineUsage::as_select())
            .load::<RoutineRoutineUsage>(&mut conn);
    }
    {
        use pg_diesel::schema::information_schema::routine_sequence_usage::routine_sequence_usage::dsl::*;
        use pg_diesel::models::RoutineSequenceUsage;
        let _ = routine_sequence_usage
            .select(RoutineSequenceUsage::as_select())
            .load::<RoutineSequenceUsage>(&mut conn);
    }
    {
        use pg_diesel::models::RoutineTableUsage;
        use pg_diesel::schema::information_schema::routine_table_usage::routine_table_usage::dsl::*;
        let _ = routine_table_usage
            .select(RoutineTableUsage::as_select())
            .load::<RoutineTableUsage>(&mut conn);
    }
    #[cfg(feature = "routines")]
    {
        use pg_diesel::models::Routines;
        use pg_diesel::schema::information_schema::routines::routines::dsl::*;
        let _ = routines
            .select(Routines::as_select())
            .load::<Routines>(&mut conn);
    }
    {
        use pg_diesel::models::Schemata;
        use pg_diesel::schema::information_schema::schemata::schemata::dsl::*;
        let _ = schemata
            .select(Schemata::as_select())
            .load::<Schemata>(&mut conn);
    }
    {
        use pg_diesel::models::Sequences;
        use pg_diesel::schema::information_schema::sequences::sequences::dsl::*;
        let _ = sequences
            .select(Sequences::as_select())
            .load::<Sequences>(&mut conn);
    }
    {
        use pg_diesel::models::SqlFeatures;
        use pg_diesel::schema::information_schema::sql_features::sql_features::dsl::*;
        let _ = sql_features
            .select(SqlFeatures::as_select())
            .load::<SqlFeatures>(&mut conn);
    }
    {
        use pg_diesel::schema::information_schema::sql_implementation_info::sql_implementation_info::dsl::*;
        use pg_diesel::models::SqlImplementationInfo;
        let _ = sql_implementation_info
            .select(SqlImplementationInfo::as_select())
            .load::<SqlImplementationInfo>(&mut conn);
    }
    {
        use pg_diesel::models::SqlParts;
        use pg_diesel::schema::information_schema::sql_parts::sql_parts::dsl::*;
        let _ = sql_parts
            .select(SqlParts::as_select())
            .load::<SqlParts>(&mut conn);
    }
    {
        use pg_diesel::models::SqlSizing;
        use pg_diesel::schema::information_schema::sql_sizing::sql_sizing::dsl::*;
        let _ = sql_sizing
            .select(SqlSizing::as_select())
            .load::<SqlSizing>(&mut conn);
    }
    {
        use pg_diesel::models::Table;
        use pg_diesel::schema::information_schema::tables::tables::dsl::*;
        let _ = tables.select(Table::as_select()).load::<Table>(&mut conn);
    }
    {
        use pg_diesel::models::TableConstraint;
        use pg_diesel::schema::information_schema::table_constraints::table_constraints::dsl::*;
        let _ = table_constraints
            .select(TableConstraint::as_select())
            .load::<TableConstraint>(&mut conn);
    }
    {
        use pg_diesel::models::TablePrivileges;
        use pg_diesel::schema::information_schema::table_privileges::table_privileges::dsl::*;
        let _ = table_privileges
            .select(TablePrivileges::as_select())
            .load::<TablePrivileges>(&mut conn);
    }
    {
        use pg_diesel::models::Transforms;
        use pg_diesel::schema::information_schema::transforms::transforms::dsl::*;
        let _ = transforms
            .select(Transforms::as_select())
            .load::<Transforms>(&mut conn);
    }
    {
        use pg_diesel::schema::information_schema::triggered_update_columns::triggered_update_columns::dsl::*;
        use pg_diesel::models::TriggeredUpdateColumns;
        let _ = triggered_update_columns
            .select(TriggeredUpdateColumns::as_select())
            .load::<TriggeredUpdateColumns>(&mut conn);
    }
    {
        use pg_diesel::models::Triggers;
        use pg_diesel::schema::information_schema::triggers::triggers::dsl::*;
        let _ = triggers
            .select(Triggers::as_select())
            .load::<Triggers>(&mut conn);
    }
    {
        use pg_diesel::models::UdtPrivileges;
        use pg_diesel::schema::information_schema::udt_privileges::udt_privileges::dsl::*;
        let _ = udt_privileges
            .select(UdtPrivileges::as_select())
            .load::<UdtPrivileges>(&mut conn);
    }
    {
        use pg_diesel::models::UsagePrivileges;
        use pg_diesel::schema::information_schema::usage_privileges::usage_privileges::dsl::*;
        let _ = usage_privileges
            .select(UsagePrivileges::as_select())
            .load::<UsagePrivileges>(&mut conn);
    }
    {
        use pg_diesel::models::UserDefinedTypes;
        use pg_diesel::schema::information_schema::user_defined_types::user_defined_types::dsl::*;
        let _ = user_defined_types
            .select(UserDefinedTypes::as_select())
            .load::<UserDefinedTypes>(&mut conn);
    }
    {
        use pg_diesel::schema::information_schema::user_mapping_options::user_mapping_options::dsl::*;
        use pg_diesel::models::UserMappingOptions;
        let _ = user_mapping_options
            .select(UserMappingOptions::as_select())
            .load::<UserMappingOptions>(&mut conn);
    }
    {
        use pg_diesel::models::UserMappings;
        use pg_diesel::schema::information_schema::user_mappings::user_mappings::dsl::*;
        let _ = user_mappings
            .select(UserMappings::as_select())
            .load::<UserMappings>(&mut conn);
    }
    {
        use pg_diesel::models::ViewColumnUsage;
        use pg_diesel::schema::information_schema::view_column_usage::view_column_usage::dsl::*;
        let _ = view_column_usage
            .select(ViewColumnUsage::as_select())
            .load::<ViewColumnUsage>(&mut conn);
    }
    {
        use pg_diesel::models::ViewRoutineUsage;
        use pg_diesel::schema::information_schema::view_routine_usage::view_routine_usage::dsl::*;
        let _ = view_routine_usage
            .select(ViewRoutineUsage::as_select())
            .load::<ViewRoutineUsage>(&mut conn);
    }
    {
        use pg_diesel::models::ViewTableUsage;
        use pg_diesel::schema::information_schema::view_table_usage::view_table_usage::dsl::*;
        let _ = view_table_usage
            .select(ViewTableUsage::as_select())
            .load::<ViewTableUsage>(&mut conn);
    }
    {
        use pg_diesel::models::Views;
        use pg_diesel::schema::information_schema::views::views::dsl::*;
        let _ = views.select(Views::as_select()).load::<Views>(&mut conn);
    }
    // pg_catalog
    {
        use pg_diesel::models::PgAggregate;
        use pg_diesel::schema::pg_catalog::pg_aggregate::pg_aggregate::dsl::*;
        let _ = pg_aggregate
            .select(PgAggregate::as_select())
            .load::<PgAggregate>(&mut conn);
    }
    #[cfg(feature = "postgres-18")]
    {
        use pg_diesel::models::PgAios;
        use pg_diesel::schema::pg_catalog::pg_aios::pg_aios::dsl::*;
        let _ = pg_aios
            .select(PgAios::as_select())
            .load::<PgAios>(&mut conn);
    }
    {
        use pg_diesel::models::PgAm;
        use pg_diesel::schema::pg_catalog::pg_am::pg_am::dsl::*;
        let _ = pg_am.select(PgAm::as_select()).load::<PgAm>(&mut conn);
    }
    {
        use pg_diesel::models::PgAmop;
        use pg_diesel::schema::pg_catalog::pg_amop::pg_amop::dsl::*;
        let _ = pg_amop
            .select(PgAmop::as_select())
            .load::<PgAmop>(&mut conn);
    }
    {
        use pg_diesel::models::PgAmproc;
        use pg_diesel::schema::pg_catalog::pg_amproc::pg_amproc::dsl::*;
        let _ = pg_amproc
            .select(PgAmproc::as_select())
            .load::<PgAmproc>(&mut conn);
    }
    {
        use pg_diesel::models::PgAttrdef;
        use pg_diesel::schema::pg_catalog::pg_attrdef::pg_attrdef::dsl::*;
        let _ = pg_attrdef
            .select(PgAttrdef::as_select())
            .load::<PgAttrdef>(&mut conn);
    }
    {
        use pg_diesel::models::PgAttribute;
        use pg_diesel::schema::pg_catalog::pg_attribute::pg_attribute::dsl::*;
        let _ = pg_attribute
            .select(PgAttribute::as_select())
            .load::<PgAttribute>(&mut conn);
    }
    {
        use pg_diesel::models::PgAuthMembers;
        use pg_diesel::schema::pg_catalog::pg_auth_members::pg_auth_members::dsl::*;
        let _ = pg_auth_members
            .select(PgAuthMembers::as_select())
            .load::<PgAuthMembers>(&mut conn);
    }
    {
        use pg_diesel::models::PgAuthid;
        use pg_diesel::schema::pg_catalog::pg_authid::pg_authid::dsl::*;
        let _ = pg_authid
            .select(PgAuthid::as_select())
            .load::<PgAuthid>(&mut conn);
    }
    {
        use pg_diesel::schema::pg_catalog::pg_available_extension_versions::pg_available_extension_versions::dsl::*;
        use pg_diesel::models::PgAvailableExtensionVersions;
        let _ = pg_available_extension_versions
            .select(PgAvailableExtensionVersions::as_select())
            .load::<PgAvailableExtensionVersions>(&mut conn);
    }
    {
        use pg_diesel::models::PgAvailableExtensions;
        use pg_diesel::schema::pg_catalog::pg_available_extensions::pg_available_extensions::dsl::*;
        let _ = pg_available_extensions
            .select(PgAvailableExtensions::as_select())
            .load::<PgAvailableExtensions>(&mut conn);
    }
    {
        use pg_diesel::schema::pg_catalog::pg_backend_memory_contexts::pg_backend_memory_contexts::dsl::*;
        use pg_diesel::models::PgBackendMemoryContexts;
        let _ = pg_backend_memory_contexts
            .select(PgBackendMemoryContexts::as_select())
            .load::<PgBackendMemoryContexts>(&mut conn);
    }
    {
        use pg_diesel::models::PgCast;
        use pg_diesel::schema::pg_catalog::pg_cast::pg_cast::dsl::*;
        let _ = pg_cast
            .select(PgCast::as_select())
            .load::<PgCast>(&mut conn);
    }
    {
        use pg_diesel::models::PGClass;
        use pg_diesel::schema::pg_catalog::pg_class::pg_class::dsl::*;
        let _ = pg_class
            .select(PGClass::as_select())
            .load::<PGClass>(&mut conn);
    }
    {
        use pg_diesel::models::PgCollation;
        use pg_diesel::schema::pg_catalog::pg_collation::pg_collation::dsl::*;
        let _ = pg_collation
            .select(PgCollation::as_select())
            .load::<PgCollation>(&mut conn);
    }
    {
        use pg_diesel::models::PgConfig;
        use pg_diesel::schema::pg_catalog::pg_config::pg_config::dsl::*;
        let _ = pg_config
            .select(PgConfig::as_select())
            .load::<PgConfig>(&mut conn);
    }
    {
        use pg_diesel::models::PgConstraint;
        use pg_diesel::schema::pg_catalog::pg_constraint::pg_constraint::dsl::*;
        let _ = pg_constraint
            .select(PgConstraint::as_select())
            .load::<PgConstraint>(&mut conn);
    }
    {
        use pg_diesel::models::PgConversion;
        use pg_diesel::schema::pg_catalog::pg_conversion::pg_conversion::dsl::*;
        let _ = pg_conversion
            .select(PgConversion::as_select())
            .load::<PgConversion>(&mut conn);
    }
    {
        use pg_diesel::models::PgCursor;
        use pg_diesel::schema::pg_catalog::pg_cursors::pg_cursors::dsl::*;
        let _ = pg_cursors
            .select(PgCursor::as_select())
            .load::<PgCursor>(&mut conn);
    }
    {
        use pg_diesel::models::PgDatabase;
        use pg_diesel::schema::pg_catalog::pg_database::pg_database::dsl::*;
        let _ = pg_database
            .select(PgDatabase::as_select())
            .load::<PgDatabase>(&mut conn);
    }
    {
        use pg_diesel::models::PgDbRoleSetting;
        use pg_diesel::schema::pg_catalog::pg_db_role_setting::pg_db_role_setting::dsl::*;
        let _ = pg_db_role_setting
            .select(PgDbRoleSetting::as_select())
            .load::<PgDbRoleSetting>(&mut conn);
    }
    {
        use pg_diesel::models::PgDefaultAcl;
        use pg_diesel::schema::pg_catalog::pg_default_acl::pg_default_acl::dsl::*;
        let _ = pg_default_acl
            .select(PgDefaultAcl::as_select())
            .load::<PgDefaultAcl>(&mut conn);
    }
    {
        use pg_diesel::models::PgDepend;
        use pg_diesel::schema::pg_catalog::pg_depend::pg_depend::dsl::*;
        let _ = pg_depend
            .select(PgDepend::as_select())
            .load::<PgDepend>(&mut conn);
    }
    {
        use pg_diesel::models::PgDescription;
        use pg_diesel::schema::pg_catalog::pg_description::pg_description::dsl::*;
        let _ = pg_description
            .select(PgDescription::as_select())
            .load::<PgDescription>(&mut conn);
    }
    {
        use pg_diesel::models::PgEnum;
        use pg_diesel::schema::pg_catalog::pg_enum::pg_enum::dsl::*;
        let _ = pg_enum
            .select(PgEnum::as_select())
            .load::<PgEnum>(&mut conn);
    }
    {
        use pg_diesel::models::PgEventTrigger;
        use pg_diesel::schema::pg_catalog::pg_event_trigger::pg_event_trigger::dsl::*;
        let _ = pg_event_trigger
            .select(PgEventTrigger::as_select())
            .load::<PgEventTrigger>(&mut conn);
    }
    {
        use pg_diesel::models::PgExtension;
        use pg_diesel::schema::pg_catalog::pg_extension::pg_extension::dsl::*;
        let _ = pg_extension
            .select(PgExtension::as_select())
            .load::<PgExtension>(&mut conn);
    }
    {
        use pg_diesel::models::PgFileSetting;
        use pg_diesel::schema::pg_catalog::pg_file_settings::pg_file_settings::dsl::*;
        let _ = pg_file_settings
            .select(PgFileSetting::as_select())
            .load::<PgFileSetting>(&mut conn);
    }
    {
        use pg_diesel::models::PgForeignDataWrapper;
        use pg_diesel::schema::pg_catalog::pg_foreign_data_wrapper::pg_foreign_data_wrapper::dsl::*;
        let _ = pg_foreign_data_wrapper
            .select(PgForeignDataWrapper::as_select())
            .load::<PgForeignDataWrapper>(&mut conn);
    }
    {
        use pg_diesel::models::PgForeignServer;
        use pg_diesel::schema::pg_catalog::pg_foreign_server::pg_foreign_server::dsl::*;
        let _ = pg_foreign_server
            .select(PgForeignServer::as_select())
            .load::<PgForeignServer>(&mut conn);
    }
    {
        use pg_diesel::models::PgForeignTable;
        use pg_diesel::schema::pg_catalog::pg_foreign_table::pg_foreign_table::dsl::*;
        let _ = pg_foreign_table
            .select(PgForeignTable::as_select())
            .load::<PgForeignTable>(&mut conn);
    }
    {
        use pg_diesel::models::PgGroup;
        use pg_diesel::schema::pg_catalog::pg_group::pg_group::dsl::*;
        let _ = pg_group
            .select(PgGroup::as_select())
            .load::<PgGroup>(&mut conn);
    }
    {
        use pg_diesel::models::PgHbaFileRule;
        use pg_diesel::schema::pg_catalog::pg_hba_file_rules::pg_hba_file_rules::dsl::*;
        let _ = pg_hba_file_rules
            .select(PgHbaFileRule::as_select())
            .load::<PgHbaFileRule>(&mut conn);
    }
    {
        use pg_diesel::models::PgIdentFileMapping;
        use pg_diesel::schema::pg_catalog::pg_ident_file_mappings::pg_ident_file_mappings::dsl::*;
        let _ = pg_ident_file_mappings
            .select(PgIdentFileMapping::as_select())
            .load::<PgIdentFileMapping>(&mut conn);
    }
    {
        use pg_diesel::models::PgIndex;
        use pg_diesel::schema::pg_catalog::pg_index::pg_index::dsl::*;
        let _ = pg_index
            .select(PgIndex::as_select())
            .load::<PgIndex>(&mut conn);
    }
    {
        use pg_diesel::models::PgInherit;
        use pg_diesel::schema::pg_catalog::pg_inherits::pg_inherits::dsl::*;
        let _ = pg_inherits
            .select(PgInherit::as_select())
            .load::<PgInherit>(&mut conn);
    }
    {
        use pg_diesel::models::PgInitPriv;
        use pg_diesel::schema::pg_catalog::pg_init_privs::pg_init_privs::dsl::*;
        let _ = pg_init_privs
            .select(PgInitPriv::as_select())
            .load::<PgInitPriv>(&mut conn);
    }
    {
        use pg_diesel::models::PgLanguage;
        use pg_diesel::schema::pg_catalog::pg_language::pg_language::dsl::*;
        let _ = pg_language
            .select(PgLanguage::as_select())
            .load::<PgLanguage>(&mut conn);
    }
    {
        use pg_diesel::models::PgLargeobject;
        use pg_diesel::schema::pg_catalog::pg_largeobject::pg_largeobject::dsl::*;
        let _ = pg_largeobject
            .select(PgLargeobject::as_select())
            .load::<PgLargeobject>(&mut conn);
    }
    {
        use pg_diesel::models::PgLargeobjectMetadatum;
        use pg_diesel::schema::pg_catalog::pg_largeobject_metadata::pg_largeobject_metadata::dsl::*;
        let _ = pg_largeobject_metadata
            .select(PgLargeobjectMetadatum::as_select())
            .load::<PgLargeobjectMetadatum>(&mut conn);
    }
    {
        use pg_diesel::models::PgLock;
        use pg_diesel::schema::pg_catalog::pg_locks::pg_locks::dsl::*;
        let _ = pg_locks
            .select(PgLock::as_select())
            .load::<PgLock>(&mut conn);
    }
    {
        use pg_diesel::models::PgMatview;
        use pg_diesel::schema::pg_catalog::pg_matviews::pg_matviews::dsl::*;
        let _ = pg_matviews
            .select(PgMatview::as_select())
            .load::<PgMatview>(&mut conn);
    }
    {
        use pg_diesel::models::PgOpclass;
        use pg_diesel::schema::pg_catalog::pg_opclass::pg_opclass::dsl::*;
        let _ = pg_opclass
            .select(PgOpclass::as_select())
            .load::<PgOpclass>(&mut conn);
    }
    {
        use pg_diesel::models::PgOperator;
        use pg_diesel::schema::pg_catalog::pg_operator::pg_operator::dsl::*;
        let _ = pg_operator
            .select(PgOperator::as_select())
            .load::<PgOperator>(&mut conn);
    }
    {
        use pg_diesel::models::PgOpfamily;
        use pg_diesel::schema::pg_catalog::pg_opfamily::pg_opfamily::dsl::*;
        let _ = pg_opfamily
            .select(PgOpfamily::as_select())
            .load::<PgOpfamily>(&mut conn);
    }
    {
        use pg_diesel::models::PgParameterAcl;
        use pg_diesel::schema::pg_catalog::pg_parameter_acl::pg_parameter_acl::dsl::*;
        let _ = pg_parameter_acl
            .select(PgParameterAcl::as_select())
            .load::<PgParameterAcl>(&mut conn);
    }
    {
        use pg_diesel::models::PgPartitionedTable;
        use pg_diesel::schema::pg_catalog::pg_partitioned_table::pg_partitioned_table::dsl::*;
        let _ = pg_partitioned_table
            .select(PgPartitionedTable::as_select())
            .load::<PgPartitionedTable>(&mut conn);
    }
    {
        use pg_diesel::models::PgPolicy;
        use pg_diesel::schema::pg_catalog::pg_policies::pg_policies::dsl::*;
        let _ = pg_policies
            .select(PgPolicy::as_select())
            .load::<PgPolicy>(&mut conn);
    }
    {
        use pg_diesel::models::PgPolicyTable;
        use pg_diesel::schema::pg_catalog::pg_policy::pg_policy::dsl::*;
        let _ = pg_policy
            .select(PgPolicyTable::as_select())
            .load::<PgPolicyTable>(&mut conn);
    }
    {
        use pg_diesel::models::PgPreparedStatement;
        use pg_diesel::schema::pg_catalog::pg_prepared_statements::pg_prepared_statements::dsl::*;
        let _ = pg_prepared_statements
            .select(PgPreparedStatement::as_select())
            .load::<PgPreparedStatement>(&mut conn);
    }
    {
        use pg_diesel::models::PgPreparedXact;
        use pg_diesel::schema::pg_catalog::pg_prepared_xacts::pg_prepared_xacts::dsl::*;
        let _ = pg_prepared_xacts
            .select(PgPreparedXact::as_select())
            .load::<PgPreparedXact>(&mut conn);
    }
    {
        use pg_diesel::models::PgProc;
        use pg_diesel::schema::pg_catalog::pg_proc::pg_proc::dsl::*;
        let _ = pg_proc
            .select(PgProc::as_select())
            .load::<PgProc>(&mut conn);
    }
    {
        use pg_diesel::models::PgPublication;
        use pg_diesel::schema::pg_catalog::pg_publication::pg_publication::dsl::*;
        let _ = pg_publication
            .select(PgPublication::as_select())
            .load::<PgPublication>(&mut conn);
    }
    {
        use pg_diesel::schema::pg_catalog::pg_publication_namespace::pg_publication_namespace::dsl::*;
        use pg_diesel::models::PgPublicationNamespace;
        let _ = pg_publication_namespace
            .select(PgPublicationNamespace::as_select())
            .load::<PgPublicationNamespace>(&mut conn);
    }
    {
        use pg_diesel::models::PgPublicationRel;
        use pg_diesel::schema::pg_catalog::pg_publication_rel::pg_publication_rel::dsl::*;
        let _ = pg_publication_rel
            .select(PgPublicationRel::as_select())
            .load::<PgPublicationRel>(&mut conn);
    }
    {
        use pg_diesel::models::PgPublicationTable;
        use pg_diesel::schema::pg_catalog::pg_publication_tables::pg_publication_tables::dsl::*;
        let _ = pg_publication_tables
            .select(PgPublicationTable::as_select())
            .load::<PgPublicationTable>(&mut conn);
    }
    {
        use pg_diesel::models::PgRange;
        use pg_diesel::schema::pg_catalog::pg_range::pg_range::dsl::*;
        let _ = pg_range
            .select(PgRange::as_select())
            .load::<PgRange>(&mut conn);
    }
    {
        use pg_diesel::models::PgReplicationOrigin;
        use pg_diesel::schema::pg_catalog::pg_replication_origin::pg_replication_origin::dsl::*;
        let _ = pg_replication_origin
            .select(PgReplicationOrigin::as_select())
            .load::<PgReplicationOrigin>(&mut conn);
    }
    {
        use pg_diesel::schema::pg_catalog::pg_replication_origin_status::pg_replication_origin_status::dsl::*;
        use pg_diesel::models::PgReplicationOriginStatus;
        let _ = pg_replication_origin_status
            .select(PgReplicationOriginStatus::as_select())
            .load::<PgReplicationOriginStatus>(&mut conn);
    }
    {
        use pg_diesel::models::PgReplicationSlot;
        use pg_diesel::schema::pg_catalog::pg_replication_slots::pg_replication_slots::dsl::*;
        let _ = pg_replication_slots
            .select(PgReplicationSlot::as_select())
            .load::<PgReplicationSlot>(&mut conn);
    }
    {
        use pg_diesel::models::PgRewrite;
        use pg_diesel::schema::pg_catalog::pg_rewrite::pg_rewrite::dsl::*;
        let _ = pg_rewrite
            .select(PgRewrite::as_select())
            .load::<PgRewrite>(&mut conn);
    }
    {
        use pg_diesel::models::PgRole;
        use pg_diesel::schema::pg_catalog::pg_roles::pg_roles::dsl::*;
        let _ = pg_roles
            .select(PgRole::as_select())
            .load::<PgRole>(&mut conn);
    }
    {
        use pg_diesel::models::PgRule;
        use pg_diesel::schema::pg_catalog::pg_rules::pg_rules::dsl::*;
        let _ = pg_rules
            .select(PgRule::as_select())
            .load::<PgRule>(&mut conn);
    }
    {
        use pg_diesel::models::PgSeclabel;
        use pg_diesel::schema::pg_catalog::pg_seclabel::pg_seclabel::dsl::*;
        let _ = pg_seclabel
            .select(PgSeclabel::as_select())
            .load::<PgSeclabel>(&mut conn);
    }
    {
        use pg_diesel::models::PgSeclabelView;
        use pg_diesel::schema::pg_catalog::pg_seclabels::pg_seclabels::dsl::*;
        let _ = pg_seclabels
            .select(PgSeclabelView::as_select())
            .load::<PgSeclabelView>(&mut conn);
    }
    {
        use pg_diesel::models::PgSequence;
        use pg_diesel::schema::pg_catalog::pg_sequence::pg_sequence::dsl::*;
        let _ = pg_sequence
            .select(PgSequence::as_select())
            .load::<PgSequence>(&mut conn);
    }
    {
        use pg_diesel::models::PgSequenceView;
        use pg_diesel::schema::pg_catalog::pg_sequences::pg_sequences::dsl::*;
        let _ = pg_sequences
            .select(PgSequenceView::as_select())
            .load::<PgSequenceView>(&mut conn);
    }
    {
        use pg_diesel::models::PgSetting;
        use pg_diesel::schema::pg_catalog::pg_settings::pg_settings::dsl::*;
        let _ = pg_settings
            .select(PgSetting::as_select())
            .load::<PgSetting>(&mut conn);
    }
    {
        use pg_diesel::models::PgShadow;
        use pg_diesel::schema::pg_catalog::pg_shadow::pg_shadow::dsl::*;
        let _ = pg_shadow
            .select(PgShadow::as_select())
            .load::<PgShadow>(&mut conn);
    }
    {
        use pg_diesel::models::PgShdepend;
        use pg_diesel::schema::pg_catalog::pg_shdepend::pg_shdepend::dsl::*;
        let _ = pg_shdepend
            .select(PgShdepend::as_select())
            .load::<PgShdepend>(&mut conn);
    }
    {
        use pg_diesel::models::PgShdescription;
        use pg_diesel::schema::pg_catalog::pg_shdescription::pg_shdescription::dsl::*;
        let _ = pg_shdescription
            .select(PgShdescription::as_select())
            .load::<PgShdescription>(&mut conn);
    }
    {
        use pg_diesel::models::PgShmemAllocation;
        use pg_diesel::schema::pg_catalog::pg_shmem_allocations::pg_shmem_allocations::dsl::*;
        let _ = pg_shmem_allocations
            .select(PgShmemAllocation::as_select())
            .load::<PgShmemAllocation>(&mut conn);
    }
    {
        use pg_diesel::models::PgShseclabel;
        use pg_diesel::schema::pg_catalog::pg_shseclabel::pg_shseclabel::dsl::*;
        let _ = pg_shseclabel
            .select(PgShseclabel::as_select())
            .load::<PgShseclabel>(&mut conn);
    }
    {
        use pg_diesel::models::PgStat;
        use pg_diesel::schema::pg_catalog::pg_stats::pg_stats::dsl::*;
        let _ = pg_stats
            .select(PgStat::as_select())
            .load::<PgStat>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatActivity;
        use pg_diesel::schema::pg_catalog::pg_stat_activity::pg_stat_activity::dsl::*;
        let _ = pg_stat_activity
            .select(PgStatActivity::as_select())
            .load::<PgStatActivity>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatAllIndex;
        use pg_diesel::schema::pg_catalog::pg_stat_all_indexes::pg_stat_all_indexes::dsl::*;
        let _ = pg_stat_all_indexes
            .select(PgStatAllIndex::as_select())
            .load::<PgStatAllIndex>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatAllTable;
        use pg_diesel::schema::pg_catalog::pg_stat_all_tables::pg_stat_all_tables::dsl::*;
        let _ = pg_stat_all_tables
            .select(PgStatAllTable::as_select())
            .load::<PgStatAllTable>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatArchiver;
        use pg_diesel::schema::pg_catalog::pg_stat_archiver::pg_stat_archiver::dsl::*;
        let _ = pg_stat_archiver
            .select(PgStatArchiver::as_select())
            .load::<PgStatArchiver>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatBgwriter;
        use pg_diesel::schema::pg_catalog::pg_stat_bgwriter::pg_stat_bgwriter::dsl::*;
        let _ = pg_stat_bgwriter
            .select(PgStatBgwriter::as_select())
            .load::<PgStatBgwriter>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatCheckpointer;
        use pg_diesel::schema::pg_catalog::pg_stat_checkpointer::pg_stat_checkpointer::dsl::*;
        let _ = pg_stat_checkpointer
            .select(PgStatCheckpointer::as_select())
            .load::<PgStatCheckpointer>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatDatabase;
        use pg_diesel::schema::pg_catalog::pg_stat_database::pg_stat_database::dsl::*;
        let _ = pg_stat_database
            .select(PgStatDatabase::as_select())
            .load::<PgStatDatabase>(&mut conn);
    }
    {
        use pg_diesel::schema::pg_catalog::pg_stat_database_conflicts::pg_stat_database_conflicts::dsl::*;
        use pg_diesel::models::PgStatDatabaseConflict;
        let _ = pg_stat_database_conflicts
            .select(PgStatDatabaseConflict::as_select())
            .load::<PgStatDatabaseConflict>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatGssapi;
        use pg_diesel::schema::pg_catalog::pg_stat_gssapi::pg_stat_gssapi::dsl::*;
        let _ = pg_stat_gssapi
            .select(PgStatGssapi::as_select())
            .load::<PgStatGssapi>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatIo;
        use pg_diesel::schema::pg_catalog::pg_stat_io::pg_stat_io::dsl::*;
        let _ = pg_stat_io
            .select(PgStatIo::as_select())
            .load::<PgStatIo>(&mut conn);
    }
    {
        use pg_diesel::schema::pg_catalog::pg_stat_progress_analyze::pg_stat_progress_analyze::dsl::*;
        use pg_diesel::models::PgStatProgressAnalyze;
        let _ = pg_stat_progress_analyze
            .select(PgStatProgressAnalyze::as_select())
            .load::<PgStatProgressAnalyze>(&mut conn);
    }
    {
        use pg_diesel::schema::pg_catalog::pg_stat_progress_basebackup::pg_stat_progress_basebackup::dsl::*;
        use pg_diesel::models::PgStatProgressBasebackup;
        let _ = pg_stat_progress_basebackup
            .select(PgStatProgressBasebackup::as_select())
            .load::<PgStatProgressBasebackup>(&mut conn);
    }
    {
        use pg_diesel::schema::pg_catalog::pg_stat_progress_cluster::pg_stat_progress_cluster::dsl::*;
        use pg_diesel::models::PgStatProgressCluster;
        let _ = pg_stat_progress_cluster
            .select(PgStatProgressCluster::as_select())
            .load::<PgStatProgressCluster>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatProgressCopy;
        use pg_diesel::schema::pg_catalog::pg_stat_progress_copy::pg_stat_progress_copy::dsl::*;
        let _ = pg_stat_progress_copy
            .select(PgStatProgressCopy::as_select())
            .load::<PgStatProgressCopy>(&mut conn);
    }
    {
        use pg_diesel::schema::pg_catalog::pg_stat_progress_create_index::pg_stat_progress_create_index::dsl::*;
        use pg_diesel::models::PgStatProgressCreateIndex;
        let _ = pg_stat_progress_create_index
            .select(PgStatProgressCreateIndex::as_select())
            .load::<PgStatProgressCreateIndex>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatProgressVacuum;
        use pg_diesel::schema::pg_catalog::pg_stat_progress_vacuum::pg_stat_progress_vacuum::dsl::*;
        let _ = pg_stat_progress_vacuum
            .select(PgStatProgressVacuum::as_select())
            .load::<PgStatProgressVacuum>(&mut conn);
    }
    {
        use pg_diesel::schema::pg_catalog::pg_stat_recovery_prefetch::pg_stat_recovery_prefetch::dsl::*;
        use pg_diesel::models::PgStatRecoveryPrefetch;
        let _ = pg_stat_recovery_prefetch
            .select(PgStatRecoveryPrefetch::as_select())
            .load::<PgStatRecoveryPrefetch>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatReplication;
        use pg_diesel::schema::pg_catalog::pg_stat_replication::pg_stat_replication::dsl::*;
        let _ = pg_stat_replication
            .select(PgStatReplication::as_select())
            .load::<PgStatReplication>(&mut conn);
    }
    {
        use pg_diesel::schema::pg_catalog::pg_stat_replication_slots::pg_stat_replication_slots::dsl::*;
        use pg_diesel::models::PgStatReplicationSlot;
        let _ = pg_stat_replication_slots
            .select(PgStatReplicationSlot::as_select())
            .load::<PgStatReplicationSlot>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatSlru;
        use pg_diesel::schema::pg_catalog::pg_stat_slru::pg_stat_slru::dsl::*;
        let _ = pg_stat_slru
            .select(PgStatSlru::as_select())
            .load::<PgStatSlru>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatSsl;
        use pg_diesel::schema::pg_catalog::pg_stat_ssl::pg_stat_ssl::dsl::*;
        let _ = pg_stat_ssl
            .select(PgStatSsl::as_select())
            .load::<PgStatSsl>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatStatement;
        use pg_diesel::schema::pg_catalog::pg_stat_statements::pg_stat_statements::dsl::*;
        let _ = pg_stat_statements
            .select(PgStatStatement::as_select())
            .load::<PgStatStatement>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatSubscription;
        use pg_diesel::schema::pg_catalog::pg_stat_subscription::pg_stat_subscription::dsl::*;
        let _ = pg_stat_subscription
            .select(PgStatSubscription::as_select())
            .load::<PgStatSubscription>(&mut conn);
    }
    {
        use pg_diesel::schema::pg_catalog::pg_stat_subscription_stats::pg_stat_subscription_stats::dsl::*;
        use pg_diesel::models::PgStatSubscriptionStat;
        let _ = pg_stat_subscription_stats
            .select(PgStatSubscriptionStat::as_select())
            .load::<PgStatSubscriptionStat>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatSysIndex;
        use pg_diesel::schema::pg_catalog::pg_stat_sys_indexes::pg_stat_sys_indexes::dsl::*;
        let _ = pg_stat_sys_indexes
            .select(PgStatSysIndex::as_select())
            .load::<PgStatSysIndex>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatSysTable;
        use pg_diesel::schema::pg_catalog::pg_stat_sys_tables::pg_stat_sys_tables::dsl::*;
        let _ = pg_stat_sys_tables
            .select(PgStatSysTable::as_select())
            .load::<PgStatSysTable>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatUserFunction;
        use pg_diesel::schema::pg_catalog::pg_stat_user_functions::pg_stat_user_functions::dsl::*;
        let _ = pg_stat_user_functions
            .select(PgStatUserFunction::as_select())
            .load::<PgStatUserFunction>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatUserIndex;
        use pg_diesel::schema::pg_catalog::pg_stat_user_indexes::pg_stat_user_indexes::dsl::*;
        let _ = pg_stat_user_indexes
            .select(PgStatUserIndex::as_select())
            .load::<PgStatUserIndex>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatUserTable;
        use pg_diesel::schema::pg_catalog::pg_stat_user_tables::pg_stat_user_tables::dsl::*;
        let _ = pg_stat_user_tables
            .select(PgStatUserTable::as_select())
            .load::<PgStatUserTable>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatWal;
        use pg_diesel::schema::pg_catalog::pg_stat_wal::pg_stat_wal::dsl::*;
        let _ = pg_stat_wal
            .select(PgStatWal::as_select())
            .load::<PgStatWal>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatWalReceiver;
        use pg_diesel::schema::pg_catalog::pg_stat_wal_receiver::pg_stat_wal_receiver::dsl::*;
        let _ = pg_stat_wal_receiver
            .select(PgStatWalReceiver::as_select())
            .load::<PgStatWalReceiver>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatXactAllTable;
        use pg_diesel::schema::pg_catalog::pg_stat_xact_all_tables::pg_stat_xact_all_tables::dsl::*;
        let _ = pg_stat_xact_all_tables
            .select(PgStatXactAllTable::as_select())
            .load::<PgStatXactAllTable>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatXactSysTable;
        use pg_diesel::schema::pg_catalog::pg_stat_xact_sys_tables::pg_stat_xact_sys_tables::dsl::*;
        let _ = pg_stat_xact_sys_tables
            .select(PgStatXactSysTable::as_select())
            .load::<PgStatXactSysTable>(&mut conn);
    }
    {
        use pg_diesel::schema::pg_catalog::pg_stat_xact_user_functions::pg_stat_xact_user_functions::dsl::*;
        use pg_diesel::models::PgStatXactUserFunction;
        let _ = pg_stat_xact_user_functions
            .select(PgStatXactUserFunction::as_select())
            .load::<PgStatXactUserFunction>(&mut conn);
    }
    {
        use pg_diesel::schema::pg_catalog::pg_stat_xact_user_tables::pg_stat_xact_user_tables::dsl::*;
        use pg_diesel::models::PgStatXactUserTable;
        let _ = pg_stat_xact_user_tables
            .select(PgStatXactUserTable::as_select())
            .load::<PgStatXactUserTable>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatioAllIndex;
        use pg_diesel::schema::pg_catalog::pg_statio_all_indexes::pg_statio_all_indexes::dsl::*;
        let _ = pg_statio_all_indexes
            .select(PgStatioAllIndex::as_select())
            .load::<PgStatioAllIndex>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatioAllSequence;
        use pg_diesel::schema::pg_catalog::pg_statio_all_sequences::pg_statio_all_sequences::dsl::*;
        let _ = pg_statio_all_sequences
            .select(PgStatioAllSequence::as_select())
            .load::<PgStatioAllSequence>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatioAllTable;
        use pg_diesel::schema::pg_catalog::pg_statio_all_tables::pg_statio_all_tables::dsl::*;
        let _ = pg_statio_all_tables
            .select(PgStatioAllTable::as_select())
            .load::<PgStatioAllTable>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatioSysIndex;
        use pg_diesel::schema::pg_catalog::pg_statio_sys_indexes::pg_statio_sys_indexes::dsl::*;
        let _ = pg_statio_sys_indexes
            .select(PgStatioSysIndex::as_select())
            .load::<PgStatioSysIndex>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatioSysSequence;
        use pg_diesel::schema::pg_catalog::pg_statio_sys_sequences::pg_statio_sys_sequences::dsl::*;
        let _ = pg_statio_sys_sequences
            .select(PgStatioSysSequence::as_select())
            .load::<PgStatioSysSequence>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatioSysTable;
        use pg_diesel::schema::pg_catalog::pg_statio_sys_tables::pg_statio_sys_tables::dsl::*;
        let _ = pg_statio_sys_tables
            .select(PgStatioSysTable::as_select())
            .load::<PgStatioSysTable>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatioUserIndex;
        use pg_diesel::schema::pg_catalog::pg_statio_user_indexes::pg_statio_user_indexes::dsl::*;
        let _ = pg_statio_user_indexes
            .select(PgStatioUserIndex::as_select())
            .load::<PgStatioUserIndex>(&mut conn);
    }
    {
        use pg_diesel::schema::pg_catalog::pg_statio_user_sequences::pg_statio_user_sequences::dsl::*;
        use pg_diesel::models::PgStatioUserSequence;
        let _ = pg_statio_user_sequences
            .select(PgStatioUserSequence::as_select())
            .load::<PgStatioUserSequence>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatioUserTable;
        use pg_diesel::schema::pg_catalog::pg_statio_user_tables::pg_statio_user_tables::dsl::*;
        let _ = pg_statio_user_tables
            .select(PgStatioUserTable::as_select())
            .load::<PgStatioUserTable>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatistic;
        use pg_diesel::schema::pg_catalog::pg_statistic::pg_statistic::dsl::*;
        let _ = pg_statistic
            .select(PgStatistic::as_select())
            .load::<PgStatistic>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatisticExt;
        use pg_diesel::schema::pg_catalog::pg_statistic_ext::pg_statistic_ext::dsl::*;
        let _ = pg_statistic_ext
            .select(PgStatisticExt::as_select())
            .load::<PgStatisticExt>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatisticExtDatum;
        use pg_diesel::schema::pg_catalog::pg_statistic_ext_data::pg_statistic_ext_data::dsl::*;
        let _ = pg_statistic_ext_data
            .select(PgStatisticExtDatum::as_select())
            .load::<PgStatisticExtDatum>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatsExt;
        use pg_diesel::schema::pg_catalog::pg_stats_ext::pg_stats_ext::dsl::*;
        let _ = pg_stats_ext
            .select(PgStatsExt::as_select())
            .load::<PgStatsExt>(&mut conn);
    }
    {
        use pg_diesel::models::PgStatsExtExpr;
        use pg_diesel::schema::pg_catalog::pg_stats_ext_exprs::pg_stats_ext_exprs::dsl::*;
        let _ = pg_stats_ext_exprs
            .select(PgStatsExtExpr::as_select())
            .load::<PgStatsExtExpr>(&mut conn);
    }
    {
        use pg_diesel::models::PgSubscription;
        use pg_diesel::schema::pg_catalog::pg_subscription::pg_subscription::dsl::*;
        let _ = pg_subscription
            .select(PgSubscription::as_select())
            .load::<PgSubscription>(&mut conn);
    }
    {
        use pg_diesel::models::PgSubscriptionRel;
        use pg_diesel::schema::pg_catalog::pg_subscription_rel::pg_subscription_rel::dsl::*;
        let _ = pg_subscription_rel
            .select(PgSubscriptionRel::as_select())
            .load::<PgSubscriptionRel>(&mut conn);
    }
    {
        use pg_diesel::models::PgTable;
        use pg_diesel::schema::pg_catalog::pg_tables::pg_tables::dsl::*;
        let _ = pg_tables
            .select(PgTable::as_select())
            .load::<PgTable>(&mut conn);
    }
    {
        use pg_diesel::models::PgTablespace;
        use pg_diesel::schema::pg_catalog::pg_tablespace::pg_tablespace::dsl::*;
        let _ = pg_tablespace
            .select(PgTablespace::as_select())
            .load::<PgTablespace>(&mut conn);
    }
    {
        use pg_diesel::models::PgTimezoneAbbrev;
        use pg_diesel::schema::pg_catalog::pg_timezone_abbrevs::pg_timezone_abbrevs::dsl::*;
        let _ = pg_timezone_abbrevs
            .select(PgTimezoneAbbrev::as_select())
            .load::<PgTimezoneAbbrev>(&mut conn);
    }
    {
        use pg_diesel::models::PgTimezoneName;
        use pg_diesel::schema::pg_catalog::pg_timezone_names::pg_timezone_names::dsl::*;
        let _ = pg_timezone_names
            .select(PgTimezoneName::as_select())
            .load::<PgTimezoneName>(&mut conn);
    }
    {
        use pg_diesel::models::PgTransform;
        use pg_diesel::schema::pg_catalog::pg_transform::pg_transform::dsl::*;
        let _ = pg_transform
            .select(PgTransform::as_select())
            .load::<PgTransform>(&mut conn);
    }
    {
        use pg_diesel::models::PgTrigger;
        use pg_diesel::schema::pg_catalog::pg_trigger::pg_trigger::dsl::*;
        let _ = pg_trigger
            .select(PgTrigger::as_select())
            .load::<PgTrigger>(&mut conn);
    }
    {
        use pg_diesel::models::PgTsConfig;
        use pg_diesel::schema::pg_catalog::pg_ts_config::pg_ts_config::dsl::*;
        let _ = pg_ts_config
            .select(PgTsConfig::as_select())
            .load::<PgTsConfig>(&mut conn);
    }
    {
        use pg_diesel::models::PgTsConfigMap;
        use pg_diesel::schema::pg_catalog::pg_ts_config_map::pg_ts_config_map::dsl::*;
        let _ = pg_ts_config_map
            .select(PgTsConfigMap::as_select())
            .load::<PgTsConfigMap>(&mut conn);
    }
    {
        use pg_diesel::models::PgTsDict;
        use pg_diesel::schema::pg_catalog::pg_ts_dict::pg_ts_dict::dsl::*;
        let _ = pg_ts_dict
            .select(PgTsDict::as_select())
            .load::<PgTsDict>(&mut conn);
    }
    {
        use pg_diesel::models::PgTsParser;
        use pg_diesel::schema::pg_catalog::pg_ts_parser::pg_ts_parser::dsl::*;
        let _ = pg_ts_parser
            .select(PgTsParser::as_select())
            .load::<PgTsParser>(&mut conn);
    }
    {
        use pg_diesel::models::PgTsTemplate;
        use pg_diesel::schema::pg_catalog::pg_ts_template::pg_ts_template::dsl::*;
        let _ = pg_ts_template
            .select(PgTsTemplate::as_select())
            .load::<PgTsTemplate>(&mut conn);
    }
    {
        use pg_diesel::models::PgType;
        use pg_diesel::schema::pg_catalog::pg_type::pg_type::dsl::*;
        let _ = pg_type
            .select(PgType::as_select())
            .load::<PgType>(&mut conn);
    }
    {
        use pg_diesel::models::PgUser;
        use pg_diesel::schema::pg_catalog::pg_user::pg_user::dsl::*;
        let _ = pg_user
            .select(PgUser::as_select())
            .load::<PgUser>(&mut conn);
    }
    {
        use pg_diesel::models::PgUserMapping;
        use pg_diesel::schema::pg_catalog::pg_user_mapping::pg_user_mapping::dsl::*;
        let _ = pg_user_mapping
            .select(PgUserMapping::as_select())
            .load::<PgUserMapping>(&mut conn);
    }
    {
        use pg_diesel::models::PgUserMappings;
        use pg_diesel::schema::pg_catalog::pg_user_mappings::pg_user_mappings::dsl::*;
        let _ = pg_user_mappings
            .select(PgUserMappings::as_select())
            .load::<PgUserMappings>(&mut conn);
    }
    {
        use pg_diesel::models::PgView;
        use pg_diesel::schema::pg_catalog::pg_views::pg_views::dsl::*;
        let _ = pg_views
            .select(PgView::as_select())
            .load::<PgView>(&mut conn);
    }
    {
        use pg_diesel::models::PgWaitEvent;
        use pg_diesel::schema::pg_catalog::pg_wait_events::pg_wait_events::dsl::*;
        let _ = pg_wait_events
            .select(PgWaitEvent::as_select())
            .load::<PgWaitEvent>(&mut conn);
    }
    // public
    {
        use pg_diesel::models::GeographyColumn;
        use pg_diesel::schema::public::geography_columns::geography_columns::dsl::*;
        let _ = geography_columns
            .select(GeographyColumn::as_select())
            .load::<GeographyColumn>(&mut conn);
    }
    {
        use pg_diesel::models::GeometryColumn;
        use pg_diesel::schema::public::geometry_columns::geometry_columns::dsl::*;
        let _ = geometry_columns
            .select(GeometryColumn::as_select())
            .load::<GeometryColumn>(&mut conn);
    }
}
