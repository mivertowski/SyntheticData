//! Error cascade modeling.
//!
//! Models how errors propagate through a system, where one error
//! leads to others (e.g., wrong account coding leads to reconciliation
//! differences which lead to correcting entries).

use chrono::NaiveDate;
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use datasynth_core::models::AnomalyType;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};

/// Configuration for cascade generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeConfig {
    /// Maximum depth of cascade (how many steps).
    pub max_depth: u32,
    /// Probability of cascade continuing at each step.
    pub continuation_probability: f64,
    /// Whether cascades can branch (multiple consequences from one step).
    pub allow_branching: bool,
    /// Maximum branches per step.
    pub max_branches: u32,
}

impl Default for CascadeConfig {
    fn default() -> Self {
        Self {
            max_depth: 4,
            continuation_probability: 0.7,
            allow_branching: true,
            max_branches: 2,
        }
    }
}

/// A step in an error cascade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeStep {
    /// Step number in cascade.
    pub step: u32,
    /// Anomaly type for this step.
    pub anomaly_type: AnomalyType,
    /// Days after previous step.
    pub lag_days: i32,
    /// Description of why this step occurs.
    pub reason: String,
    /// Whether this step was executed.
    pub executed: bool,
    /// Document ID if executed.
    pub document_id: Option<String>,
    /// Anomaly ID if labeled.
    pub anomaly_id: Option<String>,
}

impl CascadeStep {
    /// Creates a new cascade step.
    pub fn new(step: u32, anomaly_type: AnomalyType, lag_days: i32) -> Self {
        Self {
            step,
            anomaly_type,
            lag_days,
            reason: String::new(),
            executed: false,
            document_id: None,
            anomaly_id: None,
        }
    }

    /// Sets the reason for this step.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }

    /// Marks step as executed.
    pub fn mark_executed(&mut self, document_id: impl Into<String>, anomaly_id: impl Into<String>) {
        self.executed = true;
        self.document_id = Some(document_id.into());
        self.anomaly_id = Some(anomaly_id.into());
    }
}

/// An error cascade instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCascade {
    /// Unique cascade ID.
    pub cascade_id: Uuid,
    /// Trigger anomaly type.
    pub trigger: AnomalyType,
    /// Trigger document ID.
    pub trigger_document_id: String,
    /// Trigger date.
    pub trigger_date: NaiveDate,
    /// Steps in the cascade.
    pub steps: Vec<CascadeStep>,
    /// Current step index.
    pub current_step: usize,
}

impl ErrorCascade {
    /// Creates a new error cascade.
    pub fn new(
        trigger: AnomalyType,
        trigger_document_id: impl Into<String>,
        trigger_date: NaiveDate,
        uuid_factory: &DeterministicUuidFactory,
    ) -> Self {
        Self {
            cascade_id: uuid_factory.next(),
            trigger,
            trigger_document_id: trigger_document_id.into(),
            trigger_date,
            steps: Vec::new(),
            current_step: 0,
        }
    }

    /// Adds a step to the cascade.
    pub fn add_step(&mut self, step: CascadeStep) {
        self.steps.push(step);
    }

    /// Gets the next step to execute.
    pub fn next_step(&self) -> Option<&CascadeStep> {
        self.steps.get(self.current_step)
    }

    /// Gets the next step mutably.
    pub fn next_step_mut(&mut self) -> Option<&mut CascadeStep> {
        self.steps.get_mut(self.current_step)
    }

    /// Advances to the next step.
    pub fn advance(&mut self) {
        if self.current_step < self.steps.len() {
            self.current_step += 1;
        }
    }

    /// Returns whether the cascade is complete.
    pub fn is_complete(&self) -> bool {
        self.current_step >= self.steps.len()
    }

    /// Gets the expected date for the next step.
    pub fn next_step_date(&self) -> Option<NaiveDate> {
        if let Some(step) = self.next_step() {
            // Calculate date based on trigger date plus accumulated lags
            let total_lag: i32 = self.steps[..self.current_step]
                .iter()
                .map(|s| s.lag_days)
                .sum::<i32>()
                + step.lag_days;
            Some(self.trigger_date + chrono::Duration::days(total_lag as i64))
        } else {
            None
        }
    }
}

/// Generator for error cascades.
pub struct CascadeGenerator {
    config: CascadeConfig,
    /// Deterministic UUID factory for cascade IDs.
    uuid_factory: DeterministicUuidFactory,
    /// Active cascades.
    active_cascades: Vec<ErrorCascade>,
    /// Completed cascades.
    completed_cascades: Vec<ErrorCascade>,
    /// Cascade templates by trigger type.
    templates: Vec<CascadeTemplate>,
}

/// Template for a cascade based on trigger type.
#[derive(Debug, Clone)]
pub struct CascadeTemplate {
    /// Trigger anomaly type.
    pub trigger: AnomalyType,
    /// Potential cascade steps.
    pub steps: Vec<CascadeStepTemplate>,
}

/// Template for a cascade step.
#[derive(Debug, Clone)]
pub struct CascadeStepTemplate {
    /// Anomaly type for this step.
    pub anomaly_type: AnomalyType,
    /// Probability this step occurs.
    pub probability: f64,
    /// Minimum lag days.
    pub lag_min: i32,
    /// Maximum lag days.
    pub lag_max: i32,
    /// Reason description.
    pub reason: String,
}

impl CascadeStepTemplate {
    /// Creates a new step template.
    pub fn new(
        anomaly_type: AnomalyType,
        probability: f64,
        lag_range: (i32, i32),
        reason: impl Into<String>,
    ) -> Self {
        Self {
            anomaly_type,
            probability,
            lag_min: lag_range.0,
            lag_max: lag_range.1,
            reason: reason.into(),
        }
    }
}

impl Default for CascadeGenerator {
    fn default() -> Self {
        Self::new(CascadeConfig::default())
    }
}

impl CascadeGenerator {
    /// Creates a new cascade generator.
    pub fn new(config: CascadeConfig) -> Self {
        Self {
            config,
            uuid_factory: DeterministicUuidFactory::new(0, GeneratorType::Anomaly),
            active_cascades: Vec::new(),
            completed_cascades: Vec::new(),
            templates: Self::default_templates(),
        }
    }

    /// Creates default cascade templates.
    fn default_templates() -> Vec<CascadeTemplate> {
        use datasynth_core::models::{ErrorType, ProcessIssueType};

        vec![
            // Account misclassification cascade
            CascadeTemplate {
                trigger: AnomalyType::Error(ErrorType::MisclassifiedAccount),
                steps: vec![
                    CascadeStepTemplate::new(
                        AnomalyType::Error(ErrorType::DuplicateEntry),
                        0.40,
                        (5, 15),
                        "Attempt to correct via additional entry",
                    ),
                    CascadeStepTemplate::new(
                        AnomalyType::Error(ErrorType::ReversedAmount),
                        0.30,
                        (10, 30),
                        "Reversal of original entry",
                    ),
                    CascadeStepTemplate::new(
                        AnomalyType::Error(ErrorType::WrongPeriod),
                        0.25,
                        (30, 60),
                        "Correction posted to wrong period",
                    ),
                ],
            },
            // Wrong period cascade
            CascadeTemplate {
                trigger: AnomalyType::Error(ErrorType::WrongPeriod),
                steps: vec![
                    CascadeStepTemplate::new(
                        AnomalyType::ProcessIssue(ProcessIssueType::LatePosting),
                        0.50,
                        (1, 5),
                        "Late correction posting",
                    ),
                    CascadeStepTemplate::new(
                        AnomalyType::Error(ErrorType::CutoffError),
                        0.35,
                        (5, 15),
                        "Additional cutoff issues from correction",
                    ),
                ],
            },
            // Missing field cascade
            CascadeTemplate {
                trigger: AnomalyType::Error(ErrorType::MissingField),
                steps: vec![
                    CascadeStepTemplate::new(
                        AnomalyType::ProcessIssue(ProcessIssueType::MissingDocumentation),
                        0.60,
                        (1, 7),
                        "Request for missing documentation",
                    ),
                    CascadeStepTemplate::new(
                        AnomalyType::ProcessIssue(ProcessIssueType::LatePosting),
                        0.40,
                        (5, 14),
                        "Delayed posting while gathering info",
                    ),
                ],
            },
            // Duplicate entry cascade
            CascadeTemplate {
                trigger: AnomalyType::Error(ErrorType::DuplicateEntry),
                steps: vec![CascadeStepTemplate::new(
                    AnomalyType::Error(ErrorType::ReversedAmount),
                    0.70,
                    (1, 5),
                    "Reversal of duplicate",
                )],
            },
        ]
    }

    /// Starts a new cascade if a template matches.
    pub fn maybe_start_cascade<R: Rng>(
        &mut self,
        trigger: &AnomalyType,
        document_id: impl Into<String>,
        date: NaiveDate,
        rng: &mut R,
    ) -> Option<Uuid> {
        // Find matching template
        let template = self.templates.iter().find(|t| t.trigger == *trigger)?;

        let mut cascade = ErrorCascade::new(trigger.clone(), document_id, date, &self.uuid_factory);

        // Generate steps from template
        let mut step_num = 0u32;
        for step_template in &template.steps {
            if rng.random::<f64>() < step_template.probability {
                step_num += 1;

                if step_num > self.config.max_depth {
                    break;
                }

                let lag = if step_template.lag_min == step_template.lag_max {
                    step_template.lag_min
                } else {
                    rng.random_range(step_template.lag_min..=step_template.lag_max)
                };

                let step = CascadeStep::new(step_num, step_template.anomaly_type.clone(), lag)
                    .with_reason(&step_template.reason);

                cascade.add_step(step);
            }
        }

        // Only create cascade if it has steps
        if cascade.steps.is_empty() {
            return None;
        }

        let cascade_id = cascade.cascade_id;
        self.active_cascades.push(cascade);
        Some(cascade_id)
    }

    /// Gets cascades that have steps due on or before a given date.
    pub fn get_due_cascades(&mut self, date: NaiveDate) -> Vec<(Uuid, CascadeStep)> {
        let mut due = Vec::new();

        for cascade in &self.active_cascades {
            if let Some(next_date) = cascade.next_step_date() {
                if next_date <= date {
                    if let Some(step) = cascade.next_step() {
                        due.push((cascade.cascade_id, step.clone()));
                    }
                }
            }
        }

        due
    }

    /// Marks a cascade step as executed and advances.
    pub fn execute_step(
        &mut self,
        cascade_id: Uuid,
        document_id: impl Into<String>,
        anomaly_id: impl Into<String>,
    ) {
        let doc_id = document_id.into();
        let ano_id = anomaly_id.into();

        if let Some(cascade) = self
            .active_cascades
            .iter_mut()
            .find(|c| c.cascade_id == cascade_id)
        {
            if let Some(step) = cascade.next_step_mut() {
                step.mark_executed(&doc_id, &ano_id);
            }
            cascade.advance();

            // Move to completed if done
            if cascade.is_complete() {
                // Will be handled by cleanup
            }
        }
    }

    /// Cleans up completed cascades.
    pub fn cleanup(&mut self) {
        let completed: Vec<_> = self
            .active_cascades
            .drain(..)
            .filter(|c| !c.is_complete())
            .collect();

        let newly_completed: Vec<_> = self
            .active_cascades
            .iter()
            .filter(|c| c.is_complete())
            .cloned()
            .collect();

        self.completed_cascades.extend(newly_completed);
        self.active_cascades = completed;
    }

    /// Returns active cascade count.
    pub fn active_count(&self) -> usize {
        self.active_cascades.len()
    }

    /// Returns completed cascade count.
    pub fn completed_count(&self) -> usize {
        self.completed_cascades.len()
    }

    /// Adds a custom template.
    pub fn add_template(&mut self, template: CascadeTemplate) {
        self.templates.push(template);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::ErrorType;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_cascade_step() {
        let step = CascadeStep::new(1, AnomalyType::Error(ErrorType::DuplicateEntry), 5)
            .with_reason("Test reason");

        assert_eq!(step.step, 1);
        assert_eq!(step.lag_days, 5);
        assert!(!step.executed);
    }

    #[test]
    fn test_error_cascade() {
        let uuid_factory = DeterministicUuidFactory::new(42, GeneratorType::Anomaly);
        let mut cascade = ErrorCascade::new(
            AnomalyType::Error(ErrorType::MisclassifiedAccount),
            "JE001",
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            &uuid_factory,
        );

        cascade.add_step(CascadeStep::new(
            1,
            AnomalyType::Error(ErrorType::DuplicateEntry),
            5,
        ));
        cascade.add_step(CascadeStep::new(
            2,
            AnomalyType::Error(ErrorType::ReversedAmount),
            10,
        ));

        assert_eq!(cascade.steps.len(), 2);
        assert!(!cascade.is_complete());

        // Check next step date
        let next_date = cascade.next_step_date().unwrap();
        assert_eq!(next_date, NaiveDate::from_ymd_opt(2024, 1, 20).unwrap());
    }

    #[test]
    fn test_cascade_generator() {
        let mut generator = CascadeGenerator::new(CascadeConfig::default());
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // Try to start a cascade for misclassified account
        let cascade_id = generator.maybe_start_cascade(
            &AnomalyType::Error(ErrorType::MisclassifiedAccount),
            "JE001",
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            &mut rng,
        );

        // May or may not create cascade depending on RNG
        if cascade_id.is_some() {
            assert!(generator.active_count() > 0);
        }
    }

    #[test]
    fn test_cascade_generator_no_match() {
        let mut generator = CascadeGenerator::new(CascadeConfig::default());
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // Try to start a cascade for an unregistered trigger
        let cascade_id = generator.maybe_start_cascade(
            &AnomalyType::Fraud(datasynth_core::models::FraudType::SelfApproval),
            "JE001",
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            &mut rng,
        );

        assert!(cascade_id.is_none());
    }
}
