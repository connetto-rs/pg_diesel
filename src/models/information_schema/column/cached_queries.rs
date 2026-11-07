//! Submodule defining the cached queries methods used in the [`Column`] struct.

use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, OptionalExtension, PgConnection, QueryDsl,
    RunQueryDsl, SelectableHelper,
};

use crate::models::{
    CheckConstraint, Column, GeographyColumn, GeometryColumn, KeyColumnUsage, PgType, Table,
};

/// Returns the foreign keys that reference this column.
pub(super) fn foreign_keys(
    column: &Column,
    conn: &mut PgConnection,
) -> Result<Vec<KeyColumnUsage>, diesel::result::Error> {
    use crate::schema::information_schema::{
        columns::columns, constraint_column_usage::constraint_column_usage,
        key_column_usage::key_column_usage, table_constraints::table_constraints, tables::tables,
    };
    table_constraints::table
            .inner_join(
                key_column_usage::table.on(table_constraints::constraint_name
                    .eq(key_column_usage::constraint_name)
                    .and(
                        table_constraints::constraint_schema
                            .eq(key_column_usage::constraint_schema),
                    )
                    .and(
                        table_constraints::constraint_catalog
                            .eq(key_column_usage::constraint_catalog),
                    )
                    .and(table_constraints::table_name.eq(key_column_usage::table_name))
                    .and(table_constraints::table_schema.eq(key_column_usage::table_schema))
                    .and(table_constraints::table_catalog.eq(key_column_usage::table_catalog))),
            )
            .inner_join(constraint_column_usage::table.on(
                constraint_column_usage::constraint_name.eq(table_constraints::constraint_name),
            ))
            .inner_join(
                tables::table.on(tables::table_name
                    .eq(constraint_column_usage::table_name)
                    .and(tables::table_schema.eq(constraint_column_usage::table_schema))
                    .and(tables::table_catalog.eq(constraint_column_usage::table_catalog))),
            )
            .inner_join(
                columns::table.on(columns::table_name
                    .eq(constraint_column_usage::table_name)
                    .and(columns::table_schema.eq(constraint_column_usage::table_schema))
                    .and(columns::table_catalog.eq(constraint_column_usage::table_catalog))
                    .and(columns::column_name.eq(constraint_column_usage::column_name))),
            )
            .filter(table_constraints::constraint_type.eq("FOREIGN KEY"))
            .filter(table_constraints::table_name.eq(&column.table_name))
            .filter(table_constraints::table_schema.eq(&column.table_schema))
            .filter(table_constraints::table_catalog.eq(&column.table_catalog))
            .filter(key_column_usage::column_name.eq(&column.column_name))
            .order_by(key_column_usage::constraint_name.asc())
            .select(KeyColumnUsage::as_select())
            .distinct()
            .load::<KeyColumnUsage>(conn)
}

/// Returns the check constraints that apply to this column.
///
/// `information_schema.check_constraints` also carries the not-null
/// constraints, so only the rows a `pg_constraint` row of type `c` backs
/// are check constraints.
pub(super) fn check_constraints(
    column: &Column,
    conn: &mut PgConnection,
) -> Result<Vec<CheckConstraint>, diesel::result::Error> {
    use crate::schema::information_schema::{
        check_constraints::check_constraints, constraint_column_usage::constraint_column_usage,
    };
    use crate::schema::pg_catalog::{pg_constraint::pg_constraint, pg_namespace::pg_namespace};

    let constraints = check_constraints::table
        .inner_join(
            constraint_column_usage::table.on(constraint_column_usage::constraint_name
                .eq(check_constraints::constraint_name)
                .and(
                    constraint_column_usage::constraint_catalog
                        .eq(check_constraints::constraint_catalog)
                        .and(
                            constraint_column_usage::constraint_schema
                                .eq(check_constraints::constraint_schema),
                        ),
                )),
        )
        .filter(
            constraint_column_usage::column_name
                .eq(&column.column_name)
                .and(
                    constraint_column_usage::table_catalog
                        .eq(&column.table_catalog)
                        .and(
                            constraint_column_usage::table_schema
                                .eq(&column.table_schema)
                                .and(constraint_column_usage::table_name.eq(&column.table_name)),
                        ),
                ),
        )
        .select(CheckConstraint::as_select())
        .load::<CheckConstraint>(conn)?;

    let backed: Vec<String> = pg_constraint::table
        .inner_join(pg_namespace::table.on(pg_constraint::connamespace.eq(pg_namespace::oid)))
        .filter(pg_namespace::nspname.eq(&column.table_schema))
        .filter(pg_constraint::contype.eq("c"))
        .select(pg_constraint::conname)
        .load::<String>(conn)?;

    Ok(constraints
        .into_iter()
        .filter(|constraint| backed.contains(&constraint.constraint_name))
        .collect())
}

/// Returns the table that contains this column.
pub(super) fn table(
    column: &Column,
    conn: &mut PgConnection,
) -> Result<Table, diesel::result::Error> {
    use crate::schema::information_schema::tables::tables;
    tables::table
        .filter(tables::table_name.eq(&column.table_name))
        .filter(tables::table_schema.eq(&column.table_schema))
        .filter(tables::table_catalog.eq(&column.table_catalog))
        .first::<Table>(conn)
}

/// Returns the geometry column metadata if this column is a geometry column.
pub(super) fn geometry_column(
    column: &Column,
    conn: &mut PgConnection,
) -> Result<Option<GeometryColumn>, diesel::result::Error> {
    use crate::schema::public::geometry_columns::geometry_columns;

    geometry_columns::table
        .filter(geometry_columns::f_table_name.eq(&column.table_name))
        .filter(geometry_columns::f_table_schema.eq(&column.table_schema))
        .filter(geometry_columns::f_geometry_column.eq(&column.column_name))
        .first::<GeometryColumn>(conn)
        .optional()
}

/// Returns the geography column metadata if this column is a geography column.
pub(super) fn geography_column(
    column: &Column,
    conn: &mut PgConnection,
) -> Result<Option<GeographyColumn>, diesel::result::Error> {
    use crate::schema::public::geography_columns::geography_columns;

    geography_columns::table
        .filter(geography_columns::f_table_name.eq(&column.table_name))
        .filter(geography_columns::f_table_schema.eq(&column.table_schema))
        .filter(geography_columns::f_geography_column.eq(&column.column_name))
        .first::<GeographyColumn>(conn)
        .optional()
}

/// Returns the `PostgreSQL` type of this column.
pub(super) fn pg_type(
    column: &Column,
    conn: &mut PgConnection,
) -> Result<PgType, diesel::result::Error> {
    use crate::schema::pg_catalog::{
        pg_attribute::pg_attribute, pg_class::pg_class, pg_namespace::pg_namespace,
        pg_type::pg_type,
    };

    pg_type::table
        .inner_join(pg_attribute::table.on(pg_attribute::atttypid.eq(pg_type::oid)))
        .inner_join(pg_class::table.on(pg_attribute::attrelid.eq(pg_class::oid)))
        .inner_join(pg_namespace::table.on(pg_class::relnamespace.eq(pg_namespace::oid)))
        .filter(pg_class::relname.eq(&column.table_name))
        .filter(pg_namespace::nspname.eq(&column.table_schema))
        .filter(pg_attribute::attname.eq(&column.column_name))
        .select(PgType::as_select())
        .first::<PgType>(conn)
}
