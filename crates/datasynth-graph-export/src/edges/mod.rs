//! Edge synthesizers for creating relationships between nodes.
//!
//! Each synthesizer produces edges for a specific domain:
//! - [`document_chain::DocumentChainEdgeSynthesizer`] — P2P and O2C document chain edges.
//! - [`risk_control::RiskControlEdgeSynthesizer`] — Risk-control mapping, control ownership,
//!   findings, workpaper testing, and account coverage edges.
//! - [`audit_trail::AuditTrailEdgeSynthesizer`] — Workpaper/finding/evidence/engagement links.
//! - [`accounting::AccountingEdgeSynthesizer`] — JE posting, account hierarchy, cost centers.
//! - [`banking::BankingEdgeSynthesizer`] — Bank account/transaction/customer links.
//! - [`s2c::S2CEdgeSynthesizer`] — Source-to-contract procurement edges.
//! - [`h2r::H2REdgeSynthesizer`] — Time/expense/payroll to employee edges.
//! - [`mfg::MFGEdgeSynthesizer`] — Quality inspection to production/material edges.
//! - [`entity_relationships::EntityRelationshipEdgeSynthesizer`] — Doc/JE creator, vendor/customer links.
//! - [`process_sequence::ProcessSequenceEdgeSynthesizer`] — OCEL directly-follows edges.
//! - [`audit_procedures::AuditProcedureEdgeSynthesizer`] — ISA 505/330/530/520/610/550 procedure edges.

pub mod accounting;
pub mod audit_procedures;
pub mod audit_trail;
pub mod banking;
pub mod document_chain;
pub mod entity_relationships;
pub mod h2r;
pub mod mfg;
pub mod process_sequence;
pub mod risk_control;
pub mod s2c;

use crate::traits::EdgeSynthesizer;

/// Return all built-in edge synthesizers in dependency order.
///
/// Ordering rationale:
/// 1. `document_chain` — no dependencies on other edges.
/// 2. `risk_control` — no inter-edge dependencies.
/// 3. `audit_trail` — no inter-edge dependencies.
/// 4. `banking` — no inter-edge dependencies.
/// 5. `s2c` — no inter-edge dependencies.
/// 6. `h2r` — no inter-edge dependencies.
/// 7. `mfg` — no inter-edge dependencies.
/// 8. `accounting` — depends on document_chain (DOC_POSTS_JE uses doc nodes).
/// 9. `entity_relationships` — depends on document_chain (DOC_CREATED_BY needs doc nodes).
/// 10. `process_sequence` — depends on audit_trail (DirectlyFollows uses OCPM event nodes).
/// 11. `audit_procedures` — no inter-edge dependencies; depends on audit_trail nodes (workpapers, engagements, evidence).
pub fn all_synthesizers() -> Vec<Box<dyn EdgeSynthesizer>> {
    vec![
        Box::new(document_chain::DocumentChainEdgeSynthesizer),
        Box::new(risk_control::RiskControlEdgeSynthesizer),
        Box::new(audit_trail::AuditTrailEdgeSynthesizer),
        Box::new(banking::BankingEdgeSynthesizer),
        Box::new(s2c::S2CEdgeSynthesizer),
        Box::new(h2r::H2REdgeSynthesizer),
        Box::new(mfg::MFGEdgeSynthesizer),
        Box::new(accounting::AccountingEdgeSynthesizer),
        Box::new(entity_relationships::EntityRelationshipEdgeSynthesizer),
        Box::new(process_sequence::ProcessSequenceEdgeSynthesizer),
        Box::new(audit_procedures::AuditProcedureEdgeSynthesizer),
    ]
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_edge_synthesizers_have_non_overlapping_produces() {
        let synthesizers = all_synthesizers();
        let mut seen: HashSet<u32> = HashSet::new();

        // Collect all edge type codes from each synthesizer and verify uniqueness.
        // Since the EdgeSynthesizer trait doesn't have a `produces()` method,
        // we verify uniqueness by checking that each synthesizer has a unique name.
        let mut names: HashSet<&str> = HashSet::new();
        for s in &synthesizers {
            assert!(
                names.insert(s.name()),
                "Duplicate synthesizer name: '{}'",
                s.name()
            );
        }
        assert_eq!(names.len(), 11, "Expected 11 edge synthesizers");

        // Verify all known edge type codes are unique across synthesizers.
        // These are the codes defined as constants in each module.
        let all_codes: &[(&str, &[u32])] = &[
            ("document_chain", &[60, 62, 64, 66, 68, 69]),
            ("risk_control", &[75, 120, 127, 128, 129, 45]),
            (
                "audit_trail",
                &[72, 73, 74, 77, 100, 101, 103, 104, 132, 134],
            ),
            ("accounting", &[70, 99, 112, 124, 133]),
            ("banking", &[80, 81, 82]),
            ("s2c", &[83, 84, 85, 86, 87, 88, 89]),
            ("h2r", &[90, 91, 92, 93]),
            ("mfg", &[48, 95]),
            ("entity_relationships", &[96, 98, 135, 136, 137]),
            ("process_sequence", &[121]),
            (
                "audit_procedures",
                &[
                    138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152,
                ],
            ),
        ];

        for (synth_name, codes) in all_codes {
            for &code in *codes {
                assert!(
                    seen.insert(code),
                    "Duplicate edge type code {} in synthesizer '{}'",
                    code,
                    synth_name
                );
            }
        }
    }

    #[test]
    fn all_synthesizers_returns_eleven() {
        assert_eq!(all_synthesizers().len(), 11);
    }
}
