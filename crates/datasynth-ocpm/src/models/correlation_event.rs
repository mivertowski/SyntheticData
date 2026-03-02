//! Correlation events for OCEL 2.0 enrichment.
//!
//! Correlation events represent cross-process linking activities such as
//! three-way matching, payment allocation, intercompany elimination, and
//! bank reconciliation. Each correlation event ties together multiple
//! objects from different process streams.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{EventObjectRef, ObjectQualifier};

/// The kind of cross-process correlation this event represents.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CorrelationEventType {
    /// PO / Goods Receipt / Invoice three-way match.
    ThreeWayMatch,
    /// Payment allocated against one or more invoices.
    PaymentAllocation,
    /// Intercompany elimination entry.
    IntercompanyElimination,
    /// Bank statement line matched to a journal entry.
    BankReconciliation,
    /// Goods issue linking inventory to a delivery.
    GoodsIssue,
}

/// An event that correlates multiple objects across process boundaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationEvent {
    /// Unique event identifier.
    pub event_id: Uuid,
    /// Type of correlation.
    pub correlation_type: CorrelationEventType,
    /// Human-readable correlation identifier (e.g. "3WAY-abcd1234").
    pub correlation_id: String,
    /// When the correlation was established.
    pub timestamp: DateTime<Utc>,
    /// Objects involved in this correlation.
    pub object_refs: Vec<EventObjectRef>,
    /// Resource that triggered/performed the correlation.
    pub resource_id: String,
    /// Arbitrary key-value attributes.
    pub attributes: HashMap<String, serde_json::Value>,
    /// Company code.
    pub company_code: String,
}

impl CorrelationEvent {
    /// Create a three-way match correlation event linking a PO, goods receipt,
    /// and vendor invoice.
    pub fn three_way_match(
        po_id: Uuid,
        gr_id: Uuid,
        invoice_id: Uuid,
        timestamp: DateTime<Utc>,
        resource_id: &str,
        company_code: &str,
    ) -> Self {
        let correlation_id = format!("3WAY-{}", &po_id.to_string()[..8]);
        Self {
            event_id: Uuid::new_v4(),
            correlation_type: CorrelationEventType::ThreeWayMatch,
            correlation_id,
            timestamp,
            object_refs: vec![
                EventObjectRef::new(po_id, "purchase_order", ObjectQualifier::Read),
                EventObjectRef::new(gr_id, "goods_receipt", ObjectQualifier::Read),
                EventObjectRef::new(invoice_id, "vendor_invoice", ObjectQualifier::Updated),
            ],
            resource_id: resource_id.into(),
            attributes: HashMap::new(),
            company_code: company_code.into(),
        }
    }

    /// Create a payment allocation event linking a payment to one or more
    /// invoices.
    pub fn payment_allocation(
        payment_id: Uuid,
        invoice_ids: &[Uuid],
        timestamp: DateTime<Utc>,
        resource_id: &str,
        company_code: &str,
    ) -> Self {
        let correlation_id = format!("PAY-{}", &payment_id.to_string()[..8]);
        let mut refs = vec![EventObjectRef::new(
            payment_id,
            "payment",
            ObjectQualifier::Created,
        )];
        for &inv_id in invoice_ids {
            refs.push(EventObjectRef::new(
                inv_id,
                "vendor_invoice",
                ObjectQualifier::Updated,
            ));
        }
        Self {
            event_id: Uuid::new_v4(),
            correlation_type: CorrelationEventType::PaymentAllocation,
            correlation_id,
            timestamp,
            object_refs: refs,
            resource_id: resource_id.into(),
            attributes: HashMap::new(),
            company_code: company_code.into(),
        }
    }

    /// Create a bank reconciliation event linking a bank statement line to a
    /// journal entry.
    pub fn bank_reconciliation(
        statement_line_id: Uuid,
        je_id: Uuid,
        timestamp: DateTime<Utc>,
        resource_id: &str,
        company_code: &str,
    ) -> Self {
        let correlation_id = format!("RECON-{}", &statement_line_id.to_string()[..8]);
        Self {
            event_id: Uuid::new_v4(),
            correlation_type: CorrelationEventType::BankReconciliation,
            correlation_id,
            timestamp,
            object_refs: vec![
                EventObjectRef::new(
                    statement_line_id,
                    "bank_statement_line",
                    ObjectQualifier::Updated,
                ),
                EventObjectRef::new(je_id, "journal_entry", ObjectQualifier::Read),
            ],
            resource_id: resource_id.into(),
            attributes: HashMap::new(),
            company_code: company_code.into(),
        }
    }

    /// Add a custom attribute and return self (builder pattern).
    pub fn with_attribute(mut self, key: &str, value: serde_json::Value) -> Self {
        self.attributes.insert(key.into(), value);
        self
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_three_way_match_has_three_refs() {
        let po = Uuid::new_v4();
        let gr = Uuid::new_v4();
        let inv = Uuid::new_v4();
        let evt = CorrelationEvent::three_way_match(po, gr, inv, Utc::now(), "user001", "1000");

        assert_eq!(evt.object_refs.len(), 3);
        assert_eq!(evt.correlation_type, CorrelationEventType::ThreeWayMatch);
        assert!(evt.correlation_id.starts_with("3WAY-"));
    }

    #[test]
    fn test_payment_allocation_multi_invoice() {
        let pay = Uuid::new_v4();
        let invoices = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let evt =
            CorrelationEvent::payment_allocation(pay, &invoices, Utc::now(), "user002", "2000");

        // 1 payment + 3 invoices = 4
        assert_eq!(evt.object_refs.len(), 4);
        assert_eq!(
            evt.correlation_type,
            CorrelationEventType::PaymentAllocation
        );
    }

    #[test]
    fn test_bank_reconciliation() {
        let stmt = Uuid::new_v4();
        let je = Uuid::new_v4();
        let evt = CorrelationEvent::bank_reconciliation(stmt, je, Utc::now(), "user003", "1000");

        assert_eq!(evt.object_refs.len(), 2);
        assert!(evt.correlation_id.starts_with("RECON-"));
    }

    #[test]
    fn test_serde_roundtrip() {
        let evt = CorrelationEvent::three_way_match(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now(),
            "user001",
            "1000",
        )
        .with_attribute("match_score", serde_json::json!(0.98));

        let json = serde_json::to_string(&evt).unwrap();
        let deserialized: CorrelationEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.correlation_id, evt.correlation_id);
        assert_eq!(deserialized.object_refs.len(), 3);
        assert!(deserialized.attributes.contains_key("match_score"));
    }
}
