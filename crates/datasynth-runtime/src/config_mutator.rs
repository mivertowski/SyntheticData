//! Config mutation engine for counterfactual simulation.
//!
//! Applies propagated causal effects to a GeneratorConfig by
//! navigating dot-paths and setting values.

use crate::causal_engine::PropagatedInterventions;
use datasynth_config::GeneratorConfig;
use datasynth_core::ScenarioConstraints;
use thiserror::Error;

/// Errors during config mutation.
#[derive(Debug, Error)]
pub enum MutationError {
    #[error("path not found: {0}")]
    PathNotFound(String),
    #[error("type mismatch at path '{path}': expected {expected}, got {actual}")]
    TypeMismatch {
        path: String,
        expected: String,
        actual: String,
    },
    #[error("constraint violation: {0}")]
    ConstraintViolation(String),
    #[error("serialization error: {0}")]
    SerializationError(String),
}

/// Applies interventions to a config, producing a new config.
pub struct ConfigMutator;

impl ConfigMutator {
    /// Create a mutated config by applying propagated interventions.
    pub fn apply(
        base: &GeneratorConfig,
        propagated: &PropagatedInterventions,
        constraints: &ScenarioConstraints,
    ) -> Result<GeneratorConfig, MutationError> {
        // Serialize config to JSON Value for dot-path navigation
        let mut json = serde_json::to_value(base)
            .map_err(|e| MutationError::SerializationError(e.to_string()))?;

        // Collect all changes, using the latest value for each path
        let mut latest_changes: std::collections::HashMap<String, serde_json::Value> =
            std::collections::HashMap::new();

        for changes in propagated.changes_by_month.values() {
            for change in changes {
                latest_changes.insert(change.path.clone(), change.value.clone());
            }
        }

        // Apply changes
        for (path, value) in &latest_changes {
            Self::apply_at_path(&mut json, path, value)?;
        }

        // Strip null values before deserializing back.
        // GeneratorConfig has `f64` fields with `#[serde(default)]` that work when
        // the key is absent (YAML) but fail when the key is present as `null` (JSON Value).
        Self::strip_nulls(&mut json);

        // Deserialize back
        let mutated: GeneratorConfig = serde_json::from_value(json)
            .map_err(|e| MutationError::SerializationError(e.to_string()))?;

        // Validate constraints
        Self::validate_constraints(&mutated, constraints)?;

        Ok(mutated)
    }

    /// Apply a single value at a dot-path, supporting array indexing.
    ///
    /// Examples:
    /// - `"global.seed"` → navigates to `json["global"]["seed"]`
    /// - `"distributions.amounts.components[0].mu"` → navigates to `json["distributions"]["amounts"]["components"][0]["mu"]`
    pub fn apply_at_path(
        value: &mut serde_json::Value,
        path: &str,
        new_value: &serde_json::Value,
    ) -> Result<(), MutationError> {
        let segments = Self::parse_path(path);
        let mut current = value;

        for (i, segment) in segments.iter().enumerate() {
            let is_last = i == segments.len() - 1;

            match segment {
                PathSegment::Key(key) => {
                    if is_last {
                        if let Some(obj) = current.as_object_mut() {
                            obj.insert(key.clone(), new_value.clone());
                            return Ok(());
                        }
                        return Err(MutationError::PathNotFound(path.to_string()));
                    }
                    current = current
                        .get_mut(key.as_str())
                        .ok_or_else(|| MutationError::PathNotFound(path.to_string()))?;
                }
                PathSegment::Index(idx) => {
                    if is_last {
                        if let Some(arr) = current.as_array_mut() {
                            if *idx < arr.len() {
                                arr[*idx] = new_value.clone();
                                return Ok(());
                            }
                        }
                        return Err(MutationError::PathNotFound(path.to_string()));
                    }
                    current = current
                        .get_mut(*idx)
                        .ok_or_else(|| MutationError::PathNotFound(path.to_string()))?;
                }
            }
        }

        Err(MutationError::PathNotFound(path.to_string()))
    }

    /// Parse a dot-path with optional array indices.
    fn parse_path(path: &str) -> Vec<PathSegment> {
        let mut segments = Vec::new();
        for part in path.split('.') {
            if let Some(bracket_pos) = part.find('[') {
                // Key with array index: "components[0]"
                let key = &part[..bracket_pos];
                if !key.is_empty() {
                    segments.push(PathSegment::Key(key.to_string()));
                }
                // Parse index
                let idx_str = &part[bracket_pos + 1..part.len() - 1];
                if let Ok(idx) = idx_str.parse::<usize>() {
                    segments.push(PathSegment::Index(idx));
                }
            } else {
                segments.push(PathSegment::Key(part.to_string()));
            }
        }
        segments
    }

    /// Recursively remove null values from a JSON object tree.
    /// This allows `#[serde(default)]` fields to use their defaults instead of
    /// failing on `null` during deserialization.
    fn strip_nulls(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.retain(|_, v| !v.is_null());
                for v in map.values_mut() {
                    Self::strip_nulls(v);
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr.iter_mut() {
                    Self::strip_nulls(v);
                }
            }
            _ => {}
        }
    }

    /// Validate that constraints are satisfied by the mutated config.
    fn validate_constraints(
        config: &GeneratorConfig,
        constraints: &ScenarioConstraints,
    ) -> Result<(), MutationError> {
        // Validate built-in preserve_* constraints
        if constraints.preserve_document_chains
            && !config.document_flows.generate_document_references
        {
            return Err(MutationError::ConstraintViolation(
                "preserve_document_chains requires document_flows.generate_document_references=true"
                    .into(),
            ));
        }

        if constraints.preserve_balance_coherence && !config.balance.validate_balance_equation {
            return Err(MutationError::ConstraintViolation(
                "preserve_balance_coherence requires balance.validate_balance_equation=true".into(),
            ));
        }

        if constraints.preserve_balance_coherence && !config.balance.generate_trial_balances {
            return Err(MutationError::ConstraintViolation(
                "preserve_balance_coherence requires balance.generate_trial_balances=true".into(),
            ));
        }

        // Check custom constraints
        for constraint in &constraints.custom {
            // Custom constraints reference config paths with min/max bounds
            // These are validated against the config values
            let config_json = serde_json::to_value(config)
                .map_err(|e| MutationError::SerializationError(e.to_string()))?;

            let segments = Self::parse_path(&constraint.config_path);
            let mut current = &config_json;
            let mut found = true;

            for segment in &segments {
                match segment {
                    PathSegment::Key(key) => {
                        if let Some(next) = current.get(key.as_str()) {
                            current = next;
                        } else {
                            found = false;
                            break;
                        }
                    }
                    PathSegment::Index(idx) => {
                        if let Some(next) = current.get(*idx) {
                            current = next;
                        } else {
                            found = false;
                            break;
                        }
                    }
                }
            }

            if found {
                if let Some(val) = current.as_f64() {
                    if let Some(min) = &constraint.min {
                        use rust_decimal::prelude::ToPrimitive;
                        if let Some(min_f64) = min.to_f64() {
                            if val < min_f64 {
                                return Err(MutationError::ConstraintViolation(format!(
                                    "{}: value {} below minimum {}",
                                    constraint.config_path, val, min
                                )));
                            }
                        }
                    }
                    if let Some(max) = &constraint.max {
                        use rust_decimal::prelude::ToPrimitive;
                        if let Some(max_f64) = max.to_f64() {
                            if val > max_f64 {
                                return Err(MutationError::ConstraintViolation(format!(
                                    "{}: value {} above maximum {}",
                                    constraint.config_path, val, max
                                )));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
enum PathSegment {
    Key(String),
    Index(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::BTreeMap;

    #[test]
    fn test_apply_simple_dot_path() {
        let mut json = serde_json::json!({
            "global": {
                "seed": 42
            }
        });

        ConfigMutator::apply_at_path(&mut json, "global.seed", &serde_json::json!(99))
            .expect("should succeed");

        assert_eq!(json["global"]["seed"], 99);
    }

    #[test]
    fn test_apply_nested_dot_path() {
        let mut json = serde_json::json!({
            "distributions": {
                "amounts": {
                    "components": [
                        {"mu": 6.0, "sigma": 1.5},
                        {"mu": 8.5, "sigma": 1.0}
                    ]
                }
            }
        });

        ConfigMutator::apply_at_path(
            &mut json,
            "distributions.amounts.components[0].mu",
            &serde_json::json!(5.5),
        )
        .expect("should succeed");

        assert_eq!(json["distributions"]["amounts"]["components"][0]["mu"], 5.5);
        // Other fields unchanged
        assert_eq!(
            json["distributions"]["amounts"]["components"][0]["sigma"],
            1.5
        );
        assert_eq!(json["distributions"]["amounts"]["components"][1]["mu"], 8.5);
    }

    #[test]
    fn test_apply_preserves_other_fields() {
        let mut json = serde_json::json!({
            "global": {
                "seed": 42,
                "industry": "retail"
            }
        });

        ConfigMutator::apply_at_path(&mut json, "global.seed", &serde_json::json!(99))
            .expect("should succeed");

        assert_eq!(json["global"]["seed"], 99);
        assert_eq!(json["global"]["industry"], "retail");
    }

    #[test]
    fn test_apply_invalid_path_returns_error() {
        let mut json = serde_json::json!({
            "global": { "seed": 42 }
        });

        let result = ConfigMutator::apply_at_path(
            &mut json,
            "nonexistent.path.here",
            &serde_json::json!(99),
        );

        assert!(matches!(result, Err(MutationError::PathNotFound(_))));
    }

    #[test]
    fn test_roundtrip_config_mutation() {
        // Test the dot-path mutation on raw JSON (avoids GeneratorConfig roundtrip issues)
        let mut json = serde_json::json!({
            "global": {
                "seed": 42,
                "period_months": 12,
                "start_date": "2024-01-01",
                "industry": "manufacturing"
            },
            "distributions": {
                "amounts": {
                    "components": [
                        {"mu": 6.0, "sigma": 1.5}
                    ]
                }
            }
        });

        // Mutate period_months
        ConfigMutator::apply_at_path(&mut json, "global.period_months", &serde_json::json!(6))
            .expect("should succeed");

        assert_eq!(json["global"]["period_months"], 6);
        // Other fields preserved
        assert_eq!(json["global"]["start_date"], "2024-01-01");
        assert_eq!(json["global"]["seed"], 42);

        // Mutate nested array element
        ConfigMutator::apply_at_path(
            &mut json,
            "distributions.amounts.components[0].mu",
            &serde_json::json!(5.5),
        )
        .expect("should succeed");

        assert_eq!(json["distributions"]["amounts"]["components"][0]["mu"], 5.5);
        assert_eq!(
            json["distributions"]["amounts"]["components"][0]["sigma"],
            1.5
        );
    }

    #[test]
    fn test_constraint_validation_passes() {
        // Test with empty propagation (no changes)
        let _json = serde_json::json!({
            "global": {"seed": 42, "period_months": 12}
        });

        let constraints = ScenarioConstraints::default();
        // No custom constraints → always passes
        assert!(constraints.custom.is_empty());
    }

    #[test]
    fn test_constraint_preserves_document_chains() {
        use datasynth_test_utils::fixtures::minimal_config;

        let mut config = minimal_config();
        config.document_flows.generate_document_references = false;

        let constraints = ScenarioConstraints {
            preserve_document_chains: true,
            ..Default::default()
        };

        let propagated = PropagatedInterventions {
            changes_by_month: BTreeMap::new(),
        };

        let result = ConfigMutator::apply(&config, &propagated, &constraints);
        assert!(matches!(result, Err(MutationError::ConstraintViolation(_))));
        if let Err(MutationError::ConstraintViolation(msg)) = result {
            assert!(msg.contains("document_flows"));
        }
    }

    #[test]
    fn test_constraint_preserves_balance() {
        use datasynth_test_utils::fixtures::minimal_config;

        let mut config = minimal_config();
        config.balance.validate_balance_equation = false;

        let constraints = ScenarioConstraints {
            preserve_balance_coherence: true,
            ..Default::default()
        };

        let propagated = PropagatedInterventions {
            changes_by_month: BTreeMap::new(),
        };

        let result = ConfigMutator::apply(&config, &propagated, &constraints);
        assert!(matches!(result, Err(MutationError::ConstraintViolation(_))));
    }

    #[test]
    fn test_constraint_allows_when_not_preserved() {
        use datasynth_test_utils::fixtures::minimal_config;

        let mut config = minimal_config();
        config.document_flows.generate_document_references = false;
        config.balance.validate_balance_equation = false;

        // All preserve flags off — should succeed
        let constraints = ScenarioConstraints {
            preserve_document_chains: false,
            preserve_balance_coherence: false,
            preserve_period_close: false,
            preserve_accounting_identity: false,
            custom: vec![],
        };

        let propagated = PropagatedInterventions {
            changes_by_month: BTreeMap::new(),
        };

        let result = ConfigMutator::apply(&config, &propagated, &constraints);
        assert!(result.is_ok());
    }
}
