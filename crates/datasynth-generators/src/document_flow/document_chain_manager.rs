//! Document Chain Manager for coordinated document flow generation.
//!
//! This module provides a central manager that coordinates the generation
//! of both P2P and O2C document flows, maintaining document references
//! and ensuring coherent data generation.

use chrono::NaiveDate;
use datasynth_core::models::{
    documents::DocumentReference, CustomerPool, MaterialPool, VendorPool,
};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use super::{
    O2CDocumentChain, O2CGenerator, O2CGeneratorConfig, P2PDocumentChain, P2PGenerator,
    P2PGeneratorConfig,
};

/// Configuration for document chain manager.
#[derive(Debug, Clone)]
pub struct DocumentChainManagerConfig {
    /// P2P flow configuration
    pub p2p_config: P2PGeneratorConfig,
    /// O2C flow configuration
    pub o2c_config: O2CGeneratorConfig,
    /// Ratio of P2P to O2C transactions (1.0 = equal, 2.0 = 2x P2P)
    pub p2p_to_o2c_ratio: f64,
}

impl Default for DocumentChainManagerConfig {
    fn default() -> Self {
        Self {
            p2p_config: P2PGeneratorConfig::default(),
            o2c_config: O2CGeneratorConfig::default(),
            p2p_to_o2c_ratio: 1.0,
        }
    }
}

/// Summary statistics for generated document chains.
#[derive(Debug, Default)]
pub struct DocumentChainStats {
    /// Total P2P chains generated
    pub p2p_chains: usize,
    /// P2P chains with three-way match passed
    pub p2p_three_way_match_passed: usize,
    /// P2P chains fully completed (payment made)
    pub p2p_completed: usize,
    /// Total O2C chains generated
    pub o2c_chains: usize,
    /// O2C chains with credit check passed
    pub o2c_credit_check_passed: usize,
    /// O2C chains fully completed (payment received)
    pub o2c_completed: usize,
    /// Total purchase orders
    pub purchase_orders: usize,
    /// Total goods receipts
    pub goods_receipts: usize,
    /// Total vendor invoices
    pub vendor_invoices: usize,
    /// Total AP payments
    pub ap_payments: usize,
    /// Total sales orders
    pub sales_orders: usize,
    /// Total deliveries
    pub deliveries: usize,
    /// Total customer invoices
    pub customer_invoices: usize,
    /// Total AR receipts
    pub ar_receipts: usize,
}

/// Generated document flows result.
#[derive(Debug)]
pub struct GeneratedDocumentFlows {
    /// P2P chains
    pub p2p_chains: Vec<P2PDocumentChain>,
    /// O2C chains
    pub o2c_chains: Vec<O2CDocumentChain>,
    /// All document references
    pub document_references: Vec<DocumentReference>,
    /// Statistics
    pub stats: DocumentChainStats,
}

/// Document Chain Manager for coordinated P2P and O2C generation.
pub struct DocumentChainManager {
    rng: ChaCha8Rng,
    seed: u64,
    config: DocumentChainManagerConfig,
    p2p_generator: P2PGenerator,
    o2c_generator: O2CGenerator,
}

impl DocumentChainManager {
    /// Create a new document chain manager.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, DocumentChainManagerConfig::default())
    }

    /// Create a new document chain manager with custom configuration.
    pub fn with_config(seed: u64, config: DocumentChainManagerConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
            p2p_generator: P2PGenerator::with_config(seed, config.p2p_config.clone()),
            o2c_generator: O2CGenerator::with_config(seed + 1000, config.o2c_config.clone()),
            config,
        }
    }

    /// Generate document flows for a company.
    pub fn generate_flows(
        &mut self,
        company_code: &str,
        total_chains: usize,
        vendors: &VendorPool,
        customers: &CustomerPool,
        materials: &MaterialPool,
        date_range: (NaiveDate, NaiveDate),
        fiscal_year: u16,
        created_by: &str,
    ) -> GeneratedDocumentFlows {
        // Calculate P2P and O2C counts based on ratio
        let ratio = self.config.p2p_to_o2c_ratio;
        let p2p_count = ((total_chains as f64) * ratio / (1.0 + ratio)) as usize;
        let o2c_count = total_chains - p2p_count;

        // Generate P2P chains
        let p2p_chains = self.p2p_generator.generate_chains(
            p2p_count,
            company_code,
            vendors,
            materials,
            date_range,
            fiscal_year,
            created_by,
        );

        // Generate O2C chains
        let o2c_chains = self.o2c_generator.generate_chains(
            o2c_count,
            company_code,
            customers,
            materials,
            date_range,
            fiscal_year,
            created_by,
        );

        // Collect all document references
        let document_references = self.collect_document_references(&p2p_chains, &o2c_chains);

        // Calculate statistics
        let stats = self.calculate_stats(&p2p_chains, &o2c_chains);

        GeneratedDocumentFlows {
            p2p_chains,
            o2c_chains,
            document_references,
            stats,
        }
    }

    /// Generate balanced document flows (equal P2P and O2C).
    pub fn generate_balanced_flows(
        &mut self,
        chains_per_type: usize,
        company_code: &str,
        vendors: &VendorPool,
        customers: &CustomerPool,
        materials: &MaterialPool,
        date_range: (NaiveDate, NaiveDate),
        fiscal_year: u16,
        created_by: &str,
    ) -> GeneratedDocumentFlows {
        // Generate P2P chains
        let p2p_chains = self.p2p_generator.generate_chains(
            chains_per_type,
            company_code,
            vendors,
            materials,
            date_range,
            fiscal_year,
            created_by,
        );

        // Generate O2C chains
        let o2c_chains = self.o2c_generator.generate_chains(
            chains_per_type,
            company_code,
            customers,
            materials,
            date_range,
            fiscal_year,
            created_by,
        );

        let document_references = self.collect_document_references(&p2p_chains, &o2c_chains);
        let stats = self.calculate_stats(&p2p_chains, &o2c_chains);

        GeneratedDocumentFlows {
            p2p_chains,
            o2c_chains,
            document_references,
            stats,
        }
    }

    /// Generate flows for multiple company codes.
    pub fn generate_multi_company_flows(
        &mut self,
        company_codes: &[String],
        chains_per_company: usize,
        vendors_by_company: &std::collections::HashMap<String, VendorPool>,
        customers_by_company: &std::collections::HashMap<String, CustomerPool>,
        materials: &MaterialPool, // Shared materials
        date_range: (NaiveDate, NaiveDate),
        fiscal_year: u16,
        created_by: &str,
    ) -> Vec<GeneratedDocumentFlows> {
        let mut results = Vec::new();

        for company_code in company_codes {
            let vendors = vendors_by_company
                .get(company_code)
                .expect("Vendor pool not found for company");
            let customers = customers_by_company
                .get(company_code)
                .expect("Customer pool not found for company");

            let flows = self.generate_flows(
                company_code,
                chains_per_company,
                vendors,
                customers,
                materials,
                date_range,
                fiscal_year,
                created_by,
            );

            results.push(flows);
        }

        results
    }

    /// Collect all document references from chains.
    fn collect_document_references(
        &self,
        p2p_chains: &[P2PDocumentChain],
        o2c_chains: &[O2CDocumentChain],
    ) -> Vec<DocumentReference> {
        let mut references = Vec::new();

        // Collect P2P references
        for chain in p2p_chains {
            // PO references
            for ref_doc in &chain.purchase_order.header.document_references {
                references.push(ref_doc.clone());
            }

            // GR references
            for gr in &chain.goods_receipts {
                for ref_doc in &gr.header.document_references {
                    references.push(ref_doc.clone());
                }
            }

            // Invoice references
            if let Some(invoice) = &chain.vendor_invoice {
                for ref_doc in &invoice.header.document_references {
                    references.push(ref_doc.clone());
                }
            }

            // Payment references
            if let Some(payment) = &chain.payment {
                for ref_doc in &payment.header.document_references {
                    references.push(ref_doc.clone());
                }
            }
        }

        // Collect O2C references
        for chain in o2c_chains {
            // SO references
            for ref_doc in &chain.sales_order.header.document_references {
                references.push(ref_doc.clone());
            }

            // Delivery references
            for dlv in &chain.deliveries {
                for ref_doc in &dlv.header.document_references {
                    references.push(ref_doc.clone());
                }
            }

            // Invoice references
            if let Some(invoice) = &chain.customer_invoice {
                for ref_doc in &invoice.header.document_references {
                    references.push(ref_doc.clone());
                }
            }

            // Receipt references
            if let Some(receipt) = &chain.customer_receipt {
                for ref_doc in &receipt.header.document_references {
                    references.push(ref_doc.clone());
                }
            }
        }

        references
    }

    /// Calculate statistics from generated chains.
    fn calculate_stats(
        &self,
        p2p_chains: &[P2PDocumentChain],
        o2c_chains: &[O2CDocumentChain],
    ) -> DocumentChainStats {
        let mut stats = DocumentChainStats {
            p2p_chains: p2p_chains.len(),
            ..Default::default()
        };

        // P2P stats
        for chain in p2p_chains {
            stats.purchase_orders += 1;
            stats.goods_receipts += chain.goods_receipts.len();

            if chain.three_way_match_passed {
                stats.p2p_three_way_match_passed += 1;
            }

            if chain.vendor_invoice.is_some() {
                stats.vendor_invoices += 1;
            }

            if chain.payment.is_some() {
                stats.ap_payments += 1;
            }

            if chain.is_complete {
                stats.p2p_completed += 1;
            }
        }

        // O2C stats
        stats.o2c_chains = o2c_chains.len();
        for chain in o2c_chains {
            stats.sales_orders += 1;
            stats.deliveries += chain.deliveries.len();

            if chain.credit_check_passed {
                stats.o2c_credit_check_passed += 1;
            }

            if chain.customer_invoice.is_some() {
                stats.customer_invoices += 1;
            }

            if chain.customer_receipt.is_some() {
                stats.ar_receipts += 1;
            }

            if chain.is_complete {
                stats.o2c_completed += 1;
            }
        }

        stats
    }

    /// Get reference to P2P generator for direct access.
    pub fn p2p_generator(&mut self) -> &mut P2PGenerator {
        &mut self.p2p_generator
    }

    /// Get reference to O2C generator for direct access.
    pub fn o2c_generator(&mut self) -> &mut O2CGenerator {
        &mut self.o2c_generator
    }

    /// Reset all generators.
    pub fn reset(&mut self) {
        self.rng = ChaCha8Rng::seed_from_u64(self.seed);
        self.p2p_generator.reset();
        self.o2c_generator.reset();
    }
}

/// Helper to extract all journal entry-generating documents from flows.
pub fn extract_je_sources(flows: &GeneratedDocumentFlows) -> JournalEntrySources {
    let mut sources = JournalEntrySources::default();

    for chain in &flows.p2p_chains {
        // GR creates JE: DR Inventory, CR GR/IR
        for gr in &chain.goods_receipts {
            sources.goods_receipts.push(gr.clone());
        }

        // Invoice creates JE: DR Expense/GR-IR, CR AP
        if let Some(invoice) = &chain.vendor_invoice {
            sources.vendor_invoices.push(invoice.clone());
        }

        // Payment creates JE: DR AP, CR Bank
        if let Some(payment) = &chain.payment {
            sources.ap_payments.push(payment.clone());
        }
    }

    for chain in &flows.o2c_chains {
        // Delivery/GI creates JE: DR COGS, CR Inventory
        for dlv in &chain.deliveries {
            sources.deliveries.push(dlv.clone());
        }

        // Invoice creates JE: DR AR, CR Revenue
        if let Some(invoice) = &chain.customer_invoice {
            sources.customer_invoices.push(invoice.clone());
        }

        // Receipt creates JE: DR Bank, CR AR
        if let Some(receipt) = &chain.customer_receipt {
            sources.ar_receipts.push(receipt.clone());
        }
    }

    sources
}

/// Sources for journal entry generation.
#[derive(Debug, Default)]
pub struct JournalEntrySources {
    /// Goods receipts (DR Inventory, CR GR/IR)
    pub goods_receipts: Vec<datasynth_core::models::documents::GoodsReceipt>,
    /// Vendor invoices (DR Expense, CR AP)
    pub vendor_invoices: Vec<datasynth_core::models::documents::VendorInvoice>,
    /// AP payments (DR AP, CR Bank)
    pub ap_payments: Vec<datasynth_core::models::documents::Payment>,
    /// Deliveries (DR COGS, CR Inventory)
    pub deliveries: Vec<datasynth_core::models::documents::Delivery>,
    /// Customer invoices (DR AR, CR Revenue)
    pub customer_invoices: Vec<datasynth_core::models::documents::CustomerInvoice>,
    /// AR receipts (DR Bank, CR AR)
    pub ar_receipts: Vec<datasynth_core::models::documents::Payment>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::{
        CreditRating, Customer, CustomerPaymentBehavior, Material, MaterialType, Vendor,
    };

    fn create_test_pools() -> (VendorPool, CustomerPool, MaterialPool) {
        let mut vendors = VendorPool::new();
        for i in 1..=5 {
            vendors.add_vendor(Vendor::new(
                &format!("V-{:06}", i),
                &format!("Vendor {}", i),
                datasynth_core::models::VendorType::Supplier,
            ));
        }

        let mut customers = CustomerPool::new();
        for i in 1..=5 {
            let mut customer = Customer::new(
                &format!("C-{:06}", i),
                &format!("Customer {}", i),
                datasynth_core::models::CustomerType::Corporate,
            );
            customer.credit_rating = CreditRating::A;
            customer.credit_limit = rust_decimal::Decimal::from(1_000_000);
            customer.payment_behavior = CustomerPaymentBehavior::OnTime;
            customers.add_customer(customer);
        }

        let mut materials = MaterialPool::new();
        for i in 1..=10 {
            let mut mat = Material::new(
                format!("MAT-{:06}", i),
                format!("Material {}", i),
                MaterialType::FinishedGood,
            );
            mat.standard_cost = rust_decimal::Decimal::from(50 + i * 10);
            mat.list_price = rust_decimal::Decimal::from(100 + i * 20);
            materials.add_material(mat);
        }

        (vendors, customers, materials)
    }

    #[test]
    fn test_manager_creation() {
        let manager = DocumentChainManager::new(42);
        assert!(manager.config.p2p_to_o2c_ratio == 1.0);
    }

    #[test]
    fn test_generate_flows() {
        let mut manager = DocumentChainManager::new(42);
        let (vendors, customers, materials) = create_test_pools();

        let flows = manager.generate_flows(
            "1000",
            20,
            &vendors,
            &customers,
            &materials,
            (
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            ),
            2024,
            "JSMITH",
        );

        assert_eq!(flows.p2p_chains.len() + flows.o2c_chains.len(), 20);
        assert!(flows.stats.purchase_orders > 0);
        assert!(flows.stats.sales_orders > 0);
    }

    #[test]
    fn test_balanced_flows() {
        let mut manager = DocumentChainManager::new(42);
        let (vendors, customers, materials) = create_test_pools();

        let flows = manager.generate_balanced_flows(
            10,
            "1000",
            &vendors,
            &customers,
            &materials,
            (
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            ),
            2024,
            "JSMITH",
        );

        assert_eq!(flows.p2p_chains.len(), 10);
        assert_eq!(flows.o2c_chains.len(), 10);
    }

    #[test]
    fn test_document_references_collected() {
        let mut manager = DocumentChainManager::new(42);
        let (vendors, customers, materials) = create_test_pools();

        let flows = manager.generate_balanced_flows(
            5,
            "1000",
            &vendors,
            &customers,
            &materials,
            (
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            ),
            2024,
            "JSMITH",
        );

        // Should have document references
        assert!(!flows.document_references.is_empty());
    }

    #[test]
    fn test_stats_calculation() {
        let mut manager = DocumentChainManager::new(42);
        let (vendors, customers, materials) = create_test_pools();

        let flows = manager.generate_balanced_flows(
            5,
            "1000",
            &vendors,
            &customers,
            &materials,
            (
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            ),
            2024,
            "JSMITH",
        );

        let stats = &flows.stats;
        assert_eq!(stats.p2p_chains, 5);
        assert_eq!(stats.o2c_chains, 5);
        assert_eq!(stats.purchase_orders, 5);
        assert_eq!(stats.sales_orders, 5);
    }

    #[test]
    fn test_je_sources_extraction() {
        let mut manager = DocumentChainManager::new(42);
        let (vendors, customers, materials) = create_test_pools();

        let flows = manager.generate_balanced_flows(
            5,
            "1000",
            &vendors,
            &customers,
            &materials,
            (
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            ),
            2024,
            "JSMITH",
        );

        let sources = extract_je_sources(&flows);

        // Should have JE sources from both P2P and O2C
        assert!(!sources.goods_receipts.is_empty());
        assert!(!sources.vendor_invoices.is_empty());
        assert!(!sources.deliveries.is_empty());
        assert!(!sources.customer_invoices.is_empty());
    }

    #[test]
    fn test_custom_ratio() {
        let config = DocumentChainManagerConfig {
            p2p_to_o2c_ratio: 2.0, // 2x P2P
            ..Default::default()
        };

        let mut manager = DocumentChainManager::with_config(42, config);
        let (vendors, customers, materials) = create_test_pools();

        let flows = manager.generate_flows(
            "1000",
            30,
            &vendors,
            &customers,
            &materials,
            (
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            ),
            2024,
            "JSMITH",
        );

        // With 2:1 ratio, should have ~20 P2P and ~10 O2C
        assert!(flows.p2p_chains.len() > flows.o2c_chains.len());
    }

    #[test]
    fn test_reset() {
        let mut manager = DocumentChainManager::new(42);
        let (vendors, customers, materials) = create_test_pools();

        let flows1 = manager.generate_balanced_flows(
            5,
            "1000",
            &vendors,
            &customers,
            &materials,
            (
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            ),
            2024,
            "JSMITH",
        );

        manager.reset();

        let flows2 = manager.generate_balanced_flows(
            5,
            "1000",
            &vendors,
            &customers,
            &materials,
            (
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            ),
            2024,
            "JSMITH",
        );

        // After reset, should get same results
        assert_eq!(
            flows1.p2p_chains[0].purchase_order.header.document_id,
            flows2.p2p_chains[0].purchase_order.header.document_id
        );
    }
}
