//! Collusion and conspiracy network modeling.
//!
//! This module models fraud networks with multiple conspirators and coordinated schemes:
//! - Collusion rings with multiple member types
//! - Trust and loyalty dynamics
//! - Coordinated transaction generation
//! - Defection and detection risk modeling

mod network;

pub use network::{
    CollusionRing, CollusionRingConfig, CollusionRingType, Conspirator, ConspiratorRole,
    RingBehavior, RingStatus,
};
