use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The full taxonomy of supported interventions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InterventionType {
    /// Entity-level events (vendor default, customer churn, etc.)
    EntityEvent(EntityEventIntervention),
    /// Parameter shifts (config value changes)
    ParameterShift(ParameterShiftIntervention),
    /// Control failures (effectiveness reduction, bypass)
    ControlFailure(ControlFailureIntervention),
    /// Process changes (approval thresholds, automation)
    ProcessChange(ProcessChangeIntervention),
    /// Macroeconomic shocks (recession, inflation spike)
    MacroShock(MacroShockIntervention),
    /// Regulatory changes (new standards, threshold changes)
    RegulatoryChange(RegulatoryChangeIntervention),
    /// Composite bundle of multiple interventions
    Composite(CompositeIntervention),
    /// Custom user-defined intervention
    Custom(CustomIntervention),
}

// ── Entity Events ──────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityEventIntervention {
    pub subtype: InterventionEntityEvent,
    pub target: EntityTarget,
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionEntityEvent {
    VendorDefault,
    CustomerChurn,
    EmployeeDeparture,
    NewVendorOnboarding,
    MergerAcquisition,
    VendorCollusion,
    CustomerConsolidation,
    KeyPersonRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTarget {
    /// Target by cluster type.
    pub cluster: Option<String>,
    /// Target by specific entity IDs.
    pub entity_ids: Option<Vec<String>>,
    /// Target by attribute filter (e.g., country = "US").
    pub filter: Option<HashMap<String, String>>,
    /// Number of entities to affect (random selection from filter).
    pub count: Option<u32>,
    /// Fraction of entities to affect (alternative to count).
    pub fraction: Option<f64>,
}

// ── Parameter Shifts ───────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterShiftIntervention {
    /// Dot-path to config parameter.
    pub target: String,
    /// Original value (for documentation; auto-filled from config).
    pub from: Option<serde_json::Value>,
    /// New value.
    pub to: serde_json::Value,
    /// Interpolation method during ramp.
    #[serde(default)]
    pub interpolation: InterpolationType,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InterpolationType {
    #[default]
    Linear,
    Exponential,
    Logistic {
        steepness: f64,
    },
    Step,
}

// ── Control Failures ───────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlFailureIntervention {
    pub subtype: ControlFailureType,
    /// Control ID (e.g., "C003") or control category.
    pub control_target: ControlTarget,
    /// Effectiveness multiplier (0.0 = complete failure, 1.0 = normal).
    pub severity: f64,
    /// Whether the failure is detectable by monitoring.
    #[serde(default)]
    pub detectable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlFailureType {
    EffectivenessReduction,
    CompleteBypass,
    IntermittentFailure { failure_probability: f64 },
    DelayedDetection { detection_lag_months: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ControlTarget {
    ById { control_id: String },
    ByCategory { coso_component: String },
    ByScope { scope: String },
    All,
}

// ── Process Changes ────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessChangeIntervention {
    pub subtype: ProcessChangeType,
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessChangeType {
    ApprovalThresholdChange,
    NewApprovalLevel,
    SystemMigration,
    ProcessAutomation,
    OutsourcingTransition,
    PolicyChange,
    ReorganizationRestructuring,
}

// ── Macro Shocks ───────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroShockIntervention {
    pub subtype: MacroShockType,
    /// Severity multiplier (1.0 = standard severity for the shock type).
    pub severity: f64,
    /// Named preset (maps to pre-configured parameter bundles).
    pub preset: Option<String>,
    /// Override individual macro parameters.
    #[serde(default)]
    pub overrides: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MacroShockType {
    Recession,
    InflationSpike,
    CurrencyCrisis,
    InterestRateShock,
    CommodityShock,
    PandemicDisruption,
    SupplyChainCrisis,
    CreditCrunch,
}

// ── Regulatory Changes ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryChangeIntervention {
    pub subtype: RegulatoryChangeType,
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegulatoryChangeType {
    NewStandardAdoption,
    MaterialityThresholdChange,
    ReportingRequirementChange,
    ComplianceThresholdChange,
    AuditStandardChange,
    TaxRateChange,
}

// ── Composite ──────────────────────────────────────

/// Bundles multiple interventions into a named package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeIntervention {
    pub name: String,
    pub description: String,
    /// Child interventions applied together.
    pub children: Vec<InterventionType>,
    /// Conflict resolution: first_wins, last_wins, average, error.
    #[serde(default)]
    pub conflict_resolution: ConflictResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    #[default]
    FirstWins,
    LastWins,
    Average,
    Error,
}

// ── Custom ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomIntervention {
    pub name: String,
    /// Config path → value mappings.
    #[serde(default)]
    pub config_overrides: HashMap<String, serde_json::Value>,
    /// Causal downstream effects to trigger.
    #[serde(default)]
    pub downstream_triggers: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intervention_type_tagged_serde() {
        let json = r#"{
            "type": "parameter_shift",
            "target": "transactions.count",
            "from": 1000,
            "to": 2000,
            "interpolation": "linear"
        }"#;
        let intervention: InterventionType = serde_json::from_str(json).unwrap();
        assert!(matches!(intervention, InterventionType::ParameterShift(_)));

        // Roundtrip
        let serialized = serde_json::to_string(&intervention).unwrap();
        let deserialized: InterventionType = serde_json::from_str(&serialized).unwrap();
        assert!(matches!(deserialized, InterventionType::ParameterShift(_)));
    }

    #[test]
    fn test_entity_event_serde() {
        let json = r#"{
            "type": "entity_event",
            "subtype": "vendor_default",
            "target": {
                "cluster": "problematic",
                "count": 5
            },
            "parameters": {}
        }"#;
        let intervention: InterventionType = serde_json::from_str(json).unwrap();
        if let InterventionType::EntityEvent(e) = intervention {
            assert!(matches!(e.subtype, InterventionEntityEvent::VendorDefault));
            assert_eq!(e.target.cluster, Some("problematic".to_string()));
            assert_eq!(e.target.count, Some(5));
        } else {
            panic!("Expected EntityEvent");
        }
    }

    #[test]
    fn test_macro_shock_serde() {
        let json = r#"{
            "type": "macro_shock",
            "subtype": "recession",
            "severity": 1.5,
            "preset": "2008_financial_crisis",
            "overrides": {"gdp_growth": -0.03}
        }"#;
        let intervention: InterventionType = serde_json::from_str(json).unwrap();
        if let InterventionType::MacroShock(m) = intervention {
            assert!(matches!(m.subtype, MacroShockType::Recession));
            assert_eq!(m.severity, 1.5);
            assert_eq!(m.preset, Some("2008_financial_crisis".to_string()));
        } else {
            panic!("Expected MacroShock");
        }
    }

    #[test]
    fn test_control_failure_serde() {
        let json = r#"{
            "type": "control_failure",
            "subtype": "complete_bypass",
            "control_target": {"control_id": "C003"},
            "severity": 0.0,
            "detectable": false
        }"#;
        let intervention: InterventionType = serde_json::from_str(json).unwrap();
        assert!(matches!(intervention, InterventionType::ControlFailure(_)));
    }

    #[test]
    fn test_composite_serde() {
        let json = r#"{
            "type": "composite",
            "name": "recession_scenario",
            "description": "Combined recession effects",
            "children": [
                {
                    "type": "macro_shock",
                    "subtype": "recession",
                    "severity": 1.0,
                    "overrides": {}
                }
            ],
            "conflict_resolution": "first_wins"
        }"#;
        let intervention: InterventionType = serde_json::from_str(json).unwrap();
        if let InterventionType::Composite(c) = intervention {
            assert_eq!(c.name, "recession_scenario");
            assert_eq!(c.children.len(), 1);
        } else {
            panic!("Expected Composite");
        }
    }

    #[test]
    fn test_conflict_resolution_default() {
        let cr = ConflictResolution::default();
        assert!(matches!(cr, ConflictResolution::FirstWins));
    }
}
