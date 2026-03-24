//! YAML-driven audit FSM engine.
//!
//! Loads audit methodology blueprints (ISA, IIA-GIAS) as finite state machines
//! and generates realistic audit artifacts with event trail output.

pub mod context;
pub mod engine;
pub mod error;
pub mod event;
pub mod loader;
pub mod schema;
