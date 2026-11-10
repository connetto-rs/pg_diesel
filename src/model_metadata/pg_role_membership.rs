//! The role memberships read from `pg_auth_members`.
//!
//! A membership is one per granted role and member, and `pg_auth_members`
//! holds one row per grantor, so the options of the membership hold when
//! any of the grantor rows carries them, which is how PostgreSQL decides
//! `SET ROLE` and privilege inheritance. Before PostgreSQL 16 the `INHERIT`
//! and `SET` options are not stored per membership: the in-effect `INHERIT`
//! is the member's own attribute, and `SET` was always in effect.

use std::collections::BTreeMap;

use diesel::{ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl};

use crate::{
    database::CatalogCache,
    schema::pg_catalog::{pg_auth_members::pg_auth_members, pg_authid::pg_authid},
};

/// The membership rows a read of `pg_auth_members` answers.
#[derive(Debug, diesel::Queryable)]
#[cfg(not(any(feature = "postgres-15", feature = "postgres-14")))]
struct MembershipRow {
    /// The OID of the granted role.
    roleid: u32,
    /// The OID of the member.
    member: u32,
    /// The stored name of the member.
    member_name: String,
    /// Whether the grantor's grant carries `ADMIN`.
    admin_option: bool,
    /// Whether the grantor's grant carries `INHERIT`.
    inherit_option: bool,
    /// Whether the grantor's grant carries `SET`.
    set_option: bool,
}

/// The membership rows a read of `pg_auth_members` answers.
#[derive(Debug, diesel::Queryable)]
#[cfg(any(feature = "postgres-15", feature = "postgres-14"))]
struct MembershipRow {
    /// The OID of the granted role.
    roleid: u32,
    /// The OID of the member.
    member: u32,
    /// The stored name of the member.
    member_name: String,
    /// Whether the member's own attributes carry `INHERIT`.
    member_inherits: bool,
    /// Whether the grantor's grant carries `ADMIN`.
    admin_option: bool,
}

/// The membership of one role in another, the options of every grantor's
/// grant folded to the ones in effect.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PgRoleMembership {
    /// The stored name of the granted role, in exact case.
    role_name: String,
    /// The stored name of the member, in exact case.
    member_name: String,
    /// Whether the member may grant and revoke the membership.
    admin_option: bool,
    /// Whether the member uses the role's privileges without `SET ROLE`.
    inherit_option: bool,
    /// Whether the member may `SET ROLE` to the role.
    set_option: bool,
}

impl PgRoleMembership {
    /// Returns the name of the granted role.
    #[must_use]
    pub fn role_name(&self) -> &str {
        &self.role_name
    }

    /// Returns the name of the member.
    #[must_use]
    pub fn member_name(&self) -> &str {
        &self.member_name
    }

    /// Returns whether the member may grant and revoke the membership.
    #[must_use]
    pub fn admin_option(&self) -> bool {
        self.admin_option
    }

    /// Returns whether the member uses the role's privileges without
    /// `SET ROLE`.
    #[must_use]
    pub fn inherit_option(&self) -> bool {
        self.inherit_option
    }

    /// Returns whether the member may `SET ROLE` to the role.
    #[must_use]
    pub fn set_option(&self) -> bool {
        self.set_option
    }

    /// Loads every membership the database holds, the per-grantor rows
    /// folded to the options in effect.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the query fails.
    pub fn load_all(
        cache: &CatalogCache,
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        #[cfg(not(any(feature = "postgres-15", feature = "postgres-14")))]
        let rows: Vec<MembershipRow> = pg_auth_members::table
            .inner_join(pg_authid::table.on(pg_auth_members::member.eq(pg_authid::oid)))
            .select((
                pg_auth_members::roleid,
                pg_auth_members::member,
                pg_authid::rolname,
                pg_auth_members::admin_option,
                pg_auth_members::inherit_option,
                pg_auth_members::set_option,
            ))
            .load(conn)?;

        #[cfg(not(any(feature = "postgres-15", feature = "postgres-14")))]
        {
            let mut folded: BTreeMap<(u32, u32), (String, bool, bool, bool)> = BTreeMap::new();
            for row in rows {
                let entry = folded
                    .entry((row.roleid, row.member))
                    .or_insert_with(|| (row.member_name.clone(), false, false, false));
                entry.1 |= row.admin_option;
                entry.2 |= row.inherit_option;
                entry.3 |= row.set_option;
            }
            Ok(folded
                .into_iter()
                .filter_map(|((roleid, _), (member_name, admin, inherit, set))| {
                    Some(Self {
                        role_name: cache.role(roleid)?.to_string(),
                        member_name,
                        admin_option: admin,
                        inherit_option: inherit,
                        set_option: set,
                    })
                })
                .collect())
        }

        #[cfg(any(feature = "postgres-15", feature = "postgres-14"))]
        let rows: Vec<MembershipRow> = pg_auth_members::table
            .inner_join(pg_authid::table.on(pg_auth_members::member.eq(pg_authid::oid)))
            .select((
                pg_auth_members::roleid,
                pg_auth_members::member,
                pg_authid::rolname,
                pg_authid::rolinherit,
                pg_auth_members::admin_option,
            ))
            .load(conn)?;

        #[cfg(any(feature = "postgres-15", feature = "postgres-14"))]
        {
            let mut folded: BTreeMap<(u32, u32), (String, bool, bool)> = BTreeMap::new();
            for row in rows {
                let entry = folded
                    .entry((row.roleid, row.member))
                    .or_insert_with(|| (row.member_name.clone(), false, false));
                entry.1 |= row.admin_option;
                entry.2 |= row.member_inherits;
            }
            Ok(folded
                .into_iter()
                .filter_map(|((roleid, _), (member_name, admin, inherit))| {
                    Some(Self {
                        role_name: cache.role(roleid)?.to_string(),
                        member_name,
                        admin_option: admin,
                        inherit_option: inherit,
                        set_option: true,
                    })
                })
                .collect())
        }
    }
}
