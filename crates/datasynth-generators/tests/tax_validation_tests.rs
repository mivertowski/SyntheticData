//! Integration tests for the full tax data generation pipeline.
//!
//! Validates end-to-end workflows: jurisdiction setup -> tax code generation ->
//! tax line decoration -> tax return aggregation -> provision calculation ->
//! anomaly injection. Checks cross-generator consistency, determinism, and
//! return aggregation accuracy.

#![allow(clippy::unwrap_used)]

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use datasynth_config::schema::TaxConfig;
use datasynth_core::models::{TaxLine, TaxableDocumentType};
use datasynth_generators::tax::{
    TaxAnomalyInjector, TaxCodeGenerator, TaxLineGenerator, TaxLineGeneratorConfig,
    TaxProvisionGenerator, TaxReturnGenerator, WithholdingGenerator,
};

// =============================================================================
// Helpers
// =============================================================================

/// Generates tax codes and jurisdictions with subnational data for US, DE, GB, FR.
fn setup_tax_codes() -> SetupResult {
    let mut config = TaxConfig::default();
    config.jurisdictions.countries = vec![
        "US".into(),
        "DE".into(),
        "GB".into(),
        "FR".into(),
    ];
    config.jurisdictions.include_subnational = true;

    let mut gen = TaxCodeGenerator::with_config(42, config);
    let (jurisdictions, codes) = gen.generate();
    SetupResult {
        jurisdictions,
        codes,
    }
}

struct SetupResult {
    jurisdictions: Vec<datasynth_core::models::TaxJurisdiction>,
    codes: Vec<datasynth_core::models::TaxCode>,
}

fn test_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()
}

fn period_start() -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
}

fn period_end() -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 6, 30).unwrap()
}

fn filing_deadline() -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 7, 31).unwrap()
}

/// Generates a batch of mock vendor and customer invoices, returning the
/// resulting tax lines from the TaxLineGenerator.
fn generate_mixed_tax_lines(codes: Vec<datasynth_core::models::TaxCode>) -> Vec<TaxLine> {
    let config = TaxLineGeneratorConfig::default();
    let mut line_gen = TaxLineGenerator::new(42, codes, config);
    let date = test_date();

    let mut all_lines = Vec::new();

    // Vendor invoices (input tax) — DE domestic
    for i in 0..10 {
        let lines = line_gen.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            &format!("VINV-DE-{i:03}"),
            "DE",
            "DE",
            dec!(10000) + Decimal::from(i) * dec!(500),
            date,
            None,
        );
        all_lines.extend(lines);
    }

    // Customer invoices (output tax) — DE domestic
    for i in 0..15 {
        let lines = line_gen.generate_for_document(
            TaxableDocumentType::CustomerInvoice,
            &format!("CINV-DE-{i:03}"),
            "DE",
            "DE",
            dec!(8000) + Decimal::from(i) * dec!(300),
            date,
            None,
        );
        all_lines.extend(lines);
    }

    // Vendor invoices (input tax) — GB domestic
    for i in 0..5 {
        let lines = line_gen.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            &format!("VINV-GB-{i:03}"),
            "GB",
            "GB",
            dec!(5000) + Decimal::from(i) * dec!(1000),
            date,
            None,
        );
        all_lines.extend(lines);
    }

    // Customer invoices (output tax) — GB domestic
    for i in 0..8 {
        let lines = line_gen.generate_for_document(
            TaxableDocumentType::CustomerInvoice,
            &format!("CINV-GB-{i:03}"),
            "GB",
            "GB",
            dec!(6000) + Decimal::from(i) * dec!(200),
            date,
            None,
        );
        all_lines.extend(lines);
    }

    // EU cross-border (DE seller -> FR buyer) — reverse charge
    for i in 0..3 {
        let lines = line_gen.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            &format!("VINV-EU-{i:03}"),
            "DE",
            "FR",
            dec!(20000) + Decimal::from(i) * dec!(5000),
            date,
            None,
        );
        all_lines.extend(lines);
    }

    // US state-level customer invoices (destination = US-CA)
    for i in 0..4 {
        let lines = line_gen.generate_for_document(
            TaxableDocumentType::CustomerInvoice,
            &format!("CINV-US-{i:03}"),
            "US",
            "US-CA",
            dec!(3000) + Decimal::from(i) * dec!(500),
            date,
            None,
        );
        all_lines.extend(lines);
    }

    all_lines
}

// =============================================================================
// 1. Full Pipeline Test
// =============================================================================

#[test]
fn test_full_tax_pipeline() {
    // Step 1: Generate jurisdictions and tax codes
    let setup = setup_tax_codes();
    assert!(
        !setup.jurisdictions.is_empty(),
        "Should produce jurisdictions"
    );
    assert!(!setup.codes.is_empty(), "Should produce tax codes");

    // Step 2: Generate tax lines for mock invoices
    let mut tax_lines = generate_mixed_tax_lines(setup.codes.clone());
    assert!(
        !tax_lines.is_empty(),
        "Should produce tax lines from mock invoices"
    );

    // Step 3: Aggregate tax lines into returns
    let mut return_gen = TaxReturnGenerator::new(42);
    let mut tax_returns = return_gen.generate(
        "ENT-001",
        &tax_lines,
        period_start(),
        period_end(),
        filing_deadline(),
    );
    assert!(
        !tax_returns.is_empty(),
        "Should produce at least one tax return"
    );

    // Step 4: Generate tax provision
    let mut provision_gen = TaxProvisionGenerator::new(42);
    let provision = provision_gen.generate(
        "ENT-001",
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        dec!(5000000),
        dec!(0.21),
    );
    assert!(!provision.id.is_empty(), "Provision should have an ID");
    assert_eq!(provision.statutory_rate, dec!(0.21));
    assert!(
        !provision.rate_reconciliation.is_empty(),
        "Should have rate reconciliation items"
    );

    // Step 5: Generate withholding tax records
    let mut wht_gen = WithholdingGenerator::new(42, dec!(0.30)).with_standard_treaties();
    let payments = vec![
        (
            "PAY-001".to_string(),
            "V-GB-01".to_string(),
            "GB".to_string(),
            dec!(100000),
        ),
        (
            "PAY-002".to_string(),
            "V-IN-01".to_string(),
            "IN".to_string(),
            dec!(50000),
        ),
        (
            "PAY-003".to_string(),
            "V-US-01".to_string(),
            "US".to_string(), // domestic — should be excluded
            dec!(75000),
        ),
    ];
    let mut wht_records = wht_gen.generate(&payments, "US");
    assert_eq!(
        wht_records.len(),
        2,
        "Should produce 2 withholding records (domestic excluded)"
    );

    // Step 6: Inject anomalies into tax lines
    let original_line_count = tax_lines.len();
    let mut anomaly_injector = TaxAnomalyInjector::new(42, 0.10);
    let line_anomaly_labels = anomaly_injector.inject_into_tax_lines(&mut tax_lines);

    // Some lines may have been removed (MissingTaxLine anomaly)
    let missing_count = line_anomaly_labels
        .iter()
        .filter(|l| l.anomaly_type == datasynth_generators::tax::TaxAnomalyType::MissingTaxLine)
        .count();
    assert_eq!(
        tax_lines.len(),
        original_line_count - missing_count,
        "Remaining lines should account for removed ones"
    );

    // Step 7: Inject anomalies into returns and withholding
    let return_anomaly_labels = anomaly_injector.inject_into_returns(&mut tax_returns);
    let wht_anomaly_labels = anomaly_injector.inject_into_withholding(&mut wht_records);

    // Verify all anomaly labels have required fields
    let all_labels: Vec<_> = line_anomaly_labels
        .iter()
        .chain(return_anomaly_labels.iter())
        .chain(wht_anomaly_labels.iter())
        .collect();

    for label in &all_labels {
        assert!(!label.id.is_empty(), "Anomaly label should have an ID");
        assert!(
            !label.document_type.is_empty(),
            "Anomaly label should have a document_type"
        );
        assert!(
            !label.document_id.is_empty(),
            "Anomaly label should have a document_id"
        );
        assert!(
            !label.description.is_empty(),
            "Anomaly label should have a description"
        );
    }

    // Verify the pipeline produced all the expected artifact types
    assert!(!setup.jurisdictions.is_empty(), "Jurisdictions present");
    assert!(!setup.codes.is_empty(), "Tax codes present");
    assert!(!tax_lines.is_empty(), "Tax lines present (post-anomaly)");
    assert!(!tax_returns.is_empty(), "Tax returns present");
    assert!(!provision.id.is_empty(), "Tax provision present");
    assert!(!wht_records.is_empty(), "Withholding records present");
}

// =============================================================================
// 2. Cross-Generator Consistency Tests
// =============================================================================

#[test]
fn test_cross_generator_consistency() {
    let setup = setup_tax_codes();
    let tax_lines = generate_mixed_tax_lines(setup.codes.clone());

    // Collect all valid tax code IDs and jurisdiction IDs
    let valid_code_ids: HashSet<String> = setup.codes.iter().map(|c| c.id.clone()).collect();
    let valid_jurisdiction_ids: HashSet<String> =
        setup.jurisdictions.iter().map(|j| j.id.clone()).collect();

    // Verify every tax line references a valid tax code
    for line in &tax_lines {
        assert!(
            valid_code_ids.contains(&line.tax_code_id),
            "Tax line {} references unknown tax_code_id '{}'. Valid codes: {:?}",
            line.id,
            line.tax_code_id,
            valid_code_ids
        );
    }

    // Verify every tax line references a valid jurisdiction
    for line in &tax_lines {
        assert!(
            valid_jurisdiction_ids.contains(&line.jurisdiction_id),
            "Tax line {} references unknown jurisdiction_id '{}'. Valid jurisdictions: {:?}",
            line.id,
            line.jurisdiction_id,
            valid_jurisdiction_ids
        );
    }

    // Verify every tax code references a valid jurisdiction
    for code in &setup.codes {
        assert!(
            valid_jurisdiction_ids.contains(&code.jurisdiction_id),
            "Tax code {} references unknown jurisdiction_id '{}'",
            code.id,
            code.jurisdiction_id
        );
    }

    // Verify tax returns reference valid jurisdictions from the lines
    let mut return_gen = TaxReturnGenerator::new(42);
    let returns = return_gen.generate(
        "ENT-001",
        &tax_lines,
        period_start(),
        period_end(),
        filing_deadline(),
    );

    let line_jurisdiction_ids: HashSet<String> =
        tax_lines.iter().map(|l| l.jurisdiction_id.clone()).collect();

    for ret in &returns {
        assert!(
            line_jurisdiction_ids.contains(&ret.jurisdiction_id),
            "Tax return {} references jurisdiction '{}' not found in tax lines",
            ret.id,
            ret.jurisdiction_id
        );
    }
}

#[test]
fn test_cross_generator_tax_code_active_on_date() {
    // Verify that for each tax line, the referenced tax code was active
    // on the date the line was generated.
    let setup = setup_tax_codes();
    let date = test_date();

    let code_map: HashMap<String, &datasynth_core::models::TaxCode> =
        setup.codes.iter().map(|c| (c.id.clone(), c)).collect();

    let tax_lines = generate_mixed_tax_lines(setup.codes.clone());

    for line in &tax_lines {
        if let Some(code) = code_map.get(&line.tax_code_id) {
            assert!(
                code.is_active(date),
                "Tax line {} references tax code '{}' which is NOT active on {}",
                line.id,
                line.tax_code_id,
                date
            );
        }
    }
}

#[test]
fn test_vendor_invoice_lines_are_deductible() {
    let setup = setup_tax_codes();
    let config = TaxLineGeneratorConfig::default();
    let mut line_gen = TaxLineGenerator::new(42, setup.codes, config);

    // Domestic vendor invoices should be deductible
    for i in 0..5 {
        let lines = line_gen.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            &format!("VINV-TEST-{i}"),
            "DE",
            "DE",
            dec!(10000),
            test_date(),
            None,
        );
        for line in &lines {
            assert!(
                line.is_deductible,
                "Vendor invoice tax line {} should be deductible",
                line.id
            );
        }
    }

    // Customer invoices should NOT be deductible
    for i in 0..5 {
        let lines = line_gen.generate_for_document(
            TaxableDocumentType::CustomerInvoice,
            &format!("CINV-TEST-{i}"),
            "DE",
            "DE",
            dec!(10000),
            test_date(),
            None,
        );
        for line in &lines {
            assert!(
                !line.is_deductible,
                "Customer invoice tax line {} should NOT be deductible",
                line.id
            );
        }
    }
}

// =============================================================================
// 3. Determinism Tests
// =============================================================================

#[test]
fn test_deterministic_pipeline() {
    // Run the full pipeline twice with the same seed and verify identical output.

    let results: Vec<PipelineResult> = (0..2).map(|_| run_deterministic_pipeline(42)).collect();

    let r1 = &results[0];
    let r2 = &results[1];

    // 3a. Jurisdictions are identical
    assert_eq!(
        r1.jurisdictions.len(),
        r2.jurisdictions.len(),
        "Jurisdiction count should match"
    );
    for (j1, j2) in r1.jurisdictions.iter().zip(r2.jurisdictions.iter()) {
        assert_eq!(j1.id, j2.id, "Jurisdiction IDs should match");
        assert_eq!(j1.name, j2.name, "Jurisdiction names should match");
        assert_eq!(
            j1.country_code, j2.country_code,
            "Country codes should match"
        );
        assert_eq!(
            j1.jurisdiction_type, j2.jurisdiction_type,
            "Jurisdiction types should match"
        );
        assert_eq!(
            j1.vat_registered, j2.vat_registered,
            "VAT registration should match"
        );
    }

    // 3b. Tax codes are identical
    assert_eq!(
        r1.codes.len(),
        r2.codes.len(),
        "Tax code count should match"
    );
    for (c1, c2) in r1.codes.iter().zip(r2.codes.iter()) {
        assert_eq!(c1.id, c2.id, "Tax code IDs should match");
        assert_eq!(c1.code, c2.code, "Tax code mnemonics should match");
        assert_eq!(c1.rate, c2.rate, "Tax code rates should match");
        assert_eq!(c1.tax_type, c2.tax_type, "Tax types should match");
        assert_eq!(
            c1.jurisdiction_id, c2.jurisdiction_id,
            "Jurisdiction IDs should match"
        );
    }

    // 3c. Tax lines are identical
    assert_eq!(
        r1.tax_lines.len(),
        r2.tax_lines.len(),
        "Tax line count should match"
    );
    for (l1, l2) in r1.tax_lines.iter().zip(r2.tax_lines.iter()) {
        assert_eq!(l1.id, l2.id, "Tax line IDs should match");
        assert_eq!(
            l1.document_id, l2.document_id,
            "Document IDs should match"
        );
        assert_eq!(
            l1.tax_code_id, l2.tax_code_id,
            "Tax code IDs should match"
        );
        assert_eq!(
            l1.jurisdiction_id, l2.jurisdiction_id,
            "Jurisdiction IDs should match"
        );
        assert_eq!(
            l1.taxable_amount, l2.taxable_amount,
            "Taxable amounts should match"
        );
        assert_eq!(
            l1.tax_amount, l2.tax_amount,
            "Tax amounts should match"
        );
        assert_eq!(
            l1.is_deductible, l2.is_deductible,
            "Deductible flags should match"
        );
        assert_eq!(
            l1.is_reverse_charge, l2.is_reverse_charge,
            "Reverse charge flags should match"
        );
    }

    // 3d. Tax returns are identical
    assert_eq!(
        r1.returns.len(),
        r2.returns.len(),
        "Tax return count should match"
    );
    for (ret1, ret2) in r1.returns.iter().zip(r2.returns.iter()) {
        assert_eq!(ret1.id, ret2.id, "Return IDs should match");
        assert_eq!(
            ret1.jurisdiction_id, ret2.jurisdiction_id,
            "Return jurisdictions should match"
        );
        assert_eq!(
            ret1.total_output_tax, ret2.total_output_tax,
            "Output tax should match"
        );
        assert_eq!(
            ret1.total_input_tax, ret2.total_input_tax,
            "Input tax should match"
        );
        assert_eq!(
            ret1.net_payable, ret2.net_payable,
            "Net payable should match"
        );
        assert_eq!(ret1.status, ret2.status, "Status should match");
        assert_eq!(ret1.is_late, ret2.is_late, "Late flag should match");
        assert_eq!(
            ret1.actual_filing_date, ret2.actual_filing_date,
            "Filing dates should match"
        );
    }

    // 3e. Provisions are identical
    assert_eq!(
        r1.provision.id, r2.provision.id,
        "Provision IDs should match"
    );
    assert_eq!(
        r1.provision.current_tax_expense, r2.provision.current_tax_expense,
        "Current tax expense should match"
    );
    assert_eq!(
        r1.provision.effective_rate, r2.provision.effective_rate,
        "Effective rate should match"
    );
    assert_eq!(
        r1.provision.deferred_tax_asset, r2.provision.deferred_tax_asset,
        "DTA should match"
    );
    assert_eq!(
        r1.provision.deferred_tax_liability, r2.provision.deferred_tax_liability,
        "DTL should match"
    );
    assert_eq!(
        r1.provision.rate_reconciliation.len(),
        r2.provision.rate_reconciliation.len(),
        "Reconciliation item count should match"
    );

    // 3f. Withholding records are identical
    assert_eq!(
        r1.wht_records.len(),
        r2.wht_records.len(),
        "WHT record count should match"
    );
    for (w1, w2) in r1.wht_records.iter().zip(r2.wht_records.iter()) {
        assert_eq!(w1.id, w2.id, "WHT IDs should match");
        assert_eq!(
            w1.applied_rate, w2.applied_rate,
            "Applied rates should match"
        );
        assert_eq!(
            w1.withheld_amount, w2.withheld_amount,
            "Withheld amounts should match"
        );
        assert_eq!(
            w1.certificate_number, w2.certificate_number,
            "Certificate numbers should match"
        );
    }
}

struct PipelineResult {
    jurisdictions: Vec<datasynth_core::models::TaxJurisdiction>,
    codes: Vec<datasynth_core::models::TaxCode>,
    tax_lines: Vec<TaxLine>,
    returns: Vec<datasynth_core::models::TaxReturn>,
    provision: datasynth_core::models::TaxProvision,
    wht_records: Vec<datasynth_core::models::WithholdingTaxRecord>,
}

fn run_deterministic_pipeline(seed: u64) -> PipelineResult {
    // Step 1: Tax codes
    let mut config = TaxConfig::default();
    config.jurisdictions.countries = vec!["US".into(), "DE".into(), "GB".into()];
    config.jurisdictions.include_subnational = true;

    let mut code_gen = TaxCodeGenerator::with_config(seed, config);
    let (jurisdictions, codes) = code_gen.generate();

    // Step 2: Tax lines
    let line_config = TaxLineGeneratorConfig::default();
    let mut line_gen = TaxLineGenerator::new(seed, codes.clone(), line_config);

    let mut tax_lines = Vec::new();
    for i in 0..5 {
        let lines = line_gen.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            &format!("VINV-{i:03}"),
            "DE",
            "DE",
            dec!(10000) + Decimal::from(i) * dec!(1000),
            test_date(),
            None,
        );
        tax_lines.extend(lines);
    }
    for i in 0..5 {
        let lines = line_gen.generate_for_document(
            TaxableDocumentType::CustomerInvoice,
            &format!("CINV-{i:03}"),
            "DE",
            "DE",
            dec!(8000) + Decimal::from(i) * dec!(500),
            test_date(),
            None,
        );
        tax_lines.extend(lines);
    }

    // Step 3: Tax returns
    let mut return_gen = TaxReturnGenerator::new(seed);
    let returns = return_gen.generate(
        "ENT-001",
        &tax_lines,
        period_start(),
        period_end(),
        filing_deadline(),
    );

    // Step 4: Tax provision
    let mut provision_gen = TaxProvisionGenerator::new(seed);
    let provision = provision_gen.generate(
        "ENT-001",
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        dec!(2000000),
        dec!(0.21),
    );

    // Step 5: Withholding
    let mut wht_gen = WithholdingGenerator::new(seed, dec!(0.30)).with_standard_treaties();
    let payments = vec![
        (
            "PAY-001".to_string(),
            "V-GB-01".to_string(),
            "GB".to_string(),
            dec!(100000),
        ),
        (
            "PAY-002".to_string(),
            "V-IN-01".to_string(),
            "IN".to_string(),
            dec!(50000),
        ),
    ];
    let wht_records = wht_gen.generate(&payments, "US");

    PipelineResult {
        jurisdictions,
        codes,
        tax_lines,
        returns,
        provision,
        wht_records,
    }
}

// =============================================================================
// 4. Return Aggregation Accuracy Tests
// =============================================================================

#[test]
fn test_return_aggregation_matches_lines() {
    let setup = setup_tax_codes();
    let tax_lines = generate_mixed_tax_lines(setup.codes);

    let mut return_gen = TaxReturnGenerator::new(42);
    let returns = return_gen.generate(
        "ENT-001",
        &tax_lines,
        period_start(),
        period_end(),
        filing_deadline(),
    );

    assert!(
        !returns.is_empty(),
        "Should have returns to verify aggregation"
    );

    // Group tax lines by jurisdiction and compute expected totals
    let mut expected_output_by_jur: HashMap<String, Decimal> = HashMap::new();
    let mut expected_input_by_jur: HashMap<String, Decimal> = HashMap::new();

    for line in &tax_lines {
        match line.document_type {
            TaxableDocumentType::CustomerInvoice => {
                *expected_output_by_jur
                    .entry(line.jurisdiction_id.clone())
                    .or_insert(Decimal::ZERO) += line.tax_amount;
            }
            TaxableDocumentType::VendorInvoice if line.is_deductible => {
                *expected_input_by_jur
                    .entry(line.jurisdiction_id.clone())
                    .or_insert(Decimal::ZERO) += line.tax_amount;
            }
            _ => {}
        }
    }

    // For each return, verify that totals match the summed tax lines
    for ret in &returns {
        let expected_output = expected_output_by_jur
            .get(&ret.jurisdiction_id)
            .copied()
            .unwrap_or(Decimal::ZERO);
        let expected_input = expected_input_by_jur
            .get(&ret.jurisdiction_id)
            .copied()
            .unwrap_or(Decimal::ZERO);

        assert_eq!(
            ret.total_output_tax, expected_output,
            "Return {} (jurisdiction={}): total_output_tax {} should equal summed CustomerInvoice \
             tax lines {}",
            ret.id, ret.jurisdiction_id, ret.total_output_tax, expected_output
        );
        assert_eq!(
            ret.total_input_tax, expected_input,
            "Return {} (jurisdiction={}): total_input_tax {} should equal summed deductible \
             VendorInvoice tax lines {}",
            ret.id, ret.jurisdiction_id, ret.total_input_tax, expected_input
        );

        // net_payable should be output - input
        let expected_net = (expected_output - expected_input).round_dp(2);
        assert_eq!(
            ret.net_payable, expected_net,
            "Return {} (jurisdiction={}): net_payable {} should equal output - input = {}",
            ret.id, ret.jurisdiction_id, ret.net_payable, expected_net
        );
    }
}

#[test]
fn test_return_covers_all_jurisdictions_in_lines() {
    let setup = setup_tax_codes();
    let tax_lines = generate_mixed_tax_lines(setup.codes);

    let mut return_gen = TaxReturnGenerator::new(42);
    let returns = return_gen.generate(
        "ENT-001",
        &tax_lines,
        period_start(),
        period_end(),
        filing_deadline(),
    );

    // Every jurisdiction present in tax lines should have a corresponding return
    let line_jurisdictions: HashSet<String> =
        tax_lines.iter().map(|l| l.jurisdiction_id.clone()).collect();
    let return_jurisdictions: HashSet<String> =
        returns.iter().map(|r| r.jurisdiction_id.clone()).collect();

    for jur in &line_jurisdictions {
        assert!(
            return_jurisdictions.contains(jur),
            "Jurisdiction '{}' appears in tax lines but has no corresponding tax return",
            jur
        );
    }
}

#[test]
fn test_return_net_payable_sign() {
    // When output tax > input tax, net_payable should be positive (owed to authority).
    // When input tax > output tax, net_payable should be negative (refund).
    let setup = setup_tax_codes();
    let config = TaxLineGeneratorConfig::default();
    let mut line_gen = TaxLineGenerator::new(42, setup.codes, config);
    let date = test_date();

    // Scenario: lots of output tax, no input tax
    let mut output_only_lines = Vec::new();
    for i in 0..10 {
        let lines = line_gen.generate_for_document(
            TaxableDocumentType::CustomerInvoice,
            &format!("CINV-NET-{i}"),
            "DE",
            "DE",
            dec!(50000),
            date,
            None,
        );
        output_only_lines.extend(lines);
    }

    let mut return_gen = TaxReturnGenerator::new(42);
    let returns = return_gen.generate(
        "ENT-001",
        &output_only_lines,
        period_start(),
        period_end(),
        filing_deadline(),
    );

    for ret in &returns {
        assert!(
            ret.net_payable >= Decimal::ZERO,
            "With only output tax, net_payable {} should be non-negative",
            ret.net_payable
        );
        assert_eq!(
            ret.total_input_tax,
            Decimal::ZERO,
            "No input tax expected"
        );
    }
}

// =============================================================================
// 5. Additional Cross-Cutting Tests
// =============================================================================

#[test]
fn test_provision_rate_reconciliation_integrity() {
    // Verify that the sum of rate reconciliation items equals
    // effective_rate - statutory_rate (within tolerance).
    let mut gen = TaxProvisionGenerator::new(42);

    for i in 0..10 {
        let provision = gen.generate(
            &format!("ENT-{i:03}"),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(1000000) + Decimal::from(i) * dec!(500000),
            dec!(0.21),
        );

        let total_impact: Decimal = provision
            .rate_reconciliation
            .iter()
            .map(|r| r.rate_impact)
            .sum();

        let expected_diff = (provision.effective_rate - provision.statutory_rate).round_dp(6);
        let actual_diff = total_impact.round_dp(6);

        let tolerance = dec!(0.000002);
        assert!(
            (expected_diff - actual_diff).abs() <= tolerance,
            "Provision {}: reconciliation items sum ({}) should equal effective_rate - \
             statutory_rate ({}). Diff: {}",
            provision.id,
            actual_diff,
            expected_diff,
            (expected_diff - actual_diff).abs()
        );
    }
}

#[test]
fn test_withholding_treaty_benefit_consistency() {
    let mut gen = WithholdingGenerator::new(42, dec!(0.30)).with_standard_treaties();

    let payments = vec![
        // Treaty country: GB -> 0% applied (benefit)
        (
            "PAY-001".to_string(),
            "V-GB-01".to_string(),
            "GB".to_string(),
            dec!(100000),
        ),
        // Treaty country: IN -> 15% applied (benefit, lower than 30%)
        (
            "PAY-002".to_string(),
            "V-IN-01".to_string(),
            "IN".to_string(),
            dec!(80000),
        ),
        // Non-treaty: ZZ -> 30% applied (no benefit)
        (
            "PAY-003".to_string(),
            "V-ZZ-01".to_string(),
            "ZZ".to_string(),
            dec!(60000),
        ),
    ];

    let records = gen.generate(&payments, "US");
    assert_eq!(records.len(), 3);

    // GB: treaty benefit (0% < 30%)
    let gb_rec = records.iter().find(|r| r.vendor_id == "V-GB-01").unwrap();
    assert!(
        gb_rec.has_treaty_benefit(),
        "GB record should have treaty benefit"
    );
    assert_eq!(gb_rec.applied_rate, dec!(0.00));
    assert_eq!(gb_rec.withheld_amount, dec!(0.00));

    // IN: treaty benefit (15% < 30%)
    let in_rec = records.iter().find(|r| r.vendor_id == "V-IN-01").unwrap();
    assert!(
        in_rec.has_treaty_benefit(),
        "IN record should have treaty benefit"
    );
    assert_eq!(in_rec.applied_rate, dec!(0.15));
    assert_eq!(in_rec.withheld_amount, dec!(12000.00));

    // ZZ: no treaty benefit
    let zz_rec = records.iter().find(|r| r.vendor_id == "V-ZZ-01").unwrap();
    assert!(
        !zz_rec.has_treaty_benefit(),
        "ZZ record should NOT have treaty benefit"
    );
    assert_eq!(zz_rec.applied_rate, dec!(0.30));
    assert_eq!(zz_rec.withheld_amount, dec!(18000.00));
}

#[test]
fn test_anomaly_injection_preserves_non_anomalous_lines() {
    // With a 0% anomaly rate, no lines should be modified.
    let setup = setup_tax_codes();
    let mut tax_lines = generate_mixed_tax_lines(setup.codes);

    // Capture original state
    let original_ids: Vec<String> = tax_lines.iter().map(|l| l.id.clone()).collect();
    let original_amounts: Vec<Decimal> = tax_lines.iter().map(|l| l.tax_amount).collect();
    let original_jurisdictions: Vec<String> =
        tax_lines.iter().map(|l| l.jurisdiction_id.clone()).collect();
    let original_count = tax_lines.len();

    let mut injector = TaxAnomalyInjector::new(42, 0.0);
    let labels = injector.inject_into_tax_lines(&mut tax_lines);

    assert!(labels.is_empty(), "No anomalies should be injected at 0% rate");
    assert_eq!(
        tax_lines.len(),
        original_count,
        "No lines should be removed at 0% rate"
    );

    for (i, line) in tax_lines.iter().enumerate() {
        assert_eq!(line.id, original_ids[i], "Line ID should be unchanged");
        assert_eq!(
            line.tax_amount, original_amounts[i],
            "Tax amount should be unchanged"
        );
        assert_eq!(
            line.jurisdiction_id, original_jurisdictions[i],
            "Jurisdiction should be unchanged"
        );
    }
}

#[test]
fn test_different_seeds_produce_different_output() {
    let r1 = run_deterministic_pipeline(1);
    let r2 = run_deterministic_pipeline(99999);

    // The jurisdictions and codes may be similar (determined by config),
    // but the return filing dates and provision details should differ
    // since they use RNG.
    let mut any_difference = false;

    if r1.provision.effective_rate != r2.provision.effective_rate {
        any_difference = true;
    }
    if r1.provision.deferred_tax_asset != r2.provision.deferred_tax_asset {
        any_difference = true;
    }

    // Returns may have different filing dates
    for (ret1, ret2) in r1.returns.iter().zip(r2.returns.iter()) {
        if ret1.actual_filing_date != ret2.actual_filing_date {
            any_difference = true;
        }
    }

    // Withholding certificate numbers should differ
    for (w1, w2) in r1.wht_records.iter().zip(r2.wht_records.iter()) {
        if w1.certificate_number != w2.certificate_number {
            any_difference = true;
        }
    }

    assert!(
        any_difference,
        "Different seeds should produce at least some different outputs"
    );
}
