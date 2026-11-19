//! Submodule providing [`PgIndexEntry`], the index a `PgDieselDatabase` holds.

use sql_traits::structs::IndexKey;
use sqlparser::{
    ast::{Expr, Ident, IndexColumn, IndexType, ObjectName, OrderByOptions},
    dialect::PostgreSqlDialect,
    keywords::Keyword,
    parser::{Parser, ParserError},
    tokenizer::Token,
};

use crate::shared::models::PgIndex;

/// What an index is beyond its key and predicate, as `pg_class.relam` and
/// `pg_index` record it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct IndexShape {
    /// The access method of the index.
    access_method: IndexType,
    /// The names of the columns the `INCLUDE` list carries, in the order
    /// written.
    include_column_names: Vec<String>,
    /// Whether the index treats nulls as distinct from one another.
    nulls_distinct: bool,
}

impl IndexShape {
    /// Creates the shape of an index whose access method `pg_am.amname`
    /// names `access_method`.
    #[must_use]
    pub(crate) fn new(
        access_method: &str,
        include_column_names: Vec<String>,
        nulls_distinct: bool,
    ) -> Self {
        let access_method = match access_method {
            "btree" => IndexType::BTree,
            "hash" => IndexType::Hash,
            "gin" => IndexType::GIN,
            "gist" => IndexType::GiST,
            "spgist" => IndexType::SPGiST,
            "brin" => IndexType::BRIN,
            "bloom" => IndexType::Bloom,
            other => IndexType::Custom(Ident::new(other)),
        };
        Self {
            access_method,
            include_column_names,
            nulls_distinct,
        }
    }
}

/// The sort direction, null ordering and operator class of one key of an
/// index, each `None` where the key keeps its default.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct KeyOptions {
    /// The sort direction and null ordering of the key.
    order: OrderByOptions,
    /// The operator class of the key.
    operator_class: Option<ObjectName>,
}

impl KeyOptions {
    /// Returns `key` carrying these options.
    fn apply<'a>(&'a self, mut key: IndexKey<'a>) -> IndexKey<'a> {
        if let Some(order) = &self.order.sort {
            key = key.with_order(order);
        }
        if let Some(nulls_first) = self.order.nulls_first {
            key = key.with_nulls_first(nulls_first);
        }
        if let Some(operator_class) = &self.operator_class {
            key = key.with_operator_class(operator_class);
        }
        key
    }
}

/// Returns the keys of `definition`, a `CREATE INDEX` statement as
/// `pg_get_indexdef` prints it, each with the collation, operator class,
/// sort direction and null ordering the print gives it.
///
/// Only the head and the key are read. The print of an index on a
/// partitioned table names the table `ON ONLY`, and what follows the key
/// the catalog columns already answer.
///
/// # Errors
///
/// Returns the parser's error when `definition` does not open with the head
/// and the key of a `CREATE INDEX` statement.
pub(crate) fn printed_key(definition: &str) -> Result<Vec<IndexColumn>, ParserError> {
    let mut parser = Parser::new(&PostgreSqlDialect {}).try_with_sql(definition)?;
    parser.expect_keyword_is(Keyword::CREATE)?;
    let _ = parser.parse_keyword(Keyword::UNIQUE);
    parser.expect_keyword_is(Keyword::INDEX)?;
    parser.parse_identifier()?;
    parser.expect_keyword_is(Keyword::ON)?;
    let _ = parser.parse_keyword(Keyword::ONLY);
    parser.parse_object_name(false)?;
    parser.expect_keyword_is(Keyword::USING)?;
    parser.parse_identifier()?;
    parser.expect_token(&Token::LParen)?;
    let key = parser.parse_comma_separated(Parser::parse_create_index_expr)?;
    parser.expect_token(&Token::RParen)?;
    Ok(key)
}

/// A `pg_index` row paired with the name and schema its `indexrelid` names,
/// its access method, the columns of its `INCLUDE` list, its keys and the
/// predicate of a partial index.
///
/// `IndexLike` asks for these without handing over a database.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgIndexEntry {
    /// The `pg_catalog.pg_index` row describing the index.
    model: PgIndex,
    /// The schema the index lives in.
    schema: String,
    /// The name of the index.
    name: String,
    /// The access method, `INCLUDE` list and null treatment of the index.
    shape: IndexShape,
    /// The name the primary key, unique or exclusion constraint the index
    /// enforces holds, when one does.
    constraint_name: Option<String>,
    /// The expression of the single key in parentheses, or the tuple of the
    /// expressions of several keys in the order `indkey` lists them.
    expression: Expr,
    /// The options of each key, in the order of the expressions.
    key_options: Vec<KeyOptions>,
    /// The predicate `pg_get_expr` prints for a partial index.
    predicate: Option<Expr>,
}

impl PgIndexEntry {
    /// Pairs a `pg_index` row with the relation name its `indexrelid` names
    /// and the keys `key` its printed definition holds.
    #[must_use]
    pub(crate) fn new(
        model: PgIndex,
        schema: String,
        name: String,
        shape: IndexShape,
        constraint_name: Option<String>,
        key: Vec<IndexColumn>,
        predicate: Option<Expr>,
    ) -> Self {
        let (expressions, key_options): (Vec<Expr>, Vec<KeyOptions>) = key
            .into_iter()
            .map(|key| {
                (
                    key.column.expr,
                    KeyOptions {
                        order: key.column.options,
                        operator_class: key.operator_class,
                    },
                )
            })
            .unzip();
        let expression = match <[Expr; 1]>::try_from(expressions) {
            Ok([single]) => Expr::Nested(Box::new(single)),
            Err(expressions) => Expr::Tuple(expressions),
        };
        Self {
            model,
            schema,
            name,
            shape,
            constraint_name,
            expression,
            key_options,
            predicate,
        }
    }

    /// Returns the access method of the index.
    #[must_use]
    pub fn access_method(&self) -> &IndexType {
        &self.shape.access_method
    }

    /// Returns the names of the columns the `INCLUDE` list of the index
    /// carries, in the order written.
    #[must_use]
    pub fn include_column_names(&self) -> &[String] {
        &self.shape.include_column_names
    }

    /// Returns whether the index treats nulls as distinct from one another,
    /// `false` for an index created `NULLS NOT DISTINCT`.
    #[must_use]
    pub fn nulls_distinct(&self) -> bool {
        self.shape.nulls_distinct
    }

    /// Returns whether the index enforces uniqueness.
    #[must_use]
    pub fn is_unique(&self) -> bool {
        self.model.indisunique
    }

    /// Returns the schema the index lives in.
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Returns the name of the index.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the name the constraint the index enforces holds, when one
    /// does.
    #[must_use = "the index may enforce no constraint, so the caller decides"]
    pub fn constraint_name(&self) -> Option<&str> {
        self.constraint_name.as_deref()
    }

    /// Returns the predicate of a partial index, `None` for an index over
    /// every row.
    #[must_use]
    pub fn predicate(&self) -> Option<&Expr> {
        self.predicate.as_ref()
    }

    /// Returns the expression the index is defined over, its single key in
    /// parentheses or the tuple of its keys, as `ParserDB` holds the key of
    /// a `CREATE INDEX` statement.
    #[must_use]
    pub fn expression(&self) -> &Expr {
        &self.expression
    }

    /// Returns the keys of the index in the order `indkey` lists them, each
    /// borrowing its expression from [`Self::expression`] and carrying the
    /// options that differ from their defaults.
    pub fn keys(&self) -> impl Iterator<Item = IndexKey<'_>> {
        let expressions = match &self.expression {
            Expr::Nested(key) => std::slice::from_ref(key.as_ref()),
            Expr::Tuple(keys) => keys.as_slice(),
            key => std::slice::from_ref(key),
        };
        expressions
            .iter()
            .zip(&self.key_options)
            .map(|(expression, options)| options.apply(IndexKey::new(expression)))
    }
}

#[cfg(test)]
mod tests {
    use sql_traits::{prelude::ParserDB, traits::DatabaseLike, traits::IndexLike};
    use sqlparser::dialect::PostgreSqlDialect;

    use super::{IndexShape, PgIndexEntry, printed_key};
    use crate::shared::models::PgIndex;

    #[test]
    fn an_access_method_answers_what_parser_db_answers_for_its_using_clause() {
        for method in [
            "btree", "hash", "gin", "gist", "spgist", "brin", "bloom", "hnsw",
        ] {
            let database = ParserDB::parse::<PostgreSqlDialect>(&format!(
                "CREATE TABLE docs (body text);
                 CREATE INDEX probe ON docs USING {method} (body);"
            ))
            .expect("the schema parses");
            let index = database
                .indexes()
                .next()
                .expect("the schema holds one index");
            assert_eq!(
                Some(&IndexShape::new(method, Vec::new(), true).access_method),
                index.access_method(),
                "{method}"
            );
        }
    }

    #[test]
    fn printed_keys_answer_what_parser_db_answers_for_the_written_keys() {
        for (written, printed) in [
            (
                "(title)",
                "CREATE INDEX probe ON app.docs USING btree (title)",
            ),
            (
                "(title DESC NULLS LAST)",
                "CREATE INDEX probe ON app.docs USING btree (title DESC NULLS LAST)",
            ),
            (
                "(a, lower(title))",
                "CREATE INDEX probe ON app.docs USING btree (a, lower(title))",
            ),
            (
                "(lower(title), upper(body))",
                "CREATE INDEX probe ON app.docs USING btree (lower(title), upper(body))",
            ),
            (
                "((a + 1))",
                "CREATE INDEX probe ON app.docs USING btree ((a + 1))",
            ),
            (
                "(lower(title) COLLATE \"C\" text_pattern_ops DESC, a NULLS FIRST)",
                "CREATE UNIQUE INDEX probe ON ONLY app.docs USING btree \
                 (lower(title) COLLATE \"C\" text_pattern_ops DESC, a NULLS FIRST) \
                 INCLUDE (body) WHERE a > 0",
            ),
        ] {
            let database = ParserDB::parse::<PostgreSqlDialect>(&format!(
                "CREATE TABLE docs (a int, title text, body text);
                 CREATE INDEX probe ON docs {written};"
            ))
            .expect("the schema parses");
            let parsed = database
                .indexes()
                .next()
                .expect("the schema holds one index");
            let live = PgIndexEntry::new(
                PgIndex {
                    indexrelid: 1,
                    indrelid: 2,
                    indisunique: false,
                    indkey: vec![1],
                    indnkeyatts: 1,
                },
                "app".to_owned(),
                "probe".to_owned(),
                IndexShape::new("btree", Vec::new(), true),
                None,
                printed_key(printed).expect("the print parses"),
                None,
            );
            assert_eq!(
                parsed.expression(&database).expect("in database"),
                live.expression(),
                "{written}"
            );
            assert_eq!(
                parsed
                    .keys(&database)
                    .expect("in database")
                    .collect::<Vec<_>>(),
                live.keys().collect::<Vec<_>>(),
                "{written}"
            );
        }
    }
}
