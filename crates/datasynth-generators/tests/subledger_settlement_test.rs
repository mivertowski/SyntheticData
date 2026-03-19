//! Integration tests for subledger payment settlement functions.
//!
//! Verifies that `apply_ap_settlements` and `apply_ar_settlements` correctly
//! reduce `amount_remaining` on AP and AR invoices when payments are applied.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use datasynth_core::models::documents::{
    CustomerInvoice, CustomerInvoiceItem, DocumentType, Payment, PaymentAllocation, VendorInvoice,
    VendorInvoiceItem,
};
use datasynth_generators::{apply_ap_settlements, apply_ar_settlements, DocumentFlowLinker};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_vendor_invoice(doc_id: &str, vendor_id: &str) -> VendorInvoice {
    let mut inv = VendorInvoice::new(
        doc_id,
        "1000",
        vendor_id,
        format!("EXT-{}", doc_id), // external vendor number — NOT used for matching
        2024,
        1,
        NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        "SYSTEM",
    );
    // Single line: qty=1, unit_price=1000 → gross = 1000
    inv.add_item(VendorInvoiceItem::new(1, "Goods", dec!(1), dec!(1000)));
    inv
}

fn make_customer_invoice(doc_id: &str, customer_id: &str) -> CustomerInvoice {
    let mut inv = CustomerInvoice::new(
        doc_id,
        "1000",
        customer_id,
        2024,
        1,
        NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
        "SYSTEM",
    );
    // Single line: qty=1, unit_price=1200 → gross = 1200
    inv.add_item(CustomerInvoiceItem::new(1, "Product", dec!(1), dec!(1200)));
    inv
}

fn make_ap_payment(invoice_doc_id: &str, amount: Decimal) -> Payment {
    let mut p = Payment::new_ap_payment(
        format!("PAY-{}", invoice_doc_id),
        "1000",
        "VEND001",
        amount,
        2024,
        1,
        NaiveDate::from_ymd_opt(2024, 2, 10).unwrap(),
        "SYSTEM",
    );
    p.add_allocation(PaymentAllocation::new(
        invoice_doc_id,
        DocumentType::VendorInvoice,
        amount,
    ));
    p
}

fn make_ar_receipt(invoice_doc_id: &str, amount: Decimal) -> Payment {
    let mut p = Payment::new_ar_receipt(
        format!("REC-{}", invoice_doc_id),
        "1000",
        "CUST001",
        amount,
        2024,
        1,
        NaiveDate::from_ymd_opt(2024, 2, 10).unwrap(),
        "SYSTEM",
    );
    p.add_allocation(PaymentAllocation::new(
        invoice_doc_id,
        DocumentType::CustomerInvoice,
        amount,
    ));
    p
}

// ---------------------------------------------------------------------------
// AP settlement tests
// ---------------------------------------------------------------------------

#[test]
fn ap_full_settlement_reduces_amount_remaining_to_zero() {
    let vi = make_vendor_invoice("VI-1000-0000000001", "VEND001");

    let mut linker = DocumentFlowLinker::new();
    let mut ap_invoices = linker.batch_create_ap_invoices(&[vi]);

    // Before payment, amount_remaining equals gross_amount
    let gross = ap_invoices[0].gross_amount.document_amount;
    assert!(gross > Decimal::ZERO, "gross amount must be positive");
    assert_eq!(ap_invoices[0].amount_remaining, gross);

    // Full payment
    let payments = vec![make_ap_payment("VI-1000-0000000001", gross)];
    apply_ap_settlements(&mut ap_invoices, &payments);

    assert_eq!(
        ap_invoices[0].amount_remaining,
        Decimal::ZERO,
        "amount_remaining must be zero after full settlement"
    );
}

#[test]
fn ap_partial_settlement_reduces_amount_remaining_proportionally() {
    let vi = make_vendor_invoice("VI-1000-0000000002", "VEND001");

    let mut linker = DocumentFlowLinker::new();
    let mut ap_invoices = linker.batch_create_ap_invoices(&[vi]);

    let gross = ap_invoices[0].gross_amount.document_amount;
    let partial = gross / dec!(2);

    let payments = vec![make_ap_payment("VI-1000-0000000002", partial)];
    apply_ap_settlements(&mut ap_invoices, &payments);

    assert_eq!(
        ap_invoices[0].amount_remaining,
        gross - partial,
        "amount_remaining must be halved after 50% settlement"
    );
}

#[test]
fn ap_over_settlement_clamps_to_zero() {
    let vi = make_vendor_invoice("VI-1000-0000000003", "VEND001");

    let mut linker = DocumentFlowLinker::new();
    let mut ap_invoices = linker.batch_create_ap_invoices(&[vi]);

    let gross = ap_invoices[0].gross_amount.document_amount;
    let overpayment = gross + dec!(999);

    let payments = vec![make_ap_payment("VI-1000-0000000003", overpayment)];
    apply_ap_settlements(&mut ap_invoices, &payments);

    assert_eq!(
        ap_invoices[0].amount_remaining,
        Decimal::ZERO,
        "amount_remaining must not go negative"
    );
}

#[test]
fn ap_unmatched_payment_leaves_amount_remaining_unchanged() {
    let vi = make_vendor_invoice("VI-1000-0000000004", "VEND001");

    let mut linker = DocumentFlowLinker::new();
    let mut ap_invoices = linker.batch_create_ap_invoices(&[vi]);

    let gross = ap_invoices[0].gross_amount.document_amount;

    // Payment referencing a different invoice
    let payments = vec![make_ap_payment("VI-9999-9999999999", gross)];
    apply_ap_settlements(&mut ap_invoices, &payments);

    assert_eq!(
        ap_invoices[0].amount_remaining, gross,
        "amount_remaining must be unchanged when no matching invoice"
    );
}

#[test]
fn ap_ignores_ar_receipts() {
    let vi = make_vendor_invoice("VI-1000-0000000005", "VEND001");

    let mut linker = DocumentFlowLinker::new();
    let mut ap_invoices = linker.batch_create_ap_invoices(&[vi]);

    let gross = ap_invoices[0].gross_amount.document_amount;

    // AR receipt — should not touch AP invoices
    let payments = vec![make_ar_receipt("VI-1000-0000000005", gross)];
    apply_ap_settlements(&mut ap_invoices, &payments);

    assert_eq!(
        ap_invoices[0].amount_remaining, gross,
        "AR receipt must not reduce AP amount_remaining"
    );
}

// ---------------------------------------------------------------------------
// AR settlement tests
// ---------------------------------------------------------------------------

#[test]
fn ar_full_settlement_reduces_amount_remaining_to_zero() {
    let ci = make_customer_invoice("CI-1000-0000000001", "CUST001");

    let mut linker = DocumentFlowLinker::new();
    let mut ar_invoices = linker.batch_create_ar_invoices(&[ci]);

    let gross = ar_invoices[0].gross_amount.document_amount;
    assert!(gross > Decimal::ZERO);
    assert_eq!(ar_invoices[0].amount_remaining, gross);

    let payments = vec![make_ar_receipt("CI-1000-0000000001", gross)];
    apply_ar_settlements(&mut ar_invoices, &payments);

    assert_eq!(
        ar_invoices[0].amount_remaining,
        Decimal::ZERO,
        "amount_remaining must be zero after full AR settlement"
    );
}

#[test]
fn ar_partial_settlement_reduces_amount_remaining_proportionally() {
    let ci = make_customer_invoice("CI-1000-0000000002", "CUST001");

    let mut linker = DocumentFlowLinker::new();
    let mut ar_invoices = linker.batch_create_ar_invoices(&[ci]);

    let gross = ar_invoices[0].gross_amount.document_amount;
    let partial = gross / dec!(4);

    let payments = vec![make_ar_receipt("CI-1000-0000000002", partial)];
    apply_ar_settlements(&mut ar_invoices, &payments);

    assert_eq!(ar_invoices[0].amount_remaining, gross - partial);
}

#[test]
fn ar_over_settlement_clamps_to_zero() {
    let ci = make_customer_invoice("CI-1000-0000000003", "CUST001");

    let mut linker = DocumentFlowLinker::new();
    let mut ar_invoices = linker.batch_create_ar_invoices(&[ci]);

    let gross = ar_invoices[0].gross_amount.document_amount;
    let overpayment = gross * dec!(2);

    let payments = vec![make_ar_receipt("CI-1000-0000000003", overpayment)];
    apply_ar_settlements(&mut ar_invoices, &payments);

    assert_eq!(
        ar_invoices[0].amount_remaining,
        Decimal::ZERO,
        "amount_remaining must not go negative"
    );
}

#[test]
fn ar_unmatched_receipt_leaves_amount_remaining_unchanged() {
    let ci = make_customer_invoice("CI-1000-0000000004", "CUST001");

    let mut linker = DocumentFlowLinker::new();
    let mut ar_invoices = linker.batch_create_ar_invoices(&[ci]);

    let gross = ar_invoices[0].gross_amount.document_amount;

    let payments = vec![make_ar_receipt("CI-9999-9999999999", gross)];
    apply_ar_settlements(&mut ar_invoices, &payments);

    assert_eq!(ar_invoices[0].amount_remaining, gross);
}

#[test]
fn ar_ignores_ap_payments() {
    let ci = make_customer_invoice("CI-1000-0000000005", "CUST001");

    let mut linker = DocumentFlowLinker::new();
    let mut ar_invoices = linker.batch_create_ar_invoices(&[ci]);

    let gross = ar_invoices[0].gross_amount.document_amount;

    // AP payment — should not touch AR invoices
    let payments = vec![make_ap_payment("CI-1000-0000000005", gross)];
    apply_ar_settlements(&mut ar_invoices, &payments);

    assert_eq!(
        ar_invoices[0].amount_remaining, gross,
        "AP payment must not reduce AR amount_remaining"
    );
}

// ---------------------------------------------------------------------------
// Multi-invoice settlement
// ---------------------------------------------------------------------------

#[test]
fn multiple_invoices_settled_independently() {
    let vi1 = make_vendor_invoice("VI-1000-0000000010", "VEND001");
    let vi2 = make_vendor_invoice("VI-1000-0000000011", "VEND001");

    let mut linker = DocumentFlowLinker::new();
    let mut ap_invoices = linker.batch_create_ap_invoices(&[vi1, vi2]);

    let gross0 = ap_invoices[0].gross_amount.document_amount;
    let gross1 = ap_invoices[1].gross_amount.document_amount;

    // Only settle invoice 0
    let payments = vec![make_ap_payment("VI-1000-0000000010", gross0)];
    apply_ap_settlements(&mut ap_invoices, &payments);

    assert_eq!(ap_invoices[0].amount_remaining, Decimal::ZERO);
    assert_eq!(
        ap_invoices[1].amount_remaining, gross1,
        "unsettled invoice must be unchanged"
    );
}
