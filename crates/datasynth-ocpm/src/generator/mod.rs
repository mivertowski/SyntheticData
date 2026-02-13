//! OCPM event generators.
//!
//! This module provides generators for creating OCPM events from document flows
//! and business processes.

mod audit_generator;
mod bank_generator;
mod bank_recon_generator;
mod event_generator;
mod h2r_generator;
mod mfg_generator;
mod o2c_generator;
mod p2p_generator;
mod s2c_generator;

pub use audit_generator::*;
pub use bank_generator::*;
pub use bank_recon_generator::*;
pub use event_generator::*;
pub use h2r_generator::*;
pub use mfg_generator::*;
pub use o2c_generator::*;
pub use p2p_generator::*;
pub use s2c_generator::*;
