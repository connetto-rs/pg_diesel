//! The differences between `ParserDB` and `PgDieselDatabase` that are
//! understood and accepted, each listed once by name.
//!
//! Every entry names the cause. A difference no entry matches fails the
//! differential test.

use std::collections::BTreeSet;

use crate::compare::{Difference, Value};

/// What an entry may need to know about the database a document produced.
pub struct Context {
    /// The bootstrap superuser, `pg_authid` OID 10, which the DDL never
    /// names.
    pub bootstrap_superuser: String,
    /// The fact keys of the range and multirange constructor functions.
    pub range_constructors: BTreeSet<String>,
}

/// One understood difference.
pub struct Expected {
    /// A short name for the cause.
    pub name: &'static str,
    /// Why the two models answer differently and why that is accepted.
    pub reason: &'static str,
    /// Returns whether a difference is this one.
    pub matches: fn(&Difference, &Context) -> bool,
}

/// Every accepted difference.
pub const EXPECTED: &[Expected] = &[
    Expected {
        name: "range constructors owned by the bootstrap superuser",
        reason: "PostgreSQL makes the bootstrap superuser own the constructor functions of a range type, whoever creates the type, and the DDL never names that role, so ParserDB answers no owner and leaves the owner out of the EXECUTE holders",
        matches: |difference, context| {
            let (Some(Value::Text(source)), Some(Value::Text(live))) =
                (&difference.source, &difference.live)
            else {
                return false;
            };
            let Some((function, fact)) = difference
                .fact
                .strip_prefix("function ")
                .and_then(|rest| rest.rsplit_once(" / "))
            else {
                return false;
            };
            if !context.range_constructors.contains(function) {
                return false;
            }
            let bootstrap = &context.bootstrap_superuser;
            match fact {
                "owner" => source == "None" && *live == format!("Some({bootstrap:?})"),
                "execute holders" => {
                    let mut holders: Vec<&str> = source.split(", ").collect();
                    let owner = format!("Role({bootstrap:?})");
                    holders.push(&owner);
                    holders.sort_unstable();
                    holders.join(", ") == *live
                }
                _ => false,
            }
        },
    },
    Expected {
        name: "extension version chosen by the server",
        reason: "a CREATE EXTENSION without VERSION installs the default version the server ships, which the DDL does not name",
        matches: |difference, _| {
            difference.fact.ends_with(" / version")
                && difference.fact.starts_with("extension ")
                && difference.source == Some(Value::Text("None".to_owned()))
        },
    },
    Expected {
        name: "SQL comment kept as documentation",
        reason: "ParserDB reads the comment written before a CREATE as the object's documentation, which PostgreSQL discards, so the catalog holds no description",
        matches: |difference, _| {
            difference.fact.ends_with(" / comment")
                && difference.live == Some(Value::Text("None".to_owned()))
        },
    },
    Expected {
        name: "not-null constraints unnamed before PostgreSQL 18",
        reason: "ParserDB models PostgreSQL 18, which records a named pg_constraint row per NOT NULL column, while an older server records only attnotnull and has no name to answer",
        matches: |difference, _| {
            !cfg!(feature = "postgres-18")
                && difference.fact.ends_with(" / not null constraint name")
                && difference.live == Some(Value::Text("None".to_owned()))
        },
    },
    Expected {
        name: "MAINTAIN absent before PostgreSQL 17",
        reason: "ParserDB models PostgreSQL 18, whose owners hold MAINTAIN, a relation privilege an older server does not have",
        matches: |difference, _| {
            let (Some(Value::Text(source)), Some(Value::Text(live))) =
                (&difference.source, &difference.live)
            else {
                return false;
            };
            if cfg!(any(feature = "postgres-17", feature = "postgres-18")) {
                return false;
            }
            if difference.fact.contains(" / privilege MAINTAIN / role ") {
                return source == "true" && live == "false";
            }
            difference.fact.ends_with(" / owner privileges")
                && source
                    .split(", ")
                    .filter(|privilege| *privilege != "MAINTAIN")
                    .collect::<Vec<_>>()
                    .join(", ")
                    == *live
        },
    },
];

/// Returns the entry accepting `difference`, if any.
pub fn accepted(difference: &Difference, context: &Context) -> Option<&'static Expected> {
    EXPECTED
        .iter()
        .find(|expected| (expected.matches)(difference, context))
}
