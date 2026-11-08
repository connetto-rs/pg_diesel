//! Test utilities for parsing schema and model files.

pub mod docker_helpers;
pub mod model_parser;
pub mod schema_parser;

pub use docker_helpers::{establish_connection, reference_docker};

/// Represents a feature gate condition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FeatureGate {
    /// No feature gate - always enabled.
    Always,
    /// Feature must be enabled.
    Feature(String),
    /// The condition must not be met.
    Not(Box<FeatureGate>),
    /// All conditions must be met.
    All(Vec<FeatureGate>),
    /// Any condition must be met.
    Any(Vec<FeatureGate>),
}

impl FeatureGate {
    /// Check if this feature gate is enabled given the active features.
    pub fn is_enabled(&self, active_features: &[&str]) -> bool {
        match self {
            FeatureGate::Always => true,
            FeatureGate::Feature(f) => active_features.contains(&f.as_str()),
            FeatureGate::Not(gate) => !gate.is_enabled(active_features),
            FeatureGate::All(gates) => gates.iter().all(|g| g.is_enabled(active_features)),
            FeatureGate::Any(gates) => gates.iter().any(|g| g.is_enabled(active_features)),
        }
    }

    /// Parse a cfg attribute meta into a `FeatureGate`.
    pub fn from_cfg_meta(meta: &syn::Meta) -> Option<Self> {
        match meta {
            syn::Meta::List(list) if list.path.is_ident("cfg") => list
                .parse_args::<syn::Meta>()
                .ok()
                .and_then(|predicate| Self::from_predicate(&predicate)),
            _ => None,
        }
    }

    /// Parse the predicate inside `cfg(...)`, such as
    /// `all(feature = "a", not(any(feature = "b")))`.
    pub fn from_cfg_tokens(predicate: &str) -> Option<Self> {
        syn::parse_str::<syn::Meta>(predicate)
            .ok()
            .and_then(|predicate| Self::from_predicate(&predicate))
    }

    /// Reads one cfg predicate, recursing into `not`, `all` and `any`.
    fn from_predicate(predicate: &syn::Meta) -> Option<Self> {
        match predicate {
            syn::Meta::NameValue(pair) if pair.path.is_ident("feature") => match &pair.value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(name),
                    ..
                }) => Some(FeatureGate::Feature(name.value())),
                _ => None,
            },
            syn::Meta::List(list) => {
                let inner = list
                    .parse_args_with(
                        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                    )
                    .ok()?
                    .iter()
                    .map(Self::from_predicate)
                    .collect::<Option<Vec<_>>>()?;
                if list.path.is_ident("all") {
                    Some(FeatureGate::All(inner))
                } else if list.path.is_ident("any") {
                    Some(FeatureGate::Any(inner))
                } else if list.path.is_ident("not") {
                    let [gate] = <[FeatureGate; 1]>::try_from(inner).ok()?;
                    Some(FeatureGate::Not(Box::new(gate)))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Get the active postgres features based on compile-time cfg.
#[allow(unused_mut, clippy::vec_init_then_push, dead_code)]
pub fn get_active_postgres_features() -> Vec<&'static str> {
    let mut features = vec![];

    #[cfg(feature = "postgres-14")]
    features.push("postgres-14");

    #[cfg(feature = "postgres-15")]
    features.push("postgres-15");

    #[cfg(feature = "postgres-16")]
    features.push("postgres-16");

    #[cfg(feature = "postgres-17")]
    features.push("postgres-17");

    #[cfg(feature = "postgres-18")]
    features.push("postgres-18");

    #[cfg(feature = "routines")]
    features.push("routines");

    #[cfg(feature = "serde")]
    features.push("serde");

    features
}

/// Checks if a column/field should be enabled given the active features.
#[allow(dead_code)]
pub fn is_enabled_for_current_features(gate: &FeatureGate) -> bool {
    let active = get_active_postgres_features();
    gate.is_enabled(&active)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_gate_always() {
        let gate = FeatureGate::Always;
        assert!(gate.is_enabled(&[]));
        assert!(gate.is_enabled(&["postgres-17"]));
    }

    #[test]
    fn test_feature_gate_feature() {
        let gate = FeatureGate::Feature("postgres-17".to_string());
        assert!(!gate.is_enabled(&[]));
        assert!(gate.is_enabled(&["postgres-17"]));
        assert!(!gate.is_enabled(&["postgres-18"]));
    }

    #[test]
    fn test_feature_gate_not_feature() {
        let gate = FeatureGate::Not(Box::new(FeatureGate::Feature("postgres-18".to_string())));
        assert!(gate.is_enabled(&[]));
        assert!(gate.is_enabled(&["postgres-17"]));
        assert!(!gate.is_enabled(&["postgres-18"]));
    }

    #[test]
    fn test_feature_gate_reads_a_negation_nested_in_all() {
        let gate = FeatureGate::from_cfg_tokens(
            r#"all(feature = "postgres-16", not(any(feature = "postgres-17", feature = "postgres-18")))"#,
        )
        .expect("the predicate parses");
        assert!(gate.is_enabled(&["postgres-16"]));
        assert!(!gate.is_enabled(&["postgres-16", "postgres-18"]));
        assert!(!gate.is_enabled(&["postgres-17"]));
    }
}
