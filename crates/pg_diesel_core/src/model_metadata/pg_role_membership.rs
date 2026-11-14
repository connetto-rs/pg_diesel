//! The role memberships read from `pg_auth_members`.
//!
//! A membership is one per granted role and member, and `pg_auth_members`
//! holds one row per grantor, so the options of the membership hold when
//! any of the grantor rows carries them, which is how PostgreSQL decides
//! `SET ROLE` and privilege inheritance. Before PostgreSQL 16 the `INHERIT`
//! and `SET` options are not stored per membership. The in-effect `INHERIT`
//! is the member's own attribute, and `SET` was always in effect.

use std::collections::BTreeMap;

use diesel::{ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl};

use crate::{
    database::CatalogCache, schema::pg_catalog::pg_authid::pg_authid,
    shared::schema::pg_catalog::pg_auth_members,
};

/// One grantor's grant of a role to a member, as `pg_auth_members` records
/// it, with the options that grant carries.
#[derive(Debug, Clone, PartialEq, Eq, diesel::Queryable)]
pub struct MembershipGrant {
    /// The OID of the granted role.
    pub roleid: u32,
    /// The OID of the member.
    pub member: u32,
    /// The stored name of the member.
    pub member_name: String,
    /// Whether the grant carries `ADMIN`.
    pub admin_option: bool,
    /// Whether the grant carries `INHERIT`.
    pub inherit_option: bool,
    /// Whether the grant carries `SET`.
    pub set_option: bool,
}

impl MembershipGrant {
    /// Loads every grant of a server older than `PostgreSQL` 16, which
    /// stores no `INHERIT` or `SET` option per grant. Each grant takes the
    /// member's own `INHERIT` attribute and carries `SET`.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the query fails.
    pub fn load_before_16(conn: &mut PgConnection) -> Result<Vec<Self>, diesel::result::Error> {
        Ok(pg_auth_members::table
            .inner_join(pg_authid::table.on(pg_auth_members::member.eq(pg_authid::oid)))
            .select((
                pg_auth_members::roleid,
                pg_auth_members::member,
                pg_authid::rolname,
                pg_auth_members::admin_option,
                pg_authid::rolinherit,
            ))
            .load::<(u32, u32, String, bool, bool)>(conn)?
            .into_iter()
            .map(
                |(roleid, member, member_name, admin_option, member_inherits)| Self {
                    roleid,
                    member,
                    member_name,
                    admin_option,
                    inherit_option: member_inherits,
                    set_option: true,
                },
            )
            .collect())
    }
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

    /// Folds the per-grantor `grants` to one membership per granted role
    /// and member, each option in effect when any grant carries it.
    pub(crate) fn fold(grants: Vec<MembershipGrant>, cache: &CatalogCache) -> Vec<Self> {
        let mut folded: BTreeMap<(u32, u32), (String, bool, bool, bool)> = BTreeMap::new();
        for grant in grants {
            let entry = folded
                .entry((grant.roleid, grant.member))
                .or_insert_with(|| (grant.member_name, false, false, false));
            entry.1 |= grant.admin_option;
            entry.2 |= grant.inherit_option;
            entry.3 |= grant.set_option;
        }
        folded
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
            .collect()
    }
}
