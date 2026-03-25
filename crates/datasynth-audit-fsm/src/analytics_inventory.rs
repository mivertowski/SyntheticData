//! Analytics inventory loader for FSA and IA data requirements and procedures.
//!
//! Deserializes the data analytics inventory JSON files that map every audit
//! step to its data requirements and analytical procedures. These inventories
//! are embedded at compile time via [`include_str!`] and parsed on first load.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Embedded JSON data
// ---------------------------------------------------------------------------

const FSA_INVENTORY: &str = include_str!("../inventories/data_analytics_inventory_fsa.json");
const IA_INVENTORY: &str = include_str!("../inventories/data_analytics_inventory_ia.json");

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Analytics inventory entry for a single audit step.
///
/// Each step maps to zero or more data requirements (inputs needed) and
/// zero or more analytical procedures (analyses to perform).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepInventory {
    /// Unique step identifier (e.g. `"mat_step_1"`).
    pub step_id: String,
    /// Parent procedure identifier (e.g. `"planning_materiality"`).
    pub procedure_id: String,
    /// Audit phase (e.g. `"planning"`, `"execution"`, `"completion"`).
    pub phase: String,
    /// Data inputs required for this step.
    #[serde(default)]
    pub data_requirements: Vec<DataRequirement>,
    /// Analytical procedures applicable to this step.
    #[serde(default)]
    pub analytical_procedures: Vec<AnalyticalProcedure>,
}

/// A data requirement specifying what input data an audit step needs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRequirement {
    /// Category of data (e.g. `"financial_statements"`, `"organizational"`).
    pub data_type: String,
    /// Human-readable name of the data source.
    pub name: String,
    /// Scope of data needed (e.g. `"Full year, consolidated"`).
    #[serde(default)]
    pub scope: String,
    /// Specific fields required from this data source.
    #[serde(default)]
    pub fields: Vec<String>,
    /// Source system the data originates from.
    #[serde(default)]
    pub source_system: String,
    /// Expected format (e.g. `"Trial balance export"`, `"PDF/annual report"`).
    #[serde(default)]
    pub format: String,
    /// Frequency of data (e.g. `"monthly"`, `"quarterly"`).
    #[serde(default)]
    pub frequency: String,
    /// Financial statement assertion this data supports.
    #[serde(default)]
    pub assertion: String,
}

/// An analytical procedure that can be performed during an audit step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticalProcedure {
    /// Type of procedure (e.g. `"ratio_analysis"`, `"trend_analysis"`).
    pub procedure_type: String,
    /// Human-readable name of the procedure.
    pub name: String,
    /// Description of what the procedure does.
    #[serde(default)]
    pub description: String,
    /// Input data identifiers for this procedure.
    #[serde(default)]
    pub input_data: Vec<String>,
    /// Data features / columns used by the procedure.
    #[serde(default)]
    pub data_features: Vec<String>,
    /// Graph-based features used by the procedure.
    #[serde(default)]
    pub graph_features: Vec<String>,
    /// Expected output of the procedure.
    #[serde(default)]
    pub expected_output: String,
    /// Threshold or tolerance for the procedure.
    #[serde(default)]
    pub threshold: String,
    /// Suggested tool or technique hint.
    #[serde(default)]
    pub tool_hint: String,
}

// ---------------------------------------------------------------------------
// Loaders
// ---------------------------------------------------------------------------

/// Load the FSA (Financial Statement Audit) analytics inventory as a
/// `HashMap` keyed by `step_id`.
pub fn load_fsa_inventory() -> HashMap<String, StepInventory> {
    load_inventory(FSA_INVENTORY)
}

/// Load the IA (Internal Audit) analytics inventory as a `HashMap` keyed
/// by `step_id`.
pub fn load_ia_inventory() -> HashMap<String, StepInventory> {
    load_inventory(IA_INVENTORY)
}

fn load_inventory(json: &str) -> HashMap<String, StepInventory> {
    let steps: Vec<StepInventory> = serde_json::from_str(json).unwrap_or_default();
    steps.into_iter().map(|s| (s.step_id.clone(), s)).collect()
}

/// Look up a step's analytics inventory entry by `step_id`.
pub fn lookup_step<'a>(
    inventory: &'a HashMap<String, StepInventory>,
    step_id: &str,
) -> Option<&'a StepInventory> {
    inventory.get(step_id)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_fsa_inventory() {
        let inv = load_fsa_inventory();
        assert_eq!(inv.len(), 40, "FSA inventory should have 40 steps");
    }

    #[test]
    fn test_load_ia_inventory() {
        let inv = load_ia_inventory();
        assert_eq!(inv.len(), 20, "IA inventory should have 20 steps");
    }

    #[test]
    fn test_fsa_step_has_data_requirements() {
        let inv = load_fsa_inventory();
        let step = lookup_step(&inv, "mat_step_1").expect("mat_step_1 should exist");
        assert!(
            !step.data_requirements.is_empty(),
            "mat_step_1 should have data requirements"
        );
        let has_financial = step
            .data_requirements
            .iter()
            .any(|r| r.data_type == "financial_statements");
        assert!(has_financial, "mat_step_1 should require financial_statements");
    }

    #[test]
    fn test_fsa_step_has_analytical_procedures() {
        let inv = load_fsa_inventory();
        let step = lookup_step(&inv, "risk_step_1").expect("risk_step_1 should exist");
        assert!(
            !step.analytical_procedures.is_empty(),
            "risk_step_1 should have analytical procedures"
        );
        let has_trend = step
            .analytical_procedures
            .iter()
            .any(|p| p.procedure_type == "trend_analysis");
        assert!(has_trend, "risk_step_1 should have trend_analysis");
    }

    #[test]
    fn test_step_data_requirements_fields() {
        let inv = load_fsa_inventory();
        let step = lookup_step(&inv, "mat_step_1").expect("mat_step_1 should exist");
        let fs_req = step
            .data_requirements
            .iter()
            .find(|r| r.data_type == "financial_statements")
            .expect("should have financial_statements requirement");
        assert!(
            !fs_req.fields.is_empty(),
            "financial_statements requirement should have fields"
        );
        assert!(
            fs_req.fields.contains(&"revenue".to_string()),
            "financial_statements should include revenue field"
        );
    }

    #[test]
    fn test_step_analytical_procedures_types() {
        let inv = load_fsa_inventory();
        let step = lookup_step(&inv, "risk_step_1").expect("risk_step_1 should exist");
        let types: Vec<&str> = step
            .analytical_procedures
            .iter()
            .map(|p| p.procedure_type.as_str())
            .collect();
        assert!(
            types.contains(&"trend_analysis"),
            "risk_step_1 should have trend_analysis"
        );
        assert!(
            types.contains(&"ratio_analysis"),
            "risk_step_1 should have ratio_analysis"
        );
    }

    #[test]
    fn test_lookup_nonexistent_step() {
        let inv = load_fsa_inventory();
        assert!(
            lookup_step(&inv, "nonexistent_step").is_none(),
            "nonexistent step should return None"
        );
    }

    #[test]
    fn test_ia_inventory_has_procedures() {
        let inv = load_ia_inventory();
        let has_procedures = inv.values().any(|s| !s.analytical_procedures.is_empty());
        assert!(has_procedures, "IA inventory should have at least one step with procedures");
    }
}
