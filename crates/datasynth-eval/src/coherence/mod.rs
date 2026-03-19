//! Semantic coherence evaluation module.
//!
//! Validates that generated data maintains accounting coherence including
//! balance sheet equations, subledger reconciliation, and document chain integrity.

pub mod je_risk_scoring;
pub mod ratio_analysis;

mod audit;
mod balance;
mod bank_reconciliation;
mod country_packs;
mod cross_process;
mod document_chain;
mod esg;
mod financial_reporting;
mod fraud_packs;
mod hr_payroll;
mod intercompany;
mod manufacturing;
mod multi_period;
mod multi_table;
mod network;
mod project_accounting;
mod referential;
mod sales_quotes;
mod sourcing;
mod standards;
mod subledger;
mod tax;
mod treasury;

pub use audit::{
    AuditEvaluation, AuditEvaluator, AuditFindingData, AuditRiskData, AuditThresholds,
    MaterialityData, WorkpaperData,
};
pub use balance::{AccountType, BalanceSheetEvaluation, BalanceSheetEvaluator, BalanceSnapshot};
pub use bank_reconciliation::{
    BankReconciliationEvaluation, BankReconciliationEvaluator, BankReconciliationThresholds,
    ReconciliationData,
};
pub use country_packs::{
    ApprovalLevelData, CountryPackData, CountryPackEvaluation, CountryPackEvaluator,
    CountryPackThresholds, HolidayData, TaxRateData,
};
pub use cross_process::{
    CrossProcessEvaluation, CrossProcessEvaluator, CrossProcessLinkData, CrossProcessThresholds,
};
pub use document_chain::{
    DocumentChainEvaluation, DocumentChainEvaluator, DocumentReferenceData, O2CChainData,
    P2PChainData,
};
pub use esg::{
    EsgEvaluation, EsgEvaluator, EsgThresholds, GovernanceData, SafetyMetricData, SupplierEsgData,
    WaterUsageData,
};
pub use financial_reporting::{
    BudgetVarianceData, FinancialReportingEvaluation, FinancialReportingEvaluator,
    FinancialReportingThresholds, FinancialStatementData, KpiData,
};
pub use fraud_packs::{FraudPackAnalysis, FraudPackAnalyzer, FraudPackData, FraudPackThresholds};
pub use hr_payroll::{
    ExpenseReportData, HrPayrollEvaluation, HrPayrollEvaluator, HrPayrollThresholds,
    PayrollHoursData, PayrollLineItemData, PayrollRunData, TimeEntryData,
};
pub use intercompany::{
    ICMatchingData, ICMatchingEvaluation, ICMatchingEvaluator, UnmatchedICItem,
};
pub use manufacturing::{
    CycleCountData, ManufacturingEvaluation, ManufacturingEvaluator, ManufacturingThresholds,
    ProductionOrderData, QualityInspectionData, RoutingOperationData,
};
pub use multi_period::{
    MultiPeriodAnalysis, MultiPeriodAnalyzer, MultiPeriodThresholds, PeriodData,
};
pub use multi_table::{
    get_o2c_flow_relationships, get_p2p_flow_relationships, AnomalyRecord, CascadeAnomalyAnalysis,
    CascadePath, ConsistencyViolation, MultiTableConsistencyEvaluator, MultiTableData,
    MultiTableEvaluation, TableConsistencyResult, TableRecord, TableRelationship,
    TableRelationshipDef, ViolationType,
};
pub use network::{
    ConcentrationMetrics, NetworkEdge, NetworkEvaluation, NetworkEvaluator, NetworkNode,
    NetworkThresholds, StrengthStats,
};
pub use project_accounting::{
    EarnedValueData, ProjectAccountingEvaluation, ProjectAccountingEvaluator,
    ProjectAccountingThresholds, ProjectRevenueData, RetainageData,
};
pub use referential::{
    EntityReferenceData, ReferentialData, ReferentialIntegrityEvaluation,
    ReferentialIntegrityEvaluator,
};
pub use sales_quotes::{
    QuoteLineData, SalesQuoteData, SalesQuoteEvaluation, SalesQuoteEvaluator, SalesQuoteThresholds,
};
pub use sourcing::{
    BidEvaluationData, ScorecardCoverageData, SourcingEvaluation, SourcingEvaluator,
    SourcingProjectData, SourcingThresholds, SpendAnalysisData,
};
pub use standards::{
    AuditTrailEvaluation, AuditTrailGap, FairValueEvaluation, FrameworkViolation,
    ImpairmentEvaluation, IsaComplianceEvaluation, LeaseAccountingEvaluation,
    LeaseAccountingEvaluator, LeaseEvaluation, PcaobComplianceEvaluation, PerformanceObligation,
    RevenueContract, RevenueRecognitionEvaluation, RevenueRecognitionEvaluator,
    SoxComplianceEvaluation, StandardsComplianceEvaluation, StandardsThresholds,
    VariableConsideration, ViolationSeverity,
};
pub use subledger::{SubledgerEvaluator, SubledgerReconciliationEvaluation};
pub use tax::{
    TaxEvaluation, TaxEvaluator, TaxLineData, TaxReturnData, TaxThresholds, WithholdingData,
};
pub use treasury::{
    CashPositionData, CovenantData, HedgeEffectivenessData, NettingData, TreasuryEvaluation,
    TreasuryEvaluator, TreasuryThresholds,
};
pub use je_risk_scoring::{
    JeRiskScoringResult, RiskAttributeStats, RiskDistribution,
};
pub use ratio_analysis::{
    FinancialRatios, RatioAnalysisResult, RatioCheck,
};

use serde::{Deserialize, Serialize};

/// Combined coherence evaluation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceEvaluation {
    /// Balance sheet validation results.
    pub balance: Option<BalanceSheetEvaluation>,
    /// Subledger reconciliation results.
    pub subledger: Option<SubledgerReconciliationEvaluation>,
    /// Document chain completeness results.
    pub document_chain: Option<DocumentChainEvaluation>,
    /// Intercompany matching results.
    pub intercompany: Option<ICMatchingEvaluation>,
    /// Referential integrity results.
    pub referential: Option<ReferentialIntegrityEvaluation>,
    /// Multi-table consistency results.
    pub multi_table: Option<MultiTableEvaluation>,
    /// Accounting and audit standards compliance results.
    pub standards: Option<StandardsComplianceEvaluation>,
    /// Network/interconnectivity evaluation results.
    pub network: Option<NetworkEvaluation>,
    /// Financial reporting evaluation results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub financial_reporting: Option<FinancialReportingEvaluation>,
    /// HR/payroll evaluation results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hr_payroll: Option<HrPayrollEvaluation>,
    /// Manufacturing evaluation results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manufacturing: Option<ManufacturingEvaluation>,
    /// Bank reconciliation evaluation results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bank_reconciliation: Option<BankReconciliationEvaluation>,
    /// Source-to-contract evaluation results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sourcing: Option<SourcingEvaluation>,
    /// Cross-process link evaluation results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cross_process: Option<CrossProcessEvaluation>,
    /// Audit evaluation results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit: Option<AuditEvaluation>,
    /// Tax evaluation results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tax: Option<TaxEvaluation>,
    /// Treasury evaluation results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub treasury: Option<TreasuryEvaluation>,
    /// Project accounting evaluation results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_accounting: Option<ProjectAccountingEvaluation>,
    /// ESG evaluation results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub esg: Option<EsgEvaluation>,
    /// Sales quote evaluation results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sales_quotes: Option<SalesQuoteEvaluation>,
    /// Country pack evaluation results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country_packs: Option<CountryPackEvaluation>,
    /// Multi-period coherence evaluation results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multi_period: Option<MultiPeriodAnalysis>,
    /// Overall pass/fail status.
    pub passes: bool,
    /// Summary of failed checks.
    pub failures: Vec<String>,
}

impl CoherenceEvaluation {
    /// Create a new empty evaluation.
    pub fn new() -> Self {
        Self {
            balance: None,
            subledger: None,
            document_chain: None,
            intercompany: None,
            referential: None,
            multi_table: None,
            standards: None,
            network: None,
            financial_reporting: None,
            hr_payroll: None,
            manufacturing: None,
            bank_reconciliation: None,
            sourcing: None,
            cross_process: None,
            audit: None,
            tax: None,
            treasury: None,
            project_accounting: None,
            esg: None,
            sales_quotes: None,
            country_packs: None,
            multi_period: None,
            passes: true,
            failures: Vec::new(),
        }
    }

    /// Check all results against thresholds and update pass status.
    pub fn check_thresholds(&mut self, thresholds: &crate::config::EvaluationThresholds) {
        self.failures.clear();

        if let Some(ref balance) = self.balance {
            if !balance.equation_balanced {
                self.failures.push(format!(
                    "Balance sheet equation not balanced (max imbalance: {})",
                    balance.max_imbalance
                ));
            }
        }

        if let Some(ref subledger) = self.subledger {
            if subledger.completeness_score < thresholds.subledger_reconciliation_rate_min {
                self.failures.push(format!(
                    "Subledger reconciliation {} < {} (threshold)",
                    subledger.completeness_score, thresholds.subledger_reconciliation_rate_min
                ));
            }
        }

        if let Some(ref doc_chain) = self.document_chain {
            let min_rate = thresholds.document_chain_completion_min;
            if doc_chain.p2p_completion_rate < min_rate {
                self.failures.push(format!(
                    "P2P chain completion {} < {} (threshold)",
                    doc_chain.p2p_completion_rate, min_rate
                ));
            }
            if doc_chain.o2c_completion_rate < min_rate {
                self.failures.push(format!(
                    "O2C chain completion {} < {} (threshold)",
                    doc_chain.o2c_completion_rate, min_rate
                ));
            }
        }

        if let Some(ref ic) = self.intercompany {
            if ic.match_rate < thresholds.ic_match_rate_min {
                self.failures.push(format!(
                    "IC match rate {} < {} (threshold)",
                    ic.match_rate, thresholds.ic_match_rate_min
                ));
            }
        }

        if let Some(ref referential) = self.referential {
            if referential.overall_integrity_score < thresholds.referential_integrity_min {
                self.failures.push(format!(
                    "Referential integrity {} < {} (threshold)",
                    referential.overall_integrity_score, thresholds.referential_integrity_min
                ));
            }
        }

        if let Some(ref multi_table) = self.multi_table {
            if multi_table.overall_consistency_score < thresholds.referential_integrity_min {
                self.failures.push(format!(
                    "Multi-table consistency {} < {} (threshold)",
                    multi_table.overall_consistency_score, thresholds.referential_integrity_min
                ));
            }
            self.failures.extend(multi_table.issues.clone());
        }

        if let Some(ref mut standards_eval) = self.standards.clone() {
            let standards_thresholds = StandardsThresholds::default();
            standards_eval.check_thresholds(&standards_thresholds);
            self.failures.extend(standards_eval.failures.clone());
        }

        if let Some(ref network_eval) = self.network {
            if !network_eval.passes {
                self.failures.extend(network_eval.issues.clone());
            }
        }

        // New evaluators: propagate issues
        if let Some(ref eval) = self.financial_reporting {
            if !eval.passes {
                self.failures.extend(eval.issues.clone());
            }
        }
        if let Some(ref eval) = self.hr_payroll {
            if !eval.passes {
                self.failures.extend(eval.issues.clone());
            }
        }
        if let Some(ref eval) = self.manufacturing {
            if !eval.passes {
                self.failures.extend(eval.issues.clone());
            }
        }
        if let Some(ref eval) = self.bank_reconciliation {
            if !eval.passes {
                self.failures.extend(eval.issues.clone());
            }
        }
        if let Some(ref eval) = self.sourcing {
            if !eval.passes {
                self.failures.extend(eval.issues.clone());
            }
        }
        if let Some(ref eval) = self.cross_process {
            if !eval.passes {
                self.failures.extend(eval.issues.clone());
            }
        }
        if let Some(ref eval) = self.audit {
            if !eval.passes {
                self.failures.extend(eval.issues.clone());
            }
        }
        if let Some(ref eval) = self.tax {
            if !eval.passes {
                self.failures.extend(eval.issues.clone());
            }
        }
        if let Some(ref eval) = self.treasury {
            if !eval.passes {
                self.failures.extend(eval.issues.clone());
            }
        }
        if let Some(ref eval) = self.project_accounting {
            if !eval.passes {
                self.failures.extend(eval.issues.clone());
            }
        }
        if let Some(ref eval) = self.esg {
            if !eval.passes {
                self.failures.extend(eval.issues.clone());
            }
        }
        if let Some(ref eval) = self.sales_quotes {
            if !eval.passes {
                self.failures.extend(eval.issues.clone());
            }
        }
        if let Some(ref eval) = self.country_packs {
            if !eval.passes {
                self.failures.extend(eval.issues.clone());
            }
        }
        if let Some(ref eval) = self.multi_period {
            if !eval.passes {
                self.failures.extend(eval.issues.clone());
            }
        }

        self.passes = self.failures.is_empty();
    }
}

impl Default for CoherenceEvaluation {
    fn default() -> Self {
        Self::new()
    }
}
