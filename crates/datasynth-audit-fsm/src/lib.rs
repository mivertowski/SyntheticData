//! YAML-driven audit FSM engine.
//!
//! Loads audit methodology blueprints (ISA, IIA-GIAS) as finite state machines
//! and generates realistic audit artifacts with event trail output.
//!
//! # Quick Start
//!
//! ```no_run
//! use datasynth_audit_fsm::loader::{BlueprintWithPreconditions, load_overlay, OverlaySource, BuiltinOverlay};
//! use datasynth_audit_fsm::engine::AuditFsmEngine;
//! use datasynth_audit_fsm::context::EngagementContext;
//! use datasynth_audit_fsm::export::flat_log::export_events_to_json;
//! use rand::SeedableRng;
//! use rand_chacha::ChaCha8Rng;
//!
//! let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
//! bwp.validate().unwrap();
//! let overlay = load_overlay(&OverlaySource::Builtin(BuiltinOverlay::Default)).unwrap();
//! let rng = ChaCha8Rng::seed_from_u64(42);
//! let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
//! let ctx = EngagementContext::test_default();
//! let result = engine.run_engagement(&ctx).unwrap();
//! let json = export_events_to_json(&result.event_log).unwrap();
//! ```

pub mod analytics_inventory;
pub mod artifact;
pub mod benchmark;
pub mod content;
#[cfg(feature = "claude-content")]
pub mod content_claude;
pub mod context;
pub mod dispatch;
pub mod engine;
pub mod error;
pub mod event;
pub mod export;
pub mod loader;
pub mod schema;
