//! Property serializers for converting domain model fields into property maps.
//!
//! Each serializer handles a specific entity type (identified by `entity_type()`)
//! and converts the domain model's strongly-typed fields into
//! `HashMap<String, serde_json::Value>` for export.
//!
//! ## Implemented Serializers
//!
//! - [`ControlPropertySerializer`](control::ControlPropertySerializer) — `InternalControl` (code 503)
//! - [`RiskPropertySerializer`](risk::RiskPropertySerializer) — `RiskAssessment` (code 364)
//! - [`JournalEntryPropertySerializer`](journal_entry::JournalEntryPropertySerializer) — `JournalEntry` (code 300)
//! - [`AccountPropertySerializer`](account::AccountPropertySerializer) — `GLAccount` (code 301)
//! - [`EmployeePropertySerializer`](employee::EmployeePropertySerializer) — `Employee` (code 360)
//! - P2P: [`PurchaseOrderPropertySerializer`], [`GoodsReceiptPropertySerializer`], [`VendorInvoicePropertySerializer`], [`PaymentPropertySerializer`]
//! - O2C: [`SalesOrderPropertySerializer`], [`DeliveryPropertySerializer`], [`CustomerInvoicePropertySerializer`]
//! - Banking: [`BankingCustomerPropertySerializer`], [`BankAccountPropertySerializer`], [`BankTransactionPropertySerializer`]
//! - Audit: [`EngagementPropertySerializer`], [`WorkpaperPropertySerializer`], [`EvidencePropertySerializer`], [`FindingPropertySerializer`], [`JudgmentPropertySerializer`]
//! - Audit Procedures: [`ConfirmationPropertySerializer`], [`ConfirmationResponsePropertySerializer`], [`ProcedureStepPropertySerializer`], [`SamplePropertySerializer`], [`AnalyticalProcedurePropertySerializer`], [`IaFunctionPropertySerializer`], [`IaReportPropertySerializer`], [`RelatedPartyPropertySerializer`], [`RelatedPartyTransactionPropertySerializer`]
//! - S2C: [`SourcingProjectPropertySerializer`], [`RfxEventPropertySerializer`], [`SupplierBidPropertySerializer`], [`ProcurementContractPropertySerializer`]
//! - H2R: [`PayrollRunPropertySerializer`], [`TimeEntryPropertySerializer`], [`ExpenseReportPropertySerializer`]
//! - MFG: [`ProductionOrderPropertySerializer`], [`QualityInspectionPropertySerializer`], [`CycleCountPropertySerializer`]

pub mod account;
pub mod audit;
pub mod banking;
pub mod control;
pub mod employee;
pub mod h2r;
pub mod journal_entry;
pub mod mfg;
pub mod o2c;
pub mod p2p;
pub mod risk;
pub mod s2c;

use crate::traits::PropertySerializer;

/// Returns all built-in property serializers.
///
/// Used by [`GraphExportPipeline::standard()`](crate::pipeline::GraphExportPipeline::standard)
/// to register the default set of serializers.
pub fn all_serializers() -> Vec<Box<dyn PropertySerializer>> {
    vec![
        // Governance / Controls
        Box::new(control::ControlPropertySerializer),
        Box::new(risk::RiskPropertySerializer),
        // Accounting
        Box::new(journal_entry::JournalEntryPropertySerializer),
        Box::new(account::AccountPropertySerializer),
        // Master Data
        Box::new(employee::EmployeePropertySerializer),
        // P2P documents
        Box::new(p2p::PurchaseOrderPropertySerializer),
        Box::new(p2p::GoodsReceiptPropertySerializer),
        Box::new(p2p::VendorInvoicePropertySerializer),
        Box::new(p2p::PaymentPropertySerializer),
        // O2C documents
        Box::new(o2c::SalesOrderPropertySerializer),
        Box::new(o2c::DeliveryPropertySerializer),
        Box::new(o2c::CustomerInvoicePropertySerializer),
        // Banking
        Box::new(banking::BankingCustomerPropertySerializer),
        Box::new(banking::BankAccountPropertySerializer),
        Box::new(banking::BankTransactionPropertySerializer),
        // Audit
        Box::new(audit::EngagementPropertySerializer),
        Box::new(audit::WorkpaperPropertySerializer),
        Box::new(audit::EvidencePropertySerializer),
        Box::new(audit::FindingPropertySerializer),
        Box::new(audit::JudgmentPropertySerializer),
        // Audit Procedures (ISA 505/330/530/520/610/550)
        Box::new(audit::ConfirmationPropertySerializer),
        Box::new(audit::ConfirmationResponsePropertySerializer),
        Box::new(audit::ProcedureStepPropertySerializer),
        Box::new(audit::SamplePropertySerializer),
        Box::new(audit::AnalyticalProcedurePropertySerializer),
        Box::new(audit::IaFunctionPropertySerializer),
        Box::new(audit::IaReportPropertySerializer),
        Box::new(audit::RelatedPartyPropertySerializer),
        Box::new(audit::RelatedPartyTransactionPropertySerializer),
        // S2C (Source-to-Contract)
        Box::new(s2c::SourcingProjectPropertySerializer),
        Box::new(s2c::RfxEventPropertySerializer),
        Box::new(s2c::SupplierBidPropertySerializer),
        Box::new(s2c::ProcurementContractPropertySerializer),
        // H2R (Hire-to-Retire)
        Box::new(h2r::PayrollRunPropertySerializer),
        Box::new(h2r::TimeEntryPropertySerializer),
        Box::new(h2r::ExpenseReportPropertySerializer),
        // Manufacturing
        Box::new(mfg::ProductionOrderPropertySerializer),
        Box::new(mfg::QualityInspectionPropertySerializer),
        Box::new(mfg::CycleCountPropertySerializer),
    ]
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_serializers_have_non_overlapping_entity_types() {
        let serializers = all_serializers();
        let mut seen: HashSet<String> = HashSet::new();
        for s in &serializers {
            let et = s.entity_type().to_string();
            assert!(
                seen.insert(et.clone()),
                "Duplicate entity type in serializers: {}",
                et
            );
        }
    }

    #[test]
    fn all_serializers_count() {
        let serializers = all_serializers();
        // 2 (control, risk) + 2 (je, account) + 1 (employee)
        // + 4 (p2p) + 3 (o2c) + 3 (banking) + 5 (audit) + 9 (audit procedures)
        // + 4 (s2c) + 3 (h2r) + 3 (mfg)
        // = 39 total
        assert_eq!(serializers.len(), 39);
    }
}
