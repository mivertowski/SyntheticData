//! SAP HANA ACDOCA/BSEG compatible event log structures.
//!
//! This module defines data structures compatible with SAP S/4HANA's
//! Universal Journal (ACDOCA) and legacy document segment table (BSEG).
//! These formats are essential for testing real-time analytics and
//! process mining tools that work with SAP data.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::journal_entry::JournalEntry;

/// SAP HANA ACDOCA-compatible universal journal entry line.
///
/// This represents the flattened, denormalized structure used in S/4HANA's
/// Universal Journal. Each record corresponds to a line item with all
/// dimensional attributes denormalized for analytics performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcdocaEntry {
    // === Primary Keys ===
    /// Ledger (0L = Leading Ledger, 2L = Local GAAP, etc.)
    pub rldnr: String,
    /// Company Code
    pub rbukrs: String,
    /// Fiscal Year
    pub gjahr: u16,
    /// Accounting Document Number
    pub belnr: String,
    /// Line Item Number (6 digits, zero-padded)
    pub docln: String,

    // === Document Control ===
    /// Document Type
    pub blart: String,
    /// Posting Date
    pub budat: NaiveDate,
    /// Document Date
    pub bldat: NaiveDate,
    /// Entry Date
    pub cpudt: NaiveDate,
    /// Entry Time (HHMMSS format)
    pub cputm: String,
    /// User Name
    pub usnam: String,
    /// Fiscal Period
    pub poper: u8,
    /// Reversal Reason
    pub stgrd: Option<String>,
    /// Reference Document Number
    pub xblnr: Option<String>,
    /// Document Header Text
    pub bktxt: Option<String>,

    // === Account Assignment ===
    /// GL Account
    pub racct: String,
    /// Cost Center
    pub rcntr: Option<String>,
    /// Profit Center
    pub prctr: Option<String>,
    /// Segment
    pub segment: Option<String>,
    /// Functional Area
    pub rfarea: Option<String>,
    /// Business Area
    pub rbusa: Option<String>,
    /// Project (WBS Element)
    pub ps_psp_pnr: Option<String>,
    /// Internal Order
    pub aufnr: Option<String>,
    /// Sales Order
    pub kdauf: Option<String>,
    /// Sales Order Item
    pub kdpos: Option<String>,

    // === Partner Fields (Intercompany) ===
    /// Partner Company Code
    pub pbukrs: Option<String>,
    /// Partner Profit Center
    pub pprctr: Option<String>,
    /// Partner Segment
    pub psegment: Option<String>,
    /// Trading Partner
    pub rassc: Option<String>,

    // === Amounts ===
    /// Amount in Transaction Currency
    pub wsl: Decimal,
    /// Transaction Currency
    pub rwcur: String,
    /// Amount in Local Currency
    pub hsl: Decimal,
    /// Local Currency (Company Code Currency)
    pub rhcur: String,
    /// Amount in Group Currency
    pub ksl: Option<Decimal>,
    /// Group Currency
    pub rkcur: Option<String>,
    /// Amount in Global Currency
    pub osl: Option<Decimal>,
    /// Global Currency
    pub rocur: Option<String>,

    // === Quantities ===
    /// Quantity
    pub msl: Option<Decimal>,
    /// Unit of Measure
    pub runit: Option<String>,

    // === Text Fields ===
    /// Line Item Text
    pub sgtxt: Option<String>,
    /// Assignment
    pub zuonr: Option<String>,

    // === Source Document Reference ===
    /// Source System
    pub awsys: String,
    /// Reference Transaction Type
    pub awtyp: String,
    /// Reference Key
    pub awkey: String,
    /// Reference Item
    pub awitem: Option<String>,
    /// Source Document Type
    pub aworg: Option<String>,

    // === Tax ===
    /// Tax Code
    pub mwskz: Option<String>,
    /// Tax Jurisdiction
    pub txjcd: Option<String>,
    /// Tax Base Amount
    pub hwbas: Option<Decimal>,

    // === Control Flags ===
    /// Reversal Flag
    pub xstov: bool,
    /// Statistical Flag
    pub xsauf: bool,
    /// Debit/Credit Indicator (S = Debit, H = Credit)
    pub drcrk: String,
    /// Posting Key
    pub bschl: String,

    // === Asset Accounting ===
    /// Asset Number
    pub anln1: Option<String>,
    /// Asset Sub-Number
    pub anln2: Option<String>,
    /// Asset Transaction Type
    pub anbwa: Option<String>,

    // === Vendor/Customer ===
    /// Vendor Number
    pub lifnr: Option<String>,
    /// Customer Number
    pub kunnr: Option<String>,

    // === Extension Fields (Simulation Metadata) ===
    /// Simulation batch ID for traceability
    #[serde(rename = "ZSIM_BATCH_ID")]
    pub sim_batch_id: Option<Uuid>,
    /// Is fraud indicator
    #[serde(rename = "ZSIM_IS_FRAUD")]
    pub sim_is_fraud: bool,
    /// Fraud type code
    #[serde(rename = "ZSIM_FRAUD_TYPE")]
    pub sim_fraud_type: Option<String>,
    /// Business process for process mining
    #[serde(rename = "ZSIM_BUSINESS_PROCESS")]
    pub sim_business_process: Option<String>,
    /// User persona classification
    #[serde(rename = "ZSIM_USER_PERSONA")]
    pub sim_user_persona: Option<String>,
    /// Original journal entry UUID
    #[serde(rename = "ZSIM_JE_UUID")]
    pub sim_je_uuid: Option<Uuid>,

    // === Internal Controls / SOX Compliance Fields ===
    /// Comma-separated list of applicable control IDs
    #[serde(rename = "ZSIM_CONTROL_IDS")]
    pub sim_control_ids: Option<String>,
    /// SOX relevance indicator
    #[serde(rename = "ZSIM_SOX_RELEVANT")]
    pub sim_sox_relevant: bool,
    /// Control status (Effective, Exception, NotTested, Remediated)
    #[serde(rename = "ZSIM_CONTROL_STATUS")]
    pub sim_control_status: Option<String>,
    /// SoD violation indicator
    #[serde(rename = "ZSIM_SOD_VIOLATION")]
    pub sim_sod_violation: bool,
    /// SoD conflict type if violation occurred
    #[serde(rename = "ZSIM_SOD_CONFLICT")]
    pub sim_sod_conflict: Option<String>,
}

impl Default for AcdocaEntry {
    fn default() -> Self {
        Self {
            rldnr: "0L".to_string(),
            rbukrs: String::new(),
            gjahr: 0,
            belnr: String::new(),
            docln: String::new(),
            blart: "SA".to_string(),
            budat: NaiveDate::from_ymd_opt(2000, 1, 1).expect("valid default date"),
            bldat: NaiveDate::from_ymd_opt(2000, 1, 1).expect("valid default date"),
            cpudt: NaiveDate::from_ymd_opt(2000, 1, 1).expect("valid default date"),
            cputm: "000000".to_string(),
            usnam: String::new(),
            poper: 1,
            stgrd: None,
            xblnr: None,
            bktxt: None,
            racct: String::new(),
            rcntr: None,
            prctr: None,
            segment: None,
            rfarea: None,
            rbusa: None,
            ps_psp_pnr: None,
            aufnr: None,
            kdauf: None,
            kdpos: None,
            pbukrs: None,
            pprctr: None,
            psegment: None,
            rassc: None,
            wsl: Decimal::ZERO,
            rwcur: "USD".to_string(),
            hsl: Decimal::ZERO,
            rhcur: "USD".to_string(),
            ksl: None,
            rkcur: None,
            osl: None,
            rocur: None,
            msl: None,
            runit: None,
            sgtxt: None,
            zuonr: None,
            awsys: "SYNTH".to_string(),
            awtyp: "BKPF".to_string(),
            awkey: String::new(),
            awitem: None,
            aworg: None,
            mwskz: None,
            txjcd: None,
            hwbas: None,
            xstov: false,
            xsauf: false,
            drcrk: "S".to_string(),
            bschl: "40".to_string(),
            anln1: None,
            anln2: None,
            anbwa: None,
            lifnr: None,
            kunnr: None,
            sim_batch_id: None,
            sim_is_fraud: false,
            sim_fraud_type: None,
            sim_business_process: None,
            sim_user_persona: None,
            sim_je_uuid: None,
            // Internal Controls / SOX fields
            sim_control_ids: None,
            sim_sox_relevant: false,
            sim_control_status: None,
            sim_sod_violation: false,
            sim_sod_conflict: None,
        }
    }
}

/// SAP BSEG-compatible document segment structure.
///
/// Traditional line item table format used in ECC/R3 systems before S/4HANA.
/// Maintained for backward compatibility with legacy analytics tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BsegEntry {
    /// Client (SAP system client)
    pub mandt: String,
    /// Company Code
    pub bukrs: String,
    /// Document Number
    pub belnr: String,
    /// Fiscal Year
    pub gjahr: u16,
    /// Line Item Number
    pub buzei: String,

    // === Account Fields ===
    /// Posting Key
    pub bschl: String,
    /// GL Account
    pub hkont: String,
    /// Special GL Indicator
    pub umskz: Option<String>,

    // === Amount Fields ===
    /// Amount in Document Currency
    pub wrbtr: Decimal,
    /// Debit/Credit Indicator (S = Debit, H = Credit)
    pub shkzg: String,
    /// Amount in Local Currency
    pub dmbtr: Decimal,
    /// Local Currency
    pub waers: String,
    /// Tax Amount
    pub wmwst: Option<Decimal>,

    // === Additional Assignment ===
    /// Cost Center
    pub kostl: Option<String>,
    /// Profit Center
    pub prctr: Option<String>,
    /// Asset Number
    pub anln1: Option<String>,
    /// Asset Sub-Number
    pub anln2: Option<String>,
    /// Vendor Number
    pub lifnr: Option<String>,
    /// Customer Number
    pub kunnr: Option<String>,

    // === Text and Reference ===
    /// Line Item Text
    pub sgtxt: Option<String>,
    /// Assignment
    pub zuonr: Option<String>,
    /// Tax Code
    pub mwskz: Option<String>,
}

impl Default for BsegEntry {
    fn default() -> Self {
        Self {
            mandt: "100".to_string(),
            bukrs: String::new(),
            belnr: String::new(),
            gjahr: 0,
            buzei: String::new(),
            bschl: "40".to_string(),
            hkont: String::new(),
            umskz: None,
            wrbtr: Decimal::ZERO,
            shkzg: "S".to_string(),
            dmbtr: Decimal::ZERO,
            waers: "USD".to_string(),
            wmwst: None,
            kostl: None,
            prctr: None,
            anln1: None,
            anln2: None,
            lifnr: None,
            kunnr: None,
            sgtxt: None,
            zuonr: None,
            mwskz: None,
        }
    }
}

/// Factory for creating ACDOCA entries from journal entries.
///
/// Handles the conversion from internal journal entry format to
/// SAP HANA ACDOCA-compatible records.
#[derive(Debug, Clone)]
pub struct AcdocaFactory {
    /// Ledger identifier
    ledger: String,
    /// Source system identifier
    source_system: String,
    /// Local currency (company code currency)
    local_currency: String,
    /// Group currency (for consolidation)
    group_currency: Option<String>,
    /// SAP client number
    client: String,
}

impl AcdocaFactory {
    /// Create a new ACDOCA factory.
    pub fn new(ledger: &str, source_system: &str) -> Self {
        Self {
            ledger: ledger.to_string(),
            source_system: source_system.to_string(),
            local_currency: "USD".to_string(),
            group_currency: None,
            client: "100".to_string(),
        }
    }

    /// Set the local currency.
    pub fn with_local_currency(mut self, currency: &str) -> Self {
        self.local_currency = currency.to_string();
        self
    }

    /// Set the group currency.
    pub fn with_group_currency(mut self, currency: &str) -> Self {
        self.group_currency = Some(currency.to_string());
        self
    }

    /// Set the SAP client.
    pub fn with_client(mut self, client: &str) -> Self {
        self.client = client.to_string();
        self
    }

    /// Convert a JournalEntry into ACDOCA entries.
    pub fn from_journal_entry(&self, je: &JournalEntry, document_number: &str) -> Vec<AcdocaEntry> {
        let created_at = je.header.created_at;

        je.lines
            .iter()
            .map(|line| {
                // Determine if debit or credit
                let is_debit = line.debit_amount > Decimal::ZERO;
                let amount = if is_debit {
                    line.debit_amount
                } else {
                    line.credit_amount
                };

                // Signed amount (positive for debit, negative for credit)
                let wsl = if is_debit { amount } else { -amount };
                let hsl = wsl * je.header.exchange_rate;

                // Posting key
                let bschl = if is_debit { "40" } else { "50" };
                let drcrk = if is_debit { "S" } else { "H" };

                AcdocaEntry {
                    rldnr: self.ledger.clone(),
                    rbukrs: je.header.company_code.clone(),
                    gjahr: je.header.fiscal_year,
                    belnr: document_number.to_string(),
                    docln: format!("{:06}", line.line_number),
                    blart: je.header.document_type.clone(),
                    budat: je.header.posting_date,
                    bldat: je.header.document_date,
                    cpudt: created_at.date_naive(),
                    cputm: created_at.format("%H%M%S").to_string(),
                    usnam: je.header.created_by.clone(),
                    poper: je.header.fiscal_period,
                    stgrd: None,
                    xblnr: je.header.reference.clone(),
                    bktxt: je.header.header_text.clone(),
                    racct: line.gl_account.clone(),
                    rcntr: line.cost_center.clone(),
                    prctr: line.profit_center.clone(),
                    segment: line.segment.clone(),
                    rfarea: line.functional_area.clone(),
                    rbusa: None,
                    ps_psp_pnr: None,
                    aufnr: None,
                    kdauf: None,
                    kdpos: None,
                    pbukrs: None,
                    pprctr: None,
                    psegment: None,
                    rassc: line.trading_partner.clone(),
                    wsl,
                    rwcur: je.header.currency.clone(),
                    hsl,
                    rhcur: self.local_currency.clone(),
                    ksl: self.group_currency.as_ref().map(|_| hsl),
                    rkcur: self.group_currency.clone(),
                    osl: None,
                    rocur: None,
                    msl: line.quantity,
                    runit: line.unit_of_measure.clone(),
                    sgtxt: line.line_text.clone(),
                    zuonr: line.assignment.clone(),
                    awsys: self.source_system.clone(),
                    awtyp: "BKPF".to_string(),
                    awkey: format!(
                        "{}{}{}",
                        je.header.company_code, document_number, je.header.fiscal_year
                    ),
                    awitem: Some(format!("{:06}", line.line_number)),
                    aworg: None,
                    mwskz: line.tax_code.clone(),
                    txjcd: None,
                    hwbas: line.tax_amount,
                    xstov: matches!(
                        je.header.source,
                        super::journal_entry::TransactionSource::Reversal
                    ),
                    xsauf: matches!(
                        je.header.source,
                        super::journal_entry::TransactionSource::Statistical
                    ),
                    drcrk: drcrk.to_string(),
                    bschl: bschl.to_string(),
                    anln1: None,
                    anln2: None,
                    anbwa: None,
                    lifnr: None,
                    kunnr: None,
                    sim_batch_id: je.header.batch_id,
                    sim_is_fraud: je.header.is_fraud,
                    sim_fraud_type: je.header.fraud_type.map(|ft| format!("{:?}", ft)),
                    sim_business_process: je.header.business_process.map(|bp| format!("{:?}", bp)),
                    sim_user_persona: Some(je.header.user_persona.clone()),
                    sim_je_uuid: Some(je.header.document_id),
                    sim_control_ids: if je.header.control_ids.is_empty() {
                        None
                    } else {
                        Some(je.header.control_ids.join(","))
                    },
                    sim_sox_relevant: je.header.sox_relevant,
                    sim_control_status: Some(je.header.control_status.to_string()),
                    sim_sod_violation: je.header.sod_violation,
                    sim_sod_conflict: je.header.sod_conflict_type.map(|t| t.to_string()),
                }
            })
            .collect()
    }

    /// Convert a JournalEntry into BSEG entries.
    pub fn to_bseg_entries(&self, je: &JournalEntry, document_number: &str) -> Vec<BsegEntry> {
        je.lines
            .iter()
            .map(|line| {
                let is_debit = line.debit_amount > Decimal::ZERO;
                let amount = if is_debit {
                    line.debit_amount
                } else {
                    line.credit_amount
                };

                BsegEntry {
                    mandt: self.client.clone(),
                    bukrs: je.header.company_code.clone(),
                    belnr: document_number.to_string(),
                    gjahr: je.header.fiscal_year,
                    buzei: format!("{:03}", line.line_number),
                    bschl: if is_debit { "40" } else { "50" }.to_string(),
                    hkont: line.gl_account.clone(),
                    umskz: None,
                    wrbtr: amount,
                    shkzg: if is_debit { "S" } else { "H" }.to_string(),
                    dmbtr: line.local_amount.abs(),
                    waers: je.header.currency.clone(),
                    wmwst: line.tax_amount,
                    kostl: line.cost_center.clone(),
                    prctr: line.profit_center.clone(),
                    anln1: None,
                    anln2: None,
                    lifnr: None,
                    kunnr: None,
                    sgtxt: line.line_text.clone(),
                    zuonr: line.assignment.clone(),
                    mwskz: line.tax_code.clone(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::journal_entry::{JournalEntryHeader, JournalEntryLine};

    #[test]
    fn test_acdoca_factory_conversion() {
        let header = JournalEntryHeader::new(
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
        );
        let mut je = JournalEntry::new(header);

        je.add_line(JournalEntryLine::debit(
            je.header.document_id,
            1,
            "100000".to_string(),
            Decimal::from(5000),
        ));
        je.add_line(JournalEntryLine::credit(
            je.header.document_id,
            2,
            "200000".to_string(),
            Decimal::from(5000),
        ));

        let factory = AcdocaFactory::new("0L", "SYNTH");
        let acdoca_entries = factory.from_journal_entry(&je, "0000000001");

        assert_eq!(acdoca_entries.len(), 2);
        assert_eq!(acdoca_entries[0].racct, "100000");
        assert_eq!(acdoca_entries[0].drcrk, "S");
        assert_eq!(acdoca_entries[0].wsl, Decimal::from(5000));
        assert_eq!(acdoca_entries[1].racct, "200000");
        assert_eq!(acdoca_entries[1].drcrk, "H");
        assert_eq!(acdoca_entries[1].wsl, Decimal::from(-5000));
    }
}
