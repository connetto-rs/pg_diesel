//! Submodule providing a struct [`PgProc`] representing the `pg_proc` table.

use diesel::{PgConnection, Queryable, QueryableByName, Selectable};

use crate::models::PgExtension;

mod cached_queries;
pub use cached_queries::ProcAclEntry;

/// Represents the `pg_proc` system catalog table in `PostgreSQL`.
/// This table stores information about functions and procedures.
#[derive(Queryable, QueryableByName, Selectable, Debug, Clone, PartialEq)]
#[diesel(table_name = crate::schema::pg_catalog::pg_proc::pg_proc)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::struct_excessive_bools)]
pub struct PgProc {
    /// The OID of the function.
    pub oid: u32,
    /// The name of the function.
    pub proname: String,
    /// The OID of the namespace that contains this function.
    pub pronamespace: u32,
    /// The OID of the owner of the function.
    pub proowner: u32,
    /// The OID of the language in which the function is implemented.
    pub prolang: u32,
    /// The estimated execution cost of the function.
    pub procost: f32,
    /// The estimated number of rows returned by the function.
    pub prorows: f32,
    /// The OID of the variadic argument type, or 0 if none.
    pub provariadic: u32,
    /// The OID of the support function, or 0 if none.
    pub prosupport: u32,
    /// The kind of function ('f' for normal, 'p' for procedure, etc.).
    pub prokind: String,
    /// True if the function is a security definer.
    pub prosecdef: bool,
    /// True if the function is leakproof.
    pub proleakproof: bool,
    /// True if the function is strict (null in, null out).
    pub proisstrict: bool,
    /// True if the function returns a set.
    pub proretset: bool,
    /// The volatility category of the function ('i' for immutable, 's' for
    /// stable, 'v' for volatile).
    pub provolatile: String,
    /// The parallel safety category of the function ('u' for unsafe, 'r' for
    /// restricted, 's' for safe).
    pub proparallel: String,
    /// The number of arguments the function takes.
    pub pronargs: i16,
    /// The number of arguments with default values.
    pub pronargdefaults: i16,
    /// The OID of the return type.
    pub prorettype: u32,
    /// An array of OIDs of the argument types.
    pub proargtypes: Vec<u32>,
    /// An array of OIDs of all argument types, including OUT parameters.
    pub proallargtypes: Option<Vec<u32>>,
    /// An array of modes of the arguments ('i' for IN, 'o' for OUT, etc.).
    pub proargmodes: Option<Vec<String>>,
    /// An array of names of the arguments.
    pub proargnames: Option<Vec<String>>,
    /// Default values for arguments (as a node tree representation).
    pub proargdefaults: Option<String>,
    /// Array of type OIDs for output of polymorphic functions.
    pub protrftypes: Option<Vec<u32>>,
    /// The source code of the function.
    pub prosrc: String,
    /// The binary representation of the function.
    pub probin: Option<Vec<u8>>,
    /// The SQL body of the function, if any.
    /// Added in `PostgreSQL` 14.
    #[cfg(any(
        feature = "postgres-14",
        feature = "postgres-15",
        feature = "postgres-16",
        feature = "postgres-17",
        feature = "postgres-18"
    ))]
    pub prosqlbody: Option<String>,
    /// The configuration settings for the function.
    pub proconfig: Option<Vec<String>>,
}

impl PgProc {
    /// Returns the [`PgExtension`] that contains this function, if any.
    ///
    /// # Errors
    ///
    /// * If the function is not contained in an extension
    pub fn extension(&self, conn: &mut PgConnection) -> Result<PgExtension, diesel::result::Error> {
        cached_queries::extension(self, conn)
    }

    /// Loads every `pg_proc` row except the procedures.
    ///
    /// # Errors
    ///
    /// * If the database query fails.
    pub fn load_all(conn: &mut PgConnection) -> Result<Vec<PgProc>, diesel::result::Error> {
        cached_queries::load_all(conn)
    }

    /// Loads one entry per `EXECUTE` grant `pg_proc.proacl` records, for
    /// every function [`load_all`](Self::load_all) holds.
    ///
    /// # Errors
    ///
    /// * If the database query fails.
    pub fn load_all_execute_grants(
        conn: &mut PgConnection,
    ) -> Result<Vec<ProcAclEntry>, diesel::result::Error> {
        cached_queries::load_all_execute_grants(conn)
    }

    /// Loads the definition `pg_get_functiondef` prints for every function
    /// whose body `PostgreSQL` bound at creation, beside its OID.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the query fails.
    pub fn load_bound_definitions(
        conn: &mut PgConnection,
    ) -> Result<Vec<(u32, String)>, diesel::result::Error> {
        cached_queries::load_bound_definitions(conn)
    }

    /// Loads `(function, reached)` OID pairs, one per function a bound body
    /// reached.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the query fails.
    pub fn load_bound_dependencies(
        conn: &mut PgConnection,
    ) -> Result<Vec<(u32, u32)>, diesel::result::Error> {
        cached_queries::load_bound_dependencies(conn)
    }
}

// Manual implementations of Eq, Ord, PartialOrd, and Hash
// These are required because PgProc contains f32 fields (procost, prorows)
// which don't implement these traits. We use the OID as the primary key for
// ordering and hashing since it uniquely identifies a function.

impl Eq for PgProc {}

impl Ord for PgProc {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.oid.cmp(&other.oid)
    }
}

impl PartialOrd for PgProc {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::hash::Hash for PgProc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.oid.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_pg_proc(oid: u32) -> PgProc {
        PgProc {
            oid,
            proname: "func".to_string(),
            pronamespace: 1,
            proowner: 1,
            prolang: 1,
            procost: 1.0,
            prorows: 1.0,
            provariadic: 0,
            prosupport: 0,
            prokind: "f".to_string(),
            prosecdef: false,
            proleakproof: false,
            proisstrict: true,
            proretset: false,
            provolatile: "i".to_string(),
            proparallel: "s".to_string(),
            pronargs: 0,
            pronargdefaults: 0,
            prorettype: 0,
            proargtypes: vec![],
            proallargtypes: None,
            proargmodes: None,
            proargnames: None,
            proargdefaults: None,
            protrftypes: None,
            prosrc: "src".to_string(),
            probin: None,
            #[cfg(any(
                feature = "postgres-14",
                feature = "postgres-15",
                feature = "postgres-16",
                feature = "postgres-17",
                feature = "postgres-18"
            ))]
            prosqlbody: None,
            proconfig: None,
        }
    }

    #[test]
    fn test_eq() {
        let p1 = dummy_pg_proc(1);
        let p2 = dummy_pg_proc(1);
        let p3 = dummy_pg_proc(2);

        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }

    #[test]
    fn test_ord() {
        let p1 = dummy_pg_proc(1);
        let p2 = dummy_pg_proc(2);

        assert!(p1 < p2);
        assert!(p2 > p1);
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(dummy_pg_proc(1));
        assert!(set.contains(&dummy_pg_proc(1)));
        assert!(!set.contains(&dummy_pg_proc(2)));
    }
}
