//! Constraint type of `information_schema.table_constraints`.

use std::io::Write;

use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::{IsNull, Output, ToSql},
};

#[derive(Debug, FromSqlRow, AsExpression, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(sql_type = diesel::sql_types::Text)]
/// An enum representing the type of a constraint
pub enum ConstraintType {
    /// A primary key constraint
    PrimaryKey,
    /// A foreign key constraint
    ForeignKey,
    /// A unique constraint
    Unique,
    /// A check constraint
    Check,
}

impl std::str::FromStr for ConstraintType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PRIMARY KEY" => Ok(ConstraintType::PrimaryKey),
            "FOREIGN KEY" => Ok(ConstraintType::ForeignKey),
            "UNIQUE" => Ok(ConstraintType::Unique),
            "CHECK" => Ok(ConstraintType::Check),
            _ => Err(format!("Unknown constraint type: {s}")),
        }
    }
}

impl std::fmt::Display for ConstraintType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConstraintType::PrimaryKey => write!(f, "PRIMARY KEY"),
            ConstraintType::ForeignKey => write!(f, "FOREIGN KEY"),
            ConstraintType::Unique => write!(f, "UNIQUE"),
            ConstraintType::Check => write!(f, "CHECK"),
        }
    }
}

impl ToSql<diesel::sql_types::Text, diesel::pg::Pg> for ConstraintType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::pg::Pg>) -> diesel::serialize::Result {
        out.write_all(self.to_string().as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<diesel::sql_types::Text, diesel::pg::Pg> for ConstraintType {
    fn from_sql(
        bytes: <diesel::pg::Pg as Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let text = std::str::from_utf8(bytes.as_bytes())?;
        match text {
            "PRIMARY KEY" => Ok(ConstraintType::PrimaryKey),
            "FOREIGN KEY" => Ok(ConstraintType::ForeignKey),
            "UNIQUE" => Ok(ConstraintType::Unique),
            "CHECK" => Ok(ConstraintType::Check),
            _ => Err("Unknown constraint type".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ConstraintType;

    #[test]
    fn constraint_types_parse_back_from_their_sql_spelling() {
        for (kind, spelling) in [
            (ConstraintType::PrimaryKey, "PRIMARY KEY"),
            (ConstraintType::ForeignKey, "FOREIGN KEY"),
            (ConstraintType::Unique, "UNIQUE"),
            (ConstraintType::Check, "CHECK"),
        ] {
            assert_eq!(kind.to_string(), spelling);
            assert_eq!(spelling.parse::<ConstraintType>(), Ok(kind));
        }
        assert_eq!(
            "EXCLUDE".parse::<ConstraintType>(),
            Err("Unknown constraint type: EXCLUDE".to_owned())
        );
    }
}
