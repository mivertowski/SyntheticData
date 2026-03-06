//! Oracle EBS format export.
//!
//! Exports data in Oracle E-Business Suite compatible formats:
//! - GL_JE_HEADERS (Journal Entry Headers)
//! - GL_JE_LINES (Journal Entry Lines)
//! - GL_JE_BATCHES (Journal Entry Batches)

use chrono::{Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use datasynth_core::error::SynthResult;
use datasynth_core::models::JournalEntry;

/// Oracle GL_JE_HEADERS record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleJeHeader {
    /// Unique header ID
    pub je_header_id: u64,
    /// Ledger ID
    pub ledger_id: u64,
    /// Batch ID
    pub je_batch_id: u64,
    /// Period name (e.g., "JAN-24")
    pub period_name: String,
    /// Journal entry name
    pub name: String,
    /// Journal category (e.g., "MANUAL", "ADJUSTMENT")
    pub je_category: String,
    /// Journal source (e.g., "MANUAL", "PAYABLES")
    pub je_source: String,
    /// Currency code
    pub currency_code: String,
    /// Actual flag (A=Actual, B=Budget, E=Encumbrance)
    pub actual_flag: String,
    /// Status (P=Posted, U=Unposted)
    pub status: String,
    /// Default effective date
    pub default_effective_date: NaiveDate,
    /// Description
    pub description: Option<String>,
    /// External reference
    pub external_reference: Option<String>,
    /// Parent header ID (for reversals)
    pub parent_je_header_id: Option<u64>,
    /// Reversal flag
    pub accrual_rev_flag: Option<String>,
    /// Running total (debits)
    pub running_total_dr: Decimal,
    /// Running total (credits)
    pub running_total_cr: Decimal,
    /// Running total accounted (debits)
    pub running_total_accounted_dr: Decimal,
    /// Running total accounted (credits)
    pub running_total_accounted_cr: Decimal,
    /// Creation date
    pub creation_date: NaiveDate,
    /// Created by user ID
    pub created_by: u64,
    /// Last update date
    pub last_update_date: NaiveDate,
    /// Last updated by user ID
    pub last_updated_by: u64,
}

impl Default for OracleJeHeader {
    fn default() -> Self {
        let now = Utc::now().date_naive();
        Self {
            je_header_id: 0,
            ledger_id: 1,
            je_batch_id: 0,
            period_name: String::new(),
            name: String::new(),
            je_category: "MANUAL".to_string(),
            je_source: "MANUAL".to_string(),
            currency_code: "USD".to_string(),
            actual_flag: "A".to_string(),
            status: "P".to_string(),
            default_effective_date: now,
            description: None,
            external_reference: None,
            parent_je_header_id: None,
            accrual_rev_flag: None,
            running_total_dr: Decimal::ZERO,
            running_total_cr: Decimal::ZERO,
            running_total_accounted_dr: Decimal::ZERO,
            running_total_accounted_cr: Decimal::ZERO,
            creation_date: now,
            created_by: 0,
            last_update_date: now,
            last_updated_by: 0,
        }
    }
}

/// Oracle GL_JE_LINES record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleJeLine {
    /// Unique line ID
    pub je_line_num: u32,
    /// Header ID (foreign key)
    pub je_header_id: u64,
    /// Ledger ID
    pub ledger_id: u64,
    /// Code combination ID (account string)
    pub code_combination_id: u64,
    /// Period name
    pub period_name: String,
    /// Effective date
    pub effective_date: NaiveDate,
    /// Status (P=Posted, U=Unposted)
    pub status: String,
    /// Entered debit
    pub entered_dr: Option<Decimal>,
    /// Entered credit
    pub entered_cr: Option<Decimal>,
    /// Accounted debit (in functional currency)
    pub accounted_dr: Option<Decimal>,
    /// Accounted credit (in functional currency)
    pub accounted_cr: Option<Decimal>,
    /// Currency code
    pub currency_code: String,
    /// Currency conversion rate
    pub currency_conversion_rate: Option<Decimal>,
    /// Currency conversion type
    pub currency_conversion_type: Option<String>,
    /// Currency conversion date
    pub currency_conversion_date: Option<NaiveDate>,
    /// Description
    pub description: Option<String>,
    /// Reference columns
    pub reference_1: Option<String>,
    pub reference_2: Option<String>,
    pub reference_3: Option<String>,
    pub reference_4: Option<String>,
    pub reference_5: Option<String>,
    /// Statistical amount
    pub stat_amount: Option<Decimal>,
    /// Subledger document sequence ID
    pub subledger_doc_sequence_id: Option<u64>,
    /// Attribute columns (DFF)
    pub attribute1: Option<String>,
    pub attribute2: Option<String>,
    pub attribute3: Option<String>,
    /// Creation date
    pub creation_date: NaiveDate,
    /// Created by
    pub created_by: u64,
}

impl Default for OracleJeLine {
    fn default() -> Self {
        let now = Utc::now().date_naive();
        Self {
            je_line_num: 0,
            je_header_id: 0,
            ledger_id: 1,
            code_combination_id: 0,
            period_name: String::new(),
            effective_date: now,
            status: "P".to_string(),
            entered_dr: None,
            entered_cr: None,
            accounted_dr: None,
            accounted_cr: None,
            currency_code: "USD".to_string(),
            currency_conversion_rate: None,
            currency_conversion_type: None,
            currency_conversion_date: None,
            description: None,
            reference_1: None,
            reference_2: None,
            reference_3: None,
            reference_4: None,
            reference_5: None,
            stat_amount: None,
            subledger_doc_sequence_id: None,
            attribute1: None,
            attribute2: None,
            attribute3: None,
            creation_date: now,
            created_by: 0,
        }
    }
}

/// Oracle GL_JE_BATCHES record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleJeBatch {
    /// Batch ID
    pub je_batch_id: u64,
    /// Batch name
    pub name: String,
    /// Ledger ID
    pub ledger_id: u64,
    /// Status (P=Posted, U=Unposted)
    pub status: String,
    /// Actual flag
    pub actual_flag: String,
    /// Default period name
    pub default_period_name: String,
    /// Description
    pub description: Option<String>,
    /// Running total debits
    pub running_total_dr: Decimal,
    /// Running total credits
    pub running_total_cr: Decimal,
    /// Posted date
    pub posted_date: Option<NaiveDate>,
    /// Creation date
    pub creation_date: NaiveDate,
    /// Created by
    pub created_by: u64,
}

impl Default for OracleJeBatch {
    fn default() -> Self {
        let now = Utc::now().date_naive();
        Self {
            je_batch_id: 0,
            name: String::new(),
            ledger_id: 1,
            status: "P".to_string(),
            actual_flag: "A".to_string(),
            default_period_name: String::new(),
            description: None,
            running_total_dr: Decimal::ZERO,
            running_total_cr: Decimal::ZERO,
            posted_date: Some(now),
            creation_date: now,
            created_by: 0,
        }
    }
}

/// Configuration for Oracle export.
#[derive(Debug, Clone)]
pub struct OracleExportConfig {
    /// Ledger ID
    pub ledger_id: u64,
    /// Set of Books ID (legacy)
    pub set_of_books_id: u64,
    /// Functional currency
    pub functional_currency: String,
    /// User ID for created_by/last_updated_by
    pub user_id: u64,
    /// Include batches table
    pub include_batches: bool,
    /// Account segment separator
    pub segment_separator: String,
    /// Number of segments in account combination
    pub num_segments: usize,
}

impl Default for OracleExportConfig {
    fn default() -> Self {
        Self {
            ledger_id: 1,
            set_of_books_id: 1,
            functional_currency: "USD".to_string(),
            user_id: 1,
            include_batches: true,
            segment_separator: "-".to_string(),
            num_segments: 6,
        }
    }
}

/// Oracle EBS format exporter.
pub struct OracleExporter {
    config: OracleExportConfig,
    header_counter: u64,
    line_counter: u64,
    #[allow(dead_code)] // Reserved for future batch export feature
    batch_counter: u64,
    /// Maps company code + GL account to code_combination_id
    ccid_map: HashMap<String, u64>,
    next_ccid: u64,
}

impl OracleExporter {
    /// Create a new Oracle exporter.
    pub fn new(config: OracleExportConfig) -> Self {
        Self {
            config,
            header_counter: 0,
            line_counter: 0,
            batch_counter: 0,
            ccid_map: HashMap::new(),
            next_ccid: 1000,
        }
    }

    /// Get or create a code combination ID for an account.
    fn get_ccid(&mut self, company_code: &str, gl_account: &str, cost_center: Option<&str>) -> u64 {
        let key = format!(
            "{}-{}-{}",
            company_code,
            gl_account,
            cost_center.unwrap_or("0000")
        );
        if let Some(&ccid) = self.ccid_map.get(&key) {
            return ccid;
        }
        let ccid = self.next_ccid;
        self.next_ccid += 1;
        self.ccid_map.insert(key, ccid);
        ccid
    }

    /// Generate Oracle period name from date.
    fn period_name(date: NaiveDate) -> String {
        let month = match date.month() {
            1 => "JAN",
            2 => "FEB",
            3 => "MAR",
            4 => "APR",
            5 => "MAY",
            6 => "JUN",
            7 => "JUL",
            8 => "AUG",
            9 => "SEP",
            10 => "OCT",
            11 => "NOV",
            12 => "DEC",
            _ => "JAN",
        };
        format!("{}-{}", month, date.format("%y"))
    }

    /// Map document type to Oracle category.
    fn je_category(doc_type: &str) -> String {
        match doc_type {
            "SA" => "MANUAL".to_string(),
            "RE" | "KR" => "PAYABLES".to_string(),
            "RV" | "DR" => "RECEIVABLES".to_string(),
            "KZ" => "PAYMENTS".to_string(),
            "DZ" => "RECEIPTS".to_string(),
            "AB" | "AA" => "ASSETS".to_string(),
            _ => "OTHER".to_string(),
        }
    }

    /// Map transaction source to Oracle source.
    fn je_source(source: &str) -> String {
        match source {
            "Manual" | "ManualEntry" => "MANUAL".to_string(),
            "Payables" | "VendorInvoice" => "PAYABLES".to_string(),
            "Receivables" | "CustomerInvoice" => "RECEIVABLES".to_string(),
            "Assets" | "Depreciation" => "ASSETS".to_string(),
            "Inventory" => "INVENTORY".to_string(),
            _ => "OTHER".to_string(),
        }
    }

    /// Convert JournalEntry to Oracle header and lines.
    pub fn convert(&mut self, je: &JournalEntry) -> (OracleJeHeader, Vec<OracleJeLine>) {
        self.header_counter += 1;
        let header_id = self.header_counter;

        let period_name = Self::period_name(je.header.posting_date);

        // Calculate totals
        let mut total_dr = Decimal::ZERO;
        let mut total_cr = Decimal::ZERO;
        for line in &je.lines {
            total_dr += line.debit_amount;
            total_cr += line.credit_amount;
        }

        let header = OracleJeHeader {
            je_header_id: header_id,
            ledger_id: self.config.ledger_id,
            je_batch_id: 0, // Set later if batching
            period_name: period_name.clone(),
            name: format!("JE-{}", je.header.document_id),
            je_category: Self::je_category(&je.header.document_type),
            je_source: Self::je_source(&format!("{:?}", je.header.source)),
            currency_code: je.header.currency.clone(),
            actual_flag: "A".to_string(),
            status: "P".to_string(),
            default_effective_date: je.header.posting_date,
            description: je.header.header_text.clone(),
            external_reference: je.header.reference.clone(),
            parent_je_header_id: None,
            accrual_rev_flag: None,
            running_total_dr: total_dr,
            running_total_cr: total_cr,
            running_total_accounted_dr: total_dr * je.header.exchange_rate,
            running_total_accounted_cr: total_cr * je.header.exchange_rate,
            creation_date: je.header.created_at.date_naive(),
            created_by: self.config.user_id,
            last_update_date: je.header.created_at.date_naive(),
            last_updated_by: self.config.user_id,
        };

        let mut lines = Vec::new();
        for line in &je.lines {
            self.line_counter += 1;
            let ccid = self.get_ccid(
                &je.header.company_code,
                &line.gl_account,
                line.cost_center.as_deref(),
            );

            let oracle_line = OracleJeLine {
                je_line_num: line.line_number,
                je_header_id: header_id,
                ledger_id: self.config.ledger_id,
                code_combination_id: ccid,
                period_name: period_name.clone(),
                effective_date: je.header.posting_date,
                status: "P".to_string(),
                entered_dr: if line.debit_amount > Decimal::ZERO {
                    Some(line.debit_amount)
                } else {
                    None
                },
                entered_cr: if line.credit_amount > Decimal::ZERO {
                    Some(line.credit_amount)
                } else {
                    None
                },
                accounted_dr: if line.debit_amount > Decimal::ZERO {
                    Some(line.debit_amount * je.header.exchange_rate)
                } else {
                    None
                },
                accounted_cr: if line.credit_amount > Decimal::ZERO {
                    Some(line.credit_amount * je.header.exchange_rate)
                } else {
                    None
                },
                currency_code: je.header.currency.clone(),
                currency_conversion_rate: if je.header.exchange_rate != Decimal::ONE {
                    Some(je.header.exchange_rate)
                } else {
                    None
                },
                currency_conversion_type: if je.header.exchange_rate != Decimal::ONE {
                    Some("Corporate".to_string())
                } else {
                    None
                },
                currency_conversion_date: if je.header.exchange_rate != Decimal::ONE {
                    Some(je.header.posting_date)
                } else {
                    None
                },
                description: line.line_text.clone(),
                reference_1: Some(je.header.company_code.clone()),
                reference_2: Some(line.gl_account.clone()),
                reference_3: line.cost_center.clone(),
                reference_4: line.profit_center.clone(),
                reference_5: je.header.reference.clone(),
                stat_amount: line.quantity,
                subledger_doc_sequence_id: None,
                attribute1: if je.header.is_fraud {
                    Some("Y".to_string())
                } else {
                    None
                },
                attribute2: je.header.fraud_type.map(|ft| format!("{ft:?}")),
                attribute3: je.header.business_process.map(|bp| format!("{bp:?}")),
                creation_date: je.header.created_at.date_naive(),
                created_by: self.config.user_id,
            };
            lines.push(oracle_line);
        }

        (header, lines)
    }

    /// Export journal entries to Oracle format files.
    pub fn export_to_files(
        &mut self,
        entries: &[JournalEntry],
        output_dir: &Path,
    ) -> SynthResult<HashMap<String, String>> {
        std::fs::create_dir_all(output_dir)?;

        let mut output_files = HashMap::new();

        // Export headers
        let header_path = output_dir.join("gl_je_headers.csv");
        let lines_path = output_dir.join("gl_je_lines.csv");

        let header_file = File::create(&header_path)?;
        let mut header_writer = BufWriter::with_capacity(256 * 1024, header_file);

        let lines_file = File::create(&lines_path)?;
        let mut lines_writer = BufWriter::with_capacity(256 * 1024, lines_file);

        // Write header row
        writeln!(
            header_writer,
            "JE_HEADER_ID,LEDGER_ID,JE_BATCH_ID,PERIOD_NAME,NAME,JE_CATEGORY,JE_SOURCE,\
            CURRENCY_CODE,ACTUAL_FLAG,STATUS,DEFAULT_EFFECTIVE_DATE,DESCRIPTION,\
            RUNNING_TOTAL_DR,RUNNING_TOTAL_CR,CREATION_DATE,CREATED_BY"
        )?;

        writeln!(
            lines_writer,
            "JE_LINE_NUM,JE_HEADER_ID,LEDGER_ID,CODE_COMBINATION_ID,PERIOD_NAME,EFFECTIVE_DATE,\
            STATUS,ENTERED_DR,ENTERED_CR,ACCOUNTED_DR,ACCOUNTED_CR,CURRENCY_CODE,\
            DESCRIPTION,REFERENCE_1,REFERENCE_2,REFERENCE_3,ATTRIBUTE1,ATTRIBUTE2,CREATION_DATE,CREATED_BY"
        )?;

        for je in entries {
            let (header, lines) = self.convert(je);

            writeln!(
                header_writer,
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                header.je_header_id,
                header.ledger_id,
                header.je_batch_id,
                header.period_name,
                escape_csv_field(&header.name),
                header.je_category,
                header.je_source,
                header.currency_code,
                header.actual_flag,
                header.status,
                header.default_effective_date,
                escape_csv_field(&header.description.unwrap_or_default()),
                header.running_total_dr,
                header.running_total_cr,
                header.creation_date,
                header.created_by,
            )?;

            for line in lines {
                writeln!(
                    lines_writer,
                    "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                    line.je_line_num,
                    line.je_header_id,
                    line.ledger_id,
                    line.code_combination_id,
                    line.period_name,
                    line.effective_date,
                    line.status,
                    line.entered_dr.map(|d| d.to_string()).unwrap_or_default(),
                    line.entered_cr.map(|d| d.to_string()).unwrap_or_default(),
                    line.accounted_dr.map(|d| d.to_string()).unwrap_or_default(),
                    line.accounted_cr.map(|d| d.to_string()).unwrap_or_default(),
                    line.currency_code,
                    escape_csv_field(&line.description.unwrap_or_default()),
                    line.reference_1.as_deref().unwrap_or(""),
                    line.reference_2.as_deref().unwrap_or(""),
                    line.reference_3.as_deref().unwrap_or(""),
                    line.attribute1.as_deref().unwrap_or(""),
                    line.attribute2.as_deref().unwrap_or(""),
                    line.creation_date,
                    line.created_by,
                )?;
            }
        }

        header_writer.flush()?;
        lines_writer.flush()?;

        output_files.insert(
            "gl_je_headers".to_string(),
            header_path.to_string_lossy().to_string(),
        );
        output_files.insert(
            "gl_je_lines".to_string(),
            lines_path.to_string_lossy().to_string(),
        );

        // Export code combinations
        let ccid_path = output_dir.join("gl_code_combinations.csv");
        self.export_code_combinations(&ccid_path)?;
        output_files.insert(
            "gl_code_combinations".to_string(),
            ccid_path.to_string_lossy().to_string(),
        );

        Ok(output_files)
    }

    /// Export code combinations mapping.
    fn export_code_combinations(&self, filepath: &Path) -> SynthResult<()> {
        let file = File::create(filepath)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            writer,
            "CODE_COMBINATION_ID,SEGMENT1,SEGMENT2,SEGMENT3,ENABLED_FLAG"
        )?;

        for (key, ccid) in &self.ccid_map {
            let parts: Vec<&str> = key.split('-').collect();
            let segment1 = parts.first().unwrap_or(&"");
            let segment2 = parts.get(1).unwrap_or(&"");
            let segment3 = parts.get(2).unwrap_or(&"0000");

            writeln!(writer, "{ccid},{segment1},{segment2},{segment3},Y")?;
        }

        writer.flush()?;
        Ok(())
    }
}

/// Escape a field for CSV output.
fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_helpers::create_test_je;
    use rust_decimal::Decimal;
    use tempfile::TempDir;

    #[test]
    fn test_period_name_generation() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(OracleExporter::period_name(date), "JUN-24");

        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        assert_eq!(OracleExporter::period_name(date), "JAN-24");

        let date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        assert_eq!(OracleExporter::period_name(date), "DEC-24");
    }

    #[test]
    fn test_oracle_exporter_creates_files() {
        let temp_dir = TempDir::new().unwrap();
        let config = OracleExportConfig::default();
        let mut exporter = OracleExporter::new(config);

        let entries = vec![create_test_je()];
        let result = exporter.export_to_files(&entries, temp_dir.path());

        assert!(result.is_ok());
        let files = result.unwrap();
        assert!(files.contains_key("gl_je_headers"));
        assert!(files.contains_key("gl_je_lines"));
        assert!(files.contains_key("gl_code_combinations"));

        assert!(temp_dir.path().join("gl_je_headers.csv").exists());
        assert!(temp_dir.path().join("gl_je_lines.csv").exists());
        assert!(temp_dir.path().join("gl_code_combinations.csv").exists());
    }

    #[test]
    fn test_conversion_produces_balanced_totals() {
        let config = OracleExportConfig::default();
        let mut exporter = OracleExporter::new(config);
        let je = create_test_je();

        let (header, lines) = exporter.convert(&je);

        assert_eq!(header.running_total_dr, header.running_total_cr);
        assert_eq!(lines.len(), 2);

        let line_total_dr: Decimal = lines.iter().filter_map(|l| l.entered_dr).sum();
        let line_total_cr: Decimal = lines.iter().filter_map(|l| l.entered_cr).sum();
        assert_eq!(line_total_dr, line_total_cr);
    }
}
