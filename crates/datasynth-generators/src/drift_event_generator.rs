//! Generator for drift events (statistical, temporal, market, behavioral, organizational, process).
//!
//! Produces `LabeledDriftEvent` instances by:
//! 1. Observing organizational events and deriving organizational drift labels.
//! 2. Observing process evolution events and deriving process drift labels.
//! 3. Generating standalone statistical, temporal, market, and behavioral drifts.
//!
//! All generation is fully deterministic given the same seed.

use chrono::NaiveDate;
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

use datasynth_core::models::drift_events::{
    BehavioralDriftEvent, DetectionDifficulty, DriftEventType, LabeledDriftEvent, MarketDriftEvent,
    MarketEventType, OrganizationalDriftEvent, ProcessDriftEvent, StatisticalDriftEvent,
    StatisticalShiftType, TemporalDriftEvent, TemporalShiftType,
};
use datasynth_core::models::organizational_event::{OrganizationalEvent, OrganizationalEventType};
use datasynth_core::models::process_evolution::{ProcessEvolutionEvent, ProcessEvolutionType};

/// Configuration for the drift event generator.
#[derive(Debug, Clone)]
pub struct DriftEventGeneratorConfig {
    /// Average standalone drifts per year (statistical, temporal, market, behavioral).
    pub standalone_drifts_per_year: f64,
    /// Probability of generating a drift from an org event.
    pub org_event_drift_prob: f64,
    /// Probability of generating a drift from a process event.
    pub process_event_drift_prob: f64,
}

impl Default for DriftEventGeneratorConfig {
    fn default() -> Self {
        Self {
            standalone_drifts_per_year: 6.0,
            org_event_drift_prob: 0.8,
            process_event_drift_prob: 0.7,
        }
    }
}

/// Generates [`LabeledDriftEvent`] instances from organizational events,
/// process evolution events, and standalone random drifts.
pub struct DriftEventGenerator {
    rng: ChaCha8Rng,
    config: DriftEventGeneratorConfig,
    event_counter: usize,
}

/// Discriminator added to the seed so this generator's RNG stream does not
/// overlap with other generators that may share the same base seed.
const SEED_DISCRIMINATOR: u64 = 0xAE_0D;

impl DriftEventGenerator {
    /// Create a new generator with the given seed and default config.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, SEED_DISCRIMINATOR),
            config: DriftEventGeneratorConfig::default(),
            event_counter: 0,
        }
    }

    /// Create a new generator with the given seed and custom config.
    pub fn with_config(seed: u64, config: DriftEventGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, SEED_DISCRIMINATOR),
            config,
            event_counter: 0,
        }
    }

    /// Generate all drift events: from org events, process events, and standalone.
    ///
    /// Results are returned sorted by `start_date`.
    pub fn generate_all(
        &mut self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        org_events: &[OrganizationalEvent],
        proc_events: &[ProcessEvolutionEvent],
    ) -> Vec<LabeledDriftEvent> {
        let mut all = Vec::new();

        let mut from_org = self.generate_from_org_events(org_events);
        let mut from_proc = self.generate_from_process_events(proc_events);
        let mut standalone = self.generate_standalone_drifts(start_date, end_date);

        all.append(&mut from_org);
        all.append(&mut from_proc);
        all.append(&mut standalone);

        all.sort_by_key(|e| e.start_date);
        all
    }

    /// Generate drift events derived from organizational events.
    ///
    /// For each org event, with probability `config.org_event_drift_prob`, creates
    /// an Organizational drift label.
    pub fn generate_from_org_events(
        &mut self,
        org_events: &[OrganizationalEvent],
    ) -> Vec<LabeledDriftEvent> {
        let mut drifts = Vec::new();

        for org_event in org_events {
            if !self.rng.random_bool(self.config.org_event_drift_prob) {
                continue;
            }

            let event_id = self.next_event_id();

            let (detection_difficulty, magnitude) = match &org_event.event_type {
                OrganizationalEventType::Merger(_) | OrganizationalEventType::Acquisition(_) => {
                    let mag = self.rng.random_range(0.3..0.8);
                    (DetectionDifficulty::Easy, mag)
                }
                OrganizationalEventType::Reorganization(_)
                | OrganizationalEventType::WorkforceReduction(_) => {
                    let mag = self.rng.random_range(0.1..0.4);
                    (DetectionDifficulty::Medium, mag)
                }
                OrganizationalEventType::LeadershipChange(_) => {
                    let mag = self.rng.random_range(0.1..0.4);
                    (DetectionDifficulty::Hard, mag)
                }
                OrganizationalEventType::Divestiture(_) => {
                    let mag = self.rng.random_range(0.1..0.4);
                    (DetectionDifficulty::Medium, mag)
                }
            };

            let duration_days = self.rng.random_range(30..90_i64);
            let end_date = org_event.effective_date + chrono::Duration::days(duration_days);

            let affected_entities: Vec<String> = org_event
                .tags
                .iter()
                .filter(|t| t.starts_with("company:"))
                .cloned()
                .collect();

            let drift_type = DriftEventType::Organizational(OrganizationalDriftEvent {
                event_type: org_event.event_type.type_name().to_string(),
                related_event_id: org_event.event_id.clone(),
                detection_difficulty,
                affected_entities: affected_entities.clone(),
                impact_metrics: HashMap::new(),
            });

            let start_period = 0_u32;
            let end_period = (duration_days / 30) as u32;

            let mut labeled = LabeledDriftEvent::new(
                event_id,
                drift_type,
                org_event.effective_date,
                start_period,
                magnitude,
            );
            labeled.end_date = Some(end_date);
            labeled.end_period = Some(end_period);
            labeled.related_org_event = Some(org_event.event_id.clone());
            labeled.affected_fields = affected_entities;
            labeled.tags = vec![
                format!("source:organizational"),
                format!("org_type:{}", org_event.event_type.type_name()),
            ];

            drifts.push(labeled);
        }

        drifts
    }

    /// Generate drift events derived from process evolution events.
    ///
    /// For each process event, with probability `config.process_event_drift_prob`,
    /// creates a Process drift label.
    pub fn generate_from_process_events(
        &mut self,
        proc_events: &[ProcessEvolutionEvent],
    ) -> Vec<LabeledDriftEvent> {
        let mut drifts = Vec::new();

        for proc_event in proc_events {
            if !self.rng.random_bool(self.config.process_event_drift_prob) {
                continue;
            }

            let event_id = self.next_event_id();

            let detection_difficulty = match &proc_event.event_type {
                ProcessEvolutionType::ProcessAutomation(_)
                | ProcessEvolutionType::ApprovalWorkflowChange(_) => DetectionDifficulty::Medium,
                ProcessEvolutionType::PolicyChange(_)
                | ProcessEvolutionType::ControlEnhancement(_) => DetectionDifficulty::Hard,
            };

            let transition_months = proc_event.event_type.transition_months();
            let duration_days = (transition_months as i64) * 30;
            let end_date = proc_event.effective_date + chrono::Duration::days(duration_days);

            // Magnitude based on error_rate_impact, scaled to 0.1..0.6 range
            let raw_impact = proc_event.event_type.error_rate_impact().abs();
            let magnitude = (raw_impact * 6.0).clamp(0.1, 0.6);

            let drift_type = DriftEventType::Process(ProcessDriftEvent {
                process_type: proc_event.event_type.type_name().to_string(),
                related_event_id: proc_event.event_id.clone(),
                detection_difficulty,
                affected_processes: proc_event.tags.clone(),
            });

            let start_period = 0_u32;
            let end_period = transition_months;

            let mut labeled = LabeledDriftEvent::new(
                event_id,
                drift_type,
                proc_event.effective_date,
                start_period,
                magnitude,
            );
            labeled.end_date = Some(end_date);
            labeled.end_period = Some(end_period);
            labeled.tags = vec![
                "source:process".to_string(),
                format!("process_type:{}", proc_event.event_type.type_name()),
            ];

            drifts.push(labeled);
        }

        drifts
    }

    /// Generate standalone drifts (statistical, temporal, market, behavioral)
    /// randomly distributed across the date range.
    pub fn generate_standalone_drifts(
        &mut self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Vec<LabeledDriftEvent> {
        let total_days = (end_date - start_date).num_days().max(1) as f64;
        let total_years = total_days / 365.25;
        let expected_count =
            (self.config.standalone_drifts_per_year * total_years).round() as usize;
        let count = expected_count.max(1);

        let mut drifts = Vec::with_capacity(count);

        for _ in 0..count {
            let event_id = self.next_event_id();

            // Pick random start date within range
            let days_offset = self.rng.random_range(0..total_days as i64);
            let drift_start = start_date + chrono::Duration::days(days_offset);
            let duration_days = self.rng.random_range(30..180_i64);
            let drift_end = drift_start + chrono::Duration::days(duration_days);

            // Pick random category: 0=Statistical, 1=Temporal, 2=Market, 3=Behavioral
            let category = self.rng.random_range(0..4_u32);

            let (drift_type, magnitude) = match category {
                0 => self.build_statistical_drift(),
                1 => self.build_temporal_drift(),
                2 => self.build_market_drift(),
                _ => self.build_behavioral_drift(),
            };

            // Detection difficulty derived from magnitude
            let detection_difficulty = if magnitude > 0.3 {
                DetectionDifficulty::Easy
            } else if magnitude > 0.15 {
                DetectionDifficulty::Medium
            } else {
                DetectionDifficulty::Hard
            };

            let start_period = 0_u32;
            let end_period = (duration_days / 30) as u32;

            let mut labeled =
                LabeledDriftEvent::new(event_id, drift_type, drift_start, start_period, magnitude);
            labeled.end_date = Some(drift_end);
            labeled.end_period = Some(end_period);
            labeled.detection_difficulty = detection_difficulty;
            labeled.tags = vec!["source:standalone".to_string()];

            drifts.push(labeled);
        }

        drifts
    }

    // ------------------------------------------------------------------
    // Standalone drift type builders
    // ------------------------------------------------------------------

    fn build_statistical_drift(&mut self) -> (DriftEventType, f64) {
        let shift_types = [
            StatisticalShiftType::MeanShift,
            StatisticalShiftType::VarianceChange,
            StatisticalShiftType::DistributionChange,
            StatisticalShiftType::CorrelationChange,
            StatisticalShiftType::TailChange,
            StatisticalShiftType::BenfordDeviation,
        ];
        let idx = self.rng.random_range(0..shift_types.len());
        let shift_type = shift_types[idx];

        let fields = [
            "amount",
            "line_count",
            "processing_time",
            "approval_duration",
        ];
        let field_idx = self.rng.random_range(0..fields.len());
        let affected_field = fields[field_idx].to_string();

        let magnitude = self.rng.random_range(0.05..0.40);

        let detection_difficulty = if magnitude > 0.3 {
            DetectionDifficulty::Easy
        } else if magnitude > 0.15 {
            DetectionDifficulty::Medium
        } else {
            DetectionDifficulty::Hard
        };

        let drift_type = DriftEventType::Statistical(StatisticalDriftEvent {
            shift_type,
            affected_field,
            magnitude,
            detection_difficulty,
            metrics: HashMap::new(),
        });

        (drift_type, magnitude)
    }

    fn build_temporal_drift(&mut self) -> (DriftEventType, f64) {
        let shift_types = [
            TemporalShiftType::SeasonalityChange,
            TemporalShiftType::TrendChange,
            TemporalShiftType::PeriodicityChange,
            TemporalShiftType::IntradayChange,
            TemporalShiftType::LagChange,
        ];
        let idx = self.rng.random_range(0..shift_types.len());
        let shift_type = shift_types[idx];

        let magnitude = self.rng.random_range(0.10..0.50);

        let detection_difficulty = if magnitude > 0.3 {
            DetectionDifficulty::Easy
        } else if magnitude > 0.15 {
            DetectionDifficulty::Medium
        } else {
            DetectionDifficulty::Hard
        };

        let drift_type = DriftEventType::Temporal(TemporalDriftEvent {
            shift_type,
            affected_field: None,
            detection_difficulty,
            magnitude,
            description: None,
        });

        (drift_type, magnitude)
    }

    fn build_market_drift(&mut self) -> (DriftEventType, f64) {
        let market_types = [
            MarketEventType::EconomicCycle,
            MarketEventType::RecessionStart,
            MarketEventType::RecessionEnd,
            MarketEventType::PriceShock,
            MarketEventType::CommodityChange,
        ];
        let idx = self.rng.random_range(0..market_types.len());
        let market_type = market_types[idx];

        let magnitude = self.rng.random_range(0.10..0.60);

        let is_recession = matches!(
            market_type,
            MarketEventType::RecessionStart | MarketEventType::RecessionEnd
        );

        let detection_difficulty = if magnitude > 0.3 {
            DetectionDifficulty::Easy
        } else if magnitude > 0.15 {
            DetectionDifficulty::Medium
        } else {
            DetectionDifficulty::Hard
        };

        let drift_type = DriftEventType::Market(MarketDriftEvent {
            market_type,
            detection_difficulty,
            magnitude,
            is_recession,
            affected_sectors: Vec::new(),
        });

        (drift_type, magnitude)
    }

    fn build_behavioral_drift(&mut self) -> (DriftEventType, f64) {
        let behavior_types = [
            "vendor_quality",
            "customer_payment",
            "employee_productivity",
            "approval_pattern",
        ];
        let entity_types = ["vendor", "customer", "employee"];

        let bt_idx = self.rng.random_range(0..behavior_types.len());
        let et_idx = self.rng.random_range(0..entity_types.len());

        let behavior_type = behavior_types[bt_idx].to_string();
        let entity_type = entity_types[et_idx].to_string();

        let magnitude = self.rng.random_range(0.05..0.40);

        let detection_difficulty = if magnitude > 0.3 {
            DetectionDifficulty::Easy
        } else if magnitude > 0.15 {
            DetectionDifficulty::Medium
        } else {
            DetectionDifficulty::Hard
        };

        let drift_type = DriftEventType::Behavioral(BehavioralDriftEvent {
            behavior_type,
            entity_type,
            detection_difficulty,
            metrics: HashMap::new(),
        });

        (drift_type, magnitude)
    }

    // ------------------------------------------------------------------
    // Helper
    // ------------------------------------------------------------------

    fn next_event_id(&mut self) -> String {
        self.event_counter += 1;
        format!("DRIFT-{:06}", self.event_counter)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::organizational_event::{
        AcquisitionConfig, MergerConfig, OrganizationalEventType,
    };
    use datasynth_core::models::process_evolution::{
        ProcessAutomationConfig, ProcessEvolutionType,
    };

    fn make_org_events() -> Vec<OrganizationalEvent> {
        let acq = OrganizationalEvent {
            event_id: "ORG-001".to_string(),
            event_type: OrganizationalEventType::Acquisition(AcquisitionConfig {
                acquisition_date: NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
                ..Default::default()
            }),
            effective_date: NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            description: Some("Acquisition".to_string()),
            tags: vec!["company:C001".to_string(), "type:acquisition".to_string()],
        };

        let merger = OrganizationalEvent {
            event_id: "ORG-002".to_string(),
            event_type: OrganizationalEventType::Merger(MergerConfig {
                merger_date: NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
                ..Default::default()
            }),
            effective_date: NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            description: Some("Merger".to_string()),
            tags: vec!["company:C002".to_string(), "type:merger".to_string()],
        };

        vec![acq, merger]
    }

    fn make_proc_events() -> Vec<ProcessEvolutionEvent> {
        vec![
            ProcessEvolutionEvent::new(
                "PROC-001",
                ProcessEvolutionType::ProcessAutomation(ProcessAutomationConfig {
                    rollout_months: 6,
                    ..Default::default()
                }),
                NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            ),
            ProcessEvolutionEvent::new(
                "PROC-002",
                ProcessEvolutionType::ProcessAutomation(ProcessAutomationConfig {
                    rollout_months: 3,
                    ..Default::default()
                }),
                NaiveDate::from_ymd_opt(2024, 8, 1).unwrap(),
            ),
        ]
    }

    #[test]
    fn test_deterministic_generation() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let org = make_org_events();
        let proc = make_proc_events();

        let mut gen1 = DriftEventGenerator::new(42);
        let mut gen2 = DriftEventGenerator::new(42);

        let drifts1 = gen1.generate_all(start, end, &org, &proc);
        let drifts2 = gen2.generate_all(start, end, &org, &proc);

        assert_eq!(drifts1.len(), drifts2.len());
        for (d1, d2) in drifts1.iter().zip(drifts2.iter()) {
            assert_eq!(d1.event_id, d2.event_id);
            assert_eq!(d1.start_date, d2.start_date);
            assert!((d1.magnitude - d2.magnitude).abs() < 1e-10);
        }
    }

    #[test]
    fn test_drift_from_org_events() {
        let org = make_org_events();
        let config = DriftEventGeneratorConfig {
            org_event_drift_prob: 1.0, // Always generate
            ..Default::default()
        };
        let mut gen = DriftEventGenerator::with_config(42, config);

        let drifts = gen.generate_from_org_events(&org);

        // With prob=1.0, all org events should produce drifts
        assert_eq!(drifts.len(), org.len());

        for drift in &drifts {
            // Each drift should have related_org_event set
            assert!(drift.related_org_event.is_some());

            // related_org_event should match one of the org event IDs
            let related_id = drift.related_org_event.as_ref().unwrap();
            assert!(
                org.iter().any(|e| &e.event_id == related_id),
                "related_org_event should match an org event id"
            );

            // Event type should be Organizational
            assert_eq!(
                drift.event_type.category_name(),
                "organizational",
                "drift from org event should be Organizational category"
            );
        }
    }

    #[test]
    fn test_drift_from_process_events() {
        let proc = make_proc_events();
        let config = DriftEventGeneratorConfig {
            process_event_drift_prob: 1.0, // Always generate
            ..Default::default()
        };
        let mut gen = DriftEventGenerator::with_config(42, config);

        let drifts = gen.generate_from_process_events(&proc);

        // With prob=1.0, all process events should produce drifts
        assert_eq!(drifts.len(), proc.len());

        for drift in &drifts {
            assert_eq!(
                drift.event_type.category_name(),
                "process",
                "drift from process event should be Process category"
            );
        }
    }

    #[test]
    fn test_standalone_drifts() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let mut gen = DriftEventGenerator::new(42);
        let drifts = gen.generate_standalone_drifts(start, end);

        // With default 6 drifts/year and ~1 year range, we expect ~6 drifts
        assert!(!drifts.is_empty(), "should produce standalone drifts");
        assert!(
            drifts.len() >= 4,
            "should produce at least 4 standalone drifts"
        );
    }

    #[test]
    fn test_magnitude_in_valid_range() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let org = make_org_events();
        let proc = make_proc_events();

        let mut gen = DriftEventGenerator::new(42);
        let drifts = gen.generate_all(start, end, &org, &proc);

        for drift in &drifts {
            assert!(
                drift.magnitude >= 0.0 && drift.magnitude <= 1.0,
                "magnitude {} should be in [0.0, 1.0]",
                drift.magnitude
            );
        }
    }

    #[test]
    fn test_detection_difficulty_correlates_with_magnitude() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();

        let config = DriftEventGeneratorConfig {
            standalone_drifts_per_year: 100.0,
            org_event_drift_prob: 0.0,
            process_event_drift_prob: 0.0,
        };
        let mut gen = DriftEventGenerator::with_config(42, config);
        let drifts = gen.generate_standalone_drifts(start, end);

        // For standalone drifts, detection difficulty is set based on magnitude:
        // >0.3 -> Easy, >0.15 -> Medium, else Hard
        for drift in &drifts {
            if drift.magnitude > 0.3 {
                assert_eq!(
                    drift.detection_difficulty,
                    DetectionDifficulty::Easy,
                    "magnitude {} should be Easy",
                    drift.magnitude
                );
            } else if drift.magnitude > 0.15 {
                assert_eq!(
                    drift.detection_difficulty,
                    DetectionDifficulty::Medium,
                    "magnitude {} should be Medium",
                    drift.magnitude
                );
            } else {
                assert_eq!(
                    drift.detection_difficulty,
                    DetectionDifficulty::Hard,
                    "magnitude {} should be Hard",
                    drift.magnitude
                );
            }
        }
    }

    #[test]
    fn test_all_standalone_categories() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let config = DriftEventGeneratorConfig {
            standalone_drifts_per_year: 60.0,
            org_event_drift_prob: 0.0,
            process_event_drift_prob: 0.0,
        };
        let mut gen = DriftEventGenerator::with_config(42, config);
        let drifts = gen.generate_standalone_drifts(start, end);

        let has_statistical = drifts
            .iter()
            .any(|d| d.event_type.category_name() == "statistical");
        let has_temporal = drifts
            .iter()
            .any(|d| d.event_type.category_name() == "temporal");
        let has_market = drifts
            .iter()
            .any(|d| d.event_type.category_name() == "market");
        let has_behavioral = drifts
            .iter()
            .any(|d| d.event_type.category_name() == "behavioral");

        assert!(has_statistical, "should generate statistical drifts");
        assert!(has_temporal, "should generate temporal drifts");
        assert!(has_market, "should generate market drifts");
        assert!(has_behavioral, "should generate behavioral drifts");
    }
}
