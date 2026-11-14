//! Model representing applicable roles in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

#[derive(Queryable, QueryableByName, Selectable, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::information_schema::applicable_roles::applicable_roles)]
#[diesel(primary_key(grantee, role_name))]
/// Represents a row in `PostgreSQL`'s `applicable_roles` view.
pub struct ApplicableRoles {
    /// Name of the role to which the role was granted (the grantee).
    /// This represents the user or role that has been granted the privileges
    /// of the role specified in `role_name`.
    pub grantee: Option<String>,

    /// Name of the role that is applicable to the grantee.
    /// This is the role whose privileges and permissions are available
    /// to the grantee through direct or inherited role membership.
    pub role_name: Option<String>,

    /// Indicates whether the grantee has the option to grant this role to
    /// others. "YES" if the grantee can grant the role to other users/roles
    /// (WITH GRANT OPTION), "NO" if the grantee cannot grant the role to
    /// others.
    pub is_grantable: Option<String>,
}
