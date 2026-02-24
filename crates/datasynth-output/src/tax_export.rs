//! Export tax accounting data to CSV files.
//!
//! Exports tax jurisdictions, codes, lines, returns, provisions,
//! uncertain tax positions, withholding records, and anomaly labels
//! as separate CSV files for use in BI/analytics/ML systems.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use datasynth_core::error::SynthResult;
use datasynth_core::models::{
    TaxCode, TaxJurisdiction, TaxLine, TaxProvision, TaxReturn, UncertainTaxPosition,
    WithholdingTaxRecord,
};

// ---------------------------------------------------------------------------
// Anomaly label row (string-based for export; the typed version lives in
// datasynth-generators::tax::tax_anomaly)
// ---------------------------------------------------------------------------

/// A pre-serialized tax anomaly label row for CSV export.
///
/// The typed `TaxAnomalyLabel` (with enum fields) lives in
/// `datasynth-generators`. This struct accepts string representations
/// so the output crate doesn't need a dependency on generators.
#[derive(Debug, Clone)]
pub struct TaxAnomalyLabelRow {
    pub id: String,
    pub anomaly_type: String,
    pub severity: String,
    pub document_type: String,
    pub document_id: String,
    pub description: String,
    pub original_value: String,
    pub anomalous_value: String,
}

// ---------------------------------------------------------------------------
// Export summary
// ---------------------------------------------------------------------------

/// Summary of exported tax data.
#[derive(Debug, Default)]
pub struct TaxExportSummary {
    pub jurisdictions_count: usize,
    pub tax_codes_count: usize,
    pub tax_lines_count: usize,
    pub tax_returns_count: usize,
    pub tax_provisions_count: usize,
    pub rate_reconciliation_count: usize,
    pub uncertain_positions_count: usize,
    pub withholding_records_count: usize,
    pub anomaly_labels_count: usize,
}

impl TaxExportSummary {
    /// Total number of rows exported across all files.
    pub fn total(&self) -> usize {
        self.jurisdictions_count
            + self.tax_codes_count
            + self.tax_lines_count
            + self.tax_returns_count
            + self.tax_provisions_count
            + self.rate_reconciliation_count
            + self.uncertain_positions_count
            + self.withholding_records_count
            + self.anomaly_labels_count
    }
}

// ---------------------------------------------------------------------------
// Exporter
// ---------------------------------------------------------------------------

/// Exporter for tax accounting data.
pub struct TaxExporter {
    output_dir: PathBuf,
}

impl TaxExporter {
    /// Create a new tax exporter writing to the given directory.
    pub fn new(output_dir: impl AsRef<Path>) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    /// Export all tax data.
    ///
    /// Creates the following CSV files:
    /// - `tax_jurisdictions.csv`
    /// - `tax_codes.csv`
    /// - `tax_lines.csv`
    /// - `tax_returns.csv`
    /// - `tax_provisions.csv`
    /// - `rate_reconciliation.csv`
    /// - `uncertain_tax_positions.csv`
    /// - `withholding_records.csv`
    /// - `tax_anomaly_labels.csv`
    #[allow(clippy::too_many_arguments)]
    pub fn export_all(
        &self,
        jurisdictions: &[TaxJurisdiction],
        tax_codes: &[TaxCode],
        tax_lines: &[TaxLine],
        tax_returns: &[TaxReturn],
        provisions: &[TaxProvision],
        uncertain_positions: &[UncertainTaxPosition],
        withholding_records: &[WithholdingTaxRecord],
        anomaly_labels: &[TaxAnomalyLabelRow],
    ) -> SynthResult<TaxExportSummary> {
        std::fs::create_dir_all(&self.output_dir)?;

        let summary = TaxExportSummary {
            jurisdictions_count: self.export_jurisdictions(jurisdictions)?,
            tax_codes_count: self.export_tax_codes(tax_codes)?,
            tax_lines_count: self.export_tax_lines(tax_lines)?,
            tax_returns_count: self.export_tax_returns(tax_returns)?,
            tax_provisions_count: self.export_tax_provisions(provisions)?,
            rate_reconciliation_count: self.export_rate_reconciliation(provisions)?,
            uncertain_positions_count: self.export_uncertain_positions(uncertain_positions)?,
            withholding_records_count: self.export_withholding_records(withholding_records)?,
            anomaly_labels_count: self.export_anomaly_labels(anomaly_labels)?,
        };

        Ok(summary)
    }

    /// Export tax jurisdictions to `tax_jurisdictions.csv`.
    pub fn export_jurisdictions(&self, data: &[TaxJurisdiction]) -> SynthResult<usize> {
        let path = self.output_dir.join("tax_jurisdictions.csv");
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            writer,
            "id,name,country_code,region_code,jurisdiction_type,parent_jurisdiction_id,vat_registered"
        )?;

        for j in data {
            writeln!(
                writer,
                "{},{},{},{},{:?},{},{}",
                escape_csv(&j.id),
                escape_csv(&j.name),
                escape_csv(&j.country_code),
                j.region_code.as_deref().unwrap_or(""),
                j.jurisdiction_type,
                j.parent_jurisdiction_id.as_deref().unwrap_or(""),
                j.vat_registered,
            )?;
        }

        writer.flush()?;
        Ok(data.len())
    }

    /// Export tax codes to `tax_codes.csv`.
    pub fn export_tax_codes(&self, data: &[TaxCode]) -> SynthResult<usize> {
        let path = self.output_dir.join("tax_codes.csv");
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            writer,
            "id,code,description,tax_type,rate,jurisdiction_id,effective_date,expiry_date,is_reverse_charge,is_exempt"
        )?;

        for tc in data {
            writeln!(
                writer,
                "{},{},{},{:?},{},{},{},{},{},{}",
                escape_csv(&tc.id),
                escape_csv(&tc.code),
                escape_csv(&tc.description),
                tc.tax_type,
                tc.rate,
                escape_csv(&tc.jurisdiction_id),
                tc.effective_date,
                tc.expiry_date.map(|d| d.to_string()).unwrap_or_default(),
                tc.is_reverse_charge,
                tc.is_exempt,
            )?;
        }

        writer.flush()?;
        Ok(data.len())
    }

    /// Export tax lines to `tax_lines.csv`.
    pub fn export_tax_lines(&self, data: &[TaxLine]) -> SynthResult<usize> {
        let path = self.output_dir.join("tax_lines.csv");
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            writer,
            "id,document_type,document_id,line_number,tax_code_id,jurisdiction_id,\
             taxable_amount,tax_amount,is_deductible,is_reverse_charge,is_self_assessed"
        )?;

        for tl in data {
            writeln!(
                writer,
                "{},{:?},{},{},{},{},{},{},{},{},{}",
                escape_csv(&tl.id),
                tl.document_type,
                escape_csv(&tl.document_id),
                tl.line_number,
                escape_csv(&tl.tax_code_id),
                escape_csv(&tl.jurisdiction_id),
                tl.taxable_amount,
                tl.tax_amount,
                tl.is_deductible,
                tl.is_reverse_charge,
                tl.is_self_assessed,
            )?;
        }

        writer.flush()?;
        Ok(data.len())
    }

    /// Export tax returns to `tax_returns.csv`.
    pub fn export_tax_returns(&self, data: &[TaxReturn]) -> SynthResult<usize> {
        let path = self.output_dir.join("tax_returns.csv");
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            writer,
            "id,entity_id,jurisdiction_id,period_start,period_end,return_type,status,\
             total_output_tax,total_input_tax,net_payable,filing_deadline,actual_filing_date,is_late"
        )?;

        for tr in data {
            writeln!(
                writer,
                "{},{},{},{},{},{:?},{:?},{},{},{},{},{},{}",
                escape_csv(&tr.id),
                escape_csv(&tr.entity_id),
                escape_csv(&tr.jurisdiction_id),
                tr.period_start,
                tr.period_end,
                tr.return_type,
                tr.status,
                tr.total_output_tax,
                tr.total_input_tax,
                tr.net_payable,
                tr.filing_deadline,
                tr.actual_filing_date
                    .map(|d| d.to_string())
                    .unwrap_or_default(),
                tr.is_late,
            )?;
        }

        writer.flush()?;
        Ok(data.len())
    }

    /// Export tax provisions to `tax_provisions.csv`.
    pub fn export_tax_provisions(&self, data: &[TaxProvision]) -> SynthResult<usize> {
        let path = self.output_dir.join("tax_provisions.csv");
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            writer,
            "id,entity_id,period,current_tax_expense,deferred_tax_asset,\
             deferred_tax_liability,statutory_rate,effective_rate"
        )?;

        for tp in data {
            writeln!(
                writer,
                "{},{},{},{},{},{},{},{}",
                escape_csv(&tp.id),
                escape_csv(&tp.entity_id),
                tp.period,
                tp.current_tax_expense,
                tp.deferred_tax_asset,
                tp.deferred_tax_liability,
                tp.statutory_rate,
                tp.effective_rate,
            )?;
        }

        writer.flush()?;
        Ok(data.len())
    }

    /// Export rate reconciliation items to `rate_reconciliation.csv`.
    ///
    /// Flattens the reconciliation items from each provision into individual
    /// rows, using the provision id as a foreign key.
    pub fn export_rate_reconciliation(&self, provisions: &[TaxProvision]) -> SynthResult<usize> {
        let path = self.output_dir.join("rate_reconciliation.csv");
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        writeln!(writer, "provision_id,description,rate_impact")?;

        let mut row_count = 0;
        for tp in provisions {
            for item in &tp.rate_reconciliation {
                writeln!(
                    writer,
                    "{},{},{}",
                    escape_csv(&tp.id),
                    escape_csv(&item.description),
                    item.rate_impact,
                )?;
                row_count += 1;
            }
        }

        writer.flush()?;
        Ok(row_count)
    }

    /// Export uncertain tax positions to `uncertain_tax_positions.csv`.
    pub fn export_uncertain_positions(&self, data: &[UncertainTaxPosition]) -> SynthResult<usize> {
        let path = self.output_dir.join("uncertain_tax_positions.csv");
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            writer,
            "id,entity_id,description,tax_benefit,recognition_threshold,\
             recognized_amount,measurement_method"
        )?;

        for utp in data {
            writeln!(
                writer,
                "{},{},{},{},{},{},{:?}",
                escape_csv(&utp.id),
                escape_csv(&utp.entity_id),
                escape_csv(&utp.description),
                utp.tax_benefit,
                utp.recognition_threshold,
                utp.recognized_amount,
                utp.measurement_method,
            )?;
        }

        writer.flush()?;
        Ok(data.len())
    }

    /// Export withholding tax records to `withholding_records.csv`.
    pub fn export_withholding_records(&self, data: &[WithholdingTaxRecord]) -> SynthResult<usize> {
        let path = self.output_dir.join("withholding_records.csv");
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            writer,
            "id,payment_id,vendor_id,withholding_type,treaty_rate,statutory_rate,\
             applied_rate,base_amount,withheld_amount,certificate_number"
        )?;

        for wht in data {
            writeln!(
                writer,
                "{},{},{},{:?},{},{},{},{},{},{}",
                escape_csv(&wht.id),
                escape_csv(&wht.payment_id),
                escape_csv(&wht.vendor_id),
                wht.withholding_type,
                wht.treaty_rate.map(|r| r.to_string()).unwrap_or_default(),
                wht.statutory_rate,
                wht.applied_rate,
                wht.base_amount,
                wht.withheld_amount,
                wht.certificate_number.as_deref().unwrap_or(""),
            )?;
        }

        writer.flush()?;
        Ok(data.len())
    }

    /// Export tax anomaly labels to `tax_anomaly_labels.csv`.
    pub fn export_anomaly_labels(&self, data: &[TaxAnomalyLabelRow]) -> SynthResult<usize> {
        let path = self.output_dir.join("tax_anomaly_labels.csv");
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            writer,
            "id,anomaly_type,severity,document_type,document_id,description,\
             original_value,anomalous_value"
        )?;

        for label in data {
            writeln!(
                writer,
                "{},{},{},{},{},{},{},{}",
                escape_csv(&label.id),
                escape_csv(&label.anomaly_type),
                escape_csv(&label.severity),
                escape_csv(&label.document_type),
                escape_csv(&label.document_id),
                escape_csv(&label.description),
                escape_csv(&label.original_value),
                escape_csv(&label.anomalous_value),
            )?;
        }

        writer.flush()?;
        Ok(data.len())
    }
}

/// Escape a string for CSV output.
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::{
        JurisdictionType, TaxMeasurementMethod, TaxReturnType, TaxType, TaxableDocumentType,
        WithholdingType,
    };
    use rust_decimal_macros::dec;
    use tempfile::TempDir;

    // -----------------------------------------------------------------------
    // Test-data helpers
    // -----------------------------------------------------------------------

    fn sample_jurisdictions() -> Vec<TaxJurisdiction> {
        vec![
            TaxJurisdiction::new(
                "JUR-US",
                "United States - Federal",
                "US",
                JurisdictionType::Federal,
            )
            .with_vat_registered(false),
            TaxJurisdiction::new("JUR-US-CA", "California", "US", JurisdictionType::State)
                .with_region_code("CA")
                .with_parent_jurisdiction_id("JUR-US"),
        ]
    }

    fn sample_tax_codes() -> Vec<TaxCode> {
        vec![TaxCode::new(
            "TC-001",
            "VAT-STD-20",
            "Standard VAT 20%",
            TaxType::Vat,
            dec!(0.20),
            "JUR-UK",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )
        .with_expiry_date(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap())
        .with_reverse_charge(true)]
    }

    fn sample_tax_lines() -> Vec<TaxLine> {
        vec![TaxLine::new(
            "TL-001",
            TaxableDocumentType::VendorInvoice,
            "INV-001",
            1,
            "TC-001",
            "JUR-UK",
            dec!(1000.00),
            dec!(200.00),
        )
        .with_deductible(true)
        .with_reverse_charge(false)
        .with_self_assessed(false)]
    }

    fn sample_tax_returns() -> Vec<TaxReturn> {
        vec![TaxReturn::new(
            "TR-001",
            "ENT-001",
            "JUR-UK",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            TaxReturnType::VatReturn,
            dec!(50000),
            dec!(30000),
            NaiveDate::from_ymd_opt(2024, 4, 30).unwrap(),
        )
        .with_filing(NaiveDate::from_ymd_opt(2024, 4, 15).unwrap())]
    }

    fn sample_provisions() -> Vec<TaxProvision> {
        vec![TaxProvision::new(
            "TP-001",
            "ENT-001",
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(250000),
            dec!(80000),
            dec!(120000),
            dec!(0.21),
            dec!(0.245),
        )
        .with_reconciliation_item("State taxes", dec!(0.03))
        .with_reconciliation_item("R&D credits", dec!(-0.015))]
    }

    fn sample_uncertain_positions() -> Vec<UncertainTaxPosition> {
        vec![UncertainTaxPosition::new(
            "UTP-001",
            "ENT-001",
            "R&D credit claim for software development",
            dec!(500000),
            dec!(0.50),
            dec!(350000),
            TaxMeasurementMethod::MostLikelyAmount,
        )]
    }

    fn sample_withholding_records() -> Vec<WithholdingTaxRecord> {
        vec![WithholdingTaxRecord::new(
            "WHT-001",
            "PAY-001",
            "V-100",
            WithholdingType::RoyaltyWithholding,
            dec!(0.30),
            dec!(0.10),
            dec!(100000),
        )
        .with_treaty_rate(dec!(0.10))
        .with_certificate_number("CERT-2024-001")]
    }

    fn sample_anomaly_labels() -> Vec<TaxAnomalyLabelRow> {
        vec![TaxAnomalyLabelRow {
            id: "TAL-001".to_string(),
            anomaly_type: "incorrect_tax_code".to_string(),
            severity: "high".to_string(),
            document_type: "tax_line".to_string(),
            document_id: "TL-001".to_string(),
            description: "Applied rate does not match tax code rate".to_string(),
            original_value: "0.20".to_string(),
            anomalous_value: "0.25".to_string(),
        }]
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_export_creates_all_files() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = TaxExporter::new(temp_dir.path());

        exporter
            .export_all(
                &sample_jurisdictions(),
                &sample_tax_codes(),
                &sample_tax_lines(),
                &sample_tax_returns(),
                &sample_provisions(),
                &sample_uncertain_positions(),
                &sample_withholding_records(),
                &sample_anomaly_labels(),
            )
            .unwrap();

        let expected_files = [
            "tax_jurisdictions.csv",
            "tax_codes.csv",
            "tax_lines.csv",
            "tax_returns.csv",
            "tax_provisions.csv",
            "rate_reconciliation.csv",
            "uncertain_tax_positions.csv",
            "withholding_records.csv",
            "tax_anomaly_labels.csv",
        ];

        for file_name in &expected_files {
            assert!(
                temp_dir.path().join(file_name).exists(),
                "Expected file '{}' to exist",
                file_name,
            );
        }
    }

    #[test]
    fn test_export_csv_headers() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = TaxExporter::new(temp_dir.path());

        exporter
            .export_all(
                &sample_jurisdictions(),
                &sample_tax_codes(),
                &sample_tax_lines(),
                &sample_tax_returns(),
                &sample_provisions(),
                &sample_uncertain_positions(),
                &sample_withholding_records(),
                &sample_anomaly_labels(),
            )
            .unwrap();

        let expected_headers: Vec<(&str, &str)> = vec![
            (
                "tax_jurisdictions.csv",
                "id,name,country_code,region_code,jurisdiction_type,parent_jurisdiction_id,vat_registered",
            ),
            (
                "tax_codes.csv",
                "id,code,description,tax_type,rate,jurisdiction_id,effective_date,expiry_date,is_reverse_charge,is_exempt",
            ),
            (
                "tax_lines.csv",
                "id,document_type,document_id,line_number,tax_code_id,jurisdiction_id,taxable_amount,tax_amount,is_deductible,is_reverse_charge,is_self_assessed",
            ),
            (
                "tax_returns.csv",
                "id,entity_id,jurisdiction_id,period_start,period_end,return_type,status,total_output_tax,total_input_tax,net_payable,filing_deadline,actual_filing_date,is_late",
            ),
            (
                "tax_provisions.csv",
                "id,entity_id,period,current_tax_expense,deferred_tax_asset,deferred_tax_liability,statutory_rate,effective_rate",
            ),
            ("rate_reconciliation.csv", "provision_id,description,rate_impact"),
            (
                "uncertain_tax_positions.csv",
                "id,entity_id,description,tax_benefit,recognition_threshold,recognized_amount,measurement_method",
            ),
            (
                "withholding_records.csv",
                "id,payment_id,vendor_id,withholding_type,treaty_rate,statutory_rate,applied_rate,base_amount,withheld_amount,certificate_number",
            ),
            (
                "tax_anomaly_labels.csv",
                "id,anomaly_type,severity,document_type,document_id,description,original_value,anomalous_value",
            ),
        ];

        for (file_name, expected_header) in &expected_headers {
            let content = std::fs::read_to_string(temp_dir.path().join(file_name)).unwrap();
            let first_line = content.lines().next().unwrap();
            assert_eq!(
                first_line, *expected_header,
                "Header mismatch for '{}'",
                file_name,
            );
        }
    }

    #[test]
    fn test_export_counts_match() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = TaxExporter::new(temp_dir.path());

        let jurisdictions = sample_jurisdictions();
        let codes = sample_tax_codes();
        let lines = sample_tax_lines();
        let returns = sample_tax_returns();
        let provisions = sample_provisions();
        let utps = sample_uncertain_positions();
        let whts = sample_withholding_records();
        let labels = sample_anomaly_labels();

        let summary = exporter
            .export_all(
                &jurisdictions,
                &codes,
                &lines,
                &returns,
                &provisions,
                &utps,
                &whts,
                &labels,
            )
            .unwrap();

        assert_eq!(summary.jurisdictions_count, jurisdictions.len());
        assert_eq!(summary.tax_codes_count, codes.len());
        assert_eq!(summary.tax_lines_count, lines.len());
        assert_eq!(summary.tax_returns_count, returns.len());
        assert_eq!(summary.tax_provisions_count, provisions.len());
        assert_eq!(summary.uncertain_positions_count, utps.len());
        assert_eq!(summary.withholding_records_count, whts.len());
        assert_eq!(summary.anomaly_labels_count, labels.len());

        // Rate reconciliation is flattened from provisions
        let expected_recon_count: usize =
            provisions.iter().map(|p| p.rate_reconciliation.len()).sum();
        assert_eq!(summary.rate_reconciliation_count, expected_recon_count);
    }

    #[test]
    fn test_empty_export() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = TaxExporter::new(temp_dir.path());

        let summary = exporter
            .export_all(&[], &[], &[], &[], &[], &[], &[], &[])
            .unwrap();

        assert_eq!(summary.jurisdictions_count, 0);
        assert_eq!(summary.tax_codes_count, 0);
        assert_eq!(summary.tax_lines_count, 0);
        assert_eq!(summary.tax_returns_count, 0);
        assert_eq!(summary.tax_provisions_count, 0);
        assert_eq!(summary.rate_reconciliation_count, 0);
        assert_eq!(summary.uncertain_positions_count, 0);
        assert_eq!(summary.withholding_records_count, 0);
        assert_eq!(summary.anomaly_labels_count, 0);
        assert_eq!(summary.total(), 0);

        // Files should exist with only headers
        let content =
            std::fs::read_to_string(temp_dir.path().join("tax_jurisdictions.csv")).unwrap();
        let line_count = content.lines().count();
        assert_eq!(
            line_count, 1,
            "Empty export should have exactly one header line"
        );
    }

    #[test]
    fn test_decimal_precision() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = TaxExporter::new(temp_dir.path());

        exporter.export_tax_codes(&sample_tax_codes()).unwrap();
        exporter.export_tax_lines(&sample_tax_lines()).unwrap();
        exporter
            .export_tax_provisions(&sample_provisions())
            .unwrap();
        exporter
            .export_withholding_records(&sample_withholding_records())
            .unwrap();

        // Tax codes: rate 0.20 must appear as exact decimal
        let codes_csv = std::fs::read_to_string(temp_dir.path().join("tax_codes.csv")).unwrap();
        let data_line = codes_csv.lines().nth(1).unwrap();
        assert!(
            data_line.contains("0.20"),
            "Decimal rate should be written as '0.20', got: {}",
            data_line,
        );

        // Tax lines: amounts 1000.00 and 200.00
        let lines_csv = std::fs::read_to_string(temp_dir.path().join("tax_lines.csv")).unwrap();
        let data_line = lines_csv.lines().nth(1).unwrap();
        assert!(
            data_line.contains("1000.00"),
            "Decimal amount should be written as '1000.00', got: {}",
            data_line,
        );
        assert!(
            data_line.contains("200.00"),
            "Decimal amount should be written as '200.00', got: {}",
            data_line,
        );

        // Tax provisions: statutory_rate 0.21
        let provisions_csv =
            std::fs::read_to_string(temp_dir.path().join("tax_provisions.csv")).unwrap();
        let data_line = provisions_csv.lines().nth(1).unwrap();
        assert!(
            data_line.contains("0.21"),
            "Statutory rate should be written as '0.21', got: {}",
            data_line,
        );
    }

    #[test]
    fn test_escape_csv() {
        assert_eq!(escape_csv("hello"), "hello");
        assert_eq!(escape_csv("hello,world"), "\"hello,world\"");
        assert_eq!(escape_csv("hello\"world"), "\"hello\"\"world\"");
        assert_eq!(escape_csv("hello\nworld"), "\"hello\nworld\"");
    }

    #[test]
    fn test_csv_escaping_in_uncertain_positions() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = TaxExporter::new(temp_dir.path());

        let utp = UncertainTaxPosition::new(
            "UTP-ESC",
            "ENT-001",
            "R&D credit, software development",
            dec!(100000),
            dec!(0.50),
            dec!(75000),
            TaxMeasurementMethod::ExpectedValue,
        );

        exporter.export_uncertain_positions(&[utp]).unwrap();

        let content =
            std::fs::read_to_string(temp_dir.path().join("uncertain_tax_positions.csv")).unwrap();
        let data_line = content.lines().nth(1).unwrap();
        assert!(
            data_line.contains("\"R&D credit, software development\""),
            "Comma in description should be quoted, got: {}",
            data_line,
        );
    }

    #[test]
    fn test_optional_fields() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = TaxExporter::new(temp_dir.path());

        // Jurisdiction without region_code or parent
        let j = TaxJurisdiction::new(
            "JUR-INT",
            "International",
            "XX",
            JurisdictionType::Supranational,
        );
        exporter.export_jurisdictions(&[j]).unwrap();

        let content =
            std::fs::read_to_string(temp_dir.path().join("tax_jurisdictions.csv")).unwrap();
        let data_line = content.lines().nth(1).unwrap();
        // Optional fields should be empty strings
        assert!(
            data_line.contains(",,"),
            "Optional None fields should produce empty CSV cells, got: {}",
            data_line,
        );

        // Withholding record without treaty_rate or certificate_number
        let wht = WithholdingTaxRecord::new(
            "WHT-OPT",
            "PAY-001",
            "V-001",
            WithholdingType::ServiceWithholding,
            dec!(0.25),
            dec!(0.25),
            dec!(50000),
        );
        exporter.export_withholding_records(&[wht]).unwrap();

        let content =
            std::fs::read_to_string(temp_dir.path().join("withholding_records.csv")).unwrap();
        let data_line = content.lines().nth(1).unwrap();
        assert!(
            data_line.contains(",,"),
            "Optional None fields should produce empty cells, got: {}",
            data_line,
        );
    }

    #[test]
    fn test_rate_reconciliation_flattening() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = TaxExporter::new(temp_dir.path());

        let provisions = vec![
            TaxProvision::new(
                "TP-A",
                "ENT-001",
                NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
                dec!(100000),
                dec!(20000),
                dec!(30000),
                dec!(0.21),
                dec!(0.24),
            )
            .with_reconciliation_item("State taxes", dec!(0.03))
            .with_reconciliation_item("Permanent differences", dec!(0.005)),
            TaxProvision::new(
                "TP-B",
                "ENT-002",
                NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
                dec!(50000),
                dec!(10000),
                dec!(15000),
                dec!(0.21),
                dec!(0.21),
            ),
        ];

        let count = exporter.export_rate_reconciliation(&provisions).unwrap();
        assert_eq!(count, 2, "Only 2 reconciliation items should be written");

        let content =
            std::fs::read_to_string(temp_dir.path().join("rate_reconciliation.csv")).unwrap();
        // Header + 2 data rows
        assert_eq!(content.lines().count(), 3);
        assert!(content.contains("TP-A"));
        assert!(content.contains("State taxes"));
        assert!(content.contains("Permanent differences"));
        // TP-B has no items, should not appear
        assert!(!content.contains("TP-B"));
    }

    #[test]
    fn test_export_summary_total() {
        let summary = TaxExportSummary {
            jurisdictions_count: 2,
            tax_codes_count: 3,
            tax_lines_count: 10,
            tax_returns_count: 4,
            tax_provisions_count: 2,
            rate_reconciliation_count: 5,
            uncertain_positions_count: 1,
            withholding_records_count: 3,
            anomaly_labels_count: 2,
        };

        assert_eq!(summary.total(), 32);
    }
}
