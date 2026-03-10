use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;

/// A directed acyclic graph defining causal relationships
/// between parameters in the generation model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalDAG {
    pub nodes: Vec<CausalNode>,
    pub edges: Vec<CausalEdge>,
    /// Pre-computed topological order (filled at validation time).
    #[serde(skip)]
    pub topological_order: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalNode {
    /// Unique identifier (matches config parameter path or abstract name).
    pub id: String,
    pub label: String,
    pub category: NodeCategory,
    /// Default/baseline value.
    pub baseline_value: f64,
    /// Valid range for this parameter.
    pub bounds: Option<(f64, f64)>,
    /// Whether this node can be directly intervened upon.
    #[serde(default = "default_true")]
    pub interventionable: bool,
    /// Maps to config path(s) for actual generation parameters.
    #[serde(default)]
    pub config_bindings: Vec<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeCategory {
    Macro,
    Operational,
    Control,
    Financial,
    Behavioral,
    Regulatory,
    Outcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEdge {
    pub from: String,
    pub to: String,
    pub transfer: TransferFunction,
    /// Delay in months before the effect propagates.
    #[serde(default)]
    pub lag_months: u32,
    /// Strength multiplier (0.0 = no effect, 1.0 = full transfer).
    #[serde(default = "default_strength")]
    pub strength: f64,
    /// Human-readable description of the causal mechanism.
    pub mechanism: Option<String>,
}

fn default_strength() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransferFunction {
    /// output = input * coefficient + intercept
    Linear {
        coefficient: f64,
        #[serde(default)]
        intercept: f64,
    },
    /// output = base * (1 + rate)^input
    Exponential { base: f64, rate: f64 },
    /// output = capacity / (1 + e^(-steepness * (input - midpoint)))
    Logistic {
        capacity: f64,
        midpoint: f64,
        steepness: f64,
    },
    /// output = capacity / (1 + e^(steepness * (input - midpoint)))
    InverseLogistic {
        capacity: f64,
        midpoint: f64,
        steepness: f64,
    },
    /// output = magnitude when input crosses threshold, else 0
    Step { threshold: f64, magnitude: f64 },
    /// output = magnitude when input > threshold, scaling linearly above
    Threshold {
        threshold: f64,
        magnitude: f64,
        #[serde(default = "default_saturation")]
        saturation: f64,
    },
    /// output = initial * e^(-decay_rate * input)
    Decay { initial: f64, decay_rate: f64 },
    /// Lookup table with linear interpolation between points.
    Piecewise { points: Vec<(f64, f64)> },
}

fn default_saturation() -> f64 {
    f64::INFINITY
}

impl TransferFunction {
    /// Compute the output value for a given input.
    pub fn compute(&self, input: f64) -> f64 {
        match self {
            TransferFunction::Linear {
                coefficient,
                intercept,
            } => input * coefficient + intercept,

            TransferFunction::Exponential { base, rate } => base * (1.0 + rate).powf(input),

            TransferFunction::Logistic {
                capacity,
                midpoint,
                steepness,
            } => capacity / (1.0 + (-steepness * (input - midpoint)).exp()),

            TransferFunction::InverseLogistic {
                capacity,
                midpoint,
                steepness,
            } => capacity / (1.0 + (steepness * (input - midpoint)).exp()),

            TransferFunction::Step {
                threshold,
                magnitude,
            } => {
                if input > *threshold {
                    *magnitude
                } else {
                    0.0
                }
            }

            TransferFunction::Threshold {
                threshold,
                magnitude,
                saturation,
            } => {
                if input > *threshold {
                    (magnitude * (input - threshold) / threshold).min(*saturation)
                } else {
                    0.0
                }
            }

            TransferFunction::Decay {
                initial,
                decay_rate,
            } => initial * (-decay_rate * input).exp(),

            TransferFunction::Piecewise { points } => {
                if points.is_empty() {
                    return 0.0;
                }
                if points.len() == 1 {
                    return points[0].1;
                }

                // Sort points by x for interpolation
                let mut sorted = points.clone();
                sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

                // Clamp to range
                if input <= sorted[0].0 {
                    return sorted[0].1;
                }
                if input >= sorted[sorted.len() - 1].0 {
                    return sorted[sorted.len() - 1].1;
                }

                // Linear interpolation
                for window in sorted.windows(2) {
                    let (x0, y0) = window[0];
                    let (x1, y1) = window[1];
                    if input >= x0 && input <= x1 {
                        let t = (input - x0) / (x1 - x0);
                        return y0 + t * (y1 - y0);
                    }
                }

                sorted[sorted.len() - 1].1
            }
        }
    }
}

/// Errors that can occur during CausalDAG operations.
#[derive(Debug, Error)]
pub enum CausalDAGError {
    #[error("cycle detected in causal DAG")]
    CycleDetected,
    #[error("unknown node referenced in edge: {0}")]
    UnknownNode(String),
    #[error("duplicate node ID: {0}")]
    DuplicateNode(String),
    #[error("node '{0}' is not interventionable")]
    NonInterventionable(String),
}

impl CausalDAG {
    /// Validate the graph is a DAG (no cycles) and compute topological order.
    pub fn validate(&mut self) -> Result<(), CausalDAGError> {
        let node_ids: HashSet<&str> = self.nodes.iter().map(|n| n.id.as_str()).collect();

        // Check for duplicate IDs
        let mut seen = HashSet::new();
        for node in &self.nodes {
            if !seen.insert(&node.id) {
                return Err(CausalDAGError::DuplicateNode(node.id.clone()));
            }
        }

        // Check for unknown nodes in edges
        for edge in &self.edges {
            if !node_ids.contains(edge.from.as_str()) {
                return Err(CausalDAGError::UnknownNode(edge.from.clone()));
            }
            if !node_ids.contains(edge.to.as_str()) {
                return Err(CausalDAGError::UnknownNode(edge.to.clone()));
            }
        }

        // Kahn's algorithm for topological sort
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();

        for node in &self.nodes {
            in_degree.insert(&node.id, 0);
            adjacency.insert(&node.id, Vec::new());
        }

        for edge in &self.edges {
            *in_degree.entry(&edge.to).or_insert(0) += 1;
            adjacency.entry(&edge.from).or_default().push(&edge.to);
        }

        let mut queue: VecDeque<&str> = VecDeque::new();
        for (node, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node);
            }
        }

        let mut order = Vec::new();
        while let Some(node) = queue.pop_front() {
            order.push(node.to_string());
            if let Some(neighbors) = adjacency.get(node) {
                for &neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        if order.len() != self.nodes.len() {
            return Err(CausalDAGError::CycleDetected);
        }

        self.topological_order = order;
        Ok(())
    }

    /// Find a node by its ID.
    pub fn find_node(&self, id: &str) -> Option<&CausalNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Given a set of interventions (node_id → new_value), propagate
    /// effects through the DAG in topological order.
    pub fn propagate(
        &self,
        interventions: &HashMap<String, f64>,
        month: u32,
    ) -> HashMap<String, f64> {
        let mut values: HashMap<String, f64> = HashMap::new();

        // Initialize all nodes with baseline values
        for node in &self.nodes {
            values.insert(node.id.clone(), node.baseline_value);
        }

        // Override with direct interventions
        for (node_id, value) in interventions {
            values.insert(node_id.clone(), *value);
        }

        // Build edge lookup: to_node -> list of (from_node, edge)
        let mut incoming: HashMap<&str, Vec<&CausalEdge>> = HashMap::new();
        for edge in &self.edges {
            incoming.entry(&edge.to).or_default().push(edge);
        }

        // Propagate in topological order
        for node_id in &self.topological_order {
            // Skip nodes that are directly intervened upon
            if interventions.contains_key(node_id) {
                continue;
            }

            if let Some(edges) = incoming.get(node_id.as_str()) {
                let mut total_effect = 0.0;
                let mut has_effect = false;

                for edge in edges {
                    // Check lag: only apply if enough months have passed
                    if month < edge.lag_months {
                        continue;
                    }

                    let from_value = values.get(&edge.from).copied().unwrap_or(0.0);
                    let baseline = self
                        .find_node(&edge.from)
                        .map(|n| n.baseline_value)
                        .unwrap_or(0.0);

                    // Compute the delta from baseline
                    let delta = from_value - baseline;
                    if delta.abs() < f64::EPSILON {
                        continue;
                    }

                    // Apply transfer function to the delta
                    let effect = edge.transfer.compute(delta) * edge.strength;
                    total_effect += effect;
                    has_effect = true;
                }

                if has_effect {
                    let baseline = self
                        .find_node(node_id)
                        .map(|n| n.baseline_value)
                        .unwrap_or(0.0);
                    let mut new_value = baseline + total_effect;

                    // Clamp to bounds
                    if let Some(node) = self.find_node(node_id) {
                        if let Some((min, max)) = node.bounds {
                            new_value = new_value.clamp(min, max);
                        }
                    }

                    values.insert(node_id.clone(), new_value);
                }
            }
        }

        values
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_node(id: &str, baseline: f64) -> CausalNode {
        CausalNode {
            id: id.to_string(),
            label: id.to_string(),
            category: NodeCategory::Operational,
            baseline_value: baseline,
            bounds: None,
            interventionable: true,
            config_bindings: vec![],
        }
    }

    fn make_edge(from: &str, to: &str, transfer: TransferFunction) -> CausalEdge {
        CausalEdge {
            from: from.to_string(),
            to: to.to_string(),
            transfer,
            lag_months: 0,
            strength: 1.0,
            mechanism: None,
        }
    }

    #[test]
    fn test_transfer_function_linear() {
        let tf = TransferFunction::Linear {
            coefficient: 0.5,
            intercept: 1.0,
        };
        let result = tf.compute(2.0);
        assert!((result - 2.0).abs() < f64::EPSILON); // 2.0 * 0.5 + 1.0 = 2.0
    }

    #[test]
    fn test_transfer_function_logistic() {
        let tf = TransferFunction::Logistic {
            capacity: 1.0,
            midpoint: 0.0,
            steepness: 1.0,
        };
        // At midpoint, logistic returns capacity/2
        let result = tf.compute(0.0);
        assert!((result - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_transfer_function_exponential() {
        let tf = TransferFunction::Exponential {
            base: 1.0,
            rate: 1.0,
        };
        // base * (1 + rate)^input = 1.0 * 2.0^3.0 = 8.0
        let result = tf.compute(3.0);
        assert!((result - 8.0).abs() < 0.001);
    }

    #[test]
    fn test_transfer_function_step() {
        let tf = TransferFunction::Step {
            threshold: 5.0,
            magnitude: 10.0,
        };
        assert!((tf.compute(3.0) - 0.0).abs() < f64::EPSILON);
        assert!((tf.compute(6.0) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_transfer_function_threshold() {
        let tf = TransferFunction::Threshold {
            threshold: 2.0,
            magnitude: 10.0,
            saturation: f64::INFINITY,
        };
        assert!((tf.compute(1.0) - 0.0).abs() < f64::EPSILON); // below threshold
                                                               // Above threshold: 10.0 * (3.0 - 2.0) / 2.0 = 5.0
        assert!((tf.compute(3.0) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_transfer_function_decay() {
        let tf = TransferFunction::Decay {
            initial: 100.0,
            decay_rate: 0.5,
        };
        // At input=0: 100.0 * e^0 = 100.0
        assert!((tf.compute(0.0) - 100.0).abs() < 0.001);
        // At input=1: 100.0 * e^(-0.5) ≈ 60.65
        assert!((tf.compute(1.0) - 60.653).abs() < 0.1);
    }

    #[test]
    fn test_transfer_function_piecewise() {
        let tf = TransferFunction::Piecewise {
            points: vec![(0.0, 0.0), (1.0, 10.0), (2.0, 15.0)],
        };
        // At 0.5: interpolate between (0,0) and (1,10) → 5.0
        assert!((tf.compute(0.5) - 5.0).abs() < 0.001);
        // At 1.5: interpolate between (1,10) and (2,15) → 12.5
        assert!((tf.compute(1.5) - 12.5).abs() < 0.001);
        // Below range: clamp to first point
        assert!((tf.compute(-1.0) - 0.0).abs() < 0.001);
        // Above range: clamp to last point
        assert!((tf.compute(3.0) - 15.0).abs() < 0.001);
    }

    #[test]
    fn test_dag_validate_acyclic() {
        let mut dag = CausalDAG {
            nodes: vec![
                make_node("a", 1.0),
                make_node("b", 2.0),
                make_node("c", 3.0),
            ],
            edges: vec![
                make_edge(
                    "a",
                    "b",
                    TransferFunction::Linear {
                        coefficient: 1.0,
                        intercept: 0.0,
                    },
                ),
                make_edge(
                    "b",
                    "c",
                    TransferFunction::Linear {
                        coefficient: 1.0,
                        intercept: 0.0,
                    },
                ),
            ],
            topological_order: vec![],
        };
        assert!(dag.validate().is_ok());
        assert_eq!(dag.topological_order, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_dag_validate_cycle_detected() {
        let mut dag = CausalDAG {
            nodes: vec![make_node("a", 1.0), make_node("b", 2.0)],
            edges: vec![
                make_edge(
                    "a",
                    "b",
                    TransferFunction::Linear {
                        coefficient: 1.0,
                        intercept: 0.0,
                    },
                ),
                make_edge(
                    "b",
                    "a",
                    TransferFunction::Linear {
                        coefficient: 1.0,
                        intercept: 0.0,
                    },
                ),
            ],
            topological_order: vec![],
        };
        assert!(matches!(dag.validate(), Err(CausalDAGError::CycleDetected)));
    }

    #[test]
    fn test_dag_validate_unknown_node() {
        let mut dag = CausalDAG {
            nodes: vec![make_node("a", 1.0)],
            edges: vec![make_edge(
                "a",
                "nonexistent",
                TransferFunction::Linear {
                    coefficient: 1.0,
                    intercept: 0.0,
                },
            )],
            topological_order: vec![],
        };
        assert!(matches!(
            dag.validate(),
            Err(CausalDAGError::UnknownNode(_))
        ));
    }

    #[test]
    fn test_dag_validate_duplicate_node() {
        let mut dag = CausalDAG {
            nodes: vec![make_node("a", 1.0), make_node("a", 2.0)],
            edges: vec![],
            topological_order: vec![],
        };
        assert!(matches!(
            dag.validate(),
            Err(CausalDAGError::DuplicateNode(_))
        ));
    }

    #[test]
    fn test_dag_propagate_chain() {
        let mut dag = CausalDAG {
            nodes: vec![
                make_node("a", 10.0),
                make_node("b", 5.0),
                make_node("c", 0.0),
            ],
            edges: vec![
                make_edge(
                    "a",
                    "b",
                    TransferFunction::Linear {
                        coefficient: 0.5,
                        intercept: 0.0,
                    },
                ),
                make_edge(
                    "b",
                    "c",
                    TransferFunction::Linear {
                        coefficient: 1.0,
                        intercept: 0.0,
                    },
                ),
            ],
            topological_order: vec![],
        };
        dag.validate().unwrap();

        // Intervene on A: set to 20.0 (delta = 10.0)
        let mut interventions = HashMap::new();
        interventions.insert("a".to_string(), 20.0);

        let result = dag.propagate(&interventions, 0);
        // A = 20.0 (directly set)
        assert!((result["a"] - 20.0).abs() < 0.001);
        // B baseline = 5.0, delta_a = 10.0, transfer = 10.0 * 0.5 + 0.0 = 5.0 → B = 5.0 + 5.0 = 10.0
        assert!((result["b"] - 10.0).abs() < 0.001);
        // C baseline = 0.0, delta_b = 5.0, transfer = 5.0 * 1.0 + 0.0 = 5.0 → C = 0.0 + 5.0 = 5.0
        assert!((result["c"] - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_dag_propagate_with_lag() {
        let mut dag = CausalDAG {
            nodes: vec![make_node("a", 10.0), make_node("b", 5.0)],
            edges: vec![CausalEdge {
                from: "a".to_string(),
                to: "b".to_string(),
                transfer: TransferFunction::Linear {
                    coefficient: 1.0,
                    intercept: 0.0,
                },
                lag_months: 2,
                strength: 1.0,
                mechanism: None,
            }],
            topological_order: vec![],
        };
        dag.validate().unwrap();

        let mut interventions = HashMap::new();
        interventions.insert("a".to_string(), 20.0);

        // Month 1: lag is 2, so no effect yet
        let result = dag.propagate(&interventions, 1);
        assert!((result["b"] - 5.0).abs() < 0.001); // unchanged from baseline

        // Month 2: lag is met, effect propagates
        let result = dag.propagate(&interventions, 2);
        // delta_a = 10.0, transfer = 10.0, B = 5.0 + 10.0 = 15.0
        assert!((result["b"] - 15.0).abs() < 0.001);
    }

    #[test]
    fn test_dag_propagate_node_bounds_clamped() {
        let mut dag = CausalDAG {
            nodes: vec![make_node("a", 10.0), {
                let mut n = make_node("b", 5.0);
                n.bounds = Some((0.0, 8.0));
                n
            }],
            edges: vec![make_edge(
                "a",
                "b",
                TransferFunction::Linear {
                    coefficient: 1.0,
                    intercept: 0.0,
                },
            )],
            topological_order: vec![],
        };
        dag.validate().unwrap();

        let mut interventions = HashMap::new();
        interventions.insert("a".to_string(), 20.0); // delta = 10.0 → B would be 15.0

        let result = dag.propagate(&interventions, 0);
        // B should be clamped to max bound of 8.0
        assert!((result["b"] - 8.0).abs() < 0.001);
    }

    #[test]
    fn test_transfer_function_serde() {
        let tf = TransferFunction::Linear {
            coefficient: 0.5,
            intercept: 1.0,
        };
        let json = serde_json::to_string(&tf).unwrap();
        let deserialized: TransferFunction = serde_json::from_str(&json).unwrap();
        assert!((deserialized.compute(2.0) - 2.0).abs() < f64::EPSILON);
    }

    // ====================================================================
    // Comprehensive transfer function tests (Task 12)
    // ====================================================================

    #[test]
    fn test_transfer_function_linear_zero_coefficient() {
        let tf = TransferFunction::Linear {
            coefficient: 0.0,
            intercept: 5.0,
        };
        // Any input → just the intercept
        assert!((tf.compute(0.0) - 5.0).abs() < f64::EPSILON);
        assert!((tf.compute(100.0) - 5.0).abs() < f64::EPSILON);
        assert!((tf.compute(-100.0) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_transfer_function_linear_negative_coefficient() {
        let tf = TransferFunction::Linear {
            coefficient: -2.0,
            intercept: 10.0,
        };
        assert!((tf.compute(3.0) - 4.0).abs() < f64::EPSILON); // -6 + 10 = 4
        assert!((tf.compute(5.0) - 0.0).abs() < f64::EPSILON); // -10 + 10 = 0
    }

    #[test]
    fn test_transfer_function_exponential_zero_input() {
        let tf = TransferFunction::Exponential {
            base: 5.0,
            rate: 0.5,
        };
        // (1+0.5)^0 = 1, so result = 5.0
        assert!((tf.compute(0.0) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_transfer_function_exponential_negative_rate() {
        let tf = TransferFunction::Exponential {
            base: 100.0,
            rate: -0.5,
        };
        // (1 + (-0.5))^2 = 0.5^2 = 0.25, result = 25.0
        assert!((tf.compute(2.0) - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_transfer_function_logistic_far_from_midpoint() {
        let tf = TransferFunction::Logistic {
            capacity: 10.0,
            midpoint: 5.0,
            steepness: 2.0,
        };
        // Far below midpoint → near 0
        assert!(tf.compute(-10.0) < 0.01);
        // Far above midpoint → near capacity
        assert!((tf.compute(20.0) - 10.0).abs() < 0.01);
        // At midpoint → capacity/2
        assert!((tf.compute(5.0) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_transfer_function_logistic_steepness_effect() {
        // High steepness → sharper transition
        let steep = TransferFunction::Logistic {
            capacity: 1.0,
            midpoint: 0.0,
            steepness: 10.0,
        };
        let gentle = TransferFunction::Logistic {
            capacity: 1.0,
            midpoint: 0.0,
            steepness: 0.5,
        };
        // Both should be ~0.5 at midpoint
        assert!((steep.compute(0.0) - 0.5).abs() < 0.01);
        assert!((gentle.compute(0.0) - 0.5).abs() < 0.01);
        // Steep should be closer to 1.0 at input=1.0
        assert!(steep.compute(1.0) > gentle.compute(1.0));
    }

    #[test]
    fn test_transfer_function_inverse_logistic() {
        let tf = TransferFunction::InverseLogistic {
            capacity: 1.0,
            midpoint: 0.0,
            steepness: 1.0,
        };
        // At midpoint → capacity/2
        assert!((tf.compute(0.0) - 0.5).abs() < 0.001);
        // Inverse logistic decreases: far above midpoint → near 0
        assert!(tf.compute(10.0) < 0.01);
        // Far below midpoint → near capacity
        assert!((tf.compute(-10.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_transfer_function_inverse_logistic_symmetry() {
        let logistic = TransferFunction::Logistic {
            capacity: 1.0,
            midpoint: 0.0,
            steepness: 1.0,
        };
        let inverse = TransferFunction::InverseLogistic {
            capacity: 1.0,
            midpoint: 0.0,
            steepness: 1.0,
        };
        // Logistic + InverseLogistic should sum to capacity at any point
        for x in [-5.0, -1.0, 0.0, 1.0, 5.0] {
            let sum = logistic.compute(x) + inverse.compute(x);
            assert!((sum - 1.0).abs() < 0.001, "Sum at x={} was {}", x, sum);
        }
    }

    #[test]
    fn test_transfer_function_step_at_threshold() {
        let tf = TransferFunction::Step {
            threshold: 5.0,
            magnitude: 10.0,
        };
        // At exactly the threshold, should be 0 (not strictly greater)
        assert!((tf.compute(5.0) - 0.0).abs() < f64::EPSILON);
        // Just above threshold
        assert!((tf.compute(5.001) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_transfer_function_step_negative_magnitude() {
        let tf = TransferFunction::Step {
            threshold: 0.0,
            magnitude: -5.0,
        };
        assert!((tf.compute(-1.0) - 0.0).abs() < f64::EPSILON);
        assert!((tf.compute(1.0) - (-5.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_transfer_function_threshold_with_saturation() {
        let tf = TransferFunction::Threshold {
            threshold: 2.0,
            magnitude: 10.0,
            saturation: 8.0,
        };
        // Below threshold: 0
        assert!((tf.compute(1.0) - 0.0).abs() < f64::EPSILON);
        // Just above threshold: 10.0 * (2.5 - 2.0) / 2.0 = 2.5
        assert!((tf.compute(2.5) - 2.5).abs() < 0.001);
        // Way above threshold without saturation: 10.0 * (100.0 - 2.0) / 2.0 = 490
        // But capped at saturation=8.0
        assert!((tf.compute(100.0) - 8.0).abs() < 0.001);
    }

    #[test]
    fn test_transfer_function_threshold_infinite_saturation() {
        let tf = TransferFunction::Threshold {
            threshold: 1.0,
            magnitude: 5.0,
            saturation: f64::INFINITY,
        };
        // No saturation cap: grows linearly
        // 5.0 * (100.0 - 1.0) / 1.0 = 495.0
        assert!((tf.compute(100.0) - 495.0).abs() < 0.001);
    }

    #[test]
    fn test_transfer_function_decay_large_input() {
        let tf = TransferFunction::Decay {
            initial: 100.0,
            decay_rate: 1.0,
        };
        // Large input → approaches 0
        assert!(tf.compute(10.0) < 0.01);
        assert!(tf.compute(20.0) < 0.0001);
    }

    #[test]
    fn test_transfer_function_decay_zero_rate() {
        let tf = TransferFunction::Decay {
            initial: 50.0,
            decay_rate: 0.0,
        };
        // No decay → constant
        assert!((tf.compute(0.0) - 50.0).abs() < f64::EPSILON);
        assert!((tf.compute(100.0) - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_transfer_function_piecewise_single_point() {
        let tf = TransferFunction::Piecewise {
            points: vec![(5.0, 42.0)],
        };
        // Single point → always returns that value
        assert!((tf.compute(0.0) - 42.0).abs() < f64::EPSILON);
        assert!((tf.compute(100.0) - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_transfer_function_piecewise_empty() {
        let tf = TransferFunction::Piecewise { points: vec![] };
        assert!((tf.compute(5.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_transfer_function_piecewise_exact_points() {
        let tf = TransferFunction::Piecewise {
            points: vec![(0.0, 0.0), (1.0, 10.0), (2.0, 15.0), (3.0, 30.0)],
        };
        // At exact breakpoints
        assert!((tf.compute(0.0) - 0.0).abs() < 0.001);
        assert!((tf.compute(1.0) - 10.0).abs() < 0.001);
        assert!((tf.compute(2.0) - 15.0).abs() < 0.001);
        assert!((tf.compute(3.0) - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_transfer_function_piecewise_unsorted_points() {
        // Points given out of order — should still interpolate correctly
        let tf = TransferFunction::Piecewise {
            points: vec![(2.0, 20.0), (0.0, 0.0), (1.0, 10.0)],
        };
        assert!((tf.compute(0.5) - 5.0).abs() < 0.001);
        assert!((tf.compute(1.5) - 15.0).abs() < 0.001);
    }
}
