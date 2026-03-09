//! Regulatory filing models.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Type of regulatory filing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingType {
    // US filings
    /// SEC Form 10-K (Annual Report)
    Form10K,
    /// SEC Form 10-Q (Quarterly Report)
    Form10Q,
    /// SEC Form 8-K (Current Report)
    Form8K,
    /// SEC Form 20-F (Annual Report for Foreign Private Issuers)
    Form20F,

    // EU / German filings
    /// German annual financial statements (Jahresabschluss)
    Jahresabschluss,
    /// German electronic balance sheet (E-Bilanz)
    EBilanz,

    // French filings
    /// French fiscal package (Liasse fiscale)
    LiasseFiscale,

    // UK filings
    /// UK annual return to Companies House
    UkAnnualReturn,
    /// UK corporation tax return
    Ct600,

    // Japanese filings
    /// Japanese securities report (有価証券報告書)
    YukaShokenHokokusho,

    // Generic
    /// Annual financial statements (generic)
    AnnualStatements,
    /// Quarterly report (generic)
    QuarterlyReport,
    /// Tax return (generic)
    TaxReturn,
    /// Custom filing type
    Custom(String),
}

impl std::fmt::Display for FilingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Form10K => write!(f, "10-K"),
            Self::Form10Q => write!(f, "10-Q"),
            Self::Form8K => write!(f, "8-K"),
            Self::Form20F => write!(f, "20-F"),
            Self::Jahresabschluss => write!(f, "Jahresabschluss"),
            Self::EBilanz => write!(f, "E-Bilanz"),
            Self::LiasseFiscale => write!(f, "Liasse fiscale"),
            Self::UkAnnualReturn => write!(f, "Annual Return"),
            Self::Ct600 => write!(f, "CT600"),
            Self::YukaShokenHokokusho => write!(f, "有価証券報告書"),
            Self::AnnualStatements => write!(f, "Annual Financial Statements"),
            Self::QuarterlyReport => write!(f, "Quarterly Report"),
            Self::TaxReturn => write!(f, "Tax Return"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

/// Filing frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingFrequency {
    /// Filed annually
    Annual,
    /// Filed semi-annually
    SemiAnnual,
    /// Filed quarterly
    Quarterly,
    /// Filed monthly
    Monthly,
    /// Filed on occurrence of event
    EventDriven,
}

/// Filing status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    /// Not yet due
    NotDue,
    /// Pending preparation
    Pending,
    /// Filed on time
    Filed,
    /// Filed late
    FiledLate,
    /// Overdue
    Overdue,
}

/// A regulatory filing requirement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilingRequirement {
    /// Filing type
    pub filing_type: FilingType,
    /// Filing frequency
    pub frequency: FilingFrequency,
    /// Regulator receiving the filing
    pub regulator: String,
    /// Jurisdiction (ISO 3166-1 alpha-2)
    pub jurisdiction: String,
    /// Days after period end by which filing is due
    pub deadline_days: u32,
    /// Whether electronic filing is required
    pub electronic_filing: bool,
    /// Whether XBRL tagging is required
    pub xbrl_required: bool,
}

/// A specific regulatory filing instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryFiling {
    /// Filing type
    pub filing_type: FilingType,
    /// Company code
    pub company_code: String,
    /// Jurisdiction
    pub jurisdiction: String,
    /// Period end date
    pub period_end: NaiveDate,
    /// Filing deadline
    pub deadline: NaiveDate,
    /// Actual filing date (if filed)
    pub filing_date: Option<NaiveDate>,
    /// Filing status
    pub status: FilingStatus,
    /// Regulator
    pub regulator: String,
    /// Electronic filing reference
    pub filing_reference: Option<String>,
}

impl RegulatoryFiling {
    /// Creates a new filing instance.
    pub fn new(
        filing_type: FilingType,
        company_code: impl Into<String>,
        jurisdiction: impl Into<String>,
        period_end: NaiveDate,
        deadline: NaiveDate,
        regulator: impl Into<String>,
    ) -> Self {
        Self {
            filing_type,
            company_code: company_code.into(),
            jurisdiction: jurisdiction.into(),
            period_end,
            deadline,
            filing_date: None,
            status: FilingStatus::Pending,
            regulator: regulator.into(),
            filing_reference: None,
        }
    }

    /// Marks as filed.
    pub fn filed_on(mut self, date: NaiveDate) -> Self {
        self.filing_date = Some(date);
        self.status = if date <= self.deadline {
            FilingStatus::Filed
        } else {
            FilingStatus::FiledLate
        };
        self
    }

    /// Returns the number of days until/past the deadline from a given date.
    pub fn days_to_deadline(&self, from: NaiveDate) -> i64 {
        (self.deadline - from).num_days()
    }
}
