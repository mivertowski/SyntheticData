//! Integration tests for the period-close phase.
//!
//! Verifies that tax provision and income statement closing journal entries
//! are generated correctly by the orchestrator.

use datasynth_core::accounts::{equity_accounts, tax_accounts};
use datasynth_runtime::{EnhancedOrchestrator, PhaseConfig};
use datasynth_test_utils::fixtures::minimal_config;
use rust_decimal::Decimal;

/// Helper: extract period-close JEs (document_type == "CL") from the result.
fn close_entries(
    entries: &[datasynth_core::models::JournalEntry],
) -> Vec<&datasynth_core::models::JournalEntry> {
    entries
        .iter()
        .filter(|e| e.header.document_type == "CL")
        .collect()
}

/// Test that period-close entries are generated when there is income statement activity.
#[test]
fn test_period_close_generates_entries() {
    let mut config = minimal_config();
    config.global.seed = Some(200);
    config.global.period_months = 3;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        generate_period_close: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // There should be journal entries generated
    assert!(
        !result.journal_entries.is_empty(),
        "Should generate journal entries"
    );

    // Expect period-close JEs (CL document type)
    let close_jes = close_entries(&result.journal_entries);
    assert!(
        !close_jes.is_empty(),
        "Should generate at least one period-close journal entry"
    );

    // Stats should reflect the period-close count
    assert!(
        result.statistics.period_close_je_count > 0,
        "period_close_je_count should be > 0, got {}",
        result.statistics.period_close_je_count
    );
}

/// Test that all period-close JEs are balanced.
#[test]
fn test_period_close_entries_are_balanced() {
    let mut config = minimal_config();
    config.global.seed = Some(201);
    config.global.period_months = 3;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        generate_period_close: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    let close_jes = close_entries(&result.journal_entries);
    for je in &close_jes {
        assert!(
            je.is_balanced(),
            "Period-close JE {} is not balanced: debits={}, credits={}",
            je.header.document_id,
            je.total_debit(),
            je.total_credit()
        );
    }
}

/// Test that tax provision uses the correct GL accounts.
#[test]
fn test_tax_provision_accounts() {
    let mut config = minimal_config();
    config.global.seed = Some(202);
    config.global.period_months = 3;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        generate_period_close: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    let close_jes = close_entries(&result.journal_entries);

    // Find tax provision JE (header_text contains "Tax provision")
    let tax_jes: Vec<_> = close_jes
        .iter()
        .filter(|je| {
            je.header
                .header_text
                .as_ref()
                .map_or(false, |t| t.contains("Tax provision"))
        })
        .collect();

    if !tax_jes.is_empty() {
        let tax_je = tax_jes[0];

        // Should have exactly 2 lines
        assert_eq!(
            tax_je.line_count(),
            2,
            "Tax provision JE should have 2 lines"
        );

        // Verify accounts used
        let accounts: Vec<&str> = tax_je.lines.iter().map(|l| l.gl_account.as_str()).collect();
        assert!(
            accounts.contains(&tax_accounts::TAX_EXPENSE),
            "Tax provision JE should debit Tax Expense ({})",
            tax_accounts::TAX_EXPENSE
        );
        assert!(
            accounts.contains(&tax_accounts::SALES_TAX_PAYABLE),
            "Tax provision JE should credit Tax Payable ({})",
            tax_accounts::SALES_TAX_PAYABLE
        );
    }
}

/// Test that income statement closing JE transfers to retained earnings.
#[test]
fn test_closing_entry_retained_earnings() {
    let mut config = minimal_config();
    config.global.seed = Some(203);
    config.global.period_months = 3;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        generate_period_close: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    let close_jes = close_entries(&result.journal_entries);

    // Find income statement closing JE
    let closing_jes: Vec<_> = close_jes
        .iter()
        .filter(|je| {
            je.header
                .header_text
                .as_ref()
                .map_or(false, |t| t.contains("Income statement close"))
        })
        .collect();

    if !closing_jes.is_empty() {
        let closing_je = closing_jes[0];

        // Should have exactly 2 lines
        assert_eq!(closing_je.line_count(), 2, "Closing JE should have 2 lines");

        // Verify accounts used
        let accounts: Vec<&str> = closing_je
            .lines
            .iter()
            .map(|l| l.gl_account.as_str())
            .collect();
        assert!(
            accounts.contains(&equity_accounts::INCOME_SUMMARY),
            "Closing JE should use Income Summary ({})",
            equity_accounts::INCOME_SUMMARY
        );
        assert!(
            accounts.contains(&equity_accounts::RETAINED_EARNINGS),
            "Closing JE should use Retained Earnings ({})",
            equity_accounts::RETAINED_EARNINGS
        );
    }
}

/// Test that period close is skipped when disabled.
#[test]
fn test_period_close_skipped_when_disabled() {
    let mut config = minimal_config();
    config.global.seed = Some(204);
    config.global.period_months = 3;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        generate_period_close: false,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // No CL-type entries should be generated
    let close_jes = close_entries(&result.journal_entries);
    assert!(
        close_jes.is_empty(),
        "Should not generate period-close entries when disabled, got {}",
        close_jes.len()
    );
    assert_eq!(
        result.statistics.period_close_je_count, 0,
        "period_close_je_count should be 0 when disabled"
    );
}

/// Test that closing entry amounts are consistent with income and tax.
#[test]
fn test_period_close_amount_consistency() {
    let mut config = minimal_config();
    config.global.seed = Some(205);
    config.global.period_months = 6;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        generate_period_close: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    let close_jes = close_entries(&result.journal_entries);

    // If we have both a tax provision and a closing entry, verify that:
    // closing_amount = pre_tax_income - tax_amount
    let tax_jes: Vec<_> = close_jes
        .iter()
        .filter(|je| {
            je.header
                .header_text
                .as_ref()
                .map_or(false, |t| t.contains("Tax provision"))
        })
        .collect();

    let closing_jes: Vec<_> = close_jes
        .iter()
        .filter(|je| {
            je.header
                .header_text
                .as_ref()
                .map_or(false, |t| t.contains("Income statement close"))
        })
        .collect();

    if !tax_jes.is_empty() && !closing_jes.is_empty() {
        let tax_amount = tax_jes[0].total_debit(); // DR side = tax expense amount
        let close_amount = closing_jes[0].total_debit(); // DR side = net income transferred

        // Verify: close_amount = pre_tax_income * (1 - 0.21)
        // pre_tax_income = tax_amount / 0.21
        // Expected: close_amount = pre_tax_income - tax_amount
        let expected_pre_tax =
            (tax_amount * Decimal::new(100, 0) / Decimal::new(21, 0)).round_dp(2);
        let expected_close = (expected_pre_tax - tax_amount).round_dp(2);

        // Allow small rounding tolerance (1 cent)
        let diff = (close_amount - expected_close).abs();
        assert!(
            diff <= Decimal::new(1, 2),
            "Closing amount {} should be approximately {} (pre_tax={}, tax={}), diff={}",
            close_amount,
            expected_close,
            expected_pre_tax,
            tax_amount,
            diff
        );
    }
}

/// Test that close entries have the correct metadata.
#[test]
fn test_period_close_metadata() {
    let mut config = minimal_config();
    config.global.seed = Some(206);
    config.global.period_months = 3;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        generate_period_close: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    let close_jes = close_entries(&result.journal_entries);

    for je in &close_jes {
        assert_eq!(
            je.header.document_type, "CL",
            "Period-close JE should have document_type 'CL'"
        );
        assert_eq!(
            je.header.created_by, "CLOSE_ENGINE",
            "Period-close JE should be created by CLOSE_ENGINE"
        );
        assert_eq!(
            je.header.company_code, "TEST",
            "Period-close JE should use the correct company code"
        );
    }
}
