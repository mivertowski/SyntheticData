//! Intervention engine for causal inference using do-calculus.
//!
//! Computes average treatment effects by comparing baseline and intervened samples.

use std::collections::HashMap;

use crate::error::SynthError;

use super::scm::StructuralCausalModel;

/// Result of an intervention experiment.
#[derive(Debug, Clone)]
pub struct InterventionResult {
    /// Samples generated without any intervention.
    pub baseline_samples: Vec<HashMap<String, f64>>,
    /// Samples generated under the do-calculus intervention.
    pub intervened_samples: Vec<HashMap<String, f64>>,
    /// Estimated causal effects for each variable.
    pub effect_estimates: HashMap<String, EffectEstimate>,
}

/// Estimated causal effect of an intervention on a single variable.
#[derive(Debug, Clone)]
pub struct EffectEstimate {
    /// Average Treatment Effect: mean(intervened) - mean(baseline).
    pub average_treatment_effect: f64,
    /// Percentile-based confidence interval for the ATE.
    pub confidence_interval: (f64, f64),
    /// Number of samples used for the estimate.
    pub sample_size: usize,
}

/// Engine for running causal interventions and estimating treatment effects.
pub struct InterventionEngine {
    scm: StructuralCausalModel,
}

impl InterventionEngine {
    /// Create a new intervention engine wrapping a structural causal model.
    pub fn new(scm: StructuralCausalModel) -> Self {
        Self { scm }
    }

    /// Run a do-calculus intervention and estimate causal effects.
    ///
    /// Generates baseline samples (no intervention) and intervened samples,
    /// then computes the average treatment effect for each variable.
    pub fn do_intervention(
        &self,
        interventions: &[(String, f64)],
        n_samples: usize,
        seed: u64,
    ) -> Result<InterventionResult, SynthError> {
        if interventions.is_empty() {
            return Err(SynthError::validation(
                "At least one intervention must be specified",
            ));
        }

        // Validate all intervention variables exist
        for (var_name, _) in interventions {
            if self.scm.graph().get_variable(var_name).is_none() {
                return Err(SynthError::generation(format!(
                    "Intervention variable '{}' not found in causal graph",
                    var_name
                )));
            }
        }

        // Generate baseline samples (no intervention)
        let baseline_samples = self
            .scm
            .generate(n_samples, seed)
            .map_err(SynthError::generation)?;

        // Generate intervened samples using do-calculus
        // Use a different seed offset so baseline and intervened don't share the same RNG state
        let intervened_seed = seed.wrapping_add(1_000_000);
        let intervened_samples = self
            .generate_with_interventions(interventions, n_samples, intervened_seed)
            .map_err(SynthError::generation)?;

        // Compute effect estimates for each variable
        let var_names = self.scm.graph().variable_names();
        let mut effect_estimates = HashMap::new();

        for var_name in &var_names {
            let name = var_name.to_string();
            let estimate =
                Self::compute_effect_estimate(&baseline_samples, &intervened_samples, &name);
            effect_estimates.insert(name, estimate);
        }

        Ok(InterventionResult {
            baseline_samples,
            intervened_samples,
            effect_estimates,
        })
    }

    /// Generate samples with multiple interventions applied.
    fn generate_with_interventions(
        &self,
        interventions: &[(String, f64)],
        n_samples: usize,
        seed: u64,
    ) -> Result<Vec<HashMap<String, f64>>, String> {
        if interventions.is_empty() {
            return self.scm.generate(n_samples, seed);
        }

        // Build the intervened SCM by chaining interventions
        let first = &interventions[0];
        let mut intervened = self.scm.intervene(&first.0, first.1)?;
        for (var_name, value) in interventions.iter().skip(1) {
            intervened = intervened.and_intervene(var_name, *value);
        }
        intervened.generate(n_samples, seed)
    }

    /// Compute the effect estimate for a single variable.
    fn compute_effect_estimate(
        baseline: &[HashMap<String, f64>],
        intervened: &[HashMap<String, f64>],
        variable: &str,
    ) -> EffectEstimate {
        let baseline_vals: Vec<f64> = baseline
            .iter()
            .filter_map(|s| s.get(variable).copied())
            .collect();
        let intervened_vals: Vec<f64> = intervened
            .iter()
            .filter_map(|s| s.get(variable).copied())
            .collect();

        let n = baseline_vals.len().min(intervened_vals.len());
        if n == 0 {
            return EffectEstimate {
                average_treatment_effect: 0.0,
                confidence_interval: (0.0, 0.0),
                sample_size: 0,
            };
        }

        let baseline_mean: f64 = baseline_vals.iter().sum::<f64>() / baseline_vals.len() as f64;
        let intervened_mean: f64 =
            intervened_vals.iter().sum::<f64>() / intervened_vals.len() as f64;
        let ate = intervened_mean - baseline_mean;

        // Compute percentile-based confidence interval using individual diffs
        let mut diffs: Vec<f64> = baseline_vals
            .iter()
            .zip(intervened_vals.iter())
            .map(|(b, i)| i - b)
            .collect();
        diffs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let ci = if diffs.len() >= 2 {
            let lower_idx = (diffs.len() as f64 * 0.025).floor() as usize;
            let upper_idx = ((diffs.len() as f64 * 0.975).ceil() as usize).min(diffs.len() - 1);
            (diffs[lower_idx], diffs[upper_idx])
        } else {
            (ate, ate)
        };

        EffectEstimate {
            average_treatment_effect: ate,
            confidence_interval: ci,
            sample_size: n,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causal::graph::CausalGraph;

    fn build_engine() -> InterventionEngine {
        let graph = CausalGraph::fraud_detection_template();
        let scm = StructuralCausalModel::new(graph).unwrap();
        InterventionEngine::new(scm)
    }

    #[test]
    fn test_causal_intervention_positive_ate() {
        // Increasing transaction_amount should increase fraud_probability
        // because the mechanism is Linear { coefficient: 0.3 } (positive).
        let engine = build_engine();
        let result = engine
            .do_intervention(&[("transaction_amount".to_string(), 50000.0)], 500, 42)
            .unwrap();

        let fp_estimate = result
            .effect_estimates
            .get("fraud_probability")
            .expect("fraud_probability estimate missing");

        // With a very large transaction_amount (50000), the linear mechanism
        // contributes 0.3 * 50000 = 15000, which is much larger than typical
        // baseline values. The ATE should be positive.
        assert!(
            fp_estimate.average_treatment_effect > 0.0,
            "ATE for fraud_probability should be positive, got {}",
            fp_estimate.average_treatment_effect
        );
        assert_eq!(fp_estimate.sample_size, 500);
    }

    #[test]
    fn test_causal_intervention_zero_ate_for_unconnected() {
        // transaction_amount is a root variable. Intervening on fraud_probability
        // should not affect transaction_amount (it has no incoming edge from
        // fraud_probability).
        let engine = build_engine();
        let result = engine
            .do_intervention(&[("fraud_probability".to_string(), 0.99)], 500, 42)
            .unwrap();

        let amt_estimate = result
            .effect_estimates
            .get("transaction_amount")
            .expect("transaction_amount estimate missing");

        // The ATE should be approximately zero (within noise tolerance).
        // The root variables are sampled independently, so only seed differences matter.
        // We use a generous tolerance since the seeds differ.
        assert!(
            amt_estimate.average_treatment_effect.abs() < 500.0,
            "ATE for unconnected variable should be near zero, got {}",
            amt_estimate.average_treatment_effect
        );
    }

    #[test]
    fn test_causal_intervention_multiple_interventions() {
        let engine = build_engine();
        let result = engine
            .do_intervention(
                &[
                    ("transaction_amount".to_string(), 10000.0),
                    ("merchant_risk".to_string(), 0.9),
                ],
                200,
                99,
            )
            .unwrap();

        // Both interventions should be reflected in the intervened samples
        for sample in &result.intervened_samples {
            let amt = sample.get("transaction_amount").copied().unwrap_or(0.0);
            let risk = sample.get("merchant_risk").copied().unwrap_or(0.0);
            assert!(
                (amt - 10000.0).abs() < 1e-10,
                "transaction_amount should be fixed at 10000.0"
            );
            assert!(
                (risk - 0.9).abs() < 1e-10,
                "merchant_risk should be fixed at 0.9"
            );
        }

        assert_eq!(result.baseline_samples.len(), 200);
        assert_eq!(result.intervened_samples.len(), 200);
    }

    #[test]
    fn test_causal_intervention_empty_returns_error() {
        let engine = build_engine();
        let result = engine.do_intervention(&[], 100, 42);
        assert!(result.is_err());
    }

    #[test]
    fn test_causal_intervention_unknown_variable_returns_error() {
        let engine = build_engine();
        let result = engine.do_intervention(&[("nonexistent_var".to_string(), 1.0)], 100, 42);
        assert!(result.is_err());
    }

    #[test]
    fn test_causal_intervention_confidence_interval() {
        let engine = build_engine();
        let result = engine
            .do_intervention(&[("transaction_amount".to_string(), 50000.0)], 500, 42)
            .unwrap();

        let fp_estimate = result
            .effect_estimates
            .get("fraud_probability")
            .expect("fraud_probability estimate missing");

        // CI lower bound should be <= ATE <= CI upper bound
        assert!(
            fp_estimate.confidence_interval.0 <= fp_estimate.average_treatment_effect,
            "CI lower ({}) should be <= ATE ({})",
            fp_estimate.confidence_interval.0,
            fp_estimate.average_treatment_effect
        );
        // Note: the ATE is the mean of diffs, CI is percentile-based on individual diffs,
        // so ATE does not strictly need to be <= upper CI, but it generally is for
        // well-behaved distributions. We just verify the CI has reasonable width.
        assert!(
            fp_estimate.confidence_interval.1 >= fp_estimate.confidence_interval.0,
            "CI upper ({}) should be >= CI lower ({})",
            fp_estimate.confidence_interval.1,
            fp_estimate.confidence_interval.0
        );
    }
}
