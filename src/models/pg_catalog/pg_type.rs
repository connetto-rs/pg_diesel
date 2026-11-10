//! Submodule providing the [`PgType`] struct.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a `PostgreSQL` type.
#[derive(
    Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_type::pg_type)]
#[allow(clippy::struct_excessive_bools)]
pub struct PgType {
    /// The OID (Object Identifier) of the type.
    pub oid: u32,
    /// The name of the type.
    pub typname: String,
    /// The namespace (schema) of the type.
    pub typnamespace: u32,
    /// The owner of the type.
    pub typowner: u32,
    /// The size of the type in bytes.
    pub typlen: i16,
    /// Indicates if the type is passed by value.
    pub typbyval: bool,
    /// The type of the type.
    pub typtype: String,
    /// The category of the type.
    pub typcategory: String,
    /// Indicates if the type is preferred within its category.
    pub typispreferred: bool,
    /// Indicates if the type is defined.
    pub typisdefined: bool,
    /// The delimiter for array elements of this type.
    pub typdelim: String,
    /// The relation ID for a composite type.
    pub typrelid: u32,
    /// The element type of an array type.
    pub typelem: u32,
    /// The array type of a base type.
    pub typarray: u32,
    /// The input function for the type.
    pub typinput: u32,
    /// The output function for the type.
    pub typoutput: u32,
    /// The receive function for the type.
    pub typreceive: u32,
    /// The send function for the type.
    pub typsend: u32,
    /// The modifier input function for the type.
    pub typmodin: u32,
    /// The modifier output function for the type.
    pub typmodout: u32,
    /// The analyze function for the type.
    pub typanalyze: u32,
    #[cfg(any(
        feature = "postgres-14",
        feature = "postgres-15",
        feature = "postgres-16",
        feature = "postgres-17",
        feature = "postgres-18"
    ))]
    /// The subscripting function for the type.
    pub typsubscript: u32,
    /// The alignment requirement of the type.
    pub typalign: String,
    /// The storage strategy for the type.
    pub typstorage: String,
    /// Indicates if the type is not nullable.
    pub typnotnull: bool,
    /// The base type of a domain type.
    pub typbasetype: u32,
    /// The type modifier.
    pub typtypmod: i32,
    /// The number of dimensions for an array type.
    pub typndims: i32,
    /// The collation for the type.
    pub typcollation: u32,
    /// The default binary representation of the type.
    pub typdefaultbin: Option<Vec<u8>>,
    /// The default text representation of the type.
    pub typdefault: Option<String>,
}
