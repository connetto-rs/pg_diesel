//! Schema file parser for extracting column definitions and their feature gates.

use super::FeatureGate;
use std::path::Path;

/// Represents a column definition in a schema file.
#[derive(Debug, Clone)]
pub struct SchemaColumn {
    pub name: String,
    pub feature_gate: FeatureGate,
}

/// Result of parsing a schema file.
#[derive(Debug)]
pub struct SchemaParseResult {
    pub columns: Vec<SchemaColumn>,
}

/// Parse a schema file and extract column definitions with their feature gates.
pub fn parse_schema_file(path: &Path) -> Result<SchemaParseResult, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))?;

    let mut columns: Vec<SchemaColumn> = Vec::new();
    let mut in_table_macro = false;
    let mut in_table_definition = false;
    let mut brace_depth = 0;
    let mut table_feature_gate = FeatureGate::Always;
    let mut column_feature_gate = FeatureGate::Always;
    let mut current_sql_name: Option<String> = None;
    let mut pending_cfg_for_table: Option<FeatureGate> = None;

    let mut pending_attribute: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // A `#[cfg(...)]` attribute, which may span several lines, gates the
        // table macro it precedes or the column it precedes.
        if pending_attribute.is_some() || trimmed.starts_with("#[cfg(") {
            let attribute = pending_attribute.get_or_insert_with(String::new);
            attribute.push_str(trimmed);
            attribute.push(' ');
            if !attribute_is_complete(attribute) {
                continue;
            }
            let gate = pending_attribute
                .take()
                .and_then(|attribute| parse_cfg_attribute(&attribute));
            if in_table_definition {
                if let Some(gate) = gate {
                    column_feature_gate = gate;
                }
            } else if !in_table_macro {
                pending_cfg_for_table = gate;
            }
            continue;
        }

        // Detect start of diesel::table! macro
        if trimmed.starts_with("diesel::table!") {
            in_table_macro = true;
            in_table_definition = false;
            brace_depth = 0;
            table_feature_gate = pending_cfg_for_table.take().unwrap_or(FeatureGate::Always);
            continue;
        }

        if !in_table_macro {
            continue;
        }

        // Skip use statements inside diesel::table! macro (before table definition)
        if !in_table_definition && trimmed.starts_with("use ") {
            continue;
        }

        // Detect start of table definition: schema.table_name (primary_key) {
        if !in_table_definition
            && trimmed.contains('(')
            && trimmed.contains(')')
            && trimmed.contains('{')
        {
            in_table_definition = true;
            brace_depth = 1; // The opening brace of the table definition
            continue;
        }

        if !in_table_definition {
            continue;
        }

        // Track brace depth within table definition
        brace_depth += trimmed.matches('{').count();
        brace_depth = brace_depth.saturating_sub(trimmed.matches('}').count());

        // End of table definition
        if brace_depth == 0 {
            in_table_macro = false;
            in_table_definition = false;
            table_feature_gate = FeatureGate::Always;
            column_feature_gate = FeatureGate::Always;
            current_sql_name = None;
            continue;
        }

        // Check for #[sql_name = "..."] attribute
        if trimmed.starts_with("#[sql_name")
            && let Some(start) = trimmed.find('"')
            && let Some(end) = trimmed.rfind('"')
            && start < end
        {
            current_sql_name = Some(trimmed[start + 1..end].to_string());
            continue;
        }

        // Skip comment-only lines (but they might have cfg attributes above them)
        if trimmed.starts_with("///") || trimmed.starts_with("//") {
            continue;
        }

        // Parse column definition: name -> Type,
        if let Some(mut column) =
            parse_column_line(trimmed, &column_feature_gate, current_sql_name.as_deref())
        {
            // Combine table-level and column-level feature gates
            column.feature_gate = combine_feature_gates(&table_feature_gate, &column.feature_gate);

            // Check if this column already exists from another table definition
            if let Some(existing) = columns.iter_mut().find(|c| c.name == column.name) {
                // Combine with OR logic: column is available if it's in ANY of the definitions
                existing.feature_gate =
                    or_feature_gates(&existing.feature_gate, &column.feature_gate);
            } else {
                columns.push(column);
            }

            column_feature_gate = FeatureGate::Always;
            current_sql_name = None;
        }
    }

    Ok(SchemaParseResult { columns })
}

/// Combine two feature gates with AND logic (both must be satisfied).
fn combine_feature_gates(table_gate: &FeatureGate, column_gate: &FeatureGate) -> FeatureGate {
    match (table_gate, column_gate) {
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

/// Returns whether `attribute`, the text of a `#[cfg(...)]` attribute read so
/// far, is closed.
fn attribute_is_complete(attribute: &str) -> bool {
    attribute.trim_end().ends_with(")]")
        && attribute.matches('(').count() == attribute.matches(')').count()
}

/// Parse a `#[cfg(...)]` attribute and return the corresponding `FeatureGate`.
fn parse_cfg_attribute(attribute: &str) -> Option<FeatureGate> {
    let inner = attribute
        .trim()
        .strip_prefix("#[cfg(")?
        .strip_suffix(")]")?;
    FeatureGate::from_cfg_tokens(inner)
}

/// Parse a single column definition line.
fn parse_column_line(
    line: &str,
    feature_gate: &FeatureGate,
    sql_name: Option<&str>,
) -> Option<SchemaColumn> {
    let trimmed = line.trim();

    // Column definition format: name -> Type, or name -> Type
    if !trimmed.contains("->") {
        return None;
    }

    // Skip table definition lines like: pg_catalog.table_name (primary_key) {
    if trimmed.contains('(') && trimmed.contains('{') {
        return None;
    }

    // Extract column name (before ->)
    let parts: Vec<&str> = trimmed.split("->").collect();
    if parts.is_empty() {
        return None;
    }

    let name_part = parts[0].trim();

    // Handle raw identifiers like r#type
    let column_name = if let Some(stripped) = name_part.strip_prefix("r#") {
        stripped.to_string()
    } else {
        name_part.to_string()
    };

    // Skip if this looks like a table definition
    if column_name.contains('.') || column_name.contains('(') {
        return None;
    }

    // Use sql_name if provided, otherwise use the column name
    let final_name = sql_name.map(String::from).unwrap_or(column_name);

    Some(SchemaColumn {
        name: final_name,
        feature_gate: feature_gate.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_simple_schema() {
        let content = r#"
diesel::table! {
    pg_catalog.test_table (id) {
        id -> Integer,
        name -> Text,
        #[cfg(feature = "postgres-17")]
        new_column -> Text,
    }
}
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let result = parse_schema_file(file.path()).unwrap();
        assert_eq!(result.columns.len(), 3);
        assert_eq!(result.columns[0].name, "id");
        assert_eq!(result.columns[1].name, "name");
        assert_eq!(result.columns[2].name, "new_column");

        assert!(matches!(
            result.columns[0].feature_gate,
            FeatureGate::Always
        ));
        assert!(matches!(
            result.columns[1].feature_gate,
            FeatureGate::Always
        ));
        assert!(matches!(
            &result.columns[2].feature_gate,
            FeatureGate::Feature(f) if f == "postgres-17"
        ));
    }
}
