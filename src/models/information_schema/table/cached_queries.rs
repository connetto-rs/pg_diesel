//! Submodule defining the cached queries methods used in the [`Table`] struct.

use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    OptionalExtension, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
};

use crate::models::{Column, KeyColumnUsage, PgDescription, PgIndex, PgTrigger, Table};

/// Loads base and foreign tables, optionally restricted to exact table names.
pub(crate) fn load_all_tables(
    table_catalog: &str,
    table_schema: &str,
    table_names: Option<&[String]>,
    conn: &mut PgConnection,
) -> Result<Vec<Table>, diesel::result::Error> {
    use crate::schema::information_schema::tables::tables;
    let query = tables::table
        .filter(tables::table_catalog.eq(table_catalog))
        .filter(tables::table_schema.eq(table_schema))
        .filter(tables::table_type.eq_any(["BASE TABLE", "FOREIGN"]))
        .filter(tables::table_name.ne("__diesel_schema_migrations"))
        .order_by(tables::table_name)
        .select(Table::as_select());
    match table_names {
        Some(names) => query.filter(tables::table_name.eq_any(names)).load(conn),
        None => query.load(conn),
    }
}

/// Returns the columns of the table.
pub(crate) fn columns(
    table: &Table,
    conn: &mut PgConnection,
) -> Result<Vec<Column>, diesel::result::Error> {
    use crate::schema::information_schema::columns::columns;
    columns::table
        .filter(columns::table_name.eq(&table.table_name))
        .filter(columns::table_schema.eq(&table.table_schema))
        .filter(columns::table_catalog.eq(&table.table_catalog))
        .order_by(columns::ordinal_position)
        .load::<Column>(conn)
}

/// Returns the columns that are part of the primary key for the given table.
///
/// # Errors
///
/// * If the columns cannot be loaded from the database.
pub fn primary_key_columns(
    table: &Table,
    conn: &mut PgConnection,
) -> Result<Vec<Column>, diesel::result::Error> {
    use crate::schema::information_schema::{
        columns::columns, key_column_usage::key_column_usage, table_constraints::table_constraints,
    };
    key_column_usage::table
        .inner_join(
            columns::table.on(key_column_usage::table_name
                .nullable()
                .eq(columns::table_name.nullable())
                .and(
                    key_column_usage::table_schema
                        .nullable()
                        .eq(columns::table_schema.nullable()),
                )
                .and(
                    key_column_usage::table_catalog
                        .nullable()
                        .eq(columns::table_catalog.nullable()),
                )
                .and(
                    key_column_usage::column_name
                        .nullable()
                        .eq(columns::column_name.nullable()),
                )),
        )
        .inner_join(
            table_constraints::table.on(key_column_usage::constraint_name
                .nullable()
                .eq(table_constraints::constraint_name.nullable())
                .and(
                    key_column_usage::constraint_schema
                        .nullable()
                        .eq(table_constraints::constraint_schema.nullable()),
                )
                .and(
                    key_column_usage::constraint_catalog
                        .nullable()
                        .eq(table_constraints::constraint_catalog.nullable()),
                )
                .and(
                    key_column_usage::table_name
                        .nullable()
                        .eq(table_constraints::table_name.nullable()),
                )
                .and(
                    key_column_usage::table_schema
                        .nullable()
                        .eq(table_constraints::table_schema.nullable()),
                )
                .and(
                    key_column_usage::table_catalog
                        .nullable()
                        .eq(table_constraints::table_catalog.nullable()),
                )),
        )
        .filter(key_column_usage::table_name.eq(&table.table_name))
        .filter(key_column_usage::table_schema.eq(&table.table_schema))
        .filter(key_column_usage::table_catalog.eq(&table.table_catalog))
        .filter(table_constraints::constraint_type.eq("PRIMARY KEY"))
        .order_by(key_column_usage::ordinal_position)
        .select(Column::as_select())
        .load::<Column>(conn)
}

/// Returns the foreign keys of the table.
pub(crate) fn foreign_keys(
    table: &Table,
    conn: &mut PgConnection,
) -> Result<Vec<KeyColumnUsage>, diesel::result::Error> {
    use crate::schema::information_schema::{
        key_column_usage::key_column_usage, referential_constraints::referential_constraints,
    };
    key_column_usage::table
        .inner_join(
            referential_constraints::table.on(key_column_usage::constraint_name
                .eq(referential_constraints::constraint_name)
                .and(
                    key_column_usage::constraint_schema
                        .eq(referential_constraints::constraint_schema),
                )
                .and(
                    key_column_usage::constraint_catalog
                        .eq(referential_constraints::constraint_catalog),
                )),
        )
        .filter(key_column_usage::table_name.eq(&table.table_name))
        .filter(key_column_usage::table_schema.eq(&table.table_schema))
        .filter(key_column_usage::table_catalog.eq(&table.table_catalog))
        .filter(key_column_usage::ordinal_position.eq(1))
        .order_by((
            key_column_usage::constraint_catalog,
            key_column_usage::constraint_schema,
            key_column_usage::constraint_name,
            key_column_usage::ordinal_position,
        ))
        .select(KeyColumnUsage::as_select())
        .load::<KeyColumnUsage>(conn)
}

/// Returns the indices of the table with OID `table_oid` a `PRIMARY KEY` or
/// `UNIQUE` constraint created, each with the name `pg_class` records the
/// index relation under.
pub(crate) fn unique_indices(
    table_oid: u32,
    conn: &mut PgConnection,
) -> Result<Vec<(PgIndex, String)>, diesel::result::Error> {
    use crate::schema::pg_catalog::{
        pg_class::pg_class, pg_constraint::pg_constraint, pg_index::pg_index,
    };

    pg_index::table
        .inner_join(pg_class::table.on(pg_class::oid.eq(pg_index::indexrelid)))
        .filter(pg_index::indrelid.eq(table_oid))
        .filter(diesel::dsl::exists(
            pg_constraint::table
                .select(pg_constraint::oid)
                .filter(pg_constraint::conindid.eq(pg_index::indexrelid)),
        ))
        .select((PgIndex::as_select(), pg_class::relname))
        .load::<(PgIndex, String)>(conn)
}

/// Returns, for each index of the table, the name of the primary key, unique,
/// or exclusion constraint the index enforces, and the OID the engine gives
/// the index.
///
/// A foreign key constraint points at the index the referenced columns hold
/// and takes its name from the key, so the names those constraints record
/// do not describe what the index enforces, and the query skips them.
pub(crate) fn index_constraint_names(
    table_oid: u32,
    conn: &mut PgConnection,
) -> Result<Vec<(u32, String)>, diesel::result::Error> {
    use crate::schema::pg_catalog::{pg_constraint::pg_constraint, pg_index::pg_index};

    let backed = pg_constraint::contype
        .eq("p")
        .or(pg_constraint::contype.eq("u"))
        .or(pg_constraint::contype.eq("x"));

    pg_constraint::table
        .inner_join(pg_index::table.on(pg_constraint::conindid.eq(pg_index::indexrelid)))
        .filter(pg_index::indrelid.eq(table_oid))
        .filter(backed)
        .select((pg_constraint::conindid, pg_constraint::conname))
        .load::<(u32, String)>(conn)
}

/// Returns all indices of the table with OID `table_oid`, each with the name
/// `pg_class` records the index relation under.
pub(crate) fn indices(
    table_oid: u32,
    conn: &mut PgConnection,
) -> Result<Vec<(PgIndex, String)>, diesel::result::Error> {
    use crate::schema::pg_catalog::{pg_class::pg_class, pg_index::pg_index};

    pg_index::table
        .inner_join(pg_class::table.on(pg_class::oid.eq(pg_index::indexrelid)))
        .filter(pg_index::indrelid.eq(table_oid))
        .select((PgIndex::as_select(), pg_class::relname))
        .load::<(PgIndex, String)>(conn)
}

/// Returns the indices of the table with OID `table_oid` no constraint
/// created, unique or not, each with the name `pg_class` records the index
/// relation under.
pub(crate) fn standalone_indices(
    table_oid: u32,
    conn: &mut PgConnection,
) -> Result<Vec<(PgIndex, String)>, diesel::result::Error> {
    let all = indices(table_oid, conn)?;
    let backed: Vec<u32> = unique_indices(table_oid, conn)?
        .into_iter()
        .map(|(pg_index, _)| pg_index.indexrelid)
        .collect();

    Ok(all
        .into_iter()
        .filter(|(pg_index, _)| !backed.contains(&pg_index.indexrelid))
        .collect())
}

/// Returns the triggers `pg_trigger` records for the table, one row per
/// trigger, with the name the engine records the function they execute under.
pub(crate) fn triggers(
    table_oid: u32,
    conn: &mut PgConnection,
) -> Result<Vec<(PgTrigger, String)>, diesel::result::Error> {
    use crate::schema::pg_catalog::{pg_proc::pg_proc, pg_trigger::pg_trigger};

    pg_trigger::table
        .inner_join(pg_proc::table.on(pg_trigger::tgfoid.eq(pg_proc::oid)))
        .filter(pg_trigger::tgrelid.eq(table_oid))
        .filter(pg_trigger::tgisinternal.eq(false))
        .order_by(pg_trigger::oid)
        .select((PgTrigger::as_select(), pg_proc::proname))
        .load::<(PgTrigger, String)>(conn)
}

/// Returns the name each not-null constraint of the table holds, by the
/// number the engine gives the column it applies to.
///
/// The engine records such constraints as `pg_constraint` rows only in
/// `PostgreSQL` 18, so the query exists nowhere else.
#[cfg(feature = "postgres-18")]
pub(crate) fn not_null_constraint_names(
    table_oid: u32,
    conn: &mut PgConnection,
) -> Result<Vec<(i32, String)>, diesel::result::Error> {
    use crate::schema::pg_catalog::pg_constraint::pg_constraint;

    let rows = pg_constraint::table
        .filter(pg_constraint::conrelid.eq(table_oid))
        .filter(pg_constraint::contype.eq("n"))
        .select((pg_constraint::conkey, pg_constraint::conname))
        .load::<(Option<Vec<i16>>, String)>(conn)?;

    Ok(rows
        .into_iter()
        .filter_map(|(key, name)| {
            key.as_deref()?
                .first()
                .map(|attnum| (i32::from(*attnum), name))
        })
        .collect())
}

/// Returns the policies associated with the table, ordered by name, the
/// order `PostgreSQL` applies them in.
pub(crate) fn policies(
    table_oid: u32,
    conn: &mut PgConnection,
) -> Result<Vec<crate::models::PgPolicyTable>, diesel::result::Error> {
    use crate::schema::pg_catalog::pg_policy::pg_policy;

    pg_policy::table
        .filter(pg_policy::polrelid.eq(table_oid))
        .order(pg_policy::polname)
        .select(crate::models::PgPolicyTable::as_select())
        .load(conn)
}

/// What a table's `pg_attribute` row says about one of its columns.
#[derive(Debug, Clone)]
pub(crate) struct AttributeFacts {
    /// The name of the column.
    pub(crate) name: String,
    /// The position the column holds, which a description refers to.
    pub(crate) number: i16,
    /// The OID of the column's type.
    pub(crate) type_oid: u32,
    /// The type modifier the column carries, `-1` when the type stores none.
    pub(crate) typmod: i32,
    /// The array dimensions the column declares, `0` when it is not an array.
    pub(crate) ndims: i32,
    /// The foreign data wrapper options the column declares.
    pub(crate) fdw_options: Option<Vec<String>>,
    /// Whether the column comes from a parent rather than this table.
    pub(crate) inherited: bool,
}

/// The raw row `table_facts` reads, named like its `SELECT` aliases.
#[derive(diesel::QueryableByName, Debug)]
struct TableFactsRow {
    /// The OID of the table.
    #[diesel(sql_type = diesel::sql_types::Oid)]
    oid: u32,
    /// Whether Row Level Security is enabled on the table.
    #[diesel(sql_type = diesel::sql_types::Bool)]
    row_security: bool,
    /// Whether Row Level Security applies to the table's owner too.
    #[diesel(sql_type = diesel::sql_types::Bool)]
    forced_row_security: bool,
    /// The OID of the role owning the table.
    #[diesel(sql_type = diesel::sql_types::Oid)]
    owner: u32,
    /// Whether the table is a partition of another one.
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_partition: bool,
    /// Whether the table is `UNLOGGED`.
    #[diesel(sql_type = diesel::sql_types::Bool)]
    unlogged: bool,
    /// The storage parameters `pg_class.reloptions` records, each a
    /// `name=value` pair.
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Text>>)]
    reloptions: Option<Vec<String>>,
    /// The storage parameters the table's TOAST relation records, each a
    /// `name=value` pair.
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Text>>)]
    toast_reloptions: Option<Vec<String>>,
    /// The partition bound, as `pg_get_expr` writes it.
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    partition_bound: Option<String>,
    /// The name of the server a foreign table reads through.
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    server_name: Option<String>,
    /// The table-level options the foreign server records, each a
    /// `name=value` pair.
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Text>>)]
    server_options: Option<Vec<String>>,
    /// Whether `pg_class.relacl` holds no ACL at all.
    #[diesel(sql_type = diesel::sql_types::Bool)]
    acl_is_null: bool,
}

/// What `pg_catalog.pg_class` records about a table beyond its columns.
#[derive(Debug, Clone)]
pub struct TableFacts {
    /// The OID `pg_class` records the table under.
    pub(crate) oid: u32,
    /// Whether Row Level Security is enabled on the table.
    pub(crate) row_security: bool,
    /// Whether Row Level Security applies to the table's owner too.
    pub(crate) forced_row_security: bool,
    /// The OID of the role owning the table.
    pub(crate) owner: u32,
    /// Whether the table is a partition of another one, as opposed to an
    /// `INHERITS` child.
    pub(crate) is_partition: bool,
    /// Whether the table is `UNLOGGED`.
    pub(crate) unlogged: bool,
    /// The storage parameters `pg_class.reloptions` records, each as
    /// `PostgreSQL` stores the `name=value` pair.
    pub(crate) reloptions: Vec<String>,
    /// The storage parameters the table's TOAST relation records.
    pub(crate) toast_reloptions: Vec<String>,
    /// The bound the table takes as a partition, as `pg_get_expr` writes it.
    pub(crate) partition_bound: Option<String>,
    /// The server the table reads through, when it is a foreign table.
    pub(crate) server_name: Option<String>,
    /// The table-level options the foreign server recorded.
    pub(crate) server_options: Vec<String>,
    /// Whether `pg_class.relacl` holds no ACL at all, so the owner's entry
    /// stands implicit rather than recorded.
    pub(crate) acl_is_null: bool,
}

/// Returns what `pg_catalog` records about the table beyond its columns:
/// its identity, row security settings, owner, partition status and bound,
/// its storage parameters, the server a foreign table reads through, and
/// whether an explicit ACL was written.
pub(crate) fn table_facts(
    table: &Table,
    conn: &mut PgConnection,
) -> Result<TableFacts, diesel::result::Error> {
    // Raw query: `diesel` refuses to join plain tables once an `Alias` of
    // `pg_class` is in the from clause, and the TOAST row needs exactly that
    // self-join next to `pg_get_expr` on the node-tree bound.
    let name = table.table_name.replace('\'', "''");
    let schema = table.table_schema.replace('\'', "''");
    let row = diesel::sql_query(
        format!(
            r"
            SELECT
                cls.oid AS oid,
                cls.relrowsecurity AS row_security,
                cls.relforcerowsecurity AS forced_row_security,
                cls.relowner AS owner,
                cls.relispartition AS is_partition,
                cls.relpersistence = 'u' AS unlogged,
                cls.reloptions AS reloptions,
                toast.reloptions AS toast_reloptions,
                pg_get_expr(cls.relpartbound, cls.oid) AS partition_bound,
                srv.srvname AS server_name,
                ft.ftoptions AS server_options,
                cls.relacl IS NULL AS acl_is_null
            FROM pg_catalog.pg_class AS cls
            INNER JOIN pg_catalog.pg_namespace AS nsp
                ON nsp.oid = cls.relnamespace
            LEFT JOIN pg_catalog.pg_foreign_table AS ft
                ON ft.ftrelid = cls.oid
            LEFT JOIN pg_catalog.pg_foreign_server AS srv
                ON srv.oid = ft.ftserver
            LEFT JOIN pg_catalog.pg_class AS toast
                ON toast.oid = cls.reltoastrelid
            WHERE cls.relname = '{name}'
              AND nsp.nspname = '{schema}'
            "
        )
        .as_str(),
    )
    .load::<TableFactsRow>(conn)?
    .into_iter()
    .next()
    .ok_or(diesel::result::Error::NotFound)?;

    let (
        oid,
        row_security,
        forced_row_security,
        owner,
        is_partition,
        unlogged,
        reloptions,
        toast_reloptions,
        partition_bound,
        server_name,
        server_options,
        acl_is_null,
    ) = (
        row.oid,
        row.row_security,
        row.forced_row_security,
        row.owner,
        row.is_partition,
        row.unlogged,
        row.reloptions,
        row.toast_reloptions,
        row.partition_bound,
        row.server_name,
        row.server_options,
        row.acl_is_null,
    );

    Ok(TableFacts {
        oid,
        row_security,
        forced_row_security,
        owner,
        is_partition,
        unlogged,
        reloptions: reloptions.unwrap_or_default(),
        toast_reloptions: toast_reloptions.unwrap_or_default(),
        partition_bound,
        server_name,
        server_options: server_options.unwrap_or_default(),
        acl_is_null,
    })
}

/// One entry of a table's access privileges, as `aclexplode` reads
/// `pg_class.relacl`.
#[derive(diesel::QueryableByName, Debug, Clone, PartialEq, Eq)]
pub(crate) struct TableAclEntry {
    /// The OID of the role holding the privilege.
    #[diesel(sql_type = diesel::sql_types::Oid)]
    pub(crate) grantee: u32,
    /// The privilege `PostgreSQL` names for the entry.
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) privilege_type: String,
}

/// One row per privilege `pg_class.relacl` records, read raw because
/// `aclexplode` is a table function the typed DSL cannot put in a `FROM`
/// clause, the way `pg_proc`'s `proacl` grants are read.
pub(crate) fn table_acl_entries(
    table_oid: u32,
    conn: &mut PgConnection,
) -> Result<Vec<TableAclEntry>, diesel::result::Error> {
    diesel::sql_query(
        format!(
            r"
            SELECT
                acl.grantee AS grantee,
                acl.privilege_type AS privilege_type
            FROM pg_catalog.pg_class AS cls
            CROSS JOIN LATERAL aclexplode(cls.relacl) AS acl (
                grantor,
                grantee,
                privilege_type,
                is_grantable
            )
            WHERE cls.oid = {table_oid}
            "
        )
        .as_str(),
    )
    .load::<TableAclEntry>(conn)
}

/// The width `pg_attribute.attndims` has, `integer` before `PostgreSQL` 16
/// and `smallint` from it on.
#[cfg(not(any(
    feature = "postgres-16",
    feature = "postgres-17",
    feature = "postgres-18"
)))]
type Dimensions = i32;
/// The width `pg_attribute.attndims` has, `integer` before `PostgreSQL` 16
/// and `smallint` from it on.
#[cfg(any(
    feature = "postgres-16",
    feature = "postgres-17",
    feature = "postgres-18"
))]
type Dimensions = i16;

/// Returns what `pg_attribute` records for every live column of the table.
pub(crate) fn attributes(
    table_oid: u32,
    conn: &mut PgConnection,
) -> Result<Vec<AttributeFacts>, diesel::result::Error> {
    use crate::schema::pg_catalog::pg_attribute::pg_attribute;

    Ok(pg_attribute::table
        .filter(pg_attribute::attrelid.eq(table_oid))
        .filter(pg_attribute::attnum.gt(0))
        .filter(pg_attribute::attisdropped.eq(false))
        .order_by(pg_attribute::attnum)
        .select((
            pg_attribute::attname,
            pg_attribute::attnum,
            pg_attribute::atttypid,
            pg_attribute::atttypmod,
            pg_attribute::attndims,
            pg_attribute::attfdwoptions,
            pg_attribute::attislocal,
        ))
        .load::<(String, i16, u32, i32, Dimensions, Option<Vec<String>>, bool)>(conn)?
        .into_iter()
        .map(
            |(name, number, type_oid, typmod, ndims, fdw_options, local)| AttributeFacts {
                name,
                number,
                type_oid,
                typmod,
                #[cfg(any(
                    feature = "postgres-16",
                    feature = "postgres-17",
                    feature = "postgres-18"
                ))]
                ndims: i32::from(ndims),
                #[cfg(not(any(
                    feature = "postgres-16",
                    feature = "postgres-17",
                    feature = "postgres-18"
                )))]
                ndims,
                fdw_options,
                inherited: !local,
            },
        )
        .collect())
}

/// Returns the position of every column whose default draws from a sequence
/// the column owns, the columns declared with a serial pseudo-type.
///
/// `PostgreSQL` records the ownership as an automatic dependency from the
/// sequence on the column, and the default as a normal dependency from the
/// column's `pg_attrdef` row on the same sequence. A column is serial when
/// both rows name one sequence.
pub(crate) fn serial_attributes(
    table_oid: u32,
    conn: &mut PgConnection,
) -> Result<Vec<i32>, diesel::result::Error> {
    use crate::schema::pg_catalog::{
        pg_attrdef::pg_attrdef, pg_class::pg_class, pg_depend::pg_depend,
    };

    let owned = pg_depend::table
        .inner_join(pg_class::table.on(pg_depend::objid.eq(pg_class::oid)))
        .filter(
            pg_depend::classid.eq(diesel::dsl::sql::<diesel::sql_types::Oid>(
                "'pg_class'::regclass",
            )),
        )
        .filter(pg_depend::objsubid.eq(0))
        .filter(
            pg_depend::refclassid.eq(diesel::dsl::sql::<diesel::sql_types::Oid>(
                "'pg_class'::regclass",
            )),
        )
        .filter(pg_depend::refobjid.eq(table_oid))
        .filter(pg_depend::deptype.eq("a"))
        .filter(pg_class::relkind.eq("S"))
        .select((pg_depend::objid, pg_depend::refobjsubid))
        .load::<(u32, i32)>(conn)?;
    let defaulted = pg_attrdef::table
        .inner_join(pg_depend::table.on(pg_attrdef::oid.eq(pg_depend::objid)))
        .filter(pg_attrdef::adrelid.eq(table_oid))
        .filter(
            pg_depend::classid.eq(diesel::dsl::sql::<diesel::sql_types::Oid>(
                "'pg_attrdef'::regclass",
            )),
        )
        .filter(pg_depend::deptype.eq("n"))
        .filter(
            pg_depend::refclassid.eq(diesel::dsl::sql::<diesel::sql_types::Oid>(
                "'pg_class'::regclass",
            )),
        )
        .select((pg_attrdef::adnum, pg_depend::refobjid))
        .load::<(i16, u32)>(conn)?;
    Ok(defaulted
        .into_iter()
        .map(|(adnum, sequence)| (i32::from(adnum), sequence))
        .filter(|(attnum, sequence)| {
            owned
                .iter()
                .any(|(owned_sequence, attn)| *owned_sequence == *sequence && *attn == *attnum)
        })
        .map(|(attnum, _)| attnum)
        .collect())
}

/// Returns every comment recorded against the table, its own and its columns'.
///
/// A comment on the table itself carries `objsubid` zero, and a comment on a
/// column carries that column's `attnum`.
pub(crate) fn descriptions(
    table_oid: u32,
    conn: &mut PgConnection,
) -> Result<Vec<PgDescription>, diesel::result::Error> {
    use crate::schema::pg_catalog::{pg_class::pg_class, pg_description::pg_description};

    pg_description::table
        .inner_join(pg_class::table.on(pg_description::classoid.eq(pg_class::oid)))
        .filter(pg_description::objoid.eq(table_oid))
        .filter(pg_class::relname.eq("pg_class"))
        .select(PgDescription::as_select())
        .load::<PgDescription>(conn)
}

/// Returns the partitioning strategy of the table, when it is partitioned.
pub(crate) fn partition_strategy(
    table_oid: u32,
    conn: &mut PgConnection,
) -> Result<Option<String>, diesel::result::Error> {
    use crate::schema::pg_catalog::pg_partitioned_table::pg_partitioned_table;

    pg_partitioned_table::table
        .filter(pg_partitioned_table::partrelid.eq(table_oid))
        .select(pg_partitioned_table::partstrat)
        .first::<String>(conn)
        .optional()
}

/// Returns the schema and name of every relation the table descends from.
pub(crate) fn parents(
    table_oid: u32,
    conn: &mut PgConnection,
) -> Result<Vec<(String, String)>, diesel::result::Error> {
    use crate::schema::pg_catalog::{
        pg_class::pg_class, pg_inherits::pg_inherits, pg_namespace::pg_namespace,
    };

    let parent_oids = pg_inherits::table
        .filter(pg_inherits::inhrelid.eq(table_oid))
        .order_by(pg_inherits::inhseqno)
        .select(pg_inherits::inhparent)
        .load::<u32>(conn)?;

    if parent_oids.is_empty() {
        return Ok(Vec::new());
    }

    let names: std::collections::HashMap<u32, (String, String)> = pg_class::table
        .inner_join(pg_namespace::table.on(pg_class::relnamespace.eq(pg_namespace::oid)))
        .filter(pg_class::oid.eq_any(&parent_oids))
        .select((pg_class::oid, pg_namespace::nspname, pg_class::relname))
        .load::<(u32, String, String)>(conn)?
        .into_iter()
        .map(|(oid, schema, name)| (oid, (schema, name)))
        .collect();

    Ok(parent_oids
        .iter()
        .filter_map(|oid| names.get(oid).cloned())
        .collect())
}
