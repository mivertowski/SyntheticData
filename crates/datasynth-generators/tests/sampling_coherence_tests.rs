//! Integration tests for JE-aware audit sampling (ISA 530).
//!
//! Verifies that `generate_for_cras_with_population` selects key items and
//! representative items from real JournalEntry data, using actual document IDs
//! and amounts, and falls back to synthetic generation when no matching JE
//! lines exist for a given account area.

use std::collections::HashMap;

use datasynth_core::models::audit::risk_assessment_cra::{
    AuditAssertion, CombinedRiskAssessment, RiskRating,
};
use datasynth_core::models::audit::sampling_plan::SelectionType;
use datasynth_core::models::journal_entry::{JournalEntry, JournalEntryHeader, JournalEntryLine};
use datasynth_generators::audit::sampling_plan_generator::SamplingPlanGenerator;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use smallvec::smallvec;
use uuid::Uuid;

/// Build a test JournalEntry with the given account code and debit amount.
fn make_je(account_code: &str, debit_amount: Decimal) -> JournalEntry {
    let doc_id = Uuid::new_v4();
    let posting_date = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    let header = JournalEntryHeader::new("C001".to_string(), posting_date);
    // Override the document_id to be deterministic per call
    let header = JournalEntryHeader {
        document_id: doc_id,
        ..header
    };
    let line = JournalEntryLine::debit(doc_id, 1, account_code.to_string(), debit_amount);
    JournalEntry {
        header,
        lines: smallvec![line],
    }
}

/// Build a test JournalEntry with a specific UUID.
fn make_je_with_id(id: Uuid, account_code: &str, debit_amount: Decimal) -> JournalEntry {
    let posting_date = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    let header = JournalEntryHeader {
        document_id: id,
        ..JournalEntryHeader::new("C001".to_string(), posting_date)
    };
    let line = JournalEntryLine::debit(id, 1, account_code.to_string(), debit_amount);
    JournalEntry {
        header,
        lines: smallvec![line],
    }
}

fn make_cra(
    account_area: &str,
    assertion: AuditAssertion,
    ir: RiskRating,
    cr: RiskRating,
) -> CombinedRiskAssessment {
    CombinedRiskAssessment::new("C001", account_area, assertion, ir, cr, false, vec![])
}

#[test]
fn key_items_use_real_document_ids_and_exceed_tolerable_error() {
    let te = dec!(50_000);

    // Create JE lines with revenue account codes (prefix "4"), some above TE
    let jes: Vec<JournalEntry> = vec![
        make_je("4100", dec!(100_000)), // above TE
        make_je("4100", dec!(75_000)),  // above TE
        make_je("4100", dec!(60_000)),  // above TE
        make_je("4200", dec!(30_000)),  // below TE
        make_je("4200", dec!(10_000)),  // below TE
    ];

    let expected_key_ids: Vec<String> = jes[..3]
        .iter()
        .map(|je| je.header.document_id.to_string())
        .collect();

    let cra = make_cra(
        "Revenue",
        AuditAssertion::Occurrence,
        RiskRating::High,
        RiskRating::High,
    );

    let mut gen = SamplingPlanGenerator::new(42);
    let (plans, items) = gen.generate_for_cras_with_population(
        &[cra],
        Some(te),
        &jes,
        &HashMap::new(),
    );

    assert_eq!(plans.len(), 1);
    let plan = &plans[0];

    // All key items must have real UUIDs from our JE data
    assert!(
        !plan.key_items.is_empty(),
        "Should have key items from JE population"
    );
    for ki in &plan.key_items {
        assert!(
            ki.amount > te,
            "Key item amount {} must be > tolerable error {}",
            ki.amount,
            te
        );
        assert!(
            expected_key_ids.contains(&ki.item_id),
            "Key item ID {} must be a real JE document_id",
            ki.item_id
        );
    }

    // Key items in sampled_items should also have real IDs
    let key_sampled: Vec<_> = items
        .iter()
        .filter(|i| i.selection_type == SelectionType::KeyItem)
        .collect();
    for si in &key_sampled {
        assert!(
            expected_key_ids.contains(&si.item_id),
            "Sampled key item ID {} must be a real JE document_id",
            si.item_id
        );
    }
}

#[test]
fn representative_items_use_real_document_ids() {
    let te = dec!(50_000);

    // Many JEs below TE to populate representative sample
    let mut jes: Vec<JournalEntry> = Vec::new();
    for i in 0..100 {
        let amount = dec!(1_000) + Decimal::from(i * 100);
        jes.push(make_je("4100", amount));
    }
    // One JE above TE to be a key item
    jes.push(make_je("4200", dec!(80_000)));

    let all_ids: Vec<String> = jes
        .iter()
        .map(|je| je.header.document_id.to_string())
        .collect();

    let cra = make_cra(
        "Revenue",
        AuditAssertion::Occurrence,
        RiskRating::High,
        RiskRating::High,
    );

    let mut gen = SamplingPlanGenerator::new(99);
    let (plans, items) = gen.generate_for_cras_with_population(
        &[cra],
        Some(te),
        &jes,
        &HashMap::new(),
    );

    assert_eq!(plans.len(), 1);

    let rep_items: Vec<_> = items
        .iter()
        .filter(|i| i.selection_type == SelectionType::Representative)
        .collect();

    assert!(
        !rep_items.is_empty(),
        "Should have representative items from JE population"
    );

    for ri in &rep_items {
        assert!(
            all_ids.contains(&ri.item_id),
            "Representative item ID {} must be a real JE document_id",
            ri.item_id
        );
    }
}

#[test]
fn fallback_to_synthetic_when_no_matching_jes() {
    let te = dec!(50_000);

    // Create JEs with cash account codes (prefix "10"), not revenue
    let jes: Vec<JournalEntry> = vec![
        make_je("1000", dec!(100_000)),
        make_je("1010", dec!(200_000)),
    ];

    // CRA for Revenue area -- no JE lines match prefix "4"
    let cra = make_cra(
        "Revenue",
        AuditAssertion::Occurrence,
        RiskRating::High,
        RiskRating::High,
    );

    let mut gen = SamplingPlanGenerator::new(77);
    let (plans, items) = gen.generate_for_cras_with_population(
        &[cra],
        Some(te),
        &jes,
        &HashMap::new(),
    );

    assert_eq!(plans.len(), 1, "Should still generate a plan via fallback");
    let plan = &plans[0];

    // Synthetic fallback: key item IDs should be synthetic (contain "KEY-")
    for ki in &plan.key_items {
        assert!(
            ki.item_id.contains("KEY-"),
            "Synthetic fallback key item ID {} should contain 'KEY-'",
            ki.item_id
        );
    }

    // Sampled items should also be synthetic
    assert!(
        !items.is_empty(),
        "Should have sampled items from synthetic fallback"
    );
}

#[test]
fn population_metrics_reflect_real_data() {
    let te = dec!(5_000);

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();

    let jes = vec![
        make_je_with_id(id1, "1100", dec!(10_000)),
        make_je_with_id(id2, "1100", dec!(20_000)),
        make_je_with_id(id3, "1100", dec!(3_000)),
    ];

    let cra = make_cra(
        "Trade Receivables",
        AuditAssertion::Existence,
        RiskRating::Medium,
        RiskRating::Medium,
    );

    let mut gen = SamplingPlanGenerator::new(55);
    let (plans, _items) = gen.generate_for_cras_with_population(
        &[cra],
        Some(te),
        &jes,
        &HashMap::new(),
    );

    assert_eq!(plans.len(), 1);
    let plan = &plans[0];

    // Population size should equal the number of matching lines
    assert_eq!(plan.population_size, 3);
    // Population value should be the sum of all line amounts
    assert_eq!(plan.population_value, dec!(33_000));
}

#[test]
fn key_items_capped_at_20() {
    let te = dec!(1_000);

    // Create 30 JEs all above TE with receivables account codes
    let jes: Vec<JournalEntry> = (0..30)
        .map(|i| make_je("1100", dec!(5_000) + Decimal::from(i * 1000)))
        .collect();

    let cra = make_cra(
        "Trade Receivables",
        AuditAssertion::Existence,
        RiskRating::High,
        RiskRating::High,
    );

    let mut gen = SamplingPlanGenerator::new(42);
    let (plans, _items) = gen.generate_for_cras_with_population(
        &[cra],
        Some(te),
        &jes,
        &HashMap::new(),
    );

    assert_eq!(plans.len(), 1);
    let plan = &plans[0];
    assert!(
        plan.key_items.len() <= 20,
        "Key items should be capped at 20, got {}",
        plan.key_items.len()
    );
}
