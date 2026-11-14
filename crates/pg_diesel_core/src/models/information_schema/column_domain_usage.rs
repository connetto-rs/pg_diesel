//! Struct representing a row in the `information_schema.column_domain_usage`
//! table.

use diesel::{Queryable, QueryableByName, Selectable};

/// Struct defining the `information_schema.column_domain_usage` table.
#[derive(
    Queryable, QueryableByName, Selectable, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::information_schema::column_domain_usage::column_domain_usage)]
pub struct ColumnDomainUsage {
    /// Catalog (database) containing the domain.
    pub domain_catalog: Option<String>,
    /// Schema containing the domain.
    pub domain_schema: Option<String>,
    /// Name of the domain.
    pub domain_name: Option<String>,
    /// Catalog (database) containing the table.
    pub table_catalog: Option<String>,
    /// Schema containing the table.
    pub table_schema: Option<String>,
    /// Name of the table.
    pub table_name: Option<String>,
    /// Name of the column that uses the domain.
    pub column_name: Option<String>,
}
