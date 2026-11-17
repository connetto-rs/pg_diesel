//! Model file parser for extracting struct fields and their feature gates.

use super::FeatureGate;
use std::path::Path;
use syn::{Field, Fields, Item, ItemStruct};

/// Represents a field in a model struct.
#[derive(Debug, Clone)]
pub struct ModelField {
    pub name: String,
    pub feature_gate: FeatureGate,
}

/// Result of parsing a model file.
#[derive(Debug)]
pub struct ModelParseResult {
    pub struct_name: Option<String>,
    pub fields: Vec<ModelField>,
}

/// Parse a model file and extract struct fields with their feature gates.
pub fn parse_model_file(path: &Path) -> Result<ModelParseResult, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))?;

    let syntax = syn::parse_file(&content).map_err(|e| format!("Failed to parse file: {e}"))?;

    let mut result = ModelParseResult {
        struct_name: None,
        fields: Vec::new(),
    };

    // Collect all struct definitions with Diesel derives
    for item in syntax.items {
        if let Item::Struct(item_struct) = item {
            // Check if this struct has Queryable or Selectable derive
            if has_diesel_derive(&item_struct) {
                if result.struct_name.is_none() {
                    result.struct_name = Some(item_struct.ident.to_string());
                }

                // Get the feature gate for the entire struct
                let struct_feature_gate = get_struct_feature_gate(&item_struct);

                // Extract fields and combine with struct-level feature gate
                let mut fields = extract_fields(&item_struct);
                for field in &mut fields {
                    field.feature_gate =
                        combine_feature_gates(&struct_feature_gate, &field.feature_gate);
                }

                // Merge fields with existing ones
                for field in fields {
                    if let Some(existing) = result.fields.iter_mut().find(|f| f.name == field.name)
                    {
                        // Combine with OR logic: field is available if it's in ANY of the definitions
                        existing.feature_gate =
                            or_feature_gates(&existing.feature_gate, &field.feature_gate);
                    } else {
                        result.fields.push(field);
                    }
                }
            }
        }
    }

    Ok(result)
}

/// Combine two feature gates with AND logic (both must be satisfied).
fn combine_feature_gates(struct_gate: &FeatureGate, field_gate: &FeatureGate) -> FeatureGate {
    match (struct_gate, field_gate) {
        (FeatureGate::Always, gate) | (gate, FeatureGate::Always) => gate.clone(),
        (gate1, gate2) => FeatureGate::All(vec![gate1.clone(), gate2.clone()]),
    }
}

/// Combine two feature gates with OR logic (either can be satisfied).
fn or_feature_gates(gate1: &FeatureGate, gate2: &FeatureGate) -> FeatureGate {
    match (gate1, gate2) {
        (FeatureGate::Always, _) | (_, FeatureGate::Always) => FeatureGate::Always,
        (gate1, gate2) => FeatureGate::Any(vec![gate1.clone(), gate2.clone()]),
    }
}

/// Check if a struct has Diesel-related derives (Queryable, Selectable, etc.)
fn has_diesel_derive(item: &ItemStruct) -> bool {
    for attr in &item.attrs {
        if attr.path().is_ident("derive") {
            let tokens = attr
                .meta
                .require_list()
                .ok()
                .map(|list| {
                    list.tokens
                        .clone()
                        .into_iter()
                        .map(|t| t.to_string())
                        .collect::<String>()
                        .replace(' ', "")
                })
                .unwrap_or_default();

            if tokens.contains("Queryable") || tokens.contains("Selectable") {
                return true;
            }
        }
    }
    false
}

/// Extract the feature gate from a field's attributes.
fn get_field_feature_gate(field: &Field) -> FeatureGate {
    for attr in &field.attrs {
        if attr.path().is_ident("cfg")
            && let Some(gate) = FeatureGate::from_cfg_meta(&attr.meta)
        {
            return gate;
        }
    }
    FeatureGate::Always
}

/// Extract the feature gate from a struct's attributes.
fn get_struct_feature_gate(item: &ItemStruct) -> FeatureGate {
    for attr in &item.attrs {
        if attr.path().is_ident("cfg")
            && let Some(gate) = FeatureGate::from_cfg_meta(&attr.meta)
        {
            return gate;
        }
    }
    FeatureGate::Always
}

/// Extract fields from a struct.
fn extract_fields(item: &ItemStruct) -> Vec<ModelField> {
    let mut fields = Vec::new();

    if let Fields::Named(named_fields) = &item.fields {
        for field in &named_fields.named {
            if let Some(ident) = &field.ident {
                let feature_gate = get_field_feature_gate(field);
                // Strip r# prefix from raw identifiers to match schema column names
                let name = ident.to_string();
                let name = name.strip_prefix("r#").unwrap_or(&name).to_string();
                fields.push(ModelField { name, feature_gate });
            }
        }
    }

    fields
}

/// Parse a model file and return only the struct name.
#[allow(dead_code)]
pub fn parse_model_file_name(path: &Path) -> Result<ModelParseResult, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))?;

    let syntax = syn::parse_file(&content).map_err(|e| format!("Failed to parse file: {e}"))?;

    let mut result = ModelParseResult {
        struct_name: None,
        fields: Vec::new(),
    };

    for item in syntax.items {
        if let Item::Struct(item_struct) = item {
            // Check if this struct has Queryable or Selectable derive
            if has_diesel_derive(&item_struct) {
                result.struct_name = Some(item_struct.ident.to_string());
                // We extract fields here too for completeness, but caller may not need them
                let tokens_list = item_struct.attrs.iter().find_map(|attr| {
                    if attr.path().is_ident("derive") {
                        attr.meta.require_list().ok().map(|list| {
                            list.tokens
                                .clone()
                                .into_iter()
                                .map(|t| t.to_string())
                                .collect::<String>()
                                .replace(' ', "")
                        })
                    } else {
                        None
                    }
                });

                if tokens_list
                    .as_ref()
                    .is_some_and(|t| t.contains("Queryable") || t.contains("Selectable"))
                {
                    result.fields = extract_fields(&item_struct);
                }
                break;
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_simple_model() {
        let content = r#"
#[derive(Queryable, Selectable)]
pub struct TestModel {
    pub id: i32,
    pub name: String,
    #[cfg(feature = "postgres-17")]
    pub new_field: String,
}
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let result = parse_model_file(file.path()).unwrap();
        assert_eq!(result.struct_name, Some("TestModel".to_string()));
        assert_eq!(result.fields.len(), 3);
        assert_eq!(result.fields[0].name, "id");
        assert_eq!(result.fields[1].name, "name");
        assert_eq!(result.fields[2].name, "new_field");

        assert!(matches!(result.fields[0].feature_gate, FeatureGate::Always));
        assert!(matches!(result.fields[1].feature_gate, FeatureGate::Always));
        assert!(matches!(
            &result.fields[2].feature_gate,
            FeatureGate::Feature(f) if f == "postgres-17"
        ));
    }
}
