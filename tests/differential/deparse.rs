//! Brings source expressions into the form `PostgreSQL` stores, by writing
//! them into scratch objects and reading the server's print back.
//!
//! Every call runs in its own transaction and rolls it back, so the database
//! keeps no trace of the scratch objects.

use diesel::{
    Connection, ExpressionMethods, NullableExpressionMethods, PgConnection, QueryDsl, RunQueryDsl,
    connection::SimpleConnection,
    dsl::sql,
    sql_types::{Nullable, Oid, Text},
};
use pg_diesel::{
    schema::pg_catalog::{
        pg_attrdef::pg_attrdef, pg_attribute::pg_attribute, pg_class::pg_class,
        pg_constraint::pg_constraint, pg_index::pg_index, pg_policy::pg_policy,
        pg_rewrite::pg_rewrite,
    },
    sqlparser::ast::{CreateRule, Expr, Query},
};

use crate::compare::{
    Slot, StoredForm, StoredFormError, parse_expression, parse_query, parse_rule,
};

/// Writes source expressions into scratch objects on a live connection.
pub struct ThroughPostgres<'c> {
    /// The connection the DDL document was applied on.
    connection: &'c mut PgConnection,
    /// Numbers the scratch objects, so no two calls share a name.
    counter: u32,
}

impl<'c> ThroughPostgres<'c> {
    /// Wraps the connection the DDL document was applied on.
    pub fn new(connection: &'c mut PgConnection) -> Self {
        Self {
            connection,
            counter: 0,
        }
    }
}

/// Returns `name` as a quoted identifier.
fn quoted(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// Ends a scratch transaction, carrying its outcome through the rollback.
enum Undo {
    /// The scratch work finished, with this outcome.
    Done(Result<Option<String>, diesel::result::Error>),
    /// The transaction itself failed.
    Transaction(diesel::result::Error),
}

impl From<diesel::result::Error> for Undo {
    fn from(error: diesel::result::Error) -> Self {
        Self::Transaction(error)
    }
}

/// Returns the oid of the session's temporary relation `name`.
fn temporary_oid(connection: &mut PgConnection, name: &str) -> Result<u32, diesel::result::Error> {
    pg_class::table
        .filter(pg_class::relname.eq(name))
        .filter(
            pg_class::relnamespace
                .nullable()
                .eq(sql::<Nullable<Oid>>("pg_my_temp_schema()")),
        )
        .select(pg_class::oid)
        .first(connection)
}

impl ThroughPostgres<'_> {
    /// Runs `statements`, then `read`, in a transaction it rolls back, and
    /// returns what `read` printed.
    fn scratch(
        &mut self,
        statements: &str,
        read: impl FnOnce(&mut PgConnection) -> Result<Option<String>, diesel::result::Error>,
    ) -> Result<Option<String>, diesel::result::Error> {
        let outcome = self.connection.transaction::<(), Undo, _>(|connection| {
            let done = connection
                .batch_execute(statements)
                .and_then(|()| read(connection));
            Err(Undo::Done(done))
        });
        match outcome {
            Ok(()) => unreachable!("the scratch transaction always rolls back"),
            Err(Undo::Done(done)) => done,
            Err(Undo::Transaction(error)) => Err(error),
        }
    }
}

impl StoredForm for ThroughPostgres<'_> {
    fn expression(
        &mut self,
        schema: &str,
        table: &str,
        slot: Slot<'_>,
        sql_text: &str,
    ) -> Result<Expr, StoredFormError> {
        self.counter += 1;
        let scratch = format!("pgdiff_scratch_{}", self.counter);
        let object = format!("pgdiff_object_{}", self.counter);
        let create = format!(
            "CREATE TEMP TABLE {scratch} (LIKE {}.{});",
            quoted(schema),
            quoted(table)
        );
        let refused = |message: String| StoredFormError::Refused {
            table: format!("{schema}.{table}"),
            slot: slot.to_string(),
            sql: sql_text.to_owned(),
            message,
        };
        let printed = match slot {
            Slot::ColumnDefault { column } => {
                let column = column.to_owned();
                self.scratch(
                    &format!(
                        "{create} ALTER TABLE {scratch} ALTER COLUMN {} SET DEFAULT {sql_text};",
                        quoted(&column)
                    ),
                    |connection| {
                        let oid = temporary_oid(connection, &scratch)?;
                        let number: i16 = pg_attribute::table
                            .filter(pg_attribute::attrelid.eq(oid))
                            .filter(pg_attribute::attname.eq(&column))
                            .select(pg_attribute::attnum)
                            .first(connection)?;
                        pg_attrdef::table
                            .filter(pg_attrdef::adrelid.eq(oid))
                            .filter(pg_attrdef::adnum.eq(number))
                            .select(sql::<Nullable<Text>>("pg_get_expr(adbin, adrelid)"))
                            .first(connection)
                    },
                )
            }
            Slot::Check => self.scratch(
                &format!(
                    "{create} ALTER TABLE {scratch} ADD CONSTRAINT {object} CHECK ({sql_text});"
                ),
                |connection| {
                    pg_constraint::table
                        .filter(pg_constraint::conname.eq(&object))
                        .select(sql::<Nullable<Text>>("pg_get_expr(conbin, conrelid)"))
                        .first(connection)
                },
            ),
            Slot::PolicyUsing | Slot::PolicyCheck => {
                let (clause, print) = if matches!(slot, Slot::PolicyUsing) {
                    ("USING", "pg_get_expr(polqual, polrelid)")
                } else {
                    ("WITH CHECK", "pg_get_expr(polwithcheck, polrelid)")
                };
                // The scratch policy goes on the table itself, since the print
                // aliases a subquery's read of the table the policy is on.
                self.scratch(
                    &format!(
                        "CREATE POLICY {object} ON {}.{} {clause} ({sql_text});",
                        quoted(schema),
                        quoted(table)
                    ),
                    |connection| {
                        pg_policy::table
                            .filter(pg_policy::polname.eq(&object))
                            .select(sql::<Nullable<Text>>(print))
                            .first(connection)
                    },
                )
            }
            Slot::IndexPredicate => self.scratch(
                &format!("{create} CREATE INDEX {object} ON {scratch} ((1)) WHERE {sql_text};"),
                |connection| {
                    let oid = temporary_oid(connection, &scratch)?;
                    pg_index::table
                        .filter(pg_index::indrelid.eq(oid))
                        .select(sql::<Nullable<Text>>("pg_get_expr(indpred, indrelid)"))
                        .first(connection)
                },
            ),
        }
        .map_err(|error| refused(error.to_string()))?
        .ok_or_else(|| refused("PostgreSQL printed nothing".to_owned()))?;
        parse_expression(&printed)
    }

    fn view_query(&mut self, sql_text: &str) -> Result<Query, StoredFormError> {
        self.counter += 1;
        let view = format!("pgdiff_view_{}", self.counter);
        let printed = self
            .scratch(
                &format!("CREATE TEMP VIEW {view} AS {sql_text};"),
                |connection| {
                    let oid = temporary_oid(connection, &view)?;
                    pg_class::table
                        .filter(pg_class::oid.eq(oid))
                        .select(sql::<Nullable<Text>>("pg_get_viewdef(oid)"))
                        .first(connection)
                },
            )
            .map_err(move |error| StoredFormError::Refused {
                table: view,
                slot: "view definition".to_owned(),
                sql: sql_text.to_owned(),
                message: error.to_string(),
            })?
            .unwrap_or_default();
        parse_query(&printed)
    }

    fn rule(&mut self, sql_text: &str) -> Result<CreateRule, StoredFormError> {
        let refused = |message: String| StoredFormError::Refused {
            table: String::new(),
            slot: "rule".to_owned(),
            sql: sql_text.to_owned(),
            message,
        };
        let printed = self
            .scratch(&format!("{sql_text};"), |connection| {
                pg_rewrite::table
                    .filter(pg_rewrite::rulename.eq("pgdiff_rule"))
                    .select(sql::<Nullable<Text>>("pg_get_ruledef(oid)"))
                    .first(connection)
            })
            .map_err(|error| refused(error.to_string()))?
            .ok_or_else(|| refused("PostgreSQL printed nothing".to_owned()))?;
        parse_rule(&printed)
    }
}
