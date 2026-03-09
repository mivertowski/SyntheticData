//! Audit procedure template generator.
//!
//! Generates audit procedure instances from ISA/PCAOB standards,
//! including sampling parameters, assertion coverage, and step definitions.

use chrono::NaiveDate;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

use datasynth_core::models::compliance::StandardCategory;
use datasynth_core::utils::seeded_rng;
use datasynth_standards::registry::StandardRegistry;

/// An audit procedure step.
#[derive(Debug, Clone, Serialize)]
pub struct ProcedureStep {
    pub step_number: u32,
    pub step_type: String,
    pub description: String,
    pub assertion: String,
}

/// An audit procedure instance.
#[derive(Debug, Clone, Serialize)]
pub struct AuditProcedureRecord {
    pub procedure_id: String,
    pub standard_id: String,
    pub procedure_type: String,
    pub title: String,
    pub description: String,
    pub sampling_method: String,
    pub sample_size: u32,
    pub confidence_level: f64,
    pub tolerable_misstatement: f64,
    pub assertions_tested: Vec<String>,
    pub jurisdiction: String,
    pub reference_date: String,
    pub steps: Vec<ProcedureStep>,
}

/// Configuration for procedure generation.
#[derive(Debug, Clone)]
pub struct ProcedureGeneratorConfig {
    pub procedures_per_standard: usize,
    pub sampling_method: String,
    pub confidence_level: f64,
    pub tolerable_misstatement: f64,
}

impl Default for ProcedureGeneratorConfig {
    fn default() -> Self {
        Self {
            procedures_per_standard: 3,
            sampling_method: "statistical".to_string(),
            confidence_level: 0.95,
            tolerable_misstatement: 0.05,
        }
    }
}

/// Procedure type templates derived from ISA standards.
const PROCEDURE_TEMPLATES: &[(&str, &str, &[&str])] = &[
    (
        "substantive_detail",
        "Test of Details",
        &["Occurrence", "Completeness", "Accuracy"],
    ),
    (
        "analytical",
        "Analytical Procedure",
        &["Accuracy", "ValuationAndAllocation", "Completeness"],
    ),
    (
        "controls_test",
        "Test of Operating Effectiveness",
        &["Occurrence", "Cutoff", "Classification"],
    ),
    (
        "inspection",
        "Inspection of Records/Documents",
        &[
            "Existence",
            "RightsAndObligations",
            "ValuationAndAllocation",
        ],
    ),
    (
        "confirmation",
        "External Confirmation",
        &["Existence", "CompletenessBalance", "RightsAndObligations"],
    ),
    (
        "recalculation",
        "Recalculation",
        &["Accuracy", "ValuationAndAllocation"],
    ),
    (
        "observation",
        "Observation of Process",
        &["Occurrence", "Completeness"],
    ),
    (
        "inquiry",
        "Inquiry of Management",
        &["CompletenessDisclosure", "AccuracyAndValuation"],
    ),
    (
        "cutoff_test",
        "Cutoff Testing",
        &["Cutoff", "Occurrence", "Completeness"],
    ),
];

/// Generator for audit procedure instances.
pub struct ProcedureGenerator {
    rng: ChaCha8Rng,
    config: ProcedureGeneratorConfig,
    counter: u32,
}

impl ProcedureGenerator {
    /// Creates a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config: ProcedureGeneratorConfig::default(),
            counter: 0,
        }
    }

    /// Creates a generator with custom configuration.
    pub fn with_config(seed: u64, config: ProcedureGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
            counter: 0,
        }
    }

    /// Generates audit procedures for a set of standards in a jurisdiction.
    pub fn generate_procedures(
        &mut self,
        registry: &StandardRegistry,
        jurisdiction: &str,
        reference_date: NaiveDate,
    ) -> Vec<AuditProcedureRecord> {
        let standards = registry.standards_for_jurisdiction(jurisdiction, reference_date);
        let mut procedures = Vec::new();

        for std in &standards {
            // Only generate procedures for auditing and regulatory standards
            let is_audit = matches!(
                std.category,
                StandardCategory::AuditingStandard | StandardCategory::RegulatoryRequirement
            );
            if !is_audit {
                continue;
            }

            let count = self
                .config
                .procedures_per_standard
                .min(PROCEDURE_TEMPLATES.len());
            for i in 0..count {
                let template_idx = (self.counter as usize + i) % PROCEDURE_TEMPLATES.len();
                let (proc_type, title, assertions) = PROCEDURE_TEMPLATES[template_idx];

                self.counter += 1;
                let procedure_id = format!("PROC-{:05}", self.counter);

                let sample_size = self.compute_sample_size();

                let steps = self.generate_steps(proc_type, assertions);

                procedures.push(AuditProcedureRecord {
                    procedure_id,
                    standard_id: std.id.as_str().to_string(),
                    procedure_type: proc_type.to_string(),
                    title: format!("{} — {}", title, std.title),
                    description: format!(
                        "{} procedure for {} compliance in jurisdiction {}",
                        title, std.id, jurisdiction
                    ),
                    sampling_method: self.config.sampling_method.clone(),
                    sample_size,
                    confidence_level: self.config.confidence_level,
                    tolerable_misstatement: self.config.tolerable_misstatement,
                    assertions_tested: assertions.iter().map(|a| a.to_string()).collect(),
                    jurisdiction: jurisdiction.to_string(),
                    reference_date: reference_date.to_string(),
                    steps,
                });
            }
        }

        procedures
    }

    fn compute_sample_size(&mut self) -> u32 {
        // Simplified sample size based on confidence level
        let base = if self.config.confidence_level >= 0.95 {
            58 // ~95% confidence for 5% tolerable
        } else if self.config.confidence_level >= 0.90 {
            38
        } else {
            25
        };

        // Add randomness ±20%
        let variation = self.rng.random_range(0.8f64..1.2f64);
        (base as f64 * variation) as u32
    }

    fn generate_steps(&self, proc_type: &str, assertions: &[&str]) -> Vec<ProcedureStep> {
        let base_steps: &[(&str, &str)] = match proc_type {
            "substantive_detail" => &[
                (
                    "selection",
                    "Select sample from population using statistical sampling",
                ),
                (
                    "inspection",
                    "Inspect supporting documentation for each item",
                ),
                ("verification", "Verify amounts agree to source documents"),
                (
                    "evaluation",
                    "Evaluate exceptions and project to population",
                ),
            ],
            "analytical" => &[
                (
                    "expectation",
                    "Develop independent expectation using prior-year data and trends",
                ),
                ("comparison", "Compare recorded amounts to expectation"),
                (
                    "investigation",
                    "Investigate significant variances exceeding threshold",
                ),
                (
                    "conclusion",
                    "Form conclusion on reasonableness of recorded amounts",
                ),
            ],
            "controls_test" => &[
                (
                    "selection",
                    "Select sample of transactions processed during the period",
                ),
                ("inspection", "Inspect evidence of control operation"),
                ("reperformance", "Reperform the control procedure"),
                (
                    "evaluation",
                    "Evaluate control exceptions and determine impact",
                ),
            ],
            "confirmation" => &[
                ("selection", "Select accounts for external confirmation"),
                ("dispatch", "Send confirmation requests to third parties"),
                ("receipt", "Receive and evaluate confirmation responses"),
                (
                    "alternative",
                    "Perform alternative procedures for non-responses",
                ),
            ],
            "cutoff_test" => &[
                ("selection", "Select transactions around period end"),
                ("inspection", "Inspect dates on source documents"),
                ("verification", "Verify recording in correct period"),
                ("evaluation", "Evaluate cutoff exceptions"),
            ],
            _ => &[
                ("planning", "Plan the procedure scope and approach"),
                ("execution", "Execute the procedure steps"),
                ("evaluation", "Evaluate results and form conclusion"),
            ],
        };

        base_steps
            .iter()
            .enumerate()
            .map(|(i, (step_type, desc))| {
                let assertion = if i < assertions.len() {
                    assertions[i].to_string()
                } else {
                    assertions[0].to_string()
                };

                ProcedureStep {
                    step_number: (i + 1) as u32,
                    step_type: step_type.to_string(),
                    description: desc.to_string(),
                    assertion,
                }
            })
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_procedures() {
        let registry = StandardRegistry::with_built_in();
        let mut gen = ProcedureGenerator::new(42);
        let date = NaiveDate::from_ymd_opt(2025, 6, 30).unwrap();
        let procedures = gen.generate_procedures(&registry, "US", date);
        assert!(
            !procedures.is_empty(),
            "Should generate procedures for US standards"
        );

        // Each procedure should have steps
        for proc in &procedures {
            assert!(!proc.steps.is_empty());
            assert!(!proc.assertions_tested.is_empty());
        }
    }
}
