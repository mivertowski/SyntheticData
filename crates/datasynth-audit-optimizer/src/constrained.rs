//! Constraint-based path optimization.

use std::collections::{HashMap, HashSet};

use serde::Serialize;

use datasynth_audit_fsm::schema::AuditBlueprint;

use crate::shortest_path::{analyze_shortest_paths, ProcedurePath, ShortestPathReport};

// ---------------------------------------------------------------------------
// Result type
// ---------------------------------------------------------------------------

/// Output of a constrained-path analysis.
#[derive(Debug, Clone, Serialize)]
pub struct ConstrainedPathResult {
    /// The full set of required procedure ids (must-visit + transitive
    /// preconditions), sorted for deterministic output.
    pub required_procedures: Vec<String>,
    /// Total transitions across all required-procedure paths.
    pub total_transitions: usize,
    /// Shortest-path report filtered to only the required procedures.
    pub paths: ShortestPathReport,
}

// ---------------------------------------------------------------------------
// Analysis
// ---------------------------------------------------------------------------

/// Compute the shortest paths for a constrained set of procedures.
///
/// Starting from `must_visit`, the function expands the required set by
/// following `preconditions` transitively (DFS / BFS over the precondition
/// graph).  It then delegates to [`analyze_shortest_paths`] for the full
/// blueprint and returns only the entries that belong to the required set.
///
/// # Arguments
/// * `blueprint`      – The audit blueprint to analyse.
/// * `must_visit`     – Procedure ids that must be covered.
/// * `preconditions`  – Map from procedure id to its direct preconditions.
pub fn constrained_path(
    blueprint: &AuditBlueprint,
    must_visit: &[String],
    preconditions: &HashMap<String, Vec<String>>,
) -> ConstrainedPathResult {
    // ------------------------------------------------------------------
    // Expand must-visit with transitive preconditions (iterative DFS).
    // ------------------------------------------------------------------
    let mut required: HashSet<String> = must_visit.iter().cloned().collect();
    let mut queue: Vec<String> = must_visit.to_vec();

    while let Some(proc_id) = queue.pop() {
        if let Some(deps) = preconditions.get(&proc_id) {
            for dep in deps {
                if required.insert(dep.clone()) {
                    queue.push(dep.clone());
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Run full shortest-path analysis and filter to the required set.
    // ------------------------------------------------------------------
    let full_paths = analyze_shortest_paths(blueprint);
    let filtered: HashMap<String, ProcedurePath> = full_paths
        .procedure_paths
        .into_iter()
        .filter(|(k, _)| required.contains(k))
        .collect();

    let total = filtered.values().map(|p| p.transition_count).sum();

    let mut required_sorted: Vec<String> = required.into_iter().collect();
    required_sorted.sort();

    ConstrainedPathResult {
        required_procedures: required_sorted,
        total_transitions: total,
        paths: ShortestPathReport {
            procedure_paths: filtered,
            total_minimum_transitions: total,
        },
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

    /// `form_opinion` depends on `going_concern` and `subsequent_events`.
    /// Constraining to `["form_opinion"]` must therefore include all three.
    #[test]
    fn test_constrained_path_expands_preconditions() {
        let bwp = load_fsa();
        let must_visit = vec!["form_opinion".to_string()];

        let result = constrained_path(&bwp.blueprint, &must_visit, &bwp.preconditions);

        assert!(
            result
                .required_procedures
                .contains(&"form_opinion".to_string()),
            "form_opinion must be in required_procedures"
        );
        assert!(
            result
                .required_procedures
                .contains(&"going_concern".to_string()),
            "going_concern must be transitively included"
        );
        assert!(
            result
                .required_procedures
                .contains(&"subsequent_events".to_string()),
            "subsequent_events must be transitively included"
        );
        assert!(
            result.required_procedures.len() >= 3,
            "expected at least 3 required procedures, got {}",
            result.required_procedures.len()
        );
    }

    #[test]
    fn test_constrained_path_paths_are_filtered() {
        let bwp = load_fsa();
        let must_visit = vec!["form_opinion".to_string()];

        let result = constrained_path(&bwp.blueprint, &must_visit, &bwp.preconditions);

        // Every key in the paths map must appear in required_procedures.
        for key in result.paths.procedure_paths.keys() {
            assert!(
                result.required_procedures.contains(key),
                "path key '{}' is not in required_procedures",
                key
            );
        }
    }

    #[test]
    fn test_constrained_path_total_transitions_consistent() {
        let bwp = load_fsa();
        let must_visit = vec!["form_opinion".to_string()];

        let result = constrained_path(&bwp.blueprint, &must_visit, &bwp.preconditions);

        let expected_total: usize = result
            .paths
            .procedure_paths
            .values()
            .map(|p| p.transition_count)
            .sum();
        assert_eq!(
            result.total_transitions, expected_total,
            "total_transitions should equal sum of per-procedure transition counts"
        );
        assert_eq!(
            result.paths.total_minimum_transitions, expected_total,
            "ShortestPathReport.total_minimum_transitions should match"
        );
    }

    #[test]
    fn test_constrained_path_empty_must_visit() {
        let bwp = load_fsa();
        let result = constrained_path(&bwp.blueprint, &[], &bwp.preconditions);

        assert!(
            result.required_procedures.is_empty(),
            "empty must_visit should produce empty required_procedures"
        );
        assert_eq!(result.total_transitions, 0);
    }

    #[test]
    fn test_constrained_path_serializes() {
        let bwp = load_fsa();
        let must_visit = vec!["form_opinion".to_string()];
        let result = constrained_path(&bwp.blueprint, &must_visit, &bwp.preconditions);

        let json = serde_json::to_string(&result).expect("should serialize to JSON");
        assert!(json.contains("required_procedures"));
        assert!(json.contains("total_transitions"));
    }
}
