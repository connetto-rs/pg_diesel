//! Trait for types with `PostgreSQL` Object Identifiers (OIDs).

/// Trait for types that have a `PostgreSQL` Object Identifier (OID).
pub trait HasOid {
    /// Returns the OID of the struct.
    fn oid(&self) -> u32;
}
