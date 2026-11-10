//! Reads what `PostgreSQL` itself answers about roles and privileges, in the
//! facts the describer names, so the live model can be checked against the
//! server it was read from.

use diesel::{
    ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl,
    sql_types::{Bool, Oid, SmallInt, Text},
};
use pg_diesel::{
    schema::pg_catalog::{
        pg_attribute::pg_attribute, pg_class::pg_class, pg_namespace::pg_namespace,
    },
    sql_traits::traits::RolePrivilege,
};

use crate::compare::{
    COLUMN_PRIVILEGES, Facts, ROLE_PRIVILEGES, Value, WHOLE_TABLE_PRIVILEGES,
    column_privilege_value, role_privilege_fact, table_privilege_fact,
};

diesel::define_sql_function! {
    /// `pg_has_role(member, target, privilege)`.
    fn pg_has_role(member: Text, target: Text, privilege: Text) -> Bool;
}

diesel::define_sql_function! {
    /// `has_table_privilege(role, table, privilege)`.
    fn has_table_privilege(role: Text, table: Oid, privilege: Text) -> Bool;
}

diesel::define_sql_function! {
    /// `has_column_privilege(role, table, column, privilege)`.
    fn has_column_privilege(role: Text, table: Oid, column: SmallInt, privilege: Text) -> Bool;
}

/// Returns, for every ordered pair of `roles` and every privilege the server
/// knows, whether `pg_has_role` grants the first role the privilege on the
/// second, as the facts the describer names.
///
/// `SET` is a `pg_has_role` privilege from PostgreSQL 16 on, so a server of
/// an older `server_major` is asked about the others only.
///
/// # Errors
///
/// Returns a `diesel::result::Error` when a query fails.
pub fn server_role_privileges(
    connection: &mut PgConnection,
    roles: &[String],
    server_major: u32,
) -> Result<Facts, diesel::result::Error> {
    let mut facts = Facts::new();
    let privileges = ROLE_PRIVILEGES
        .iter()
        .filter(|(privilege, _)| server_major >= 16 || *privilege != RolePrivilege::Set);
    for member in roles {
        for target in roles {
            for (_, keyword) in privileges.clone() {
                let holds: bool =
                    diesel::select(pg_has_role(member, target, *keyword)).get_result(connection)?;
                facts.insert(
                    role_privilege_fact(member, keyword, target),
                    Value::Text(holds.to_string()),
                );
            }
        }
    }
    Ok(facts)
}

/// Returns the oid of the relation `schema.name`.
fn relation_oid(
    connection: &mut PgConnection,
    schema: &str,
    name: &str,
) -> Result<u32, diesel::result::Error> {
    pg_class::table
        .inner_join(pg_namespace::table.on(pg_namespace::oid.eq(pg_class::relnamespace)))
        .filter(pg_namespace::nspname.eq(schema))
        .filter(pg_class::relname.eq(name))
        .select(pg_class::oid)
        .first(connection)
}

/// Returns the number and name of every column of the relation `oid`, in
/// table order.
fn columns(
    connection: &mut PgConnection,
    oid: u32,
) -> Result<Vec<(i16, String)>, diesel::result::Error> {
    pg_attribute::table
        .filter(pg_attribute::attrelid.eq(oid))
        .filter(pg_attribute::attnum.gt(0))
        .filter(pg_attribute::attisdropped.eq(false))
        .order(pg_attribute::attnum)
        .select((pg_attribute::attnum, pg_attribute::attname))
        .load(connection)
}

/// Returns, for every table of `tables`, every role of `roles` and every
/// table privilege the server knows, where `has_table_privilege` and
/// `has_column_privilege` grant the role the privilege, as the facts the
/// describer names.
///
/// `MAINTAIN` is a table privilege from PostgreSQL 17 on, so a server of an
/// older `server_major` is asked about the others only.
///
/// # Errors
///
/// Returns a `diesel::result::Error` when a query fails.
pub fn server_table_privileges(
    connection: &mut PgConnection,
    tables: &[(String, String)],
    roles: &[String],
    server_major: u32,
) -> Result<Facts, diesel::result::Error> {
    let mut facts = Facts::new();
    for (schema, name) in tables {
        let oid = relation_oid(connection, schema, name)?;
        let columns = columns(connection, oid)?;
        for role in roles {
            for keyword in COLUMN_PRIVILEGES {
                let on_table: bool = diesel::select(has_table_privilege(role, oid, keyword))
                    .get_result(connection)?;
                let mut holding = Vec::new();
                if !on_table {
                    for (number, column) in &columns {
                        if diesel::select(has_column_privilege(role, oid, *number, keyword))
                            .get_result(connection)?
                        {
                            holding.push(column.clone());
                        }
                    }
                }
                facts.insert(
                    table_privilege_fact(schema, name, keyword, role),
                    Value::Text(column_privilege_value(on_table, &holding)),
                );
            }
            for keyword in WHOLE_TABLE_PRIVILEGES {
                if keyword == "MAINTAIN" && server_major < 17 {
                    continue;
                }
                let holds: bool = diesel::select(has_table_privilege(role, oid, keyword))
                    .get_result(connection)?;
                facts.insert(
                    table_privilege_fact(schema, name, keyword, role),
                    Value::Text(holds.to_string()),
                );
            }
        }
    }
    Ok(facts)
}
