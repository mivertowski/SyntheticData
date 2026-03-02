//! OCPM domain models for Object-Centric Process Mining.
//!
//! This module provides all OCPM data structures following the OCEL 2.0 standard:
//!
//! - Object types and instances with lifecycle states
//! - Activity types and event instances with lifecycle transitions
//! - Object relationships (many-to-many)
//! - Event-to-object relationships (many-to-many)
//! - Process variants and case traces
//! - Resources (users/systems performing activities)

mod activity_type;
mod correlation_event;
mod event;
mod event_log;
mod lifecycle_state_machine;
mod object_instance;
mod object_relationship;
mod object_type;
mod process_variant;
mod resource;
mod resource_pool;

pub use activity_type::*;
pub use correlation_event::*;
pub use event::*;
pub use event_log::*;
pub use lifecycle_state_machine::*;
pub use object_instance::*;
pub use object_relationship::*;
pub use object_type::*;
pub use process_variant::*;
pub use resource::*;
pub use resource_pool::*;
