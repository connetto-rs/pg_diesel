//! Implementation of [`ColumnLike`] for [`PgColumn`].

use std::borrow::Cow;

use sql_traits::{
    errors::{LookupError, ObjectKind},
    structs::TargetName,
    traits::{
        ColumnCollation, ColumnLike, CreatedFromQuery, DatabaseLike, IdentityGeneration, Metadata,
        NamedColumnCollation,
    },
};

use crate::{
    PgDieselDatabase,
    model_metadata::{ColumnMetadata, PgColumn},
};

impl Metadata for PgColumn {
    type Meta = ColumnMetadata;
}

/// Returns the metadata `database` holds for `column`.
fn metadata<'db>(
    column: &PgColumn,
    database: &'db PgDieselDatabase,
) -> Result<&'db ColumnMetadata, LookupError> {
    let model = column.model();
    database
        .column_metadata(column)
        .ok_or_else(|| LookupError::ObjectNotInDatabase {
            object_kind: ObjectKind::Column,
            object: format!(
                "{}.{}.{}",
                model.table_schema, model.table_name, model.column_name
            ),
        })
}

impl ColumnLike for PgColumn {
    type DB = PgDieselDatabase;

    fn column_name(&self) -> &str {
        self.column_name()
    }

    fn column_name_is_quoted(&self) -> bool {
        // Catalog names are stored names: folding again would rename them.
        true
    }

    fn column_doc<'db>(&'db self, database: &'db Self::DB) -> Result<Option<&'db str>, LookupError>
    where
        Self: 'db,
    {
        Ok(metadata(self, database)?
            .description()
            .map(|desc| desc.description.as_str()))
    }

    fn table<'db>(&'db self, database: &'db Self::DB) -> &'db <Self::DB as DatabaseLike>::Table
    where
        Self: 'db,
    {
        database
            .column_metadata(self)
            .expect("Column must exist in database")
            .table()
    }

    fn data_type<'db>(
        &'db self,
        _database: &'db Self::DB,
    ) -> &'db <Self::DB as DatabaseLike>::DataType {
        self.declared_type()
    }

    fn collation<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<ColumnCollation<'db>, LookupError> {
        let model = self.model();
        let Some(name) = model.collation_name.as_deref() else {
            // Empty collation columns mean the database default.
            return Ok(ColumnCollation::DatabaseDefault);
        };
        let mut target = TargetName::new(name, true);
        if let Some(schema) = model.collation_schema.as_deref() {
            target = target.with_schema(schema, true);
        }
        let deterministic = metadata(self, database)?.collation_is_deterministic();
        Ok(ColumnCollation::Named(
            NamedColumnCollation::new(target).with_postgres_deterministic(deterministic),
        ))
    }

    fn identity_generation(&self, _database: &Self::DB) -> Option<IdentityGeneration> {
        match self.model().identity_generation.as_deref() {
            Some("ALWAYS") => Some(IdentityGeneration::Always),
            Some("BY DEFAULT") => Some(IdentityGeneration::ByDefault),
            _ => None,
        }
    }

    // `PostgreSQL` records a type for every column a `CREATE TABLE … AS`
    // creates, in `pg_attribute.atttypid`, so no column is left untyped.
    fn created_from_query<'db>(
        &'db self,
        _database: &'db Self::DB,
    ) -> Option<CreatedFromQuery<'db, Self::DB>> {
        None
    }

    fn is_serial(&self, _database: &Self::DB) -> bool {
        self.is_serial()
    }

    fn is_nullable(&self, _database: &Self::DB) -> Result<bool, LookupError> {
        Ok(self.model().__is_nullable == "YES")
    }

    fn not_null_constraint_name<'db>(
        &'db self,
        database: &'db Self::DB,
    ) -> Result<Option<Cow<'db, str>>, LookupError> {
        metadata(self, database)?;
        Ok(PgColumn::not_null_constraint_name(self).map(Cow::Borrowed))
    }

    fn default_value(&self) -> Option<String> {
        self.model().column_default.clone()
    }

    fn foreign_options(&self) -> impl Iterator<Item = (Cow<'_, str>, &str)> {
        self.foreign_options()
            .map(|(key, value)| (Cow::Borrowed(key), value))
    }
}
