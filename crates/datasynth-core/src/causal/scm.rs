use std::collections::HashMap;

use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Beta, Distribution, LogNormal, Normal};

use super::graph::{CausalGraph, CausalVarType, CausalVariable};

/// Structural Causal Model for generating data from a causal graph.
pub struct StructuralCausalModel {
    graph: CausalGraph,
}

impl StructuralCausalModel {
    pub fn new(graph: CausalGraph) -> Result<Self, String> {
        graph.validate()?;
        Ok(Self { graph })
    }

    /// Get reference to the underlying graph.
    pub fn graph(&self) -> &CausalGraph {
        &self.graph
    }

    /// Generate samples from the causal model.
    pub fn generate(
        &self,
        n_samples: usize,
        seed: u64,
    ) -> Result<Vec<HashMap<String, f64>>, String> {
        let order = self.graph.topological_order()?;
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut samples = Vec::with_capacity(n_samples);

        for _ in 0..n_samples {
            let mut record: HashMap<String, f64> = HashMap::new();

            for var_name in &order {
                let var = self
                    .graph
                    .get_variable(var_name)
                    .ok_or_else(|| format!("Variable '{}' not found", var_name))?;

                // Sample exogenous noise
                let noise = self.sample_exogenous(var, &mut rng);

                // Compute contribution from parents
                let parent_edges = self.graph.parent_edges(var_name);
                let parent_contribution: f64 = parent_edges
                    .iter()
                    .map(|edge| {
                        let parent_val = record.get(&edge.from).copied().unwrap_or(0.0);
                        edge.mechanism.apply(parent_val) * edge.strength
                    })
                    .sum();

                // Combine: noise + parent contributions
                let value = match var.var_type {
                    CausalVarType::Binary => {
                        let prob = (noise + parent_contribution).clamp(0.0, 1.0);
                        if rng.random::<f64>() < prob {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    CausalVarType::Count => (noise + parent_contribution).max(0.0).round(),
                    _ => noise + parent_contribution,
                };

                record.insert(var_name.clone(), value);
            }

            samples.push(record);
        }

        Ok(samples)
    }

    /// Sample exogenous noise for a variable based on its distribution specification.
    fn sample_exogenous(&self, var: &CausalVariable, rng: &mut ChaCha8Rng) -> f64 {
        let dist = var.distribution.as_deref().unwrap_or("normal");
        match dist {
            "lognormal" => {
                let mu = var.params.get("mu").copied().unwrap_or(0.0);
                let sigma = var.params.get("sigma").copied().unwrap_or(1.0);
                if let Ok(d) = LogNormal::new(mu, sigma) {
                    d.sample(rng)
                } else {
                    0.0
                }
            }
            "beta" => {
                let alpha = var.params.get("alpha").copied().unwrap_or(2.0);
                let beta_param = var.params.get("beta_param").copied().unwrap_or(2.0);
                if let Ok(d) = Beta::new(alpha, beta_param) {
                    d.sample(rng)
                } else {
                    // Fallback to mean if parameters are invalid
                    let sum = alpha + beta_param;
                    if sum > 0.0 {
                        alpha / sum
                    } else {
                        0.5
                    }
                }
            }
            "uniform" => {
                let low = var.params.get("low").copied().unwrap_or(0.0);
                let high = var.params.get("high").copied().unwrap_or(1.0);
                rng.random::<f64>() * (high - low) + low
            }
            _ => {
                // Default to normal distribution
                let mean = var.params.get("mean").copied().unwrap_or(0.0);
                let std = var.params.get("std").copied().unwrap_or(1.0);
                if let Ok(d) = Normal::new(mean, std) {
                    d.sample(rng)
                } else {
                    mean
                }
            }
        }
    }

    /// Create an intervened SCM where a variable is set to a fixed value.
    /// This implements the do-calculus do(X=x) operation.
    pub fn intervene(&self, variable: &str, value: f64) -> Result<IntervenedScm<'_>, String> {
        // Verify variable exists
        if self.graph.get_variable(variable).is_none() {
            return Err(format!(
                "Variable '{}' not found for intervention",
                variable
            ));
        }
        Ok(IntervenedScm {
            base: self,
            interventions: vec![(variable.to_string(), value)],
        })
    }
}

/// An SCM with active interventions (do-calculus).
pub struct IntervenedScm<'a> {
    base: &'a StructuralCausalModel,
    interventions: Vec<(String, f64)>,
}

impl<'a> IntervenedScm<'a> {
    /// Add another intervention.
    pub fn and_intervene(mut self, variable: &str, value: f64) -> Self {
        self.interventions.push((variable.to_string(), value));
        self
    }

    /// Generate samples under intervention.
    pub fn generate(
        &self,
        n_samples: usize,
        seed: u64,
    ) -> Result<Vec<HashMap<String, f64>>, String> {
        let order = self.base.graph.topological_order()?;
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let intervention_map: HashMap<&str, f64> = self
            .interventions
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect();
        let mut samples = Vec::with_capacity(n_samples);

        for _ in 0..n_samples {
            let mut record: HashMap<String, f64> = HashMap::new();

            for var_name in &order {
                // If this variable is intervened on, use fixed value
                if let Some(&fixed_val) = intervention_map.get(var_name.as_str()) {
                    record.insert(var_name.clone(), fixed_val);
                    continue;
                }

                let var = self
                    .base
                    .graph
                    .get_variable(var_name)
                    .ok_or_else(|| format!("Variable '{}' not found", var_name))?;

                let noise = self.base.sample_exogenous(var, &mut rng);
                let parent_edges = self.base.graph.parent_edges(var_name);
                let parent_contribution: f64 = parent_edges
                    .iter()
                    .map(|edge| {
                        let parent_val = record.get(&edge.from).copied().unwrap_or(0.0);
                        edge.mechanism.apply(parent_val) * edge.strength
                    })
                    .sum();

                let value = match var.var_type {
                    CausalVarType::Binary => {
                        let prob = (noise + parent_contribution).clamp(0.0, 1.0);
                        if rng.random::<f64>() < prob {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    CausalVarType::Count => (noise + parent_contribution).max(0.0).round(),
                    _ => noise + parent_contribution,
                };

                record.insert(var_name.clone(), value);
            }

            samples.push(record);
        }

        Ok(samples)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::super::graph::CausalGraph;
    use super::*;

    #[test]
    fn test_scm_generates_correct_count() {
        let graph = CausalGraph::fraud_detection_template();
        let scm = StructuralCausalModel::new(graph).unwrap();
        let samples = scm.generate(100, 42).unwrap();
        assert_eq!(samples.len(), 100);
    }

    #[test]
    fn test_scm_deterministic() {
        let graph = CausalGraph::fraud_detection_template();
        let scm = StructuralCausalModel::new(graph).unwrap();
        let s1 = scm.generate(50, 42).unwrap();
        let s2 = scm.generate(50, 42).unwrap();
        for (a, b) in s1.iter().zip(s2.iter()) {
            assert_eq!(a.get("transaction_amount"), b.get("transaction_amount"));
        }
    }

    #[test]
    fn test_scm_all_variables_present() {
        let graph = CausalGraph::fraud_detection_template();
        let var_names: Vec<String> = graph.variables.iter().map(|v| v.name.clone()).collect();
        let scm = StructuralCausalModel::new(graph).unwrap();
        let samples = scm.generate(10, 42).unwrap();
        for sample in &samples {
            for name in &var_names {
                assert!(
                    sample.contains_key(name),
                    "Sample missing variable '{}'",
                    name
                );
            }
        }
    }

    #[test]
    fn test_scm_is_fraud_binary() {
        let graph = CausalGraph::fraud_detection_template();
        let scm = StructuralCausalModel::new(graph).unwrap();
        let samples = scm.generate(100, 42).unwrap();
        for sample in &samples {
            let val = sample.get("is_fraud").copied().unwrap_or(-1.0);
            assert!(
                val == 0.0 || val == 1.0,
                "is_fraud should be binary, got {}",
                val
            );
        }
    }

    #[test]
    fn test_intervention_sets_value() {
        let graph = CausalGraph::fraud_detection_template();
        let scm = StructuralCausalModel::new(graph).unwrap();
        let intervened = scm.intervene("transaction_amount", 10000.0).unwrap();
        let samples = intervened.generate(50, 42).unwrap();
        for sample in &samples {
            assert_eq!(sample.get("transaction_amount").copied(), Some(10000.0));
        }
    }

    #[test]
    fn test_intervention_affects_downstream() {
        let graph = CausalGraph::fraud_detection_template();
        let scm = StructuralCausalModel::new(graph).unwrap();

        // Generate with very high transaction amount - should increase fraud probability
        let high_intervened = scm.intervene("transaction_amount", 100000.0).unwrap();
        let high_samples = high_intervened.generate(200, 42).unwrap();
        let high_fraud_rate: f64 = high_samples
            .iter()
            .map(|s| s.get("is_fraud").copied().unwrap_or(0.0))
            .sum::<f64>()
            / 200.0;

        // Generate with very low transaction amount
        let low_intervened = scm.intervene("transaction_amount", 1.0).unwrap();
        let low_samples = low_intervened.generate(200, 42).unwrap();
        let low_fraud_rate: f64 = low_samples
            .iter()
            .map(|s| s.get("is_fraud").copied().unwrap_or(0.0))
            .sum::<f64>()
            / 200.0;

        // High amount should generally lead to higher fraud rate
        assert!(
            high_fraud_rate >= low_fraud_rate,
            "High transaction amount ({}) should increase fraud rate ({} vs {})",
            100000.0,
            high_fraud_rate,
            low_fraud_rate
        );
    }

    #[test]
    fn test_intervention_unknown_variable() {
        let graph = CausalGraph::fraud_detection_template();
        let scm = StructuralCausalModel::new(graph).unwrap();
        assert!(scm.intervene("nonexistent", 0.0).is_err());
    }

    #[test]
    fn test_cyclic_graph_rejected_by_scm() {
        use super::super::graph::{CausalEdge, CausalMechanism, CausalVarType, CausalVariable};
        let mut graph = CausalGraph::new();
        graph.add_variable(CausalVariable::new("a", CausalVarType::Continuous));
        graph.add_variable(CausalVariable::new("b", CausalVarType::Continuous));
        graph.add_edge(CausalEdge {
            from: "a".into(),
            to: "b".into(),
            mechanism: CausalMechanism::Linear { coefficient: 1.0 },
            strength: 1.0,
        });
        graph.add_edge(CausalEdge {
            from: "b".into(),
            to: "a".into(),
            mechanism: CausalMechanism::Linear { coefficient: 1.0 },
            strength: 1.0,
        });
        assert!(StructuralCausalModel::new(graph).is_err());
    }
}
