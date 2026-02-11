//! Counterfactual reasoning via abduction-action-prediction.
//!
//! Given a factual observation, generates "what-if" scenarios by:
//! 1. **Abduction**: infer exogenous noise from the factual record
//! 2. **Action**: set intervention variable to a new value
//! 3. **Prediction**: regenerate downstream variables using inferred noise

use std::collections::{HashMap, HashSet};

use crate::error::SynthError;

use super::graph::CausalGraph;
use super::scm::StructuralCausalModel;

/// A paired factual/counterfactual observation.
#[derive(Debug, Clone)]
pub struct CounterfactualPair {
    /// The original observed record.
    pub factual: HashMap<String, f64>,
    /// The counterfactual record under the intervention.
    pub counterfactual: HashMap<String, f64>,
    /// Variables whose values changed between factual and counterfactual.
    pub changed_variables: Vec<String>,
}

/// Generator for counterfactual scenarios using the abduction-action-prediction framework.
pub struct CounterfactualGenerator {
    scm: StructuralCausalModel,
}

impl CounterfactualGenerator {
    /// Create a new counterfactual generator wrapping a structural causal model.
    pub fn new(scm: StructuralCausalModel) -> Self {
        Self { scm }
    }

    /// Generate a counterfactual for a single factual observation.
    ///
    /// Three-step process:
    /// 1. **Abduction**: infer exogenous noise from the factual record
    /// 2. **Action**: set intervention_var to new_value
    /// 3. **Prediction**: regenerate all downstream variables
    pub fn generate_counterfactual(
        &self,
        factual: &HashMap<String, f64>,
        intervention_var: &str,
        new_value: f64,
        _seed: u64,
    ) -> Result<HashMap<String, f64>, SynthError> {
        let graph = self.scm.graph();

        // Validate intervention variable exists
        if graph.get_variable(intervention_var).is_none() {
            return Err(SynthError::generation(format!(
                "Intervention variable '{}' not found in causal graph",
                intervention_var
            )));
        }

        let order = graph.topological_order().map_err(SynthError::generation)?;

        // Step 1: Abduction - infer exogenous noise for each variable
        let noise = self.abduce_noise(factual, graph, &order)?;

        // Step 2 + 3: Action + Prediction - rebuild values with intervention
        let downstream = self.find_downstream_variables(graph, intervention_var, &order);
        let mut counterfactual = factual.clone();

        // Set the intervention variable
        counterfactual.insert(intervention_var.to_string(), new_value);

        // Re-generate downstream variables in topological order
        for var_name in &order {
            if var_name == intervention_var {
                continue;
            }
            if !downstream.contains(var_name.as_str()) {
                continue;
            }

            let parent_edges = graph.parent_edges(var_name);
            let parent_contribution: f64 = parent_edges
                .iter()
                .map(|edge| {
                    let parent_val = counterfactual.get(&edge.from).copied().unwrap_or(0.0);
                    edge.mechanism.apply(parent_val) * edge.strength
                })
                .sum();

            let var_noise = noise.get(var_name.as_str()).copied().unwrap_or(0.0);
            let value = var_noise + parent_contribution;

            counterfactual.insert(var_name.clone(), value);
        }

        Ok(counterfactual)
    }

    /// Generate counterfactuals for a batch of factual observations.
    pub fn generate_batch_counterfactuals(
        &self,
        factuals: &[HashMap<String, f64>],
        intervention_var: &str,
        new_value: f64,
        seed: u64,
    ) -> Result<Vec<CounterfactualPair>, SynthError> {
        let mut results = Vec::with_capacity(factuals.len());

        for (i, factual) in factuals.iter().enumerate() {
            let counterfactual = self.generate_counterfactual(
                factual,
                intervention_var,
                new_value,
                seed.wrapping_add(i as u64),
            )?;

            let changed_variables = find_changed_variables(factual, &counterfactual);

            results.push(CounterfactualPair {
                factual: factual.clone(),
                counterfactual,
                changed_variables,
            });
        }

        Ok(results)
    }

    /// Abduce exogenous noise from a factual record.
    ///
    /// For each variable, noise = observed_value - sum(mechanism(parent_value) for each parent).
    fn abduce_noise(
        &self,
        factual: &HashMap<String, f64>,
        graph: &CausalGraph,
        order: &[String],
    ) -> Result<HashMap<String, f64>, SynthError> {
        let mut noise = HashMap::new();

        for var_name in order {
            let observed = factual.get(var_name.as_str()).copied().unwrap_or(0.0);

            let parent_edges = graph.parent_edges(var_name);
            let parent_contribution: f64 = parent_edges
                .iter()
                .map(|edge| {
                    let parent_val = factual.get(&edge.from).copied().unwrap_or(0.0);
                    edge.mechanism.apply(parent_val) * edge.strength
                })
                .sum();

            noise.insert(var_name.clone(), observed - parent_contribution);
        }

        Ok(noise)
    }

    /// Find all variables that are downstream of a given variable (in topological order).
    fn find_downstream_variables(
        &self,
        graph: &CausalGraph,
        variable: &str,
        order: &[String],
    ) -> HashSet<String> {
        let mut downstream: HashSet<String> = HashSet::new();
        let variable_owned = variable.to_string();
        downstream.insert(variable_owned.clone());

        // Walk topological order; if any parent is in the downstream set,
        // this variable is also downstream.
        for var_name in order {
            if downstream.contains(var_name.as_str()) {
                continue;
            }
            let parent_edges = graph.parent_edges(var_name);
            let has_downstream_parent = parent_edges
                .iter()
                .any(|edge| downstream.contains(&edge.from));
            if has_downstream_parent {
                downstream.insert(var_name.clone());
            }
        }

        // The intervention variable itself is not considered "downstream"
        // (it was set directly), but we included it for traversal. Remove it.
        downstream.remove(&variable_owned);
        downstream
    }
}

/// Find variables whose values differ between factual and counterfactual.
fn find_changed_variables(
    factual: &HashMap<String, f64>,
    counterfactual: &HashMap<String, f64>,
) -> Vec<String> {
    let mut changed = Vec::new();
    for (key, &cf_val) in counterfactual {
        let f_val = factual.get(key).copied().unwrap_or(0.0);
        if (cf_val - f_val).abs() > 1e-10 {
            changed.push(key.clone());
        }
    }
    changed.sort();
    changed
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::causal::graph::CausalGraph;

    fn build_generator_and_samples() -> (CounterfactualGenerator, Vec<HashMap<String, f64>>) {
        let graph = CausalGraph::fraud_detection_template();
        let scm = StructuralCausalModel::new(graph).unwrap();
        let samples = scm.generate(100, 42).unwrap();
        let generator = CounterfactualGenerator::new(scm);
        (generator, samples)
    }

    #[test]
    fn test_causal_counterfactual_no_change_recovers_original() {
        // If we set the intervention variable to its original value,
        // the counterfactual should approximately recover the factual.
        let (generator, samples) = build_generator_and_samples();

        let factual = &samples[0];
        let original_amount = factual.get("transaction_amount").copied().unwrap_or(0.0);

        let cf = generator
            .generate_counterfactual(factual, "transaction_amount", original_amount, 42)
            .unwrap();

        // All variables should be very close to original
        for (key, &original_val) in factual {
            let cf_val = cf.get(key).copied().unwrap_or(f64::NAN);
            assert!(
                (cf_val - original_val).abs() < 1e-6,
                "Variable '{}' should recover original value: expected {}, got {}",
                key,
                original_val,
                cf_val
            );
        }
    }

    #[test]
    fn test_causal_counterfactual_intervention_changes_downstream() {
        let (generator, samples) = build_generator_and_samples();

        let factual = &samples[0];
        // Set transaction_amount to a very different value
        let cf = generator
            .generate_counterfactual(factual, "transaction_amount", 99999.0, 42)
            .unwrap();

        // fraud_probability should change (it depends on transaction_amount)
        let original_fp = factual.get("fraud_probability").copied().unwrap_or(0.0);
        let cf_fp = cf.get("fraud_probability").copied().unwrap_or(0.0);

        assert!(
            (cf_fp - original_fp).abs() > 1e-6,
            "Counterfactual fraud_probability should differ from original"
        );

        // transaction_amount should be set to the new value
        let cf_amount = cf.get("transaction_amount").copied().unwrap_or(0.0);
        assert!(
            (cf_amount - 99999.0).abs() < 1e-10,
            "Intervention variable should be set to new value"
        );

        // merchant_risk and transaction_frequency should NOT change
        // (they are root variables not downstream of transaction_amount)
        let orig_risk = factual.get("merchant_risk").copied().unwrap_or(0.0);
        let cf_risk = cf.get("merchant_risk").copied().unwrap_or(0.0);
        assert!(
            (cf_risk - orig_risk).abs() < 1e-10,
            "merchant_risk should not change"
        );
    }

    #[test]
    fn test_causal_counterfactual_batch_produces_correct_count() {
        let (generator, samples) = build_generator_and_samples();

        let batch = &samples[..10];
        let pairs = generator
            .generate_batch_counterfactuals(batch, "transaction_amount", 5000.0, 42)
            .unwrap();

        assert_eq!(pairs.len(), 10);
        for pair in &pairs {
            assert!(!pair.factual.is_empty(), "Factual should not be empty");
            assert!(
                !pair.counterfactual.is_empty(),
                "Counterfactual should not be empty"
            );
        }
    }

    #[test]
    fn test_causal_counterfactual_changed_variables_detected() {
        let (generator, samples) = build_generator_and_samples();

        let factual = &samples[0];
        let cf = generator
            .generate_counterfactual(factual, "transaction_amount", 99999.0, 42)
            .unwrap();

        let changed = find_changed_variables(factual, &cf);
        // At minimum, transaction_amount and fraud_probability should be changed
        assert!(
            changed.contains(&"transaction_amount".to_string()),
            "transaction_amount should be in changed list"
        );
        assert!(
            changed.contains(&"fraud_probability".to_string()),
            "fraud_probability should be in changed list"
        );
    }

    #[test]
    fn test_causal_counterfactual_unknown_variable_returns_error() {
        let (generator, samples) = build_generator_and_samples();
        let result = generator.generate_counterfactual(&samples[0], "nonexistent_var", 1.0, 42);
        assert!(result.is_err());
    }

    #[test]
    fn test_causal_counterfactual_batch_changed_variables_populated() {
        let (generator, samples) = build_generator_and_samples();

        let batch = &samples[..5];
        let pairs = generator
            .generate_batch_counterfactuals(batch, "transaction_amount", 99999.0, 42)
            .unwrap();

        for pair in &pairs {
            // The intervention variable should always be in changed (unless it was already 99999.0)
            // and downstream variables should be listed
            assert!(
                !pair.changed_variables.is_empty(),
                "At least some variables should change"
            );
        }
    }
}
