//! Submodule providing the `PgStatsExtExpr` struct representing a row of the
//! `pg_stats_ext_exprs` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_stats_ext_exprs` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_stats_ext_exprs::pg_stats_ext_exprs)]
pub struct PgStatsExtExpr {
    /// Schema name.
    pub schemaname: Option<String>,
    /// Table name.
    pub tablename: Option<String>,
    /// Statistics schema name.
    pub statistics_schemaname: Option<String>,
    /// Statistics name.
    pub statistics_name: Option<String>,
    /// Statistics owner.
    pub statistics_owner: Option<String>,
    /// Expression text.
    pub expr: Option<String>,
    /// Inherited stats flag (`PostgreSQL` 15+).
    #[cfg(not(feature = "postgres-14"))]
    pub inherited: Option<bool>,
    /// Null fraction.
    pub null_frac: Option<f32>,
    /// Average width.
    pub avg_width: Option<i32>,
    /// Number of distinct values.
    pub n_distinct: Option<f32>,
    /// Most common values.
    pub most_common_vals: Option<Vec<Vec<u8>>>,
    /// Most common value frequencies.
    pub most_common_freqs: Option<Vec<f32>>,
    /// Histogram bounds.
    pub histogram_bounds: Option<Vec<Vec<u8>>>,
    /// Correlation coefficient.
    pub correlation: Option<f32>,
    /// Most common elements.
    pub most_common_elems: Option<Vec<Vec<u8>>>,
    /// Most common element frequencies.
    pub most_common_elem_freqs: Option<Vec<f32>>,
    /// Element count histogram.
    pub elem_count_histogram: Option<Vec<f32>>,
}
