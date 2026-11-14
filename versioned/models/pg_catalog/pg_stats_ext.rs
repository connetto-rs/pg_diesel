//! Submodule providing the `PgStatsExt` struct representing a row of the
//! `pg_stats_ext` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_stats_ext` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_stats_ext::pg_stats_ext)]
pub struct PgStatsExt {
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
    /// Attribute names.
    pub attnames: Option<Vec<String>>,
    /// Expressions.
    pub exprs: Option<Vec<String>>,
    /// Statistics kinds.
    pub kinds: Option<Vec<String>>,
    /// Inherited stats flag (`PostgreSQL` 15+).
    #[cfg(not(feature = "postgres-14"))]
    pub inherited: Option<bool>,
    /// N-distinct counts (`pg_ndistinct` type).
    pub n_distinct: Option<Vec<u8>>,
    /// Functional dependencies (`pg_dependencies` type).
    pub dependencies: Option<Vec<u8>>,
    /// Most common values.
    pub most_common_vals: Option<Vec<String>>,
    /// Most common value nulls flags.
    pub most_common_val_nulls: Option<Vec<bool>>,
    /// Most common frequencies.
    pub most_common_freqs: Option<Vec<f64>>,
    /// Most common base frequencies.
    pub most_common_base_freqs: Option<Vec<f64>>,
}
