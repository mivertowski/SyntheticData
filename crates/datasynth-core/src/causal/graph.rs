use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Type of a causal variable.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CausalVarType {
    #[default]
    Continuous,
    Categorical,
    Count,
    Binary,
}

/// A variable in the causal graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalVariable {
    pub name: String,
    #[serde(default)]
    pub var_type: CausalVarType,
    /// Base distribution for exogenous noise (e.g., "normal", "lognormal", "beta").
    #[serde(default)]
    pub distribution: Option<String>,
    /// Distribution parameters.
    #[serde(default)]
    pub params: HashMap<String, f64>,
}

impl CausalVariable {
    pub fn new(name: impl Into<String>, var_type: CausalVarType) -> Self {
        Self {
            name: name.into(),
            var_type,
            distribution: None,
            params: HashMap::new(),
        }
    }

    pub fn with_distribution(mut self, dist: impl Into<String>) -> Self {
        self.distribution = Some(dist.into());
        self
    }

    pub fn with_param(mut self, key: impl Into<String>, value: f64) -> Self {
        self.params.insert(key.into(), value);
        self
    }
}

/// Causal mechanism defining how a parent influences a child.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CausalMechanism {
    /// Linear: child += coefficient * parent
    Linear { coefficient: f64 },
    /// Threshold: child = 1 if parent > cutoff else 0
    Threshold { cutoff: f64 },
    /// Polynomial: child += sum(coeff[i] * parent^i)
    Polynomial { coefficients: Vec<f64> },
    /// Logistic: child += 1 / (1 + exp(-scale * (parent - midpoint)))
    Logistic { scale: f64, midpoint: f64 },
}

impl CausalMechanism {
    /// Apply this mechanism to compute the contribution from a parent value.
    pub fn apply(&self, parent_value: f64) -> f64 {
        match self {
            CausalMechanism::Linear { coefficient } => coefficient * parent_value,
            CausalMechanism::Threshold { cutoff } => {
                if parent_value > *cutoff {
                    1.0
                } else {
                    0.0
                }
            }
            CausalMechanism::Polynomial { coefficients } => coefficients
                .iter()
                .enumerate()
                .map(|(i, c)| c * parent_value.powi(i as i32))
                .sum(),
            CausalMechanism::Logistic { scale, midpoint } => {
                1.0 / (1.0 + (-scale * (parent_value - midpoint)).exp())
            }
        }
    }
}

/// A directed edge in the causal graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEdge {
    pub from: String,
    pub to: String,
    pub mechanism: CausalMechanism,
    #[serde(default = "default_strength")]
    pub strength: f64,
}

fn default_strength() -> f64 {
    1.0
}

/// A causal directed acyclic graph (DAG).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalGraph {
    pub variables: Vec<CausalVariable>,
    pub edges: Vec<CausalEdge>,
}

impl CausalGraph {
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_variable(&mut self, var: CausalVariable) {
        self.variables.push(var);
    }

    pub fn add_edge(&mut self, edge: CausalEdge) {
        self.edges.push(edge);
    }

    /// Get variable names.
    pub fn variable_names(&self) -> Vec<&str> {
        self.variables.iter().map(|v| v.name.as_str()).collect()
    }

    /// Get variable by name.
    pub fn get_variable(&self, name: &str) -> Option<&CausalVariable> {
        self.variables.iter().find(|v| v.name == name)
    }

    /// Get all edges pointing TO a given variable (its parents).
    pub fn parent_edges(&self, variable: &str) -> Vec<&CausalEdge> {
        self.edges.iter().filter(|e| e.to == variable).collect()
    }

    /// Validate the graph: check acyclicity, no self-loops, all referenced vars exist.
    pub fn validate(&self) -> Result<(), String> {
        let var_names: HashSet<&str> = self.variables.iter().map(|v| v.name.as_str()).collect();

        // Check for self-loops
        for edge in &self.edges {
            if edge.from == edge.to {
                return Err(format!("Self-loop detected on variable '{}'", edge.from));
            }
        }

        // Check all referenced variables exist
        for edge in &self.edges {
            if !var_names.contains(edge.from.as_str()) {
                return Err(format!("Edge references unknown variable '{}'", edge.from));
            }
            if !var_names.contains(edge.to.as_str()) {
                return Err(format!("Edge references unknown variable '{}'", edge.to));
            }
        }

        // Check acyclicity via topological sort
        self.topological_order().map(|_| ())
    }

    /// Compute topological ordering of variables. Returns error if cyclic.
    pub fn topological_order(&self) -> Result<Vec<String>, String> {
        let var_names: Vec<String> = self.variables.iter().map(|v| v.name.clone()).collect();
        let n = var_names.len();
        let name_to_idx: HashMap<&str, usize> = var_names
            .iter()
            .enumerate()
            .map(|(i, n)| (n.as_str(), i))
            .collect();

        // Build adjacency and in-degree
        let mut in_degree = vec![0usize; n];
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];

        for edge in &self.edges {
            if let (Some(&from_idx), Some(&to_idx)) = (
                name_to_idx.get(edge.from.as_str()),
                name_to_idx.get(edge.to.as_str()),
            ) {
                adj[from_idx].push(to_idx);
                in_degree[to_idx] += 1;
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<usize> = VecDeque::new();
        for (i, &deg) in in_degree.iter().enumerate() {
            if deg == 0 {
                queue.push_back(i);
            }
        }

        let mut order = Vec::with_capacity(n);
        while let Some(node) = queue.pop_front() {
            order.push(var_names[node].clone());
            for &neighbor in &adj[node] {
                in_degree[neighbor] -= 1;
                if in_degree[neighbor] == 0 {
                    queue.push_back(neighbor);
                }
            }
        }

        if order.len() != n {
            Err("Causal graph contains a cycle".to_string())
        } else {
            Ok(order)
        }
    }

    /// Built-in fraud detection SCM template.
    pub fn fraud_detection_template() -> Self {
        let mut graph = Self::new();
        graph.add_variable(
            CausalVariable::new("transaction_amount", CausalVarType::Continuous)
                .with_distribution("lognormal")
                .with_param("mu", 6.0)
                .with_param("sigma", 1.5),
        );
        graph.add_variable(
            CausalVariable::new("merchant_risk", CausalVarType::Continuous)
                .with_distribution("beta")
                .with_param("alpha", 2.0)
                .with_param("beta_param", 5.0),
        );
        graph.add_variable(
            CausalVariable::new("transaction_frequency", CausalVarType::Count)
                .with_distribution("normal")
                .with_param("mean", 10.0)
                .with_param("std", 3.0),
        );
        graph.add_variable(CausalVariable::new(
            "fraud_probability",
            CausalVarType::Continuous,
        ));
        graph.add_variable(CausalVariable::new("is_fraud", CausalVarType::Binary));

        graph.add_edge(CausalEdge {
            from: "transaction_amount".into(),
            to: "fraud_probability".into(),
            mechanism: CausalMechanism::Linear { coefficient: 0.3 },
            strength: 1.0,
        });
        graph.add_edge(CausalEdge {
            from: "merchant_risk".into(),
            to: "fraud_probability".into(),
            mechanism: CausalMechanism::Linear { coefficient: 0.5 },
            strength: 1.0,
        });
        graph.add_edge(CausalEdge {
            from: "transaction_frequency".into(),
            to: "fraud_probability".into(),
            mechanism: CausalMechanism::Linear { coefficient: 0.2 },
            strength: 1.0,
        });
        graph.add_edge(CausalEdge {
            from: "fraud_probability".into(),
            to: "is_fraud".into(),
            mechanism: CausalMechanism::Threshold { cutoff: 0.7 },
            strength: 1.0,
        });

        graph
    }

    /// Built-in revenue cycle SCM template.
    pub fn revenue_cycle_template() -> Self {
        let mut graph = Self::new();
        graph.add_variable(
            CausalVariable::new("order_volume", CausalVarType::Continuous)
                .with_distribution("normal")
                .with_param("mean", 100.0)
                .with_param("std", 30.0),
        );
        graph.add_variable(
            CausalVariable::new("shipment_rate", CausalVarType::Continuous)
                .with_distribution("beta")
                .with_param("alpha", 8.0)
                .with_param("beta_param", 2.0),
        );
        graph.add_variable(CausalVariable::new(
            "invoice_amount",
            CausalVarType::Continuous,
        ));
        graph.add_variable(CausalVariable::new(
            "collection_rate",
            CausalVarType::Continuous,
        ));

        graph.add_edge(CausalEdge {
            from: "order_volume".into(),
            to: "shipment_rate".into(),
            mechanism: CausalMechanism::Logistic {
                scale: 0.05,
                midpoint: 50.0,
            },
            strength: 1.0,
        });
        graph.add_edge(CausalEdge {
            from: "order_volume".into(),
            to: "invoice_amount".into(),
            mechanism: CausalMechanism::Linear { coefficient: 100.0 },
            strength: 1.0,
        });
        graph.add_edge(CausalEdge {
            from: "shipment_rate".into(),
            to: "invoice_amount".into(),
            mechanism: CausalMechanism::Linear { coefficient: 0.5 },
            strength: 1.0,
        });
        graph.add_edge(CausalEdge {
            from: "invoice_amount".into(),
            to: "collection_rate".into(),
            mechanism: CausalMechanism::Logistic {
                scale: -0.0001,
                midpoint: 5000.0,
            },
            strength: 1.0,
        });

        graph
    }
}

impl Default for CausalGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acyclic_graph_validates() {
        let graph = CausalGraph::fraud_detection_template();
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_cyclic_graph_rejected() {
        let mut graph = CausalGraph::new();
        graph.add_variable(CausalVariable::new("a", CausalVarType::Continuous));
        graph.add_variable(CausalVariable::new("b", CausalVarType::Continuous));
        graph.add_edge(CausalEdge {
            from: "a".into(),
            to: "b".into(),
            mechanism: CausalMechanism::Linear { coefficient: 1.0 },
            strength: 1.0,
        });
        graph.add_edge(CausalEdge {
            from: "b".into(),
            to: "a".into(),
            mechanism: CausalMechanism::Linear { coefficient: 1.0 },
            strength: 1.0,
        });
        assert!(graph.validate().is_err());
    }

    #[test]
    fn test_self_loop_rejected() {
        let mut graph = CausalGraph::new();
        graph.add_variable(CausalVariable::new("a", CausalVarType::Continuous));
        graph.add_edge(CausalEdge {
            from: "a".into(),
            to: "a".into(),
            mechanism: CausalMechanism::Linear { coefficient: 1.0 },
            strength: 1.0,
        });
        let result = graph.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Self-loop"));
    }

    #[test]
    fn test_topological_order() {
        let graph = CausalGraph::fraud_detection_template();
        let order = graph.topological_order().unwrap();
        // Root variables (no parents) should come first
        let amount_pos = order
            .iter()
            .position(|n| n == "transaction_amount")
            .unwrap();
        let fraud_prob_pos = order.iter().position(|n| n == "fraud_probability").unwrap();
        let is_fraud_pos = order.iter().position(|n| n == "is_fraud").unwrap();
        assert!(amount_pos < fraud_prob_pos);
        assert!(fraud_prob_pos < is_fraud_pos);
    }

    #[test]
    fn test_unknown_variable_rejected() {
        let mut graph = CausalGraph::new();
        graph.add_variable(CausalVariable::new("a", CausalVarType::Continuous));
        graph.add_edge(CausalEdge {
            from: "a".into(),
            to: "nonexistent".into(),
            mechanism: CausalMechanism::Linear { coefficient: 1.0 },
            strength: 1.0,
        });
        assert!(graph.validate().is_err());
    }

    #[test]
    fn test_mechanism_linear() {
        let m = CausalMechanism::Linear { coefficient: 2.0 };
        assert!((m.apply(3.0) - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_mechanism_threshold() {
        let m = CausalMechanism::Threshold { cutoff: 0.5 };
        assert!((m.apply(0.3) - 0.0).abs() < 1e-10);
        assert!((m.apply(0.7) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_mechanism_logistic() {
        let m = CausalMechanism::Logistic {
            scale: 1.0,
            midpoint: 0.0,
        };
        assert!((m.apply(0.0) - 0.5).abs() < 1e-10);
        assert!(m.apply(10.0) > 0.99);
        assert!(m.apply(-10.0) < 0.01);
    }

    #[test]
    fn test_mechanism_polynomial() {
        let m = CausalMechanism::Polynomial {
            coefficients: vec![1.0, 2.0, 3.0],
        };
        // 1 + 2*x + 3*x^2 at x=2 = 1 + 4 + 12 = 17
        assert!((m.apply(2.0) - 17.0).abs() < 1e-10);
    }

    #[test]
    fn test_revenue_cycle_validates() {
        let graph = CausalGraph::revenue_cycle_template();
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_graph_serde_roundtrip() {
        let graph = CausalGraph::fraud_detection_template();
        let json = serde_json::to_string(&graph).unwrap();
        let deserialized: CausalGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.variables.len(), graph.variables.len());
        assert_eq!(deserialized.edges.len(), graph.edges.len());
    }
}
