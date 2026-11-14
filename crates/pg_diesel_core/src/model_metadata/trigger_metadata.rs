//! Submodule providing the `TriggerMetadata` struct for a [`PgTrigger`] model.

use std::sync::Arc;

use sqlparser::ast::{Ident, TriggerEvent, TriggerObject, TriggerObjectKind, TriggerPeriod};

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
}

impl TriggerMetadata {
    /// Creates a `TriggerMetadata` over a `pg_trigger` row, decoding the
    /// events, timing and orientation the row's `tgtype` mask holds and the
    /// columns an `UPDATE` event fires on.
    #[must_use]
    pub fn new(
        model: PgTrigger,
        table: Arc<PgTable>,
        function_name: String,
        update_columns: Vec<String>,
    ) -> Self {
        Self {
            events: decode_events(model.tgtype, update_columns),
            timing: Some(decode_timing(model.tgtype)),
            orientation: Some(decode_orientation(model.tgtype)),
            function_oid: model.tgfoid,
            function_name,
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
