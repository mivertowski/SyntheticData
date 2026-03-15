//! Procedure step generator for audit workpapers.
//!
//! Generates individual `AuditProcedureStep` records for a given workpaper,
//! aligned to the workpaper's `ProcedureType` per ISA 330.

use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use datasynth_core::models::audit::{
    Assertion, AuditProcedureStep, ProcedureType, StepProcedureType, StepResult, Workpaper,
};

/// Configuration for the procedure step generator (ISA 330).
#[derive(Debug, Clone)]
pub struct ProcedureStepGeneratorConfig {
    /// Number of steps to generate per workpaper (min, max)
    pub steps_per_workpaper: (u32, u32),
    /// Fraction of completed steps that pass with no exception
    pub pass_ratio: f64,
    /// Fraction of completed steps that yield an exception
    pub exception_ratio: f64,
    /// Fraction of completed steps that fail (material deviation)
    pub fail_ratio: f64,
    /// Fraction of planned steps that are actually performed
    pub completion_ratio: f64,
}

impl Default for ProcedureStepGeneratorConfig {
    fn default() -> Self {
        Self {
            steps_per_workpaper: (3, 8),
            pass_ratio: 0.85,
            exception_ratio: 0.10,
            fail_ratio: 0.05,
            completion_ratio: 0.90,
        }
    }
}

/// Generator for `AuditProcedureStep` records per ISA 330.
pub struct ProcedureStepGenerator {
    /// Seeded random number generator
    rng: ChaCha8Rng,
    /// Configuration
    config: ProcedureStepGeneratorConfig,
}

impl ProcedureStepGenerator {
    /// Create a new generator with the given seed and default configuration.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config: ProcedureStepGeneratorConfig::default(),
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: ProcedureStepGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
        }
    }

    /// Generate procedure steps for a workpaper.
    ///
    /// # Arguments
    /// * `workpaper` — The workpaper these steps belong to.
    /// * `engagement_id` — The engagement UUID (used directly; matches `workpaper.engagement_id`).
    /// * `team_members` — Slice of `(employee_id, employee_name)` pairs used to assign performers.
    pub fn generate_steps(
        &mut self,
        workpaper: &Workpaper,
        team_members: &[(String, String)],
    ) -> Vec<AuditProcedureStep> {
        let count = self
            .rng
            .random_range(self.config.steps_per_workpaper.0..=self.config.steps_per_workpaper.1)
            as usize;

        // Build an ordered list of assertions to cycle through.
        let assertions = self.assertions_for_procedure(workpaper.procedure_type);
        // Build the procedure types appropriate for this workpaper.
        let step_types = self.step_types_for_procedure(workpaper.procedure_type);

        let mut steps = Vec::with_capacity(count);

        for i in 0..count {
            let step_number = (i + 1) as u32;

            let assertion = assertions[i % assertions.len()];
            let proc_type = step_types[i % step_types.len()];
            let description = self.description_for(proc_type, assertion);

            let mut step = AuditProcedureStep::new(
                workpaper.workpaper_id,
                workpaper.engagement_id,
                step_number,
                description,
                proc_type,
                assertion,
            );

            // Determine whether this step is actually performed.
            if self.rng.random::<f64>() < self.config.completion_ratio {
                // Choose a random team member (fall back to a generic performer if none).
                let (performer_id, performer_name) = if !team_members.is_empty() {
                    let idx = self.rng.random_range(0..team_members.len());
                    (team_members[idx].0.clone(), team_members[idx].1.clone())
                } else {
                    ("STAFF001".to_string(), "Audit Staff".to_string())
                };

                // Pick a performance date within fieldwork — we use a simple offset so
                // the generator doesn't need the engagement (keeps the API minimal).
                let performed_date = workpaper.preparer_date;

                let result = self.random_result();

                step.perform(performer_id, performer_name, performed_date, result);

                if matches!(result, StepResult::Exception | StepResult::Fail) {
                    step.exception_description = Some(self.exception_text(assertion).to_string());
                }
            }

            steps.push(step);
        }

        steps
    }

    // -------------------------------------------------------------------------
    // Private helpers
    // -------------------------------------------------------------------------

    /// Choose assertion cycle based on workpaper procedure type.
    fn assertions_for_procedure(&self, proc_type: ProcedureType) -> Vec<Assertion> {
        match proc_type {
            ProcedureType::TestOfControls => vec![
                Assertion::Occurrence,
                Assertion::Completeness,
                Assertion::Accuracy,
                Assertion::Cutoff,
                Assertion::Classification,
            ],
            ProcedureType::SubstantiveTest => vec![
                Assertion::Existence,
                Assertion::Completeness,
                Assertion::ValuationAndAllocation,
                Assertion::RightsAndObligations,
                Assertion::Cutoff,
            ],
            ProcedureType::AnalyticalProcedures => vec![
                Assertion::Completeness,
                Assertion::ValuationAndAllocation,
                Assertion::Occurrence,
                Assertion::PresentationAndDisclosure,
            ],
            _ => vec![
                Assertion::Existence,
                Assertion::Completeness,
                Assertion::Accuracy,
                Assertion::Occurrence,
                Assertion::ValuationAndAllocation,
                Assertion::Classification,
            ],
        }
    }

    /// Choose step procedure types based on workpaper procedure type.
    fn step_types_for_procedure(&self, proc_type: ProcedureType) -> Vec<StepProcedureType> {
        match proc_type {
            ProcedureType::TestOfControls => vec![
                StepProcedureType::Reperformance,
                StepProcedureType::Observation,
                StepProcedureType::Inquiry,
            ],
            ProcedureType::SubstantiveTest => vec![
                StepProcedureType::Inspection,
                StepProcedureType::Vouching,
                StepProcedureType::Recalculation,
            ],
            ProcedureType::AnalyticalProcedures => {
                vec![StepProcedureType::AnalyticalProcedure]
            }
            _ => vec![
                StepProcedureType::Inspection,
                StepProcedureType::Observation,
                StepProcedureType::Inquiry,
                StepProcedureType::Reperformance,
                StepProcedureType::Vouching,
            ],
        }
    }

    /// Build a human-readable step description.
    fn description_for(&self, proc_type: StepProcedureType, assertion: Assertion) -> String {
        let proc_name = match proc_type {
            StepProcedureType::Inspection => "Inspect documents to verify",
            StepProcedureType::Observation => "Observe process controls to confirm",
            StepProcedureType::Inquiry => "Inquire of management regarding",
            StepProcedureType::Confirmation => "Obtain external confirmation of",
            StepProcedureType::Recalculation => "Recalculate amounts to verify",
            StepProcedureType::Reperformance => "Re-perform procedure to test",
            StepProcedureType::AnalyticalProcedure => "Apply analytical procedure to evaluate",
            StepProcedureType::Vouching => "Vouch transactions back to source documents for",
            StepProcedureType::Scanning => "Scan population for unusual items affecting",
        };

        let assertion_name = match assertion {
            Assertion::Occurrence => "occurrence of transactions",
            Assertion::Completeness => "completeness of recording",
            Assertion::Accuracy => "accuracy of amounts",
            Assertion::Cutoff => "period-end cutoff",
            Assertion::Classification => "proper classification",
            Assertion::Existence => "existence of balances",
            Assertion::RightsAndObligations => "rights and obligations",
            Assertion::ValuationAndAllocation => "valuation and allocation",
            Assertion::PresentationAndDisclosure => "presentation and disclosure",
        };

        format!("{proc_name} {assertion_name}.")
    }

    /// Pick a step result according to configured ratios.
    fn random_result(&mut self) -> StepResult {
        let roll: f64 = self.rng.random();
        let fail_cutoff = self.config.fail_ratio;
        let exception_cutoff = fail_cutoff + self.config.exception_ratio;
        // Anything above exception_cutoff → Pass (majority).

        if roll < fail_cutoff {
            StepResult::Fail
        } else if roll < exception_cutoff {
            StepResult::Exception
        } else {
            StepResult::Pass
        }
    }

    /// Return a short textual description of the exception.
    fn exception_text(&self, assertion: Assertion) -> &'static str {
        match assertion {
            Assertion::Occurrence => "Transaction cannot be traced to an approved source document.",
            Assertion::Completeness => {
                "Item exists in the population but was not recorded in the ledger."
            }
            Assertion::Accuracy => {
                "Recorded amount differs from the supporting document by more than 1%."
            }
            Assertion::Cutoff => "Transaction recorded in the wrong accounting period.",
            Assertion::Classification => {
                "Amount posted to incorrect expense or balance sheet account."
            }
            Assertion::Existence => {
                "Asset could not be physically located or confirmed with a third party."
            }
            Assertion::RightsAndObligations => {
                "Evidence of ownership or obligation could not be obtained."
            }
            Assertion::ValuationAndAllocation => {
                "Carrying value is inconsistent with observable market inputs."
            }
            Assertion::PresentationAndDisclosure => {
                "Disclosure is incomplete or does not meet the applicable framework."
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::audit::{StepStatus, Workpaper, WorkpaperSection};
    use uuid::Uuid;

    fn make_gen(seed: u64) -> ProcedureStepGenerator {
        ProcedureStepGenerator::new(seed)
    }

    fn make_workpaper(proc_type: ProcedureType) -> Workpaper {
        Workpaper::new(
            Uuid::new_v4(),
            "C-100",
            "Test Workpaper",
            WorkpaperSection::ControlTesting,
        )
        .with_procedure("Test procedure", proc_type)
    }

    fn team() -> Vec<(String, String)> {
        vec![
            ("EMP001".to_string(), "Alice Auditor".to_string()),
            ("EMP002".to_string(), "Bob Checker".to_string()),
        ]
    }

    // -------------------------------------------------------------------------

    /// Count falls within the configured (min, max) range.
    #[test]
    fn test_generates_steps() {
        let wp = make_workpaper(ProcedureType::SubstantiveTest);
        let mut gen = make_gen(42);
        let steps = gen.generate_steps(&wp, &team());

        let cfg = ProcedureStepGeneratorConfig::default();
        let min = cfg.steps_per_workpaper.0 as usize;
        let max = cfg.steps_per_workpaper.1 as usize;
        assert!(
            steps.len() >= min && steps.len() <= max,
            "expected {min}..={max}, got {}",
            steps.len()
        );
    }

    /// With the default completion_ratio of 0.90 most steps should be Complete.
    #[test]
    fn test_step_completion() {
        let wp = make_workpaper(ProcedureType::TestOfControls);
        // Use a large fixed count so the ratio is measurable.
        let config = ProcedureStepGeneratorConfig {
            steps_per_workpaper: (100, 100),
            completion_ratio: 0.80,
            ..Default::default()
        };
        let mut gen = ProcedureStepGenerator::with_config(99, config);
        let steps = gen.generate_steps(&wp, &team());

        let completed = steps
            .iter()
            .filter(|s| s.status == StepStatus::Complete)
            .count();
        let ratio = completed as f64 / steps.len() as f64;
        // Expect within ±15% of the 80% target.
        assert!(
            (0.65..=0.95).contains(&ratio),
            "completion ratio {ratio:.2} outside expected 65–95%"
        );
    }

    /// Pass / exception / fail distribution should roughly match configured ratios.
    #[test]
    fn test_result_distribution() {
        let wp = make_workpaper(ProcedureType::SubstantiveTest);
        let config = ProcedureStepGeneratorConfig {
            steps_per_workpaper: (200, 200),
            completion_ratio: 1.0, // perform every step so we get full sample
            pass_ratio: 0.85,
            exception_ratio: 0.10,
            fail_ratio: 0.05,
        };
        let mut gen = ProcedureStepGenerator::with_config(77, config);
        let steps = gen.generate_steps(&wp, &team());

        let pass_count = steps
            .iter()
            .filter(|s| s.result == Some(StepResult::Pass))
            .count() as f64;
        let total = steps.len() as f64;

        // Pass ratio should be within ±15% of 85%.
        let pass_ratio = pass_count / total;
        assert!(
            (0.70..=1.00).contains(&pass_ratio),
            "pass ratio {pass_ratio:.2} outside expected 70–100%"
        );
    }

    /// TestOfControls workpapers should only get Reperformance / Observation / Inquiry steps.
    #[test]
    fn test_procedure_type_alignment() {
        let wp = make_workpaper(ProcedureType::TestOfControls);
        let config = ProcedureStepGeneratorConfig {
            steps_per_workpaper: (50, 50),
            ..Default::default()
        };
        let mut gen = ProcedureStepGenerator::with_config(11, config);
        let steps = gen.generate_steps(&wp, &team());

        let expected = [
            StepProcedureType::Reperformance,
            StepProcedureType::Observation,
            StepProcedureType::Inquiry,
        ];
        for step in &steps {
            assert!(
                expected.contains(&step.procedure_type),
                "unexpected procedure_type {:?} for TestOfControls workpaper",
                step.procedure_type,
            );
        }
    }

    /// Same seed produces identical output.
    #[test]
    fn test_deterministic() {
        let wp = make_workpaper(ProcedureType::SubstantiveTest);

        let steps_a = ProcedureStepGenerator::new(1234).generate_steps(&wp, &team());
        let steps_b = ProcedureStepGenerator::new(1234).generate_steps(&wp, &team());

        assert_eq!(steps_a.len(), steps_b.len());
        for (a, b) in steps_a.iter().zip(steps_b.iter()) {
            assert_eq!(a.step_number, b.step_number);
            assert_eq!(a.procedure_type, b.procedure_type);
            assert_eq!(a.assertion, b.assertion);
            assert_eq!(a.status, b.status);
            assert_eq!(a.result, b.result);
        }
    }
}
