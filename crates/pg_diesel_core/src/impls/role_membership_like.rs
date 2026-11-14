//! Implementation of [`RoleMembershipLike`] for [`PgRoleMembership`].

use sql_traits::traits::{Metadata, RoleMembershipLike};

use crate::{PgDieselDatabase, model_metadata::PgRoleMembership};

impl Metadata for PgRoleMembership {
    type Meta = ();
}

impl RoleMembershipLike for PgRoleMembership {
    type DB = PgDieselDatabase;

    fn role_name(&self) -> &str {
        PgRoleMembership::role_name(self)
    }

    fn role_name_is_quoted(&self) -> bool {
        // Catalog names are stored names: folding again would rename them.
        true
    }

    fn member_name(&self) -> &str {
        PgRoleMembership::member_name(self)
    }

    fn member_name_is_quoted(&self) -> bool {
        true
    }

    fn admin_option(&self) -> bool {
        PgRoleMembership::admin_option(self)
    }

    fn inherit_option(&self) -> bool {
        PgRoleMembership::inherit_option(self)
    }

    fn set_option(&self) -> bool {
        PgRoleMembership::set_option(self)
    }
}
