//! NetSuite format export.
//!
//! Exports data in NetSuite-compatible formats for journal entries
//! with support for custom fields, subsidiaries, and multi-book accounting.

use chrono::{Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use datasynth_core::error::{SynthError, SynthResult};
use datasynth_core::models::JournalEntry;

/// NetSuite journal entry header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetSuiteJournalEntry {
    /// Internal ID
    pub internal_id: u64,
    /// External ID (for import)
    pub external_id: String,
    /// Transaction number
    pub tran_id: String,
    /// Transaction date
    pub tran_date: NaiveDate,
    /// Posting period (internal ID)
    pub posting_period: String,
    /// Subsidiary (internal ID)
    pub subsidiary: u64,
    /// Currency (internal ID or ISO code)
    pub currency: String,
    /// Exchange rate
    pub exchange_rate: Decimal,
    /// Memo
    pub memo: Option<String>,
    /// Is approved
    pub approved: bool,
    /// Created date
    pub created_date: NaiveDate,
    /// Last modified date
    pub last_modified_date: NaiveDate,
    /// Created by (employee ID)
    pub created_by: Option<u64>,
    /// Reversal date (for reversing journals)
    pub reversal_date: Option<NaiveDate>,
    /// Reversal defer (if reversal is deferred)
    pub reversal_defer: bool,
    /// Department (if header-level)
    pub department: Option<u64>,
    /// Class (if header-level)
    pub class: Option<u64>,
    /// Location (if header-level)
    pub location: Option<u64>,
    /// Custom fields
    pub custom_fields: HashMap<String, String>,
    /// Total debits
    pub total_debit: Decimal,
    /// Total credits
    pub total_credit: Decimal,
}

impl Default for NetSuiteJournalEntry {
    fn default() -> Self {
        let now = Utc::now().date_naive();
        Self {
            internal_id: 0,
            external_id: String::new(),
            tran_id: String::new(),
            tran_date: now,
            posting_period: String::new(),
            subsidiary: 1,
            currency: "USD".to_string(),
            exchange_rate: Decimal::ONE,
            memo: None,
            approved: true,
            created_date: now,
            last_modified_date: now,
            created_by: None,
            reversal_date: None,
            reversal_defer: false,
            department: None,
            class: None,
            location: None,
            custom_fields: HashMap::new(),
            total_debit: Decimal::ZERO,
            total_credit: Decimal::ZERO,
        }
    }
}

/// NetSuite journal entry line.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetSuiteJournalLine {
    /// Line number
    pub line: u32,
    /// Account (internal ID)
    pub account: u64,
    /// Account name (for reference)
    pub account_name: Option<String>,
    /// Debit amount
    pub debit: Option<Decimal>,
    /// Credit amount
    pub credit: Option<Decimal>,
    /// Line memo
    pub memo: Option<String>,
    /// Entity (customer/vendor internal ID)
    pub entity: Option<u64>,
    /// Entity type
    pub entity_type: Option<String>,
    /// Department
    pub department: Option<u64>,
    /// Class
    pub class: Option<u64>,
    /// Location
    pub location: Option<u64>,
    /// Eliminate intercompany
    pub eliminate: bool,
    /// Tax code (if applicable)
    pub tax_code: Option<String>,
    /// Tax amount
    pub tax_amount: Option<Decimal>,
    /// Custom fields for the line
    pub custom_fields: HashMap<String, String>,
}

/// NetSuite export configuration.
#[derive(Debug, Clone)]
pub struct NetSuiteExportConfig {
    /// Default subsidiary ID
    pub default_subsidiary: u64,
    /// Subsidiary mapping (company code -> NetSuite subsidiary ID)
    pub subsidiary_map: HashMap<String, u64>,
    /// Account mapping (GL account -> NetSuite account ID)
    pub account_map: HashMap<String, u64>,
    /// Currency mapping (ISO -> NetSuite currency ID)
    pub currency_map: HashMap<String, u64>,
    /// Department mapping
    pub department_map: HashMap<String, u64>,
    /// Class mapping
    pub class_map: HashMap<String, u64>,
    /// Location mapping
    pub location_map: HashMap<String, u64>,
    /// Include custom fields
    pub include_custom_fields: bool,
    /// Custom field definitions for fraud/anomaly flags
    pub fraud_custom_field: Option<String>,
    /// Custom field for business process
    pub process_custom_field: Option<String>,
}

impl Default for NetSuiteExportConfig {
    fn default() -> Self {
        Self {
            default_subsidiary: 1,
            subsidiary_map: HashMap::new(),
            account_map: HashMap::new(),
            currency_map: HashMap::new(),
            department_map: HashMap::new(),
            class_map: HashMap::new(),
            location_map: HashMap::new(),
            include_custom_fields: true,
            fraud_custom_field: Some("custbody_fraud_flag".to_string()),
            process_custom_field: Some("custbody_business_process".to_string()),
        }
    }
}

/// NetSuite format exporter.
pub struct NetSuiteExporter {
    config: NetSuiteExportConfig,
    journal_counter: u64,
    /// Account ID mapping (generated if not provided)
    generated_account_ids: HashMap<String, u64>,
    next_account_id: u64,
}

impl NetSuiteExporter {
    /// Create a new NetSuite exporter.
    pub fn new(config: NetSuiteExportConfig) -> Self {
        Self {
            config,
            journal_counter: 0,
            generated_account_ids: HashMap::new(),
            next_account_id: 1000,
        }
    }

    /// Get subsidiary ID for a company code.
    fn get_subsidiary(&self, company_code: &str) -> u64 {
        self.config
            .subsidiary_map
            .get(company_code)
            .copied()
            .unwrap_or(self.config.default_subsidiary)
    }

    /// Get or generate account ID for a GL account.
    fn get_account_id(&mut self, gl_account: &str) -> u64 {
        if let Some(&id) = self.config.account_map.get(gl_account) {
            return id;
        }
        if let Some(&id) = self.generated_account_ids.get(gl_account) {
            return id;
        }
        let id = self.next_account_id;
        self.next_account_id += 1;
        self.generated_account_ids
            .insert(gl_account.to_string(), id);
        id
    }

    /// Generate posting period from date.
    fn posting_period(date: NaiveDate) -> String {
        // NetSuite period format: "Jun 2024"
        let month = match date.month() {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => "Jan",
        };
        format!("{} {}", month, date.year())
    }

    /// Convert JournalEntry to NetSuite format.
    pub fn convert(
        &mut self,
        je: &JournalEntry,
    ) -> (NetSuiteJournalEntry, Vec<NetSuiteJournalLine>) {
        self.journal_counter += 1;

        // Calculate totals
        let mut total_debit = Decimal::ZERO;
        let mut total_credit = Decimal::ZERO;
        for line in &je.lines {
            total_debit += line.debit_amount;
            total_credit += line.credit_amount;
        }

        let mut custom_fields = HashMap::new();
        if self.config.include_custom_fields {
            if let Some(ref fraud_field) = self.config.fraud_custom_field {
                if je.header.is_fraud {
                    custom_fields.insert(fraud_field.clone(), "T".to_string());
                    if let Some(fraud_type) = je.header.fraud_type {
                        custom_fields
                            .insert(format!("{}_type", fraud_field), format!("{:?}", fraud_type));
                    }
                }
            }
            if let Some(ref process_field) = self.config.process_custom_field {
                if let Some(business_process) = je.header.business_process {
                    custom_fields.insert(process_field.clone(), format!("{:?}", business_process));
                }
            }
        }

        let header = NetSuiteJournalEntry {
            internal_id: self.journal_counter,
            external_id: format!("JE_{}", je.header.document_id),
            tran_id: format!("JE{:08}", self.journal_counter),
            tran_date: je.header.posting_date,
            posting_period: Self::posting_period(je.header.posting_date),
            subsidiary: self.get_subsidiary(&je.header.company_code),
            currency: je.header.currency.clone(),
            exchange_rate: je.header.exchange_rate,
            memo: je.header.header_text.clone(),
            approved: true,
            created_date: je.header.created_at.date_naive(),
            last_modified_date: je.header.created_at.date_naive(),
            created_by: None,
            reversal_date: None,
            reversal_defer: false,
            department: None,
            class: None,
            location: None,
            custom_fields,
            total_debit,
            total_credit,
        };

        let mut lines = Vec::new();
        for je_line in &je.lines {
            let account_id = self.get_account_id(&je_line.gl_account);

            let mut line_custom_fields = HashMap::new();
            if self.config.include_custom_fields {
                if let Some(ref cost_center) = je_line.cost_center {
                    line_custom_fields
                        .insert("custcol_cost_center".to_string(), cost_center.clone());
                }
                if let Some(ref profit_center) = je_line.profit_center {
                    line_custom_fields
                        .insert("custcol_profit_center".to_string(), profit_center.clone());
                }
            }

            let ns_line = NetSuiteJournalLine {
                line: je_line.line_number,
                account: account_id,
                account_name: Some(je_line.gl_account.clone()),
                debit: if je_line.debit_amount > Decimal::ZERO {
                    Some(je_line.debit_amount)
                } else {
                    None
                },
                credit: if je_line.credit_amount > Decimal::ZERO {
                    Some(je_line.credit_amount)
                } else {
                    None
                },
                memo: je_line.line_text.clone(),
                entity: None,
                entity_type: None,
                department: je_line
                    .cost_center
                    .as_ref()
                    .and_then(|cc| self.config.department_map.get(cc).copied()),
                class: je_line
                    .profit_center
                    .as_ref()
                    .and_then(|pc| self.config.class_map.get(pc).copied()),
                location: None,
                eliminate: je_line.trading_partner.is_some(),
                tax_code: je_line.tax_code.clone(),
                tax_amount: je_line.tax_amount,
                custom_fields: line_custom_fields,
            };
            lines.push(ns_line);
        }

        (header, lines)
    }

    /// Export journal entries to NetSuite CSV format.
    pub fn export_to_files(
        &mut self,
        entries: &[JournalEntry],
        output_dir: &Path,
    ) -> SynthResult<HashMap<String, String>> {
        std::fs::create_dir_all(output_dir)?;

        let mut output_files = HashMap::new();

        // Export main journal entries
        let je_path = output_dir.join("netsuite_journal_entries.csv");
        let lines_path = output_dir.join("netsuite_journal_lines.csv");

        let je_file = File::create(&je_path)?;
        let mut je_writer = BufWriter::with_capacity(256 * 1024, je_file);

        let lines_file = File::create(&lines_path)?;
        let mut lines_writer = BufWriter::with_capacity(256 * 1024, lines_file);

        // Write headers
        let mut je_header = "Internal ID,External ID,Tran ID,Tran Date,Posting Period,Subsidiary,\
            Currency,Exchange Rate,Memo,Approved,Total Debit,Total Credit"
            .to_string();
        if self.config.include_custom_fields {
            if let Some(ref fraud_field) = self.config.fraud_custom_field {
                je_header.push_str(&format!(",{},{}_type", fraud_field, fraud_field));
            }
            if let Some(ref process_field) = self.config.process_custom_field {
                je_header.push_str(&format!(",{}", process_field));
            }
        }
        writeln!(je_writer, "{}", je_header)?;

        let mut line_header = "Journal Internal ID,Line,Account,Account Name,Debit,Credit,Memo,\
            Department,Class,Location,Eliminate,Tax Code,Tax Amount"
            .to_string();
        if self.config.include_custom_fields {
            line_header.push_str(",custcol_cost_center,custcol_profit_center");
        }
        writeln!(lines_writer, "{}", line_header)?;

        for je in entries {
            let (header, lines) = self.convert(je);

            // Write journal entry
            let mut je_row = format!(
                "{},{},{},{},{},{},{},{},{},{},{},{}",
                header.internal_id,
                escape_csv_field(&header.external_id),
                escape_csv_field(&header.tran_id),
                header.tran_date,
                escape_csv_field(&header.posting_period),
                header.subsidiary,
                header.currency,
                header.exchange_rate,
                escape_csv_field(header.memo.as_deref().unwrap_or("")),
                if header.approved { "T" } else { "F" },
                header.total_debit,
                header.total_credit,
            );

            if self.config.include_custom_fields {
                if let Some(ref fraud_field) = self.config.fraud_custom_field {
                    je_row.push_str(&format!(
                        ",{},{}",
                        header
                            .custom_fields
                            .get(fraud_field)
                            .map(|s| s.as_str())
                            .unwrap_or(""),
                        header
                            .custom_fields
                            .get(&format!("{}_type", fraud_field))
                            .map(|s| s.as_str())
                            .unwrap_or(""),
                    ));
                }
                if let Some(ref process_field) = self.config.process_custom_field {
                    je_row.push_str(&format!(
                        ",{}",
                        header
                            .custom_fields
                            .get(process_field)
                            .map(|s| s.as_str())
                            .unwrap_or(""),
                    ));
                }
            }
            writeln!(je_writer, "{}", je_row)?;

            // Write lines
            for line in lines {
                let mut line_row = format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{},{}",
                    header.internal_id,
                    line.line,
                    line.account,
                    escape_csv_field(line.account_name.as_deref().unwrap_or("")),
                    line.debit.map(|d| d.to_string()).unwrap_or_default(),
                    line.credit.map(|d| d.to_string()).unwrap_or_default(),
                    escape_csv_field(line.memo.as_deref().unwrap_or("")),
                    line.department.map(|d| d.to_string()).unwrap_or_default(),
                    line.class.map(|d| d.to_string()).unwrap_or_default(),
                    line.location.map(|d| d.to_string()).unwrap_or_default(),
                    if line.eliminate { "T" } else { "F" },
                    line.tax_code.as_deref().unwrap_or(""),
                    line.tax_amount.map(|d| d.to_string()).unwrap_or_default(),
                );

                if self.config.include_custom_fields {
                    line_row.push_str(&format!(
                        ",{},{}",
                        line.custom_fields
                            .get("custcol_cost_center")
                            .map(|s| s.as_str())
                            .unwrap_or(""),
                        line.custom_fields
                            .get("custcol_profit_center")
                            .map(|s| s.as_str())
                            .unwrap_or(""),
                    ));
                }
                writeln!(lines_writer, "{}", line_row)?;
            }
        }

        je_writer.flush()?;
        lines_writer.flush()?;

        output_files.insert(
            "journal_entries".to_string(),
            je_path.to_string_lossy().to_string(),
        );
        output_files.insert(
            "journal_lines".to_string(),
            lines_path.to_string_lossy().to_string(),
        );

        // Export account mapping
        let account_path = output_dir.join("netsuite_accounts.csv");
        self.export_accounts(&account_path)?;
        output_files.insert(
            "accounts".to_string(),
            account_path.to_string_lossy().to_string(),
        );

        Ok(output_files)
    }

    /// Export account mapping.
    fn export_accounts(&self, filepath: &Path) -> SynthResult<()> {
        let file = File::create(filepath)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        writeln!(writer, "Internal ID,Account Number,External ID")?;

        // Export configured mappings
        for (account_num, &account_id) in &self.config.account_map {
            writeln!(
                writer,
                "{},{},ACCT_{}",
                account_id,
                escape_csv_field(account_num),
                account_num,
            )?;
        }

        // Export generated mappings
        for (account_num, &account_id) in &self.generated_account_ids {
            writeln!(
                writer,
                "{},{},ACCT_{}",
                account_id,
                escape_csv_field(account_num),
                account_num,
            )?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Export to NetSuite SuiteScript-compatible JSON format.
    pub fn export_to_json(
        &mut self,
        entries: &[JournalEntry],
        output_dir: &Path,
    ) -> SynthResult<String> {
        std::fs::create_dir_all(output_dir)?;

        let json_path = output_dir.join("netsuite_journal_entries.json");
        let file = File::create(&json_path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        let mut records = Vec::new();
        for je in entries {
            let (header, lines) = self.convert(je);
            records.push(serde_json::json!({
                "recordType": "journalentry",
                "externalId": header.external_id,
                "tranId": header.tran_id,
                "tranDate": header.tran_date.to_string(),
                "postingPeriod": header.posting_period,
                "subsidiary": header.subsidiary,
                "currency": header.currency,
                "exchangeRate": header.exchange_rate.to_string(),
                "memo": header.memo,
                "approved": header.approved,
                "customFields": header.custom_fields,
                "lines": lines.iter().map(|l| serde_json::json!({
                    "line": l.line,
                    "account": l.account,
                    "debit": l.debit.map(|d| d.to_string()),
                    "credit": l.credit.map(|d| d.to_string()),
                    "memo": l.memo,
                    "department": l.department,
                    "class": l.class,
                    "location": l.location,
                    "eliminate": l.eliminate,
                    "taxCode": l.tax_code,
                    "customFields": l.custom_fields,
                })).collect::<Vec<_>>(),
            }));
        }

        let json_output = serde_json::to_string_pretty(&records)
            .map_err(|e| SynthError::generation(format!("JSON serialization error: {}", e)))?;
        writer.write_all(json_output.as_bytes())?;
        writer.flush()?;

        Ok(json_path.to_string_lossy().to_string())
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
    use tempfile::TempDir;

    #[test]
    fn test_posting_period_generation() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(NetSuiteExporter::posting_period(date), "Jun 2024");

        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        assert_eq!(NetSuiteExporter::posting_period(date), "Jan 2024");
    }

    #[test]
    fn test_netsuite_exporter_creates_files() {
        let temp_dir = TempDir::new().unwrap();
        let config = NetSuiteExportConfig::default();
        let mut exporter = NetSuiteExporter::new(config);

        let entries = vec![create_test_je()];
        let result = exporter.export_to_files(&entries, temp_dir.path());

        assert!(result.is_ok());
        let files = result.unwrap();
        assert!(files.contains_key("journal_entries"));
        assert!(files.contains_key("journal_lines"));
        assert!(files.contains_key("accounts"));

        assert!(temp_dir
            .path()
            .join("netsuite_journal_entries.csv")
            .exists());
        assert!(temp_dir.path().join("netsuite_journal_lines.csv").exists());
        assert!(temp_dir.path().join("netsuite_accounts.csv").exists());
    }

    #[test]
    fn test_netsuite_json_export() {
        let temp_dir = TempDir::new().unwrap();
        let config = NetSuiteExportConfig::default();
        let mut exporter = NetSuiteExporter::new(config);

        let entries = vec![create_test_je()];
        let result = exporter.export_to_json(&entries, temp_dir.path());

        assert!(result.is_ok());
        assert!(temp_dir
            .path()
            .join("netsuite_journal_entries.json")
            .exists());
    }

    #[test]
    fn test_conversion_produces_balanced_totals() {
        let config = NetSuiteExportConfig::default();
        let mut exporter = NetSuiteExporter::new(config);
        let je = create_test_je();

        let (header, lines) = exporter.convert(&je);

        assert_eq!(header.total_debit, header.total_credit);
        assert_eq!(lines.len(), 2);
    }
}
