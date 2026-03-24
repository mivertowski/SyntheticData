//! Shortest-path analysis for audit procedure FSMs.
//!
//! Uses BFS on each procedure's `ProcedureAggregate` to find the minimum
//! number of transitions required to move from `initial_state` to any
//! terminal state (a state with no outgoing transitions).

use std::collections::{HashMap, HashSet, VecDeque};

use serde::Serialize;

use datasynth_audit_fsm::schema::AuditBlueprint;

// ---------------------------------------------------------------------------
// Report types
// ---------------------------------------------------------------------------

/// The minimum-transition path through a single procedure's FSM.
#[derive(Debug, Clone, Serialize)]
pub struct ProcedurePath {
    /// Ordered sequence of states visited, from `initial_state` to terminal.
    pub states: Vec<String>,
    /// Number of transitions taken (= `states.len() - 1`).
    pub transition_count: usize,
    /// Commands used along the path (one per transition; empty string when the
    /// transition has no associated command).
    pub commands: Vec<String>,
}

/// Aggregate shortest-path report across all procedures in a blueprint.
#[derive(Debug, Clone, Serialize)]
pub struct ShortestPathReport {
    /// Per-procedure minimum paths, keyed by procedure id.
    pub procedure_paths: HashMap<String, ProcedurePath>,
    /// Sum of `transition_count` across all procedures.
    pub total_minimum_transitions: usize,
}

// ---------------------------------------------------------------------------
// Analysis
// ---------------------------------------------------------------------------

/// Analyse every procedure in `blueprint` and return the shortest path from
/// `initial_state` to any terminal state for each procedure that has a
/// non-empty aggregate.
///
/// A terminal state is any state that has no outgoing transitions in the
/// procedure's FSM.  BFS guarantees the first path found to a terminal state
/// is the shortest.
pub fn analyze_shortest_paths(blueprint: &AuditBlueprint) -> ShortestPathReport {
    let mut procedure_paths: HashMap<String, ProcedurePath> = HashMap::new();

    for phase in &blueprint.phases {
        for procedure in &phase.procedures {
            let agg = &procedure.aggregate;

            // Skip procedures with no FSM content.
            if agg.transitions.is_empty() && agg.initial_state.is_empty() {
                continue;
            }

            // ------------------------------------------------------------------
            // Build adjacency list: state -> Vec<(to_state, command)>
            // ------------------------------------------------------------------
            let mut adj: HashMap<&str, Vec<(&str, &str)>> = HashMap::new();

            // Ensure every declared state has an entry (even if it has no
            // outgoing transitions) so we can detect terminal states correctly.
            for state in &agg.states {
                adj.entry(state.as_str()).or_default();
            }

            for transition in &agg.transitions {
                // Also lazily create entries for states only referenced in
                // transitions (not in the explicit states list).
                adj.entry(transition.from_state.as_str()).or_default();
                adj.entry(transition.to_state.as_str()).or_default();

                let cmd = transition.command.as_deref().unwrap_or("");
                adj.entry(transition.from_state.as_str())
                    .or_default()
                    .push((transition.to_state.as_str(), cmd));
            }

            let initial = agg.initial_state.as_str();

            // Nothing to do if there is no initial state or it is not in the
            // adjacency map.
            if initial.is_empty() || !adj.contains_key(initial) {
                continue;
            }

            // ------------------------------------------------------------------
            // Identify terminal states (no outgoing transitions).
            // ------------------------------------------------------------------
            let terminal_states: HashSet<&str> = adj
                .iter()
                .filter(|(_, neighbours)| neighbours.is_empty())
                .map(|(state, _)| *state)
                .collect();

            if terminal_states.is_empty() {
                // No terminal state exists; skip this procedure.
                continue;
            }

            // ------------------------------------------------------------------
            // BFS from initial_state.
            // Each queue entry: (current_state, path_of_states, path_of_commands)
            // ------------------------------------------------------------------
            // Store (state, Vec<state>, Vec<command>) per BFS node.
            let mut visited: HashSet<&str> = HashSet::new();
            let mut queue: VecDeque<(&str, Vec<&str>, Vec<&str>)> = VecDeque::new();

            visited.insert(initial);
            queue.push_back((initial, vec![initial], vec![]));

            let mut best: Option<ProcedurePath> = None;

            'bfs: while let Some((current, states_path, commands_path)) = queue.pop_front() {
                if terminal_states.contains(current) {
                    let transition_count = commands_path.len();
                    best = Some(ProcedurePath {
                        states: states_path.iter().map(|s| s.to_string()).collect(),
                        transition_count,
                        commands: commands_path.iter().map(|c| c.to_string()).collect(),
                    });
                    break 'bfs;
                }

                if let Some(neighbours) = adj.get(current) {
                    for &(next_state, cmd) in neighbours {
                        if !visited.contains(next_state) {
                            visited.insert(next_state);
                            let mut new_states = states_path.clone();
                            new_states.push(next_state);
                            let mut new_cmds = commands_path.clone();
                            new_cmds.push(cmd);
                            queue.push_back((next_state, new_states, new_cmds));
                        }
                    }
                }
            }

            if let Some(path) = best {
                procedure_paths.insert(procedure.id.clone(), path);
            }
        }
    }

    let total_minimum_transitions = procedure_paths.values().map(|p| p.transition_count).sum();

    ShortestPathReport {
        procedure_paths,
        total_minimum_transitions,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_audit_fsm::loader::BlueprintWithPreconditions;

    #[test]
    fn test_fsa_shortest_paths() {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let report = analyze_shortest_paths(&bwp.blueprint);

        assert!(
            !report.procedure_paths.is_empty(),
            "Expected at least one procedure path in the FSA blueprint, got none"
        );

        for (proc_id, path) in &report.procedure_paths {
            assert!(
                path.transition_count >= 2,
                "Procedure '{}' path has {} transitions; expected >= 2",
                proc_id,
                path.transition_count
            );
            assert_eq!(
                path.states.len(),
                path.transition_count + 1,
                "Procedure '{}': states.len() ({}) should equal transition_count + 1 ({})",
                proc_id,
                path.states.len(),
                path.transition_count + 1,
            );
            assert_eq!(
                path.commands.len(),
                path.transition_count,
                "Procedure '{}': commands.len() ({}) should equal transition_count ({})",
                proc_id,
                path.commands.len(),
                path.transition_count,
            );
        }
    }

    #[test]
    fn test_shortest_path_report_serializes() {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let report = analyze_shortest_paths(&bwp.blueprint);
        let json = serde_json::to_string(&report).expect("serialization should succeed");
        assert!(
            json.contains("procedure_paths"),
            "Serialized JSON should contain 'procedure_paths'"
        );
    }
}
