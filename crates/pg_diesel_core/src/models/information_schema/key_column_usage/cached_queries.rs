//! Submodule defining the cached queries methods used in the [`KeyColumnUsage`]
//! struct.

use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl,
    SelectableHelper,
};

use crate::models::{KeyColumnUsage, ReferentialConstraint, Table};

/// Returns the referential constraint associated with this key column usage.
pub(super) fn referential_constraint(
    key_column_usage: &KeyColumnUsage,
    conn: &mut PgConnection,
) -> Result<ReferentialConstraint, diesel::result::Error> {
    use crate::schema::information_schema::referential_constraints::referential_constraints;
    referential_constraints::table
        .filter(referential_constraints::constraint_name.eq(&key_column_usage.constraint_name))
        .filter(referential_constraints::constraint_schema.eq(&key_column_usage.constraint_schema))
        .filter(
            referential_constraints::constraint_catalog.eq(&key_column_usage.constraint_catalog),
        )
        .select(ReferentialConstraint::as_select())
        .first::<ReferentialConstraint>(conn)
}

/// Returns the names of the columns this foreign key references, in the
/// order the foreign key lists them, each paired with the host column at the
/// same position.
pub(super) fn referenced_columns(
    key_column_usage: &KeyColumnUsage,
    conn: &mut PgConnection,
) -> Result<Vec<String>, diesel::result::Error> {
    use crate::schema::information_schema::key_column_usage::key_column_usage as usage;

    let referential_constraint = referential_constraint(key_column_usage, conn)?;
    let positions: Vec<Option<i32>> = usage::table
        .filter(usage::constraint_name.eq(&key_column_usage.constraint_name))
        .filter(usage::constraint_schema.eq(&key_column_usage.constraint_schema))
        .filter(usage::constraint_catalog.eq(&key_column_usage.constraint_catalog))
        .order_by(usage::ordinal_position)
        .select(usage::position_in_unique_constraint)
        .load(conn)?;
    let unique_key: Vec<(i32, String)> = usage::table
        .filter(
            usage::constraint_catalog.eq(referential_constraint
                .unique_constraint_catalog
                .ok_or(diesel::result::Error::NotFound)?),
        )
        .filter(
            usage::constraint_schema.eq(referential_constraint
                .unique_constraint_schema
                .ok_or(diesel::result::Error::NotFound)?),
        )
        .filter(
            usage::constraint_name.eq(referential_constraint
                .unique_constraint_name
                .ok_or(diesel::result::Error::NotFound)?),
        )
        .select((usage::ordinal_position, usage::column_name))
        .load(conn)?;
    positions
        .into_iter()
        .map(|position| {
            let position = position.ok_or(diesel::result::Error::NotFound)?;
            unique_key
                .iter()
                .find(|(ordinal, _)| *ordinal == position)
                .map(|(_, name)| name.clone())
                .ok_or(diesel::result::Error::NotFound)
        })
        .collect()
}

/// Returns the names of the host columns of this key column usage, in the
/// order the constraint declares them.
pub(crate) fn host_columns(
    key_column_usage: &KeyColumnUsage,
    conn: &mut PgConnection,
) -> Result<Vec<String>, diesel::result::Error> {
    use crate::schema::information_schema::{columns::columns, key_column_usage::key_column_usage};
    key_column_usage::table
        .filter(key_column_usage::constraint_name.eq(&key_column_usage.constraint_name))
        .filter(key_column_usage::constraint_schema.eq(&key_column_usage.constraint_schema))
        .filter(key_column_usage::constraint_catalog.eq(&key_column_usage.constraint_catalog))
        .inner_join(
            columns::table.on(columns::table_name
                .eq(key_column_usage::table_name)
                .and(columns::table_schema.eq(key_column_usage::table_schema))
                .and(columns::table_catalog.eq(key_column_usage::table_catalog))
                .and(columns::column_name.eq(key_column_usage::column_name))),
        )
        .order_by(key_column_usage::ordinal_position.asc())
        .select(columns::column_name)
        .load(conn)
}

/// Returns the referenced table associated with this key column usage
///
/// # Errors
///
/// * If an error occurs while loading the referenced table from the database
pub(super) fn referenced_table(
    key_column_usage: &KeyColumnUsage,
    conn: &mut PgConnection,
) -> Result<Table, diesel::result::Error> {
    use crate::schema::information_schema::{
        constraint_table_usage::constraint_table_usage, tables::tables,
    };

    let constraint = referential_constraint(key_column_usage, conn)?;

    constraint_table_usage::table
        .inner_join(
            tables::table.on(tables::table_name
                .eq(constraint_table_usage::table_name)
                .and(tables::table_schema.eq(constraint_table_usage::table_schema))
                .and(tables::table_catalog.eq(constraint_table_usage::table_catalog))),
        )
        .filter(constraint_table_usage::constraint_name.eq(&constraint.constraint_name))
        .filter(constraint_table_usage::constraint_schema.eq(&constraint.constraint_schema))
        .filter(constraint_table_usage::constraint_catalog.eq(&constraint.constraint_catalog))
        .select(Table::as_select())
        .first::<Table>(conn)
}
