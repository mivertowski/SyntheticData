//! Diffusion model abstraction for statistical data generation.
//!
//! Implements a pure-Rust statistical diffusion process:
//! - Forward process: progressively adds noise
//! - Reverse process: denoises guided by target statistics
//! - Multiple noise schedules: linear, cosine, sigmoid

pub mod backend;
pub mod hybrid;
pub mod schedule;
pub mod statistical;
pub mod training;
pub mod utils;

pub use backend::*;
pub use hybrid::*;
pub use schedule::*;
pub use statistical::*;
pub use training::*;
pub use utils::*;
