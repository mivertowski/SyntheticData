#![deny(clippy::unwrap_used)]
//! Object-Centric Process Mining (OCPM) module for synthetic data generation.
//!
//! This crate provides OCPM data structures compatible with the OCEL 2.0 standard,
//! enabling event-to-object many-to-many relationships for process mining analysis.
//!
//! # Key Concepts
//!
//! - **Objects**: Business entities that evolve through processes (Orders, Invoices, etc.)
//! - **Events**: Activities that occur on objects (Create, Approve, Post, etc.)
//! - **Relationships**: Many-to-many links between objects (Order contains OrderLines)
//! - **Variants**: Distinct execution patterns through processes
//!
//! # Modules
//!
//! - `models`: Core OCPM data structures (OCEL 2.0 compatible)
//! - `generator`: Event generator from document flows
//! - `export`: OCEL 2.0 JSON export functionality
//!
//! # Example
//!
//! ```ignore
//! use datasynth_ocpm::{OcpmEventGenerator, OcpmEventLog, Ocel2Exporter};
//!
//! // Create event generator
//! let mut generator = OcpmEventGenerator::new(42);
//!
//! // Generate events from P2P document flow
//! let events = generator.generate_p2p_events(&purchase_order, &goods_receipt, &invoice, &payment);
//!
//! // Export to OCEL 2.0 format
//! let exporter = Ocel2Exporter::new();
//! exporter.export_to_file(&event_log, "output/ocel2.json")?;
//! ```

pub mod export;
pub mod generator;
pub mod models;

pub use export::*;
pub use generator::*;
pub use models::*;
