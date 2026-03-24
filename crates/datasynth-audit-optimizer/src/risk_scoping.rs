//! Risk-based audit scoping with coverage analysis and what-if impact reports.
//!
//! Provides functions to measure how well a set of included procedures covers
//! the standards and risk dimensions defined in a blueprint, and to estimate
//! the impact of removing a single procedure from the plan.

use std::collections::{HashMap, HashSet};

use serde::Serialize;

use datasynth_audit_fsm::schema::AuditBlueprint;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Coverage metrics for a set of included procedures against a blueprint.
#[derive(Debug, Clone, Serialize)]
pub struct CoverageReport {
    /// Fraction of distinct standards covered (0.0 to 1.0).
    pub standards_coverage: f64,
    /// Standards ref_ids that are covered by included procedures.
    pub standards_covered: Vec<String>,
    /// Standards ref_ids that are *not* covered by any included procedure.
    pub standards_uncovered: Vec<String>,
    /// Per-discriminator-category coverage fraction.
    pub risk_coverage: HashMap<String, f64>,
    /// Total number of procedures in the blueprint.
    pub total_procedures: usize,
    /// Number of procedures in the included set that exist in the blueprint.
    pub included_procedures: usize,
}

/// What-if impact report for removing a single procedure from the plan.
#[derive(Debug, Clone, Serialize)]
pub struct ImpactReport {
    /// The procedure being hypothetically removed.
    pub removed_procedure: String,
    /// Standards that would become uncovered after removal.
    pub standards_lost: Vec<String>,
    /// Change in standards_coverage (negative means coverage decreases).
    pub standards_coverage_delta: f64,
    /// Per-category change in risk coverage.
    pub risk_coverage_delta: HashMap<String, f64>,
    /// Procedures whose preconditions list the removed procedure.
    pub dependent_procedures_affected: Vec<String>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Analyse the coverage of a set of included procedures against the blueprint.
///
/// Standards are identified via `step.standards[].ref_id` across all procedures.
/// Risk coverage is computed per discriminator category: the fraction of values
/// that appear in at least one included procedure.
pub fn analyze_coverage(
    blueprint: &AuditBlueprint,
    included_procedures: &[String],
) -> CoverageReport {
    let included_set: HashSet<&str> = included_procedures.iter().map(|s| s.as_str()).collect();

    // Collect all procedure ids.
    let all_proc_ids: Vec<&str> = blueprint
        .phases
        .iter()
        .flat_map(|ph| ph.procedures.iter())
        .map(|p| p.id.as_str())
        .collect();

    let total_procedures = all_proc_ids.len();
    let included_count = all_proc_ids
        .iter()
        .filter(|id| included_set.contains(**id))
        .count();

    // --- Standards coverage ---
    let mut total_standards: HashSet<String> = HashSet::new();
    let mut covered_standards: HashSet<String> = HashSet::new();

    for phase in &blueprint.phases {
        for proc in &phase.procedures {
            for step in &proc.steps {
                for std_ref in &step.standards {
                    total_standards.insert(std_ref.ref_id.clone());
                    if included_set.contains(proc.id.as_str()) {
                        covered_standards.insert(std_ref.ref_id.clone());
                    }
                }
            }
        }
    }

    let standards_coverage = if total_standards.is_empty() {
        1.0
    } else {
        covered_standards.len() as f64 / total_standards.len() as f64
    };

    let mut standards_covered: Vec<String> = covered_standards.iter().cloned().collect();
    standards_covered.sort();

    let mut standards_uncovered: Vec<String> = total_standards
        .difference(&covered_standards)
        .cloned()
        .collect();
    standards_uncovered.sort();

    // --- Risk (discriminator) coverage per category ---
    // For each category, collect the total set of values and the included set.
    let mut cat_total: HashMap<String, HashSet<String>> = HashMap::new();
    let mut cat_included: HashMap<String, HashSet<String>> = HashMap::new();

    for phase in &blueprint.phases {
        for proc in &phase.procedures {
            for (cat, vals) in &proc.discriminators {
                let total_entry = cat_total.entry(cat.clone()).or_default();
                let inc_entry = cat_included.entry(cat.clone()).or_default();
                for v in vals {
                    total_entry.insert(v.clone());
                    if included_set.contains(proc.id.as_str()) {
                        inc_entry.insert(v.clone());
                    }
                }
            }
        }
    }

    let mut risk_coverage: HashMap<String, f64> = HashMap::new();
    for (cat, total_vals) in &cat_total {
        let inc_vals = cat_included.get(cat).map(|s| s.len()).unwrap_or(0);
        let frac = if total_vals.is_empty() {
            1.0
        } else {
            inc_vals as f64 / total_vals.len() as f64
        };
        risk_coverage.insert(cat.clone(), frac);
    }

    CoverageReport {
        standards_coverage,
        standards_covered,
        standards_uncovered,
        risk_coverage,
        total_procedures,
        included_procedures: included_count,
    }
}

/// Estimate the impact of removing a single procedure from the current plan.
///
/// Returns the delta in standards coverage and risk coverage, as well as which
/// procedures depend on the removed one via preconditions.
pub fn impact_of_removing(
    blueprint: &AuditBlueprint,
    preconditions: &HashMap<String, Vec<String>>,
    current_plan: &[String],
    remove_procedure: &str,
) -> ImpactReport {
    // Compute coverage with the full plan.
    let before = analyze_coverage(blueprint, current_plan);

    // Compute coverage without the removed procedure.
    let after_plan: Vec<String> = current_plan
        .iter()
        .filter(|id| id.as_str() != remove_procedure)
        .cloned()
        .collect();
    let after = analyze_coverage(blueprint, &after_plan);

    // Standards that become uncovered.
    let after_covered: HashSet<&str> = after.standards_covered.iter().map(|s| s.as_str()).collect();
    let mut standards_lost: Vec<String> = before
        .standards_covered
        .iter()
        .filter(|s| !after_covered.contains(s.as_str()))
        .cloned()
        .collect();
    standards_lost.sort();

    let standards_coverage_delta = after.standards_coverage - before.standards_coverage;

    // Risk coverage delta per category.
    let mut risk_coverage_delta: HashMap<String, f64> = HashMap::new();
    for (cat, before_val) in &before.risk_coverage {
        let after_val = after.risk_coverage.get(cat).copied().unwrap_or(0.0);
        risk_coverage_delta.insert(cat.clone(), after_val - before_val);
    }
    // Include categories that only appear in after (unlikely, but complete).
    for (cat, after_val) in &after.risk_coverage {
        risk_coverage_delta
            .entry(cat.clone())
            .or_insert_with(|| after_val - 0.0);
    }

    // Dependent procedures: those whose preconditions include the removed one.
    let mut dependent_procedures_affected: Vec<String> = preconditions
        .iter()
        .filter(|(proc_id, deps)| {
            current_plan.contains(proc_id) && deps.iter().any(|d| d == remove_procedure)
        })
        .map(|(proc_id, _)| proc_id.clone())
        .collect();
    dependent_procedures_affected.sort();

    ImpactReport {
        removed_procedure: remove_procedure.to_string(),
        standards_lost,
        standards_coverage_delta,
        risk_coverage_delta,
        dependent_procedures_affected,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_audit_fsm::loader::BlueprintWithPreconditions;

    fn load_fsa() -> BlueprintWithPreconditions {
        BlueprintWithPreconditions::load_builtin_fsa().expect("builtin FSA blueprint should load")
    }

    /// All procedures included should yield 100% standards coverage.
    #[test]
    fn test_full_scope_100_percent() {
        let bwp = load_fsa();
        let all_procs: Vec<String> = bwp
            .blueprint
            .phases
            .iter()
            .flat_map(|ph| ph.procedures.iter())
            .map(|p| p.id.clone())
            .collect();

        let report = analyze_coverage(&bwp.blueprint, &all_procs);

        assert!(
            (report.standards_coverage - 1.0).abs() < f64::EPSILON,
            "full scope should give 100% standards coverage, got {}",
            report.standards_coverage
        );
        assert!(
            report.standards_uncovered.is_empty(),
            "full scope should have no uncovered standards"
        );
        assert_eq!(report.included_procedures, report.total_procedures);
    }

    /// Empty included set should give 0% coverage.
    #[test]
    fn test_empty_scope_zero_percent() {
        let bwp = load_fsa();
        let report = analyze_coverage(&bwp.blueprint, &[]);

        assert!(
            report.standards_coverage.abs() < f64::EPSILON,
            "empty scope should give 0% standards coverage, got {}",
            report.standards_coverage
        );
        assert!(
            report.standards_covered.is_empty(),
            "empty scope should cover no standards"
        );
        assert_eq!(report.included_procedures, 0);
    }

    /// Including a subset of procedures should give partial coverage.
    #[test]
    fn test_partial_scope_coverage() {
        let bwp = load_fsa();
        // Include only the first procedure from the first phase.
        let first_proc = bwp.blueprint.phases[0].procedures[0].id.clone();
        let report = analyze_coverage(&bwp.blueprint, &[first_proc]);

        assert!(
            report.standards_coverage > 0.0,
            "partial scope should have > 0% coverage"
        );
        assert!(
            report.standards_coverage < 1.0,
            "partial scope should have < 100% coverage"
        );
        assert_eq!(report.included_procedures, 1);
    }

    /// Removing a procedure should report its dependents.
    #[test]
    fn test_removal_impact_reports_dependents() {
        let bwp = load_fsa();
        let all_procs: Vec<String> = bwp
            .blueprint
            .phases
            .iter()
            .flat_map(|ph| ph.procedures.iter())
            .map(|p| p.id.clone())
            .collect();

        // substantive_testing is a precondition of going_concern and subsequent_events
        let impact = impact_of_removing(
            &bwp.blueprint,
            &bwp.preconditions,
            &all_procs,
            "substantive_testing",
        );

        assert_eq!(impact.removed_procedure, "substantive_testing");
        assert!(
            impact
                .dependent_procedures_affected
                .contains(&"going_concern".to_string()),
            "going_concern depends on substantive_testing"
        );
        assert!(
            impact
                .dependent_procedures_affected
                .contains(&"subsequent_events".to_string()),
            "subsequent_events depends on substantive_testing"
        );
    }

    /// Both report types should serialize to JSON.
    #[test]
    fn test_reports_serialize() {
        let bwp = load_fsa();
        let all_procs: Vec<String> = bwp
            .blueprint
            .phases
            .iter()
            .flat_map(|ph| ph.procedures.iter())
            .map(|p| p.id.clone())
            .collect();

        let coverage = analyze_coverage(&bwp.blueprint, &all_procs);
        let json = serde_json::to_string(&coverage).expect("CoverageReport should serialize");
        assert!(json.contains("standards_coverage"));
        assert!(json.contains("risk_coverage"));

        let impact = impact_of_removing(
            &bwp.blueprint,
            &bwp.preconditions,
            &all_procs,
            "accept_engagement",
        );
        let json = serde_json::to_string(&impact).expect("ImpactReport should serialize");
        assert!(json.contains("removed_procedure"));
        assert!(json.contains("standards_lost"));
        assert!(json.contains("dependent_procedures_affected"));
    }
}
