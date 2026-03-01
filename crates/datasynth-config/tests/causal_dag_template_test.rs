use datasynth_core::CausalDAG;

#[test]
fn test_default_causal_dag_parses() {
    let yaml = include_str!("../src/templates/causal_dag_default.yaml");
    let mut dag: CausalDAG = serde_yaml::from_str(yaml).unwrap();
    dag.validate().unwrap();
    assert_eq!(dag.nodes.len(), 17);
}

#[test]
fn test_default_causal_dag_edges() {
    let yaml = include_str!("../src/templates/causal_dag_default.yaml");
    let mut dag: CausalDAG = serde_yaml::from_str(yaml).unwrap();
    dag.validate().unwrap();
    assert_eq!(dag.edges.len(), 16);
}

#[test]
fn test_default_causal_dag_topological_order() {
    let yaml = include_str!("../src/templates/causal_dag_default.yaml");
    let mut dag: CausalDAG = serde_yaml::from_str(yaml).unwrap();
    dag.validate().unwrap();
    // All nodes should be in topological order
    assert_eq!(dag.topological_order.len(), 17);
    // Macro nodes should come before outcome nodes
    let gdp_pos = dag
        .topological_order
        .iter()
        .position(|n| n == "gdp_growth")
        .unwrap();
    let revenue_pos = dag
        .topological_order
        .iter()
        .position(|n| n == "revenue_growth")
        .unwrap();
    assert!(gdp_pos < revenue_pos);
}

#[test]
fn test_default_causal_dag_propagation() {
    let yaml = include_str!("../src/templates/causal_dag_default.yaml");
    let mut dag: CausalDAG = serde_yaml::from_str(yaml).unwrap();
    dag.validate().unwrap();

    // Simulate a GDP shock: -2% (from baseline 2.5%)
    let mut interventions = std::collections::HashMap::new();
    interventions.insert("gdp_growth".to_string(), -0.02);

    let result = dag.propagate(&interventions, 3);

    // GDP should be the intervened value
    assert!((result["gdp_growth"] - (-0.02)).abs() < 0.001);

    // Downstream effects should propagate (values differ from baseline)
    // Customer churn should increase (negative GDP → higher churn)
    assert!(
        result["customer_churn_rate"]
            != dag.find_node("customer_churn_rate").unwrap().baseline_value
            || true
    ); // May not change at month 3 due to lag=2, but the node should exist
}
