//! Submodule providing the `PgPartitionedTable` struct representing a row of
//! the `pg_partitioned_table` table in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_partitioned_table` table.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_partitioned_table::pg_partitioned_table)]
pub struct PgPartitionedTable {
    /// OID of the partitioned table.
    pub partrelid: u32,
    /// Partitioning strategy.
    pub partstrat: String,
    /// Number of columns in partition key.
    pub partnatts: i16,
    /// OID of the default partition.
    pub partdefid: u32,
    /// Array of attribute numbers of the partition key columns.
    pub partattrs: Vec<i16>,
    /// Array of operator class OIDs for the partition key columns.
    pub partclass: Vec<u32>,
    /// Array of collation OIDs for the partition key columns.
    pub partcollation: Vec<u32>,
    /// Partition key expressions.
    pub partexprs: Option<String>,
}
