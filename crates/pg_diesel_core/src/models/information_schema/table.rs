//! Table model and related cached queries.

use std::{collections::HashMap, fmt::Display, sync::Arc};

use diesel::{PgConnection, Queryable, QueryableByName, Selectable};
use sql_traits::structs::ForeignTableSource;
use sqlparser::{
    ast::{Action, Expr, GeneratedExpressionMode},
    parser::ParserError,
};

use crate::{
    database::{CatalogCache, ServerVersion},
    impls::string_to_action,
    model_metadata::{
        ColumnMetadata, IndexDefinitionError, PgCheckConstraint, PgColumn, PgDataType,
        PgForeignKey, PgIndexEntry, PgPolicy, PgTable, ReplicaIdentitySetting, TableMetadata,
        TableMetadataDetails, parse_printed_expression, storage_option_entries,
    },
    models::Column,
};

mod cached_queries;
pub use cached_queries::TableFacts;
pub(crate) use cached_queries::*;

/// The table-level privileges the owner's own `pg_class.relacl` entry holds
/// on a server of `version`, every one of them while `PostgreSQL` left the
/// ACL itself implicit.
fn owner_privileges(
    version: ServerVersion,
    acl_is_null: bool,
    owner: u32,
    entries: &[TableAclEntry],
) -> Vec<Action> {
    version
        .table_privileges()
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
    /// The default or generation expression a column's
    /// `information_schema` row prints does not parse.
    #[error(
        "the expression `{expression}` of column {column} of table {schema}.{table} does not parse: {source}"
    )]
    UnparsableColumnExpression {
        /// The schema of the table.
        schema: String,
        /// The name of the table.
        table: String,
        /// The name of the column.
        column: String,
        /// The expression as `PostgreSQL` printed it.
        expression: String,
        /// Why the parser rejected it, boxed to keep the error small.
        source: Box<ParserError>,
    },
    /// A `pg_index` row of one of the table's indexes does not read as an
    /// index.
    #[error(transparent)]
    IndexDefinition(#[from] IndexDefinitionError),
    /// `pg_class.relreplident` records a replica identity `PostgreSQL` does
    /// not document.
    #[error("table {schema}.{table} records the undocumented replica identity `{letter}`")]
    UnknownReplicaIdentity {
        /// The schema of the table.
        schema: String,
        /// The name of the table.
        table: String,
        /// The `pg_class.relreplident` value.
        letter: String,
    },
    /// `pg_attribute.attgenerated` records a generated column storage
    /// `PostgreSQL` does not document.
    #[error(
        "column {column} of table {schema}.{table} records the undocumented generated column storage `{letter}`"
    )]
    UnknownGenerationStorage {
        /// The schema of the table.
        schema: String,
        /// The name of the table.
        table: String,
        /// The name of the column.
        column: String,
        /// The `pg_attribute.attgenerated` value.
        letter: String,
    },
}

impl Table {
    /// Parses `printed`, an expression the `information_schema` row of
    /// `column` prints.
    ///
    /// # Errors
    ///
    /// Returns [`TableMetadataError::UnparsableColumnExpression`] when
    /// `printed` does not parse.
    fn column_expression(
        &self,
        column: &Column,
        printed: Option<&str>,
    ) -> Result<Option<Expr>, TableMetadataError> {
        printed
            .map(|printed| {
                parse_printed_expression(printed).map_err(|source| {
                    TableMetadataError::UnparsableColumnExpression {
                        schema: self.table_schema.clone(),
                        table: self.table_name.clone(),
                        column: column.column_name.clone(),
                        expression: printed.to_owned(),
                        source: Box::new(source),
                    }
                })
            })
            .transpose()
    }

    /// Reads `letter`, the `pg_class.relreplident` the table records.
    ///
    /// # Errors
    ///
    /// Returns [`TableMetadataError::UnknownReplicaIdentity`] when `letter`
    /// is not one `PostgreSQL` documents.
    fn replica_identity(
        &self,
        letter: String,
    ) -> Result<ReplicaIdentitySetting, TableMetadataError> {
        ReplicaIdentitySetting::from_relreplident(&letter).ok_or_else(|| {
            TableMetadataError::UnknownReplicaIdentity {
                schema: self.table_schema.clone(),
                table: self.table_name.clone(),
                letter,
            }
        })
    }

    /// Reads `letter`, the `pg_attribute.attgenerated` the column named
    /// `column` records: `s` for `STORED`, `v` for `VIRTUAL`, and empty for
    /// a column that is not generated, which answers [`None`].
    ///
    /// # Errors
    ///
    /// Returns [`TableMetadataError::UnknownGenerationStorage`] when `letter`
    /// is not one `PostgreSQL` documents.
    fn generation_storage(
        &self,
        column: &str,
        letter: &str,
    ) -> Result<Option<GeneratedExpressionMode>, TableMetadataError> {
        match letter {
            "" => Ok(None),
            "s" => Ok(Some(GeneratedExpressionMode::Stored)),
            "v" => Ok(Some(GeneratedExpressionMode::Virtual)),
            _ => Err(TableMetadataError::UnknownGenerationStorage {
                schema: self.table_schema.clone(),
                table: self.table_name.clone(),
                column: column.to_owned(),
                letter: letter.to_owned(),
            }),
        }
    }

    /// Initializes and returns the metadata for the table, read from a server
    /// of `version`.
    ///
    /// # Errors
    ///
    /// Returns a [`TableMetadataError`] when a catalog query fails, a
    /// column's type reaches a type the catalog cache does not hold, a
    /// column's default or generation expression or an index's predicate
    /// does not parse, or the table records an undocumented replica
    /// identity or a column an undocumented generated column storage.
    #[allow(clippy::too_many_lines)]
    pub fn metadata(
        &self,
        table: &Arc<PgTable>,
        conn: &mut PgConnection,
        version: ServerVersion,
        denylist_types: &[String],
        cache: &CatalogCache,
    ) -> Result<TableMetadata, TableMetadataError> {
        let facts = cached_queries::table_facts(self, conn)?;
        let attributes = cached_queries::attributes(facts.oid, conn)?;
        let descriptions = cached_queries::descriptions(facts.oid, conn)?;
        let serial_numbers = cached_queries::serial_attributes(facts.oid, conn)?;
        let not_null_names = cached_queries::not_null_constraint_names(facts.oid, conn)?;

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
            let default_expression =
                self.column_expression(&column, column.column_default.as_deref())?;
            let generation_storage =
                self.generation_storage(&column.column_name, &attribute.generation)?;
            let generation_expression = self.column_expression(
                &column,
                column
                    .generation_expression
                    .as_deref()
                    .filter(|_| generation_storage.is_some()),
            )?;
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
                default_expression,
                generation_expression.zip(generation_storage),
            )));
        }
        for check_constraint in PgCheckConstraint::load(table, facts.oid, conn)? {
            sql_metadata.add_check_constraint(Arc::new(check_constraint));
        }
        for foreign_key in cached_queries::foreign_keys(self, conn)? {
            let (schema, name) = foreign_key.referenced_relation(conn)?;
            let columns = cached_queries::referenced_column_names(
                facts.oid,
                &foreign_key.constraint_name,
                conn,
            )?;
            sql_metadata.add_foreign_key(Arc::new(PgForeignKey::new(
                foreign_key,
                schema,
                name,
                columns,
            )));
        }
        let mut constraint_names = index_constraint_names(facts.oid, conn)?
            .into_iter()
            .collect::<HashMap<_, _>>();
        for row in cached_queries::unique_indices(facts.oid, version, conn)? {
            let constraint_name = constraint_names.remove(&row.index.indexrelid);
            sql_metadata.add_unique_index(Arc::new(PgIndexEntry::from_catalog(
                row,
                self.table_schema.clone(),
                &attributes,
                constraint_name,
            )?));
        }
        for row in cached_queries::standalone_indices(facts.oid, version, conn)? {
            sql_metadata.add_index(Arc::new(PgIndexEntry::from_catalog(
                row,
                self.table_schema.clone(),
                &attributes,
                None,
            )?));
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

        let replica_identity = self.replica_identity(facts.replica_identity)?;

        Ok(TableMetadata::new(
            sql_metadata,
            TableMetadataDetails {
                description: descriptions
                    .into_iter()
                    .find(|description| description.objsubid == 0),
                policies,
                parents,
                partition_root,
                column_metadata,
                owner_privileges: owner_privileges(
                    version,
                    facts.acl_is_null,
                    facts.owner,
                    &acl_entries,
                ),
                replica_identity,
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

#[cfg(test)]
mod tests {
    use sqlparser::ast::GeneratedExpressionMode;

    use super::{Table, TableMetadataError};
    use crate::model_metadata::ReplicaIdentitySetting;

    /// Returns the `information_schema.tables` row of `app.events`.
    fn events() -> Table {
        Table {
            table_catalog: "catalog".to_owned(),
            table_schema: "app".to_owned(),
            table_name: "events".to_owned(),
            table_type: "BASE TABLE".to_owned(),
            self_referencing_column_name: None,
            reference_generation: None,
            user_defined_type_catalog: None,
            user_defined_type_schema: None,
            user_defined_type_name: None,
            is_insertable_into: "YES".to_owned(),
            is_typed: "NO".to_owned(),
            commit_action: None,
        }
    }

    #[test]
    fn every_documented_replica_identity_letter_reads() {
        let table = events();
        for (letter, setting) in [
            ("d", ReplicaIdentitySetting::Default),
            ("n", ReplicaIdentitySetting::Nothing),
            ("f", ReplicaIdentitySetting::Full),
            ("i", ReplicaIdentitySetting::Index),
        ] {
            assert_eq!(
                table
                    .replica_identity(letter.to_owned())
                    .expect("PostgreSQL documents the letter"),
                setting
            );
        }
    }

    #[test]
    fn an_undocumented_replica_identity_letter_fails_the_load_naming_the_table() {
        let error = events()
            .replica_identity("x".to_owned())
            .expect_err("PostgreSQL documents no `x` replica identity");
        assert!(matches!(
            &error,
            TableMetadataError::UnknownReplicaIdentity { schema, table, letter }
                if schema == "app" && table == "events" && letter == "x"
        ));
        assert_eq!(
            error.to_string(),
            "table app.events records the undocumented replica identity `x`"
        );
    }

    #[test]
    fn every_documented_generated_column_storage_letter_reads() {
        let table = events();
        for (letter, storage) in [
            ("", None),
            ("s", Some(GeneratedExpressionMode::Stored)),
            ("v", Some(GeneratedExpressionMode::Virtual)),
        ] {
            assert_eq!(
                table
                    .generation_storage("total", letter)
                    .expect("PostgreSQL documents the letter"),
                storage
            );
        }
    }

    #[test]
    fn an_undocumented_generated_column_storage_letter_fails_the_load_naming_the_column() {
        let error = events()
            .generation_storage("total", "x")
            .expect_err("PostgreSQL documents no `x` generated column storage");
        assert!(matches!(
            &error,
            TableMetadataError::UnknownGenerationStorage { schema, table, column, letter }
                if schema == "app" && table == "events" && column == "total" && letter == "x"
        ));
        assert_eq!(
            error.to_string(),
            "column total of table app.events records the undocumented generated column storage `x`"
        );
    }
}
