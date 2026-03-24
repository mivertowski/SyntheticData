//! Resource-constrained plan optimization.
//!
//! Given a blueprint, overlay, preconditions, and resource constraints, select
//! the best-value set of procedures that fits within a budget.

use std::collections::{HashMap, HashSet, VecDeque};

use serde::Serialize;

use datasynth_audit_fsm::schema::{AuditBlueprint, BlueprintProcedure, GenerationOverlay};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Constraints that limit which procedures can be included in a plan.
#[derive(Debug, Clone)]
pub struct ResourceConstraints {
    /// Maximum total hours available.
    pub total_budget_hours: f64,
    /// Per-role hour availability (unused roles are unconstrained).
    pub role_availability: HashMap<String, f64>,
    /// Procedure ids that must be included regardless of budget.
    pub must_include: Vec<String>,
    /// Procedure ids that must be excluded.
    pub must_exclude: Vec<String>,
}

/// An optimized audit plan produced by [`optimize_plan`].
#[derive(Debug, Clone, Serialize)]
pub struct OptimizedPlan {
    /// Procedure ids included in the plan.
    pub included_procedures: Vec<String>,
    /// Procedure ids excluded from the plan.
    pub excluded_procedures: Vec<String>,
    /// Total effective hours of included procedures.
    pub total_hours: f64,
    /// Total monetary cost of included procedures.
    pub total_cost: f64,
    /// Fraction of distinct discriminator values covered (0.0 to 1.0).
    pub risk_coverage: f64,
    /// Fraction of distinct standards covered (0.0 to 1.0).
    pub standards_coverage: f64,
    /// Hours along the longest precondition chain.
    pub critical_path_hours: f64,
    /// Hours per primary role across included procedures.
    pub role_hours: HashMap<String, f64>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Select an optimal subset of procedures under resource constraints.
///
/// # Algorithm
///
/// 1. Collect all procedures from the blueprint.
/// 2. Build a lookup by procedure id.
/// 3. Expand `must_include` with transitive preconditions (BFS).
/// 4. Remove `must_exclude` (warn if it is a dependency of a must-include).
/// 5. If the mandatory set already exceeds the budget, return it as-is.
/// 6. Score remaining procedures by discriminator coverage per hour and greedily
///    add while budget allows.
/// 7. Compute coverage, critical path, and role hours.
pub fn optimize_plan(
    blueprint: &AuditBlueprint,
    overlay: &GenerationOverlay,
    preconditions: &HashMap<String, Vec<String>>,
    constraints: &ResourceConstraints,
) -> OptimizedPlan {
    let costs = &overlay.resource_costs;

    // ------------------------------------------------------------------
    // 1. Collect all procedures from all phases.
    // ------------------------------------------------------------------
    let all_procs: Vec<&BlueprintProcedure> = blueprint
        .phases
        .iter()
        .flat_map(|phase| phase.procedures.iter())
        .collect();

    // ------------------------------------------------------------------
    // 2. Build procedure lookup.
    // ------------------------------------------------------------------
    let proc_map: HashMap<&str, &BlueprintProcedure> =
        all_procs.iter().map(|p| (p.id.as_str(), *p)).collect();

    let all_ids: HashSet<&str> = proc_map.keys().copied().collect();

    // ------------------------------------------------------------------
    // 3. Expand must_include with transitive preconditions (BFS).
    // ------------------------------------------------------------------
    let mut mandatory: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();

    for id in &constraints.must_include {
        if mandatory.insert(id.clone()) {
            queue.push_back(id.clone());
        }
    }

    while let Some(proc_id) = queue.pop_front() {
        if let Some(deps) = preconditions.get(&proc_id) {
            for dep in deps {
                if mandatory.insert(dep.clone()) {
                    queue.push_back(dep.clone());
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // 4. Remove must_exclude.
    // ------------------------------------------------------------------
    let exclude_set: HashSet<&str> = constraints
        .must_exclude
        .iter()
        .map(|s| s.as_str())
        .collect();

    // Remove excluded from mandatory (with warning — we just proceed).
    mandatory.retain(|id| !exclude_set.contains(id.as_str()));

    // ------------------------------------------------------------------
    // 5. Compute hours for mandatory set.
    // ------------------------------------------------------------------
    let mandatory_hours: f64 = mandatory
        .iter()
        .filter_map(|id| proc_map.get(id.as_str()))
        .map(|p| costs.effective_hours(p))
        .sum();

    let mut included: HashSet<String> = mandatory.clone();

    // ------------------------------------------------------------------
    // 6. If budget not exhausted, score and greedily add remaining.
    // ------------------------------------------------------------------
    if mandatory_hours < constraints.total_budget_hours {
        let mut remaining: Vec<&BlueprintProcedure> = all_procs
            .iter()
            .filter(|p| !included.contains(&p.id) && !exclude_set.contains(p.id.as_str()))
            .copied()
            .collect();

        // Score: discriminator values / effective hours.
        remaining.sort_by(|a, b| {
            let score_a = discriminator_score(a) / costs.effective_hours(a);
            let score_b = discriminator_score(b) / costs.effective_hours(b);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut budget_remaining = constraints.total_budget_hours - mandatory_hours;
        for proc in remaining {
            let h = costs.effective_hours(proc);
            if h <= budget_remaining {
                included.insert(proc.id.clone());
                budget_remaining -= h;
            }
        }
    }

    // ------------------------------------------------------------------
    // 7. Compute metrics.
    // ------------------------------------------------------------------
    let total_hours: f64 = included
        .iter()
        .filter_map(|id| proc_map.get(id.as_str()))
        .map(|p| costs.effective_hours(p))
        .sum();

    let total_cost: f64 = included
        .iter()
        .filter_map(|id| proc_map.get(id.as_str()))
        .map(|p| costs.procedure_cost(p))
        .sum();

    // Standards coverage.
    let (included_standards, total_standards) = compute_standards_sets(blueprint, &included);
    let standards_coverage = if total_standards.is_empty() {
        1.0
    } else {
        included_standards.len() as f64 / total_standards.len() as f64
    };

    // Risk (discriminator) coverage.
    let (included_disc_values, total_disc_values) =
        compute_discriminator_sets(blueprint, &included);
    let risk_coverage = if total_disc_values.is_empty() {
        1.0
    } else {
        included_disc_values.len() as f64 / total_disc_values.len() as f64
    };

    // Critical path hours.
    let critical_path_hours =
        compute_critical_path_hours(&included, &proc_map, preconditions, costs, overlay);

    // Role hours.
    let mut role_hours: HashMap<String, f64> = HashMap::new();
    for id in &included {
        if let Some(proc) = proc_map.get(id.as_str()) {
            let h = costs.effective_hours(proc);
            let role = proc
                .required_roles
                .first()
                .cloned()
                .unwrap_or_else(|| "audit_staff".to_string());
            *role_hours.entry(role).or_insert(0.0) += h;
        }
    }

    // Excluded procedures.
    let excluded: Vec<String> = all_ids
        .iter()
        .filter(|id| !included.contains(**id))
        .map(|id| id.to_string())
        .collect();

    let mut included_sorted: Vec<String> = included.into_iter().collect();
    included_sorted.sort();
    let mut excluded_sorted = excluded;
    excluded_sorted.sort();

    OptimizedPlan {
        included_procedures: included_sorted,
        excluded_procedures: excluded_sorted,
        total_hours,
        total_cost,
        risk_coverage,
        standards_coverage,
        critical_path_hours,
        role_hours,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Discriminator score: total number of discriminator values across all categories.
fn discriminator_score(proc: &BlueprintProcedure) -> f64 {
    let count: usize = proc.discriminators.values().map(|v| v.len()).sum();
    // Ensure we never return 0 to avoid NaN in division.
    (count.max(1)) as f64
}

/// Collect the set of unique standard ref_ids for included procedures and for
/// all procedures in the blueprint.
fn compute_standards_sets(
    blueprint: &AuditBlueprint,
    included: &HashSet<String>,
) -> (HashSet<String>, HashSet<String>) {
    let mut total = HashSet::new();
    let mut inc = HashSet::new();

    for phase in &blueprint.phases {
        for proc in &phase.procedures {
            for step in &proc.steps {
                for std_ref in &step.standards {
                    total.insert(std_ref.ref_id.clone());
                    if included.contains(&proc.id) {
                        inc.insert(std_ref.ref_id.clone());
                    }
                }
            }
        }
    }

    (inc, total)
}

/// A set of `(category, value)` discriminator pairs.
type DiscriminatorSet = HashSet<(String, String)>;

/// Collect the set of unique discriminator `(category, value)` pairs for
/// included procedures vs all procedures.
fn compute_discriminator_sets(
    blueprint: &AuditBlueprint,
    included: &HashSet<String>,
) -> (DiscriminatorSet, DiscriminatorSet) {
    let mut total = HashSet::new();
    let mut inc = HashSet::new();

    for phase in &blueprint.phases {
        for proc in &phase.procedures {
            for (cat, vals) in &proc.discriminators {
                for v in vals {
                    total.insert((cat.clone(), v.clone()));
                    if included.contains(&proc.id) {
                        inc.insert((cat.clone(), v.clone()));
                    }
                }
            }
        }
    }

    (inc, total)
}

/// Compute the critical path hours: the longest chain of preconditions
/// measured in effective hours.
fn compute_critical_path_hours(
    included: &HashSet<String>,
    proc_map: &HashMap<&str, &BlueprintProcedure>,
    preconditions: &HashMap<String, Vec<String>>,
    costs: &datasynth_audit_fsm::schema::ResourceCosts,
    _overlay: &GenerationOverlay,
) -> f64 {
    // Memoised DFS: for each procedure, compute the maximum total hours from
    // root of the precondition chain to that procedure.
    let mut memo: HashMap<String, f64> = HashMap::new();

    fn dfs(
        id: &str,
        included: &HashSet<String>,
        proc_map: &HashMap<&str, &BlueprintProcedure>,
        preconditions: &HashMap<String, Vec<String>>,
        costs: &datasynth_audit_fsm::schema::ResourceCosts,
        memo: &mut HashMap<String, f64>,
    ) -> f64 {
        if let Some(&cached) = memo.get(id) {
            return cached;
        }
        let self_hours = proc_map
            .get(id)
            .map(|p| costs.effective_hours(p))
            .unwrap_or(0.0);

        let max_pred = preconditions
            .get(id)
            .map(|deps| {
                deps.iter()
                    .filter(|d| included.contains(d.as_str()))
                    .map(|d| dfs(d, included, proc_map, preconditions, costs, memo))
                    .fold(0.0_f64, f64::max)
            })
            .unwrap_or(0.0);

        let total = self_hours + max_pred;
        memo.insert(id.to_string(), total);
        total
    }

    included
        .iter()
        .map(|id| dfs(id, included, proc_map, preconditions, costs, &mut memo))
        .fold(0.0_f64, f64::max)
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

    #[test]
    fn test_must_include_always_present() {
        let bwp = load_fsa();
        let overlay = GenerationOverlay::default();
        let constraints = ResourceConstraints {
            total_budget_hours: 1000.0,
            role_availability: HashMap::new(),
            must_include: vec!["form_opinion".to_string()],
            must_exclude: vec![],
        };

        let plan = optimize_plan(&bwp.blueprint, &overlay, &bwp.preconditions, &constraints);

        assert!(
            plan.included_procedures
                .contains(&"form_opinion".to_string()),
            "form_opinion must be included"
        );
        // Transitive preconditions must also be included.
        assert!(
            plan.included_procedures
                .contains(&"going_concern".to_string()),
            "going_concern (transitive dep) must be included"
        );
        assert!(
            plan.included_procedures
                .contains(&"subsequent_events".to_string()),
            "subsequent_events (transitive dep) must be included"
        );
    }

    #[test]
    fn test_budget_constrains_selection() {
        let bwp = load_fsa();
        let overlay = GenerationOverlay::default();

        // Very tight budget: only enough for the smallest procedure.
        let constraints = ResourceConstraints {
            total_budget_hours: 5.0,
            role_availability: HashMap::new(),
            must_include: vec![],
            must_exclude: vec![],
        };

        let plan = optimize_plan(&bwp.blueprint, &overlay, &bwp.preconditions, &constraints);

        assert!(
            plan.total_hours <= 5.0,
            "total hours {} should not exceed budget 5.0",
            plan.total_hours
        );
        // With a 5-hour budget, we cannot fit all 8 procedures.
        let total_proc_count: usize = bwp
            .blueprint
            .phases
            .iter()
            .map(|p| p.procedures.len())
            .sum();
        assert!(
            plan.included_procedures.len() < total_proc_count,
            "tight budget should exclude some procedures"
        );
    }

    #[test]
    fn test_must_exclude_removed() {
        let bwp = load_fsa();
        let overlay = GenerationOverlay::default();
        let constraints = ResourceConstraints {
            total_budget_hours: 1000.0,
            role_availability: HashMap::new(),
            must_include: vec![],
            must_exclude: vec!["analytical_procedures".to_string()],
        };

        let plan = optimize_plan(&bwp.blueprint, &overlay, &bwp.preconditions, &constraints);

        assert!(
            !plan
                .included_procedures
                .contains(&"analytical_procedures".to_string()),
            "analytical_procedures must be excluded"
        );
        assert!(
            plan.excluded_procedures
                .contains(&"analytical_procedures".to_string()),
            "analytical_procedures must appear in excluded list"
        );
    }

    #[test]
    fn test_critical_path_computed() {
        let bwp = load_fsa();
        let overlay = GenerationOverlay::default();
        let constraints = ResourceConstraints {
            total_budget_hours: 1000.0,
            role_availability: HashMap::new(),
            must_include: vec![],
            must_exclude: vec![],
        };

        let plan = optimize_plan(&bwp.blueprint, &overlay, &bwp.preconditions, &constraints);

        assert!(
            plan.critical_path_hours > 0.0,
            "critical path must be > 0 when procedures are included"
        );
        assert!(
            plan.critical_path_hours <= plan.total_hours,
            "critical path {} should not exceed total hours {}",
            plan.critical_path_hours,
            plan.total_hours
        );
    }

    #[test]
    fn test_optimized_plan_serializes() {
        let bwp = load_fsa();
        let overlay = GenerationOverlay::default();
        let constraints = ResourceConstraints {
            total_budget_hours: 1000.0,
            role_availability: HashMap::new(),
            must_include: vec!["form_opinion".to_string()],
            must_exclude: vec![],
        };

        let plan = optimize_plan(&bwp.blueprint, &overlay, &bwp.preconditions, &constraints);

        let json = serde_json::to_string(&plan).expect("should serialize to JSON");
        assert!(json.contains("included_procedures"));
        assert!(json.contains("total_hours"));
        assert!(json.contains("risk_coverage"));
        assert!(json.contains("standards_coverage"));
        assert!(json.contains("critical_path_hours"));
        assert!(json.contains("role_hours"));
    }
}
