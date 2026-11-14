//! Struct representing a row in the `information_schema.collations` table.

use diesel::{Queryable, QueryableByName, Selectable};

/// Struct defining the `information_schema.collations` table.
#[derive(
    Queryable, QueryableByName, Selectable, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::information_schema::collations::collations)]
pub struct Collation {
    /// Catalog (database) containing the collation.
    /// This identifies the database where the collation is defined.
    pub collation_catalog: Option<String>,
    /// Schema containing the collation.
    /// This identifies the schema where the collation is defined.
    pub collation_schema: Option<String>,
    /// Name of the collation.
    /// This is the identifier for the specific collation.
    pub collation_name: Option<String>,
    /// Padding attribute for the collation; typically "PAD SPACE" or "NO PAD".
    /// This defines how trailing spaces are handled in string comparisons.
    pub pad_attribute: Option<String>,
}
