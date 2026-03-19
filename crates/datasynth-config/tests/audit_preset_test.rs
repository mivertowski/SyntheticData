//! Integration tests for the audit_group_overlay preset.

#![allow(clippy::unwrap_used)]

use datasynth_config::presets::{audit_group_overlay, create_preset};
use datasynth_config::schema::TransactionVolume;
use datasynth_core::models::{CoAComplexity, IndustrySector};

/// Convenience: build a base manufacturing config and apply the overlay.
fn overlay_config() -> datasynth_config::GeneratorConfig {
    let base = create_preset(
        IndustrySector::Manufacturing,
        3,
        12,
        CoAComplexity::Medium,
        TransactionVolume::TenK,
    );
    audit_group_overlay(base)
}

// ─── Audit standards ─────────────────────────────────────────────────────────

#[test]
fn test_audit_standards_enabled() {
    let config = overlay_config();
    assert!(
        config.audit_standards.enabled,
        "audit_standards must be enabled"
    );
}

#[test]
fn test_isa_compliance_enabled() {
    let config = overlay_config();
    assert!(
        config.audit_standards.isa_compliance.enabled,
        "ISA compliance must be enabled"
    );
}

#[test]
fn test_isa_framework_dual() {
    let config = overlay_config();
    assert_eq!(
        config.audit_standards.isa_compliance.framework, "dual",
        "Framework must be 'dual' (ISA + PCAOB)"
    );
}

#[test]
fn test_audit_trail_enabled() {
    let config = overlay_config();
    assert!(
        config.audit_standards.generate_audit_trail,
        "Audit trail generation must be enabled"
    );
}

#[test]
fn test_sox_enabled() {
    let config = overlay_config();
    assert!(config.audit_standards.sox.enabled, "SOX must be enabled");
}

#[test]
fn test_pcaob_enabled() {
    let config = overlay_config();
    assert!(
        config.audit_standards.pcaob.enabled,
        "PCAOB must be enabled"
    );
}

// ─── Accounting standards ─────────────────────────────────────────────────────

#[test]
fn test_accounting_standards_enabled() {
    let config = overlay_config();
    assert!(
        config.accounting_standards.enabled,
        "accounting_standards must be enabled"
    );
}

#[test]
fn test_revenue_recognition_enabled() {
    let config = overlay_config();
    assert!(
        config.accounting_standards.revenue_recognition.enabled,
        "Revenue recognition must be enabled"
    );
}

#[test]
fn test_leases_enabled() {
    let config = overlay_config();
    assert!(
        config.accounting_standards.leases.enabled,
        "Leases must be enabled"
    );
}

#[test]
fn test_fair_value_enabled() {
    let config = overlay_config();
    assert!(
        config.accounting_standards.fair_value.enabled,
        "Fair value must be enabled"
    );
}

#[test]
fn test_impairment_enabled() {
    let config = overlay_config();
    assert!(
        config.accounting_standards.impairment.enabled,
        "Impairment must be enabled"
    );
}

// ─── Internal controls ────────────────────────────────────────────────────────

#[test]
fn test_internal_controls_enabled() {
    let config = overlay_config();
    assert!(
        config.internal_controls.enabled,
        "Internal controls must be enabled"
    );
}

#[test]
fn test_coso_enabled() {
    let config = overlay_config();
    assert!(
        config.internal_controls.coso_enabled,
        "COSO must be enabled"
    );
}

#[test]
fn test_entity_level_controls_enabled() {
    let config = overlay_config();
    assert!(
        config.internal_controls.include_entity_level_controls,
        "Entity-level controls must be enabled"
    );
}

#[test]
fn test_maturity_level_managed() {
    let config = overlay_config();
    assert_eq!(
        config.internal_controls.target_maturity_level, "managed",
        "Maturity level should be 'managed'"
    );
}

#[test]
fn test_exception_rate() {
    let config = overlay_config();
    assert!(
        (config.internal_controls.exception_rate - 0.02).abs() < 1e-9,
        "Exception rate should be 0.02"
    );
}

// ─── Anomaly injection ────────────────────────────────────────────────────────

#[test]
fn test_anomaly_injection_enabled() {
    let config = overlay_config();
    assert!(
        config.anomaly_injection.enabled,
        "Anomaly injection must be enabled"
    );
}

#[test]
fn test_anomaly_total_rate() {
    let config = overlay_config();
    assert!(
        (config.anomaly_injection.rates.total_rate - 0.02).abs() < 1e-9,
        "Total anomaly rate should be 0.02"
    );
}

#[test]
fn test_anomaly_fraud_rate() {
    let config = overlay_config();
    assert!(
        (config.anomaly_injection.rates.fraud_rate - 0.01).abs() < 1e-9,
        "Fraud rate should be 0.01"
    );
}

// ─── Network features ─────────────────────────────────────────────────────────

#[test]
fn test_vendor_network_enabled() {
    let config = overlay_config();
    assert!(
        config.vendor_network.enabled,
        "Vendor network must be enabled"
    );
}

#[test]
fn test_customer_segmentation_enabled() {
    let config = overlay_config();
    assert!(
        config.customer_segmentation.enabled,
        "Customer segmentation must be enabled"
    );
}

#[test]
fn test_cross_process_links_enabled() {
    let config = overlay_config();
    assert!(
        config.cross_process_links.enabled,
        "Cross-process links must be enabled"
    );
}

#[test]
fn test_relationship_strength_enabled() {
    let config = overlay_config();
    assert!(
        config.relationship_strength.enabled,
        "Relationship strength must be enabled"
    );
}

// ─── Intercompany ─────────────────────────────────────────────────────────────

#[test]
fn test_intercompany_enabled() {
    let config = overlay_config();
    assert!(config.intercompany.enabled, "Intercompany must be enabled");
}

#[test]
fn test_intercompany_matched_pairs() {
    let config = overlay_config();
    assert!(
        config.intercompany.generate_matched_pairs,
        "Intercompany matched pairs must be enabled"
    );
}

#[test]
fn test_intercompany_eliminations() {
    let config = overlay_config();
    assert!(
        config.intercompany.generate_eliminations,
        "Intercompany eliminations must be enabled"
    );
}

// ─── Balance generation ───────────────────────────────────────────────────────

#[test]
fn test_opening_balances_enabled() {
    let config = overlay_config();
    assert!(
        config.balance.generate_opening_balances,
        "Opening balances must be enabled"
    );
}

#[test]
fn test_trial_balances_enabled() {
    let config = overlay_config();
    assert!(
        config.balance.generate_trial_balances,
        "Trial balances must be enabled"
    );
}

#[test]
fn test_subledger_reconciliation_enabled() {
    let config = overlay_config();
    assert!(
        config.balance.reconcile_subledgers,
        "Subledger reconciliation must be enabled"
    );
}

// ─── Scenario tags ────────────────────────────────────────────────────────────

#[test]
fn test_scenario_tags_contain_audit_group() {
    let config = overlay_config();
    assert!(
        config.scenario.tags.contains(&"audit_group".to_string()),
        "Tags should contain 'audit_group'"
    );
}

#[test]
fn test_scenario_tags_contain_audit_simulation() {
    let config = overlay_config();
    assert!(
        config
            .scenario
            .tags
            .contains(&"audit_simulation".to_string()),
        "Tags should contain 'audit_simulation'"
    );
}

// ─── Idempotency / overlay on different industries ────────────────────────────

#[test]
fn test_overlay_on_retail() {
    let base = create_preset(
        IndustrySector::Retail,
        2,
        12,
        CoAComplexity::Small,
        TransactionVolume::TenK,
    );
    let config = audit_group_overlay(base);
    assert!(config.audit_standards.enabled);
    assert!(config.internal_controls.enabled);
    assert!(config.accounting_standards.enabled);
    // Company structure should be preserved
    assert_eq!(config.companies.len(), 2);
    assert_eq!(config.global.industry, IndustrySector::Retail);
}

#[test]
fn test_overlay_preserves_company_count() {
    let base = create_preset(
        IndustrySector::Technology,
        3,
        6,
        CoAComplexity::Medium,
        TransactionVolume::TenK,
    );
    let original_count = base.companies.len();
    let config = audit_group_overlay(base);
    assert_eq!(config.companies.len(), original_count);
}

#[test]
fn test_overlay_preserves_period_months() {
    let base = create_preset(
        IndustrySector::Manufacturing,
        1,
        24,
        CoAComplexity::Small,
        TransactionVolume::TenK,
    );
    let config = audit_group_overlay(base);
    assert_eq!(config.global.period_months, 24);
}

#[test]
fn test_double_overlay_idempotent() {
    // Applying the overlay twice should not change anything.
    let base = create_preset(
        IndustrySector::Manufacturing,
        2,
        12,
        CoAComplexity::Medium,
        TransactionVolume::TenK,
    );
    let once = audit_group_overlay(base.clone());
    let twice = audit_group_overlay(once.clone());
    // Key fields remain the same
    assert_eq!(once.audit_standards.enabled, twice.audit_standards.enabled);
    assert_eq!(
        once.internal_controls.enabled,
        twice.internal_controls.enabled
    );
    assert_eq!(
        once.accounting_standards.enabled,
        twice.accounting_standards.enabled
    );
    assert_eq!(
        once.internal_controls.target_maturity_level,
        twice.internal_controls.target_maturity_level
    );
}
