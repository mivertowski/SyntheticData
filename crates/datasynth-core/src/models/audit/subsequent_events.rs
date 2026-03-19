//! Subsequent events models per ISA 560 and IAS 10.
//!
//! Subsequent events are events that occur between the balance sheet date and
//! the date when the financial statements are authorised for issue.  IAS 10
//! distinguishes between adjusting events (that provide evidence of conditions
//! at the balance sheet date) and non-adjusting events (that arise after the
//! balance sheet date).

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A subsequent event identified during the completion phase of an audit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsequentEvent {
    /// Unique identifier for this event
    pub id: String,
    /// Entity code of the reporting entity
    pub entity_code: String,
    /// Date the event occurred
    pub event_date: NaiveDate,
    /// Date the event was discovered by the auditor
    pub discovery_date: NaiveDate,
    /// Type of subsequent event
    pub event_type: SubsequentEventType,
    /// Classification per IAS 10 (adjusting or non-adjusting)
    pub classification: EventClassification,
    /// Narrative description of the event
    pub description: String,
    /// Financial impact, if quantifiable (adjusting events or disclosed amounts)
    #[serde(
        with = "rust_decimal::serde::str_option",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub financial_impact: Option<Decimal>,
    /// Whether a disclosure in the notes is required
    pub disclosure_required: bool,
    /// IDs of adjustment journal entries raised for this event (future enhancement)
    pub adjustment_entry_ids: Vec<String>,
}

/// Type of subsequent event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SubsequentEventType {
    /// Settlement of litigation after period-end
    #[default]
    LitigationSettlement,
    /// Customer bankruptcy or insolvency after period-end
    CustomerBankruptcy,
    /// Material impairment of an asset after period-end
    AssetImpairment,
    /// Announcement of a restructuring programme
    RestructuringAnnouncement,
    /// Natural disaster affecting operations or assets
    NaturalDisaster,
    /// Significant regulatory change affecting the entity
    RegulatoryChange,
    /// Announcement of a merger or acquisition
    MergerAnnouncement,
    /// Declaration of dividends after period-end
    DividendDeclaration,
}

/// Classification of a subsequent event per IAS 10.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EventClassification {
    /// Adjusting event — provides evidence of conditions existing at the balance sheet date
    /// (IAS 10.8).  The financial statements are adjusted to reflect the event.
    #[default]
    Adjusting,
    /// Non-adjusting event — arises after the balance sheet date.
    /// Disclosure in the notes is required if material (IAS 10.21).
    NonAdjusting,
}

impl SubsequentEvent {
    /// Create a new subsequent event.
    pub fn new(
        entity_code: impl Into<String>,
        event_date: NaiveDate,
        discovery_date: NaiveDate,
        event_type: SubsequentEventType,
        classification: EventClassification,
        description: impl Into<String>,
    ) -> Self {
        let disclosure_required = matches!(classification, EventClassification::NonAdjusting);
        Self {
            id: Uuid::new_v4().to_string(),
            entity_code: entity_code.into(),
            event_date,
            discovery_date,
            event_type,
            classification,
            description: description.into(),
            financial_impact: None,
            disclosure_required,
            adjustment_entry_ids: Vec::new(),
        }
    }

    /// Attach a financial impact amount.
    pub fn with_financial_impact(mut self, impact: Decimal) -> Self {
        self.financial_impact = Some(impact);
        self
    }

    /// Mark adjustment entry IDs for this event.
    pub fn with_adjustment_entries(mut self, ids: Vec<String>) -> Self {
        self.adjustment_entry_ids = ids;
        self
    }
}
