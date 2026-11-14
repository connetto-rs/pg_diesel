//! Submodule providing the `PgStat` struct representing a row of the
//! `pg_stats` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_stats` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_stats::pg_stats)]
pub struct PgStat {
    /// Schema name.
    pub schemaname: Option<String>,
    /// Table name.
    pub tablename: Option<String>,
    /// Column name.
    pub attname: Option<String>,
    /// Inherited stats flag.
    pub inherited: Option<bool>,
    /// Null fraction.
    pub null_frac: Option<f32>,
    /// Average width.
    pub avg_width: Option<i32>,
    /// Number of distinct values.
    pub n_distinct: Option<f32>,
    /// Most common value frequencies.
    pub most_common_freqs: Option<Vec<f32>>,
    /// Correlation coefficient.
    pub correlation: Option<f32>,
    /// Most common element frequencies.
    pub most_common_elem_freqs: Option<Vec<f32>>,
    /// Element count histogram.
    pub elem_count_histogram: Option<Vec<f32>>,
    /// Range empty fraction.
    /// Added in `PostgreSQL` 17.
    #[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
    pub range_empty_frac: Option<f32>,
}
