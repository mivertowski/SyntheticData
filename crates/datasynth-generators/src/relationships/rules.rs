//! Relationship rules and configuration.
//!
//! Provides cardinality rules, property generation rules, and relationship
//! type configurations for the relationship generator.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Configuration for relationship generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipConfig {
    /// Relationship type definitions.
    pub relationship_types: Vec<RelationshipTypeConfig>,
    /// Allow orphan entities (entities with no relationships).
    pub allow_orphans: bool,
    /// Probability of creating an orphan entity.
    pub orphan_probability: f64,
    /// Allow circular relationships.
    pub allow_circular: bool,
    /// Maximum depth for circular relationship detection.
    pub max_circular_depth: u32,
}

impl Default for RelationshipConfig {
    fn default() -> Self {
        Self {
            relationship_types: Vec::new(),
            allow_orphans: true,
            orphan_probability: 0.01,
            allow_circular: false,
            max_circular_depth: 3,
        }
    }
}

impl RelationshipConfig {
    /// Creates a new configuration with the given relationship types.
    pub fn with_types(types: Vec<RelationshipTypeConfig>) -> Self {
        Self {
            relationship_types: types,
            ..Default::default()
        }
    }

    /// Sets whether orphan entities are allowed.
    pub fn allow_orphans(mut self, allow: bool) -> Self {
        self.allow_orphans = allow;
        self
    }

    /// Sets the orphan probability.
    pub fn orphan_probability(mut self, prob: f64) -> Self {
        self.orphan_probability = prob.clamp(0.0, 1.0);
        self
    }

    /// Sets whether circular relationships are allowed.
    pub fn allow_circular(mut self, allow: bool) -> Self {
        self.allow_circular = allow;
        self
    }

    /// Sets the maximum circular depth.
    pub fn max_circular_depth(mut self, depth: u32) -> Self {
        self.max_circular_depth = depth;
        self
    }
}

/// Configuration for a specific relationship type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipTypeConfig {
    /// Name of the relationship type (e.g., "debits", "credits", "created").
    pub name: String,
    /// Source entity type (e.g., "journal_entry").
    pub source_type: String,
    /// Target entity type (e.g., "account").
    pub target_type: String,
    /// Cardinality rule for this relationship.
    pub cardinality: CardinalityRule,
    /// Weight for this relationship in random selection.
    pub weight: f64,
    /// Property generation rules for this relationship.
    pub properties: Vec<PropertyGenerationRule>,
    /// Whether this relationship is required.
    pub required: bool,
    /// Whether this relationship is directed.
    pub directed: bool,
}

impl Default for RelationshipTypeConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            source_type: String::new(),
            target_type: String::new(),
            cardinality: CardinalityRule::OneToMany { min: 1, max: 5 },
            weight: 1.0,
            properties: Vec::new(),
            required: false,
            directed: true,
        }
    }
}

impl RelationshipTypeConfig {
    /// Creates a new relationship type configuration.
    pub fn new(
        name: impl Into<String>,
        source_type: impl Into<String>,
        target_type: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            source_type: source_type.into(),
            target_type: target_type.into(),
            ..Default::default()
        }
    }

    /// Sets the cardinality rule.
    pub fn with_cardinality(mut self, cardinality: CardinalityRule) -> Self {
        self.cardinality = cardinality;
        self
    }

    /// Sets the weight.
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight.max(0.0);
        self
    }

    /// Adds a property generation rule.
    pub fn with_property(mut self, property: PropertyGenerationRule) -> Self {
        self.properties.push(property);
        self
    }

    /// Sets whether this relationship is required.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Sets whether this relationship is directed.
    pub fn directed(mut self, directed: bool) -> Self {
        self.directed = directed;
        self
    }
}

/// Cardinality rule for relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardinalityRule {
    /// One source to one target.
    OneToOne,
    /// One source to many targets.
    OneToMany {
        /// Minimum number of targets.
        min: u32,
        /// Maximum number of targets.
        max: u32,
    },
    /// Many sources to one target.
    ManyToOne {
        /// Minimum number of sources.
        min: u32,
        /// Maximum number of sources.
        max: u32,
    },
    /// Many sources to many targets.
    ManyToMany {
        /// Minimum targets per source.
        min_per_source: u32,
        /// Maximum targets per source.
        max_per_source: u32,
    },
}

impl Default for CardinalityRule {
    fn default() -> Self {
        Self::OneToMany { min: 1, max: 5 }
    }
}

impl CardinalityRule {
    /// Creates a OneToOne cardinality.
    pub fn one_to_one() -> Self {
        Self::OneToOne
    }

    /// Creates a OneToMany cardinality.
    pub fn one_to_many(min: u32, max: u32) -> Self {
        Self::OneToMany {
            min,
            max: max.max(min),
        }
    }

    /// Creates a ManyToOne cardinality.
    pub fn many_to_one(min: u32, max: u32) -> Self {
        Self::ManyToOne {
            min,
            max: max.max(min),
        }
    }

    /// Creates a ManyToMany cardinality.
    pub fn many_to_many(min_per_source: u32, max_per_source: u32) -> Self {
        Self::ManyToMany {
            min_per_source,
            max_per_source: max_per_source.max(min_per_source),
        }
    }

    /// Returns the minimum and maximum counts for this cardinality.
    pub fn bounds(&self) -> (u32, u32) {
        match self {
            Self::OneToOne => (1, 1),
            Self::OneToMany { min, max } => (*min, *max),
            Self::ManyToOne { min, max } => (*min, *max),
            Self::ManyToMany {
                min_per_source,
                max_per_source,
            } => (*min_per_source, *max_per_source),
        }
    }

    /// Checks if this cardinality allows multiple targets.
    pub fn is_multi_target(&self) -> bool {
        matches!(self, Self::OneToMany { .. } | Self::ManyToMany { .. })
    }

    /// Checks if this cardinality allows multiple sources.
    pub fn is_multi_source(&self) -> bool {
        matches!(self, Self::ManyToOne { .. } | Self::ManyToMany { .. })
    }
}

/// Property generation rule for relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyGenerationRule {
    /// Property name.
    pub name: String,
    /// Property value type.
    pub value_type: PropertyValueType,
    /// Property generator.
    pub generator: PropertyGenerator,
}

impl PropertyGenerationRule {
    /// Creates a new property generation rule.
    pub fn new(
        name: impl Into<String>,
        value_type: PropertyValueType,
        generator: PropertyGenerator,
    ) -> Self {
        Self {
            name: name.into(),
            value_type,
            generator,
        }
    }

    /// Creates a constant string property.
    pub fn constant_string(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(
            name,
            PropertyValueType::String,
            PropertyGenerator::Constant(Value::String(value.into())),
        )
    }

    /// Creates a constant numeric property.
    pub fn constant_number(name: impl Into<String>, value: f64) -> Self {
        Self::new(
            name,
            PropertyValueType::Float,
            PropertyGenerator::Constant(Value::Number(
                serde_json::Number::from_f64(value).unwrap_or_else(|| serde_json::Number::from(0)),
            )),
        )
    }

    /// Creates a range property.
    pub fn range(name: impl Into<String>, min: f64, max: f64) -> Self {
        Self::new(
            name,
            PropertyValueType::Float,
            PropertyGenerator::Range { min, max },
        )
    }

    /// Creates a random choice property.
    pub fn random_choice(name: impl Into<String>, choices: Vec<Value>) -> Self {
        Self::new(
            name,
            PropertyValueType::String,
            PropertyGenerator::RandomChoice(choices),
        )
    }
}

/// Property value type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyValueType {
    /// String value.
    String,
    /// Integer value.
    Integer,
    /// Float value.
    Float,
    /// Boolean value.
    Boolean,
    /// Date/time value.
    DateTime,
}

/// Property generator for relationship properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyGenerator {
    /// Constant value.
    Constant(Value),
    /// Random choice from a list.
    RandomChoice(Vec<Value>),
    /// Range of numeric values.
    Range {
        /// Minimum value.
        min: f64,
        /// Maximum value.
        max: f64,
    },
    /// Copy from source node property.
    FromSourceProperty(String),
    /// Copy from target node property.
    FromTargetProperty(String),
    /// UUID generator.
    Uuid,
    /// Timestamp generator.
    Timestamp,
}

impl Default for PropertyGenerator {
    fn default() -> Self {
        Self::Constant(Value::Null)
    }
}

/// Relationship validation result.
#[derive(Debug, Clone)]
pub struct RelationshipValidation {
    /// Whether the relationship is valid.
    pub valid: bool,
    /// Validation errors.
    pub errors: Vec<String>,
    /// Validation warnings.
    pub warnings: Vec<String>,
}

impl RelationshipValidation {
    /// Creates a valid result.
    pub fn valid() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Creates an invalid result with the given error.
    pub fn invalid(error: impl Into<String>) -> Self {
        Self {
            valid: false,
            errors: vec![error.into()],
            warnings: Vec::new(),
        }
    }

    /// Adds an error.
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.valid = false;
        self.errors.push(error.into());
        self
    }

    /// Adds a warning.
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }
}

/// Common relationship type definitions for accounting domain.
pub mod accounting {
    use super::*;

    /// Creates a "debits" relationship configuration.
    pub fn debits_relationship() -> RelationshipTypeConfig {
        RelationshipTypeConfig::new("debits", "journal_entry", "account")
            .with_cardinality(CardinalityRule::one_to_many(1, 5))
            .required(true)
            .with_property(PropertyGenerationRule::range("amount", 0.01, 1_000_000.0))
    }

    /// Creates a "credits" relationship configuration.
    pub fn credits_relationship() -> RelationshipTypeConfig {
        RelationshipTypeConfig::new("credits", "journal_entry", "account")
            .with_cardinality(CardinalityRule::one_to_many(1, 5))
            .required(true)
            .with_property(PropertyGenerationRule::range("amount", 0.01, 1_000_000.0))
    }

    /// Creates a "created_by" relationship configuration.
    pub fn created_by_relationship() -> RelationshipTypeConfig {
        RelationshipTypeConfig::new("created_by", "journal_entry", "user")
            .with_cardinality(CardinalityRule::ManyToOne { min: 1, max: 1 })
            .required(true)
    }

    /// Creates an "approved_by" relationship configuration.
    pub fn approved_by_relationship() -> RelationshipTypeConfig {
        RelationshipTypeConfig::new("approved_by", "journal_entry", "user")
            .with_cardinality(CardinalityRule::ManyToOne { min: 0, max: 1 })
    }

    /// Creates a "belongs_to" relationship configuration for vendor to company.
    pub fn vendor_belongs_to_company() -> RelationshipTypeConfig {
        RelationshipTypeConfig::new("belongs_to", "vendor", "company")
            .with_cardinality(CardinalityRule::ManyToOne { min: 1, max: 1 })
            .required(true)
    }

    /// Creates a "references" relationship configuration for document chains.
    pub fn document_references() -> RelationshipTypeConfig {
        RelationshipTypeConfig::new("references", "document", "document")
            .with_cardinality(CardinalityRule::ManyToMany {
                min_per_source: 0,
                max_per_source: 5,
            })
            .with_property(PropertyGenerationRule::random_choice(
                "reference_type",
                vec![
                    Value::String("follow_on".into()),
                    Value::String("reversal".into()),
                    Value::String("payment".into()),
                ],
            ))
    }

    /// Creates a default accounting relationship configuration.
    pub fn default_accounting_config() -> RelationshipConfig {
        RelationshipConfig::with_types(vec![
            debits_relationship(),
            credits_relationship(),
            created_by_relationship(),
            approved_by_relationship(),
        ])
        .allow_orphans(true)
        .orphan_probability(0.01)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_cardinality_bounds() {
        let one_to_one = CardinalityRule::one_to_one();
        assert_eq!(one_to_one.bounds(), (1, 1));

        let one_to_many = CardinalityRule::one_to_many(2, 5);
        assert_eq!(one_to_many.bounds(), (2, 5));

        let many_to_one = CardinalityRule::many_to_one(1, 3);
        assert_eq!(many_to_one.bounds(), (1, 3));

        let many_to_many = CardinalityRule::many_to_many(1, 10);
        assert_eq!(many_to_many.bounds(), (1, 10));
    }

    #[test]
    fn test_cardinality_multi() {
        assert!(!CardinalityRule::one_to_one().is_multi_target());
        assert!(!CardinalityRule::one_to_one().is_multi_source());

        assert!(CardinalityRule::one_to_many(1, 5).is_multi_target());
        assert!(!CardinalityRule::one_to_many(1, 5).is_multi_source());

        assert!(!CardinalityRule::many_to_one(1, 5).is_multi_target());
        assert!(CardinalityRule::many_to_one(1, 5).is_multi_source());

        assert!(CardinalityRule::many_to_many(1, 5).is_multi_target());
        assert!(CardinalityRule::many_to_many(1, 5).is_multi_source());
    }

    #[test]
    fn test_relationship_type_config() {
        let config = RelationshipTypeConfig::new("debits", "journal_entry", "account")
            .with_cardinality(CardinalityRule::one_to_many(1, 5))
            .with_weight(2.0)
            .required(true)
            .directed(true);

        assert_eq!(config.name, "debits");
        assert_eq!(config.source_type, "journal_entry");
        assert_eq!(config.target_type, "account");
        assert_eq!(config.weight, 2.0);
        assert!(config.required);
        assert!(config.directed);
    }

    #[test]
    fn test_property_generation_rule() {
        let constant = PropertyGenerationRule::constant_string("status", "active");
        assert_eq!(constant.name, "status");

        let range = PropertyGenerationRule::range("amount", 0.0, 1000.0);
        assert_eq!(range.name, "amount");

        let choice = PropertyGenerationRule::random_choice(
            "type",
            vec![Value::String("A".into()), Value::String("B".into())],
        );
        assert_eq!(choice.name, "type");
    }

    #[test]
    fn test_relationship_config() {
        let config = RelationshipConfig::default()
            .allow_orphans(false)
            .orphan_probability(0.05)
            .allow_circular(true)
            .max_circular_depth(5);

        assert!(!config.allow_orphans);
        assert_eq!(config.orphan_probability, 0.05);
        assert!(config.allow_circular);
        assert_eq!(config.max_circular_depth, 5);
    }

    #[test]
    fn test_accounting_relationships() {
        let config = accounting::default_accounting_config();
        assert_eq!(config.relationship_types.len(), 4);

        let debits = config
            .relationship_types
            .iter()
            .find(|t| t.name == "debits")
            .unwrap();
        assert!(debits.required);
        assert_eq!(debits.source_type, "journal_entry");
        assert_eq!(debits.target_type, "account");
    }

    #[test]
    fn test_validation() {
        let valid = RelationshipValidation::valid();
        assert!(valid.valid);
        assert!(valid.errors.is_empty());

        let invalid = RelationshipValidation::invalid("Missing source")
            .with_warning("Consider adding target");
        assert!(!invalid.valid);
        assert_eq!(invalid.errors.len(), 1);
        assert_eq!(invalid.warnings.len(), 1);
    }
}
