//! Submodule providing the `ColumnMetadata` struct for a
//! [`PgColumn`](crate::model_metadata::PgColumn) model.

use std::sync::Arc;

use sql_traits::structs::metadata::ExpressionCalls;

use crate::{
    model_metadata::{PgFunction, PgTable},
    models::PgDescription,
};

#[derive(Clone, Debug)]
/// Rich metadata about a `PostgreSQL` table column.
pub struct ColumnMetadata {
    /// The table the column belongs to.
    table: Arc<PgTable>,
    /// The description of the column, if any.
    description: Option<PgDescription>,
    /// Whether the collation the column declares compares deterministically,
    /// or [`None`] when the column declares none.
    collation_is_deterministic: Option<bool>,
    /// The functions each call of the default expression may reach.
    default_calls: ExpressionCalls<PgFunction>,
    /// The functions each call of the generation expression may reach.
    generation_calls: ExpressionCalls<PgFunction>,
}

impl ColumnMetadata {
    /// Creates a new `ColumnMetadata` instance, with no calls resolved.
    #[must_use]
    pub fn new(
        table: Arc<PgTable>,
        description: Option<PgDescription>,
        collation_is_deterministic: Option<bool>,
    ) -> Self {
        Self {
            table,
            description,
            collation_is_deterministic,
            default_calls: ExpressionCalls::default(),
            generation_calls: ExpressionCalls::default(),
        }
    }

    /// Sets the functions the calls of the default and generation
    /// expressions may reach, resolved under the search path in force where
    /// `PostgreSQL` printed them.
    #[must_use]
    pub fn with_calls(
        mut self,
        default_calls: ExpressionCalls<PgFunction>,
        generation_calls: ExpressionCalls<PgFunction>,
    ) -> Self {
        self.default_calls = default_calls;
        self.generation_calls = generation_calls;
        self
    }

    /// Returns the call record of the default expression.
    #[must_use]
    pub fn default_calls(&self) -> &ExpressionCalls<PgFunction> {
        &self.default_calls
    }

    /// Returns the call record of the generation expression.
    #[must_use]
    pub fn generation_calls(&self) -> &ExpressionCalls<PgFunction> {
        &self.generation_calls
    }

    /// Returns whether the collation the column declares is deterministic.
    #[must_use]
    pub fn collation_is_deterministic(&self) -> Option<bool> {
        self.collation_is_deterministic
    }

    /// Returns the table the column belongs to.
    #[must_use]
    pub fn table(&self) -> &PgTable {
        self.table.as_ref()
    }

    /// Returns the description of the column, if any.
    #[must_use]
    pub fn description(&self) -> Option<&PgDescription> {
        self.description.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_table() -> PgTable {
        PgTable::new(
            crate::models::Table {
                table_catalog: "db".to_string(),
                table_schema: "schema".to_string(),
                table_name: "table".to_string(),
                table_type: "BASE TABLE".to_string(),
                self_referencing_column_name: None,
                reference_generation: None,
                user_defined_type_catalog: None,
                user_defined_type_schema: None,
                user_defined_type_name: None,
                is_insertable_into: "YES".to_string(),
                is_typed: "NO".to_string(),
                commit_action: None,
            },
            None,
            &crate::models::TableFacts {
                oid: 1,
                row_security: false,
                forced_row_security: false,
                replica_identity: "d".to_owned(),
                owner: 1,
                is_partition: false,
                unlogged: false,
                reloptions: Vec::new(),
                toast_reloptions: Vec::new(),
                partition_bound: None,
                server_name: None,
                server_options: Vec::new(),
                acl_is_null: true,
            },
        )
        .expect("a table with no partitioning strategy is accepted")
    }

    #[test]
    fn test_column_metadata() {
        let table = Arc::new(dummy_table());
        let description = Some(PgDescription {
            objoid: 1,
            classoid: 1,
            objsubid: 1,
            description: "desc".to_string(),
        });

        let metadata = ColumnMetadata::new(Arc::clone(&table), description, Some(true));

        assert_eq!(metadata.table().name(), "table");
        assert_eq!(metadata.collation_is_deterministic(), Some(true));
        assert_eq!(metadata.description().unwrap().description, "desc");
    }
}
