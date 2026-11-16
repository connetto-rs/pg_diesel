//! Submodule providing the `TriggerMetadata` struct for a [`PgTrigger`] model.

use std::sync::Arc;

use sqlparser::{
    ast::{Expr, Ident, Statement, TriggerEvent, TriggerObject, TriggerObjectKind, TriggerPeriod},
    dialect::PostgreSqlDialect,
    parser::{Parser, ParserError},
};

use crate::{model_metadata::PgTable, models::PgTrigger};

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
    /// The table that the trigger is defined on.
    pub table: Arc<PgTable>,
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
pub(crate) fn when_condition(definition: &str) -> Result<Expr, TriggerConditionError> {
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

impl TriggerMetadata {
    /// Creates a `TriggerMetadata` over a `pg_trigger` row, decoding the
    /// events, timing and orientation the row's `tgtype` mask holds, beside
    /// the columns an `UPDATE` event fires on and the `WHEN` condition.
    #[must_use]
    pub fn new(
        model: PgTrigger,
        table: Arc<PgTable>,
        function_name: String,
        update_columns: Vec<String>,
        condition: Option<Expr>,
    ) -> Self {
        Self {
            events: decode_events(model.tgtype, update_columns),
            timing: Some(decode_timing(model.tgtype)),
            orientation: Some(decode_orientation(model.tgtype)),
            function_oid: model.tgfoid,
            function_name,
            condition,
            model,
            table,
        }
    }

    /// Returns the table that the trigger is defined on.
    #[must_use]
    pub fn table(&self) -> &PgTable {
        &self.table
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
}
