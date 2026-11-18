//! Rows read through the shared-column relations of [`super::schema`].

use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, Queryable,
    RunQueryDsl, Selectable, SelectableHelper,
    dsl::sql,
    sql_types::{Nullable, Text},
};
use sqlparser::{ast::Expr, parser::Parser};

use crate::models::Column;

/// The columns of a `pg_index` row the catalog loaders read.
#[derive(Clone, Queryable, Selectable, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[diesel(table_name = super::schema::pg_catalog::pg_index)]
pub(crate) struct PgIndex {
    /// The OID of the index.
    pub(crate) indexrelid: u32,
    /// The OID of the table this index belongs to.
    pub(crate) indrelid: u32,
    /// `true` if the index enforces uniqueness.
    pub(crate) indisunique: bool,
    /// The column numbers of the indexed columns, `0` for an expression,
    /// the key first and the `INCLUDE` list after it.
    pub(crate) indkey: Vec<i16>,
    /// The number of key columns, those `indkey` lists before the `INCLUDE`
    /// list.
    pub(crate) indnkeyatts: i16,
}

impl PgIndex {
    /// Returns the column numbers of the key, `0` for an expression.
    pub(crate) fn key_attributes(&self) -> &[i16] {
        self.indkey.split_at(self.key_count()).0
    }

    /// Returns the column numbers of the `INCLUDE` list, in the order
    /// written.
    pub(crate) fn included_attributes(&self) -> &[i16] {
        self.indkey.split_at(self.key_count()).1
    }

    /// Returns how many entries of `indkey` belong to the key.
    fn key_count(&self) -> usize {
        usize::try_from(self.indnkeyatts)
            .expect("pg_index.indnkeyatts counts columns")
            .min(self.indkey.len())
    }

    /// Returns the expression the index is defined over, its expression
    /// when it has one and the tuple of its columns otherwise.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if a catalog query fails.
    pub(crate) fn index_expression(
        &self,
        conn: &mut PgConnection,
    ) -> Result<Expr, diesel::result::Error> {
        if let Some(expression) = self.expression(conn)? {
            return Ok(expression);
        }
        let expression_string = format!(
            "({})",
            self.columns(conn)?
                .iter()
                .map(|column| column.column_name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        Ok(Parser::new(&sqlparser::dialect::GenericDialect {})
            .try_with_sql(expression_string.as_str())
            .expect("Failed to parse unique constraint expression")
            .parse_expr()
            .expect("No expression found in parsed unique constraint"))
    }

    /// Returns the expression `pg_get_expr` prints for the index, when it is
    /// an expression index.
    fn expression(&self, conn: &mut PgConnection) -> Result<Option<Expr>, diesel::result::Error> {
        use super::schema::pg_catalog::pg_index;

        let Some(expression) = pg_index::table
            .filter(pg_index::indexrelid.eq(self.indexrelid))
            .select(sql::<Nullable<Text>>("pg_get_expr(indexprs, indrelid)"))
            .get_result::<Option<String>>(conn)?
        else {
            return Ok(None);
        };

        Ok(Some(
            Parser::new(&sqlparser::dialect::PostgreSqlDialect {})
                .try_with_sql(expression.as_str())
                .expect("Failed to parse unique constraint expression")
                .parse_expr()
                .expect("No expression found in parsed unique constraint"),
        ))
    }

    /// Returns the key columns of the index, in the order `indkey` lists
    /// them, read from the table the index is defined on.
    fn columns(&self, conn: &mut PgConnection) -> Result<Vec<Column>, diesel::result::Error> {
        use super::schema::pg_catalog::{pg_attribute, pg_class};
        use crate::schema::{
            information_schema::columns::columns, pg_catalog::pg_namespace::pg_namespace,
        };

        let key = self.key_attributes();
        let mut numbered: Vec<(i16, Column)> = pg_attribute::table
            .inner_join(pg_class::table.on(pg_class::oid.eq(pg_attribute::attrelid)))
            .inner_join(pg_namespace::table.on(pg_namespace::oid.eq(pg_class::relnamespace)))
            .inner_join(
                columns::table.on(columns::table_name
                    .eq(pg_class::relname)
                    .and(columns::table_schema.eq(pg_namespace::nspname))
                    .and(columns::column_name.eq(pg_attribute::attname))),
            )
            .filter(
                pg_attribute::attrelid
                    .eq(self.indrelid)
                    .and(pg_attribute::attnum.eq_any(key)),
            )
            .select((pg_attribute::attnum, Column::as_select()))
            .load::<(i16, Column)>(conn)?;
        numbered.sort_by_key(|(number, _)| key.iter().position(|attribute| attribute == number));
        Ok(numbered.into_iter().map(|(_, column)| column).collect())
    }
}
