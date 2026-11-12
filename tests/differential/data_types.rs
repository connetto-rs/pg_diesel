//! Structured type facts loaded from PostgreSQL catalogs.

use diesel::{
    RunQueryDsl,
    dsl::sql,
    sql_types::{Oid, Text},
};
use pg_diesel::{
    PgDieselDatabase,
    database::CatalogCache,
    dialect::PostgresDialect,
    model_metadata::PgDataType,
    sql_traits::{
        structs::{TargetName, generic_db::GenericDBBuilder},
        traits::{ColumnCollation, DataTypeLike, StructureKind},
    },
};

use super::{PgConnection, SimpleConnection, establish_connection, reference_docker};

/// Resolves a declared type through the cached catalog.
fn declared_type(
    connection: &mut PgConnection,
    cache: &CatalogCache,
    name: &str,
) -> Option<PgDataType> {
    let oid: u32 = diesel::select(
        sql::<Oid>("CAST(")
            .bind::<Text, _>(name)
            .sql(" AS regtype)::oid"),
    )
    .get_result(connection)
    .unwrap();
    PgDataType::from_type(cache.pg_type(oid).unwrap(), -1, cache, 0)
}

#[tokio::test]
async fn structured_fields_preserve_nested_types_names_constraints_and_collations() {
    let docker = reference_docker("data_types").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "data_types").unwrap();
    connection
        .batch_execute(
            r#"
        CREATE TYPE leaf AS ("Label" varchar(7) COLLATE "C", discarded text, amount numeric(8,2));
        ALTER TYPE leaf DROP ATTRIBUTE discarded;
        CREATE DOMAIN leaf_domain AS leaf;
        CREATE TYPE empty_record AS ();
        CREATE TABLE row_type (required integer NOT NULL, optional text);
        CREATE TYPE outer_record AS (
            item leaf_domain, items leaf[], row_value row_type, vacant empty_record
        );
    "#,
        )
        .unwrap();
    let cache = CatalogCache::load(&mut connection).unwrap();
    let database: PgDieselDatabase =
        GenericDBBuilder::new("data_types".to_owned(), PostgresDialect).into();
    let outer = declared_type(&mut connection, &cache, "outer_record").unwrap();
    assert_eq!(outer.structure_kind(&database), Some(StructureKind::Struct));
    let fields: Vec<_> = outer.structured_fields(&database).unwrap().collect();
    assert_eq!(
        fields
            .iter()
            .map(|field| field.name().unwrap())
            .collect::<Vec<_>>(),
        ["item", "items", "row_value", "vacant"]
    );
    assert!(
        fields
            .iter()
            .all(|field| field.name_is_quoted() && field.is_nullable())
    );
    let domain = fields[0].data_type();
    assert!(domain.structure_kind(&database).is_none());
    assert!(domain.structured_fields(&database).is_none());
    assert!(domain.admits_null(&database));
    let leaf = domain.base_type(&database).unwrap();
    let leaf_fields: Vec<_> = leaf.structured_fields(&database).unwrap().collect();
    assert_eq!(
        leaf_fields
            .iter()
            .map(|field| field.name().unwrap())
            .collect::<Vec<_>>(),
        ["Label", "amount"]
    );
    assert_eq!(
        leaf_fields[0].data_type().max_character_length(&database),
        Some(7)
    );
    let ColumnCollation::Named(collation) = leaf_fields[0].collation() else {
        panic!("the field declares a named collation");
    };
    assert_eq!(
        collation.name(),
        &TargetName::new("C", true).with_schema("pg_catalog", true)
    );
    assert_eq!(collation.postgres_deterministic(), Some(true));
    assert_eq!(
        leaf_fields[1].data_type().numeric_precision(&database),
        Some(8)
    );
    assert_eq!(leaf_fields[1].data_type().numeric_scale(&database), Some(2));
    assert_eq!(
        leaf_fields[1].data_type().is_unsigned(&database),
        Some(false)
    );
    assert_eq!(fields[1].data_type().array_element(&database), Some(leaf));
    let row_fields: Vec<_> = fields[2]
        .data_type()
        .structured_fields(&database)
        .unwrap()
        .collect();
    assert!(!row_fields[0].is_nullable());
    assert!(row_fields[0].data_type().admits_null(&database));
    assert!(row_fields[1].is_nullable());
    assert_eq!(row_fields[1].collation(), &ColumnCollation::DatabaseDefault);
    let empty = fields[3].data_type();
    assert_eq!(empty.structure_kind(&database), Some(StructureKind::Struct));
    assert_eq!(empty.structured_fields(&database).unwrap().count(), 0);
    for (name, unsigned) in [
        ("int4", Some(false)),
        ("float8", Some(false)),
        ("text", None),
        ("leaf_domain", None),
    ] {
        let data_type = declared_type(&mut connection, &cache, name).unwrap();
        assert_eq!(data_type.is_unsigned(&database), unsigned, "{name}");
        assert!(data_type.admits_null(&database), "{name}");
        assert!(data_type.map_entry(&database).is_none(), "{name}");
    }
}

#[tokio::test]
async fn incomplete_or_cyclic_composites_fail_type_construction() {
    let docker = reference_docker("invalid_types").await.unwrap();
    let mut connection: PgConnection = establish_connection(docker.port, "invalid_types").unwrap();
    connection
        .batch_execute("CREATE TYPE broken AS (value text);")
        .unwrap();
    for target in ["0", "'broken'::regtype::oid", "'broken[]'::regtype::oid"] {
        connection.batch_execute(&format!(
            "BEGIN; UPDATE pg_catalog.pg_attribute SET atttypid = {target}
             WHERE attrelid = (SELECT typrelid FROM pg_catalog.pg_type WHERE oid = 'broken'::regtype)
             AND attnum = 1;"
        )).unwrap();
        let cache = CatalogCache::load(&mut connection).unwrap();
        assert!(
            declared_type(&mut connection, &cache, "broken").is_none(),
            "{target}"
        );
        connection.batch_execute("ROLLBACK").unwrap();
    }
}
