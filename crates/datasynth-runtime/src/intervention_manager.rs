//! Intervention validation, conflict resolution, and config path resolution.

use crate::causal_engine::ValidatedIntervention;
use datasynth_config::GeneratorConfig;
use datasynth_core::{Intervention, InterventionTiming, InterventionType};
use thiserror::Error;

/// Errors during intervention validation.
#[derive(Debug, Error)]
pub enum InterventionError {
    #[error("invalid target: {0}")]
    InvalidTarget(String),
    #[error(
        "timing out of range: intervention start_month {start} exceeds period_months {period}"
    )]
    TimingOutOfRange { start: u32, period: u32 },
    #[error("timing invalid: start_month must be >= 1, got {0}")]
    TimingInvalid(u32),
    #[error("conflict detected: interventions at priority {0} overlap on path '{1}'")]
    ConflictDetected(u32, String),
    #[error("bounds violation: {0}")]
    BoundsViolation(String),
}

/// Validates, resolves conflicts, and normalizes interventions.
pub struct InterventionManager;

impl InterventionManager {
    /// Validate a set of interventions against the config.
    pub fn validate(
        interventions: &[Intervention],
        config: &GeneratorConfig,
    ) -> Result<Vec<ValidatedIntervention>, InterventionError> {
        let mut validated = Vec::new();

        for intervention in interventions {
            Self::validate_timing(&intervention.timing, config)?;
            Self::validate_bounds(&intervention.intervention_type)?;

            let paths = Self::resolve_config_paths(&intervention.intervention_type);

            validated.push(ValidatedIntervention {
                intervention: intervention.clone(),
                affected_config_paths: paths,
            });
        }

        Self::check_conflicts(&validated)?;
        Ok(validated)
    }

    /// Validate timing is within generation period.
    fn validate_timing(
        timing: &InterventionTiming,
        config: &GeneratorConfig,
    ) -> Result<(), InterventionError> {
        if timing.start_month < 1 {
            return Err(InterventionError::TimingInvalid(timing.start_month));
        }
        if timing.start_month > config.global.period_months {
            return Err(InterventionError::TimingOutOfRange {
                start: timing.start_month,
                period: config.global.period_months,
            });
        }
        Ok(())
    }

    /// Validate intervention-specific bounds.
    fn validate_bounds(intervention_type: &InterventionType) -> Result<(), InterventionError> {
        match intervention_type {
            InterventionType::ControlFailure(cf) => {
                if !(0.0..=1.0).contains(&cf.severity) {
                    return Err(InterventionError::BoundsViolation(format!(
                        "control failure severity must be between 0.0 and 1.0, got {}",
                        cf.severity
                    )));
                }
            }
            InterventionType::MacroShock(ms) => {
                if ms.severity < 0.0 {
                    return Err(InterventionError::BoundsViolation(format!(
                        "macro shock severity must be >= 0.0, got {}",
                        ms.severity
                    )));
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Resolve which config paths an intervention affects.
    fn resolve_config_paths(intervention_type: &InterventionType) -> Vec<String> {
        match intervention_type {
            InterventionType::ParameterShift(ps) => vec![ps.target.clone()],
            InterventionType::ControlFailure(_) => {
                vec![
                    "internal_controls.exception_rate".to_string(),
                    "internal_controls.sod_violation_rate".to_string(),
                ]
            }
            InterventionType::MacroShock(_) => {
                vec![
                    "distributions.drift.economic_cycle.amplitude".to_string(),
                    "transactions.volume_multiplier".to_string(),
                ]
            }
            InterventionType::EntityEvent(ee) => {
                use datasynth_core::InterventionEntityEvent;
                match ee.subtype {
                    InterventionEntityEvent::VendorDefault => {
                        vec![
                            "vendor_network.dependencies.max_single_vendor_concentration"
                                .to_string(),
                        ]
                    }
                    InterventionEntityEvent::CustomerChurn => {
                        vec!["customer_segmentation.lifecycle.churned_rate".to_string()]
                    }
                    _ => vec![],
                }
            }
            InterventionType::ProcessChange(_) => {
                vec!["approval.thresholds".to_string()]
            }
            InterventionType::RegulatoryChange(_) => {
                vec!["accounting_standards".to_string()]
            }
            InterventionType::Custom(ci) => ci.config_overrides.keys().cloned().collect(),
            InterventionType::Composite(comp) => {
                let mut paths = Vec::new();
                for child in &comp.children {
                    paths.extend(Self::resolve_config_paths(child));
                }
                paths.sort();
                paths.dedup();
                paths
            }
        }
    }

    /// Check for conflicting interventions on the same config paths.
    fn check_conflicts(validated: &[ValidatedIntervention]) -> Result<(), InterventionError> {
        for i in 0..validated.len() {
            for j in (i + 1)..validated.len() {
                let a = &validated[i];
                let b = &validated[j];

                // Check for overlapping config paths
                for path_a in &a.affected_config_paths {
                    for path_b in &b.affected_config_paths {
                        if path_a == path_b
                            && Self::timing_overlaps(&a.intervention.timing, &b.intervention.timing)
                        {
                            // Same priority = conflict
                            if a.intervention.priority == b.intervention.priority {
                                return Err(InterventionError::ConflictDetected(
                                    a.intervention.priority,
                                    path_a.clone(),
                                ));
                            }
                            // Different priorities: higher wins, no error
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Check if two intervention timings overlap.
    fn timing_overlaps(a: &InterventionTiming, b: &InterventionTiming) -> bool {
        let a_end = a.start_month + a.duration_months.unwrap_or(u32::MAX - a.start_month);
        let b_end = b.start_month + b.duration_months.unwrap_or(u32::MAX - b.start_month);
        a.start_month < b_end && b.start_month < a_end
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_core::{
        ControlFailureIntervention, ControlFailureType, ControlTarget, OnsetType,
        ParameterShiftIntervention,
    };
    use datasynth_test_utils::fixtures::minimal_config;
    use uuid::Uuid;

    fn make_intervention(
        intervention_type: InterventionType,
        start_month: u32,
        priority: u32,
    ) -> Intervention {
        Intervention {
            id: Uuid::new_v4(),
            intervention_type,
            timing: InterventionTiming {
                start_month,
                duration_months: None,
                onset: OnsetType::Sudden,
                ramp_months: None,
            },
            label: None,
            priority,
        }
    }

    #[test]
    fn test_validate_timing_out_of_range() {
        let config = minimal_config();
        let intervention = make_intervention(
            InterventionType::ParameterShift(ParameterShiftIntervention {
                target: "test.path".to_string(),
                from: None,
                to: serde_json::json!(100),
                interpolation: Default::default(),
            }),
            999, // way beyond period_months
            0,
        );
        let result = InterventionManager::validate(&[intervention], &config);
        assert!(matches!(
            result,
            Err(InterventionError::TimingOutOfRange { .. })
        ));
    }

    #[test]
    fn test_validate_empty_interventions() {
        let config = minimal_config();
        let result = InterventionManager::validate(&[], &config);
        assert!(result.is_ok());
        assert!(result.expect("should be ok").is_empty());
    }

    #[test]
    fn test_validate_parameter_shift() {
        let config = minimal_config();
        let intervention = make_intervention(
            InterventionType::ParameterShift(ParameterShiftIntervention {
                target: "transactions.count".to_string(),
                from: None,
                to: serde_json::json!(2000),
                interpolation: Default::default(),
            }),
            1,
            0,
        );
        let result = InterventionManager::validate(&[intervention], &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_conflict_detection() {
        let config = minimal_config();
        let a = make_intervention(
            InterventionType::ParameterShift(ParameterShiftIntervention {
                target: "transactions.count".to_string(),
                from: None,
                to: serde_json::json!(2000),
                interpolation: Default::default(),
            }),
            1,
            0, // same priority
        );
        let b = make_intervention(
            InterventionType::ParameterShift(ParameterShiftIntervention {
                target: "transactions.count".to_string(),
                from: None,
                to: serde_json::json!(3000),
                interpolation: Default::default(),
            }),
            1,
            0, // same priority → conflict
        );
        let result = InterventionManager::validate(&[a, b], &config);
        assert!(matches!(
            result,
            Err(InterventionError::ConflictDetected(_, _))
        ));
    }

    #[test]
    fn test_conflict_resolution_by_priority() {
        let config = minimal_config();
        let a = make_intervention(
            InterventionType::ParameterShift(ParameterShiftIntervention {
                target: "transactions.count".to_string(),
                from: None,
                to: serde_json::json!(2000),
                interpolation: Default::default(),
            }),
            1,
            1, // lower priority
        );
        let b = make_intervention(
            InterventionType::ParameterShift(ParameterShiftIntervention {
                target: "transactions.count".to_string(),
                from: None,
                to: serde_json::json!(3000),
                interpolation: Default::default(),
            }),
            1,
            2, // higher priority → no conflict
        );
        let result = InterventionManager::validate(&[a, b], &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_bounds_control_failure() {
        let config = minimal_config();
        let intervention = make_intervention(
            InterventionType::ControlFailure(ControlFailureIntervention {
                subtype: ControlFailureType::EffectivenessReduction,
                control_target: ControlTarget::ById {
                    control_id: "C001".to_string(),
                },
                severity: 1.5, // out of bounds
                detectable: true,
            }),
            1,
            0,
        );
        let result = InterventionManager::validate(&[intervention], &config);
        assert!(matches!(result, Err(InterventionError::BoundsViolation(_))));
    }
}
