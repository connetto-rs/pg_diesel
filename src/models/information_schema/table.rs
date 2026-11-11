//! Table model and related cached queries.

use std::{collections::HashMap, fmt::Display, sync::Arc};

use diesel::{PgConnection, Queryable, QueryableByName, Selectable};
use sql_traits::structs::ForeignTableSource;
use sqlparser::ast::Action;

use crate::{
    database::CatalogCache,
    impls::string_to_action,
    model_metadata::{
        ColumnMetadata, PgCheckConstraint, PgColumn, PgDataType, PgForeignKey, PgIndexEntry,
        PgPolicy, PgTable, TableMetadata, TableMetadataDetails, storage_option_entries,
    },
};

mod cached_queries;
pub use cached_queries::TableFacts;
pub(crate) use cached_queries::*;

/// The table privileges `PostgreSQL` hands the owner of a table, in the
/// order the server lists them, which a `REVOKE` naming the owner takes from.
#[cfg(not(any(feature = "postgres-17", feature = "postgres-18")))]
const OWNER_PRIVILEGES: [&str; 7] = [
    "SELECT",
    "INSERT",
    "UPDATE",
    "DELETE",
    "TRUNCATE",
    "REFERENCES",
    "TRIGGER",
];

/// The table privileges `PostgreSQL` hands the owner of a table, in the
/// order the server lists them, which a `REVOKE` naming the owner takes from.
#[cfg(any(feature = "postgres-17", feature = "postgres-18"))]
const OWNER_PRIVILEGES: [&str; 8] = [
    "SELECT",
    "INSERT",
    "UPDATE",
    "DELETE",
    "TRUNCATE",
    "REFERENCES",
    "TRIGGER",
    "MAINTAIN",
];

/// The table-level privileges the owner's own `pg_class.relacl` entry holds,
/// every one of them while `PostgreSQL` left the ACL itself implicit.
fn owner_privileges(acl_is_null: bool, owner: u32, entries: &[TableAclEntry]) -> Vec<Action> {
    OWNER_PRIVILEGES
        .iter()
        .filter(|privilege| {
            acl_is_null
                || entries.iter().any(|entry| {
                    entry.grantee == owner && entry.privilege_type.eq_ignore_ascii_case(privilege)
                })
        })
        .filter_map(|privilege| string_to_action(privilege))
        .collect()
}

/// The foreign data wrapper options `PostgreSQL` stores for a column, each
/// key and value as stored, key first.
///
/// The catalogs store one entry per option as `key=value`, and a key never
/// holds an `=`, so the first one separates the two halves.
fn foreign_options_of(options: Option<&[String]>) -> Vec<(String, String)> {
    let Some(options) = options else {
        return Vec::new();
    };
    options
        .iter()
        .filter_map(|option| option.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

#[derive(
    Queryable, QueryableByName, PartialEq, Eq, PartialOrd, Ord, Selectable, Debug, Clone, Hash,
)]
#[diesel(table_name = crate::schema::information_schema::tables::tables)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Struct defining the `information_schema.tables` table.
pub struct Table {
    /// The catalog name of the table.
    pub table_catalog: String,
    /// The schema name of the table.
    pub table_schema: String,
    /// The name of the table.
    pub table_name: String,
    /// The type of the table, e.g. `BASE TABLE` or `VIEW`.
    pub table_type: String,
    /// The name of the table that the current table is a temporary table for.
    pub self_referencing_column_name: Option<String>,
    /// The name of the column that is a foreign key to the current table.
    pub reference_generation: Option<String>,
    /// The user-defined type catalog.
    pub user_defined_type_catalog: Option<String>,
    /// The user-defined type schema.
    pub user_defined_type_schema: Option<String>,
    /// The user-defined type name.
    pub user_defined_type_name: Option<String>,
    /// The user-defined type name that the current table is a temporary table
    /// for.
    pub is_insertable_into: String,
    /// Whether the table is typed.
    pub is_typed: String,
    /// Whether the table is updatable.
    pub commit_action: Option<String>,
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}.{}`", self.table_schema, self.table_name)
    }
}

/// Why the metadata of a table could not be read.
#[derive(Debug, thiserror::Error)]
pub enum TableMetadataError {
    /// A catalog query failed.
    #[error(transparent)]
    Query(#[from] diesel::result::Error),
    /// The type of a column reaches an array element or a domain base the
    /// catalog cache does not hold.
    #[error(
        "the type of column {column} of table {schema}.{table} reaches a type the catalog does not hold"
    )]
    UnresolvedColumnType {
        /// The schema of the table.
        schema: String,
        /// The name of the table.
        table: String,
        /// The name of the column.
        column: String,
    },
}

impl Table {
    /// Initializes and returns the metadata for the table.
    ///
    /// # Errors
    ///
    /// Returns a [`TableMetadataError`] when a catalog query fails or a
    /// column's type reaches a type the catalog cache does not hold.
    #[allow(clippy::too_many_lines)]
    pub fn metadata(
        &self,
        table: &Arc<PgTable>,
        conn: &mut PgConnection,
        denylist_types: &[String],
        cache: &CatalogCache,
    ) -> Result<TableMetadata, TableMetadataError> {
        let facts = cached_queries::table_facts(self, conn)?;
        let attributes = cached_queries::attributes(facts.oid, conn)?;
        let descriptions = cached_queries::descriptions(facts.oid, conn)?;
        let serial_numbers = cached_queries::serial_attributes(facts.oid, conn)?;
        #[cfg(feature = "postgres-18")]
        let not_null_names = cached_queries::not_null_constraint_names(facts.oid, conn)?;
        #[cfg(not(feature = "postgres-18"))]
        let not_null_names: Vec<(i32, String)> = Vec::new();

        let mut sql_metadata = sql_traits::structs::TableMetadata::default();
        let mut column_metadata = HashMap::new();
        for column in cached_queries::columns(self, conn)? {
            let Some(attribute) = attributes
                .iter()
                .find(|attribute| attribute.name == column.column_name)
            else {
                continue;
            };
            let Some(pg_type) = cache.pg_type(attribute.type_oid) else {
                continue;
            };
            if denylist_types.contains(&pg_type.typname) {
                continue;
            }
            let description = descriptions
                .iter()
                .find(|description| description.objsubid == i32::from(attribute.number))
                .cloned();
            let collation = column
                .collation_schema
                .as_deref()
                .zip(column.collation_name.as_deref())
                .and_then(|(schema, name)| cache.collation_is_deterministic(schema, name));
            let serial = serial_numbers.contains(&i32::from(attribute.number))
                && PgDataType::is_serial_integer(&pg_type.typname);
            let data_type =
                PgDataType::from_type(pg_type, attribute.typmod, cache, attribute.ndims)
                    .ok_or_else(|| TableMetadataError::UnresolvedColumnType {
                        schema: self.table_schema.clone(),
                        table: self.table_name.clone(),
                        column: column.column_name.clone(),
                    })?;
            let foreign_options = foreign_options_of(attribute.fdw_options.as_deref());
            let not_null = not_null_names
                .iter()
                .find(|(number, _)| *number == i32::from(attribute.number))
                .map(|(_, name)| name.clone());
            column_metadata.insert(
                column.column_name.clone(),
                ColumnMetadata::new(Arc::clone(table), description, collation),
            );
            sql_metadata.add_column(Arc::new(PgColumn::new(
                column,
                data_type,
                foreign_options,
                not_null,
                serial,
            )));
        }
        for check_constraint in PgCheckConstraint::load(table, facts.oid, conn)? {
            sql_metadata.add_check_constraint(Arc::new(check_constraint));
        }
        for foreign_key in cached_queries::foreign_keys(self, conn)? {
            let (schema, name) = foreign_key.referenced_relation(conn)?;
            sql_metadata.add_foreign_key(Arc::new(PgForeignKey::new(foreign_key, schema, name)));
        }
        let mut constraint_names = index_constraint_names(facts.oid, conn)?
            .into_iter()
            .collect::<HashMap<_, _>>();
        for (index, index_name) in cached_queries::unique_indices(facts.oid, conn)? {
            let constraint_name = constraint_names.remove(&index.indexrelid);
            sql_metadata.add_unique_index(Arc::new(PgIndexEntry::new(
                index,
                self.table_schema.clone(),
                index_name,
                constraint_name,
            )));
        }
        for (index, index_name) in cached_queries::standalone_indices(facts.oid, conn)? {
            sql_metadata.add_index(Arc::new(PgIndexEntry::new(
                index,
                self.table_schema.clone(),
                index_name,
                None,
            )));
        }
        let mut primary_key_columns = Vec::new();
        for pk_column in cached_queries::primary_key_columns(self, conn)? {
            primary_key_columns.extend(
                sql_metadata
                    .column_arcs()
                    .filter(|col: &&Arc<PgColumn>| {
                        col.as_ref().column_name() == pk_column.column_name
                    })
                    .cloned(),
            );
        }
        sql_metadata.set_primary_key(primary_key_columns);

        let triggers = cached_queries::triggers(facts.oid, conn)?
            .into_iter()
            .map(|(trigger, function_name)| {
                let update_columns = trigger
                    .tgattr
                    .iter()
                    .filter(|number| **number != 0)
                    .filter_map(|number| {
                        attributes
                            .iter()
                            .find(|attribute| attribute.number == *number)
                            .map(|attribute| attribute.name.clone())
                    })
                    .collect();
                (Arc::new(trigger), function_name, update_columns)
            })
            .collect();

        let policies = cached_queries::policies(facts.oid, conn)?
            .into_iter()
            .map(|policy| Arc::new(PgPolicy::new(policy, Arc::clone(table))))
            .collect();

        sql_metadata.set_rls_enabled(facts.row_security);
        sql_metadata.set_rls_forced(facts.forced_row_security);
        sql_metadata.set_owner(cache.role(facts.owner).map(ToOwned::to_owned));
        if let Some(server_name) = &facts.server_name {
            // Catalog option names are stored unquoted and folded.
            let server_options = storage_option_entries(&facts.server_options);
            sql_metadata.set_foreign_source(Some(ForeignTableSource::new(
                server_name.as_str(),
                false,
                server_options
                    .iter()
                    .map(|(key, value)| (key.as_str(), false, value.as_str())),
            )));
        }
        sql_metadata.set_inherited_column_names(
            attributes
                .iter()
                .filter(|attribute| attribute.inherited)
                .map(|attribute| attribute.name.clone())
                .collect(),
        );

        let ancestors = cached_queries::parents(facts.oid, conn)?;
        let (parents, partition_root) = if facts.is_partition {
            (Vec::new(), ancestors.into_iter().next())
        } else {
            (ancestors, None)
        };

        // An implicit ACL holds no entry to read, so the rows are fetched
        // only when the server wrote one.
        let acl_entries = if facts.acl_is_null {
            Vec::new()
        } else {
            cached_queries::table_acl_entries(facts.oid, conn)?
        };

        Ok(TableMetadata::new(
            sql_metadata,
            TableMetadataDetails {
                description: descriptions
                    .into_iter()
                    .find(|description| description.objsubid == 0),
                triggers,
                policies,
                parents,
                partition_root,
                column_metadata,
                owner_privileges: owner_privileges(facts.acl_is_null, facts.owner, &acl_entries),
            },
        ))
    }

    /// Returns the partitioning strategy `pg_partitioned_table` records for
    /// the table, or [`None`] when the table is not partitioned.
    ///
    /// # Errors
    ///
    /// * If the strategy cannot be loaded from the database.
    pub fn partition_strategy(
        &self,
        conn: &mut PgConnection,
    ) -> Result<Option<String>, diesel::result::Error> {
        cached_queries::partition_strategy(cached_queries::table_facts(self, conn)?.oid, conn)
    }

    /// Returns what `pg_catalog` records about the table beyond its columns.
    ///
    /// # Errors
    ///
    /// * If the facts cannot be loaded from the database.
    pub fn table_facts(
        &self,
        conn: &mut PgConnection,
    ) -> Result<TableFacts, diesel::result::Error> {
        cached_queries::table_facts(self, conn)
    }

    /// Returns all tables in the database.
    ///
    /// # Errors
    ///
    /// * If the tables cannot be loaded from the database.
    pub fn load_all(
        conn: &mut PgConnection,
        table_catalog: &str,
        table_schema: &str,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        Self::load_filtered(conn, table_catalog, table_schema, None)
    }

    /// Loads tables within a schema, optionally restricted to exact names.
    pub(crate) fn load_filtered(
        conn: &mut PgConnection,
        table_catalog: &str,
        table_schema: &str,
        table_names: Option<&[String]>,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        load_all_tables(table_catalog, table_schema, table_names, conn)
    }
}
