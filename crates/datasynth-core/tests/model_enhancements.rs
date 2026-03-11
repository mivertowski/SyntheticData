//! Tests for AuditFinding + JournalEntry model enhancements (Task 3).

use chrono::NaiveDate;
use datasynth_core::models::audit::AuditFinding;
use datasynth_core::models::audit::FindingType;
use datasynth_core::models::journal_entry::{DocumentRef, JournalEntryHeader};
use uuid::Uuid;

#[test]
fn audit_finding_has_relationship_fields() {
    let finding = AuditFinding::new(
        Uuid::new_v4(),
        FindingType::ControlDeficiency,
        "Test finding",
    );
    assert!(finding.related_control_ids.is_empty());
    assert!(finding.related_risk_id.is_none());
    assert!(finding.workpaper_id.is_none());
}

#[test]
fn audit_finding_set_relationship_fields() {
    let mut finding = AuditFinding::new(
        Uuid::new_v4(),
        FindingType::MaterialWeakness,
        "Material weakness in revenue",
    );
    finding.related_control_ids = vec!["CTRL-001".into(), "CTRL-002".into()];
    finding.related_risk_id = Some("RISK-001".into());
    finding.workpaper_id = Some("WP-001".into());

    assert_eq!(finding.related_control_ids.len(), 2);
    assert_eq!(finding.related_risk_id.as_deref(), Some("RISK-001"));
    assert_eq!(finding.workpaper_id.as_deref(), Some("WP-001"));
}

#[test]
fn journal_entry_has_document_ref_and_approval() {
    let je = JournalEntryHeader::new(
        "1000".into(),
        NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
    );
    assert!(je.source_document.is_none());
    assert!(je.approved_by.is_none());
    assert!(je.approval_date.is_none());
    // Verify existing sod_violation field is still present
    assert!(!je.sod_violation);
}

#[test]
fn document_ref_variants() {
    let po = DocumentRef::PurchaseOrder("PO-2024-000001".into());
    assert!(matches!(po, DocumentRef::PurchaseOrder(_)));

    let vi = DocumentRef::VendorInvoice("INV-2024-000001".into());
    assert!(matches!(vi, DocumentRef::VendorInvoice(_)));

    let ci = DocumentRef::CustomerInvoice("SO-2024-000001".into());
    assert!(matches!(ci, DocumentRef::CustomerInvoice(_)));

    let gr = DocumentRef::GoodsReceipt("GR-2024-000001".into());
    assert!(matches!(gr, DocumentRef::GoodsReceipt(_)));

    let dl = DocumentRef::Delivery("DL-001".into());
    assert!(matches!(dl, DocumentRef::Delivery(_)));

    let pay = DocumentRef::Payment("PAY-2024-000001".into());
    assert!(matches!(pay, DocumentRef::Payment(_)));

    let rcpt = DocumentRef::Receipt("REC-001".into());
    assert!(matches!(rcpt, DocumentRef::Receipt(_)));

    let manual = DocumentRef::Manual;
    assert!(matches!(manual, DocumentRef::Manual));
}

#[test]
fn document_ref_parse_from_reference_string() {
    // Test the parse helper that converts reference strings to DocumentRef
    let po = DocumentRef::parse("PO-2024-000001");
    assert!(matches!(po, Some(DocumentRef::PurchaseOrder(_))));

    let inv = DocumentRef::parse("INV-2024-000001");
    assert!(matches!(inv, Some(DocumentRef::VendorInvoice(_))));

    let so = DocumentRef::parse("SO-2024-000001");
    assert!(matches!(so, Some(DocumentRef::CustomerInvoice(_))));

    let gr = DocumentRef::parse("GR-2024-000001");
    assert!(matches!(gr, Some(DocumentRef::GoodsReceipt(_))));

    let pay = DocumentRef::parse("PAY-2024-000001");
    assert!(matches!(pay, Some(DocumentRef::Payment(_))));

    let fa = DocumentRef::parse("FA-2024-000001");
    assert!(matches!(fa, None)); // AssetTag is not a document ref

    let doc = DocumentRef::parse("DOC-2024-000001");
    assert!(matches!(doc, None)); // Internal doc is not a specific document ref
}

#[test]
fn journal_entry_set_approval_fields() {
    let mut je = JournalEntryHeader::new(
        "1000".into(),
        NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
    );
    je.approved_by = Some("MGR0001".into());
    je.approval_date = Some(NaiveDate::from_ymd_opt(2024, 12, 2).unwrap());
    je.source_document = Some(DocumentRef::PurchaseOrder("PO-2024-000001".into()));

    assert_eq!(je.approved_by.as_deref(), Some("MGR0001"));
    assert_eq!(
        je.approval_date,
        Some(NaiveDate::from_ymd_opt(2024, 12, 2).unwrap())
    );
    assert!(matches!(
        je.source_document,
        Some(DocumentRef::PurchaseOrder(_))
    ));
}
