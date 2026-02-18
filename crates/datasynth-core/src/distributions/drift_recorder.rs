//! Drift label recorder for ground truth generation.
//!
//! Records drift events during data generation for use as
//! ground truth labels in ML model training and evaluation.

use crate::distributions::drift::{DriftAdjustments, RegimeChange, RegimeChangeType};
use crate::models::drift_events::{
    CategoricalDriftEvent, CategoricalShiftType, DetectionDifficulty, DriftEventType,
    LabeledDriftEvent, MarketDriftEvent, MarketEventType, OrganizationalDriftEvent,
    ProcessDriftEvent, StatisticalDriftEvent, StatisticalShiftType, TechnologyDriftEvent,
    TemporalDriftEvent, TemporalShiftType,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

/// Configuration for drift recording.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftRecorderConfig {
    /// Enable recording.
    #[serde(default)]
    pub enabled: bool,
    /// Record statistical drift events.
    #[serde(default = "default_true")]
    pub statistical: bool,
    /// Record categorical drift events.
    #[serde(default = "default_true")]
    pub categorical: bool,
    /// Record temporal drift events.
    #[serde(default = "default_true")]
    pub temporal: bool,
    /// Record organizational events.
    #[serde(default = "default_true")]
    pub organizational: bool,
    /// Record process events.
    #[serde(default = "default_true")]
    pub process_events: bool,
    /// Record technology events.
    #[serde(default = "default_true")]
    pub technology_events: bool,
    /// Record regulatory events.
    #[serde(default = "default_true")]
    pub regulatory: bool,
    /// Record market events.
    #[serde(default = "default_true")]
    pub market: bool,
    /// Record behavioral events.
    #[serde(default = "default_true")]
    pub behavioral: bool,
    /// Minimum magnitude threshold to record.
    #[serde(default = "default_min_magnitude")]
    pub min_magnitude_threshold: f64,
}

fn default_true() -> bool {
    true
}

fn default_min_magnitude() -> f64 {
    0.05
}

impl Default for DriftRecorderConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            statistical: true,
            categorical: true,
            temporal: true,
            organizational: true,
            process_events: true,
            technology_events: true,
            regulatory: true,
            market: true,
            behavioral: true,
            min_magnitude_threshold: 0.05,
        }
    }
}

/// Drift label recorder.
pub struct DriftLabelRecorder {
    /// Recorded events.
    events: Vec<LabeledDriftEvent>,
    /// Configuration.
    config: DriftRecorderConfig,
    /// Start date of the simulation.
    start_date: NaiveDate,
    /// Event ID counter.
    event_counter: u64,
    /// Track previous drift state for delta detection.
    previous_drift: Option<DriftAdjustments>,
    /// Track if in recession (for recession end detection).
    was_in_recession: bool,
}

impl DriftLabelRecorder {
    /// Create a new drift label recorder.
    pub fn new(config: DriftRecorderConfig, start_date: NaiveDate) -> Self {
        Self {
            events: Vec::new(),
            config,
            start_date,
            event_counter: 0,
            previous_drift: None,
            was_in_recession: false,
        }
    }

    /// Check if recording is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Generate a unique event ID.
    fn next_event_id(&mut self) -> String {
        self.event_counter += 1;
        format!("DRIFT-{:06}", self.event_counter)
    }

    /// Convert a period to a date.
    fn period_to_date(&self, period: u32) -> NaiveDate {
        self.start_date + chrono::Duration::days(period as i64 * 30)
    }

    /// Record a regime change event.
    pub fn record_regime_change(&mut self, regime: &RegimeChange, period: u32, _date: NaiveDate) {
        if !self.config.enabled || !self.config.organizational {
            return;
        }

        let event_type = match regime.change_type {
            RegimeChangeType::Acquisition => "acquisition",
            RegimeChangeType::Divestiture => "divestiture",
            RegimeChangeType::PriceIncrease => "price_increase",
            RegimeChangeType::PriceDecrease => "price_decrease",
            RegimeChangeType::ProductLaunch => "product_launch",
            RegimeChangeType::ProductDiscontinuation => "product_discontinuation",
            RegimeChangeType::PolicyChange => "policy_change",
            RegimeChangeType::CompetitorEntry => "competitor_entry",
            RegimeChangeType::Custom => "custom",
        };

        let magnitude = (regime.volume_multiplier() - 1.0)
            .abs()
            .max((regime.amount_mean_multiplier() - 1.0).abs());

        if magnitude < self.config.min_magnitude_threshold {
            return;
        }

        let detection_difficulty = if magnitude > 0.20 {
            DetectionDifficulty::Easy
        } else if magnitude > 0.10 {
            DetectionDifficulty::Medium
        } else {
            DetectionDifficulty::Hard
        };

        let mut event = LabeledDriftEvent::new(
            self.next_event_id(),
            DriftEventType::Organizational(OrganizationalDriftEvent {
                event_type: event_type.to_string(),
                related_event_id: regime.description.clone().unwrap_or_default(),
                detection_difficulty,
                affected_entities: Vec::new(),
                impact_metrics: {
                    let mut m = HashMap::new();
                    m.insert("volume_multiplier".to_string(), regime.volume_multiplier());
                    m.insert(
                        "amount_multiplier".to_string(),
                        regime.amount_mean_multiplier(),
                    );
                    m
                },
            }),
            self.period_to_date(period),
            period,
            magnitude,
        );

        event.end_period = Some(period + regime.transition_periods);
        event.tags.push("regime_change".to_string());
        event.tags.push(event_type.to_string());

        self.events.push(event);
    }

    /// Record statistical drift from drift adjustments.
    pub fn record_statistical_drift(&mut self, adjustments: &DriftAdjustments, period: u32) {
        if !self.config.enabled || !self.config.statistical {
            return;
        }

        let date = self.period_to_date(period);

        // Check for mean shift - extract values before borrowing self mutably
        if let Some(ref prev) = self.previous_drift {
            let mean_delta =
                (adjustments.amount_mean_multiplier - prev.amount_mean_multiplier).abs();
            let var_delta =
                (adjustments.amount_variance_multiplier - prev.amount_variance_multiplier).abs();
            let prev_mean = prev.amount_mean_multiplier;
            let current_mean = adjustments.amount_mean_multiplier;
            let min_threshold = self.config.min_magnitude_threshold;

            if mean_delta >= min_threshold {
                let detection_difficulty = if mean_delta > 0.20 {
                    DetectionDifficulty::Easy
                } else if mean_delta > 0.10 {
                    DetectionDifficulty::Medium
                } else {
                    DetectionDifficulty::Hard
                };

                let event_id = self.next_event_id();
                let event = LabeledDriftEvent::new(
                    event_id,
                    DriftEventType::Statistical(StatisticalDriftEvent {
                        shift_type: StatisticalShiftType::MeanShift,
                        affected_field: "amount".to_string(),
                        magnitude: mean_delta,
                        detection_difficulty,
                        metrics: {
                            let mut m = HashMap::new();
                            m.insert("previous_multiplier".to_string(), prev_mean);
                            m.insert("current_multiplier".to_string(), current_mean);
                            m
                        },
                    }),
                    date,
                    period,
                    mean_delta,
                );

                self.events.push(event);
            }

            // Check for variance change
            if var_delta >= min_threshold {
                let event_id = self.next_event_id();
                let event = LabeledDriftEvent::new(
                    event_id,
                    DriftEventType::Statistical(StatisticalDriftEvent {
                        shift_type: StatisticalShiftType::VarianceChange,
                        affected_field: "amount".to_string(),
                        magnitude: var_delta,
                        detection_difficulty: DetectionDifficulty::Medium,
                        metrics: HashMap::new(),
                    }),
                    date,
                    period,
                    var_delta,
                );

                self.events.push(event);
            }
        }

        // Check for sudden drift
        if adjustments.sudden_drift_occurred {
            let event = LabeledDriftEvent::new(
                self.next_event_id(),
                DriftEventType::Statistical(StatisticalDriftEvent {
                    shift_type: StatisticalShiftType::DistributionChange,
                    affected_field: "amount".to_string(),
                    magnitude: 0.5, // Sudden drifts are typically significant
                    detection_difficulty: DetectionDifficulty::Easy,
                    metrics: HashMap::new(),
                }),
                date,
                period,
                0.5,
            );

            self.events.push(event);
        }

        self.previous_drift = Some(adjustments.clone());
    }

    /// Record a market/economic drift event.
    pub fn record_market_drift(
        &mut self,
        market_type: MarketEventType,
        period: u32,
        magnitude: f64,
        is_recession: bool,
    ) {
        if !self.config.enabled || !self.config.market {
            return;
        }

        if magnitude < self.config.min_magnitude_threshold
            && market_type != MarketEventType::RecessionStart
            && market_type != MarketEventType::RecessionEnd
        {
            return;
        }

        // Detect recession transitions
        let actual_type = if is_recession && !self.was_in_recession {
            self.was_in_recession = true;
            MarketEventType::RecessionStart
        } else if !is_recession && self.was_in_recession {
            self.was_in_recession = false;
            MarketEventType::RecessionEnd
        } else {
            market_type
        };

        let detection_difficulty = match actual_type {
            MarketEventType::RecessionStart | MarketEventType::RecessionEnd => {
                DetectionDifficulty::Easy
            }
            MarketEventType::PriceShock => DetectionDifficulty::Easy,
            MarketEventType::EconomicCycle => DetectionDifficulty::Medium,
            MarketEventType::CommodityChange => DetectionDifficulty::Medium,
        };

        let event = LabeledDriftEvent::new(
            self.next_event_id(),
            DriftEventType::Market(MarketDriftEvent {
                market_type: actual_type,
                detection_difficulty,
                magnitude,
                is_recession,
                affected_sectors: Vec::new(),
            }),
            self.period_to_date(period),
            period,
            magnitude,
        );

        self.events.push(event);
    }

    /// Record a process evolution drift event.
    pub fn record_process_drift(
        &mut self,
        process_type: &str,
        related_event_id: &str,
        period: u32,
        magnitude: f64,
        affected_processes: Vec<String>,
    ) {
        if !self.config.enabled || !self.config.process_events {
            return;
        }

        if magnitude < self.config.min_magnitude_threshold {
            return;
        }

        let mut event = LabeledDriftEvent::new(
            self.next_event_id(),
            DriftEventType::Process(ProcessDriftEvent {
                process_type: process_type.to_string(),
                related_event_id: related_event_id.to_string(),
                detection_difficulty: DetectionDifficulty::Medium,
                affected_processes,
            }),
            self.period_to_date(period),
            period,
            magnitude,
        );

        event.related_org_event = Some(related_event_id.to_string());
        self.events.push(event);
    }

    /// Record a technology transition drift event.
    pub fn record_technology_drift(
        &mut self,
        transition_type: &str,
        related_event_id: &str,
        period: u32,
        magnitude: f64,
        systems: Vec<String>,
        current_phase: Option<&str>,
    ) {
        if !self.config.enabled || !self.config.technology_events {
            return;
        }

        if magnitude < self.config.min_magnitude_threshold {
            return;
        }

        let mut event = LabeledDriftEvent::new(
            self.next_event_id(),
            DriftEventType::Technology(TechnologyDriftEvent {
                transition_type: transition_type.to_string(),
                related_event_id: related_event_id.to_string(),
                detection_difficulty: DetectionDifficulty::Easy, // Tech transitions are usually obvious
                systems,
                current_phase: current_phase.map(String::from),
            }),
            self.period_to_date(period),
            period,
            magnitude,
        );

        event.related_org_event = Some(related_event_id.to_string());
        self.events.push(event);
    }

    /// Record a temporal pattern drift event.
    pub fn record_temporal_drift(
        &mut self,
        shift_type: TemporalShiftType,
        period: u32,
        magnitude: f64,
        affected_field: Option<&str>,
        description: Option<&str>,
    ) {
        if !self.config.enabled || !self.config.temporal {
            return;
        }

        if magnitude < self.config.min_magnitude_threshold {
            return;
        }

        let event = LabeledDriftEvent::new(
            self.next_event_id(),
            DriftEventType::Temporal(TemporalDriftEvent {
                shift_type,
                affected_field: affected_field.map(String::from),
                detection_difficulty: DetectionDifficulty::Hard, // Temporal drifts are subtle
                magnitude,
                description: description.map(String::from),
            }),
            self.period_to_date(period),
            period,
            magnitude,
        );

        self.events.push(event);
    }

    /// Record a categorical drift event.
    pub fn record_categorical_drift(
        &mut self,
        shift_type: CategoricalShiftType,
        affected_field: &str,
        period: u32,
        proportions_before: HashMap<String, f64>,
        proportions_after: HashMap<String, f64>,
    ) {
        if !self.config.enabled || !self.config.categorical {
            return;
        }

        // Calculate magnitude as max proportion change
        let magnitude = proportions_before
            .keys()
            .chain(proportions_after.keys())
            .map(|k| {
                let before = proportions_before.get(k).copied().unwrap_or(0.0);
                let after = proportions_after.get(k).copied().unwrap_or(0.0);
                (after - before).abs()
            })
            .fold(0.0f64, f64::max);

        if magnitude < self.config.min_magnitude_threshold {
            return;
        }

        let new_categories: Vec<String> = proportions_after
            .keys()
            .filter(|k| !proportions_before.contains_key(*k))
            .cloned()
            .collect();

        let removed_categories: Vec<String> = proportions_before
            .keys()
            .filter(|k| !proportions_after.contains_key(*k))
            .cloned()
            .collect();

        let event = LabeledDriftEvent::new(
            self.next_event_id(),
            DriftEventType::Categorical(CategoricalDriftEvent {
                shift_type,
                affected_field: affected_field.to_string(),
                detection_difficulty: DetectionDifficulty::Medium,
                proportions_before,
                proportions_after,
                new_categories,
                removed_categories,
            }),
            self.period_to_date(period),
            period,
            magnitude,
        );

        self.events.push(event);
    }

    /// Get all recorded events.
    pub fn events(&self) -> &[LabeledDriftEvent] {
        &self.events
    }

    /// Get events in a specific period range.
    pub fn events_in_range(&self, start_period: u32, end_period: u32) -> Vec<&LabeledDriftEvent> {
        self.events
            .iter()
            .filter(|e| e.start_period >= start_period && e.start_period <= end_period)
            .collect()
    }

    /// Get events by category.
    pub fn events_by_category(&self, category: &str) -> Vec<&LabeledDriftEvent> {
        self.events
            .iter()
            .filter(|e| e.event_type.category_name() == category)
            .collect()
    }

    /// Get total event count.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Export events to CSV file.
    pub fn export_to_csv(&self, path: &Path) -> std::io::Result<usize> {
        let mut file = std::fs::File::create(path)?;

        // Write header
        writeln!(
            file,
            "event_id,category,type,start_date,end_date,start_period,end_period,magnitude,detection_difficulty,affected_fields,tags"
        )?;

        // Write events
        for event in &self.events {
            let end_date = event.end_date.map(|d| d.to_string()).unwrap_or_default();
            let end_period = event.end_period.map(|p| p.to_string()).unwrap_or_default();
            let affected_fields = event.affected_fields.join(";");
            let tags = event.tags.join(";");

            writeln!(
                file,
                "{},{},{},{},{},{},{},{:.4},{:?},{},{}",
                event.event_id,
                event.event_type.category_name(),
                event.event_type.type_name(),
                event.start_date,
                end_date,
                event.start_period,
                end_period,
                event.magnitude,
                event.detection_difficulty,
                affected_fields,
                tags
            )?;
        }

        Ok(self.events.len())
    }

    /// Export events to JSON file.
    pub fn export_to_json(&self, path: &Path) -> std::io::Result<usize> {
        let json = serde_json::to_string_pretty(&self.events).map_err(std::io::Error::other)?;
        std::fs::write(path, json)?;
        Ok(self.events.len())
    }

    /// Get summary statistics.
    pub fn summary(&self) -> DriftRecorderSummary {
        let mut by_category: HashMap<String, usize> = HashMap::new();
        let mut by_difficulty: HashMap<String, usize> = HashMap::new();
        let mut total_magnitude = 0.0;

        for event in &self.events {
            *by_category
                .entry(event.event_type.category_name().to_string())
                .or_insert(0) += 1;
            *by_difficulty
                .entry(format!("{:?}", event.detection_difficulty))
                .or_insert(0) += 1;
            total_magnitude += event.magnitude;
        }

        DriftRecorderSummary {
            total_events: self.events.len(),
            by_category,
            by_difficulty,
            avg_magnitude: if self.events.is_empty() {
                0.0
            } else {
                total_magnitude / self.events.len() as f64
            },
        }
    }
}

/// Summary statistics for drift recording.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftRecorderSummary {
    /// Total number of events.
    pub total_events: usize,
    /// Events by category.
    pub by_category: HashMap<String, usize>,
    /// Events by detection difficulty.
    pub by_difficulty: HashMap<String, usize>,
    /// Average magnitude.
    pub avg_magnitude: f64,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_recorder_creation() {
        let config = DriftRecorderConfig {
            enabled: true,
            ..Default::default()
        };
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let recorder = DriftLabelRecorder::new(config, start);

        assert!(recorder.is_enabled());
        assert_eq!(recorder.event_count(), 0);
    }

    #[test]
    fn test_record_regime_change() {
        let config = DriftRecorderConfig {
            enabled: true,
            min_magnitude_threshold: 0.0,
            ..Default::default()
        };
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let mut recorder = DriftLabelRecorder::new(config, start);

        let regime = RegimeChange::new(6, RegimeChangeType::Acquisition);
        recorder.record_regime_change(&regime, 6, start);

        assert_eq!(recorder.event_count(), 1);
        let event = &recorder.events()[0];
        assert_eq!(event.event_type.category_name(), "organizational");
    }

    #[test]
    fn test_record_statistical_drift() {
        let config = DriftRecorderConfig {
            enabled: true,
            min_magnitude_threshold: 0.01, // Low but not zero to avoid edge case
            ..Default::default()
        };
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let mut recorder = DriftLabelRecorder::new(config, start);

        // First call establishes baseline
        let adj1 = DriftAdjustments {
            amount_mean_multiplier: 1.0,
            ..DriftAdjustments::none()
        };
        recorder.record_statistical_drift(&adj1, 0);

        // Second call detects drift (mean shift of 0.25 > threshold of 0.01)
        let adj2 = DriftAdjustments {
            amount_mean_multiplier: 1.25,
            ..DriftAdjustments::none()
        };
        recorder.record_statistical_drift(&adj2, 1);

        // Only mean shift should be recorded (variance delta is 0)
        assert_eq!(recorder.event_count(), 1);
    }

    #[test]
    fn test_summary() {
        let config = DriftRecorderConfig {
            enabled: true,
            min_magnitude_threshold: 0.0,
            ..Default::default()
        };
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let mut recorder = DriftLabelRecorder::new(config, start);

        let regime = RegimeChange::new(6, RegimeChangeType::Acquisition);
        recorder.record_regime_change(&regime, 6, start);

        let summary = recorder.summary();
        assert_eq!(summary.total_events, 1);
        assert!(summary.by_category.contains_key("organizational"));
    }
}
