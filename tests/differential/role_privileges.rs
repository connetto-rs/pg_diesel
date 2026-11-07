//! Reads the privileges `PostgreSQL` answers one role holds on another,
//! through `pg_has_role`.

use diesel::{
    PgConnection, RunQueryDsl,
    dsl::sql,
    sql_types::{Bool, Text},
};
use pg_diesel::sql_traits::traits::RolePrivilege;

use crate::compare::{Facts, ROLE_PRIVILEGES, Value, role_privilege_fact};

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
                let holds: bool = diesel::select(
                    sql::<Bool>("pg_has_role(")
                        .bind::<Text, _>(member)
                        .sql("::name, ")
                        .bind::<Text, _>(target)
                        .sql("::name, ")
                        .bind::<Text, _>(*keyword)
                        .sql(")"),
                )
                .get_result(connection)?;
                facts.insert(
                    role_privilege_fact(member, keyword, target),
                    Value::Text(holds.to_string()),
                );
            }
        }
    }
    Ok(facts)
}
