//! Notes to financial statements models.
//!
//! Provides structured data models for the disclosures that accompany the
//! primary financial statements.  Notes are required by virtually every
//! financial reporting framework (IFRS IAS 1, ASC 235, etc.) and include
//! accounting policies, detail disclosures, contingencies, subsequent events,
//! related parties, segment information, and standard-specific items.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// High-level category of a note to the financial statements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteCategory {
    /// Summary of significant accounting policies (IAS 1.117 / ASC 235-10).
    #[default]
    AccountingPolicy,
    /// Detailed breakdown supporting a line item on the face of the statements.
    DetailDisclosure,
    /// Provisions, contingent liabilities, and contingent assets (IAS 37 / ASC 450).
    Contingency,
    /// Events after the reporting period (IAS 10 / ASC 855).
    SubsequentEvent,
    /// Related party transactions and balances (IAS 24 / ASC 850).
    RelatedParty,
    /// Operating segment disclosures (IFRS 8 / ASC 280).
    SegmentInformation,
    /// Disclosures specific to a particular accounting standard.
    StandardSpecific,
}

/// A typed cell value inside a disclosure table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum NoteTableValue {
    /// Plain text cell.
    Text(String),
    /// Monetary or numeric amount.
    Amount(#[serde(with = "rust_decimal::serde::str")] Decimal),
    /// Percentage value (stored as a fraction, e.g. 0.25 = 25 %).
    Percentage(#[serde(with = "rust_decimal::serde::str")] Decimal),
    /// Date cell.
    Date(NaiveDate),
    /// Empty / not applicable cell.
    Empty,
}

// ---------------------------------------------------------------------------
// Note building blocks
// ---------------------------------------------------------------------------

/// A two-dimensional disclosure table within a note section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteTable {
    /// Brief caption describing the table's content.
    pub caption: String,
    /// Column headers.
    pub headers: Vec<String>,
    /// Data rows — each row has the same length as `headers`.
    pub rows: Vec<Vec<NoteTableValue>>,
}

/// A single section within a note (heading + narrative + optional tables).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSection {
    /// Section heading (e.g. "Basis of preparation").
    pub heading: String,
    /// Explanatory narrative text.
    pub narrative: String,
    /// Optional tabular data supporting the narrative.
    pub tables: Vec<NoteTable>,
}

// ---------------------------------------------------------------------------
// Top-level note
// ---------------------------------------------------------------------------

/// A complete note to the financial statements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialStatementNote {
    /// Sequential note number (1 = first note in the set).
    pub note_number: u32,
    /// Short descriptive title.
    pub title: String,
    /// Disclosure category.
    pub category: NoteCategory,
    /// One or more content sections within this note.
    pub content_sections: Vec<NoteSection>,
    /// Cross-references to other notes or financial statement line items.
    pub cross_references: Vec<String>,
}

impl FinancialStatementNote {
    /// Create a minimal note with a single section and no tables.
    pub fn simple(
        note_number: u32,
        title: impl Into<String>,
        category: NoteCategory,
        heading: impl Into<String>,
        narrative: impl Into<String>,
    ) -> Self {
        Self {
            note_number,
            title: title.into(),
            category,
            content_sections: vec![NoteSection {
                heading: heading.into(),
                narrative: narrative.into(),
                tables: Vec::new(),
            }],
            cross_references: Vec::new(),
        }
    }

    /// Attach a cross-reference to another note.
    pub fn with_cross_reference(mut self, reference: impl Into<String>) -> Self {
        self.cross_references.push(reference.into());
        self
    }
}
