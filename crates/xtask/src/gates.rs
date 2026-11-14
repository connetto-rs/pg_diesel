//! Resolves the `postgres-N` gates of a template for one `PostgreSQL` major
//! version, keeping what that version has and dropping the rest.

use syn::{Expr, ExprLit, Lit, Meta, Token, parse::Parser as _, punctuated::Punctuated};

/// Why a template could not be resolved for a version.
#[derive(Debug, thiserror::Error)]
pub enum GateError {
    /// A `cfg` predicate does not parse.
    #[error("the cfg predicate `{predicate}` does not parse: {source}")]
    Unparsable {
        /// The predicate as the template spells it.
        predicate: String,
        /// The parser's complaint.
        source: syn::Error,
    },
    /// A `cfg` predicate mixes a version feature with something else.
    #[error("the cfg predicate `{predicate}` holds `{part}`, which is no version feature")]
    Unsupported {
        /// The predicate as the template spells it.
        predicate: String,
        /// The part that is no `feature = "postgres-N"`, `not`, `any` or `all`.
        part: String,
    },
    /// An attribute or the item it applies to never closes.
    #[error("the template ends inside the item a cfg attribute at byte {offset} applies to")]
    Unterminated {
        /// The byte offset of the attribute.
        offset: usize,
    },
}

/// Returns `source` as the `PostgreSQL` `major` version sees it, or `None`
/// when an inner `#![cfg(...)]` excludes the whole file.
///
/// Every `#[cfg(...)]` whose predicate names a `postgres-N` feature is
/// evaluated as if `postgres-{major}` were the only feature enabled. A true
/// gate disappears and a false one takes the item it applies to with it,
/// together with the doc comments and attributes written above it.
///
/// # Errors
///
/// Returns a [`GateError`] when a version predicate does not parse, mixes in
/// a feature other than a version, or an item never closes.
pub fn resolve(source: &str, major: u32) -> Result<Option<String>, GateError> {
    let bytes = source.as_bytes();
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    loop {
        let outer = source[cursor..].find("#[cfg(");
        let inner_found = source[cursor..].find("#![cfg(");
        let (offset, inner) = match (outer, inner_found) {
            (None, None) => break,
            (Some(outer), Some(inner)) if inner < outer => (inner, true),
            (Some(outer), _) => (outer, false),
            (None, Some(inner)) => (inner, true),
        };
        let start = cursor + offset;
        let open = start
            + if inner {
                "#![cfg(".len()
            } else {
                "#[cfg(".len()
            };
        let close = matching(bytes, open - 1).ok_or(GateError::Unterminated { offset: start })?;
        let predicate = &source[open..close];
        let attribute_end = close + 2;
        if !predicate.contains("postgres-") {
            output.push_str(&source[cursor..attribute_end]);
            cursor = attribute_end;
            continue;
        }
        let enabled = evaluate(predicate, major)?;
        if inner {
            if !enabled {
                return Ok(None);
            }
            output.push_str(&source[cursor..start]);
            cursor = skip_line_end(bytes, attribute_end);
            continue;
        }
        if enabled {
            let line_start = line_start(bytes, start);
            if source[line_start..start].trim().is_empty() {
                output.push_str(&source[cursor..line_start]);
                cursor = skip_line_end(bytes, attribute_end);
            } else {
                output.push_str(&source[cursor..start]);
                cursor = attribute_end;
            }
            continue;
        }
        let item_start = leading_decorations(source, line_start(bytes, start)).max(cursor);
        let item_end =
            item_end(bytes, attribute_end).ok_or(GateError::Unterminated { offset: start })?;
        output.push_str(&source[cursor..item_start]);
        cursor = skip_line_end(bytes, item_end);
    }
    output.push_str(&source[cursor..]);
    Ok(Some(output))
}

/// Returns whether `predicate`, the inside of a `cfg(...)`, holds when
/// `postgres-{major}` is the only feature enabled.
fn evaluate(predicate: &str, major: u32) -> Result<bool, GateError> {
    let parsed = Punctuated::<Meta, Token![,]>::parse_terminated
        .parse_str(predicate)
        .map_err(|source| GateError::Unparsable {
            predicate: predicate.to_owned(),
            source,
        })?;
    let mut parts = parsed.iter();
    match (parts.next(), parts.next()) {
        (Some(meta), None) => holds(meta, predicate, major),
        _ => Err(GateError::Unsupported {
            predicate: predicate.to_owned(),
            part: predicate.to_owned(),
        }),
    }
}

/// Returns whether one predicate term holds under `postgres-{major}` alone.
fn holds(meta: &Meta, predicate: &str, major: u32) -> Result<bool, GateError> {
    let unsupported = || GateError::Unsupported {
        predicate: predicate.to_owned(),
        part: quote_meta(meta),
    };
    match meta {
        Meta::NameValue(pair) if pair.path.is_ident("feature") => match &pair.value {
            Expr::Lit(ExprLit {
                lit: Lit::Str(feature),
                ..
            }) if feature.value().starts_with("postgres-") => {
                Ok(feature.value() == format!("postgres-{major}"))
            }
            _ => Err(unsupported()),
        },
        Meta::List(list) => {
            let terms = list
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .map_err(|source| GateError::Unparsable {
                    predicate: predicate.to_owned(),
                    source,
                })?;
            let mut values = Vec::with_capacity(terms.len());
            for term in &terms {
                values.push(holds(term, predicate, major)?);
            }
            if list.path.is_ident("not") && values.len() == 1 {
                Ok(!values[0])
            } else if list.path.is_ident("any") {
                Ok(values.contains(&true))
            } else if list.path.is_ident("all") {
                Ok(!values.contains(&false))
            } else {
                Err(unsupported())
            }
        }
        Meta::Path(_) | Meta::NameValue(_) => Err(unsupported()),
    }
}

/// The source spelling of `meta`, for an error message.
fn quote_meta(meta: &Meta) -> String {
    match meta {
        Meta::Path(path) | Meta::List(syn::MetaList { path, .. }) => path_name(path),
        Meta::NameValue(pair) => path_name(&pair.path),
    }
}

/// The `::`-joined segments of `path`.
fn path_name(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// Returns the index of the bracket closing the one at `open`, skipping
/// string literals and line comments.
fn matching(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = open;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => index = skip_string(bytes, index)?,
            b'/' if bytes.get(index + 1) == Some(&b'/') => index = skip_comment(bytes, index),
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

/// Returns the index just past the item that starts after `from`: past the
/// doc comments and attributes in between, then through the first `,` or
/// `;` at the top level, or through the brace closing a top-level block and
/// one `,` or `;` right after it.
fn item_end(bytes: &[u8], from: usize) -> Option<usize> {
    let mut index = skip_decorations(bytes, from)?;
    let mut depth = 0usize;
    let mut angles = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => index = skip_string(bytes, index)?,
            b'/' if bytes.get(index + 1) == Some(&b'/') => index = skip_comment(bytes, index),
            b'-' | b'=' if bytes.get(index + 1) == Some(&b'>') => index += 1,
            b'<' if depth == 0 => angles += 1,
            b'>' if depth == 0 => angles = angles.saturating_sub(1),
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' if depth == 0 => return Some(index),
            b')' | b']' => depth -= 1,
            b'}' => {
                depth -= 1;
                if depth == 0 && angles == 0 {
                    let next = skip_spaces(bytes, index + 1);
                    return Some(match bytes.get(next) {
                        Some(b',' | b';') => next + 1,
                        _ => index + 1,
                    });
                }
            }
            b',' | b';' if depth == 0 && angles == 0 => return Some(index + 1),
            _ => {}
        }
        index += 1;
    }
    None
}

/// Returns the index of the first byte after `from` that is no whitespace,
/// doc comment, comment or attribute.
fn skip_decorations(bytes: &[u8], from: usize) -> Option<usize> {
    let mut index = from;
    loop {
        index = skip_spaces(bytes, index);
        match (bytes.get(index), bytes.get(index + 1)) {
            (Some(b'/'), Some(b'/')) => index = skip_comment(bytes, index) + 1,
            (Some(b'#'), Some(b'[')) => index = matching(bytes, index + 1)? + 1,
            _ => return Some(index),
        }
    }
}

/// Returns where the doc comments and outer attributes directly above the
/// line starting at `line` begin, so a dropped item takes them along.
fn leading_decorations(source: &str, line: usize) -> usize {
    let mut start = line;
    while start > 0 {
        let previous_end = start - 1;
        let previous_start = source[..previous_end]
            .rfind('\n')
            .map_or(0, |index| index + 1);
        let previous = source[previous_start..previous_end].trim_start();
        if previous.starts_with("///") || (previous.starts_with("#[") && previous.ends_with(']')) {
            start = previous_start;
        } else {
            return start;
        }
    }
    start
}

/// Returns the index of the first byte of the line holding `index`.
fn line_start(bytes: &[u8], index: usize) -> usize {
    bytes[..index]
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |newline| newline + 1)
}

/// Returns `index` moved past trailing spaces and one newline, when nothing
/// but spaces follow on its line.
fn skip_line_end(bytes: &[u8], index: usize) -> usize {
    let mut end = index;
    while matches!(bytes.get(end), Some(b' ' | b'\t' | b'\r')) {
        end += 1;
    }
    match bytes.get(end) {
        Some(b'\n') => end + 1,
        None => end,
        Some(_) => index,
    }
}

/// Returns `index` moved past whitespace.
fn skip_spaces(bytes: &[u8], index: usize) -> usize {
    let mut end = index;
    while bytes.get(end).is_some_and(u8::is_ascii_whitespace) {
        end += 1;
    }
    end
}

/// Returns the index of the quote closing the string literal opened at
/// `open`.
fn skip_string(bytes: &[u8], open: usize) -> Option<usize> {
    let mut index = open + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 1,
            b'"' => return Some(index),
            _ => {}
        }
        index += 1;
    }
    None
}

/// Returns the index of the newline ending the line comment at `open`.
fn skip_comment(bytes: &[u8], open: usize) -> usize {
    bytes[open..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(bytes.len(), |offset| open + offset)
}

#[cfg(test)]
mod tests {
    use super::resolve;

    fn at(source: &str, major: u32) -> String {
        resolve(source, major)
            .expect("the template resolves")
            .expect("the file is kept")
    }

    #[test]
    fn a_column_gate_keeps_or_drops_the_column_line() {
        let source = "    x -> Int4,\n    /// Since 15.\n    #[cfg(not(feature = \"postgres-14\"))]\n    y -> Nullable<Array<Text>>,\n    z -> Text,\n";
        assert_eq!(at(source, 14), "    x -> Int4,\n    z -> Text,\n");
        assert_eq!(
            at(source, 15),
            "    x -> Int4,\n    /// Since 15.\n    y -> Nullable<Array<Text>>,\n    z -> Text,\n"
        );
    }

    #[test]
    fn a_dropped_block_takes_its_braces_and_doc_comments() {
        let source = "/// Old.\n#[cfg(any(feature = \"postgres-14\", feature = \"postgres-15\"))]\ndiesel::table! {\n    t (a) { a -> Int4, }\n}\n\n/// New.\n#[cfg(not(any(feature = \"postgres-14\", feature = \"postgres-15\")))]\ndiesel::table! {\n    t (a) { a -> Int4, b -> Bool, }\n}\n";
        assert_eq!(
            at(source, 16),
            "\n/// New.\ndiesel::table! {\n    t (a) { a -> Int4, b -> Bool, }\n}\n"
        );
        assert_eq!(
            at(source, 14),
            "/// Old.\ndiesel::table! {\n    t (a) { a -> Int4, }\n}\n\n"
        );
    }

    #[test]
    fn a_multiline_gate_on_a_generic_field_drops_through_its_comma() {
        let source = "struct S {\n    #[cfg(all(\n        feature = \"postgres-16\",\n        not(any(feature = \"postgres-17\", feature = \"postgres-18\"))\n    ))]\n    pub m: HashMap<u32, Vec<i16>>,\n    pub n: bool,\n}\n";
        assert_eq!(at(source, 17), "struct S {\n    pub n: bool,\n}\n");
        assert_eq!(
            at(source, 16),
            "struct S {\n    pub m: HashMap<u32, Vec<i16>>,\n    pub n: bool,\n}\n"
        );
    }

    #[test]
    fn an_inner_gate_keeps_or_drops_the_whole_file() {
        let source = "//! Doc.\n#![cfg(feature = \"postgres-18\")]\n\nstruct A;\n";
        assert_eq!(resolve(source, 17).expect("the template resolves"), None);
        assert_eq!(at(source, 18), "//! Doc.\n\nstruct A;\n");
    }

    #[test]
    fn a_non_version_gate_stays() {
        let source =
            "#[cfg_attr(feature = \"serde\", derive(Serialize))]\n#[cfg(test)]\nmod tests {}\n";
        assert_eq!(at(source, 14), source);
    }

    #[test]
    fn a_version_gate_mixed_with_another_feature_is_refused() {
        let source = "#[cfg(all(feature = \"serde\", feature = \"postgres-18\"))]\nstruct A;\n";
        assert!(matches!(
            resolve(source, 18),
            Err(super::GateError::Unsupported { .. })
        ));
    }
}
