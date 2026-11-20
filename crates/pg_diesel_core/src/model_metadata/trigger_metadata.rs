//! Submodule providing the `TriggerMetadata` struct for a [`PgTrigger`] model.

use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl,
    SelectableHelper,
    dsl::sql,
    sql_types::{Array, Bool, Nullable, Text},
};
use sql_traits::structs::TriggerEnablement;
use sqlparser::{
    ast::{Expr, Ident, Statement, TriggerEvent, TriggerObject, TriggerObjectKind, TriggerPeriod},
    dialect::PostgreSqlDialect,
    parser::{Parser, ParserError},
};

use crate::{
    model_metadata::TargetRelation,
    models::PgTrigger,
    schema::pg_catalog::{pg_namespace::pg_namespace, pg_proc::pg_proc, pg_trigger::pg_trigger},
    shared::schema::pg_catalog::pg_class,
};

/// One bit of the `pg_trigger.tgtype` mask, which a trigger decodes against.
const TYPE_ROW: i16 = 1;
/// The trigger runs before the statement or rows that fire it run.
const TYPE_BEFORE: i16 = 2;
/// The trigger fires on an `INSERT`.
const TYPE_INSERT: i16 = 4;
/// The trigger fires on a `DELETE`.
const TYPE_DELETE: i16 = 8;
/// The trigger fires on an `UPDATE`.
const TYPE_UPDATE: i16 = 16;
/// The trigger fires on a `TRUNCATE`.
const TYPE_TRUNCATE: i16 = 32;
/// The trigger runs instead of the statement that fires it.
const TYPE_INSTEAD: i16 = 64;

#[derive(Clone, Debug)]
/// A `pg_trigger` row with the facts its `tgtype` mask and `tgattr` record.
pub struct TriggerMetadata {
    /// The underlying trigger model.
    pub model: PgTrigger,
    /// The table or view the trigger is defined on.
    pub target: TargetRelation,
    /// The events that fire the trigger.
    pub events: Vec<TriggerEvent>,
    /// The timing of the trigger.
    pub timing: Option<TriggerPeriod>,
    /// The orientation of the trigger.
    pub orientation: Option<TriggerObjectKind>,
    /// The OID of the function called by the trigger.
    pub function_oid: u32,
    /// The name the engine records the function the trigger executes under.
    pub function_name: String,
    /// The `WHEN` condition the trigger fires under, without the parentheses
    /// `WHEN` writes around it.
    pub condition: Option<Expr>,
    /// The sessions the trigger fires in, as `pg_trigger.tgenabled` records
    /// them.
    pub enablement: TriggerEnablement,
}

/// The catalog facts a trigger row loads beside its `pg_trigger` row.
#[derive(Debug, diesel::Queryable)]
pub struct TriggerRow {
    /// The `pg_trigger` row.
    trigger: PgTrigger,
    /// The name the engine records the function the trigger executes under.
    function_name: String,
    /// The `CREATE TRIGGER` statement `pg_get_triggerdef` prints, read only
    /// for a trigger with a `WHEN` condition.
    definition: Option<String>,
    /// The stored name of the schema the relation lives in.
    nspname: String,
    /// The stored name of the relation the trigger is defined on.
    relname: String,
    /// The kind of the relation the trigger is defined on.
    relkind: String,
    /// The stored names of the columns an `UPDATE OF` fires the trigger on,
    /// in the order `pg_trigger.tgattr` lists them.
    update_columns: Vec<String>,
}

/// What stopped a `pg_trigger` row from becoming a [`TriggerMetadata`].
#[derive(Debug, thiserror::Error)]
pub enum TriggerDefinitionError {
    /// The definition `pg_get_triggerdef` printed for a trigger with a `WHEN`
    /// condition yields no condition.
    #[error("the definition `{definition}` of trigger {trigger} on {schema}.{relation} {source}")]
    UnreadableCondition {
        /// The schema of the relation.
        schema: String,
        /// The name of the relation.
        relation: String,
        /// The name of the trigger.
        trigger: String,
        /// The definition as `PostgreSQL` printed it.
        definition: String,
        /// Why the definition yields no condition, boxed to keep the error
        /// small.
        source: Box<TriggerConditionError>,
    },
    /// A trigger's `pg_trigger.tgenabled` holds a value `PostgreSQL` does not
    /// document.
    #[error("trigger {trigger} on {schema}.{relation} has the unknown tgenabled `{tgenabled}`")]
    UnknownEnablement {
        /// The schema of the relation.
        schema: String,
        /// The name of the relation.
        relation: String,
        /// The name of the trigger.
        trigger: String,
        /// The value `pg_trigger.tgenabled` holds.
        tgenabled: String,
    },
}

/// What stopped a trigger definition from yielding its `WHEN` condition.
#[derive(Debug, thiserror::Error)]
pub enum TriggerConditionError {
    /// `sqlparser` could not read the definition the catalog rendered.
    #[error("does not parse: {0}")]
    Unparsable(#[from] ParserError),
    /// The definition is not one `CREATE TRIGGER` statement with a `WHEN`
    /// condition.
    #[error("is not one CREATE TRIGGER statement with a WHEN condition")]
    UnexpectedShape,
}

/// Returns the `WHEN` condition of `definition`, the `CREATE TRIGGER`
/// statement `pg_get_triggerdef` prints for a trigger with one, without the
/// parentheses `WHEN` writes around it.
///
/// # Errors
///
/// Returns [`TriggerConditionError::Unparsable`] when `sqlparser` cannot read
/// `definition`, and [`TriggerConditionError::UnexpectedShape`] when it is not
/// one `CREATE TRIGGER` carrying a condition.
fn when_condition(definition: &str) -> Result<Expr, TriggerConditionError> {
    let mut statements = Parser::parse_sql(&PostgreSqlDialect {}, definition)?;
    match statements.pop() {
        Some(Statement::CreateTrigger(trigger)) if statements.is_empty() => {
            match trigger.condition {
                Some(Expr::Nested(condition)) => Ok(*condition),
                Some(condition) => Ok(condition),
                None => Err(TriggerConditionError::UnexpectedShape),
            }
        }
        _ => Err(TriggerConditionError::UnexpectedShape),
    }
}

/// Decodes the sessions a `pg_trigger.tgenabled` value fires the trigger in,
/// or `None` for a value `PostgreSQL` does not document.
fn decode_enablement(tgenabled: &str) -> Option<TriggerEnablement> {
    match tgenabled {
        "O" => Some(TriggerEnablement::Origin),
        "D" => Some(TriggerEnablement::Disabled),
        "R" => Some(TriggerEnablement::Replica),
        "A" => Some(TriggerEnablement::Always),
        _ => None,
    }
}

impl TriggerMetadata {
    /// Loads the trigger rows of the tables, foreign tables and plain views
    /// the schemas `schemas` hold, restricting the tables to the OIDs
    /// `table_oids` when it names some.
    ///
    /// A partition's copy of a trigger on its partitioned table is internal
    /// before `PostgreSQL` 15, and it is told apart from the engine's own
    /// triggers by the dependency on the trigger it copies.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the query fails.
    pub(crate) fn load_filtered(
        schemas: &[String],
        table_oids: Option<&[u32]>,
        conn: &mut PgConnection,
    ) -> Result<Vec<TriggerRow>, diesel::result::Error> {
        let mut query = pg_trigger::table
            .inner_join(pg_proc::table.on(pg_trigger::tgfoid.eq(pg_proc::oid)))
            .inner_join(pg_class::table.on(pg_trigger::tgrelid.eq(pg_class::oid)))
            .inner_join(pg_namespace::table.on(pg_class::relnamespace.eq(pg_namespace::oid)))
            .filter(pg_class::relkind.eq_any(&["r", "p", "f", "v"]))
            .filter(pg_namespace::nspname.eq_any(schemas))
            .filter(pg_trigger::tgisinternal.eq(false).or(sql::<Bool>(
                "EXISTS (SELECT 1 FROM pg_catalog.pg_depend \
                 WHERE pg_depend.objid = pg_trigger.oid \
                 AND pg_depend.classid = 'pg_catalog.pg_trigger'::pg_catalog.regclass \
                 AND pg_depend.refclassid = 'pg_catalog.pg_trigger'::pg_catalog.regclass)",
            )))
            .order_by((pg_namespace::nspname, pg_class::relname, pg_trigger::oid))
            .select((
                PgTrigger::as_select(),
                pg_proc::proname,
                sql::<Nullable<Text>>(
                    "CASE WHEN pg_trigger.tgqual IS NULL THEN NULL \
                     ELSE pg_get_triggerdef(pg_trigger.oid) END",
                ),
                pg_namespace::nspname,
                pg_class::relname,
                pg_class::relkind,
                sql::<Array<Text>>(
                    "ARRAY(SELECT attribute.attname::text \
                     FROM unnest(pg_trigger.tgattr::int2[]) WITH ORDINALITY AS fired(number, position) \
                     JOIN pg_catalog.pg_attribute AS attribute \
                     ON attribute.attrelid = pg_trigger.tgrelid AND attribute.attnum = fired.number \
                     ORDER BY fired.position)",
                ),
            ))
            .into_boxed();
        if let Some(oids) = table_oids {
            query = query.filter(pg_class::relkind.eq("v").or(pg_class::oid.eq_any(oids)));
        }
        query.load(conn)
    }

    /// Reads one trigger row, decoding the events, timing and orientation the
    /// row's `tgtype` mask holds, beside the columns an `UPDATE` event fires
    /// on, the `WHEN` condition and the sessions the trigger fires in.
    ///
    /// # Errors
    ///
    /// Returns [`TriggerDefinitionError::UnreadableCondition`] when the
    /// printed definition yields no `WHEN` condition, and
    /// [`TriggerDefinitionError::UnknownEnablement`] when `tgenabled` holds a
    /// value `PostgreSQL` does not document.
    pub fn new(row: TriggerRow) -> Result<Self, TriggerDefinitionError> {
        let TriggerRow {
            trigger: model,
            function_name,
            definition,
            nspname,
            relname,
            relkind,
            update_columns,
        } = row;
        let condition = match definition {
            Some(definition) => match when_condition(&definition) {
                Ok(condition) => Some(condition),
                Err(source) => {
                    return Err(TriggerDefinitionError::UnreadableCondition {
                        schema: nspname,
                        relation: relname,
                        trigger: model.tgname,
                        definition,
                        source: Box::new(source),
                    });
                }
            },
            None => None,
        };
        let Some(enablement) = decode_enablement(&model.tgenabled) else {
            return Err(TriggerDefinitionError::UnknownEnablement {
                schema: nspname,
                relation: relname,
                trigger: model.tgname,
                tgenabled: model.tgenabled,
            });
        };
        Ok(Self {
            events: decode_events(model.tgtype, update_columns),
            timing: Some(decode_timing(model.tgtype)),
            orientation: Some(decode_orientation(model.tgtype)),
            function_oid: model.tgfoid,
            function_name,
            condition,
            enablement,
            target: TargetRelation::new(nspname, relname, &relkind),
            model,
        })
    }
}

/// Decodes the events a `tgtype` mask fires the trigger on, an `UPDATE`
/// naming the columns the mask fired it for.
fn decode_events(tgtype: i16, update_columns: Vec<String>) -> Vec<TriggerEvent> {
    let mut events = Vec::new();
    if tgtype & TYPE_INSERT != 0 {
        events.push(TriggerEvent::Insert);
    }
    if tgtype & TYPE_DELETE != 0 {
        events.push(TriggerEvent::Delete);
    }
    if tgtype & TYPE_UPDATE != 0 {
        let columns = update_columns
            .into_iter()
            .map(|column| Ident::new(column.as_str()))
            .collect();
        events.push(TriggerEvent::Update(columns));
    }
    if tgtype & TYPE_TRUNCATE != 0 {
        events.push(TriggerEvent::Truncate);
    }
    events
}

/// Decodes the timing a `tgtype` mask gives the trigger, which the mask
/// records as a `BEFORE` or an `INSTEAD OF`, the trigger running after
/// where it names neither.
fn decode_timing(tgtype: i16) -> TriggerPeriod {
    if tgtype & TYPE_BEFORE != 0 {
        TriggerPeriod::Before
    } else if tgtype & TYPE_INSTEAD != 0 {
        TriggerPeriod::InsteadOf
    } else {
        TriggerPeriod::After
    }
}

/// Decodes the orientation a `tgtype` mask gives the trigger, statement level
/// where the mask names no rows.
fn decode_orientation(tgtype: i16) -> TriggerObjectKind {
    if tgtype & TYPE_ROW != 0 {
        TriggerObjectKind::ForEach(TriggerObject::Row)
    } else {
        TriggerObjectKind::ForEach(TriggerObject::Statement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_condition_comes_without_the_parentheses_when_writes() {
        let condition = when_condition(
            "CREATE TRIGGER filtered AFTER UPDATE ON public.notes FOR EACH ROW \
             WHEN ((new.body <> old.body)) EXECUTE FUNCTION log()",
        )
        .expect("the definition carries a condition");
        assert_eq!(condition.to_string(), "(new.body <> old.body)");
    }

    #[test]
    fn a_definition_the_parser_refuses_is_reported_with_the_complaint() {
        let error = when_condition("not a statement").expect_err("the text does not parse");
        assert!(matches!(error, TriggerConditionError::Unparsable(_)));
    }

    #[test]
    fn a_trigger_without_a_condition_is_reported_as_the_wrong_shape() {
        let error = when_condition(
            "CREATE TRIGGER plain AFTER UPDATE ON public.notes FOR EACH ROW \
             EXECUTE FUNCTION log()",
        )
        .expect_err("the definition carries no condition");
        assert!(matches!(error, TriggerConditionError::UnexpectedShape));
    }

    #[test]
    fn an_undocumented_tgenabled_value_decodes_to_no_enablement() {
        assert_eq!(decode_enablement("X"), None);
    }

    #[test]
    fn a_row_whose_definition_yields_no_condition_is_refused_on_its_relation() {
        let row = TriggerRow {
            trigger: PgTrigger {
                oid: 1,
                tgrelid: 2,
                tgparentid: 0,
                tgname: "audit".to_string(),
                tgfoid: 3,
                tgtype: TYPE_ROW | TYPE_INSTEAD | TYPE_INSERT,
                tgenabled: "O".to_string(),
                tgisinternal: false,
                tgconstrrelid: 0,
                tgconstrindid: 0,
                tgconstraint: 0,
                tgdeferrable: false,
                tginitdeferred: false,
                tgnargs: 0,
                tgattr: Vec::new(),
                tgargs: Vec::new(),
                tgoldtable: None,
                tgnewtable: None,
                tgqual: Some("{OPEXPR}".to_string()),
            },
            function_name: "touch".to_string(),
            definition: Some("not a statement".to_string()),
            nspname: "app".to_string(),
            relname: "note_view".to_string(),
            relkind: "v".to_string(),
            update_columns: Vec::new(),
        };
        let error = TriggerMetadata::new(row).expect_err("the definition does not parse");
        assert!(matches!(
            error,
            TriggerDefinitionError::UnreadableCondition { schema, relation, trigger, .. }
                if (schema.as_str(), relation.as_str(), trigger.as_str())
                    == ("app", "note_view", "audit")
        ));
    }
}
