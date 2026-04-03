//! Integration tests verifying that the audit FSM produces coherent
//! financial data when real journal entries are provided.

use chrono::NaiveDate;
use datasynth_audit_fsm::context::EngagementContext;
use datasynth_audit_fsm::engine::AuditFsmEngine;
use datasynth_audit_fsm::loader::{default_overlay, BlueprintWithPreconditions};
use datasynth_core::models::audit::sampling_plan::SelectionType;
use datasynth_core::models::journal_entry::{JournalEntry, JournalEntryHeader, JournalEntryLine};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use smallvec::smallvec;
use std::collections::{HashMap, HashSet};

/// Build a minimal JournalEntry with one line for testing.
fn make_test_je(account_code: &str, debit: Decimal, credit: Decimal) -> JournalEntry {
    let posting_date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    let mut header = JournalEntryHeader::new("C001".to_string(), posting_date);
    header.fiscal_year = 2024;
    header.fiscal_period = 6;
    header.document_type = "SA".to_string();
    header.currency = "USD".to_string();
    header.created_by = "TEST".to_string();

    let doc_id = header.document_id;

    JournalEntry {
        header,
        lines: smallvec![JournalEntryLine {
            document_id: doc_id,
            line_number: 1,
            gl_account: account_code.to_string(),
            account_code: account_code.to_string(),
            debit_amount: debit,
            credit_amount: credit,
            local_amount: debit - credit,
            ..Default::default()
        }],
    }
}

#[test]
fn test_fsm_sampling_uses_real_je_ids() {
    // Create a realistic JE population
    let mut entries = Vec::new();

    // Revenue JEs (4000) -- large ones should become key items
    for _ in 0..5 {
        entries.push(make_test_je("4000", dec!(0), dec!(200000)));
    }
    for _ in 0..20 {
        entries.push(make_test_je("4000", dec!(0), dec!(15000)));
    }
    // AR JEs (1100)
    for _ in 0..30 {
        entries.push(make_test_je("1100", dec!(25000), dec!(0)));
    }
    // Expense JEs
    for _ in 0..15 {
        entries.push(make_test_je("5000", dec!(30000), dec!(0)));
        entries.push(make_test_je("6100", dec!(10000), dec!(0)));
    }

    // Collect all JE document IDs for verification
    let all_je_ids: HashSet<String> = entries
        .iter()
        .map(|e| e.header.document_id.to_string())
        .collect();

    // Build account balances
    let mut account_balances = HashMap::new();
    for je in &entries {
        for line in &je.lines {
            let d: f64 = line.debit_amount.to_string().parse().unwrap_or(0.0);
            let c: f64 = line.credit_amount.to_string().parse().unwrap_or(0.0);
            *account_balances
                .entry(line.account_code.clone())
                .or_insert(0.0) += d - c;
        }
    }

    // Build context with real JE data
    let context = EngagementContext {
        company_code: "C001".to_string(),
        company_name: "Test Corp".to_string(),
        fiscal_year: 2024,
        currency: "USD".to_string(),
        total_revenue: dec!(1300000),
        total_assets: dec!(750000),
        engagement_start: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        report_date: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        pretax_income: dec!(400000),
        equity: dec!(500000),
        gross_profit: dec!(455000),
        working_capital: dec!(300000),
        operating_cash_flow: dec!(340000),
        total_debt: dec!(200000),
        team_member_ids: vec!["EMP001".to_string()],
        team_member_pairs: vec![("EMP001".to_string(), "Auditor One".to_string())],
        accounts: vec![
            "4000".to_string(),
            "1100".to_string(),
            "5000".to_string(),
            "6100".to_string(),
        ],
        vendor_names: vec!["Vendor A".to_string()],
        customer_names: vec!["Customer B".to_string()],
        journal_entry_ids: entries
            .iter()
            .take(50)
            .map(|e| e.header.document_id.to_string())
            .collect(),
        account_balances,
        control_ids: vec![],
        anomaly_refs: vec![],
        is_us_listed: false,
        entity_codes: vec!["C001".to_string()],
        journal_entries: entries,
        auditor_firm_name: "DataSynth Audit LLP".to_string(),
        accounting_framework: "IFRS".to_string(),
    };

    // Run FSM with builtin FSA blueprint
    let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
    bwp.validate().unwrap();
    let overlay = default_overlay();
    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
    let result = engine.run_engagement(&context).unwrap();

    // COHERENCE CHECKS
    let sampled = &result.artifacts.sampled_items;
    assert!(!sampled.is_empty(), "FSM should produce sampled items");

    // Count how many sampled items reference real JE document IDs
    let real_id_count = sampled
        .iter()
        .filter(|item| all_je_ids.contains(&item.item_id))
        .count();

    assert!(
        real_id_count > 0,
        "Expected at least some sampled items with real JE document IDs, \
         but none of {} items matched",
        sampled.len()
    );

    // Key items with real IDs should have amounts > performance materiality
    if let Some(mat) = result.artifacts.materiality_calculations.first() {
        let te = mat.performance_materiality;
        let real_key_items: Vec<_> = sampled
            .iter()
            .filter(|i| {
                i.selection_type == SelectionType::KeyItem && all_je_ids.contains(&i.item_id)
            })
            .collect();

        for ki in &real_key_items {
            assert!(
                ki.amount > te,
                "Key item {} has amount {} which should be > tolerable error {}",
                ki.item_id,
                ki.amount,
                te
            );
        }
    }
}

#[test]
fn test_fsm_without_je_data_falls_back_to_synthetic() {
    // Context WITHOUT journal_entries -- should use synthetic sampling
    let context = EngagementContext::demo();

    let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
    bwp.validate().unwrap();
    let overlay = default_overlay();
    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
    let result = engine.run_engagement(&context).unwrap();

    // Should still produce sampled items (synthetic path)
    assert!(
        !result.artifacts.sampled_items.is_empty(),
        "FSM should produce sampled items even without JE data (synthetic fallback)"
    );

    // Synthetic items should use slug format (not UUID format)
    let first_item = &result.artifacts.sampled_items[0];
    // Synthetic items have format like "C001-TRADE_RECEIVABLES-KEY-001" or "SP-...-REP-0001"
    assert!(
        !first_item.item_id.contains('-')
            || first_item.item_id.contains("KEY")
            || first_item.item_id.contains("REP"),
        "Synthetic item IDs should use slug format, got: {}",
        first_item.item_id
    );
}
