//! Submodule providing [`ServerVersion`], the version of the server a catalog
//! is read from.

use diesel::{PgConnection, RunQueryDsl, dsl::sql, sql_types::Oid};

/// The table privileges every supported `PostgreSQL` hands the owner of a
/// table and folds into `GRANT ALL`, in the order the server lists them.
const TABLE_PRIVILEGES: [&str; 8] = [
    "SELECT",
    "INSERT",
    "UPDATE",
    "DELETE",
    "TRUNCATE",
    "REFERENCES",
    "TRIGGER",
    "MAINTAIN",
];

/// The `PostgreSQL` major version that introduced the `MAINTAIN` privilege.
const MAINTAIN_SINCE: u32 = 17;

/// The version a `PostgreSQL` server reports as `server_version_num`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ServerVersion(u32);

impl ServerVersion {
    /// Wraps a `server_version_num` value such as `170004`.
    #[must_use]
    pub fn from_server_version_num(server_version_num: u32) -> Self {
        Self(server_version_num)
    }

    /// Reads the version of the server behind `connection`.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the query fails.
    pub fn load(connection: &mut PgConnection) -> Result<Self, diesel::result::Error> {
        diesel::select(sql::<Oid>("current_setting('server_version_num')::oid"))
            .get_result(connection)
            .map(Self)
    }

    /// Returns the major version, `17` for `server_version_num` `170004`.
    #[must_use]
    pub fn major(self) -> u32 {
        self.0 / 10_000
    }

    /// Returns the table privileges this server hands the owner of a table
    /// and folds into `GRANT ALL`, in the order it lists them.
    pub(crate) fn table_privileges(self) -> &'static [&'static str] {
        if self.major() >= MAINTAIN_SINCE {
            &TABLE_PRIVILEGES
        } else {
            &TABLE_PRIVILEGES[..TABLE_PRIVILEGES.len() - 1]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ServerVersion;

    #[test]
    fn major_drops_the_minor_release() {
        assert_eq!(ServerVersion::from_server_version_num(140_011).major(), 14);
        assert_eq!(ServerVersion::from_server_version_num(170_004).major(), 17);
        assert_eq!(ServerVersion::from_server_version_num(180_000).major(), 18);
    }

    #[test]
    fn maintain_is_a_table_privilege_from_17_on() {
        let before = ServerVersion::from_server_version_num(160_009).table_privileges();
        let since = ServerVersion::from_server_version_num(170_000).table_privileges();
        assert!(!before.contains(&"MAINTAIN"));
        assert_eq!(since.last(), Some(&"MAINTAIN"));
        assert_eq!(since[..since.len() - 1], *before);
    }
}
