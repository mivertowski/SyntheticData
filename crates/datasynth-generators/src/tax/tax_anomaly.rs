//! Tax Anomaly Injector.
//!
//! Injects labeled anomalies into tax data (tax lines, tax returns,
//! withholding records) for ML ground-truth generation. Each injected
//! anomaly produces a [`TaxAnomalyLabel`] that records the anomaly type,
//! severity, affected document, and original vs. anomalous values.

use chrono::Duration;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use datasynth_core::models::{TaxLine, TaxReturn, WithholdingTaxRecord};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Types of tax anomalies that can be injected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxAnomalyType {
    /// Wrong tax code applied for the jurisdiction/product combination.
    IncorrectTaxCode,
    /// Taxable transaction missing tax lines entirely.
    MissingTaxLine,
    /// Artificial routing through low-tax jurisdictions.
    RateArbitrage,
    /// Filing pattern trending toward deadline.
    LateFilingRisk,
    /// Transfer pricing outside arm's-length range.
    TransferPricingDeviation,
    /// Applied withholding rate below statutory without treaty justification.
    WithholdingUnderstatement,
}

/// Severity of the anomaly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxAnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

// ---------------------------------------------------------------------------
// Label
// ---------------------------------------------------------------------------

/// A labeled tax anomaly for ground truth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxAnomalyLabel {
    /// Unique anomaly label identifier.
    pub id: String,
    /// Type of the anomaly.
    pub anomaly_type: TaxAnomalyType,
    /// Severity of the anomaly.
    pub severity: TaxAnomalySeverity,
    /// Kind of document affected: `"tax_line"`, `"tax_return"`, or `"withholding_record"`.
    pub document_type: String,
    /// ID of the affected record.
    pub document_id: String,
    /// Human-readable description of the anomaly.
    pub description: String,
    /// What the correct value should be (if applicable).
    pub original_value: Option<String>,
    /// What was injected (if applicable).
    pub anomalous_value: Option<String>,
}

// ---------------------------------------------------------------------------
// Injector
// ---------------------------------------------------------------------------

/// Known low-tax jurisdictions used for rate-arbitrage injection.
const LOW_TAX_JURISDICTIONS: &[&str] = &[
    "JUR-IE",  // Ireland (12.5%)
    "JUR-SG",  // Singapore (17%)
    "JUR-HK",  // Hong Kong (16.5%)
    "JUR-BM",  // Bermuda (0%)
    "JUR-KY",  // Cayman Islands (0%)
    "JUR-LU",  // Luxembourg (varies, favorable regimes)
];

/// Injects tax anomalies into generated data.
pub struct TaxAnomalyInjector {
    rng: ChaCha8Rng,
    anomaly_rate: f64,
    counter: u64,
}

impl TaxAnomalyInjector {
    /// Creates a new tax anomaly injector.
    ///
    /// # Arguments
    ///
    /// * `seed` - Deterministic RNG seed.
    /// * `anomaly_rate` - Probability (0.0 to 1.0) that any given record is anomalous.
    pub fn new(seed: u64, anomaly_rate: f64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            anomaly_rate: anomaly_rate.clamp(0.0, 1.0),
            counter: 0,
        }
    }

    /// Inject anomalies into tax lines. Modifies lines in-place and returns labels.
    ///
    /// For each line, with probability `anomaly_rate`:
    /// - **IncorrectTaxCode** (40%): Change `tax_amount` to a wrong rate.
    /// - **MissingTaxLine** (30%): Remove the line from the vec (label still returned).
    /// - **RateArbitrage** (15%): Set `jurisdiction_id` to a known low-tax jurisdiction.
    /// - **WithholdingUnderstatement** (15%): Reduce effective tax rate below correct amount.
    pub fn inject_into_tax_lines(&mut self, lines: &mut Vec<TaxLine>) -> Vec<TaxAnomalyLabel> {
        let mut labels = Vec::new();
        let mut indices_to_remove: Vec<usize> = Vec::new();

        for (i, line) in lines.iter_mut().enumerate() {
            if !self.should_inject() {
                continue;
            }

            let roll: f64 = self.rng.gen();
            if roll < 0.40 {
                // IncorrectTaxCode
                labels.push(self.inject_incorrect_tax_code(line));
            } else if roll < 0.70 {
                // MissingTaxLine
                labels.push(self.create_missing_tax_line_label(line));
                indices_to_remove.push(i);
            } else if roll < 0.85 {
                // RateArbitrage
                labels.push(self.inject_rate_arbitrage(line));
            } else {
                // WithholdingUnderstatement (on the tax line: reduce tax_amount)
                labels.push(self.inject_tax_line_understatement(line));
            }
        }

        // Remove lines marked for deletion (reverse order to maintain indices)
        for &i in indices_to_remove.iter().rev() {
            lines.remove(i);
        }

        labels
    }

    /// Inject anomalies into tax returns.
    ///
    /// - **LateFilingRisk**: Set `actual_filing_date` close to or past the deadline.
    pub fn inject_into_returns(&mut self, returns: &mut [TaxReturn]) -> Vec<TaxAnomalyLabel> {
        let mut labels = Vec::new();

        for ret in returns.iter_mut() {
            if !self.should_inject() {
                continue;
            }
            labels.push(self.inject_late_filing(ret));
        }

        labels
    }

    /// Inject anomalies into withholding records.
    ///
    /// - **WithholdingUnderstatement**: Reduce `applied_rate` without treaty justification.
    pub fn inject_into_withholding(
        &mut self,
        records: &mut [WithholdingTaxRecord],
    ) -> Vec<TaxAnomalyLabel> {
        let mut labels = Vec::new();

        for record in records.iter_mut() {
            if !self.should_inject() {
                continue;
            }
            labels.push(self.inject_withholding_understatement(record));
        }

        labels
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Returns `true` with probability `anomaly_rate`.
    fn should_inject(&mut self) -> bool {
        self.rng.gen::<f64>() < self.anomaly_rate
    }

    /// Returns the next sequential anomaly label ID.
    fn next_id(&mut self) -> String {
        self.counter += 1;
        format!("TXANO-{:06}", self.counter)
    }

    /// Determines severity based on the monetary impact ratio.
    fn severity_from_impact(impact_ratio: Decimal) -> TaxAnomalySeverity {
        if impact_ratio >= dec!(0.50) {
            TaxAnomalySeverity::Critical
        } else if impact_ratio >= dec!(0.25) {
            TaxAnomalySeverity::High
        } else if impact_ratio >= dec!(0.10) {
            TaxAnomalySeverity::Medium
        } else {
            TaxAnomalySeverity::Low
        }
    }

    /// Inject IncorrectTaxCode: apply a wrong rate to the tax_amount.
    fn inject_incorrect_tax_code(&mut self, line: &mut TaxLine) -> TaxAnomalyLabel {
        let original_amount = line.tax_amount;
        let original_rate = line.effective_rate();

        // Choose a wrong rate that differs from the current effective rate.
        // Candidates are common worldwide rates; pick one at random that
        // differs from the current rate.
        let wrong_rates = [
            dec!(0.05),
            dec!(0.07),
            dec!(0.10),
            dec!(0.13),
            dec!(0.15),
            dec!(0.21),
            dec!(0.23),
            dec!(0.25),
        ];

        let idx = self.rng.gen_range(0..wrong_rates.len());
        let mut wrong_rate = wrong_rates[idx];
        // If we accidentally picked the same rate, shift by one index.
        if wrong_rate == original_rate.round_dp(2) {
            wrong_rate = wrong_rates[(idx + 1) % wrong_rates.len()];
        }

        let new_amount = (line.taxable_amount * wrong_rate).round_dp(2);
        line.tax_amount = new_amount;

        let impact = if original_amount.is_zero() {
            dec!(1.0)
        } else {
            ((new_amount - original_amount).abs() / original_amount.abs()).round_dp(4)
        };

        TaxAnomalyLabel {
            id: self.next_id(),
            anomaly_type: TaxAnomalyType::IncorrectTaxCode,
            severity: Self::severity_from_impact(impact),
            document_type: "tax_line".to_string(),
            document_id: line.id.clone(),
            description: format!(
                "Incorrect tax code applied: effective rate changed from {} to {} on tax line {}",
                original_rate, wrong_rate, line.id
            ),
            original_value: Some(original_amount.to_string()),
            anomalous_value: Some(new_amount.to_string()),
        }
    }

    /// Create a label for a missing tax line (line will be removed from the vec
    /// by the caller).
    fn create_missing_tax_line_label(&mut self, line: &TaxLine) -> TaxAnomalyLabel {
        TaxAnomalyLabel {
            id: self.next_id(),
            anomaly_type: TaxAnomalyType::MissingTaxLine,
            severity: TaxAnomalySeverity::High,
            document_type: "tax_line".to_string(),
            document_id: line.id.clone(),
            description: format!(
                "Tax line {} removed from document {}: taxable amount {} has no tax applied",
                line.id, line.document_id, line.taxable_amount
            ),
            original_value: Some(line.tax_amount.to_string()),
            anomalous_value: None,
        }
    }

    /// Inject RateArbitrage: re-route the line to a low-tax jurisdiction.
    fn inject_rate_arbitrage(&mut self, line: &mut TaxLine) -> TaxAnomalyLabel {
        let original_jurisdiction = line.jurisdiction_id.clone();

        let idx = self.rng.gen_range(0..LOW_TAX_JURISDICTIONS.len());
        let new_jurisdiction = LOW_TAX_JURISDICTIONS[idx].to_string();

        line.jurisdiction_id = new_jurisdiction.clone();

        // Also reduce the tax amount to reflect the low-tax jurisdiction
        let reduction_factor = dec!(0.25) + dec!(0.25) * Decimal::from(self.rng.gen_range(0u32..4));
        let original_amount = line.tax_amount;
        line.tax_amount = (line.tax_amount * reduction_factor).round_dp(2);

        TaxAnomalyLabel {
            id: self.next_id(),
            anomaly_type: TaxAnomalyType::RateArbitrage,
            severity: TaxAnomalySeverity::Critical,
            document_type: "tax_line".to_string(),
            document_id: line.id.clone(),
            description: format!(
                "Rate arbitrage: jurisdiction changed from {} to {} on tax line {}",
                original_jurisdiction, new_jurisdiction, line.id
            ),
            original_value: Some(format!(
                "jurisdiction={}, tax_amount={}",
                original_jurisdiction, original_amount
            )),
            anomalous_value: Some(format!(
                "jurisdiction={}, tax_amount={}",
                new_jurisdiction, line.tax_amount
            )),
        }
    }

    /// Inject a tax-line-level understatement (reduces tax_amount below correct amount).
    fn inject_tax_line_understatement(&mut self, line: &mut TaxLine) -> TaxAnomalyLabel {
        let original_amount = line.tax_amount;

        // Reduce the tax amount by 30-70%
        let reduction: f64 = 0.30 + self.rng.gen::<f64>() * 0.40;
        let reduction_dec = Decimal::from_f64_retain(reduction).unwrap_or(dec!(0.50));
        let new_amount = (line.tax_amount * (Decimal::ONE - reduction_dec)).round_dp(2);
        line.tax_amount = new_amount;

        let impact = if original_amount.is_zero() {
            dec!(0.50)
        } else {
            ((original_amount - new_amount) / original_amount).round_dp(4)
        };

        TaxAnomalyLabel {
            id: self.next_id(),
            anomaly_type: TaxAnomalyType::WithholdingUnderstatement,
            severity: Self::severity_from_impact(impact),
            document_type: "tax_line".to_string(),
            document_id: line.id.clone(),
            description: format!(
                "Tax understatement on line {}: tax reduced from {} to {} ({:.0}% reduction)",
                line.id,
                original_amount,
                new_amount,
                reduction * 100.0
            ),
            original_value: Some(original_amount.to_string()),
            anomalous_value: Some(new_amount.to_string()),
        }
    }

    /// Inject LateFilingRisk: set actual_filing_date close to or past the deadline.
    fn inject_late_filing(&mut self, ret: &mut TaxReturn) -> TaxAnomalyLabel {
        let deadline = ret.filing_deadline;

        // Decide how late: -2 days to +30 days from deadline
        let days_offset: i64 = self.rng.gen_range(-2..=30);
        let filing_date = deadline + Duration::days(days_offset);

        ret.actual_filing_date = Some(filing_date);
        ret.is_late = filing_date > deadline;

        let severity = if days_offset > 14 {
            TaxAnomalySeverity::Critical
        } else if days_offset > 5 {
            TaxAnomalySeverity::High
        } else if days_offset > 0 {
            TaxAnomalySeverity::Medium
        } else {
            TaxAnomalySeverity::Low
        };

        TaxAnomalyLabel {
            id: self.next_id(),
            anomaly_type: TaxAnomalyType::LateFilingRisk,
            severity,
            document_type: "tax_return".to_string(),
            document_id: ret.id.clone(),
            description: format!(
                "Late filing risk for return {}: deadline={}, actual_filing_date={}, {} days {}",
                ret.id,
                deadline,
                filing_date,
                days_offset.unsigned_abs(),
                if days_offset > 0 {
                    "past deadline"
                } else {
                    "before deadline"
                }
            ),
            original_value: Some(deadline.to_string()),
            anomalous_value: Some(filing_date.to_string()),
        }
    }

    /// Inject WithholdingUnderstatement: reduce applied_rate below statutory_rate
    /// without treaty justification.
    fn inject_withholding_understatement(
        &mut self,
        record: &mut WithholdingTaxRecord,
    ) -> TaxAnomalyLabel {
        let original_rate = record.applied_rate;
        let statutory = record.statutory_rate;

        // Set applied_rate to 30-70% of statutory rate, without a treaty justification
        let fraction: f64 = 0.30 + self.rng.gen::<f64>() * 0.40;
        let fraction_dec = Decimal::from_f64_retain(fraction).unwrap_or(dec!(0.50));
        let new_rate = (statutory * fraction_dec).round_dp(4);

        record.applied_rate = new_rate;
        record.treaty_rate = None; // Remove treaty justification
        record.withheld_amount = (record.base_amount * new_rate).round_dp(2);
        record.certificate_number = None; // Remove certificate

        let impact = if statutory.is_zero() {
            dec!(0.50)
        } else {
            ((statutory - new_rate) / statutory).round_dp(4)
        };

        TaxAnomalyLabel {
            id: self.next_id(),
            anomaly_type: TaxAnomalyType::WithholdingUnderstatement,
            severity: Self::severity_from_impact(impact),
            document_type: "withholding_record".to_string(),
            document_id: record.id.clone(),
            description: format!(
                "Withholding understatement on {}: applied_rate reduced from {} to {} \
                 (statutory_rate={}) without treaty justification",
                record.id, original_rate, new_rate, statutory
            ),
            original_value: Some(original_rate.to_string()),
            anomalous_value: Some(new_rate.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::{
        TaxReturnType, TaxableDocumentType, WithholdingType,
    };

    /// Helper: create a test tax line.
    fn make_tax_line(id: &str, taxable: Decimal, tax: Decimal) -> TaxLine {
        TaxLine::new(
            id,
            TaxableDocumentType::VendorInvoice,
            "DOC-001",
            1,
            "TC-VAT-20",
            "JUR-DE",
            taxable,
            tax,
        )
    }

    /// Helper: create a test tax return.
    fn make_tax_return(id: &str) -> TaxReturn {
        TaxReturn::new(
            id,
            "ENT-001",
            "JUR-DE",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            TaxReturnType::VatReturn,
            dec!(50000),
            dec!(30000),
            NaiveDate::from_ymd_opt(2024, 4, 30).unwrap(),
        )
    }

    /// Helper: create a test withholding record.
    fn make_withholding(id: &str) -> WithholdingTaxRecord {
        WithholdingTaxRecord::new(
            id,
            "PAY-001",
            "V-100",
            WithholdingType::ServiceWithholding,
            dec!(0.30),
            dec!(0.30),
            dec!(100000),
        )
    }

    // -----------------------------------------------------------------------
    // Required tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_inject_tax_line_anomalies() {
        // At 100% rate, every line should get an anomaly.
        let mut injector = TaxAnomalyInjector::new(42, 1.0);
        let mut lines: Vec<TaxLine> = (0..10)
            .map(|i| make_tax_line(&format!("TL-{:03}", i), dec!(10000), dec!(2000)))
            .collect();

        let labels = injector.inject_into_tax_lines(&mut lines);

        // All 10 original lines should produce labels (some may be removed)
        assert_eq!(labels.len(), 10, "Expected 10 labels at 100% rate");

        // Some lines may have been removed (MissingTaxLine)
        let missing_count = labels
            .iter()
            .filter(|l| l.anomaly_type == TaxAnomalyType::MissingTaxLine)
            .count();
        assert_eq!(
            lines.len(),
            10 - missing_count,
            "Remaining lines should be 10 minus missing count"
        );
    }

    #[test]
    fn test_anomaly_rate_respected() {
        // At 10% rate on 1000 lines, expect roughly 100 anomalies.
        let mut injector = TaxAnomalyInjector::new(123, 0.10);
        let mut lines: Vec<TaxLine> = (0..1000)
            .map(|i| make_tax_line(&format!("TL-{:04}", i), dec!(5000), dec!(1000)))
            .collect();

        let labels = injector.inject_into_tax_lines(&mut lines);

        // Allow wide range for randomness: 50 to 200 out of 1000
        assert!(
            labels.len() >= 50 && labels.len() <= 200,
            "Expected ~100 anomalies at 10% rate, got {}",
            labels.len()
        );
    }

    #[test]
    fn test_incorrect_tax_code_anomaly() {
        // Force IncorrectTaxCode by using a seed that produces a roll < 0.40.
        // We'll use 100% rate and check that at least one IncorrectTaxCode exists
        // across many lines.
        let mut injector = TaxAnomalyInjector::new(42, 1.0);
        let mut lines: Vec<TaxLine> = (0..20)
            .map(|i| make_tax_line(&format!("TL-{:03}", i), dec!(10000), dec!(2000)))
            .collect();

        let original_amounts: Vec<Decimal> = lines.iter().map(|l| l.tax_amount).collect();
        let labels = injector.inject_into_tax_lines(&mut lines);

        let incorrect_labels: Vec<_> = labels
            .iter()
            .filter(|l| l.anomaly_type == TaxAnomalyType::IncorrectTaxCode)
            .collect();

        assert!(
            !incorrect_labels.is_empty(),
            "Expected at least one IncorrectTaxCode anomaly"
        );

        // For IncorrectTaxCode labels, verify that the original and anomalous values differ
        for label in &incorrect_labels {
            assert_ne!(
                label.original_value, label.anomalous_value,
                "Incorrect tax code should change the tax amount"
            );
        }

        // Verify that at least one remaining line had its tax_amount changed
        // (lines not removed are present; check against originals)
        let remaining_ids: Vec<&str> = lines.iter().map(|l| l.id.as_str()).collect();
        let mut found_changed = false;
        for (i, orig_amount) in original_amounts.iter().enumerate() {
            let id = format!("TL-{:03}", i);
            if let Some(pos) = remaining_ids.iter().position(|&lid| lid == id) {
                if lines[pos].tax_amount != *orig_amount {
                    found_changed = true;
                    break;
                }
            }
        }
        assert!(found_changed, "At least one tax_amount should be changed");
    }

    #[test]
    fn test_late_filing_anomaly() {
        let mut injector = TaxAnomalyInjector::new(42, 1.0);
        let mut returns: Vec<TaxReturn> = (0..10)
            .map(|i| make_tax_return(&format!("TR-{:03}", i)))
            .collect();

        let labels = injector.inject_into_returns(&mut returns);

        assert_eq!(labels.len(), 10, "All returns should get anomalies at 100%");

        for (label, ret) in labels.iter().zip(returns.iter()) {
            assert_eq!(label.anomaly_type, TaxAnomalyType::LateFilingRisk);
            assert!(
                ret.actual_filing_date.is_some(),
                "Filing date should be set"
            );

            let filing_date = ret.actual_filing_date.unwrap();
            let deadline = ret.filing_deadline;

            // Filing date should be within -2 to +30 days of deadline
            let diff = (filing_date - deadline).num_days();
            assert!(
                (-2..=30).contains(&diff),
                "Filing date offset should be -2 to +30 days, got {}",
                diff
            );

            // Verify is_late consistency
            assert_eq!(
                ret.is_late,
                filing_date > deadline,
                "is_late flag should match actual vs deadline comparison"
            );
        }
    }

    #[test]
    fn test_withholding_understatement() {
        let mut injector = TaxAnomalyInjector::new(42, 1.0);
        let mut records: Vec<WithholdingTaxRecord> = (0..10)
            .map(|i| make_withholding(&format!("WHT-{:03}", i)))
            .collect();

        let labels = injector.inject_into_withholding(&mut records);

        assert_eq!(labels.len(), 10, "All records should get anomalies at 100%");

        for (label, record) in labels.iter().zip(records.iter()) {
            assert_eq!(
                label.anomaly_type,
                TaxAnomalyType::WithholdingUnderstatement
            );

            // applied_rate should be strictly less than statutory_rate
            assert!(
                record.applied_rate < record.statutory_rate,
                "applied_rate ({}) should be less than statutory_rate ({})",
                record.applied_rate,
                record.statutory_rate
            );

            // Treaty justification should be removed
            assert!(
                record.treaty_rate.is_none(),
                "Treaty rate should be removed for unjustified understatement"
            );

            // withheld_amount should be recalculated
            let expected_withheld = (record.base_amount * record.applied_rate).round_dp(2);
            assert_eq!(
                record.withheld_amount, expected_withheld,
                "withheld_amount should match base_amount * applied_rate"
            );
        }
    }

    #[test]
    fn test_labels_have_descriptions() {
        let mut injector = TaxAnomalyInjector::new(42, 1.0);

        // Inject into tax lines
        let mut lines: Vec<TaxLine> = (0..5)
            .map(|i| make_tax_line(&format!("TL-{:03}", i), dec!(10000), dec!(2000)))
            .collect();
        let line_labels = injector.inject_into_tax_lines(&mut lines);

        // Inject into returns
        let mut returns = vec![make_tax_return("TR-001")];
        let return_labels = injector.inject_into_returns(&mut returns);

        // Inject into withholding
        let mut records = vec![make_withholding("WHT-001")];
        let wht_labels = injector.inject_into_withholding(&mut records);

        let all_labels: Vec<&TaxAnomalyLabel> = line_labels
            .iter()
            .chain(return_labels.iter())
            .chain(wht_labels.iter())
            .collect();

        assert!(
            !all_labels.is_empty(),
            "Should have at least some labels to test"
        );

        for label in &all_labels {
            assert!(
                !label.description.is_empty(),
                "Label {} should have a non-empty description",
                label.id
            );
            assert!(
                !label.id.is_empty(),
                "Label should have a non-empty ID"
            );
            assert!(
                !label.document_type.is_empty(),
                "Label {} should have a non-empty document_type",
                label.id
            );
            assert!(
                !label.document_id.is_empty(),
                "Label {} should have a non-empty document_id",
                label.id
            );
        }
    }

    #[test]
    fn test_deterministic() {
        // Two injectors with the same seed should produce identical results.
        let mut injector1 = TaxAnomalyInjector::new(999, 0.5);
        let mut injector2 = TaxAnomalyInjector::new(999, 0.5);

        let mut lines1: Vec<TaxLine> = (0..50)
            .map(|i| make_tax_line(&format!("TL-{:03}", i), dec!(10000), dec!(2000)))
            .collect();
        let mut lines2: Vec<TaxLine> = (0..50)
            .map(|i| make_tax_line(&format!("TL-{:03}", i), dec!(10000), dec!(2000)))
            .collect();

        let labels1 = injector1.inject_into_tax_lines(&mut lines1);
        let labels2 = injector2.inject_into_tax_lines(&mut lines2);

        assert_eq!(labels1.len(), labels2.len(), "Label counts should match");
        assert_eq!(lines1.len(), lines2.len(), "Remaining line counts should match");

        for (l1, l2) in labels1.iter().zip(labels2.iter()) {
            assert_eq!(l1.id, l2.id, "Label IDs should match");
            assert_eq!(l1.anomaly_type, l2.anomaly_type, "Anomaly types should match");
            assert_eq!(l1.severity, l2.severity, "Severities should match");
            assert_eq!(l1.document_id, l2.document_id, "Document IDs should match");
            assert_eq!(l1.original_value, l2.original_value, "Original values should match");
            assert_eq!(l1.anomalous_value, l2.anomalous_value, "Anomalous values should match");
        }

        for (ln1, ln2) in lines1.iter().zip(lines2.iter()) {
            assert_eq!(ln1.id, ln2.id);
            assert_eq!(ln1.tax_amount, ln2.tax_amount);
            assert_eq!(ln1.jurisdiction_id, ln2.jurisdiction_id);
        }
    }

    #[test]
    fn test_zero_rate_no_anomalies() {
        let mut injector = TaxAnomalyInjector::new(42, 0.0);
        let mut lines: Vec<TaxLine> = (0..100)
            .map(|i| make_tax_line(&format!("TL-{:03}", i), dec!(10000), dec!(2000)))
            .collect();

        let labels = injector.inject_into_tax_lines(&mut lines);

        assert!(labels.is_empty(), "Zero rate should produce no anomalies");
        assert_eq!(lines.len(), 100, "No lines should be removed");
    }

    #[test]
    fn test_label_ids_are_sequential() {
        let mut injector = TaxAnomalyInjector::new(42, 1.0);

        let mut lines: Vec<TaxLine> = (0..5)
            .map(|i| make_tax_line(&format!("TL-{:03}", i), dec!(10000), dec!(2000)))
            .collect();
        let labels = injector.inject_into_tax_lines(&mut lines);

        for (i, label) in labels.iter().enumerate() {
            let expected_id = format!("TXANO-{:06}", i + 1);
            assert_eq!(label.id, expected_id, "Labels should have sequential IDs");
        }
    }

    #[test]
    fn test_serde_roundtrip() {
        let label = TaxAnomalyLabel {
            id: "TXANO-000001".to_string(),
            anomaly_type: TaxAnomalyType::IncorrectTaxCode,
            severity: TaxAnomalySeverity::High,
            document_type: "tax_line".to_string(),
            document_id: "TL-001".to_string(),
            description: "Test anomaly".to_string(),
            original_value: Some("2000".to_string()),
            anomalous_value: Some("1500".to_string()),
        };

        let json = serde_json::to_string_pretty(&label).unwrap();
        let deserialized: TaxAnomalyLabel = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, label.id);
        assert_eq!(deserialized.anomaly_type, label.anomaly_type);
        assert_eq!(deserialized.severity, label.severity);
        assert_eq!(deserialized.document_type, label.document_type);
        assert_eq!(deserialized.document_id, label.document_id);
        assert_eq!(deserialized.original_value, label.original_value);
        assert_eq!(deserialized.anomalous_value, label.anomalous_value);
    }
}
