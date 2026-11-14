//! The sequence a `pg_sequence` row becomes, beside the column that owns it.
//!
//! A sequence has no column recording why it exists, so the origin is read
//! from the column it owns. The catalog ties a sequence to its owning column
//! with a `pg_depend` row of type `a` (a column-owned or `OWNED BY` sequence)
//! or `i` (an identity sequence), and an identity column is told by
//! `pg_attribute.attidentity`. A serial column is told by the normal
//! `pg_depend` row from its `pg_attrdef` row on the sequence.

use std::collections::{HashMap, HashSet};

use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, PgConnection,
    QueryDsl, RunQueryDsl,
    dsl::sql,
    sql_types::{Bool, Oid},
};
use sql_traits::{
    structs::{SequenceParameters, SequenceType},
    traits::SequenceOrigin,
};

use crate::{
    schema::pg_catalog::{
        pg_attrdef::pg_attrdef, pg_depend::pg_depend, pg_namespace::pg_namespace,
        pg_sequence::pg_sequence,
    },
    shared::schema::pg_catalog::{pg_attribute, pg_class},
};

/// The sequence rows a read of `pg_sequence` answers.
#[derive(Debug, diesel::Queryable)]
struct SequenceRow {
    /// The OID of the sequence.
    seqrelid: u32,
    /// The stored name of the sequence.
    relname: String,
    /// The stored name of the schema the sequence lives in.
    nspname: String,
    /// The OID of the type the sequence counts in.
    seqtypid: u32,
    /// The start value of the sequence.
    seqstart: i64,
    /// The increment the sequence steps by.
    seqincrement: i64,
    /// The value the sequence stops at when it steps up.
    seqmax: i64,
    /// The value the sequence stops at when it steps down.
    seqmin: i64,
    /// The number of values allocated from the server per request.
    seqcache: i64,
    /// Whether the sequence restarts at its bound when it reaches the other.
    seqcycle: bool,
}

/// The owning-column rows a read of `pg_depend` answers, one per sequence.
#[derive(Debug, diesel::Queryable)]
struct OwnerRow {
    /// The OID of the sequence the dependency ties to a column.
    objid: u32,
    /// The position of the owned column in its relation, one-based.
    refobjsubid: i32,
    /// The stored name of the owning relation.
    owner_relname: String,
    /// The stored name of the schema the owning relation lives in.
    owner_nspname: String,
    /// The kind of the owning relation.
    owner_relkind: String,
    /// The identity marker of the owned column, if it is one.
    attidentity: Option<String>,
    /// The stored name of the owned column.
    attname: Option<String>,
    /// The OID of the `pg_attrdef` row of the owned column, if it has a
    /// default.
    attrdef_oid: Option<u32>,
}

/// The column or view column a sequence is owned by.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OwnedColumn {
    /// The stored name of the schema the owning relation lives in.
    pub schema: String,
    /// The stored name of the owning relation.
    pub name: String,
    /// Whether the owning relation is a plain view.
    pub is_view: bool,
    /// Whether the owned column is an identity column.
    pub is_identity: bool,
    /// The position of the owned column in its relation, one-based.
    pub attnum: i32,
    /// The stored name of the owned column.
    pub column: Option<String>,
    /// The OID of the `pg_attrdef` row of the owned column, if it has a
    /// default.
    pub attrdef_oid: Option<u32>,
}

/// A sequence read from `pg_sequence`, with its owner and origin resolved.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgSequenceEntry {
    /// The OID of the sequence.
    oid: u32,
    /// The stored name of the sequence, in exact case.
    name: String,
    /// The stored name of the schema the sequence lives in, in exact case.
    schema: String,
    /// The column the sequence is owned by, if a column owns it.
    owner: Option<OwnedColumn>,
    /// Why the sequence exists, read from the rows of its owning column.
    origin: SequenceOrigin,
    /// The values the sequence counts with, as `pg_sequence` records them.
    parameters: SequenceParameters,
}

impl PgSequenceEntry {
    /// Returns the name of the sequence.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the schema the sequence lives in.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the column the sequence is owned by, if a column owns it.
    #[must_use]
    pub fn owner(&self) -> Option<&OwnedColumn> {
        self.owner.as_ref()
    }

    /// Returns why the sequence exists.
    #[must_use]
    pub fn origin(&self) -> SequenceOrigin {
        self.origin
    }

    /// Returns the values the sequence counts with, as `pg_sequence`
    /// records them.
    #[must_use]
    pub fn parameters(&self) -> SequenceParameters {
        self.parameters
    }

    /// Loads every sequence of the schemas `schemas` names, with the owner
    /// and origin each one resolves to.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if a query fails.
    pub fn load_all(
        schemas: &[String],
        conn: &mut PgConnection,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        let rows = Self::sequence_rows(schemas, conn)?;
        let oids: Vec<u32> = rows.iter().map(|row| row.seqrelid).collect();
        let mut owners = Self::owner_rows(&oids, conn)?;
        let default_draws = Self::default_draws(&oids, conn)?;
        Ok(rows
            .into_iter()
            .map(|row| {
                let owner = owners.remove(&row.seqrelid).map(|owner| OwnedColumn {
                    is_view: owner.owner_relkind == "v",
                    is_identity: matches!(owner.attidentity.as_deref(), Some("a" | "d")),
                    attnum: owner.refobjsubid,
                    column: owner.attname,
                    schema: owner.owner_nspname,
                    name: owner.owner_relname,
                    attrdef_oid: owner.attrdef_oid,
                });
                let origin = owner.as_ref().map_or(SequenceOrigin::Declared, |owned| {
                    let draws = owned.attrdef_oid.is_some_and(|attrdef_oid| {
                        default_draws.contains(&(attrdef_oid, row.seqrelid))
                    });
                    Self::origin_for(owned, draws)
                });
                Self {
                    oid: row.seqrelid,
                    name: row.relname,
                    schema: row.nspname,
                    owner,
                    origin,
                    parameters: SequenceParameters::new(
                        Self::sequence_type(row.seqtypid),
                        row.seqincrement,
                        row.seqmin,
                        row.seqmax,
                        row.seqstart,
                        row.seqcache,
                        row.seqcycle,
                    )
                    .expect("pg_sequence holds consistent sequence parameters"),
                }
            })
            .collect())
    }

    /// The type the sequence `seqtypid` counts in.
    ///
    /// `PostgreSQL` counts sequences only in the three integer types, whose
    /// bootstrap OIDs are stable.
    fn sequence_type(seqtypid: u32) -> SequenceType {
        match seqtypid {
            21 => SequenceType::SmallInt,
            23 => SequenceType::Integer,
            20 => SequenceType::BigInt,
            other => panic!("pg_sequence.seqtypid {other} names no sequence type"),
        }
    }

    /// The sequence's own rows, one per sequence of the requested schemas.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the query fails.
    fn sequence_rows(
        schemas: &[String],
        conn: &mut PgConnection,
    ) -> Result<Vec<SequenceRow>, diesel::result::Error> {
        pg_sequence::table
            .inner_join(pg_class::table.on(pg_sequence::seqrelid.eq(pg_class::oid)))
            .inner_join(pg_namespace::table.on(pg_class::relnamespace.eq(pg_namespace::oid)))
            .filter(pg_namespace::nspname.eq_any(schemas))
            .select((
                pg_sequence::seqrelid,
                pg_class::relname,
                pg_namespace::nspname,
                pg_sequence::seqtypid,
                pg_sequence::seqstart,
                pg_sequence::seqincrement,
                pg_sequence::seqmax,
                pg_sequence::seqmin,
                pg_sequence::seqcache,
                pg_sequence::seqcycle,
            ))
            .load(conn)
    }

    /// The owning-column rows of the sequences `oids` names, ordered so the
    /// first row of a sequence is the one its origin reads.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the query fails.
    fn owner_rows(
        oids: &[u32],
        conn: &mut PgConnection,
    ) -> Result<HashMap<u32, OwnerRow>, diesel::result::Error> {
        let rows = pg_depend::table
            .inner_join(pg_class::table.on(pg_depend::refobjid.eq(pg_class::oid)))
            .inner_join(pg_namespace::table.on(pg_class::relnamespace.eq(pg_namespace::oid)))
            // `refobjsubid` is an int4 and `attnum`, `adnum` are int2, a
            // comparison the server coerces and the DSL cannot express.
            .left_join(
                pg_attribute::table.on(pg_depend::refobjid.eq(pg_attribute::attrelid).and(sql::<
                    Bool,
                >(
                    "pg_depend.refobjsubid = pg_attribute.attnum",
                ))),
            )
            .left_join(
                pg_attrdef::table.on(pg_depend::refobjid
                    .eq(pg_attrdef::adrelid)
                    .and(sql::<Bool>("pg_depend.refobjsubid = pg_attrdef.adnum"))),
            )
            .filter(pg_depend::classid.eq(sql::<Oid>("'pg_class'::regclass")))
            .filter(pg_depend::refclassid.eq(sql::<Oid>("'pg_class'::regclass")))
            .filter(pg_depend::deptype.eq_any(&["a", "i"]))
            .filter(pg_depend::refobjsubid.gt(0))
            .filter(pg_depend::objid.eq_any(oids))
            .order(pg_depend::objid.asc())
            .order(pg_depend::refobjsubid.asc())
            .select((
                pg_depend::objid,
                pg_depend::refobjsubid,
                pg_class::relname,
                pg_namespace::nspname,
                pg_class::relkind,
                pg_attribute::attidentity.nullable(),
                pg_attribute::attname.nullable(),
                pg_attrdef::oid.nullable(),
            ))
            .load::<OwnerRow>(conn)?;
        Ok(rows.into_iter().map(|row| (row.objid, row)).fold(
            HashMap::new(),
            |mut first, (oid, row)| {
                first.entry(oid).or_insert(row);
                first
            },
        ))
    }

    /// The origin a sequence whose owner is `owned` has, given whether the
    /// owner's default draws from the sequence.
    fn origin_for(owned: &OwnedColumn, draws: bool) -> SequenceOrigin {
        if owned.is_identity {
            return SequenceOrigin::Identity;
        }
        if draws {
            return SequenceOrigin::Serial;
        }
        SequenceOrigin::Declared
    }

    /// The pairs of `pg_attrdef` row and sequence whose default draws from
    /// the sequence, for the sequences `oids` names.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::result::Error` if the query fails.
    fn default_draws(
        oids: &[u32],
        conn: &mut PgConnection,
    ) -> Result<HashSet<(u32, u32)>, diesel::result::Error> {
        pg_depend::table
            .filter(pg_depend::classid.eq(sql::<Oid>("'pg_attrdef'::regclass")))
            .filter(pg_depend::deptype.eq("n"))
            .filter(pg_depend::refclassid.eq(sql::<Oid>("'pg_class'::regclass")))
            .filter(pg_depend::refobjid.eq_any(oids))
            .select((pg_depend::objid, pg_depend::refobjid))
            .load::<(u32, u32)>(conn)
            .map(|rows| rows.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_identity_beats_draw() {
        let owned = OwnedColumn {
            schema: "app".to_string(),
            name: "tickets".to_string(),
            is_view: false,
            is_identity: true,
            attnum: 1,
            column: Some("id".to_string()),
            attrdef_oid: Some(16401),
        };
        assert_eq!(
            PgSequenceEntry::origin_for(&owned, true),
            SequenceOrigin::Identity
        );
    }

    #[test]
    fn test_origin_serial_when_the_default_draws() {
        let owned = OwnedColumn {
            schema: "app".to_string(),
            name: "tickets".to_string(),
            is_view: false,
            is_identity: false,
            attnum: 2,
            column: Some("n".to_string()),
            attrdef_oid: Some(16402),
        };
        assert_eq!(
            PgSequenceEntry::origin_for(&owned, true),
            SequenceOrigin::Serial
        );
    }

    #[test]
    fn test_origin_declared_when_the_default_draws_elsewhere() {
        let owned = OwnedColumn {
            schema: "app".to_string(),
            name: "tickets".to_string(),
            is_view: false,
            is_identity: false,
            attnum: 3,
            column: Some("t".to_string()),
            attrdef_oid: Some(16403),
        };
        assert_eq!(
            PgSequenceEntry::origin_for(&owned, false),
            SequenceOrigin::Declared
        );
    }

    #[test]
    #[should_panic(expected = "names no sequence type")]
    fn an_unknown_sequence_type_panics() {
        let _ = PgSequenceEntry::sequence_type(25);
    }
}
