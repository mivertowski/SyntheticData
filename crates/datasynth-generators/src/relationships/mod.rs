//! Relationship generation module.
//!
//! This module provides generators for creating relationships between entities,
//! supporting configurable cardinality rules and property generation.
//!
//! # Features
//!
//! - **Cardinality Rules**: OneToOne, OneToMany, ManyToOne, ManyToMany
//! - **Property Generation**: Generate relationship properties from rules
//! - **Orphan Control**: Allow/prevent orphan entities
//! - **Circular Detection**: Detect and optionally prevent circular relationships
//! - **Entity Graph Generation**: Comprehensive entity relationship graphs
//! - **Cross-Process Links**: P2P ↔ O2C via inventory connections
//! - **Relationship Strength**: Calculated from transaction history
//!
//! # Example
//!
//! ```ignore
//! use datasynth_generators::relationships::{RelationshipGenerator, RelationshipConfig};
//!
//! let config = RelationshipConfig::default();
//! let mut generator = RelationshipGenerator::new(config, 42);
//!
//! // Generate relationships between nodes
//! let edges = generator.generate_relationships(&nodes);
//! ```

mod entity_graph_generator;
mod generator;
mod rules;

pub use entity_graph_generator::*;
pub use generator::*;
pub use rules::*;
