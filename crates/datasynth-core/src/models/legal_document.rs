//! Legal document models for audit engagement support.
//!
//! Legal documents are produced and consumed during audit engagements,
//! covering engagement letters, management representation letters,
//! legal opinions, regulatory filings, and board resolutions.
//! Referenced by 686 GAM procedures.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A legal document associated with an audit engagement.
///
/// Examples include engagement letters (ISA 210), management
/// representation letters (ISA 580), legal opinions, regulatory
/// filings, and board resolutions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalDocument {
    /// Unique identifier for this document.
    pub document_id: Uuid,
    /// Document type classification.
    ///
    /// Common values: `engagement_letter`, `management_rep`,
    /// `legal_opinion`, `regulatory_filing`, `board_resolution`.
    pub document_type: String,
    /// Entity code this document belongs to.
    pub entity_code: String,
    /// Date the document was issued or signed.
    pub date: NaiveDate,
    /// Human-readable title.
    pub title: String,
    /// Names or roles of signatories.
    #[serde(default)]
    pub signatories: Vec<String>,
    /// Key contractual or regulatory terms.
    #[serde(default)]
    pub key_terms: Vec<String>,
    /// Current lifecycle status.
    ///
    /// Common values: `draft`, `final`, `signed`, `expired`.
    pub status: String,
}
