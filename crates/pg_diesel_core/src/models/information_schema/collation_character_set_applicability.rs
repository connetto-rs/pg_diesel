//! Struct representing a row in the
//! `information_schema.collation_character_set_applicability` table.

use diesel::{Queryable, QueryableByName, Selectable};

/// Struct defining the
/// `information_schema.collation_character_set_applicability` table.
#[derive(
    Queryable, QueryableByName, Selectable, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::information_schema::collation_character_set_applicability::collation_character_set_applicability)]
pub struct CollationCharacterSetApplicability {
    /// Catalog (database) containing the collation.
    /// This identifies the database where the collation is defined.
    pub collation_catalog: Option<String>,
    /// Schema containing the collation.
    /// This identifies the schema where the collation is defined.
    pub collation_schema: Option<String>,
    /// Name of the collation.
    /// This is the identifier for the specific collation (e.g., "en_US.UTF-8",
    /// "C").
    pub collation_name: Option<String>,
    /// Catalog (database) containing the character set that this collation
    /// applies to. This identifies the database where the compatible
    /// character set is defined.
    pub character_set_catalog: Option<String>,
    /// Schema containing the character set that this collation applies to.
    /// This identifies the schema where the compatible character set is
    /// defined.
    pub character_set_schema: Option<String>,
    /// Name of the character set that this collation applies to.
    /// This is the identifier for the character set that is compatible with the
    /// collation.
    pub character_set_name: Option<String>,
}
