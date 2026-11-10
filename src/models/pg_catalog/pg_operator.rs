//! This module defines the `PgOperator` model representing the
//! `pg_catalog.pg_operator` system catalog table in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents the `pg_operator` system catalog table in `PostgreSQL`.
/// This table stores information about operators.
#[derive(Queryable, QueryableByName, Selectable, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_operator::pg_operator)]
pub struct PgOperator {
    /// The OID of the operator.
    pub oid: u32,
    /// The name of the operator.
    pub oprname: String,
    /// The OID of the namespace that contains this operator.
    pub oprnamespace: u32,
    /// The OID of the owner of the operator.
    pub oprowner: u32,
    /// The kind of operator ('l' for left unary, 'r' for right unary, 'b' for
    /// binary).
    pub oprkind: String,
    /// True if the operator can be merged.
    pub oprcanmerge: bool,
    /// True if the operator can be hashed.
    pub oprcanhash: bool,
    /// The OID of the left operand type.
    pub oprleft: u32,
    /// The OID of the right operand type.
    pub oprright: u32,
    /// The OID of the result type.
    pub oprresult: u32,
    /// The OID of the commutator operator, or 0 if none.
    pub oprcom: u32,
    /// The OID of the negator operator, or 0 if none.
    pub oprnegate: u32,
    /// The OID of the function implementing the operator.
    pub oprcode: u32,
    /// The OID of the restriction selectivity estimator function, or 0 if none.
    pub oprrest: u32,
    /// The OID of the join selectivity estimator function, or 0 if none.
    pub oprjoin: u32,
}
