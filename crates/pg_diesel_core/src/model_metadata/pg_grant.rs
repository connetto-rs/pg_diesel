//! Submodule providing [`PgTableGrant`] and [`PgColumnGrant`], the privileges
//! the access lists of relations, schemas and columns hold.

use sqlparser::ast::{
    Action, Grantee, GranteeName, GranteesType, Ident, ObjectName, ObjectNamePart,
};

/// Returns the grantee of one access-list entry, the `PUBLIC` pseudo-role for
/// `None` and otherwise the role by its stored name, quoted so that folding
/// cannot rename it.
fn acl_grantee(role: Option<&str>) -> Grantee {
    match role {
        None => Grantee {
            grantee_type: GranteesType::Public,
            name: None,
        },
        Some(role) => Grantee {
            grantee_type: GranteesType::None,
            name: Some(GranteeName::ObjectName(ObjectName(vec![
                ObjectNamePart::Identifier(Ident::with_quote('"', role)),
            ]))),
        },
    }
}

/// Returns the stored name of the role `grantee` names, `None` for `PUBLIC`.
fn grantee_role(grantee: &Grantee) -> Option<&str> {
    match &grantee.name {
        Some(GranteeName::ObjectName(ObjectName(parts))) => match parts.as_slice() {
            [ObjectNamePart::Identifier(ident)] => Some(&ident.value),
            _ => None,
        },
        _ => None,
    }
}

/// One privilege, or every privilege of a `GRANT ALL`, that the access list
/// of a relation or a schema holds.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgTableGrant {
    /// The role that granted it, its OID when the role is gone.
    grantor: String,
    /// The role that received it, or the `PUBLIC` pseudo-role.
    grantee: Grantee,
    /// The schema the grant reaches, or the schema of the relation it reaches.
    schema: String,
    /// The relation the grant reaches, `None` for a schema grant.
    relation: Option<String>,
    /// The privilege, `None` for a `GRANT ALL`.
    privilege: Option<Action>,
    /// Whether it was granted `WITH GRANT OPTION`.
    grantable: bool,
}

impl PgTableGrant {
    /// Creates the grant `grantor` gave `grantee`, `None` for `PUBLIC`, on
    /// the relation `relation` of `schema`, or on `schema` itself when
    /// `relation` is `None`, of `privilege`, or of every privilege when it is
    /// `None`.
    #[must_use]
    pub fn new(
        grantor: String,
        grantee: Option<&str>,
        schema: String,
        relation: Option<String>,
        privilege: Option<Action>,
        grantable: bool,
    ) -> Self {
        Self {
            grantor,
            grantee: acl_grantee(grantee),
            schema,
            relation,
            privilege,
            grantable,
        }
    }

    /// Returns the role that granted the privilege.
    #[must_use]
    pub fn grantor(&self) -> &str {
        &self.grantor
    }

    /// Returns the role that received the privilege, or the `PUBLIC`
    /// pseudo-role.
    #[must_use]
    pub fn grantee(&self) -> &Grantee {
        &self.grantee
    }

    /// Returns the stored name of the role that received the privilege,
    /// `None` for `PUBLIC`.
    #[must_use]
    pub fn grantee_role(&self) -> Option<&str> {
        grantee_role(&self.grantee)
    }

    /// Returns the schema the grant reaches or the schema of its relation.
    #[must_use]
    pub fn schema_name(&self) -> &str {
        &self.schema
    }

    /// Returns the relation the grant reaches, `None` for a schema grant.
    #[must_use]
    pub fn relation_name(&self) -> Option<&str> {
        self.relation.as_deref()
    }

    /// Returns the privilege, `None` for a `GRANT ALL`.
    #[must_use]
    pub fn privilege(&self) -> Option<&Action> {
        self.privilege.as_ref()
    }

    /// Returns whether the privilege was granted `WITH GRANT OPTION`.
    #[must_use]
    pub fn grantable(&self) -> bool {
        self.grantable
    }
}

/// One privilege the access list of a column holds.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgColumnGrant {
    /// The role that granted it, its OID when the role is gone.
    grantor: String,
    /// The role that received it, or the `PUBLIC` pseudo-role.
    grantee: Grantee,
    /// The schema of the column's relation.
    schema: String,
    /// The relation the column belongs to.
    relation: String,
    /// The column the privilege reaches.
    column: String,
    /// The privilege.
    privilege: Action,
    /// Whether it was granted `WITH GRANT OPTION`.
    grantable: bool,
}

impl PgColumnGrant {
    /// Creates the grant `grantor` gave `grantee`, `None` for `PUBLIC`, of
    /// `privilege` on `column` of the relation `relation` of `schema`.
    #[must_use]
    pub fn new(
        grantor: String,
        grantee: Option<&str>,
        schema: String,
        relation: String,
        column: String,
        privilege: Action,
        grantable: bool,
    ) -> Self {
        Self {
            grantor,
            grantee: acl_grantee(grantee),
            schema,
            relation,
            column,
            privilege,
            grantable,
        }
    }

    /// Returns the role that granted the privilege.
    #[must_use]
    pub fn grantor(&self) -> &str {
        &self.grantor
    }

    /// Returns the role that received the privilege, or the `PUBLIC`
    /// pseudo-role.
    #[must_use]
    pub fn grantee(&self) -> &Grantee {
        &self.grantee
    }

    /// Returns the stored name of the role that received the privilege,
    /// `None` for `PUBLIC`.
    #[must_use]
    pub fn grantee_role(&self) -> Option<&str> {
        grantee_role(&self.grantee)
    }

    /// Returns the schema of the column's relation.
    #[must_use]
    pub fn schema_name(&self) -> &str {
        &self.schema
    }

    /// Returns the relation the column belongs to.
    #[must_use]
    pub fn relation_name(&self) -> &str {
        &self.relation
    }

    /// Returns the column the privilege reaches.
    #[must_use]
    pub fn column_name(&self) -> &str {
        &self.column
    }

    /// Returns the privilege.
    #[must_use]
    pub fn privilege(&self) -> &Action {
        &self.privilege
    }

    /// Returns whether the privilege was granted `WITH GRANT OPTION`.
    #[must_use]
    pub fn grantable(&self) -> bool {
        self.grantable
    }
}
