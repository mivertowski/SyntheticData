//! Convert audit blueprints to petgraph directed graphs.
//!
//! Each node represents a `(procedure_id, state)` pair and each edge
//! represents a transition within a procedure's FSM aggregate.

use std::collections::HashMap;

use petgraph::graph::{DiGraph, NodeIndex};

use datasynth_audit_fsm::schema::AuditBlueprint;

// ---------------------------------------------------------------------------
// Node and edge types
// ---------------------------------------------------------------------------

/// A graph node representing a procedure state.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StateNode {
    /// The procedure this state belongs to.
    pub procedure_id: String,
    /// The state name within the procedure's FSM aggregate.
    pub state: String,
}

/// A graph edge representing a transition between states.
#[derive(Debug, Clone)]
pub struct TransitionEdge {
    /// Command that triggers this transition (if any).
    pub command: Option<String>,
    /// Event emitted when the transition fires (if any).
    pub emits: Option<String>,
    /// Guard predicates that must pass before the transition can fire.
    pub guards: Vec<String>,
}

// ---------------------------------------------------------------------------
// Conversion
// ---------------------------------------------------------------------------

/// Convert an [`AuditBlueprint`] into a petgraph [`DiGraph`].
///
/// Iterates over all phases and their procedures, creating one node per
/// `(procedure_id, state)` pair and one edge per FSM transition.
pub fn blueprint_to_graph(blueprint: &AuditBlueprint) -> DiGraph<StateNode, TransitionEdge> {
    let mut graph = DiGraph::new();
    // Map from (procedure_id, state) -> NodeIndex for deduplication.
    let mut node_map: HashMap<(String, String), NodeIndex> = HashMap::new();

    for phase in &blueprint.phases {
        for procedure in &phase.procedures {
            let agg = &procedure.aggregate;

            // Ensure all declared states have nodes.
            for state in &agg.states {
                let key = (procedure.id.clone(), state.clone());
                node_map.entry(key.clone()).or_insert_with(|| {
                    graph.add_node(StateNode {
                        procedure_id: key.0,
                        state: key.1,
                    })
                });
            }

            // Create edges for each transition.
            for transition in &agg.transitions {
                let from_key = (procedure.id.clone(), transition.from_state.clone());
                let to_key = (procedure.id.clone(), transition.to_state.clone());

                // Lazily create nodes for states referenced in transitions but
                // not listed in the explicit states vector.
                let from_idx = *node_map.entry(from_key.clone()).or_insert_with(|| {
                    graph.add_node(StateNode {
                        procedure_id: from_key.0,
                        state: from_key.1,
                    })
                });
                let to_idx = *node_map.entry(to_key.clone()).or_insert_with(|| {
                    graph.add_node(StateNode {
                        procedure_id: to_key.0,
                        state: to_key.1,
                    })
                });

                graph.add_edge(
                    from_idx,
                    to_idx,
                    TransitionEdge {
                        command: transition.command.clone(),
                        emits: transition.emits.clone(),
                        guards: transition.guards.clone(),
                    },
                );
            }
        }
    }

    graph
}

// ---------------------------------------------------------------------------
// Analysis helpers
// ---------------------------------------------------------------------------

/// Return nodes with no incoming edges (entry points / initial states).
pub fn find_initial_nodes(graph: &DiGraph<StateNode, TransitionEdge>) -> Vec<NodeIndex> {
    graph
        .node_indices()
        .filter(|&idx| {
            graph
                .neighbors_directed(idx, petgraph::Direction::Incoming)
                .next()
                .is_none()
        })
        .collect()
}

/// Return nodes with no outgoing edges (terminal / completed states).
pub fn find_terminal_nodes(graph: &DiGraph<StateNode, TransitionEdge>) -> Vec<NodeIndex> {
    graph
        .node_indices()
        .filter(|&idx| {
            graph
                .neighbors_directed(idx, petgraph::Direction::Outgoing)
                .next()
                .is_none()
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_audit_fsm::loader::BlueprintWithPreconditions;

    #[test]
    fn test_fsa_graph_has_nodes_and_edges() {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let graph = blueprint_to_graph(&bwp.blueprint);
        assert!(
            graph.node_count() > 0,
            "FSA graph should have nodes, got {}",
            graph.node_count()
        );
        assert!(
            graph.edge_count() > 0,
            "FSA graph should have edges, got {}",
            graph.edge_count()
        );
    }

    #[test]
    fn test_fsa_graph_has_initial_and_terminal_nodes() {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let graph = blueprint_to_graph(&bwp.blueprint);
        let initials = find_initial_nodes(&graph);
        let terminals = find_terminal_nodes(&graph);
        assert!(
            !initials.is_empty(),
            "FSA graph should have initial nodes (no incoming edges)"
        );
        assert!(
            !terminals.is_empty(),
            "FSA graph should have terminal nodes (no outgoing edges)"
        );
    }

    #[test]
    fn test_ia_graph_larger_than_fsa() {
        let fsa_bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        let ia_bwp = BlueprintWithPreconditions::load_builtin_ia().unwrap();
        let fsa_graph = blueprint_to_graph(&fsa_bwp.blueprint);
        let ia_graph = blueprint_to_graph(&ia_bwp.blueprint);
        assert!(
            ia_graph.node_count() > fsa_graph.node_count(),
            "IA graph ({} nodes) should have more nodes than FSA graph ({} nodes)",
            ia_graph.node_count(),
            fsa_graph.node_count()
        );
        assert!(
            ia_graph.edge_count() > fsa_graph.edge_count(),
            "IA graph ({} edges) should have more edges than FSA graph ({} edges)",
            ia_graph.edge_count(),
            fsa_graph.edge_count()
        );
    }
}
