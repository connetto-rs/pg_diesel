//! Submodule defining the cached queries methods used in the [`Table`] struct.

use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    OptionalExtension, PgConnection, QueryDsl, Queryable, RunQueryDsl, SelectableHelper,
    dsl::sql,
    sql_types::{Bool, Nullable, Text},
};

use crate::{
    database::ServerVersion,
    model_metadata::{IndexConstraint, decode_deferral},
    models::{Column, KeyColumnUsage, PgDescription, Table},
    shared::models::PgIndex,
};

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
    use crate::{
        schema::information_schema::{columns::columns, key_column_usage::key_column_usage},
        shared::schema::information_schema::table_constraints,
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

/// An index row with the name `pg_class` records the index relation under,
/// the name of its access method, whether it treats nulls as equal to one
/// another, the statement `pg_get_indexdef` prints for it, and the predicate
/// `pg_get_expr` prints for a partial index.
#[derive(Queryable)]
pub(crate) struct IndexRow {
    /// The `pg_index` row.
    pub(crate) index: PgIndex,
    /// The name of the index relation.
    pub(crate) name: String,
    /// The `pg_am.amname` of the index's access method.
    pub(crate) access_method: String,
    /// Whether the index was created `NULLS NOT DISTINCT`.
    pub(crate) nulls_not_distinct: bool,
    /// The `CREATE INDEX` statement `pg_get_indexdef` prints for the index.
    pub(crate) definition: String,
    /// The predicate of a partial index as `pg_get_expr` prints it.
    pub(crate) predicate: Option<String>,
}

/// The `CREATE INDEX` statement of the selected `pg_index` row as
/// `pg_get_indexdef` prints it, each key with the collation, operator class,
/// sort direction and null ordering that differ from their defaults.
///
/// The pretty print wraps an expression key in the one pair of parentheses
/// `CREATE INDEX` asks for, where the plain print adds a second pair around
/// an operator expression.
fn printed_definition() -> diesel::expression::SqlLiteral<Text> {
    sql::<Text>("pg_get_indexdef(pg_index.indexrelid, 0, true)")
}

/// The predicate of the selected `pg_index` row as `pg_get_expr` prints it,
/// `NULL` for an index over every row.
fn printed_predicate() -> diesel::expression::SqlLiteral<Nullable<Text>> {
    sql::<Nullable<Text>>("pg_get_expr(indpred, indrelid)")
}

/// Whether the selected `pg_index` row treats nulls as equal to one another,
/// which a server of `version` records only from `PostgreSQL` 15 on.
fn nulls_not_distinct(version: ServerVersion) -> diesel::expression::SqlLiteral<Bool> {
    sql::<Bool>(if version.records_nulls_not_distinct() {
        "indnullsnotdistinct"
    } else {
        "false"
    })
}

/// The `pg_constraint.contype` of the primary key, unique and exclusion
/// constraints, which an index of their own table enforces.
///
/// A foreign key's `conindid` names the index the referenced columns hold,
/// on another table, so a foreign key says nothing about what that index
/// enforces.
const INDEX_BACKED_CONSTRAINT_TYPES: [&str; 3] = ["p", "u", "x"];

/// Returns the indices of the table with OID `table_oid` a `PRIMARY KEY`,
/// `UNIQUE` or `EXCLUDE` constraint created, read from a server of `version`.
pub(crate) fn constraint_indices(
    table_oid: u32,
    version: ServerVersion,
    conn: &mut PgConnection,
) -> Result<Vec<IndexRow>, diesel::result::Error> {
    use crate::{
        schema::pg_catalog::pg_am::pg_am,
        shared::schema::pg_catalog::{pg_class, pg_constraint, pg_index},
    };

    pg_index::table
        .inner_join(pg_class::table.on(pg_class::oid.eq(pg_index::indexrelid)))
        .inner_join(pg_am::table.on(pg_am::oid.eq(pg_class::relam)))
        .filter(pg_index::indrelid.eq(table_oid))
        .filter(diesel::dsl::exists(
            pg_constraint::table
                .select(pg_constraint::oid)
                .filter(pg_constraint::conindid.eq(pg_index::indexrelid))
                .filter(pg_constraint::contype.eq_any(INDEX_BACKED_CONSTRAINT_TYPES)),
        ))
        .select((
            PgIndex::as_select(),
            pg_class::relname,
            pg_am::amname,
            nulls_not_distinct(version),
            printed_definition(),
            printed_predicate(),
        ))
        .load::<IndexRow>(conn)
}

/// Returns, for each index of the table, the primary key, unique or
/// exclusion constraint the index enforces, by the OID the engine gives the
/// index.
pub(crate) fn index_constraints(
    table_oid: u32,
    conn: &mut PgConnection,
) -> Result<Vec<(u32, IndexConstraint)>, diesel::result::Error> {
    use crate::shared::schema::pg_catalog::{pg_constraint, pg_index};

    Ok(pg_constraint::table
        .inner_join(pg_index::table.on(pg_constraint::conindid.eq(pg_index::indexrelid)))
        .filter(pg_index::indrelid.eq(table_oid))
        .filter(pg_constraint::contype.eq_any(INDEX_BACKED_CONSTRAINT_TYPES))
        .select((
            pg_constraint::conindid,
            pg_constraint::conname,
            pg_constraint::condeferrable,
            pg_constraint::condeferred,
        ))
        .load::<(u32, String, bool, bool)>(conn)?
        .into_iter()
        .map(|(index, name, deferrable, initially_deferred)| {
            (
                index,
                IndexConstraint::new(name, decode_deferral(deferrable, initially_deferred)),
            )
        })
        .collect())
}

/// Returns all indices of the table or materialized view with OID
/// `relation_oid`, read from a server of `version`.
pub(crate) fn indices(
    relation_oid: u32,
    version: ServerVersion,
    conn: &mut PgConnection,
) -> Result<Vec<IndexRow>, diesel::result::Error> {
    use crate::{
        schema::pg_catalog::pg_am::pg_am,
        shared::schema::pg_catalog::{pg_class, pg_index},
    };

    pg_index::table
        .inner_join(pg_class::table.on(pg_class::oid.eq(pg_index::indexrelid)))
        .inner_join(pg_am::table.on(pg_am::oid.eq(pg_class::relam)))
        .filter(pg_index::indrelid.eq(relation_oid))
        .select((
            PgIndex::as_select(),
            pg_class::relname,
            pg_am::amname,
            nulls_not_distinct(version),
            printed_definition(),
            printed_predicate(),
        ))
        .load::<IndexRow>(conn)
}

/// Returns the indices of the table with OID `table_oid` no constraint
/// created, unique or not, read from a server of `version`.
pub(crate) fn standalone_indices(
    table_oid: u32,
    version: ServerVersion,
    conn: &mut PgConnection,
) -> Result<Vec<IndexRow>, diesel::result::Error> {
    let all = indices(table_oid, version, conn)?;
    let backed: Vec<u32> = constraint_indices(table_oid, version, conn)?
        .into_iter()
        .map(|row| row.index.indexrelid)
        .collect();

    Ok(all
        .into_iter()
        .filter(|row| !backed.contains(&row.index.indexrelid))
        .collect())
}

/// Returns the name each not-null constraint of the table holds, by the
/// number the engine gives the column it applies to.
///
/// The engine records such constraints as `pg_constraint` rows only from
/// `PostgreSQL` 18 on, so an older server returns none.
pub(crate) fn not_null_constraint_names(
    table_oid: u32,
    conn: &mut PgConnection,
) -> Result<Vec<(i32, String)>, diesel::result::Error> {
    use crate::shared::schema::pg_catalog::pg_constraint;

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

/// Returns the names of the columns the foreign key `constraint_name` of the
/// table references, in the order its `confkey` lists them.
pub(crate) fn referenced_column_names(
    table_oid: u32,
    constraint_name: &str,
    conn: &mut PgConnection,
) -> Result<Vec<String>, diesel::result::Error> {
    use crate::shared::schema::pg_catalog::{pg_attribute, pg_constraint};

    let (referenced_oid, key) = pg_constraint::table
        .filter(pg_constraint::conrelid.eq(table_oid))
        .filter(pg_constraint::conname.eq(constraint_name))
        .filter(pg_constraint::contype.eq("f"))
        .select((pg_constraint::confrelid, pg_constraint::confkey))
        .first::<(u32, Option<Vec<i16>>)>(conn)?;
    let key = key.ok_or(diesel::result::Error::NotFound)?;
    let mut names = pg_attribute::table
        .filter(pg_attribute::attrelid.eq(referenced_oid))
        .filter(pg_attribute::attnum.eq_any(&key))
        .select((pg_attribute::attnum, pg_attribute::attname))
        .load::<(i16, String)>(conn)?;
    key.iter()
        .map(|number| {
            names
                .iter_mut()
                .find(|(attnum, _)| attnum == number)
                .map(|(_, name)| std::mem::take(name))
                .ok_or(diesel::result::Error::NotFound)
        })
        .collect()
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
    /// The `attgenerated` letter: `s` for a stored generated column, `v`
    /// for a virtual one, and empty for a column that is not generated.
    pub(crate) generation: String,
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
    /// The replica identity `pg_class.relreplident` records, one letter.
    #[diesel(sql_type = diesel::sql_types::Text)]
    replica_identity: String,
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
    /// The replica identity `pg_class.relreplident` records: `d` for
    /// `DEFAULT`, `n` for `NOTHING`, `f` for `FULL` and `i` for
    /// `USING INDEX`.
    pub(crate) replica_identity: String,
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
/// its identity, row security settings, replica identity, owner, partition
/// status and bound, its storage parameters, the server a foreign table
/// reads through, and whether an explicit ACL was written.
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
                cls.relreplident::text AS replica_identity,
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
        replica_identity,
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
        row.replica_identity,
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
        replica_identity,
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

/// Returns what `pg_attribute` records for every live column of the table or
/// materialized view with OID `relation_oid`.
pub(crate) fn attributes(
    relation_oid: u32,
    conn: &mut PgConnection,
) -> Result<Vec<AttributeFacts>, diesel::result::Error> {
    use crate::shared::schema::pg_catalog::pg_attribute;

    // `attndims` is an `integer` before `PostgreSQL` 16 and a `smallint`
    // from it on, so the cast reads both widths alike. `attgenerated` is a
    // `"char"`, whose empty value reads as an empty `text`.
    Ok(pg_attribute::table
        .filter(pg_attribute::attrelid.eq(relation_oid))
        .filter(pg_attribute::attnum.gt(0))
        .filter(pg_attribute::attisdropped.eq(false))
        .order_by(pg_attribute::attnum)
        .select((
            pg_attribute::attname,
            pg_attribute::attnum,
            pg_attribute::atttypid,
            pg_attribute::atttypmod,
            diesel::dsl::sql::<diesel::sql_types::Integer>("attndims::integer"),
            pg_attribute::attfdwoptions,
            pg_attribute::attislocal,
            diesel::dsl::sql::<diesel::sql_types::Text>("attgenerated::text"),
        ))
        .load::<(
            String,
            i16,
            u32,
            i32,
            i32,
            Option<Vec<String>>,
            bool,
            String,
        )>(conn)?
        .into_iter()
        .map(
            |(name, number, type_oid, typmod, ndims, fdw_options, local, generation)| {
                AttributeFacts {
                    name,
                    number,
                    type_oid,
                    typmod,
                    ndims,
                    fdw_options,
                    inherited: !local,
                    generation,
                }
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
    use crate::{
        schema::pg_catalog::{pg_attrdef::pg_attrdef, pg_depend::pg_depend},
        shared::schema::pg_catalog::pg_class,
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
    use crate::{
        schema::pg_catalog::pg_description::pg_description, shared::schema::pg_catalog::pg_class,
    };

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
    use crate::{
        schema::pg_catalog::{pg_inherits::pg_inherits, pg_namespace::pg_namespace},
        shared::schema::pg_catalog::pg_class,
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
