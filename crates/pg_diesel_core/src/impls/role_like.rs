//! Implementations of the `RoleLike` trait for `PgRole`.
use crate::models::PgRole;
use crate::{PgDieselDatabase, model_metadata::RoleMetadata};
use sql_traits::traits::{Metadata, RoleLike};

impl Metadata for PgRole {
    type Meta = RoleMetadata;
}

impl RoleLike for PgRole {
    type DB = PgDieselDatabase;

    fn name(&self) -> &str {
        self.rolname.as_deref().unwrap_or("<unknown>")
    }

    fn is_superuser(&self) -> bool {
        self.rolsuper.unwrap_or(false)
    }

    fn can_create_db(&self) -> bool {
        self.rolcreatedb.unwrap_or(false)
    }

    fn can_create_role(&self) -> bool {
        self.rolcreaterole.unwrap_or(false)
    }

    fn inherits(&self) -> bool {
        self.rolinherit.unwrap_or(true)
    }

    fn can_login(&self) -> bool {
        self.rolcanlogin.unwrap_or(false)
    }

    fn can_bypass_rls(&self) -> bool {
        self.rolbypassrls.unwrap_or(false)
    }

    fn is_replication(&self) -> bool {
        self.rolreplication.unwrap_or(false)
    }

    fn connection_limit(&self) -> Option<i32> {
        // `-1` is the catalog's spelling of no limit.
        self.rolconnlimit.filter(|limit| *limit != -1)
    }

    fn member_of<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> impl Iterator<Item = &'db <Self::DB as sql_traits::traits::DatabaseLike>::Role> {
        database
            .role_metadata(self)
            .expect("Role metadata must exist")
            .member_of()
            .map(std::convert::AsRef::as_ref)
    }

    fn policies<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> impl Iterator<Item = &'db <Self::DB as sql_traits::traits::DatabaseLike>::Policy> {
        database
            .role_metadata(self)
            .expect("Role metadata must exist")
            .policies()
            .map(std::convert::AsRef::as_ref)
    }
}
