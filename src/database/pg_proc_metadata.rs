//! Metadata for `PostgreSQL` functions and procedures.

use crate::{
    database::CatalogCache,
    models::{PgProc, PgType, ProcAclEntry},
};

/// Returns the name a function declares a type under, the way the session
/// the metadata was read in resolves it, which `format_type` answers with
/// the schema prefix when the type is not visible on the search path.
fn type_name(oid: u32, cache: &CatalogCache, pg_type: &PgType) -> String {
    cache
        .type_name(oid)
        .map_or_else(|| pg_type.typname.clone(), ToOwned::to_owned)
}

/// The OID `pg_type` records the `record` pseudo-type under.
const RECORD_TYPE_OID: u32 = 2249;

/// Who holds `EXECUTE` on a function, as `pg_proc.proacl` records it.
///
/// A function starts out executable by `PUBLIC` and by its owner. Each other
/// grantee holds one entry, which a later grant to the same grantee widens
/// rather than repeats, and a `REVOKE` takes the entry away.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ExecutePrivileges {
    /// Whether `PUBLIC` holds `EXECUTE`.
    public: bool,
    /// Whether the owner holds its own entry.
    owner: bool,
    /// The entries of every other grantee, with whether the grant was made
    /// `WITH GRANT OPTION`.
    grants: Vec<(u32, bool)>,
}

impl ExecutePrivileges {
    /// Decodes the `aclexplode` rows of one function, `None` when its
    /// `proacl` is `NULL`.
    ///
    /// A `NULL` ACL is the default the server starts a function with, under
    /// which `PUBLIC` and the owner hold `EXECUTE` and no other grantee does.
    /// A set ACL holds exactly the entries its rows list, none at all when a
    /// single row carries no grantee.
    fn decode(rows: Option<&[ProcAclEntry]>, proowner: u32) -> Self {
        let Some(rows) = rows else {
            return Self {
                public: true,
                owner: true,
                grants: Vec::new(),
            };
        };

        let mut privileges = Self {
            public: false,
            owner: false,
            grants: Vec::new(),
        };
        for row in rows {
            match row.grantee {
                None => {}
                Some(0) => privileges.public = true,
                Some(grantee) if grantee == proowner => privileges.owner = true,
                Some(grantee) => privileges
                    .grants
                    .push((grantee, row.grantable.unwrap_or_default())),
            }
        }
        privileges
    }
}

/// One holder of `EXECUTE` a function's access list names directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecuteHolder {
    /// Every role, through the `PUBLIC` pseudo-role.
    Public,
    /// A role, by the OID `pg_roles` records it under.
    Role(u32),
}

#[derive(Debug, Clone)]
/// Struct collecting metadata about a `PostgreSQL` function represented by a
/// [`PgProc`] entry.
pub struct PgProcMetadata {
    /// Every argument type the function declares, `OUT` included.
    argument_types: Vec<PgType>,
    /// The name each argument type declares under, `OUT` included, in the
    /// spelling the reading session resolves it to.
    argument_type_names: Vec<String>,
    /// The return type.
    return_type: Option<PgType>,
    /// The return type the function declares, the `SETOF` marker included.
    return_type_name: Option<String>,
    /// The type one call of the function yields.
    result_type_name: Option<String>,
    /// The role owning the function.
    owner: Option<String>,
    /// The OID the catalog records the owner under.
    owner_oid: u32,
    /// Who holds `EXECUTE` on the function.
    execute: ExecutePrivileges,
    /// The OIDs of the functions a body bound at creation reached, or the
    /// parser's complaint when its printed definition does not parse.
    bound_functions: Result<Vec<u32>, String>,
}

impl PgProcMetadata {
    /// Reads the metadata of `pg_proc` from the catalogs already loaded.
    ///
    /// A type or role the cache does not hold is left out rather than looked
    /// up: the cache reads whole catalogs, so a miss means the object is gone.
    ///
    #[must_use]
    pub fn new(
        pg_proc: &PgProc,
        cache: &CatalogCache,
        execute_grants: Option<&[ProcAclEntry]>,
    ) -> Self {
        // `proallargtypes` is `NULL` for a function every argument of which is `IN`.
        let argument_oids: &[u32] = pg_proc
            .proallargtypes
            .as_deref()
            .unwrap_or(pg_proc.proargtypes.as_slice());
        let mut argument_types: Vec<PgType> = Vec::new();
        let mut argument_type_names: Vec<String> = Vec::new();
        for oid in argument_oids {
            if let Some(pg_type) = cache.pg_type(*oid) {
                argument_type_names.push(type_name(*oid, cache, pg_type));
                argument_types.push(pg_type.clone());
            }
        }
        let return_type = cache.pg_type(pg_proc.prorettype).cloned();
        let return_type_name = Self::declared_return_type(pg_proc, return_type.as_ref(), cache);
        let result_type_name =
            Self::yielded_result_type(pg_proc, return_type.as_ref(), &argument_types, cache);
        let owner = cache.role(pg_proc.proowner).map(ToOwned::to_owned);
        let owner_oid = pg_proc.proowner;

        Self {
            argument_types,
            argument_type_names,
            return_type,
            return_type_name,
            result_type_name,
            owner,
            owner_oid,
            execute: ExecutePrivileges::decode(execute_grants, owner_oid),
            bound_functions: Ok(Vec::new()),
        }
    }

    /// Records the functions a body bound at creation reached, or why its
    /// printed definition could not be read.
    #[must_use]
    pub fn with_bound_functions(mut self, bound_functions: Result<Vec<u32>, String>) -> Self {
        self.bound_functions = bound_functions;
        self
    }

    /// Returns the OIDs of the functions a body bound at creation reached.
    ///
    /// # Errors
    ///
    /// Returns the parser's complaint when the definition `PostgreSQL`
    /// prints for the bound body does not parse.
    pub fn bound_functions(&self) -> Result<&[u32], &str> {
        self.bound_functions.as_deref().map_err(String::as_str)
    }

    /// The entries the `EXECUTE` access list holds, `PUBLIC` first while it
    /// holds the privilege, then the owner while its own entry holds, then every
    /// other grantee in the order first granted, each role once.
    pub fn execute_holders(&self) -> impl Iterator<Item = ExecuteHolder> {
        let owner = self
            .execute
            .owner
            .then_some(ExecuteHolder::Role(self.owner_oid));
        let mut held = std::collections::HashSet::new();
        let grantees: Vec<ExecuteHolder> = self
            .execute
            .grants
            .iter()
            .filter_map(|(grantee, _)| held.insert(*grantee).then_some(*grantee))
            .map(ExecuteHolder::Role)
            .collect();
        self.execute
            .public
            .then_some(ExecuteHolder::Public)
            .into_iter()
            .chain(owner)
            .chain(grantees)
    }

    /// The return type the function declares, the `SETOF` marker included.
    ///
    /// A result described by output arguments declares no return type, which
    /// `pg_proc` records as `record` alongside the output arguments.
    fn declared_return_type(
        pg_proc: &PgProc,
        return_type: Option<&PgType>,
        cache: &CatalogCache,
    ) -> Option<String> {
        let pg_type = return_type?;
        if pg_type.oid == RECORD_TYPE_OID && Self::has_output_arguments(pg_proc) {
            return None;
        }

        let name = type_name(pg_type.oid, cache, pg_type);
        Some(if pg_proc.proretset {
            format!("SETOF {name}")
        } else {
            name
        })
    }

    /// Whether the function declares an `OUT` or `INOUT` argument.
    fn has_output_arguments(pg_proc: &PgProc) -> bool {
        pg_proc
            .proargmodes
            .iter()
            .flatten()
            .any(|mode| matches!(mode.as_str(), "o" | "b"))
    }

    /// The type one call of the function yields.
    ///
    /// `prorettype` names it when it is a concrete type. `record`, which
    /// `PostgreSQL` records for a result described by output arguments,
    /// yields the type of the single `OUT` or `INOUT` argument when the
    /// function has one and `record` when it has several.
    fn yielded_result_type(
        pg_proc: &PgProc,
        return_type: Option<&PgType>,
        argument_types: &[PgType],
        cache: &CatalogCache,
    ) -> Option<String> {
        let pg_type = return_type?;
        if pg_type.oid != RECORD_TYPE_OID {
            return Some(type_name(pg_type.oid, cache, pg_type));
        }

        match Self::single_output_argument_type(pg_proc, argument_types, cache) {
            Some(name) => Some(name),
            None => Some("record".to_owned()),
        }
    }

    /// The type the single `OUT` or `INOUT` argument of the function
    /// declares.
    fn single_output_argument_type(
        pg_proc: &PgProc,
        argument_types: &[PgType],
        cache: &CatalogCache,
    ) -> Option<String> {
        let outputs: Vec<usize> = pg_proc
            .proargmodes
            .iter()
            .flatten()
            .enumerate()
            .filter(|(_, mode)| matches!(mode.as_str(), "o" | "b"))
            .map(|(position, _)| position)
            .collect();
        let [position] = outputs.as_slice() else {
            return None;
        };

        argument_types
            .get(*position)
            .map(|pg_type| type_name(pg_type.oid, cache, pg_type))
    }

    /// Returns the argument types, every argument the function declares,
    /// `OUT` included.
    #[must_use]
    pub fn argument_types(&self) -> &[PgType] {
        &self.argument_types
    }

    /// Returns the name each argument type declares under, in the spelling
    /// the reading session resolves it to, `OUT` included.
    #[must_use]
    pub fn argument_type_names(&self) -> &[String] {
        &self.argument_type_names
    }

    /// Returns the return type.
    #[must_use]
    pub fn return_type(&self) -> Option<&PgType> {
        self.return_type.as_ref()
    }

    /// The return type the function declares, the `SETOF` marker included.
    ///
    /// A result described by output arguments declares no return type, so this
    /// answers `None` for those functions.
    #[must_use]
    pub fn return_type_name(&self) -> Option<&str> {
        self.return_type_name.as_deref()
    }

    /// The type one call of the function yields.
    #[must_use]
    pub fn result_type_name(&self) -> Option<&str> {
        self.result_type_name.as_deref()
    }

    /// Returns the role owning the function.
    #[must_use]
    pub fn owner(&self) -> Option<&str> {
        self.owner.as_deref()
    }

    /// Returns whether `PUBLIC` holds `EXECUTE` on the function.
    #[must_use]
    pub fn is_executable_by_public(&self) -> bool {
        self.execute.public
    }

    /// Returns whether the role holding `role_oid` may execute the
    /// function.
    ///
    /// It may when `PUBLIC` holds the privilege, when the role holds an
    /// entry of its own, or when it is the owner and the owner's own entry
    /// was not revoked.
    #[must_use]
    pub fn can_execute(&self, role_oid: Option<u32>) -> bool {
        let Some(role) = role_oid else { return false };
        self.execute.public
            || self
                .execute
                .grants
                .iter()
                .any(|(grantee, _)| *grantee == role)
            || (self.execute.owner && role == self.owner_oid)
    }

    /// Returns whether the role holding `role_oid` may grant `EXECUTE` on
    /// the function to other roles.
    ///
    /// It may when an entry of its own was granted `WITH GRANT OPTION`, or
    /// when it is the owner, whom ownership lets grant whatever its own
    /// entry says.
    #[must_use]
    pub fn can_grant_execute(&self, role_oid: Option<u32>) -> bool {
        let Some(role) = role_oid else { return false };
        role == self.owner_oid
            || self
                .execute
                .grants
                .iter()
                .any(|(grantee, grantable)| *grantable && *grantee == role)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_pg_type(oid: u32, name: &str) -> PgType {
        PgType {
            oid,
            typname: name.to_string(),
            typnamespace: 1,
            typowner: 1,
            typlen: 4,
            typbyval: true,
            typtype: "b".to_string(),
            typcategory: "N".to_string(),
            typispreferred: false,
            typisdefined: true,
            typdelim: ",".to_string(),
            typrelid: 0,
            typelem: 0,
            typarray: 0,
            typinput: 0,
            typoutput: 0,
            typreceive: 0,
            typsend: 0,
            typmodin: 0,
            typmodout: 0,
            typanalyze: 0,
            #[cfg(any(
                feature = "postgres-14",
                feature = "postgres-15",
                feature = "postgres-16",
                feature = "postgres-17",
                feature = "postgres-18"
            ))]
            typsubscript: 0,
            typalign: "i".to_string(),
            typstorage: "p".to_string(),
            typnotnull: false,
            typbasetype: 0,
            typtypmod: -1,
            typndims: 0,
            typcollation: 0,
            typdefaultbin: None,
            typdefault: None,
        }
    }

    fn dummy_pg_proc() -> PgProc {
        PgProc {
            oid: 1,
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

    fn execute_grants(grantees: &[(u32, bool)]) -> Vec<ProcAclEntry> {
        grantees
            .iter()
            .map(|(grantee, grantable)| ProcAclEntry {
                function_oid: 1,
                grantee: Some(*grantee),
                grantable: Some(*grantable),
            })
            .collect()
    }

    #[test]
    fn test_pg_proc_metadata() {
        let arg_type = dummy_pg_type(1, "int4");
        let ret_type = dummy_pg_type(2, "int4");

        let metadata = PgProcMetadata {
            argument_types: vec![arg_type],
            argument_type_names: vec!["int4".to_string()],
            return_type: Some(ret_type),
            return_type_name: Some("int4".to_string()),
            result_type_name: Some("int4".to_string()),
            owner: Some("app_owner".to_string()),
            owner_oid: 1,
            execute: ExecutePrivileges::decode(None, 1),
            bound_functions: Ok(Vec::new()),
        };

        assert_eq!(metadata.argument_types().len(), 1);
        assert_eq!(metadata.argument_types()[0].typname, "int4");
        assert_eq!(metadata.return_type().unwrap().typname, "int4");
        assert_eq!(metadata.return_type_name(), Some("int4"));
        assert_eq!(metadata.result_type_name(), Some("int4"));
        assert_eq!(metadata.owner(), Some("app_owner"));
    }

    // A `NULL` `proacl` is the default the server starts a function with.
    #[test]
    fn test_default_execute_privileges() {
        let metadata = PgProcMetadata {
            argument_types: vec![],
            argument_type_names: vec![],
            return_type: None,
            return_type_name: None,
            result_type_name: None,
            owner: None,
            owner_oid: 7,
            execute: ExecutePrivileges::decode(None, 7),
            bound_functions: Ok(Vec::new()),
        };

        assert!(metadata.is_executable_by_public());
        assert!(metadata.can_execute(Some(7)), "the owner keeps its entry");
        assert!(
            metadata.can_execute(Some(3)),
            "`PUBLIC` holds the privilege"
        );
        assert!(!metadata.can_execute(None));
        assert!(
            metadata.can_grant_execute(Some(7)),
            "ownership lets grant whatever the owner's entry says"
        );
        assert!(!metadata.can_grant_execute(Some(3)));
    }

    // `REVOKE EXECUTE FROM PUBLIC` leaves `proacl` set with the `PUBLIC`
    // entry gone.
    #[test]
    fn test_public_revoked() {
        let metadata = PgProcMetadata {
            argument_types: vec![],
            argument_type_names: vec![],
            return_type: None,
            return_type_name: None,
            result_type_name: None,
            owner: None,
            owner_oid: 7,
            execute: ExecutePrivileges::decode(Some(&execute_grants(&[(7, true)])), 7),
            bound_functions: Ok(Vec::new()),
        };

        assert!(!metadata.is_executable_by_public());
        assert!(metadata.can_execute(Some(7)), "the owner keeps its entry");
        assert!(!metadata.can_execute(Some(3)), "no other grantee holds it");
        assert!(metadata.can_grant_execute(Some(7)));
        assert!(!metadata.can_grant_execute(Some(3)));
    }

    // `GRANT EXECUTE ... WITH GRANT OPTION` reaches one grantee, and the
    // option does not reach another.
    #[test]
    fn test_grant_option_reaches_one_grantee() {
        let metadata = PgProcMetadata {
            argument_types: vec![],
            argument_type_names: vec![],
            return_type: None,
            return_type_name: None,
            result_type_name: None,
            owner: None,
            owner_oid: 7,
            execute: ExecutePrivileges::decode(
                Some(&execute_grants(&[
                    (0, false),
                    (7, true),
                    (3, false),
                    (4, true),
                ])),
                7,
            ),
            bound_functions: Ok(Vec::new()),
        };

        assert!(metadata.is_executable_by_public());
        assert!(metadata.can_execute(Some(3)));
        assert!(metadata.can_execute(Some(4)));
        assert!(metadata.can_grant_execute(Some(4)));
        assert!(!metadata.can_grant_execute(Some(3)));
    }

    // The owner's own entry is an ordinary one, which the owner can revoke.
    #[test]
    fn test_owner_entry_revoked() {
        let metadata = PgProcMetadata {
            argument_types: vec![],
            argument_type_names: vec![],
            return_type: None,
            return_type_name: None,
            result_type_name: None,
            owner: None,
            owner_oid: 7,
            execute: ExecutePrivileges::decode(Some(&execute_grants(&[(3, false)])), 7),
            bound_functions: Ok(Vec::new()),
        };

        assert!(metadata.can_execute(Some(3)));
        assert!(!metadata.can_execute(Some(7)), "the owner's entry is gone");
        assert!(
            metadata.can_grant_execute(Some(7)),
            "ownership still lets the owner grant"
        );
    }

    // Revoking every entry leaves `proacl` set and empty, which `aclexplode`
    // answers with one row naming no grantee, and nobody holds `EXECUTE`.
    #[test]
    fn test_every_entry_revoked() {
        let empty = [ProcAclEntry {
            function_oid: 1,
            grantee: None,
            grantable: None,
        }];
        let privileges = ExecutePrivileges::decode(Some(&empty), 7);
        assert_eq!(
            privileges,
            ExecutePrivileges {
                public: false,
                owner: false,
                grants: Vec::new(),
            }
        );
    }

    // The declared return type mirrors the `RETURNS` clause, so a result
    // described by output arguments declares nothing and a set keeps its marker.
    #[test]
    fn test_return_type_name() {
        let cache = CatalogCache::default();
        let record = dummy_pg_type(2249, "record");
        let int4 = dummy_pg_type(23, "int4");

        let mut output_arguments = dummy_pg_proc();
        output_arguments.prorettype = 2249;
        output_arguments.proretset = true;
        output_arguments.proargmodes = Some(vec!["o".to_string()]);
        assert_eq!(
            PgProcMetadata::declared_return_type(&output_arguments, Some(&record), &cache),
            None,
            "output arguments declare no return type"
        );

        let mut set = dummy_pg_proc();
        set.proretset = true;
        assert_eq!(
            PgProcMetadata::declared_return_type(&set, Some(&int4), &cache),
            Some("SETOF int4".to_string()),
            "a set keeps its marker with the element's stored name"
        );

        assert_eq!(
            PgProcMetadata::declared_return_type(&dummy_pg_proc(), Some(&int4), &cache),
            Some("int4".to_string()),
            "a plain function keeps the stored type name"
        );
    }

    // `prorettype` is `record` for a result described by output arguments,
    // which then yields the type of its single output argument.
    #[test]
    fn test_single_output_argument_type() {
        fn argument_types(names: &[(&str, u32)]) -> Vec<PgType> {
            names
                .iter()
                .map(|(name, oid)| dummy_pg_type(*oid, name))
                .collect()
        }

        let cache = CatalogCache::default();

        let mut two_outputs = dummy_pg_proc();
        two_outputs.proargtypes = vec![2, 3];
        two_outputs.proallargtypes = Some(vec![1, 2, 3]);
        two_outputs.proargmodes = Some(vec!["i".to_string(), "o".to_string(), "b".to_string()]);

        assert_eq!(
            PgProcMetadata::single_output_argument_type(
                &two_outputs,
                &argument_types(&[("text", 1), ("int4", 2), ("uuid", 3)]),
                &cache
            ),
            None,
            "several output arguments leave the result a `record`"
        );

        let mut single_output = dummy_pg_proc();
        single_output.proargtypes = vec![2];
        single_output.proallargtypes = Some(vec![2, 3]);
        single_output.proargmodes = Some(vec!["i".to_string(), "o".to_string()]);

        assert_eq!(
            PgProcMetadata::single_output_argument_type(
                &single_output,
                &argument_types(&[("int4", 2), ("uuid", 3)]),
                &cache
            ),
            Some("uuid".to_string()),
            "the single output argument yields its type"
        );

        let no_output = dummy_pg_proc();
        assert_eq!(
            PgProcMetadata::single_output_argument_type(&no_output, &argument_types(&[]), &cache),
            None,
            "no output argument and `RETURNS record` leaves the result a `record`"
        );
    }
}
