//! Edge synthesizers for creating relationships between nodes.
//!
//! Each synthesizer produces edges for a specific domain:
//! - [`document_chain::DocumentChainEdgeSynthesizer`] — P2P and O2C document chain edges.
//! - [`risk_control::RiskControlEdgeSynthesizer`] — Risk-control mapping, control ownership,
//!   findings, workpaper testing, and account coverage edges.

pub mod document_chain;
pub mod risk_control;

use crate::traits::EdgeSynthesizer;

/// Return all built-in edge synthesizers in dependency order.
///
/// Document chain edges are produced first (no dependencies on other edges),
/// then risk-control edges (which also have no inter-edge dependencies but
/// are logically downstream in the audit domain).
pub fn all_synthesizers() -> Vec<Box<dyn EdgeSynthesizer>> {
    vec![
        Box::new(document_chain::DocumentChainEdgeSynthesizer),
        Box::new(risk_control::RiskControlEdgeSynthesizer),
    ]
}
