//! Submodule providing the `PgColumn` type, a column read from
//! `information_schema.columns` with the facts its `pg_attribute` row
//! carries beside the `information_schema` row.

use crate::{model_metadata::PgDataType, models::Column};

/// A table column with the declared type and foreign data wrapper options
/// the engine records for it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgColumn {
    /// The column the `information_schema` records.
    model: Column,
    /// The declared type the column's `pg_attribute` row records.
    data_type: PgDataType,
    /// The foreign data wrapper options the column declares, each key first.
    foreign_options: Vec<(String, String)>,
    /// The name the not-null constraint of the column holds, when the engine
    /// records one.
    not_null_constraint_name: Option<String>,
    /// Whether the engine numbers the column with a sequence the column owns.
    serial: bool,
}

impl PgColumn {
    /// Creates a `PgColumn` over the `information_schema` row it wraps.
    #[must_use]
    pub fn new(
        model: Column,
        data_type: PgDataType,
        foreign_options: Vec<(String, String)>,
        not_null_constraint_name: Option<String>,
        serial: bool,
    ) -> Self {
        Self {
            model,
            data_type,
            foreign_options,
            not_null_constraint_name,
            serial,
        }
    }

    /// The `information_schema` row the column wraps.
    #[must_use]
    pub fn model(&self) -> &Column {
        &self.model
    }

    /// The name of the column.
    #[must_use]
    pub fn column_name(&self) -> &str {
        &self.model.column_name
    }

    /// The declared type of the column.
    #[must_use]
    pub fn declared_type(&self) -> &PgDataType {
        &self.data_type
    }

    /// The name the not-null constraint of the column holds, when the engine
    /// records one.
    #[must_use = "the name may be absent, so the caller decides how to answer"]
    pub fn not_null_constraint_name(&self) -> Option<&str> {
        self.not_null_constraint_name.as_deref()
    }

    /// Whether the engine numbers the column with a sequence the column owns.
    #[must_use]
    pub fn is_serial(&self) -> bool {
        self.serial
    }

    /// The foreign data wrapper options the column declares, each key first.
    pub fn foreign_options(&self) -> impl Iterator<Item = (&str, &str)> {
        self.foreign_options
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }
}
