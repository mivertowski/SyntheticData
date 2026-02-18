//! Relationship generator implementation.
//!
//! Provides generation of relationships between entities based on
//! cardinality rules and property generation configurations.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};

use super::rules::{
    CardinalityRule, PropertyGenerator, PropertyValueType, RelationshipConfig,
    RelationshipTypeConfig, RelationshipValidation,
};

/// Generated relationship output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedRelationship {
    /// Relationship type name.
    pub relationship_type: String,
    /// Unique relationship ID.
    pub id: String,
    /// Source entity ID.
    pub source_id: String,
    /// Target entity ID.
    pub target_id: String,
    /// Relationship properties.
    pub properties: HashMap<String, Value>,
    /// Relationship metadata.
    pub metadata: RelationshipMetadata,
}

/// Metadata for a generated relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipMetadata {
    /// Data source.
    pub source: String,
    /// Generation timestamp.
    pub generated_at: DateTime<Utc>,
    /// Relationship weight.
    pub weight: Option<f64>,
    /// Valid from timestamp.
    pub valid_from: Option<DateTime<Utc>>,
    /// Valid to timestamp.
    pub valid_to: Option<DateTime<Utc>>,
    /// Custom labels.
    pub labels: HashMap<String, String>,
    /// Feature vector for ML.
    pub features: Option<Vec<f64>>,
    /// Whether the relationship is directed.
    pub is_directed: bool,
}

impl Default for RelationshipMetadata {
    fn default() -> Self {
        Self {
            source: "datasynth".to_string(),
            generated_at: Utc::now(),
            weight: None,
            valid_from: None,
            valid_to: None,
            labels: HashMap::new(),
            features: None,
            is_directed: true,
        }
    }
}

/// Simple node representation for relationship generation.
#[derive(Debug, Clone)]
pub struct NodeRef {
    /// Node ID.
    pub id: String,
    /// Node type.
    pub node_type: String,
    /// Node properties.
    pub properties: HashMap<String, Value>,
}

impl NodeRef {
    /// Creates a new node reference.
    pub fn new(id: impl Into<String>, node_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            node_type: node_type.into(),
            properties: HashMap::new(),
        }
    }

    /// Adds a property.
    pub fn with_property(mut self, key: impl Into<String>, value: Value) -> Self {
        self.properties.insert(key.into(), value);
        self
    }
}

/// Generator for relationships between entities.
pub struct RelationshipGenerator {
    /// Configuration.
    config: RelationshipConfig,
    /// Random number generator.
    rng: ChaCha8Rng,
    /// Deterministic UUID factory.
    uuid_factory: DeterministicUuidFactory,
    /// Generation count.
    count: u64,
    /// Track relationships by source ID for cardinality validation.
    relationships_by_source: HashMap<String, HashMap<String, Vec<String>>>,
    /// Track relationships by target ID for cardinality validation.
    relationships_by_target: HashMap<String, HashMap<String, Vec<String>>>,
    /// Visited nodes for circular detection.
    visited: HashSet<String>,
}

impl RelationshipGenerator {
    /// Creates a new relationship generator.
    pub fn new(config: RelationshipConfig, seed: u64) -> Self {
        Self {
            config,
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::Customer),
            count: 0,
            relationships_by_source: HashMap::new(),
            relationships_by_target: HashMap::new(),
            visited: HashSet::new(),
        }
    }

    /// Creates a generator with default configuration.
    pub fn with_defaults(seed: u64) -> Self {
        Self::new(RelationshipConfig::default(), seed)
    }

    /// Generates relationships for a set of nodes.
    pub fn generate_relationships(&mut self, nodes: &[NodeRef]) -> Vec<GeneratedRelationship> {
        let mut relationships = Vec::new();

        // Group nodes by type
        let nodes_by_type = self.group_nodes_by_type(nodes);

        // Clone relationship types to avoid borrow issues
        let relationship_types = self.config.relationship_types.clone();

        // For each relationship type, generate relationships
        for rel_type in &relationship_types {
            let rels = self.generate_for_type(rel_type, &nodes_by_type);
            relationships.extend(rels);
        }

        relationships
    }

    /// Generates relationships for a single node.
    pub fn generate_for_node(
        &mut self,
        node: &NodeRef,
        available_targets: &HashMap<String, Vec<NodeRef>>,
    ) -> Vec<GeneratedRelationship> {
        // Check for orphan generation
        if self.config.allow_orphans && self.rng.gen_bool(self.config.orphan_probability) {
            return Vec::new();
        }

        let mut relationships = Vec::new();

        // Clone applicable relationship types to avoid borrow issues
        let applicable_types: Vec<_> = self
            .config
            .relationship_types
            .iter()
            .filter(|rt| rt.source_type == node.node_type)
            .cloned()
            .collect();

        for rel_type in &applicable_types {
            if let Some(targets) = available_targets.get(&rel_type.target_type) {
                let rels = self.generate_edges_for_node(node, targets, rel_type);
                relationships.extend(rels);
            }
        }

        relationships
    }

    /// Checks if a relationship would create a valid cardinality.
    pub fn check_cardinality(
        &self,
        source_id: &str,
        target_id: &str,
        rel_type: &str,
    ) -> RelationshipValidation {
        // Find the relationship type config
        let type_config = self
            .config
            .relationship_types
            .iter()
            .find(|rt| rt.name == rel_type);

        let Some(type_config) = type_config else {
            return RelationshipValidation::invalid(format!(
                "Unknown relationship type: {}",
                rel_type
            ));
        };

        let (_min, max) = type_config.cardinality.bounds();

        // Check source-side cardinality
        let current_count = self
            .relationships_by_source
            .get(source_id)
            .and_then(|m| m.get(rel_type))
            .map(|v| v.len())
            .unwrap_or(0);

        if current_count >= max as usize {
            return RelationshipValidation::invalid(format!(
                "Source {} already has maximum {} {} relationships",
                source_id, max, rel_type
            ));
        }

        // For OneToOne and ManyToOne, check if target already has a relationship
        if matches!(
            type_config.cardinality,
            CardinalityRule::OneToOne | CardinalityRule::ManyToOne { .. }
        ) {
            let target_count = self
                .relationships_by_target
                .get(target_id)
                .and_then(|m| m.get(rel_type))
                .map(|v| v.len())
                .unwrap_or(0);

            if target_count > 0 {
                return RelationshipValidation::invalid(format!(
                    "Target {} already has a {} relationship",
                    target_id, rel_type
                ));
            }
        }

        RelationshipValidation::valid()
    }

    /// Checks if a relationship would create a circular reference.
    pub fn check_circular(&mut self, source_id: &str, target_id: &str) -> bool {
        if !self.config.allow_circular {
            // Simple check: direct circular reference
            if source_id == target_id {
                return true;
            }

            // DFS to check for circular paths
            self.visited.clear();
            self.visited.insert(source_id.to_string());

            return self.has_path_to(target_id, source_id, 0);
        }

        false
    }

    /// Returns the number of relationships generated.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Resets the generator.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
        self.uuid_factory = DeterministicUuidFactory::new(seed, GeneratorType::Customer);
        self.count = 0;
        self.relationships_by_source.clear();
        self.relationships_by_target.clear();
        self.visited.clear();
    }

    /// Returns the configuration.
    pub fn config(&self) -> &RelationshipConfig {
        &self.config
    }

    /// Groups nodes by their type.
    fn group_nodes_by_type(&self, nodes: &[NodeRef]) -> HashMap<String, Vec<NodeRef>> {
        let mut grouped: HashMap<String, Vec<NodeRef>> = HashMap::new();

        for node in nodes {
            grouped
                .entry(node.node_type.clone())
                .or_default()
                .push(node.clone());
        }

        grouped
    }

    /// Generates relationships for a specific relationship type.
    fn generate_for_type(
        &mut self,
        rel_type: &RelationshipTypeConfig,
        nodes_by_type: &HashMap<String, Vec<NodeRef>>,
    ) -> Vec<GeneratedRelationship> {
        let mut relationships = Vec::new();

        let Some(source_nodes) = nodes_by_type.get(&rel_type.source_type) else {
            return relationships;
        };

        let Some(target_nodes) = nodes_by_type.get(&rel_type.target_type) else {
            return relationships;
        };

        for source in source_nodes {
            let rels = self.generate_edges_for_node(source, target_nodes, rel_type);
            relationships.extend(rels);
        }

        relationships
    }

    /// Generates edges from a single source node.
    fn generate_edges_for_node(
        &mut self,
        source: &NodeRef,
        targets: &[NodeRef],
        rel_type: &RelationshipTypeConfig,
    ) -> Vec<GeneratedRelationship> {
        let mut relationships = Vec::new();

        if targets.is_empty() {
            return relationships;
        }

        // Determine number of relationships based on cardinality
        let (min, max) = rel_type.cardinality.bounds();
        let count = if min == max {
            min as usize
        } else {
            self.rng.gen_range(min..=max) as usize
        };

        // Filter available targets
        let available_targets: Vec<_> = targets
            .iter()
            .filter(|t| {
                // Check if this relationship is valid
                let validation = self.check_cardinality(&source.id, &t.id, &rel_type.name);
                if !validation.valid {
                    return false;
                }

                // Check for circular references
                if self.check_circular(&source.id, &t.id) {
                    return false;
                }

                true
            })
            .collect();

        if available_targets.is_empty() && rel_type.required {
            // Log warning or handle required relationship with no valid targets
            return relationships;
        }

        // Select targets
        let selected_count = count.min(available_targets.len());
        let mut selected_indices: Vec<usize> = (0..available_targets.len()).collect();
        selected_indices.shuffle(&mut self.rng);
        selected_indices.truncate(selected_count);

        for idx in selected_indices {
            let target = available_targets[idx];
            let relationship = self.create_relationship(source, target, rel_type);

            // Track the relationship for cardinality validation
            self.track_relationship(&source.id, &target.id, &rel_type.name);

            relationships.push(relationship);
        }

        relationships
    }

    /// Creates a single relationship.
    fn create_relationship(
        &mut self,
        source: &NodeRef,
        target: &NodeRef,
        rel_type: &RelationshipTypeConfig,
    ) -> GeneratedRelationship {
        self.count += 1;

        let id = self.uuid_factory.next().to_string();
        let properties = self.generate_properties(source, target, &rel_type.properties);

        let metadata = RelationshipMetadata {
            source: "datasynth".to_string(),
            generated_at: Utc::now(),
            weight: Some(rel_type.weight),
            valid_from: None,
            valid_to: None,
            labels: HashMap::new(),
            features: None,
            is_directed: rel_type.directed,
        };

        GeneratedRelationship {
            relationship_type: rel_type.name.clone(),
            id,
            source_id: source.id.clone(),
            target_id: target.id.clone(),
            properties,
            metadata,
        }
    }

    /// Generates properties for a relationship.
    fn generate_properties(
        &mut self,
        source: &NodeRef,
        target: &NodeRef,
        rules: &[super::rules::PropertyGenerationRule],
    ) -> HashMap<String, Value> {
        let mut properties = HashMap::new();

        for rule in rules {
            let value =
                self.generate_property_value(source, target, &rule.generator, &rule.value_type);
            properties.insert(rule.name.clone(), value);
        }

        properties
    }

    /// Generates a single property value.
    fn generate_property_value(
        &mut self,
        source: &NodeRef,
        target: &NodeRef,
        generator: &PropertyGenerator,
        value_type: &PropertyValueType,
    ) -> Value {
        match generator {
            PropertyGenerator::Constant(value) => value.clone(),

            PropertyGenerator::RandomChoice(choices) => {
                if choices.is_empty() {
                    Value::Null
                } else {
                    let idx = self.rng.gen_range(0..choices.len());
                    choices[idx].clone()
                }
            }

            PropertyGenerator::Range { min, max } => {
                let value = self.rng.gen_range(*min..=*max);
                match value_type {
                    PropertyValueType::Integer => {
                        Value::Number(serde_json::Number::from(value as i64))
                    }
                    _ => Value::Number(
                        serde_json::Number::from_f64(value)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                }
            }

            PropertyGenerator::FromSourceProperty(prop_name) => source
                .properties
                .get(prop_name)
                .cloned()
                .unwrap_or(Value::Null),

            PropertyGenerator::FromTargetProperty(prop_name) => target
                .properties
                .get(prop_name)
                .cloned()
                .unwrap_or(Value::Null),

            PropertyGenerator::Uuid => Value::String(self.uuid_factory.next().to_string()),

            PropertyGenerator::Timestamp => Value::String(Utc::now().to_rfc3339()),
        }
    }

    /// Tracks a relationship for cardinality validation.
    fn track_relationship(&mut self, source_id: &str, target_id: &str, rel_type: &str) {
        // Track by source
        self.relationships_by_source
            .entry(source_id.to_string())
            .or_default()
            .entry(rel_type.to_string())
            .or_default()
            .push(target_id.to_string());

        // Track by target
        self.relationships_by_target
            .entry(target_id.to_string())
            .or_default()
            .entry(rel_type.to_string())
            .or_default()
            .push(source_id.to_string());
    }

    /// DFS to check if there's a path from current to target.
    fn has_path_to(&mut self, current: &str, target: &str, depth: u32) -> bool {
        if depth >= self.config.max_circular_depth {
            return false;
        }

        if current == target {
            return true;
        }

        if self.visited.contains(current) {
            return false;
        }

        self.visited.insert(current.to_string());

        // Collect all next nodes to avoid holding borrow during recursion
        let next_nodes: Vec<String> = self
            .relationships_by_source
            .get(current)
            .map(|outgoing| outgoing.values().flatten().cloned().collect())
            .unwrap_or_default();

        // Now check paths without holding the borrow
        for next in next_nodes {
            if self.has_path_to(&next, target, depth + 1) {
                return true;
            }
        }

        false
    }
}

/// Builder for relationship configuration.
pub struct RelationshipConfigBuilder {
    config: RelationshipConfig,
}

impl RelationshipConfigBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            config: RelationshipConfig::default(),
        }
    }

    /// Adds a relationship type.
    pub fn add_type(mut self, type_config: RelationshipTypeConfig) -> Self {
        self.config.relationship_types.push(type_config);
        self
    }

    /// Sets whether orphans are allowed.
    pub fn allow_orphans(mut self, allow: bool) -> Self {
        self.config.allow_orphans = allow;
        self
    }

    /// Sets the orphan probability.
    pub fn orphan_probability(mut self, prob: f64) -> Self {
        self.config.orphan_probability = prob.clamp(0.0, 1.0);
        self
    }

    /// Sets whether circular relationships are allowed.
    pub fn allow_circular(mut self, allow: bool) -> Self {
        self.config.allow_circular = allow;
        self
    }

    /// Sets the maximum circular depth.
    pub fn max_circular_depth(mut self, depth: u32) -> Self {
        self.config.max_circular_depth = depth;
        self
    }

    /// Builds the configuration.
    pub fn build(self) -> RelationshipConfig {
        self.config
    }
}

impl Default for RelationshipConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn create_test_nodes() -> Vec<NodeRef> {
        vec![
            NodeRef::new("je_1", "journal_entry"),
            NodeRef::new("je_2", "journal_entry"),
            NodeRef::new("acc_1", "account"),
            NodeRef::new("acc_2", "account"),
            NodeRef::new("acc_3", "account"),
            NodeRef::new("user_1", "user"),
        ]
    }

    #[test]
    fn test_generate_relationships() {
        let config = RelationshipConfig::with_types(vec![RelationshipTypeConfig::new(
            "debits",
            "journal_entry",
            "account",
        )
        .with_cardinality(CardinalityRule::one_to_many(1, 2))]);

        let mut generator = RelationshipGenerator::new(config, 42);
        let nodes = create_test_nodes();
        let relationships = generator.generate_relationships(&nodes);

        assert!(!relationships.is_empty());
        for rel in &relationships {
            assert_eq!(rel.relationship_type, "debits");
            assert!(rel.source_id.starts_with("je_"));
            assert!(rel.target_id.starts_with("acc_"));
        }
    }

    #[test]
    fn test_cardinality_validation() {
        let config = RelationshipConfig::with_types(vec![RelationshipTypeConfig::new(
            "debits",
            "journal_entry",
            "account",
        )
        .with_cardinality(CardinalityRule::one_to_one())]);

        let generator = RelationshipGenerator::new(config, 42);

        let validation = generator.check_cardinality("je_1", "acc_1", "debits");
        assert!(validation.valid);

        let validation = generator.check_cardinality("je_1", "acc_1", "unknown");
        assert!(!validation.valid);
    }

    #[test]
    fn test_circular_detection() {
        let config = RelationshipConfig::default()
            .allow_circular(false)
            .max_circular_depth(3);

        let mut generator = RelationshipGenerator::new(config, 42);

        // Direct circular
        assert!(generator.check_circular("a", "a"));

        // No circular (different nodes)
        assert!(!generator.check_circular("a", "b"));
    }

    #[test]
    fn test_property_generation() {
        let config = RelationshipConfig::with_types(vec![RelationshipTypeConfig::new(
            "test", "source", "target",
        )
        .with_property(super::super::rules::PropertyGenerationRule::range(
            "amount", 100.0, 1000.0,
        ))
        .with_property(
            super::super::rules::PropertyGenerationRule::constant_string("status", "active"),
        )]);

        let mut generator = RelationshipGenerator::new(config, 42);
        let nodes = vec![NodeRef::new("s1", "source"), NodeRef::new("t1", "target")];

        let relationships = generator.generate_relationships(&nodes);

        assert!(!relationships.is_empty());
        let rel = &relationships[0];
        assert!(rel.properties.contains_key("amount"));
        assert!(rel.properties.contains_key("status"));
        assert_eq!(
            rel.properties.get("status"),
            Some(&Value::String("active".into()))
        );
    }

    #[test]
    fn test_orphan_generation() {
        let config = RelationshipConfig::with_types(vec![RelationshipTypeConfig::new(
            "test", "source", "target",
        )
        .with_cardinality(CardinalityRule::one_to_one())])
        .allow_orphans(true)
        .orphan_probability(1.0); // Always create orphans

        let mut generator = RelationshipGenerator::new(config, 42);

        let source = NodeRef::new("s1", "source");
        let available: HashMap<String, Vec<NodeRef>> =
            [("target".to_string(), vec![NodeRef::new("t1", "target")])]
                .into_iter()
                .collect();

        let relationships = generator.generate_for_node(&source, &available);
        assert!(relationships.is_empty());
    }

    #[test]
    fn test_config_builder() {
        let config = RelationshipConfigBuilder::new()
            .add_type(RelationshipTypeConfig::new("test", "a", "b"))
            .allow_orphans(false)
            .orphan_probability(0.1)
            .allow_circular(true)
            .max_circular_depth(5)
            .build();

        assert_eq!(config.relationship_types.len(), 1);
        assert!(!config.allow_orphans);
        assert_eq!(config.orphan_probability, 0.1);
        assert!(config.allow_circular);
        assert_eq!(config.max_circular_depth, 5);
    }

    #[test]
    fn test_generator_count_and_reset() {
        let config = RelationshipConfig::with_types(vec![RelationshipTypeConfig::new(
            "test", "source", "target",
        )
        .with_cardinality(CardinalityRule::one_to_one())]);

        let mut generator = RelationshipGenerator::new(config, 42);
        assert_eq!(generator.count(), 0);

        let nodes = vec![NodeRef::new("s1", "source"), NodeRef::new("t1", "target")];
        generator.generate_relationships(&nodes);

        assert!(generator.count() > 0);

        generator.reset(42);
        assert_eq!(generator.count(), 0);
    }
}
