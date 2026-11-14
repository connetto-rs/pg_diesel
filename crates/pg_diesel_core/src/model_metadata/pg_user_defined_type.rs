//! The user-defined type a `pg_type` row of type `c`, `e`, `r` or `b`, or a
//! shell, becomes, the object `CREATE TYPE` defines.
//!
//! `PostgreSQL` holds a user-defined type beside the relations of the schema
//! it names. A composite type is the one whose relation `pg_class` records
//! as a composite, so the row type of a table, view or materialized view is
//! not one. A base type is the one `pg_type` records as a base that is no
//! array type, built-in ones excluded. A range type brings a multirange of
//! its own, which names the pool but is not a type the schema lists of its
//! own. A shell is the bare `CREATE TYPE name;`, which stays undefined until
//! a later definition fills it.

use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl,
};
use sql_traits::traits::UserDefinedTypeKind;

use crate::database::FIRST_NORMAL_OBJECT_ID;
use crate::{
    schema::pg_catalog::{pg_namespace::pg_namespace, pg_type::pg_type},
    shared::schema::pg_catalog::pg_class,
};

/// A user-defined type read from `pg_type`, beside the form it takes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgUserDefinedType {
    /// The stored name of the type, in exact case.
    name: String,
    /// The stored name of the schema the type lives in, in exact case.
    schema: String,
    /// The form the type takes, as `CREATE TYPE` writes it.
    kind: UserDefinedTypeKind,
}

impl PgUserDefinedType {
    /// Returns the name of the type.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the schema the type lives in.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the form the type takes.
    #[must_use]
    pub fn kind(&self) -> UserDefinedTypeKind {
        self.kind
    }

    /// Loads every user-defined type of the schemas `schemas` names, from
    /// the rows `pg_type` holds for them.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the query fails.
    pub fn load_all(
        schemas: &[String],
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        // A composite type is one whose relation `pg_class` records as a
        // composite, so a table row type is not.
        let composite = pg_type::typtype.eq("c").and(
            pg_type::typrelid.eq_any(
                pg_class::table
                    .filter(pg_class::relkind.eq("c"))
                    .select(pg_class::oid),
            ),
        );
        // A base type is no array type, which `pg_type` marks with the
        // element it holds.
        let base = pg_type::typtype.eq("b").and(pg_type::typelem.eq(0));
        // A shell is an undefined pseudo type, the bare `CREATE TYPE name;`,
        // every built-in pseudo type excluded by the OID filter.
        let shell = pg_type::typtype
            .eq("p")
            .and(pg_type::typisdefined.eq(false));

        Ok(pg_type::table
            .inner_join(pg_namespace::table.on(pg_type::typnamespace.eq(pg_namespace::oid)))
            .filter(pg_namespace::nspname.eq_any(schemas))
            .filter(
                pg_type::oid.ge(FIRST_NORMAL_OBJECT_ID).and(
                    pg_type::typtype
                        .eq("e")
                        .or(pg_type::typtype.eq("r"))
                        .or(composite)
                        .or(base)
                        .or(shell),
                ),
            )
            .select((pg_type::typname, pg_namespace::nspname, pg_type::typtype))
            .load::<(String, String, String)>(conn)?
            .into_iter()
            .map(|(name, schema, typtype)| Self {
                kind: match typtype.as_str() {
                    "c" => UserDefinedTypeKind::Composite,
                    "e" => UserDefinedTypeKind::Enum,
                    "r" => UserDefinedTypeKind::Range,
                    "b" => UserDefinedTypeKind::Base,
                    _ => UserDefinedTypeKind::Shell,
                },
                name,
                schema,
            })
            .collect())
    }
}
