//! Submodule defining the cached queries methods used in the [`KeyColumnUsage`]
//! struct.

use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use sql_traits::structs::ConstraintDeferral;

use crate::{
    model_metadata::decode_deferral,
    models::{KeyColumnUsage, ReferentialConstraint, Table},
};

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

/// Returns when the foreign key of this key column usage is checked within a
/// transaction.
pub(super) fn deferral(
    key_column_usage: &KeyColumnUsage,
    conn: &mut PgConnection,
) -> Result<ConstraintDeferral, diesel::result::Error> {
    use crate::shared::schema::information_schema::table_constraints;
    let (deferrable, initially_deferred): (bool, bool) = table_constraints::table
        .filter(table_constraints::constraint_name.eq(&key_column_usage.constraint_name))
        .filter(table_constraints::constraint_schema.eq(&key_column_usage.constraint_schema))
        .filter(table_constraints::constraint_catalog.eq(&key_column_usage.constraint_catalog))
        .filter(table_constraints::table_name.eq(&key_column_usage.table_name))
        .filter(table_constraints::table_schema.eq(&key_column_usage.table_schema))
        .filter(table_constraints::table_catalog.eq(&key_column_usage.table_catalog))
        .select((
            table_constraints::is_deferrable.eq("YES"),
            table_constraints::initially_deferred.eq("YES"),
        ))
        .first(conn)?;
    Ok(decode_deferral(deferrable, initially_deferred))
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
