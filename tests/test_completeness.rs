//! Submodule testing that all relevant postgres tables exist in the schema.
#![allow(clippy::too_many_lines)]

mod test_utils;

use std::collections::HashSet;

use diesel::{PgConnection, RunQueryDsl};

use test_utils::{
    docker_helpers::get_postgres_version_tag, establish_connection, get_active_postgres_features,
    is_enabled_for_current_features, model_parser::parse_model_file, reference_docker,
    schema_parser::parse_schema_file,
};

#[tokio::test]
/// Test retrieval of extensions from a column
async fn test_schema_completeness() {
    use diesel::QueryableByName;
    use diesel::sql_types::Text;

    #[derive(QueryableByName, Debug)]
    struct TableInfo {
        #[diesel(sql_type = Text)]
        table_schema: String,
        #[diesel(sql_type = Text)]
        table_name: String,
    }

    #[derive(QueryableByName, Debug)]
    struct ColumnInfo {
        #[diesel(sql_type = Text)]
        column_name: String,
        #[diesel(sql_type = Text)]
        udt_name: String,
    }

    /// Types PostgreSQL has no binary output function for.
    const UNREADABLE_UDT_NAMES: [&str; 2] = ["aclitem", "_aclitem"];

    let database_name = "test_schema_completeness";
    let docker = reference_docker(database_name)
        .await
        .expect("Failed to start docker container");
    let mut conn: PgConnection = establish_connection(docker.port, database_name)
        .expect("Failed to establish connection to database");

    let source_roots = source_roots();

    // Get all tables from information_schema
    let tables: Vec<TableInfo> = diesel::sql_query(
        "SELECT table_schema, table_name FROM information_schema.tables 
         WHERE table_schema IN ('pg_catalog', 'information_schema') 
         AND table_type IN ('BASE TABLE', 'VIEW')
         ORDER BY table_schema, table_name",
    )
    .load(&mut conn)
    .expect("Failed to load tables");

    for table in tables {
        let table_schema = &table.table_schema;
        let table_name = &table.table_name;

        if table_name.starts_with('_') {
            continue;
        }

        let Some(source_root) = source_roots.iter().find(|root| {
            root.join("schema")
                .join(table_schema)
                .join(format!("{table_name}.rs"))
                .exists()
        }) else {
            continue;
        };
        let expected_schema_path = source_root
            .join("schema")
            .join(table_schema)
            .join(format!("{table_name}.rs"));

        let model_filename = match table_name.as_str() {
            "pg_policy" => "pg_policy_table",
            _ => table_name,
        };

        let expected_model_path = source_root
            .join("models")
            .join(table_schema)
            .join(format!("{model_filename}.rs"));

        let schema_result = parse_schema_file(&expected_schema_path).unwrap_or_else(|e| {
            panic!(
                "Failed to parse schema file {}: {e}",
                expected_schema_path.display(),
            )
        });

        let enabled_schema_columns: HashSet<String> = schema_result
            .columns
            .iter()
            .filter(|col| is_enabled_for_current_features(&col.feature_gate))
            .map(|col| col.name.clone())
            .collect();

        let enabled_model_fields: HashSet<String> = if expected_model_path.exists() {
            match parse_model_file(&expected_model_path) {
                Ok(model_result) => model_result
                    .fields
                    .iter()
                    .filter(|field| is_enabled_for_current_features(&field.feature_gate))
                    .map(|field| field.name.clone())
                    .collect(),
                Err(_) => HashSet::new(),
            }
        } else {
            HashSet::new()
        };

        let query = format!(
            "SELECT column_name, udt_name FROM information_schema.columns WHERE table_schema = '{table_schema}' AND table_name = '{table_name}' ORDER BY ordinal_position"
        );
        let columns: Vec<ColumnInfo> =
            diesel::sql_query(&query)
                .load(&mut conn)
                .unwrap_or_else(|e| {
                    panic!("Failed to load columns for {table_schema}.{table_name}: {e}")
                });

        let db_columns: HashSet<String> = columns
            .into_iter()
            .filter(|col| !UNREADABLE_UDT_NAMES.contains(&col.udt_name.as_str()))
            .map(|col| col.column_name)
            .collect();

        for db_col in &db_columns {
            let schema_col_candidates =
                [db_col.clone(), format!("__{db_col}"), format!("r#{db_col}")];
            let found = schema_col_candidates
                .iter()
                .any(|c| enabled_schema_columns.contains(c));

            assert!(
                found,
                "Database column `{table_schema}.{table_name}.{db_col}` exists in postgres {} but is not enabled in schema file.\n\
                 Active features: {:?}\n\
                 Enabled schema columns: {enabled_schema_columns:?}",
                get_postgres_version_tag(),
                get_active_postgres_features(),
            );
        }

        for schema_col in &enabled_schema_columns {
            let db_col_candidates: Vec<String> =
                if let Some(stripped) = schema_col.strip_prefix("__") {
                    vec![schema_col.clone(), stripped.to_string()]
                } else if let Some(stripped) = schema_col.strip_prefix("r#") {
                    vec![schema_col.clone(), stripped.to_string()]
                } else {
                    vec![schema_col.clone()]
                };

            let found = db_col_candidates.iter().any(|c| db_columns.contains(c));

            assert!(
                found,
                "Schema column `{table_schema}.{table_name}.{schema_col}` is enabled for current features but doesn't exist in database.\n\
                 This might indicate an incorrect feature gate.\n\
                 Active features: {:?}\n\
                 Database columns: {db_columns:?}",
                get_active_postgres_features(),
            );
        }

        if !enabled_model_fields.is_empty() {
            for model_field in &enabled_model_fields {
                assert!(
                    enabled_schema_columns.contains(model_field),
                    "Model field `{table_schema}.{table_name}.{model_field}` is enabled but not found in enabled schema columns.\n\
                     This indicates a mismatch between model and schema feature gates.\n\
                     Active features: {:?}\n\
                     Enabled schema columns: {enabled_schema_columns:?}",
                    get_active_postgres_features(),
                );
            }

            for schema_col in &enabled_schema_columns {
                assert!(
                    enabled_model_fields.contains(schema_col)
                        || enabled_model_fields.contains(&format!("__{schema_col}")),
                    "Schema column `{table_schema}.{table_name}.{schema_col}` is enabled but has no corresponding model field.\n\
                     This indicates a mismatch between model and schema.\n\
                     Active features: {:?}\n\
                     Enabled model fields: {enabled_model_fields:?}",
                    get_active_postgres_features(),
                );
            }
        }
    }
}

/// Test that `test_load_all.rs` has test blocks for all tables in the schema.
#[test]
fn test_load_all_coverage() {
    use std::collections::HashMap;

    let crate_root_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    let test_file_path = crate_root_path.join("tests").join("test_load_all.rs");
    let test_content =
        std::fs::read_to_string(&test_file_path).expect("Failed to read test_load_all.rs");

    let mut tested_models: HashMap<String, Option<String>> = HashMap::new();
    let mut current_feature_gate: Option<String> = None;

    for line in test_content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("#[cfg(feature = \"") {
            if let Some(start) = trimmed.find('"')
                && let Some(end) = trimmed.rfind('"')
                && start < end
            {
                current_feature_gate = Some(trimmed[start + 1..end].to_string());
            }
            continue;
        }

        if trimmed == "}" {
            current_feature_gate = None;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("use pg_diesel::models::")
            && let Some(end) = rest.find(';')
        {
            let model_name = rest[..end].trim().to_string();
            tested_models.insert(model_name, current_feature_gate.clone());
        }
    }

    let schemas = ["information_schema", "pg_catalog", "public"];
    let mut missing_tests: Vec<String> = Vec::new();
    let source_roots = source_roots();

    for (source_root, schema) in source_roots
        .iter()
        .flat_map(|root| schemas.iter().map(move |schema| (root, *schema)))
    {
        let schema_dir = source_root.join("schema").join(schema);
        let model_dir = source_root.join("models").join(schema);

        if !schema_dir.exists() {
            continue;
        }

        for entry in std::fs::read_dir(&schema_dir).expect("Failed to read schema directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();

            if path.extension().is_some_and(|e| e == "rs") {
                let file_name = path.file_stem().unwrap().to_str().unwrap();

                if file_name == "mod" {
                    continue;
                }

                let schema_result = parse_schema_file(&path);

                let has_feature_gated_columns = if let Ok(ref result) = schema_result {
                    result
                        .columns
                        .iter()
                        .any(|col| !matches!(col.feature_gate, test_utils::FeatureGate::Always))
                } else {
                    false
                };

                let model_file_candidates = [
                    file_name.to_string(),
                    file_name.trim_end_matches('s').to_string(),
                    file_name.trim_end_matches("es").to_string(),
                    file_name.replace("_constraints", "_constraint"),
                    file_name.replace("_tables", "_table"),
                    file_name.replace("_columns", "_column"),
                    file_name.replace("_indexes", "_index"),
                    file_name.replace("_privileges", "_privilege"),
                    file_name.replace("_grants", "_grant"),
                ];

                let mut actual_model_name: Option<String> = None;
                let mut model_file_exists = false;
                for candidate in &model_file_candidates {
                    let model_path = model_dir.join(format!("{candidate}.rs"));
                    if model_path.exists() {
                        model_file_exists = true;
                        if let Ok(model_result) = parse_model_file(&model_path)
                            && model_result.struct_name.is_some()
                        {
                            actual_model_name = model_result.struct_name;
                            break;
                        }
                    }
                }

                if !model_file_exists {
                    continue;
                }

                let model_to_check = actual_model_name.unwrap_or_else(|| to_pascal_case(file_name));

                if !tested_models.contains_key(&model_to_check) {
                    let singular_model = model_to_check.trim_end_matches('s').to_string();
                    let without_es = if model_to_check.ends_with("es") {
                        model_to_check[..model_to_check.len() - 2].to_string()
                    } else {
                        model_to_check.clone()
                    };

                    let variations = [
                        model_to_check.clone(),
                        format!("Pg{model_to_check}"),
                        model_to_check.replace("Pg", "PG"),
                        to_pascal_case(file_name),
                        singular_model,
                        without_es,
                        model_to_check.replace("Policies", "Policy"),
                        model_to_check.replace("Indexes", "Index"),
                        model_to_check.replace("Privileges", "Privilege"),
                        model_to_check.replace("Tables", "Table"),
                        model_to_check.replace("Columns", "Column"),
                        model_to_check.replace("Grants", "Grant"),
                        model_to_check.replace("Data", "Datum"),
                    ];

                    let found = variations.iter().any(|v| tested_models.contains_key(v));

                    if !found {
                        missing_tests.push(format!(
                            "{schema}.{file_name} (model: {model_to_check}{})",
                            if has_feature_gated_columns {
                                " - has feature-gated columns"
                            } else {
                                ""
                            }
                        ));
                    }
                }
            }
        }
    }

    assert!(
        missing_tests.is_empty(),
        "The following tables are missing tests in test_load_all.rs:\n{}",
        missing_tests.join("\n")
    );
}

/// The source directories of the facade and of `pg_diesel_core`, between
/// them holding every schema and model file.
fn source_roots() -> [std::path::PathBuf; 2] {
    let crate_root_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    [
        crate_root_path.join("src"),
        crate_root_path.join("crates/pg_diesel_core/src"),
    ]
}

/// Convert `snake_case` to `PascalCase`.
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}
