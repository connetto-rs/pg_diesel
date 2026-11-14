//! Struct representing a row in the `information_schema.domain_udt_usage`
//! table.

use diesel::{Queryable, QueryableByName, Selectable};

/// Struct defining the `information_schema.domain_udt_usage` table.
#[derive(
    Queryable, QueryableByName, Selectable, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::information_schema::domain_udt_usage::domain_udt_usage)]
pub struct DomainUdtUsage {
    /// Catalog (database) containing the user-defined type.
    pub udt_catalog: Option<String>,
    /// Schema containing the user-defined type.
    pub udt_schema: Option<String>,
    /// Name of the user-defined type.
    pub udt_name: Option<String>,
    /// Catalog (database) containing the domain.
    pub domain_catalog: Option<String>,
    /// Schema containing the domain.
    pub domain_schema: Option<String>,
    /// Name of the domain.
    pub domain_name: Option<String>,
}
