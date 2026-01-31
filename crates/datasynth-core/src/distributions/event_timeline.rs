//! Event timeline orchestrator for pattern drift simulation.
//!
//! Provides a unified controller that manages organizational events,
//! process evolution, and technology transitions, computing their
//! combined effects on data generation.

use crate::distributions::drift::{DriftAdjustments, DriftConfig, DriftController};
use crate::models::{
    organizational_event::{OrganizationalEvent, OrganizationalEventType},
    process_evolution::{ProcessEvolutionEvent, ProcessEvolutionType},
    technology_transition::{TechnologyTransitionEvent, TechnologyTransitionType},
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Effect blending mode for combining multiple event effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EffectBlendingMode {
    /// Multiply effects together.
    #[default]
    Multiplicative,
    /// Add effects together.
    Additive,
    /// Take the maximum effect.
    Maximum,
    /// Take the minimum effect.
    Minimum,
}

/// Timeline effects computed for a specific date.
#[derive(Debug, Clone, Default)]
pub struct TimelineEffects {
    /// Drift adjustments from base drift controller.
    pub drift: DriftAdjustments,
    /// Volume multiplier from all events.
    pub volume_multiplier: f64,
    /// Amount multiplier from all events.
    pub amount_multiplier: f64,
    /// Error rate delta (additive).
    pub error_rate_delta: f64,
    /// Processing time multiplier.
    pub processing_time_multiplier: f64,
    /// Entity changes (additions, removals).
    pub entity_changes: EntityChanges,
    /// Account remapping (old account -> new account).
    pub account_remapping: HashMap<String, String>,
    /// Control changes active.
    pub control_changes: ControlChanges,
    /// Special entries to generate (e.g., goodwill, fair value adjustments).
    pub special_entries: Vec<SpecialEntryRequest>,
    /// Active organizational events.
    pub active_org_events: Vec<String>,
    /// Active process events.
    pub active_process_events: Vec<String>,
    /// Active technology events.
    pub active_tech_events: Vec<String>,
    /// Whether in parallel posting mode (dual system entry).
    pub in_parallel_posting: bool,
    /// Current ERP migration phase if applicable.
    pub migration_phase: Option<String>,
}

impl TimelineEffects {
    /// Create a new effects instance with neutral values.
    pub fn neutral() -> Self {
        Self {
            drift: DriftAdjustments::none(),
            volume_multiplier: 1.0,
            amount_multiplier: 1.0,
            error_rate_delta: 0.0,
            processing_time_multiplier: 1.0,
            entity_changes: EntityChanges::default(),
            account_remapping: HashMap::new(),
            control_changes: ControlChanges::default(),
            special_entries: Vec::new(),
            active_org_events: Vec::new(),
            active_process_events: Vec::new(),
            active_tech_events: Vec::new(),
            in_parallel_posting: false,
            migration_phase: None,
        }
    }

    /// Get the combined volume multiplier including drift.
    pub fn combined_volume_multiplier(&self) -> f64 {
        self.volume_multiplier * self.drift.combined_volume_multiplier()
    }

    /// Get the combined amount multiplier including drift.
    pub fn combined_amount_multiplier(&self) -> f64 {
        self.amount_multiplier * self.drift.combined_amount_multiplier()
    }

    /// Get the total error rate (base + delta).
    pub fn total_error_rate(&self, base_error_rate: f64) -> f64 {
        (base_error_rate + self.error_rate_delta).clamp(0.0, 1.0)
    }

    /// Check if any events are active.
    pub fn has_active_events(&self) -> bool {
        !self.active_org_events.is_empty()
            || !self.active_process_events.is_empty()
            || !self.active_tech_events.is_empty()
    }
}

/// Entity changes from organizational events.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityChanges {
    /// Entities added (e.g., from acquisition).
    pub entities_added: Vec<String>,
    /// Entities removed (e.g., from divestiture).
    pub entities_removed: Vec<String>,
    /// Cost center remapping.
    pub cost_center_remapping: HashMap<String, String>,
    /// Department remapping.
    pub department_remapping: HashMap<String, String>,
}

impl EntityChanges {
    /// Check if there are any changes.
    pub fn has_changes(&self) -> bool {
        !self.entities_added.is_empty()
            || !self.entities_removed.is_empty()
            || !self.cost_center_remapping.is_empty()
            || !self.department_remapping.is_empty()
    }
}

/// Control changes from process and organizational events.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ControlChanges {
    /// New controls added.
    pub controls_added: Vec<String>,
    /// Controls modified.
    pub controls_modified: Vec<String>,
    /// Controls removed or deprecated.
    pub controls_removed: Vec<String>,
    /// Threshold changes (control_id -> (old, new)).
    pub threshold_changes: HashMap<String, (f64, f64)>,
}

impl ControlChanges {
    /// Check if there are any changes.
    pub fn has_changes(&self) -> bool {
        !self.controls_added.is_empty()
            || !self.controls_modified.is_empty()
            || !self.controls_removed.is_empty()
            || !self.threshold_changes.is_empty()
    }
}

/// Request for a special journal entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialEntryRequest {
    /// Entry type (e.g., "goodwill", "fair_value_adjustment", "severance").
    pub entry_type: String,
    /// Description.
    pub description: String,
    /// Debit account.
    pub debit_account: String,
    /// Credit account.
    pub credit_account: String,
    /// Amount (if known).
    pub amount: Option<rust_decimal::Decimal>,
    /// Related event ID.
    pub related_event_id: String,
}

/// Summary of active events at a point in time.
#[derive(Debug, Clone, Default)]
pub struct ActiveEventsSummary {
    /// Active organizational events.
    pub org_events: Vec<ActiveEventInfo>,
    /// Active process events.
    pub process_events: Vec<ActiveEventInfo>,
    /// Active technology events.
    pub tech_events: Vec<ActiveEventInfo>,
}

/// Information about an active event.
#[derive(Debug, Clone)]
pub struct ActiveEventInfo {
    /// Event ID.
    pub event_id: String,
    /// Event type name.
    pub event_type: String,
    /// Progress through the event (0.0 to 1.0).
    pub progress: f64,
}

/// Configuration for the event timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTimelineConfig {
    /// Organizational events.
    #[serde(default)]
    pub org_events: Vec<OrganizationalEvent>,
    /// Process evolution events.
    #[serde(default)]
    pub process_events: Vec<ProcessEvolutionEvent>,
    /// Technology transition events.
    #[serde(default)]
    pub tech_events: Vec<TechnologyTransitionEvent>,
    /// Effect blending mode.
    #[serde(default)]
    pub effect_blending: EffectBlendingMode,
    /// Base drift configuration.
    #[serde(default)]
    pub drift_config: DriftConfig,
}

impl Default for EventTimelineConfig {
    fn default() -> Self {
        Self {
            org_events: Vec::new(),
            process_events: Vec::new(),
            tech_events: Vec::new(),
            effect_blending: EffectBlendingMode::Multiplicative,
            drift_config: DriftConfig::default(),
        }
    }
}

/// Event timeline controller that orchestrates all events.
pub struct EventTimeline {
    /// Organizational events.
    org_events: Vec<OrganizationalEvent>,
    /// Process evolution events.
    process_events: Vec<ProcessEvolutionEvent>,
    /// Technology transition events.
    tech_events: Vec<TechnologyTransitionEvent>,
    /// Drift controller for base drift patterns.
    drift_controller: DriftController,
    /// Effect blending mode.
    effect_blending: EffectBlendingMode,
    /// Start date of the simulation.
    start_date: NaiveDate,
}

impl EventTimeline {
    /// Create a new event timeline.
    pub fn new(
        config: EventTimelineConfig,
        seed: u64,
        total_periods: u32,
        start_date: NaiveDate,
    ) -> Self {
        Self {
            org_events: config.org_events,
            process_events: config.process_events,
            tech_events: config.tech_events,
            drift_controller: DriftController::new(config.drift_config, seed, total_periods),
            effect_blending: config.effect_blending,
            start_date,
        }
    }

    /// Compute effects for a specific date.
    pub fn compute_effects_for_date(&self, date: NaiveDate) -> TimelineEffects {
        let period = self.date_to_period(date);
        let mut effects = TimelineEffects::neutral();

        // Get base drift adjustments
        effects.drift = self.drift_controller.compute_adjustments(period);

        // Initialize multipliers
        effects.volume_multiplier = 1.0;
        effects.amount_multiplier = 1.0;
        effects.processing_time_multiplier = 1.0;

        // Process organizational events
        for event in &self.org_events {
            if event.is_active_at(date) {
                self.apply_org_event_effects(&mut effects, event, date);
            }
        }

        // Process evolution events
        for event in &self.process_events {
            if event.is_active_at(date) {
                self.apply_process_event_effects(&mut effects, event, date);
            }
        }

        // Process technology events
        for event in &self.tech_events {
            if event.is_active_at(date) {
                self.apply_tech_event_effects(&mut effects, event, date);
            }
        }

        effects
    }

    /// Compute effects for a specific period.
    pub fn compute_effects_for_period(&self, period: u32) -> TimelineEffects {
        let date = self.period_to_date(period);
        self.compute_effects_for_date(date)
    }

    /// Get active events at a specific period.
    pub fn active_events_at(&self, period: u32) -> ActiveEventsSummary {
        let date = self.period_to_date(period);
        let mut summary = ActiveEventsSummary::default();

        for event in &self.org_events {
            if event.is_active_at(date) {
                summary.org_events.push(ActiveEventInfo {
                    event_id: event.event_id.clone(),
                    event_type: event.event_type.type_name().to_string(),
                    progress: event.progress_at(date),
                });
            }
        }

        for event in &self.process_events {
            if event.is_active_at(date) {
                summary.process_events.push(ActiveEventInfo {
                    event_id: event.event_id.clone(),
                    event_type: event.event_type.type_name().to_string(),
                    progress: event.progress_at(date),
                });
            }
        }

        for event in &self.tech_events {
            if event.is_active_at(date) {
                summary.tech_events.push(ActiveEventInfo {
                    event_id: event.event_id.clone(),
                    event_type: event.event_type.type_name().to_string(),
                    progress: event.progress_at(date),
                });
            }
        }

        summary
    }

    /// Check if in parallel run mode at a given date.
    pub fn in_parallel_run(&self, date: NaiveDate) -> Option<&TechnologyTransitionEvent> {
        for event in &self.tech_events {
            if let TechnologyTransitionType::ErpMigration(config) = &event.event_type {
                if let Some(parallel_start) = config.phases.parallel_run_start {
                    if date >= parallel_start && date < config.phases.cutover_date {
                        return Some(event);
                    }
                }
            }
        }
        None
    }

    /// Get the drift controller.
    pub fn drift_controller(&self) -> &DriftController {
        &self.drift_controller
    }

    /// Convert a date to a period number.
    fn date_to_period(&self, date: NaiveDate) -> u32 {
        let days = (date - self.start_date).num_days();
        (days / 30).max(0) as u32
    }

    /// Convert a period number to a date.
    fn period_to_date(&self, period: u32) -> NaiveDate {
        self.start_date + chrono::Duration::days(period as i64 * 30)
    }

    /// Apply effects from an organizational event.
    fn apply_org_event_effects(
        &self,
        effects: &mut TimelineEffects,
        event: &OrganizationalEvent,
        date: NaiveDate,
    ) {
        let progress = event.progress_at(date);
        effects.active_org_events.push(event.event_id.clone());

        match &event.event_type {
            OrganizationalEventType::Acquisition(config) => {
                // Volume and amount multipliers scale with progress
                let vol_mult = 1.0 + (config.volume_multiplier - 1.0) * progress;
                self.blend_multiplier(&mut effects.volume_multiplier, vol_mult);

                // Error rate during integration
                let error_rate = config.integration_phases.error_rate_at(date);
                effects.error_rate_delta += error_rate;

                // Add acquired entity
                if progress >= 0.0 {
                    effects
                        .entity_changes
                        .entities_added
                        .push(config.acquired_entity_code.clone());
                }

                // Check for parallel posting
                if config.parallel_posting_days > 0 {
                    let parallel_end = config.acquisition_date
                        + chrono::Duration::days(config.parallel_posting_days as i64);
                    if date >= config.acquisition_date && date <= parallel_end {
                        effects.in_parallel_posting = true;
                    }
                }

                // Special entries for purchase price allocation
                if let Some(ppa) = &config.purchase_price_allocation {
                    if progress < 0.1 {
                        // Only at the start
                        effects.special_entries.push(SpecialEntryRequest {
                            entry_type: "goodwill".to_string(),
                            description: format!(
                                "Goodwill from acquisition of {}",
                                config.acquired_entity_code
                            ),
                            debit_account: "1800".to_string(), // Goodwill account
                            credit_account: "2100".to_string(), // Consider payable
                            amount: Some(ppa.goodwill),
                            related_event_id: event.event_id.clone(),
                        });
                    }
                }
            }
            OrganizationalEventType::Divestiture(config) => {
                // Volume reduction scales with progress
                let vol_mult = 1.0 - (1.0 - config.volume_reduction) * progress;
                self.blend_multiplier(&mut effects.volume_multiplier, vol_mult);

                // Add divested entity to removals
                if config.remove_entity && progress >= 1.0 {
                    effects
                        .entity_changes
                        .entities_removed
                        .push(config.divested_entity_code.clone());
                }

                // Small error rate during transition
                if progress < 1.0 {
                    effects.error_rate_delta += 0.02;
                }
            }
            OrganizationalEventType::Reorganization(config) => {
                // Apply remappings
                for (old, new) in &config.cost_center_remapping {
                    effects
                        .entity_changes
                        .cost_center_remapping
                        .insert(old.clone(), new.clone());
                }
                for (old, new) in &config.department_remapping {
                    effects
                        .entity_changes
                        .department_remapping
                        .insert(old.clone(), new.clone());
                }

                // Transition error rate
                effects.error_rate_delta += config.transition_error_rate * (1.0 - progress);
            }
            OrganizationalEventType::LeadershipChange(config) => {
                // Policy change error rate
                effects.error_rate_delta += config.policy_change_error_rate * (1.0 - progress);
            }
            OrganizationalEventType::WorkforceReduction(config) => {
                // Error rate increase
                effects.error_rate_delta += config.error_rate_increase * (1.0 - progress * 0.5);

                // Processing time increase
                let time_mult = 1.0 + (config.processing_time_increase - 1.0) * (1.0 - progress);
                self.blend_multiplier(&mut effects.processing_time_multiplier, time_mult);

                // Volume slightly reduced
                let vol_mult = 1.0 - (config.reduction_percent * 0.3) * progress;
                self.blend_multiplier(&mut effects.volume_multiplier, vol_mult);

                // Severance entry at start
                if let Some(severance) = config.severance_costs {
                    if progress < 0.1 {
                        effects.special_entries.push(SpecialEntryRequest {
                            entry_type: "severance".to_string(),
                            description: "Workforce reduction severance costs".to_string(),
                            debit_account: "6500".to_string(), // Severance expense
                            credit_account: "2200".to_string(), // Accrued liabilities
                            amount: Some(severance),
                            related_event_id: event.event_id.clone(),
                        });
                    }
                }
            }
            OrganizationalEventType::Merger(config) => {
                // Volume multiplier
                let vol_mult = 1.0 + (config.volume_multiplier - 1.0) * progress;
                self.blend_multiplier(&mut effects.volume_multiplier, vol_mult);

                // Integration error rate
                let error_rate = config.integration_phases.error_rate_at(date);
                effects.error_rate_delta += error_rate;

                // Add merged entity
                effects
                    .entity_changes
                    .entities_added
                    .push(config.merged_entity_code.clone());
            }
        }
    }

    /// Apply effects from a process evolution event.
    fn apply_process_event_effects(
        &self,
        effects: &mut TimelineEffects,
        event: &ProcessEvolutionEvent,
        date: NaiveDate,
    ) {
        let progress = event.progress_at(date);
        effects.active_process_events.push(event.event_id.clone());

        match &event.event_type {
            ProcessEvolutionType::ApprovalWorkflowChange(config) => {
                // Processing time change
                let time_mult = 1.0 + (config.time_delta - 1.0) * progress;
                self.blend_multiplier(&mut effects.processing_time_multiplier, time_mult);

                // Transition error rate
                effects.error_rate_delta += config.error_rate_impact * (1.0 - progress);
            }
            ProcessEvolutionType::ProcessAutomation(config) => {
                // Processing time reduction
                let time_mult = 1.0
                    - (1.0 - config.processing_time_reduction)
                        * config.automation_rate_at_progress(progress);
                self.blend_multiplier(&mut effects.processing_time_multiplier, time_mult);

                // Error rate change
                let error_rate = config.error_rate_at_progress(progress);
                effects.error_rate_delta += error_rate - config.error_rate_before;
            }
            ProcessEvolutionType::PolicyChange(config) => {
                // Transition error rate
                effects.error_rate_delta += config.transition_error_rate * (1.0 - progress);

                // Track control changes
                for control_id in &config.affected_controls {
                    effects
                        .control_changes
                        .controls_modified
                        .push(control_id.clone());
                }
            }
            ProcessEvolutionType::ControlEnhancement(config) => {
                // Error reduction (negative delta)
                effects.error_rate_delta -= config.error_reduction * progress;

                // Processing time impact
                let time_mult = 1.0 + (config.processing_time_impact - 1.0) * progress;
                self.blend_multiplier(&mut effects.processing_time_multiplier, time_mult);

                // Track control changes
                effects
                    .control_changes
                    .controls_modified
                    .push(config.control_id.clone());
            }
        }
    }

    /// Apply effects from a technology transition event.
    fn apply_tech_event_effects(
        &self,
        effects: &mut TimelineEffects,
        event: &TechnologyTransitionEvent,
        date: NaiveDate,
    ) {
        effects.active_tech_events.push(event.event_id.clone());

        match &event.event_type {
            TechnologyTransitionType::ErpMigration(config) => {
                let phase = config.phases.phase_at(date);
                effects.migration_phase = Some(format!("{:?}", phase));

                // Error rate multiplier based on phase
                effects.error_rate_delta +=
                    config.migration_issues.combined_error_rate() * phase.error_rate_multiplier();

                // Processing time multiplier
                self.blend_multiplier(
                    &mut effects.processing_time_multiplier,
                    phase.processing_time_multiplier(),
                );

                // Parallel posting during parallel run
                if matches!(
                    phase,
                    crate::models::technology_transition::MigrationPhase::ParallelRun
                ) {
                    effects.in_parallel_posting = true;
                }
            }
            TechnologyTransitionType::ModuleImplementation(config) => {
                let progress = event.progress_at(date);

                // Error rate during implementation
                effects.error_rate_delta += config.implementation_error_rate * (1.0 - progress);

                // Processing time slightly higher initially
                let time_mult = 1.0 + 0.2 * (1.0 - progress);
                self.blend_multiplier(&mut effects.processing_time_multiplier, time_mult);
            }
            TechnologyTransitionType::IntegrationUpgrade(config) => {
                let progress = event.progress_at(date);

                // Transition error rate
                effects.error_rate_delta += config.transition_error_rate * (1.0 - progress);
            }
        }
    }

    /// Blend a multiplier according to the blending mode.
    fn blend_multiplier(&self, current: &mut f64, new: f64) {
        *current = match self.effect_blending {
            EffectBlendingMode::Multiplicative => *current * new,
            EffectBlendingMode::Additive => *current + new - 1.0,
            EffectBlendingMode::Maximum => current.max(new),
            EffectBlendingMode::Minimum => current.min(new),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::organizational_event::AcquisitionConfig;

    #[test]
    fn test_empty_timeline() {
        let config = EventTimelineConfig::default();
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let timeline = EventTimeline::new(config, 42, 12, start);

        let effects = timeline.compute_effects_for_period(6);
        assert!((effects.volume_multiplier - 1.0).abs() < 0.001);
        assert!((effects.amount_multiplier - 1.0).abs() < 0.001);
        assert!(!effects.has_active_events());
    }

    #[test]
    fn test_acquisition_effects() {
        let acq_date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let acq_config = AcquisitionConfig {
            acquired_entity_code: "ACME".to_string(),
            acquisition_date: acq_date,
            volume_multiplier: 1.35,
            ..Default::default()
        };

        let event =
            OrganizationalEvent::new("ACQ-001", OrganizationalEventType::Acquisition(acq_config));

        let config = EventTimelineConfig {
            org_events: vec![event],
            ..Default::default()
        };

        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let timeline = EventTimeline::new(config, 42, 12, start);

        // Before acquisition
        let before =
            timeline.compute_effects_for_date(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        assert!((before.volume_multiplier - 1.0).abs() < 0.001);

        // During acquisition
        let during =
            timeline.compute_effects_for_date(NaiveDate::from_ymd_opt(2024, 4, 1).unwrap());
        assert!(during.volume_multiplier > 1.0);
        assert!(during.has_active_events());
    }

    #[test]
    fn test_timeline_effects_neutral() {
        let effects = TimelineEffects::neutral();
        assert!((effects.volume_multiplier - 1.0).abs() < 0.001);
        assert!((effects.amount_multiplier - 1.0).abs() < 0.001);
        assert!((effects.error_rate_delta).abs() < 0.001);
    }

    #[test]
    fn test_active_events_summary() {
        let config = EventTimelineConfig::default();
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let timeline = EventTimeline::new(config, 42, 12, start);

        let summary = timeline.active_events_at(6);
        assert!(summary.org_events.is_empty());
        assert!(summary.process_events.is_empty());
        assert!(summary.tech_events.is_empty());
    }
}
