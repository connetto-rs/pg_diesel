//! Column model.

use std::fmt::Display;

use diesel::{Queryable, QueryableByName, Selectable};

/// Struct defining the `information_schema.columns` table.
#[derive(
    Queryable, QueryableByName, Selectable, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::information_schema::columns::columns)]
pub struct Column {
    /// Name of the database containing the table (always the current database)
    pub table_catalog: String,
    /// Name of the schema containing the table
    pub table_schema: String,
    /// Name of the table containing the column
    pub table_name: String,
    /// Name of the column
    pub column_name: String,
    /// Ordinal position of the column within the table (co nt starts at 1)
    pub ordinal_position: i32,
    /// Default expression of the column
    pub column_default: Option<String>,
    /// Indicates if the column is nullable ("YES" or "NO")
    #[allow(clippy::pub_underscore_fields)]
    pub __is_nullable: String,
    /// Data type of the column
    data_type: String,
    /// Maximum length of the character data type
    pub character_maximum_length: Option<i32>,
    /// Maximum length in bytes of the character data type
    pub character_octet_length: Option<i32>,
    /// Catalog containing the character set of the column
    pub character_set_catalog: Option<String>,
    /// Schema containing the character set of the column
    pub character_set_schema: Option<String>,
    /// Name of the character set of the column
    pub character_set_name: Option<String>,
    /// Catalog containing the collation of the column
    pub collation_catalog: Option<String>,
    /// Schema containing the collation of the column
    pub collation_schema: Option<String>,
    /// Name of the collation of the column
    pub collation_name: Option<String>,
    /// Precision of the numeric data type
    pub numeric_precision: Option<i32>,
    /// Radix (base) of the numeric data type
    pub numeric_precision_radix: Option<i32>,
    /// Scale of the numeric data type
    pub numeric_scale: Option<i32>,
    /// Precision of the datetime data type
    pub datetime_precision: Option<i32>,
    /// Interval type of the interval data type
    pub interval_type: Option<String>,
    /// Precision of the interval data type
    pub interval_precision: Option<i32>,
    /// Catalog name of the underlying type of the column
    pub udt_catalog: Option<String>,
    /// Schema name of the underlying type of the column
    pub udt_schema: Option<String>,
    /// Name of the UDT; `NULL` if not applicable.
    pub udt_name: Option<String>,
    /// Catalog of the domain type
    pub domain_catalog: Option<String>,
    /// Schema of the domain type
    pub domain_schema: Option<String>,
    /// Name of the domain type
    pub domain_name: Option<String>,
    /// Catalog name of the scope of the column
    pub scope_catalog: Option<String>,
    /// Schema name of the scope of the column
    pub scope_schema: Option<String>,
    /// Name of the scope of the column
    pub scope_name: Option<String>,
    /// Maximum cardinality of the column
    pub maximum_cardinality: Option<i32>,
    /// Identifier of the data type descriptor
    pub dtd_identifier: Option<String>,
    /// Indicates if the column is self-referencing
    pub is_self_referencing: Option<String>,
    /// "YES" if the column is an identity column; "NO" otherwise.
    pub is_identity: Option<String>,
    /// "ALWAYS" or "BY DEFAULT" for identity columns
    pub identity_generation: Option<String>,
    /// Start value of the identity sequence
    pub identity_start: Option<String>,
    /// Increment of the identity sequence
    pub identity_increment: Option<String>,
    /// Maximum value of the identity sequence
    pub identity_maximum: Option<String>,
    /// Minimum value of the identity sequence
    pub identity_minimum: Option<String>,
    /// "YES" if identity sequence cycles, "NO" otherwise
    pub identity_cycle: Option<String>,
    /// "ALWAYS", "BY DEFAULT", or empty string for computed/generated columns.
    pub is_generated: String,
    /// Generation expression of the column
    pub generation_expression: Option<String>,
    /// Indicates if the column is updatable ("YES" or "NO")
    pub is_updatable: String,
}

impl Display for Column {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "`{}.{}.{}`",
            self.table_schema, self.table_name, self.column_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_column() -> Column {
        Column {
            table_catalog: "db".to_string(),
            table_schema: "schema".to_string(),
            table_name: "table".to_string(),
            column_name: "column".to_string(),
            ordinal_position: 1,
            column_default: None,
            __is_nullable: "NO".to_string(),
            data_type: "integer".to_string(),
            character_maximum_length: None,
            character_octet_length: None,
            character_set_catalog: None,
            character_set_schema: None,
            character_set_name: None,
            collation_catalog: None,
            collation_schema: None,
            collation_name: None,
            numeric_precision: None,
            numeric_precision_radix: None,
            numeric_scale: None,
            datetime_precision: None,
            interval_type: None,
            interval_precision: None,
            udt_catalog: None,
            udt_schema: None,
            udt_name: None,
            domain_catalog: None,
            domain_schema: None,
            domain_name: None,
            scope_catalog: None,
            scope_schema: None,
            scope_name: None,
            maximum_cardinality: None,
            dtd_identifier: None,
            is_self_referencing: None,
            is_identity: None,
            identity_generation: None,
            identity_start: None,
            identity_increment: None,
            identity_maximum: None,
            identity_minimum: None,
            identity_cycle: None,
            is_generated: "NO".to_string(),
            generation_expression: None,
            is_updatable: "YES".to_string(),
        }
    }

    #[test]
    fn test_display() {
        let col = dummy_column();
        assert_eq!(format!("{col}"), "`schema.table.column`");
    }
}
