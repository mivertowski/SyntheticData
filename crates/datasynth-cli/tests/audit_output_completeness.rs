/// Verify that output_writer.rs serializes all 15 AuditSnapshot fields to disk.
///
/// This test uses include_str! to read the source at compile time and checks
/// that every expected audit JSON filename is present.
#[test]
fn all_audit_snapshot_fields_are_written() {
    let source = include_str!("../src/output_writer.rs");

    let expected_files = [
        "audit_engagements.json",
        "audit_workpapers.json",
        "audit_evidence.json",
        "audit_risk_assessments.json",
        "audit_findings.json",
        "audit_judgments.json",
        "audit_confirmations.json",
        "audit_confirmation_responses.json",
        "audit_procedure_steps.json",
        "audit_samples.json",
        "audit_analytical_results.json",
        "audit_ia_functions.json",
        "audit_ia_reports.json",
        "audit_related_parties.json",
        "audit_related_party_transactions.json",
    ];

    let mut missing = Vec::new();
    for filename in &expected_files {
        if !source.contains(filename) {
            missing.push(*filename);
        }
    }

    assert!(
        missing.is_empty(),
        "The following audit output files are missing from output_writer.rs: {:?}",
        missing
    );
}
