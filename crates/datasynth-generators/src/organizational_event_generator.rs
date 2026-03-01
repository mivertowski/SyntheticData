//! Generator for organizational events (acquisitions, divestitures, etc.).
//!
//! Produces realistic organizational events across a date range for a set of
//! companies. Event types are drawn from a weighted distribution and each
//! event is populated with sensible random configuration values while
//! remaining fully deterministic given the same seed.

use chrono::NaiveDate;
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

use datasynth_core::models::organizational_event::{
    AcquisitionConfig, DateRange, DivestitureConfig, IntegrationPhaseConfig, LeadershipChangeConfig,
    MergerConfig, OrganizationalEvent, OrganizationalEventType, PolicyArea, PolicyChangeDetail,
    ReorganizationConfig, ReportingChange, WorkforceReductionConfig,
};

/// Probability distribution across event types.
///
/// The six weights correspond to:
/// `[acquisition, divestiture, reorganization, leadership_change, workforce_reduction, merger]`
#[derive(Debug, Clone)]
pub struct OrgEventGeneratorConfig {
    /// Probability distribution across event types.
    pub type_weights: [f64; 6],
    /// Average number of events per year.
    pub events_per_year: f64,
}

impl Default for OrgEventGeneratorConfig {
    fn default() -> Self {
        Self {
            type_weights: [0.15, 0.10, 0.25, 0.20, 0.15, 0.15],
            events_per_year: 3.0,
        }
    }
}

/// Generates [`OrganizationalEvent`] instances for a given date range.
///
/// Each generated event includes a fully-populated type-specific configuration
/// (acquisition, divestiture, reorganization, leadership change, workforce
/// reduction, or merger).
pub struct OrganizationalEventGenerator {
    rng: ChaCha8Rng,
    config: OrgEventGeneratorConfig,
    event_counter: usize,
}

/// Discriminator added to the seed so this generator's RNG stream does not
/// overlap with other generators that may share the same base seed.
const SEED_DISCRIMINATOR: u64 = 0xAE_0B;

impl OrganizationalEventGenerator {
    /// Create a new generator with the given seed and default config.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, SEED_DISCRIMINATOR),
            config: OrgEventGeneratorConfig::default(),
            event_counter: 0,
        }
    }

    /// Create a new generator with the given seed and custom config.
    pub fn with_config(seed: u64, config: OrgEventGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, SEED_DISCRIMINATOR),
            config,
            event_counter: 0,
        }
    }

    /// Generate organizational events within the given date range for the
    /// given company codes.
    ///
    /// Events are returned sorted by effective date.
    pub fn generate_events(
        &mut self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        company_codes: &[String],
    ) -> Vec<OrganizationalEvent> {
        let total_days = (end_date - start_date).num_days().max(1) as f64;
        let total_years = total_days / 365.25;
        let expected_count = (self.config.events_per_year * total_years).round() as usize;
        let count = expected_count.max(1);

        let mut events = Vec::with_capacity(count);

        for _ in 0..count {
            self.event_counter += 1;
            let days_offset = self.rng.random_range(0..total_days as i64);
            let effective_date = start_date + chrono::Duration::days(days_offset);

            let company_code = if company_codes.is_empty() {
                "C001".to_string()
            } else {
                let idx = self.rng.random_range(0..company_codes.len());
                company_codes[idx].clone()
            };

            let event = self.build_event(effective_date, &company_code);
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

    /// Build a complete [`OrganizationalEvent`] with a randomly chosen type
    /// and populated configuration.
    fn build_event(
        &mut self,
        effective_date: NaiveDate,
        company_code: &str,
    ) -> OrganizationalEvent {
        let event_id = format!("ORG-EVT-{:06}", self.event_counter);
        let type_idx = self.pick_event_type_index();

        let event_type = match type_idx {
            0 => self.build_acquisition(effective_date, company_code),
            1 => self.build_divestiture(effective_date, company_code),
            2 => self.build_reorganization(effective_date),
            3 => self.build_leadership_change(effective_date),
            4 => self.build_workforce_reduction(effective_date),
            _ => self.build_merger(effective_date, company_code),
        };

        let description = match &event_type {
            OrganizationalEventType::Acquisition(c) => Some(format!(
                "Acquisition of {} by {}",
                c.acquired_entity_code, company_code
            )),
            OrganizationalEventType::Divestiture(c) => Some(format!(
                "Divestiture of {} from {}",
                c.divested_entity_code, company_code
            )),
            OrganizationalEventType::Reorganization(_) => {
                Some(format!("Organizational restructuring at {}", company_code))
            }
            OrganizationalEventType::LeadershipChange(c) => {
                Some(format!("{} transition at {}", c.role, company_code))
            }
            OrganizationalEventType::WorkforceReduction(c) => Some(format!(
                "Workforce reduction ({:.0}%) at {}",
                c.reduction_percent * 100.0,
                company_code
            )),
            OrganizationalEventType::Merger(c) => Some(format!(
                "Merger with {} for {}",
                c.merged_entity_code, company_code
            )),
        };

        let tags = vec![
            format!("company:{}", company_code),
            format!("type:{}", event_type.type_name()),
        ];

        OrganizationalEvent {
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

    fn build_acquisition(
        &mut self,
        effective_date: NaiveDate,
        company_code: &str,
    ) -> OrganizationalEventType {
        let seq = self.event_counter;
        let entity_code = format!("ACQ-{}-{:04}", company_code, seq);
        let volume_mult = self.rng.random_range(1.10..1.60);
        let parallel_days = self.rng.random_range(15..60_u32);

        let cutover = effective_date + chrono::Duration::days(parallel_days as i64);
        let stabilization_end = cutover + chrono::Duration::days(self.rng.random_range(60..120));

        OrganizationalEventType::Acquisition(AcquisitionConfig {
            acquired_entity_code: entity_code.clone(),
            acquired_entity_name: Some(format!("Acquired Entity {}", seq)),
            acquisition_date: effective_date,
            volume_multiplier: volume_mult,
            integration_error_rate: self.rng.random_range(0.02..0.08),
            parallel_posting_days: parallel_days,
            coding_error_rate: self.rng.random_range(0.01..0.05),
            integration_phases: IntegrationPhaseConfig {
                parallel_run: Some(DateRange {
                    start: effective_date,
                    end: cutover - chrono::Duration::days(1),
                }),
                cutover_date: cutover,
                stabilization_end,
                parallel_run_error_rate: self.rng.random_range(0.05..0.12),
                stabilization_error_rate: self.rng.random_range(0.01..0.05),
            },
            purchase_price_allocation: None,
        })
    }

    fn build_divestiture(
        &mut self,
        effective_date: NaiveDate,
        company_code: &str,
    ) -> OrganizationalEventType {
        let seq = self.event_counter;
        let entity_code = format!("DIV-{}-{:04}", company_code, seq);
        let transition = self.rng.random_range(2..6_u32);

        OrganizationalEventType::Divestiture(DivestitureConfig {
            divested_entity_code: entity_code,
            divested_entity_name: Some(format!("Divested Unit {}", seq)),
            divestiture_date: effective_date,
            volume_reduction: self.rng.random_range(0.50..0.85),
            transition_months: transition,
            remove_entity: true,
            account_closures: Vec::new(),
            disposal_gain_loss: None,
        })
    }

    fn build_reorganization(&mut self, effective_date: NaiveDate) -> OrganizationalEventType {
        let transition = self.rng.random_range(2..6_u32);
        let remap_count = self.rng.random_range(1..4_usize);

        let mut cost_center_remapping = HashMap::new();
        for i in 0..remap_count {
            cost_center_remapping.insert(
                format!("CC-{:03}", 100 + i * 10),
                format!("CC-{:03}", 500 + i * 10),
            );
        }

        let reporting_changes = if self.rng.random_bool(0.5) {
            vec![ReportingChange {
                entity: "Engineering".to_string(),
                from_reports_to: "VP Engineering".to_string(),
                to_reports_to: "CTO".to_string(),
            }]
        } else {
            Vec::new()
        };

        OrganizationalEventType::Reorganization(ReorganizationConfig {
            description: Some("Organizational restructuring".to_string()),
            effective_date,
            cost_center_remapping,
            department_remapping: HashMap::new(),
            reporting_changes,
            transition_months: transition,
            transition_error_rate: self.rng.random_range(0.02..0.06),
        })
    }

    fn build_leadership_change(&mut self, effective_date: NaiveDate) -> OrganizationalEventType {
        let roles = ["CFO", "CEO", "Controller", "COO", "CTO", "VP Finance"];
        let role_idx = self.rng.random_range(0..roles.len());
        let role = roles[role_idx].to_string();

        let policy_changes = if self.rng.random_bool(0.6) {
            vec![PolicyChangeDetail {
                policy_area: PolicyArea::ApprovalThreshold,
                description: "Updated approval thresholds".to_string(),
                old_value: None,
                new_value: None,
            }]
        } else {
            Vec::new()
        };

        OrganizationalEventType::LeadershipChange(LeadershipChangeConfig {
            role,
            change_date: effective_date,
            policy_changes,
            vendor_review_triggered: self.rng.random_bool(0.3),
            policy_transition_months: self.rng.random_range(3..9_u32),
            policy_change_error_rate: self.rng.random_range(0.01..0.04),
        })
    }

    fn build_workforce_reduction(&mut self, effective_date: NaiveDate) -> OrganizationalEventType {
        let departments = ["Finance", "Operations", "Sales", "Engineering", "HR"];
        let affected_count = self.rng.random_range(1..=3_usize);
        let mut affected = Vec::with_capacity(affected_count);
        for i in 0..affected_count {
            let idx = (self.rng.random_range(0..departments.len()) + i) % departments.len();
            let dept = departments[idx].to_string();
            if !affected.contains(&dept) {
                affected.push(dept);
            }
        }

        OrganizationalEventType::WorkforceReduction(WorkforceReductionConfig {
            reduction_date: effective_date,
            reduction_percent: self.rng.random_range(0.05..0.20),
            affected_departments: affected,
            error_rate_increase: self.rng.random_range(0.02..0.08),
            processing_time_increase: self.rng.random_range(1.1..1.5),
            transition_months: self.rng.random_range(3..9_u32),
            severance_costs: None,
        })
    }

    fn build_merger(
        &mut self,
        effective_date: NaiveDate,
        company_code: &str,
    ) -> OrganizationalEventType {
        let seq = self.event_counter;
        let entity_code = format!("MRG-{}-{:04}", company_code, seq);
        let volume_mult = self.rng.random_range(1.50..2.20);

        let cutover = effective_date + chrono::Duration::days(self.rng.random_range(30..90));
        let stabilization_end = cutover + chrono::Duration::days(self.rng.random_range(90..180));

        OrganizationalEventType::Merger(MergerConfig {
            merged_entity_code: entity_code,
            merged_entity_name: Some(format!("Merged Entity {}", seq)),
            merger_date: effective_date,
            volume_multiplier: volume_mult,
            integration_error_rate: self.rng.random_range(0.03..0.08),
            integration_phases: IntegrationPhaseConfig {
                parallel_run: Some(DateRange {
                    start: effective_date,
                    end: cutover - chrono::Duration::days(1),
                }),
                cutover_date: cutover,
                stabilization_end,
                parallel_run_error_rate: self.rng.random_range(0.05..0.12),
                stabilization_error_rate: self.rng.random_range(0.02..0.05),
            },
            fair_value_adjustments: Vec::new(),
            goodwill: None,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_generation() {
        let mut gen1 = OrganizationalEventGenerator::new(42);
        let mut gen2 = OrganizationalEventGenerator::new(42);
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let companies = vec!["C001".to_string(), "C002".to_string()];

        let events1 = gen1.generate_events(start, end, &companies);
        let events2 = gen2.generate_events(start, end, &companies);

        assert_eq!(events1.len(), events2.len());
        for (e1, e2) in events1.iter().zip(events2.iter()) {
            assert_eq!(e1.event_id, e2.event_id);
            assert_eq!(e1.effective_date, e2.effective_date);
            assert_eq!(e1.event_type.type_name(), e2.event_type.type_name());
        }
    }

    #[test]
    fn test_events_sorted_by_date() {
        let mut gen = OrganizationalEventGenerator::new(42);
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        let companies = vec!["C001".to_string()];

        let events = gen.generate_events(start, end, &companies);
        for w in events.windows(2) {
            assert!(w[0].effective_date <= w[1].effective_date);
        }
    }

    #[test]
    fn test_all_event_types_generated() {
        let config = OrgEventGeneratorConfig {
            type_weights: [1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            events_per_year: 100.0,
        };
        let mut gen = OrganizationalEventGenerator::with_config(42, config);
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let companies = vec!["C001".to_string()];

        let events = gen.generate_events(start, end, &companies);

        let has_acquisition = events
            .iter()
            .any(|e| matches!(e.event_type, OrganizationalEventType::Acquisition(_)));
        let has_divestiture = events
            .iter()
            .any(|e| matches!(e.event_type, OrganizationalEventType::Divestiture(_)));
        let has_reorg = events
            .iter()
            .any(|e| matches!(e.event_type, OrganizationalEventType::Reorganization(_)));
        let has_leadership = events
            .iter()
            .any(|e| matches!(e.event_type, OrganizationalEventType::LeadershipChange(_)));
        let has_workforce = events
            .iter()
            .any(|e| matches!(e.event_type, OrganizationalEventType::WorkforceReduction(_)));
        let has_merger = events
            .iter()
            .any(|e| matches!(e.event_type, OrganizationalEventType::Merger(_)));

        assert!(has_acquisition, "should generate acquisitions");
        assert!(has_divestiture, "should generate divestitures");
        assert!(has_reorg, "should generate reorganizations");
        assert!(has_leadership, "should generate leadership changes");
        assert!(has_workforce, "should generate workforce reductions");
        assert!(has_merger, "should generate mergers");
    }

    #[test]
    fn test_events_within_date_range() {
        let mut gen = OrganizationalEventGenerator::new(42);
        let start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let companies = vec!["C001".to_string()];

        let events = gen.generate_events(start, end, &companies);
        for e in &events {
            assert!(e.effective_date >= start, "event date before start");
            assert!(e.effective_date <= end, "event date after end");
        }
    }

    #[test]
    fn test_empty_company_codes() {
        let mut gen = OrganizationalEventGenerator::new(42);
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let events = gen.generate_events(start, end, &[]);
        assert!(!events.is_empty(), "should still generate events");
        // With empty company_codes, tags should use "C001" fallback
        for e in &events {
            assert!(
                e.tags.iter().any(|t| t == "company:C001"),
                "should use C001 fallback"
            );
        }
    }

    #[test]
    fn test_event_has_tags_and_description() {
        let mut gen = OrganizationalEventGenerator::new(99);
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let companies = vec!["ACME".to_string()];

        let events = gen.generate_events(start, end, &companies);
        for e in &events {
            assert!(e.description.is_some(), "event should have a description");
            assert!(!e.tags.is_empty(), "event should have tags");
            assert!(
                e.tags.iter().any(|t| t.starts_with("company:")),
                "should have company tag"
            );
            assert!(
                e.tags.iter().any(|t| t.starts_with("type:")),
                "should have type tag"
            );
        }
    }

    #[test]
    fn test_acquisition_config_populated() {
        let config = OrgEventGeneratorConfig {
            type_weights: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            events_per_year: 5.0,
        };
        let mut gen = OrganizationalEventGenerator::with_config(42, config);
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let companies = vec!["C001".to_string()];

        let events = gen.generate_events(start, end, &companies);
        for e in &events {
            if let OrganizationalEventType::Acquisition(ref acq) = e.event_type {
                assert!(!acq.acquired_entity_code.is_empty());
                assert!(acq.volume_multiplier >= 1.10);
                assert!(acq.integration_error_rate > 0.0);
                assert!(acq.integration_phases.parallel_run.is_some());
            } else {
                panic!("expected only acquisitions");
            }
        }
    }

    #[test]
    fn test_event_is_active_at_effective_date() {
        let mut gen = OrganizationalEventGenerator::new(42);
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let companies = vec!["C001".to_string()];

        let events = gen.generate_events(start, end, &companies);
        for e in &events {
            assert!(
                e.is_active_at(e.effective_date),
                "event should be active at its effective date"
            );
        }
    }
}
