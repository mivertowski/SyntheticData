//! SAP S/4HANA format export.
//!
//! Exports data in SAP-compatible table formats:
//! - BKPF (Document Header)
//! - BSEG (Document Segments/Line Items)
//! - ACDOCA (Universal Journal)

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use datasynth_core::error::SynthResult;
use datasynth_core::models::{AcdocaFactory, JournalEntry};

/// SAP table types for export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SapTableType {
    /// Document header table
    Bkpf,
    /// Document segment (line item) table
    Bseg,
    /// Universal journal (S/4HANA)
    Acdoca,
    /// Vendor master
    Lfa1,
    /// Customer master
    Kna1,
    /// Material master
    Mara,
    /// Cost center master
    Csks,
    /// Profit center master
    Cepc,
}

impl SapTableType {
    /// Get the SAP table name.
    pub fn table_name(&self) -> &'static str {
        match self {
            SapTableType::Bkpf => "BKPF",
            SapTableType::Bseg => "BSEG",
            SapTableType::Acdoca => "ACDOCA",
            SapTableType::Lfa1 => "LFA1",
            SapTableType::Kna1 => "KNA1",
            SapTableType::Mara => "MARA",
            SapTableType::Csks => "CSKS",
            SapTableType::Cepc => "CEPC",
        }
    }
}

/// SAP BKPF (Document Header) record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BkpfEntry {
    /// Client
    pub mandt: String,
    /// Company Code
    pub bukrs: String,
    /// Document Number
    pub belnr: String,
    /// Fiscal Year
    pub gjahr: u16,
    /// Document Type
    pub blart: String,
    /// Document Date
    pub bldat: NaiveDate,
    /// Posting Date
    pub budat: NaiveDate,
    /// Fiscal Period
    pub monat: u8,
    /// Entry Date
    pub cpudt: NaiveDate,
    /// Entry Time
    pub cputm: String,
    /// User Name
    pub usnam: String,
    /// Transaction Code
    pub tcode: String,
    /// Reference Number
    pub xblnr: Option<String>,
    /// Document Header Text
    pub bktxt: Option<String>,
    /// Transaction Currency
    pub waers: String,
    /// Exchange Rate
    pub kursf: Decimal,
    /// Cross-Company Code
    pub bvorg: Option<String>,
    /// Reversal Document Number
    pub stblg: Option<String>,
    /// Reversal Reason
    pub stgrd: Option<String>,
}

impl Default for BkpfEntry {
    fn default() -> Self {
        Self {
            mandt: "100".to_string(),
            bukrs: String::new(),
            belnr: String::new(),
            gjahr: 0,
            blart: "SA".to_string(),
            bldat: NaiveDate::from_ymd_opt(2000, 1, 1).expect("valid default date"),
            budat: NaiveDate::from_ymd_opt(2000, 1, 1).expect("valid default date"),
            monat: 1,
            cpudt: NaiveDate::from_ymd_opt(2000, 1, 1).expect("valid default date"),
            cputm: "000000".to_string(),
            usnam: String::new(),
            tcode: "FB01".to_string(),
            xblnr: None,
            bktxt: None,
            waers: "USD".to_string(),
            kursf: Decimal::ONE,
            bvorg: None,
            stblg: None,
            stgrd: None,
        }
    }
}

/// Configuration for SAP export.
#[derive(Debug, Clone)]
pub struct SapExportConfig {
    /// SAP client number
    pub client: String,
    /// Ledger for ACDOCA
    pub ledger: String,
    /// Source system identifier
    pub source_system: String,
    /// Local currency
    pub local_currency: String,
    /// Group currency (optional)
    pub group_currency: Option<String>,
    /// Tables to export
    pub tables: Vec<SapTableType>,
    /// Include extension fields (ZSIM_*)
    pub include_extension_fields: bool,
    /// Date format (SAP internal: YYYYMMDD)
    pub use_sap_date_format: bool,
}

impl Default for SapExportConfig {
    fn default() -> Self {
        Self {
            client: "100".to_string(),
            ledger: "0L".to_string(),
            source_system: "SYNTH".to_string(),
            local_currency: "USD".to_string(),
            group_currency: None,
            tables: vec![SapTableType::Bkpf, SapTableType::Bseg, SapTableType::Acdoca],
            include_extension_fields: true,
            use_sap_date_format: true,
        }
    }
}

/// SAP format exporter.
pub struct SapExporter {
    config: SapExportConfig,
    acdoca_factory: AcdocaFactory,
    document_counter: HashMap<String, u64>, // Company code -> counter
}

impl SapExporter {
    /// Create a new SAP exporter.
    pub fn new(config: SapExportConfig) -> Self {
        let mut acdoca_factory = AcdocaFactory::new(&config.ledger, &config.source_system)
            .with_local_currency(&config.local_currency)
            .with_client(&config.client);

        if let Some(ref group_currency) = config.group_currency {
            acdoca_factory = acdoca_factory.with_group_currency(group_currency);
        }

        Self {
            config,
            acdoca_factory,
            document_counter: HashMap::new(),
        }
    }

    /// Generate next document number for a company code.
    fn next_document_number(&mut self, company_code: &str) -> String {
        let counter = self
            .document_counter
            .entry(company_code.to_string())
            .or_insert(0);
        *counter += 1;
        format!("{:010}", *counter)
    }

    /// Convert JournalEntry to BKPF record.
    pub fn to_bkpf(&self, je: &JournalEntry, document_number: &str) -> BkpfEntry {
        BkpfEntry {
            mandt: self.config.client.clone(),
            bukrs: je.header.company_code.clone(),
            belnr: document_number.to_string(),
            gjahr: je.header.fiscal_year,
            blart: je.header.document_type.clone(),
            bldat: je.header.document_date,
            budat: je.header.posting_date,
            monat: je.header.fiscal_period,
            cpudt: je.header.created_at.date_naive(),
            cputm: je.header.created_at.format("%H%M%S").to_string(),
            usnam: je.header.created_by.clone(),
            tcode: self.get_transaction_code(je),
            xblnr: je.header.reference.clone(),
            bktxt: je.header.header_text.clone(),
            waers: je.header.currency.clone(),
            kursf: je.header.exchange_rate,
            bvorg: None,
            stblg: None,
            stgrd: None,
        }
    }

    /// Get appropriate transaction code based on document type.
    fn get_transaction_code(&self, je: &JournalEntry) -> String {
        match je.header.document_type.as_str() {
            "SA" => "FB01".to_string(),  // GL posting
            "RE" => "MIRO".to_string(),  // Vendor invoice
            "RV" => "VF01".to_string(),  // Customer invoice
            "KZ" => "F110".to_string(),  // Vendor payment
            "DZ" => "F28".to_string(),   // Customer payment
            "AB" => "ABZON".to_string(), // Depreciation
            "AA" => "ABSO1".to_string(), // Asset acquisition
            _ => "FB01".to_string(),
        }
    }

    /// Export journal entries to SAP format files.
    pub fn export_to_files(
        &mut self,
        entries: &[JournalEntry],
        output_dir: &Path,
    ) -> SynthResult<HashMap<SapTableType, String>> {
        let mut output_files = HashMap::new();

        // Create output directory if needed
        std::fs::create_dir_all(output_dir)?;

        // Clone tables to avoid borrow checker issues
        let tables = self.config.tables.clone();
        for table_type in tables {
            let filename = format!("{}.csv", table_type.table_name().to_lowercase());
            let filepath = output_dir.join(&filename);

            match table_type {
                SapTableType::Bkpf => self.export_bkpf(entries, &filepath)?,
                SapTableType::Bseg => self.export_bseg(entries, &filepath)?,
                SapTableType::Acdoca => self.export_acdoca(entries, &filepath)?,
                _ => {
                    // Master data tables are handled separately
                    continue;
                }
            }

            output_files.insert(table_type, filepath.to_string_lossy().to_string());
        }

        Ok(output_files)
    }

    /// Export BKPF (document headers).
    fn export_bkpf(&mut self, entries: &[JournalEntry], filepath: &Path) -> SynthResult<()> {
        let file = File::create(filepath)?;
        let mut writer = BufWriter::new(file);

        // Write header
        writeln!(
            writer,
            "MANDT,BUKRS,BELNR,GJAHR,BLART,BLDAT,BUDAT,MONAT,CPUDT,CPUTM,USNAM,TCODE,XBLNR,BKTXT,WAERS,KURSF"
        )?;

        for je in entries {
            let doc_num = self.next_document_number(&je.header.company_code);
            let bkpf = self.to_bkpf(je, &doc_num);

            let bldat = if self.config.use_sap_date_format {
                bkpf.bldat.format("%Y%m%d").to_string()
            } else {
                bkpf.bldat.to_string()
            };
            let budat = if self.config.use_sap_date_format {
                bkpf.budat.format("%Y%m%d").to_string()
            } else {
                bkpf.budat.to_string()
            };
            let cpudt = if self.config.use_sap_date_format {
                bkpf.cpudt.format("%Y%m%d").to_string()
            } else {
                bkpf.cpudt.to_string()
            };

            writeln!(
                writer,
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                bkpf.mandt,
                bkpf.bukrs,
                bkpf.belnr,
                bkpf.gjahr,
                bkpf.blart,
                bldat,
                budat,
                bkpf.monat,
                cpudt,
                bkpf.cputm,
                bkpf.usnam,
                bkpf.tcode,
                bkpf.xblnr.as_deref().unwrap_or(""),
                escape_csv_field(bkpf.bktxt.as_deref().unwrap_or("")),
                bkpf.waers,
                bkpf.kursf,
            )?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Export BSEG (document segments).
    fn export_bseg(&mut self, entries: &[JournalEntry], filepath: &Path) -> SynthResult<()> {
        let file = File::create(filepath)?;
        let mut writer = BufWriter::new(file);

        // Write header
        writeln!(
            writer,
            "MANDT,BUKRS,BELNR,GJAHR,BUZEI,BSCHL,HKONT,WRBTR,SHKZG,DMBTR,WAERS,KOSTL,PRCTR,SGTXT,ZUONR,MWSKZ"
        )?;

        // Reset document counter for BSEG to match BKPF
        self.document_counter.clear();

        for je in entries {
            let doc_num = self.next_document_number(&je.header.company_code);
            let bseg_entries = self.acdoca_factory.to_bseg_entries(je, &doc_num);

            for bseg in bseg_entries {
                writeln!(
                    writer,
                    "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                    bseg.mandt,
                    bseg.bukrs,
                    bseg.belnr,
                    bseg.gjahr,
                    bseg.buzei,
                    bseg.bschl,
                    bseg.hkont,
                    bseg.wrbtr,
                    bseg.shkzg,
                    bseg.dmbtr,
                    bseg.waers,
                    bseg.kostl.as_deref().unwrap_or(""),
                    bseg.prctr.as_deref().unwrap_or(""),
                    escape_csv_field(bseg.sgtxt.as_deref().unwrap_or("")),
                    bseg.zuonr.as_deref().unwrap_or(""),
                    bseg.mwskz.as_deref().unwrap_or(""),
                )?;
            }
        }

        writer.flush()?;
        Ok(())
    }

    /// Export ACDOCA (Universal Journal).
    fn export_acdoca(&mut self, entries: &[JournalEntry], filepath: &Path) -> SynthResult<()> {
        let file = File::create(filepath)?;
        let mut writer = BufWriter::new(file);

        // Write header - includes extension fields if configured
        let mut header =
            "RLDNR,RBUKRS,GJAHR,BELNR,DOCLN,BLART,BUDAT,BLDAT,CPUDT,CPUTM,USNAM,POPER,\
            RACCT,RCNTR,PRCTR,WSL,RWCUR,HSL,RHCUR,DRCRK,BSCHL,SGTXT,ZUONR,AWSYS,AWTYP,AWKEY"
                .to_string();

        if self.config.include_extension_fields {
            header.push_str(
                ",ZSIM_BATCH_ID,ZSIM_IS_FRAUD,ZSIM_FRAUD_TYPE,ZSIM_BUSINESS_PROCESS,\
                ZSIM_CONTROL_IDS,ZSIM_SOX_RELEVANT,ZSIM_SOD_VIOLATION",
            );
        }
        writeln!(writer, "{}", header)?;

        // Reset document counter for ACDOCA
        self.document_counter.clear();

        for je in entries {
            let doc_num = self.next_document_number(&je.header.company_code);
            let acdoca_entries = self.acdoca_factory.from_journal_entry(je, &doc_num);

            for entry in acdoca_entries {
                let budat = if self.config.use_sap_date_format {
                    entry.budat.format("%Y%m%d").to_string()
                } else {
                    entry.budat.to_string()
                };
                let bldat = if self.config.use_sap_date_format {
                    entry.bldat.format("%Y%m%d").to_string()
                } else {
                    entry.bldat.to_string()
                };
                let cpudt = if self.config.use_sap_date_format {
                    entry.cpudt.format("%Y%m%d").to_string()
                } else {
                    entry.cpudt.to_string()
                };

                let mut line = format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                    entry.rldnr,
                    entry.rbukrs,
                    entry.gjahr,
                    entry.belnr,
                    entry.docln,
                    entry.blart,
                    budat,
                    bldat,
                    cpudt,
                    entry.cputm,
                    entry.usnam,
                    entry.poper,
                    entry.racct,
                    entry.rcntr.as_deref().unwrap_or(""),
                    entry.prctr.as_deref().unwrap_or(""),
                    entry.wsl,
                    entry.rwcur,
                    entry.hsl,
                    entry.rhcur,
                    entry.drcrk,
                    entry.bschl,
                    escape_csv_field(entry.sgtxt.as_deref().unwrap_or("")),
                    entry.zuonr.as_deref().unwrap_or(""),
                    entry.awsys,
                    entry.awtyp,
                    entry.awkey,
                );

                if self.config.include_extension_fields {
                    line.push_str(&format!(
                        ",{},{},{},{},{},{},{}",
                        entry
                            .sim_batch_id
                            .map(|u| u.to_string())
                            .unwrap_or_default(),
                        entry.sim_is_fraud,
                        entry.sim_fraud_type.as_deref().unwrap_or(""),
                        entry.sim_business_process.as_deref().unwrap_or(""),
                        entry.sim_control_ids.as_deref().unwrap_or(""),
                        entry.sim_sox_relevant,
                        entry.sim_sod_violation,
                    ));
                }

                writeln!(writer, "{}", line)?;
            }
        }

        writer.flush()?;
        Ok(())
    }

    /// Export vendor master data (LFA1 format).
    pub fn export_vendor_master<V: SapVendorExportable>(
        &self,
        vendors: &[V],
        filepath: &Path,
    ) -> SynthResult<()> {
        let file = File::create(filepath)?;
        let mut writer = BufWriter::new(file);

        writeln!(
            writer,
            "MANDT,LIFNR,LAND1,NAME1,NAME2,ORT01,PSTLZ,STRAS,REGIO,SPRAS,STCD1,KTOKK"
        )?;

        for vendor in vendors {
            let v = vendor.to_sap_vendor(&self.config.client);
            writeln!(
                writer,
                "{},{},{},{},{},{},{},{},{},{},{},{}",
                v.mandt,
                v.lifnr,
                v.land1,
                escape_csv_field(&v.name1),
                escape_csv_field(&v.name2.unwrap_or_default()),
                escape_csv_field(&v.ort01.unwrap_or_default()),
                v.pstlz.as_deref().unwrap_or(""),
                escape_csv_field(&v.stras.unwrap_or_default()),
                v.regio.as_deref().unwrap_or(""),
                v.spras,
                v.stcd1.as_deref().unwrap_or(""),
                v.ktokk,
            )?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Export customer master data (KNA1 format).
    pub fn export_customer_master<C: SapCustomerExportable>(
        &self,
        customers: &[C],
        filepath: &Path,
    ) -> SynthResult<()> {
        let file = File::create(filepath)?;
        let mut writer = BufWriter::new(file);

        writeln!(
            writer,
            "MANDT,KUNNR,LAND1,NAME1,NAME2,ORT01,PSTLZ,STRAS,REGIO,SPRAS,STCD1,KTOKD"
        )?;

        for customer in customers {
            let c = customer.to_sap_customer(&self.config.client);
            writeln!(
                writer,
                "{},{},{},{},{},{},{},{},{},{},{},{}",
                c.mandt,
                c.kunnr,
                c.land1,
                escape_csv_field(&c.name1),
                escape_csv_field(&c.name2.unwrap_or_default()),
                escape_csv_field(&c.ort01.unwrap_or_default()),
                c.pstlz.as_deref().unwrap_or(""),
                escape_csv_field(&c.stras.unwrap_or_default()),
                c.regio.as_deref().unwrap_or(""),
                c.spras,
                c.stcd1.as_deref().unwrap_or(""),
                c.ktokd,
            )?;
        }

        writer.flush()?;
        Ok(())
    }
}

/// SAP vendor record (LFA1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SapVendor {
    pub mandt: String,
    pub lifnr: String,
    pub land1: String,
    pub name1: String,
    pub name2: Option<String>,
    pub ort01: Option<String>,
    pub pstlz: Option<String>,
    pub stras: Option<String>,
    pub regio: Option<String>,
    pub spras: String,
    pub stcd1: Option<String>,
    pub ktokk: String,
}

/// SAP customer record (KNA1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SapCustomer {
    pub mandt: String,
    pub kunnr: String,
    pub land1: String,
    pub name1: String,
    pub name2: Option<String>,
    pub ort01: Option<String>,
    pub pstlz: Option<String>,
    pub stras: Option<String>,
    pub regio: Option<String>,
    pub spras: String,
    pub stcd1: Option<String>,
    pub ktokd: String,
}

/// Trait for exporting entities as SAP vendors.
pub trait SapVendorExportable {
    fn to_sap_vendor(&self, client: &str) -> SapVendor;
}

/// Trait for exporting entities as SAP customers.
pub trait SapCustomerExportable {
    fn to_sap_customer(&self, client: &str) -> SapCustomer;
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
    fn test_sap_exporter_creates_files() {
        let temp_dir = TempDir::new().unwrap();
        let config = SapExportConfig::default();
        let mut exporter = SapExporter::new(config);

        let entries = vec![create_test_je()];
        let result = exporter.export_to_files(&entries, temp_dir.path());

        assert!(result.is_ok());
        let files = result.unwrap();
        assert!(files.contains_key(&SapTableType::Bkpf));
        assert!(files.contains_key(&SapTableType::Bseg));
        assert!(files.contains_key(&SapTableType::Acdoca));

        // Verify files exist
        assert!(temp_dir.path().join("bkpf.csv").exists());
        assert!(temp_dir.path().join("bseg.csv").exists());
        assert!(temp_dir.path().join("acdoca.csv").exists());
    }

    #[test]
    fn test_bkpf_conversion() {
        let config = SapExportConfig::default();
        let exporter = SapExporter::new(config);
        let je = create_test_je();

        let bkpf = exporter.to_bkpf(&je, "0000000001");

        assert_eq!(bkpf.bukrs, "1000");
        assert_eq!(bkpf.belnr, "0000000001");
        assert_eq!(bkpf.gjahr, je.header.fiscal_year);
    }

    #[test]
    fn test_document_number_generation() {
        let config = SapExportConfig::default();
        let mut exporter = SapExporter::new(config);

        let num1 = exporter.next_document_number("1000");
        let num2 = exporter.next_document_number("1000");
        let num3 = exporter.next_document_number("2000");

        assert_eq!(num1, "0000000001");
        assert_eq!(num2, "0000000002");
        assert_eq!(num3, "0000000001"); // Different company code
    }
}
