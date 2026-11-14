//! Submodule providing [`PgSchema`], the schema a `PgDieselDatabase` holds.

use diesel::{PgConnection, QueryableByName, RunQueryDsl};

use crate::models::Schemata;

/// The `USAGE` a schema's `pg_namespace.nspacl` records for `PUBLIC` and for
/// the owner, beside the schema's name.
#[derive(QueryableByName)]
struct SchemaUsageRow {
    /// The stored name of the schema.
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    /// Whether `PUBLIC` holds the `USAGE` the schema was created with.
    #[diesel(sql_type = diesel::sql_types::Bool)]
    public_usage: bool,
    /// Whether the owner's own entry holds `USAGE`.
    #[diesel(sql_type = diesel::sql_types::Bool)]
    owner_usage: bool,
}

/// An `information_schema.schemata` row with the `USAGE` facts
/// `pg_namespace.nspacl` records for it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgSchema {
    /// The `information_schema.schemata` row describing the schema.
    model: Schemata,
    /// Whether `PUBLIC` holds the `USAGE` the schema was created with.
    public_usage: bool,
    /// Whether the owner's own entry holds `USAGE`.
    owner_usage: bool,
}

impl PgSchema {
    /// Loads the named schemas of the given catalog with their `USAGE`
    /// facts.
    ///
    /// Only the built-in `public` schema, OID 2200, gives `PUBLIC` `USAGE`
    /// from creation. A `NULL` `nspacl` is the state a schema is created in,
    /// where the owner holds `USAGE`, and a set `nspacl` holds exactly the
    /// entries `aclexplode` lists, so an owner whose entry a `REVOKE` removed
    /// holds none.
    ///
    /// # Errors
    ///
    /// Returns an error if a database query fails.
    pub fn load_all(
        catalog: &str,
        schemas: &[String],
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        // Raw `sql_query`, since the DSL cannot place `aclexplode` in a `FROM`.
        let usage: Vec<SchemaUsageRow> = diesel::sql_query(
            "SELECT n.nspname AS name, \
             (n.oid = 2200 AND (n.nspacl IS NULL OR EXISTS ( \
                 SELECT 1 FROM aclexplode(n.nspacl) AS entry \
                 WHERE entry.grantee = 0 AND entry.privilege_type = 'USAGE' \
             ))) AS public_usage, \
             (n.nspacl IS NULL OR EXISTS ( \
                 SELECT 1 FROM aclexplode(n.nspacl) AS entry \
                 WHERE entry.grantee = n.nspowner AND entry.privilege_type = 'USAGE' \
             )) AS owner_usage \
             FROM pg_catalog.pg_namespace AS n \
             WHERE n.nspname = ANY($1)",
        )
        .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(schemas)
        .load(conn)?;
        Ok(Schemata::load_all(catalog, schemas, conn)?
            .into_iter()
            .filter_map(|model| {
                let row = usage.iter().find(|row| row.name == model.schema_name)?;
                Some(Self {
                    public_usage: row.public_usage,
                    owner_usage: row.owner_usage,
                    model,
                })
            })
            .collect())
    }

    /// Returns the `information_schema.schemata` row describing the schema.
    #[must_use]
    pub fn model(&self) -> &Schemata {
        &self.model
    }

    /// Returns whether `PUBLIC` holds the `USAGE` the schema was created with.
    #[must_use]
    pub fn public_usage(&self) -> bool {
        self.public_usage
    }

    /// Returns whether the owner's own entry holds `USAGE`.
    #[must_use]
    pub fn owner_usage(&self) -> bool {
        self.owner_usage
    }
}
