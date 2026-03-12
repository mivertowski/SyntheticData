//! Regression tests for document FK population in P2P and O2C chains.
//!
//! DS-GEP-004: Ensures top-level FK fields (`purchase_order_id`,
//! `goods_receipt_id`, `sales_order_id`, `delivery_id`) are
//! consistently populated when documents are generated as part of a
//! chain. This removes the need for expensive FK lookups in the
//! graph-export pipeline's edge synthesizers.

use chrono::NaiveDate;
use rust_decimal::Decimal;

use datasynth_core::models::{
    CreditRating, Customer, CustomerPaymentBehavior, CustomerType, Material, MaterialType, Vendor,
    VendorType,
};
use datasynth_generators::document_flow::{
    O2CGenerator, O2CGeneratorConfig, P2PGenerator, P2PGeneratorConfig,
};

// ---------------------------------------------------------------------------
// Helper factories
// ---------------------------------------------------------------------------

fn test_vendor() -> Vendor {
    Vendor::new("V-000001", "Test Vendor Inc.", VendorType::Supplier)
}

fn test_customer() -> Customer {
    let mut customer = Customer::new("C-000001", "Test Customer Inc.", CustomerType::Corporate);
    customer.credit_rating = CreditRating::A;
    customer.credit_limit = Decimal::from(1_000_000);
    customer.payment_behavior = CustomerPaymentBehavior::OnTime;
    customer
}

fn test_materials() -> Vec<Material> {
    vec![
        Material::new("MAT-001", "Material A", MaterialType::RawMaterial)
            .with_standard_cost(Decimal::from(100)),
        Material::new("MAT-002", "Material B", MaterialType::FinishedGood)
            .with_standard_cost(Decimal::from(50)),
    ]
}

fn test_materials_o2c() -> Vec<Material> {
    let mut mat1 = Material::new("MAT-001", "Product A", MaterialType::FinishedGood);
    mat1.list_price = Decimal::from(100);
    mat1.standard_cost = Decimal::from(60);

    let mut mat2 = Material::new("MAT-002", "Product B", MaterialType::FinishedGood);
    mat2.list_price = Decimal::from(200);
    mat2.standard_cost = Decimal::from(120);

    vec![mat1, mat2]
}

// ---------------------------------------------------------------------------
// P2P FK tests
// ---------------------------------------------------------------------------

#[test]
fn p2p_vendor_invoice_has_purchase_order_id() {
    let mut gen = P2PGenerator::new(42);
    let vendor = test_vendor();
    let materials = test_materials();
    let refs: Vec<&Material> = materials.iter().collect();

    let chain = gen.generate_chain(
        "1000",
        &vendor,
        &refs,
        NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
        2024,
        6,
        "TESTUSER",
    );

    let invoice = chain
        .vendor_invoice
        .as_ref()
        .expect("chain should have vendor invoice");

    assert!(
        invoice.purchase_order_id.is_some(),
        "VendorInvoice.purchase_order_id must be populated in chain"
    );
    assert_eq!(
        invoice.purchase_order_id.as_deref().unwrap(),
        chain.purchase_order.header.document_id,
        "VendorInvoice.purchase_order_id must match the PO document_id"
    );
}

#[test]
fn p2p_vendor_invoice_has_goods_receipt_id() {
    let mut gen = P2PGenerator::new(42);
    let vendor = test_vendor();
    let materials = test_materials();
    let refs: Vec<&Material> = materials.iter().collect();

    let chain = gen.generate_chain(
        "1000",
        &vendor,
        &refs,
        NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
        2024,
        6,
        "TESTUSER",
    );

    let invoice = chain
        .vendor_invoice
        .as_ref()
        .expect("chain should have vendor invoice");

    assert!(
        invoice.goods_receipt_id.is_some(),
        "VendorInvoice.goods_receipt_id must be populated in chain"
    );
    assert_eq!(
        invoice.goods_receipt_id.as_deref().unwrap(),
        chain.goods_receipts[0].header.document_id,
        "VendorInvoice.goods_receipt_id must match the first GR document_id"
    );
}

#[test]
fn p2p_goods_receipt_has_purchase_order_id() {
    let mut gen = P2PGenerator::new(42);
    let vendor = test_vendor();
    let materials = test_materials();
    let refs: Vec<&Material> = materials.iter().collect();

    let chain = gen.generate_chain(
        "1000",
        &vendor,
        &refs,
        NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
        2024,
        6,
        "TESTUSER",
    );

    for (i, gr) in chain.goods_receipts.iter().enumerate() {
        assert!(
            gr.purchase_order_id.is_some(),
            "GoodsReceipt[{i}].purchase_order_id must be populated in chain"
        );
        assert_eq!(
            gr.purchase_order_id.as_deref().unwrap(),
            chain.purchase_order.header.document_id,
            "GoodsReceipt[{i}].purchase_order_id must match the PO document_id"
        );
    }
}

#[test]
fn p2p_partial_delivery_all_grs_have_po_fk() {
    let config = P2PGeneratorConfig {
        partial_delivery_rate: 1.0, // Force partial delivery → 2 GRs
        ..Default::default()
    };

    let mut gen = P2PGenerator::with_config(42, config);
    let vendor = test_vendor();
    let materials = test_materials();
    let refs: Vec<&Material> = materials.iter().collect();

    let chain = gen.generate_chain(
        "1000",
        &vendor,
        &refs,
        NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
        2024,
        6,
        "TESTUSER",
    );

    assert!(
        chain.goods_receipts.len() >= 2,
        "partial delivery must produce >= 2 GRs"
    );

    for (i, gr) in chain.goods_receipts.iter().enumerate() {
        assert!(
            gr.purchase_order_id.is_some(),
            "GoodsReceipt[{i}] (partial delivery) must have purchase_order_id"
        );
    }

    // Invoice should still have the FK
    if let Some(ref invoice) = chain.vendor_invoice {
        assert!(
            invoice.purchase_order_id.is_some(),
            "VendorInvoice (after partial delivery) must have purchase_order_id"
        );
    }
}

// ---------------------------------------------------------------------------
// O2C FK tests
// ---------------------------------------------------------------------------

#[test]
fn o2c_customer_invoice_has_sales_order_id() {
    let mut gen = O2CGenerator::new(42);
    let customer = test_customer();
    let materials = test_materials_o2c();
    let refs: Vec<&Material> = materials.iter().collect();

    let chain = gen.generate_chain(
        "1000",
        &customer,
        &refs,
        NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
        2024,
        6,
        "TESTUSER",
    );

    assert!(chain.credit_check_passed, "credit check must pass");

    let invoice = chain
        .customer_invoice
        .as_ref()
        .expect("chain should have customer invoice");

    assert!(
        invoice.sales_order_id.is_some(),
        "CustomerInvoice.sales_order_id must be populated in chain"
    );
    assert_eq!(
        invoice.sales_order_id.as_deref().unwrap(),
        chain.sales_order.header.document_id,
        "CustomerInvoice.sales_order_id must match the SO document_id"
    );
}

#[test]
fn o2c_customer_invoice_has_delivery_id() {
    let mut gen = O2CGenerator::new(42);
    let customer = test_customer();
    let materials = test_materials_o2c();
    let refs: Vec<&Material> = materials.iter().collect();

    let chain = gen.generate_chain(
        "1000",
        &customer,
        &refs,
        NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
        2024,
        6,
        "TESTUSER",
    );

    let invoice = chain
        .customer_invoice
        .as_ref()
        .expect("chain should have customer invoice");

    assert!(
        invoice.delivery_id.is_some(),
        "CustomerInvoice.delivery_id must be populated in chain"
    );
    assert_eq!(
        invoice.delivery_id.as_deref().unwrap(),
        chain.deliveries[0].header.document_id,
        "CustomerInvoice.delivery_id must match the first Delivery document_id"
    );
}

#[test]
fn o2c_delivery_has_sales_order_id() {
    let mut gen = O2CGenerator::new(42);
    let customer = test_customer();
    let materials = test_materials_o2c();
    let refs: Vec<&Material> = materials.iter().collect();

    let chain = gen.generate_chain(
        "1000",
        &customer,
        &refs,
        NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
        2024,
        6,
        "TESTUSER",
    );

    for (i, dlv) in chain.deliveries.iter().enumerate() {
        assert!(
            dlv.sales_order_id.is_some(),
            "Delivery[{i}].sales_order_id must be populated in chain"
        );
        assert_eq!(
            dlv.sales_order_id.as_deref().unwrap(),
            chain.sales_order.header.document_id,
            "Delivery[{i}].sales_order_id must match the SO document_id"
        );
    }
}

#[test]
fn o2c_partial_shipment_all_deliveries_have_so_fk() {
    let config = O2CGeneratorConfig {
        partial_shipment_rate: 1.0, // Force partial → 2 deliveries
        ..Default::default()
    };

    let mut gen = O2CGenerator::with_config(42, config);
    let customer = test_customer();
    let materials = test_materials_o2c();
    let refs: Vec<&Material> = materials.iter().collect();

    let chain = gen.generate_chain(
        "1000",
        &customer,
        &refs,
        NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
        2024,
        6,
        "TESTUSER",
    );

    assert!(
        chain.deliveries.len() >= 2,
        "partial shipment must produce >= 2 deliveries"
    );

    for (i, dlv) in chain.deliveries.iter().enumerate() {
        assert!(
            dlv.sales_order_id.is_some(),
            "Delivery[{i}] (partial shipment) must have sales_order_id"
        );
    }

    // Invoice should still have the FK
    if let Some(ref invoice) = chain.customer_invoice {
        assert!(
            invoice.sales_order_id.is_some(),
            "CustomerInvoice (after partial shipment) must have sales_order_id"
        );
        assert!(
            invoice.delivery_id.is_some(),
            "CustomerInvoice (after partial shipment) must have delivery_id"
        );
    }
}

// ---------------------------------------------------------------------------
// created_by_employee_id tests
// ---------------------------------------------------------------------------

#[test]
fn document_header_created_by_employee_id_defaults_to_none() {
    use datasynth_core::models::documents::{DocumentHeader, DocumentType};

    let header = DocumentHeader::new(
        "TEST-001",
        DocumentType::PurchaseOrder,
        "1000",
        2024,
        6,
        NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
        "JSMITH",
    );

    assert!(
        header.created_by_employee_id.is_none(),
        "created_by_employee_id should default to None"
    );
}

#[test]
fn document_header_created_by_employee_id_builder() {
    use datasynth_core::models::documents::{DocumentHeader, DocumentType};

    let header = DocumentHeader::new(
        "TEST-001",
        DocumentType::PurchaseOrder,
        "1000",
        2024,
        6,
        NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
        "JSMITH",
    )
    .with_created_by_employee_id("E-001234");

    assert_eq!(
        header.created_by_employee_id.as_deref(),
        Some("E-001234"),
        "created_by_employee_id should be set by builder"
    );
    assert_eq!(
        header.created_by, "JSMITH",
        "created_by (user_id) should remain unchanged"
    );
}

#[test]
fn document_header_created_by_employee_id_serializes() {
    use datasynth_core::models::documents::{DocumentHeader, DocumentType};

    let header = DocumentHeader::new(
        "TEST-001",
        DocumentType::PurchaseOrder,
        "1000",
        2024,
        6,
        NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
        "JSMITH",
    )
    .with_created_by_employee_id("E-001234");

    let json = serde_json::to_string(&header).expect("should serialize");
    assert!(
        json.contains("created_by_employee_id"),
        "serialized JSON should contain created_by_employee_id"
    );
    assert!(
        json.contains("E-001234"),
        "serialized JSON should contain the employee ID value"
    );

    // Verify skip_serializing_if: when None, field should be absent
    let header_no_eid = DocumentHeader::new(
        "TEST-002",
        DocumentType::PurchaseOrder,
        "1000",
        2024,
        6,
        NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
        "JSMITH",
    );

    let json_no_eid = serde_json::to_string(&header_no_eid).expect("should serialize");
    assert!(
        !json_no_eid.contains("created_by_employee_id"),
        "serialized JSON should not contain created_by_employee_id when None"
    );
}
