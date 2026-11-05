//! The extension a `pg_extension` row becomes.

use diesel::{ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl};

use crate::schema::pg_catalog::{pg_extension::pg_extension, pg_namespace::pg_namespace};

/// An extension read from `pg_extension`, beside the schema it landed in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PgExtensionEntry {
    /// The stored name of the extension, in exact case.
    name: String,
    /// The stored name of the schema the extension's objects landed in.
    schema: String,
    /// The version the extension was created with, from `pg_extension`.
    version: String,
}

impl PgExtensionEntry {
    /// Returns the name of the extension.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the schema the extension's objects landed in.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the version the extension was created with.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Loads every extension the database holds.
    ///
    /// An extension is a database-wide object, so no schema filter applies,
    /// and the `plpgsql` row every database carries is among the rows.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the query fails.
    pub fn load_all(conn: &mut PgConnection) -> Result<Vec<Self>, diesel::result::Error> {
        pg_extension::table
            .inner_join(pg_namespace::table.on(pg_extension::extnamespace.eq(pg_namespace::oid)))
            .select((
                pg_extension::extname,
                pg_namespace::nspname,
                pg_extension::extversion,
            ))
            .load::<(String, String, String)>(conn)
            .map(|rows| {
                rows.into_iter()
                    .map(|(name, schema, version)| Self {
                        name,
                        schema,
                        version,
                    })
                    .collect()
            })
    }
}
