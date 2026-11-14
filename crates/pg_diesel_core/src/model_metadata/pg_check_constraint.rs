//! Submodule providing [`PgCheckConstraint`], the check constraint a table holds.

use std::sync::Arc;

use diesel::{
    ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl,
    dsl::sql,
    sql_types::{Nullable, Text},
};
use sql_traits::{
    structs::{
        IdentifierCase, TargetName,
        metadata::{CheckMetadata, ExpressionCalls},
    },
    utils::columns_in_expression,
};
use sqlparser::{ast::Expr, dialect::PostgreSqlDialect, parser::Parser};

use crate::model_metadata::{PgFunction, PgTable, TableMetadata};

/// A check constraint as `pg_constraint` records it for its table, with the
/// expression `pg_get_expr` prints for the row's `conbin`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgCheckConstraint {
    /// The name the constraint is recorded under.
    name: String,
    /// The table the constraint holds.
    table: Arc<PgTable>,
    /// The expression the constraint enforces, as `PostgreSQL` prints it.
    expression: Expr,
    /// Whether the rows the table held when the check arrived have been
    /// checked against it.
    validated: bool,
    /// Whether the constraint stops where the table's inheritance ends.
    no_inherit: bool,
}

impl PgCheckConstraint {
    /// Loads the check constraints `pg_constraint` records for the table
    /// `oid`, each holding the expression `pg_get_expr` prints for it.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn load(
        table: &Arc<PgTable>,
        oid: u32,
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::shared::schema::pg_catalog::pg_constraint;

        let rows = pg_constraint::table
            .filter(pg_constraint::conrelid.eq(oid))
            .filter(pg_constraint::contype.eq("c"))
            .select((
                pg_constraint::conname,
                sql::<Nullable<Text>>("pg_get_expr(conbin, conrelid)"),
                pg_constraint::convalidated,
                pg_constraint::connoinherit,
            ))
            .load::<(String, Option<String>, bool, bool)>(conn)?;

        let mut checks = Vec::with_capacity(rows.len());
        for (name, printed, validated, no_inherit) in rows {
            let printed = printed.expect("pg_get_expr prints a check constraint expression");
            let expression = Parser::new(&PostgreSqlDialect {})
                .try_with_sql(&printed)
                .expect("the catalog prints a readable expression")
                .parse_expr()
                .expect("the catalog prints a check constraint expression");
            checks.push(Self {
                name,
                table: Arc::clone(table),
                expression,
                validated,
                no_inherit,
            });
        }
        Ok(checks)
    }

    /// Returns the metadata the database holds for the constraint, resolving
    /// the columns and calls the expression names.
    ///
    /// The calls the expression makes resolve against `functions` under
    /// `search_path`, the path in force where the expression was written.
    #[must_use]
    pub fn metadata(
        &self,
        table_metadata: &TableMetadata,
        functions: &[Arc<PgFunction>],
        search_path: &[(&str, bool)],
    ) -> CheckMetadata<PgCheckConstraint> {
        let target =
            TargetName::new(self.table.name(), true).with_schema(self.table.schema(), true);
        let columns = columns_in_expression(
            &self.expression,
            self.table.catalog(),
            &target,
            table_metadata.column_arc_slice(),
        )
        .expect("the check expression names only its table's columns");
        let calls = ExpressionCalls::resolve(
            &self.expression,
            functions,
            search_path,
            IdentifierCase::AsWritten,
        );
        CheckMetadata::new(
            self.expression.clone(),
            Arc::clone(&self.table),
            columns,
            calls,
        )
    }

    /// Returns the name the constraint is recorded under.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the table the constraint holds.
    #[must_use]
    pub fn table(&self) -> &PgTable {
        &self.table
    }

    /// Returns the expression the constraint enforces, as `PostgreSQL` prints
    /// it.
    #[must_use]
    pub fn expression(&self) -> &Expr {
        &self.expression
    }

    /// Returns whether the rows the table held when the check arrived have
    /// been checked against it.
    #[must_use]
    pub fn is_validated(&self) -> bool {
        self.validated
    }

    /// Returns whether the constraint stops where the table's inheritance
    /// ends.
    #[must_use]
    pub fn no_inherit(&self) -> bool {
        self.no_inherit
    }
}
