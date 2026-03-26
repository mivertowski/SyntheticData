//! ISA 600 group audit simulation.
//!
//! Runs independent FSM engagements for each component entity, then
//! consolidates findings, coverage, and misstatement amounts at the group
//! level.  Components marked [`ComponentType::NotInScope`] are skipped
//! entirely, while significant components receive full FSM execution.

use std::path::PathBuf;

use datasynth_audit_fsm::context::EngagementContext;
use datasynth_audit_fsm::engine::AuditFsmEngine;
use datasynth_audit_fsm::error::AuditFsmError;
use datasynth_audit_fsm::loader::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Config types
// ---------------------------------------------------------------------------

/// Top-level configuration for a group audit simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAuditConfig {
    /// Identifier for the group (parent) entity.
    pub group_entity: String,
    /// Component configurations.
    pub components: Vec<ComponentConfig>,
    /// Blueprint used for the group-level engagement.
    pub group_blueprint: String,
    /// Overlay used for the group-level engagement.
    pub overlay: String,
    /// Group materiality threshold.
    pub group_materiality: f64,
    /// Base RNG seed; component `i` uses `base_seed + i + 1`.
    pub base_seed: u64,
}

/// Configuration for a single component within the group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    /// Entity identifier for the component.
    pub entity_id: String,
    /// Significance classification.
    pub component_type: ComponentType,
    /// Blueprint selector for the component's engagement.
    pub blueprint: String,
    /// Overlay selector.
    pub overlay: String,
    /// Component materiality (should be <= group materiality).
    pub component_materiality: f64,
}

/// ISA 600 component significance classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentType {
    /// Significant component — requires full audit or specified procedures.
    Significant,
    /// Non-significant component — analytical procedures at group level.
    NonSignificant,
    /// Not in scope — excluded from the group audit.
    NotInScope,
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentType::Significant => write!(f, "significant"),
            ComponentType::NonSignificant => write!(f, "non_significant"),
            ComponentType::NotInScope => write!(f, "not_in_scope"),
        }
    }
}

// ---------------------------------------------------------------------------
// Report types
// ---------------------------------------------------------------------------

/// Consolidated group audit report.
#[derive(Debug, Clone, Serialize)]
pub struct GroupAuditReport {
    /// Group (parent) entity identifier.
    pub group_entity: String,
    /// Per-component results.
    pub component_results: Vec<ComponentResult>,
    /// Total findings aggregated across all components.
    pub aggregated_findings: usize,
    /// Total misstatement amount (sum of component finding amounts).
    pub aggregated_misstatements: Decimal,
    /// Fraction of group covered by significant components.
    pub group_coverage: f64,
    /// Number of components with at least one finding.
    pub components_with_findings: usize,
    /// Overall group opinion risk assessment.
    pub group_opinion_risk: String,
}

/// Result of a single component's engagement.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentResult {
    /// Entity identifier.
    pub entity_id: String,
    /// Component significance type.
    pub component_type: String,
    /// FSM events emitted.
    pub events: usize,
    /// Typed artifacts generated.
    pub artifacts: usize,
    /// Audit findings generated.
    pub findings: usize,
    /// Fraction of procedures that completed.
    pub completion_rate: f64,
}

// ---------------------------------------------------------------------------
// Blueprint / overlay resolution
// ---------------------------------------------------------------------------

fn resolve_blueprint(name: &str) -> Result<BlueprintWithPreconditions, AuditFsmError> {
    match name {
        "fsa" | "builtin:fsa" => BlueprintWithPreconditions::load_builtin_fsa(),
        "ia" | "builtin:ia" => BlueprintWithPreconditions::load_builtin_ia(),
        "kpmg" | "builtin:kpmg" => BlueprintWithPreconditions::load_builtin_kpmg(),
        "pwc" | "builtin:pwc" => BlueprintWithPreconditions::load_builtin_pwc(),
        "deloitte" | "builtin:deloitte" => BlueprintWithPreconditions::load_builtin_deloitte(),
        "ey_gam_lite" | "builtin:ey_gam_lite" => {
            BlueprintWithPreconditions::load_builtin_ey_gam_lite()
        }
        path => BlueprintWithPreconditions::load_from_file(PathBuf::from(path)),
    }
}

fn resolve_overlay(
    name: &str,
) -> Result<datasynth_audit_fsm::schema::GenerationOverlay, AuditFsmError> {
    match name {
        "default" | "builtin:default" => {
            load_overlay(&OverlaySource::Builtin(BuiltinOverlay::Default))
        }
        "thorough" | "builtin:thorough" => {
            load_overlay(&OverlaySource::Builtin(BuiltinOverlay::Thorough))
        }
        "rushed" | "builtin:rushed" => {
            load_overlay(&OverlaySource::Builtin(BuiltinOverlay::Rushed))
        }
        "retail" | "builtin:retail" => {
            load_overlay(&OverlaySource::Builtin(BuiltinOverlay::IndustryRetail))
        }
        "manufacturing" | "builtin:manufacturing" => load_overlay(&OverlaySource::Builtin(
            BuiltinOverlay::IndustryManufacturing,
        )),
        "financial_services" | "builtin:financial_services" => load_overlay(
            &OverlaySource::Builtin(BuiltinOverlay::IndustryFinancialServices),
        ),
        path => load_overlay(&OverlaySource::Custom(PathBuf::from(path))),
    }
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/// Run a group audit simulation.
///
/// Each in-scope component entity runs its own FSM engagement independently.
/// Components marked [`ComponentType::NotInScope`] are skipped.  After all
/// component engagements complete, findings are aggregated, group coverage
/// is computed, and an overall opinion risk is assessed.
///
/// # Errors
///
/// Returns an error if any blueprint/overlay cannot be loaded or if the
/// engine fails for any in-scope component.
pub fn run_group_audit(config: &GroupAuditConfig) -> Result<GroupAuditReport, AuditFsmError> {
    let mut component_results = Vec::with_capacity(config.components.len());
    let mut total_findings: usize = 0;
    let mut total_misstatement = Decimal::ZERO;
    let mut components_with_findings: usize = 0;
    let mut significant_count: usize = 0;
    let mut in_scope_count: usize = 0;

    for (i, comp) in config.components.iter().enumerate() {
        // Skip not-in-scope components entirely.
        if comp.component_type == ComponentType::NotInScope {
            component_results.push(ComponentResult {
                entity_id: comp.entity_id.clone(),
                component_type: comp.component_type.to_string(),
                events: 0,
                artifacts: 0,
                findings: 0,
                completion_rate: 0.0,
            });
            continue;
        }

        in_scope_count += 1;
        if comp.component_type == ComponentType::Significant {
            significant_count += 1;
        }

        let bwp = resolve_blueprint(&comp.blueprint)?;
        let overlay = resolve_overlay(&comp.overlay)?;
        let seed = config.base_seed.wrapping_add(i as u64 + 1);
        let rng = ChaCha8Rng::seed_from_u64(seed);

        let mut engine = AuditFsmEngine::new(bwp, overlay, rng);

        let mut ctx = EngagementContext::demo();
        ctx.company_code = comp.entity_id.clone();

        let result = engine.run_engagement(&ctx)?;

        let findings = result.artifacts.findings.len();
        let total_procs = result.procedure_states.len();
        let completed = result
            .procedure_states
            .values()
            .filter(|s| s.as_str() == "completed" || s.as_str() == "closed")
            .count();

        if findings > 0 {
            components_with_findings += 1;
        }
        total_findings += findings;

        // Accumulate misstatement amounts from findings.
        for finding in &result.artifacts.findings {
            if let Some(amount) = finding.monetary_impact {
                total_misstatement += amount.abs();
            }
        }

        component_results.push(ComponentResult {
            entity_id: comp.entity_id.clone(),
            component_type: comp.component_type.to_string(),
            events: result.event_log.len(),
            artifacts: result.artifacts.total_artifacts(),
            findings,
            completion_rate: if total_procs > 0 {
                completed as f64 / total_procs as f64
            } else {
                0.0
            },
        });
    }

    // Group coverage: significant / in-scope.
    let group_coverage = if in_scope_count > 0 {
        significant_count as f64 / in_scope_count as f64
    } else {
        0.0
    };

    // Assess group opinion risk.
    let group_opinion_risk = assess_opinion_risk(
        total_findings,
        total_misstatement,
        config.group_materiality,
        group_coverage,
    );

    Ok(GroupAuditReport {
        group_entity: config.group_entity.clone(),
        component_results,
        aggregated_findings: total_findings,
        aggregated_misstatements: total_misstatement,
        group_coverage,
        components_with_findings,
        group_opinion_risk,
    })
}

/// Determine the group opinion risk level based on aggregate metrics.
fn assess_opinion_risk(
    total_findings: usize,
    total_misstatement: Decimal,
    group_materiality: f64,
    group_coverage: f64,
) -> String {
    let mat_decimal =
        Decimal::from_f64_retain(group_materiality).unwrap_or_else(|| Decimal::new(1_000_000, 0));

    if total_misstatement > mat_decimal || total_findings > 10 {
        "high".to_string()
    } else if group_coverage < 0.5 || total_findings > 5 {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_group_config() -> GroupAuditConfig {
        GroupAuditConfig {
            group_entity: "GROUP_PARENT".into(),
            components: vec![
                ComponentConfig {
                    entity_id: "COMP_A".into(),
                    component_type: ComponentType::Significant,
                    blueprint: "fsa".into(),
                    overlay: "default".into(),
                    component_materiality: 50_000.0,
                },
                ComponentConfig {
                    entity_id: "COMP_B".into(),
                    component_type: ComponentType::NonSignificant,
                    blueprint: "fsa".into(),
                    overlay: "default".into(),
                    component_materiality: 100_000.0,
                },
                ComponentConfig {
                    entity_id: "COMP_C".into(),
                    component_type: ComponentType::NotInScope,
                    blueprint: "fsa".into(),
                    overlay: "default".into(),
                    component_materiality: 200_000.0,
                },
            ],
            group_blueprint: "fsa".into(),
            overlay: "default".into(),
            group_materiality: 500_000.0,
            base_seed: 42,
        }
    }

    #[test]
    fn test_group_with_three_components() {
        let config = make_group_config();
        let report = run_group_audit(&config).unwrap();

        assert_eq!(report.group_entity, "GROUP_PARENT");
        assert_eq!(report.component_results.len(), 3);

        // All three should appear in results, but only two are in-scope.
        let in_scope: Vec<&ComponentResult> = report
            .component_results
            .iter()
            .filter(|c| c.events > 0)
            .collect();
        assert_eq!(
            in_scope.len(),
            2,
            "expected 2 in-scope component results with events"
        );
    }

    #[test]
    fn test_significant_component_coverage() {
        let config = make_group_config();
        let report = run_group_audit(&config).unwrap();

        // 1 Significant out of 2 in-scope = 0.5 coverage.
        assert!(
            (report.group_coverage - 0.5).abs() < 0.01,
            "expected 50% coverage, got {}",
            report.group_coverage
        );
    }

    #[test]
    fn test_findings_aggregation() {
        let config = make_group_config();
        let report = run_group_audit(&config).unwrap();

        let sum: usize = report.component_results.iter().map(|c| c.findings).sum();
        assert_eq!(
            report.aggregated_findings, sum,
            "aggregated findings should equal sum of component findings"
        );
    }

    #[test]
    fn test_not_in_scope_skipped() {
        let config = make_group_config();
        let report = run_group_audit(&config).unwrap();

        let comp_c = report
            .component_results
            .iter()
            .find(|c| c.entity_id == "COMP_C")
            .expect("COMP_C should be in results");

        assert_eq!(
            comp_c.events, 0,
            "not-in-scope component should have 0 events"
        );
        assert_eq!(
            comp_c.artifacts, 0,
            "not-in-scope component should have 0 artifacts"
        );
        assert_eq!(
            comp_c.findings, 0,
            "not-in-scope component should have 0 findings"
        );
        assert_eq!(
            comp_c.component_type, "not_in_scope",
            "component type should be not_in_scope"
        );
    }
}
