//! Submodule providing [`PgTable`], the base table a `PgDieselDatabase` holds.

use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use sql_traits::{
    structs::{ListDatum, PartitionBound, RangeDatum},
    traits::PartitionStrategy,
};
use sqlparser::{
    ast::{Expr, ForValues, PartitionBoundValue, Statement, Value, ValueWithSpan},
    dialect::PostgreSqlDialect,
    parser::{Parser, ParserError},
};

use crate::models::{Table, TableFacts};

/// A base table row paired with the partitioning strategy it omits.
///
/// `TableLike` asks for the partitioning facts without handing over a
/// database, so the answers have to live on the value itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PgTable {
    /// The `information_schema.tables` row describing the table.
    model: Table,
    /// How the table routes rows to its partitions, when it is partitioned.
    partition_strategy: Option<PartitionStrategy>,
    /// Whether the table is `UNLOGGED`.
    unlogged: bool,
    /// The storage parameters `pg_class.reloptions` records.
    storage_parameters: Vec<(String, String)>,
    /// The storage parameters of the table's TOAST relation.
    toast_storage_parameters: Vec<(String, String)>,
    /// The bound the table takes as a partition.
    partition_bound: Option<PartitionBound>,
}

/// What stopped a catalog row from becoming a [`PgTable`].
#[derive(Debug, thiserror::Error)]
pub enum PgTableError {
    /// A `pg_partitioned_table.partstrat` value this crate does not know.
    #[error("`{0}` is not a partitioning strategy PostgreSQL documents")]
    UnknownPartitionStrategy(String),
    /// A `FOR VALUES` bound `pg_get_expr` wrote that `sqlparser` cannot read
    /// back.
    #[error("the partition bound `{0}` does not parse: {1}")]
    UnparsablePartitionBound(String, #[source] ParserError),
}

/// Ranks a strategy, since [`PartitionStrategy`] derives neither [`Ord`] nor
/// [`Hash`] and `TableLike` requires both.
fn strategy_rank(strategy: Option<PartitionStrategy>) -> u8 {
    match strategy {
        None => 0,
        Some(PartitionStrategy::Range) => 1,
        Some(PartitionStrategy::List) => 2,
        Some(PartitionStrategy::Hash) => 3,
    }
}

impl Hash for PgTable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.model.hash(state);
        strategy_rank(self.partition_strategy).hash(state);
    }
}

impl PartialOrd for PgTable {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PgTable {
    fn cmp(&self, other: &Self) -> Ordering {
        self.model.cmp(&other.model).then_with(|| {
            strategy_rank(self.partition_strategy).cmp(&strategy_rank(other.partition_strategy))
        })
    }
}

impl std::fmt::Display for PgTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.model.fmt(f)
    }
}

impl AsRef<Table> for PgTable {
    fn as_ref(&self) -> &Table {
        &self.model
    }
}

/// The storage parameters a `PostgreSQL` option array records, each
/// `name=value` entry as the catalog stores it and a bare name read as
/// `true`.
pub type StorageOptionPairs = Vec<(String, String)>;

/// The storage parameters one of `PostgreSQL`'s option arrays records, each
/// `name=value` entry as the catalog stores it and a bare name read as
/// `true`, as `PostgreSQL` stores one.
#[must_use]
pub fn storage_option_entries(entries: &[String]) -> StorageOptionPairs {
    entries
        .iter()
        .map(|entry| match entry.split_once('=') {
            Some((name, value)) => (name.to_owned(), value.to_owned()),
            None => (entry.clone(), "true".to_owned()),
        })
        .collect()
}

/// Routes the storage parameters the parent relation and its TOAST relation
/// record, the `toast.` namespace belonging to the TOAST side, which
/// `PostgreSQL` keeps on the TOAST relation unprefixed, so the parent's
/// namespaced entries and the TOAST relation's own merge, the parent's first.
fn route_storage_parameters(
    parent: &[String],
    toast: &[String],
) -> (StorageOptionPairs, StorageOptionPairs) {
    let (mut storage, mut toast_side) = (Vec::new(), Vec::new());
    for (name, value) in storage_option_entries(parent) {
        match name.strip_prefix("toast.") {
            Some(stripped) => toast_side.push((stripped.to_owned(), value)),
            None => storage.push((name, value)),
        }
    }
    toast_side.extend(storage_option_entries(toast));
    (storage, toast_side)
}

/// The bound in `text`, which `PostgreSQL` deparses without the partition it
/// belongs to, so it is read back through a `PARTITION OF` declaration
/// because `sqlparser` only accepts `FOR VALUES` clauses in that form.
pub(crate) fn parse_partition_bound(text: &str) -> Result<PartitionBound, PgTableError> {
    let sql = format!("CREATE TABLE __p PARTITION OF __r {text}");
    let statements = Parser::parse_sql(&PostgreSqlDialect {}, &sql)
        .map_err(|source| PgTableError::UnparsablePartitionBound(text.to_owned(), source))?;
    let [Statement::CreateTable(create)] = statements.as_slice() else {
        return Err(PgTableError::UnparsablePartitionBound(
            text.to_owned(),
            ParserError::ParserError("the wrapped `PARTITION OF` form did not parse".to_owned()),
        ));
    };
    let Some(for_values) = create.for_values.clone() else {
        return Err(PgTableError::UnparsablePartitionBound(
            text.to_owned(),
            ParserError::ParserError("no `FOR VALUES` clause after `PARTITION OF`".to_owned()),
        ));
    };
    Ok(match for_values {
        ForValues::Default => PartitionBound::Default,
        ForValues::From { from, to } => PartitionBound::Range {
            from: range_data(&from),
            to: range_data(&to),
        },
        ForValues::In(values) => PartitionBound::List {
            values: values.into_iter().map(list_datum).collect(),
        },
        ForValues::With { modulus, remainder } => PartitionBound::Hash { modulus, remainder },
    })
}

/// The entries of one side of a range bound, each value as the catalog wrote
/// it.
fn range_data(values: &[PartitionBoundValue]) -> Vec<RangeDatum> {
    values
        .iter()
        .map(|value| match value {
            PartitionBoundValue::MinValue => RangeDatum::MinValue,
            PartitionBoundValue::MaxValue => RangeDatum::MaxValue,
            PartitionBoundValue::Expr(expr) => RangeDatum::Value(expr.to_string()),
        })
        .collect()
}

/// One entry of a list bound, `NULL` as the catalog writes it and every
/// other value as the expression reads back.
fn list_datum(value: Expr) -> ListDatum {
    match value {
        Expr::Value(ValueWithSpan {
            value: Value::Null, ..
        }) => ListDatum::Null,
        other => ListDatum::Value(other.to_string()),
    }
}

impl PgTable {
    /// Pairs an `information_schema.tables` row with the facts
    /// `pg_catalog` records for it.
    ///
    /// `partstrat` is the `pg_catalog.pg_partitioned_table.partstrat` value
    /// recorded for the table, or [`None`] when the table is not partitioned.
    ///
    /// # Errors
    ///
    /// Returns [`PgTableError::UnknownPartitionStrategy`] when `partstrat`
    /// is neither `l`, `r` nor `h`, rather than reading an unrecognised
    /// strategy as no partitioning at all, and
    /// [`PgTableError::UnparsablePartitionBound`] when the bound the catalog
    /// recorded does not read back.
    pub fn new(
        model: Table,
        partstrat: Option<&str>,
        facts: &TableFacts,
    ) -> Result<Self, PgTableError> {
        let partition_strategy = match partstrat {
            None => None,
            Some("l") => Some(PartitionStrategy::List),
            Some("r") => Some(PartitionStrategy::Range),
            Some("h") => Some(PartitionStrategy::Hash),
            Some(other) => return Err(PgTableError::UnknownPartitionStrategy(other.to_owned())),
        };

        let (storage_parameters, toast_storage_parameters) =
            route_storage_parameters(&facts.reloptions, &facts.toast_reloptions);
        let partition_bound = facts
            .partition_bound
            .as_deref()
            .map(parse_partition_bound)
            .transpose()?;

        Ok(Self {
            model,
            partition_strategy,
            unlogged: facts.unlogged,
            storage_parameters,
            toast_storage_parameters,
            partition_bound,
        })
    }

    /// Returns the `information_schema.tables` row describing the table.
    #[must_use]
    pub fn model(&self) -> &Table {
        &self.model
    }

    /// Returns the catalog the table belongs to.
    #[must_use]
    pub fn catalog(&self) -> &str {
        &self.model.table_catalog
    }

    /// Returns the schema the table belongs to.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.model.table_schema
    }

    /// Returns the name of the table.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.model.table_name
    }

    /// Returns whether the table is `UNLOGGED`.
    #[must_use]
    pub fn unlogged(&self) -> bool {
        self.unlogged
    }

    /// Returns the storage parameters `pg_class.reloptions` records.
    #[must_use]
    pub fn storage_parameters(&self) -> &[(String, String)] {
        &self.storage_parameters
    }

    /// Returns the storage parameters of the table's TOAST relation.
    #[must_use]
    pub fn toast_storage_parameters(&self) -> &[(String, String)] {
        &self.toast_storage_parameters
    }

    /// Returns the bound the table takes as a partition.
    #[must_use]
    pub fn partition_bound(&self) -> Option<&PartitionBound> {
        self.partition_bound.as_ref()
    }

    /// Returns how the table routes rows to its partitions.
    #[must_use]
    pub fn partition_strategy(&self) -> Option<PartitionStrategy> {
        self.partition_strategy
    }

    /// Initializes and returns the metadata for the table.
    ///
    /// # Errors
    ///
    /// * If the metadata cannot be loaded from the database.
    pub fn metadata(
        self: &std::sync::Arc<Self>,
        conn: &mut diesel::PgConnection,
        denylist_types: &[String],
        cache: &crate::database::CatalogCache,
    ) -> Result<crate::model_metadata::TableMetadata, diesel::result::Error> {
        self.model.metadata(self, conn, denylist_types, cache)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries(values: &[&str]) -> Vec<String> {
        values.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn storage_entries_read_names_and_values_as_stored() {
        assert_eq!(
            storage_option_entries(&entries(&[
                "fillfactor=70",
                "autovacuum_enabled",
                "toast_tuple_target=256"
            ])),
            vec![
                ("fillfactor".to_string(), "70".to_string()),
                ("autovacuum_enabled".to_string(), "true".to_string()),
                ("toast_tuple_target".to_string(), "256".to_string()),
            ],
        );
        assert_eq!(
            storage_option_entries(&entries(&[])),
            Vec::<(String, String)>::new()
        );
    }

    #[test]
    fn toast_namespaced_entries_route_to_the_toast_side() {
        let (storage, toast) = route_storage_parameters(
            &entries(&["fillfactor=70", "toast.autovacuum_enabled=false"]),
            &entries(&["vacuum_truncate"]),
        );
        assert_eq!(storage, vec![("fillfactor".to_string(), "70".to_string())],);
        assert_eq!(
            toast,
            vec![
                ("autovacuum_enabled".to_string(), "false".to_string()),
                ("vacuum_truncate".to_string(), "true".to_string()),
            ],
        );
    }

    #[test]
    fn partition_bounds_read_back_through_the_partition_of_form() {
        let bound =
            parse_partition_bound("FOR VALUES IN ('eu',NULL)").expect("the catalog text parses");
        assert_eq!(
            bound,
            PartitionBound::List {
                values: vec![ListDatum::Value("'eu'".to_string()), ListDatum::Null],
            },
        );

        let bound =
            parse_partition_bound("FOR VALUES FROM (1) TO (9)").expect("the catalog text parses");
        assert_eq!(
            bound,
            PartitionBound::Range {
                from: vec![RangeDatum::Value("1".to_string())],
                to: vec![RangeDatum::Value("9".to_string())],
            },
        );

        let bound = parse_partition_bound("FOR VALUES WITH (MODULUS 4, REMAINDER 0)")
            .expect("the catalog text parses");
        assert_eq!(
            bound,
            PartitionBound::Hash {
                modulus: 4,
                remainder: 0,
            },
        );

        let bound = parse_partition_bound("DEFAULT").expect("the catalog text parses");
        assert_eq!(bound, PartitionBound::Default);
    }

    #[test]
    #[should_panic]
    fn partition_bound_rejects_text_without_a_bound() {
        let _ = parse_partition_bound("PARTITION BY RANGE (id)").unwrap();
    }
}
