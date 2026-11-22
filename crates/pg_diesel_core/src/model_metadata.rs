//! Metadata wrappers providing enriched information about database objects.

pub(crate) mod catalog_identity;
mod constraint_deferral;
pub(crate) use constraint_deferral::decode_deferral;
mod pg_cast_entry;
pub use pg_cast_entry::PgCastEntry;
mod table_metadata;
pub use table_metadata::{ReplicaIdentitySetting, TableMetadata, TableMetadataDetails};
mod column_metadata;
pub use column_metadata::ColumnMetadata;
mod pg_check_constraint;
pub use pg_check_constraint::PgCheckConstraint;
mod pg_column;
pub use pg_column::PgColumn;
mod pg_data_type;
pub use pg_data_type::PgDataType;
mod trigger_metadata;
pub use trigger_metadata::{
    TriggerConditionError, TriggerDefinitionError, TriggerMetadata, TriggerRow,
};
mod target_relation;
pub use target_relation::TargetRelation;
mod policy_metadata;
pub use policy_metadata::PolicyMetadata;
mod role_metadata;
pub use role_metadata::RoleMetadata;
mod pg_function;
pub use pg_function::PgFunction;
pub(crate) use pg_function::RETURNED_COLUMN_MODE;
mod pg_operator_entry;
pub use pg_operator_entry::PgOperatorEntry;
mod pg_foreign_key;
pub use pg_foreign_key::PgForeignKey;
mod pg_policy;
pub use pg_policy::PgPolicy;
mod pg_index_entry;
pub(crate) use pg_index_entry::IndexConstraint;
pub use pg_index_entry::{IndexDefinitionError, PgIndexEntry};
mod indexed_relation;
pub use indexed_relation::{IndexedRelation, OnMaterializedView, OnTable};
mod index_metadata;
pub use index_metadata::IndexMetadata;
mod pg_table;
pub use pg_table::{PgTable, PgTableError, storage_option_entries};
mod pg_schema;
pub use pg_schema::PgSchema;
mod view_definition;
pub use view_definition::{PgViewDefinition, ViewDefinitionError};
mod pg_sequence_entry;
pub use pg_sequence_entry::PgSequenceEntry;
mod pg_rule_entry;
pub use pg_rule_entry::{PgRuleEntry, RuleDefinitionError};
mod pg_extension_entry;
pub use pg_extension_entry::PgExtensionEntry;
mod pg_domain_entry;
pub use pg_domain_entry::{DomainDefinitionError, PgDomainEntry};
mod domain_metadata;
pub use domain_metadata::DomainMetadata;
mod pg_role_membership;
pub use pg_role_membership::{MembershipGrant, PgRoleMembership};
mod pg_user_defined_type;
pub use pg_user_defined_type::{PgUserDefinedType, UserDefinedTypeError};

/// Parses the whole of an expression `PostgreSQL` printed, such as
/// `pg_get_expr` output.
///
/// # Errors
///
/// Returns the parser's error when `printed` is no single expression it
/// reads.
pub(crate) fn parse_printed_expression(
    printed: &str,
) -> Result<sqlparser::ast::Expr, sqlparser::parser::ParserError> {
    let mut parser = sqlparser::parser::Parser::new(&sqlparser::dialect::PostgreSqlDialect {})
        .try_with_sql(printed)?;
    let expression = parser.parse_expr()?;
    parser.expect_token(&sqlparser::tokenizer::Token::EOF)?;
    Ok(expression)
}
