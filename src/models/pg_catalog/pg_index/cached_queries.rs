//! Submodule defining the cached queries methods used in the [`PgIndex`]
//! struct.

use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl,
    SelectableHelper, prelude::QueryableByName, sql_types::Oid,
};
use sqlparser::{ast::Expr, parser::Parser};

use crate::models::{Column, PgIndex, Table};

/// Returns the columns that make up the index, in the order `indkey` lists
/// them, read from the table the index is defined on.
pub(crate) fn columns(
    index: &PgIndex,
    conn: &mut PgConnection,
) -> Result<Vec<Column>, diesel::result::Error> {
    use crate::schema::{
        information_schema::columns::columns,
        pg_catalog::{pg_attribute::pg_attribute, pg_class::pg_class, pg_namespace::pg_namespace},
    };

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
                .eq(index.indrelid)
                .and(pg_attribute::attnum.eq_any(&index.indkey)),
        )
        .select((pg_attribute::attnum, Column::as_select()))
        .load::<(i16, Column)>(conn)?;
    numbered.sort_by_key(|(number, _)| index.indkey.iter().position(|key| key == number));
    Ok(numbered.into_iter().map(|(_, column)| column).collect())
}

/// Returns the table on which the index is defined.
pub(crate) fn table(
    index: &PgIndex,
    conn: &mut PgConnection,
) -> Result<crate::models::Table, diesel::result::Error> {
    use crate::schema::{information_schema::tables::tables, pg_catalog::pg_class::pg_class};

    pg_class::table
        .inner_join(tables::table.on(tables::table_name.eq(pg_class::relname)))
        .filter(pg_class::oid.eq(index.indrelid))
        .select(Table::as_select())
        .first(conn)
}

/// Represents an index expression.
#[derive(QueryableByName, Debug)]
struct IndexExpr {
    /// The expression string.
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    expr: Option<String>,
}

/// Returns the string expressions for the index, if any.
pub(crate) fn expression(
    index: &PgIndex,
    conn: &mut PgConnection,
) -> Result<Option<Expr>, diesel::result::Error> {
    let Some(expression) = diesel::sql_query(
        r"
        SELECT
            pg_get_expr(i.indexprs, i.indrelid) as expr
        FROM pg_index i
        WHERE i.indexrelid = $1
        ",
    )
    .bind::<Oid, _>(index.indexrelid)
    .get_result::<IndexExpr>(conn)?
    .expr
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
