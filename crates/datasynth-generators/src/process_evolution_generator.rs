//! Generator for process evolution events (automation, workflow changes, etc.).

use chrono::NaiveDate;
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use datasynth_core::models::process_evolution::{
    ApprovalWorkflowChangeConfig, ControlEnhancementConfig, PolicyCategory, PolicyChangeConfig,
    ProcessAutomationConfig, ProcessEvolutionEvent, ProcessEvolutionType, RolloutCurve,
    ThresholdChange, WorkflowType,
};

/// Configuration for the process evolution generator.
///
/// Controls the mix of event types and the average frequency.
#[derive(Debug, Clone)]
pub struct ProcEvoGeneratorConfig {
    /// Weights: [workflow_change, automation, policy_change, control_enhancement]
    pub type_weights: [f64; 4],
    /// Average events per year.
    pub events_per_year: f64,
}

impl Default for ProcEvoGeneratorConfig {
    fn default() -> Self {
        Self {
            type_weights: [0.25, 0.30, 0.25, 0.20],
            events_per_year: 4.0,
        }
    }
}

/// Generates [`ProcessEvolutionEvent`] instances for a given date range.
///
/// Each generated event includes a fully-populated type-specific configuration
/// (approval workflow change, process automation, policy change, or control
/// enhancement).
pub struct ProcessEvolutionGenerator {
    rng: ChaCha8Rng,
    config: ProcEvoGeneratorConfig,
    event_counter: usize,
}

/// Discriminator added to the seed so this generator's RNG stream does not
/// overlap with other generators that may share the same base seed.
const SEED_DISCRIMINATOR: u64 = 0xAE_0C;

impl ProcessEvolutionGenerator {
    /// Create a new generator with the given seed and default config.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, SEED_DISCRIMINATOR),
            config: ProcEvoGeneratorConfig::default(),
            event_counter: 0,
        }
    }

    /// Create a new generator with the given seed and custom config.
    pub fn with_config(seed: u64, config: ProcEvoGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, SEED_DISCRIMINATOR),
            config,
            event_counter: 0,
        }
    }

    /// Generate process evolution events within the given date range.
    ///
    /// Events are returned sorted by effective date.
    pub fn generate_events(
        &mut self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Vec<ProcessEvolutionEvent> {
        let total_days = (end_date - start_date).num_days().max(1) as f64;
        let total_years = total_days / 365.25;
        let expected_count = (self.config.events_per_year * total_years).round() as usize;
        let count = expected_count.max(1);

        let mut events = Vec::with_capacity(count);

        for _ in 0..count {
            self.event_counter += 1;
            let days_offset = self.rng.random_range(0..total_days as i64);
            let effective_date = start_date + chrono::Duration::days(days_offset);

            let event = self.build_event(effective_date);
            events.push(event);
        }

        events.sort_by_key(|e| e.effective_date);
        events
    }

    /// Pick an event type variant index from the configured weights.
    fn pick_event_type_index(&mut self) -> usize {
        let weights = &self.config.type_weights;
        let total: f64 = weights.iter().sum();
        let mut r: f64 = self.rng.random_range(0.0..total);

        for (i, &w) in weights.iter().enumerate() {
            r -= w;
            if r <= 0.0 {
                return i;
            }
        }
        0
    }

    /// Build a complete [`ProcessEvolutionEvent`] with a randomly chosen type
    /// and populated configuration.
    fn build_event(&mut self, effective_date: NaiveDate) -> ProcessEvolutionEvent {
        let event_id = format!("PROC-EVT-{:06}", self.event_counter);
        let type_idx = self.pick_event_type_index();

        let event_type = match type_idx {
            0 => self.build_workflow_change(),
            1 => self.build_automation(),
            2 => self.build_policy_change(),
            _ => self.build_control_enhancement(),
        };

        let description = match &event_type {
            ProcessEvolutionType::ApprovalWorkflowChange(c) => {
                Some(format!("Workflow change from {:?} to {:?}", c.from, c.to))
            }
            ProcessEvolutionType::ProcessAutomation(c) => {
                Some(format!("Automation of {} process", c.process_name))
            }
            ProcessEvolutionType::PolicyChange(c) => {
                Some(format!("Policy change in {:?} category", c.category))
            }
            ProcessEvolutionType::ControlEnhancement(c) => {
                Some(format!("Enhancement of control {}", c.control_id))
            }
        };

        let tags = vec![format!("type:{}", event_type.type_name())];

        ProcessEvolutionEvent {
            event_id,
            event_type,
            effective_date,
            description,
            tags,
        }
    }

    // ------------------------------------------------------------------
    // Type-specific builders
    // ------------------------------------------------------------------

    fn build_workflow_change(&mut self) -> ProcessEvolutionType {
        let all_types = [
            WorkflowType::SingleApprover,
            WorkflowType::DualApproval,
            WorkflowType::MultiLevel,
            WorkflowType::Automated,
            WorkflowType::Matrix,
            WorkflowType::Parallel,
        ];

        let from_idx = self.rng.random_range(0..all_types.len());
        let mut to_idx = self.rng.random_range(0..all_types.len());
        // Ensure from != to
        while to_idx == from_idx {
            to_idx = self.rng.random_range(0..all_types.len());
        }

        let from = all_types[from_idx];
        let to = all_types[to_idx];
        let time_delta = to.processing_time_multiplier() / from.processing_time_multiplier();
        let error_rate_impact = self.rng.random_range(0.01..0.04);
        let transition_months = self.rng.random_range(2..6_u32);

        let threshold_changes = if self.rng.random_bool(0.4) {
            let old_val = Decimal::from(self.rng.random_range(5000..10000_i64));
            let new_val = Decimal::from(self.rng.random_range(10000..25000_i64));
            vec![ThresholdChange {
                category: "amount".to_string(),
                old_threshold: old_val,
                new_threshold: new_val,
            }]
        } else {
            Vec::new()
        };

        ProcessEvolutionType::ApprovalWorkflowChange(ApprovalWorkflowChangeConfig {
            from,
            to,
            time_delta,
            error_rate_impact,
            transition_months,
            threshold_changes,
        })
    }

    fn build_automation(&mut self) -> ProcessEvolutionType {
        let process_names = [
            "three_way_match",
            "invoice_processing",
            "expense_approval",
            "bank_reconciliation",
            "period_close_checklist",
        ];
        let idx = self.rng.random_range(0..process_names.len());
        let process_name = process_names[idx].to_string();

        let manual_rate_before = self.rng.random_range(0.60..0.90);
        let manual_rate_after = self.rng.random_range(0.05..0.25);
        let error_rate_before = self.rng.random_range(0.03..0.08);
        let error_rate_after = self.rng.random_range(0.005..0.02);
        let processing_time_reduction = self.rng.random_range(0.20..0.50);
        let rollout_months = self.rng.random_range(3..12_u32);

        let curves = [
            RolloutCurve::Linear,
            RolloutCurve::SCurve,
            RolloutCurve::Exponential,
            RolloutCurve::Step,
        ];
        let curve_idx = self.rng.random_range(0..curves.len());
        let rollout_curve = curves[curve_idx];

        ProcessEvolutionType::ProcessAutomation(ProcessAutomationConfig {
            process_name,
            manual_rate_before,
            manual_rate_after,
            error_rate_before,
            error_rate_after,
            processing_time_reduction,
            rollout_months,
            rollout_curve,
            affected_transaction_types: Vec::new(),
        })
    }

    fn build_policy_change(&mut self) -> ProcessEvolutionType {
        let categories = [
            PolicyCategory::ApprovalThreshold,
            PolicyCategory::ExpensePolicy,
            PolicyCategory::TravelPolicy,
            PolicyCategory::ProcurementPolicy,
            PolicyCategory::CreditPolicy,
            PolicyCategory::InventoryPolicy,
            PolicyCategory::DocumentationRequirement,
            PolicyCategory::Other,
        ];
        let cat_idx = self.rng.random_range(0..categories.len());
        let category = categories[cat_idx];

        // Generate threshold values for threshold-type policies
        let (old_value, new_value) = match category {
            PolicyCategory::ApprovalThreshold
            | PolicyCategory::ExpensePolicy
            | PolicyCategory::CreditPolicy => {
                let old = Decimal::from(self.rng.random_range(1000..10000_i64));
                let new = Decimal::from(self.rng.random_range(5000..20000_i64));
                (Some(old), Some(new))
            }
            PolicyCategory::InventoryPolicy | PolicyCategory::ProcurementPolicy => {
                let old = Decimal::from(self.rng.random_range(100..500_i64));
                let new = Decimal::from(self.rng.random_range(200..1000_i64));
                (Some(old), Some(new))
            }
            _ => (None, None),
        };

        let transition_error_rate = self.rng.random_range(0.02..0.06);
        let transition_months = self.rng.random_range(2..6_u32);

        ProcessEvolutionType::PolicyChange(PolicyChangeConfig {
            category,
            description: Some(format!("Updated {} policy", category.code())),
            old_value,
            new_value,
            transition_error_rate,
            transition_months,
            affected_controls: Vec::new(),
        })
    }

    fn build_control_enhancement(&mut self) -> ProcessEvolutionType {
        let control_id = format!("C-{:03}", self.event_counter);
        let error_reduction = self.rng.random_range(0.01..0.05);
        let processing_time_impact = self.rng.random_range(1.02..1.15);
        let implementation_months = self.rng.random_range(1..4_u32);

        let tolerance_change = if self.rng.random_bool(0.5) {
            let old_tol = dec!(100) + Decimal::from(self.rng.random_range(0..400_i64));
            let new_tol = dec!(50) + Decimal::from(self.rng.random_range(0..200_i64));
            Some(datasynth_core::models::process_evolution::ToleranceChange {
                old_tolerance: old_tol,
                new_tolerance: new_tol,
                tolerance_type: datasynth_core::models::process_evolution::ToleranceType::Absolute,
            })
        } else {
            None
        };

        ProcessEvolutionType::ControlEnhancement(ControlEnhancementConfig {
            control_id,
            description: Some(format!(
                "Enhanced control with {:.1}% error reduction",
                error_reduction * 100.0
            )),
            tolerance_change,
            error_reduction,
            processing_time_impact,
            implementation_months,
            additional_evidence: Vec::new(),
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_generation() {
        let mut gen1 = ProcessEvolutionGenerator::new(42);
        let mut gen2 = ProcessEvolutionGenerator::new(42);
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let events1 = gen1.generate_events(start, end);
        let events2 = gen2.generate_events(start, end);

        assert_eq!(events1.len(), events2.len());
        for (e1, e2) in events1.iter().zip(events2.iter()) {
            assert_eq!(e1.event_id, e2.event_id);
            assert_eq!(e1.effective_date, e2.effective_date);
            assert_eq!(e1.event_type.type_name(), e2.event_type.type_name());
        }
    }

    #[test]
    fn test_events_sorted_by_date() {
        let mut gen = ProcessEvolutionGenerator::new(42);
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();

        let events = gen.generate_events(start, end);
        for w in events.windows(2) {
            assert!(w[0].effective_date <= w[1].effective_date);
        }
    }

    #[test]
    fn test_all_event_types_generated() {
        let config = ProcEvoGeneratorConfig {
            type_weights: [1.0, 1.0, 1.0, 1.0],
            events_per_year: 100.0,
        };
        let mut gen = ProcessEvolutionGenerator::with_config(42, config);
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let events = gen.generate_events(start, end);

        let has_workflow = events.iter().any(|e| {
            matches!(
                e.event_type,
                ProcessEvolutionType::ApprovalWorkflowChange(_)
            )
        });
        let has_automation = events
            .iter()
            .any(|e| matches!(e.event_type, ProcessEvolutionType::ProcessAutomation(_)));
        let has_policy = events
            .iter()
            .any(|e| matches!(e.event_type, ProcessEvolutionType::PolicyChange(_)));
        let has_control = events
            .iter()
            .any(|e| matches!(e.event_type, ProcessEvolutionType::ControlEnhancement(_)));

        assert!(has_workflow, "should generate workflow changes");
        assert!(has_automation, "should generate automation events");
        assert!(has_policy, "should generate policy changes");
        assert!(has_control, "should generate control enhancements");
    }

    #[test]
    fn test_events_within_date_range() {
        let mut gen = ProcessEvolutionGenerator::new(42);
        let start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let events = gen.generate_events(start, end);
        for e in &events {
            assert!(e.effective_date >= start, "event date before start");
            assert!(e.effective_date <= end, "event date after end");
        }
    }

    #[test]
    fn test_workflow_change_valid_transitions() {
        let config = ProcEvoGeneratorConfig {
            type_weights: [1.0, 0.0, 0.0, 0.0],
            events_per_year: 50.0,
        };
        let mut gen = ProcessEvolutionGenerator::with_config(42, config);
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let events = gen.generate_events(start, end);
        for e in &events {
            if let ProcessEvolutionType::ApprovalWorkflowChange(ref c) = e.event_type {
                assert_ne!(c.from, c.to, "from and to workflow types must differ");
            } else {
                panic!("expected only workflow change events");
            }
        }
    }

    #[test]
    fn test_automation_config_populated() {
        let config = ProcEvoGeneratorConfig {
            type_weights: [0.0, 1.0, 0.0, 0.0],
            events_per_year: 20.0,
        };
        let mut gen = ProcessEvolutionGenerator::with_config(42, config);
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let events = gen.generate_events(start, end);
        for e in &events {
            if let ProcessEvolutionType::ProcessAutomation(ref c) = e.event_type {
                assert!(
                    !c.process_name.is_empty(),
                    "process_name should not be empty"
                );
                assert!(
                    c.manual_rate_before >= 0.60 && c.manual_rate_before <= 0.90,
                    "manual_rate_before out of range: {}",
                    c.manual_rate_before
                );
                assert!(
                    c.manual_rate_after >= 0.05 && c.manual_rate_after <= 0.25,
                    "manual_rate_after out of range: {}",
                    c.manual_rate_after
                );
                assert!(
                    c.manual_rate_before > c.manual_rate_after,
                    "manual_rate_before should exceed manual_rate_after"
                );
            } else {
                panic!("expected only automation events");
            }
        }
    }

    #[test]
    fn test_s_curve_automation_progression() {
        let config = ProcEvoGeneratorConfig {
            type_weights: [0.0, 1.0, 0.0, 0.0],
            events_per_year: 20.0,
        };
        let mut gen = ProcessEvolutionGenerator::with_config(42, config);
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let events = gen.generate_events(start, end);

        // Find an automation event with SCurve rollout
        let s_curve_event = events.iter().find(|e| {
            if let ProcessEvolutionType::ProcessAutomation(ref c) = e.event_type {
                c.rollout_curve == RolloutCurve::SCurve
            } else {
                false
            }
        });

        // With 20 events/year and only automation type, we should find at least one SCurve
        // (SCurve is one of 4 curve types, so ~25% chance per event, 20 events => very likely)
        if let Some(evt) = s_curve_event {
            if let ProcessEvolutionType::ProcessAutomation(ref c) = evt.event_type {
                let rate_0 = c.automation_rate_at_progress(0.0);
                let rate_25 = c.automation_rate_at_progress(0.25);
                let rate_50 = c.automation_rate_at_progress(0.5);
                let rate_75 = c.automation_rate_at_progress(0.75);
                let rate_100 = c.automation_rate_at_progress(1.0);

                // S-curve: monotonically increasing
                assert!(rate_0 <= rate_25, "rate should increase from 0% to 25%");
                assert!(rate_25 <= rate_50, "rate should increase from 25% to 50%");
                assert!(rate_50 <= rate_75, "rate should increase from 50% to 75%");
                assert!(rate_75 <= rate_100, "rate should increase from 75% to 100%");

                // S-curve should have faster growth in the middle than at the edges
                let delta_first_quarter = rate_25 - rate_0;
                let delta_second_quarter = rate_50 - rate_25;
                let delta_last_quarter = rate_100 - rate_75;

                assert!(
                    delta_second_quarter > delta_first_quarter,
                    "S-curve: middle growth ({}) should exceed early growth ({})",
                    delta_second_quarter,
                    delta_first_quarter
                );
                assert!(
                    delta_second_quarter > delta_last_quarter,
                    "S-curve: middle growth ({}) should exceed late growth ({})",
                    delta_second_quarter,
                    delta_last_quarter
                );
            }
        } else {
            // If no SCurve found (unlikely), test with a manual config
            let manual_config = ProcessAutomationConfig {
                manual_rate_before: 0.80,
                manual_rate_after: 0.10,
                rollout_curve: RolloutCurve::SCurve,
                ..Default::default()
            };
            let rate_0 = manual_config.automation_rate_at_progress(0.0);
            let rate_50 = manual_config.automation_rate_at_progress(0.5);
            let rate_100 = manual_config.automation_rate_at_progress(1.0);
            assert!(rate_0 < rate_50);
            assert!(rate_50 < rate_100);
        }
    }
}
