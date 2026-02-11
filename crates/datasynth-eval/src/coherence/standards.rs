//! Accounting and Audit Standards Compliance Evaluation.
//!
//! This module validates that generated data complies with accounting and
//! auditing standards including IFRS, US GAAP, ISA, SOX, and PCAOB.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ============================================================================
// Standards Compliance Evaluation
// ============================================================================

/// Comprehensive standards compliance evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardsComplianceEvaluation {
    /// Revenue recognition compliance (ASC 606/IFRS 15).
    pub revenue_recognition: Option<RevenueRecognitionEvaluation>,
    /// Lease accounting compliance (ASC 842/IFRS 16).
    pub lease_accounting: Option<LeaseAccountingEvaluation>,
    /// Fair value measurement compliance (ASC 820/IFRS 13).
    pub fair_value: Option<FairValueEvaluation>,
    /// Impairment testing compliance (ASC 360/IAS 36).
    pub impairment: Option<ImpairmentEvaluation>,
    /// ISA compliance.
    pub isa_compliance: Option<IsaComplianceEvaluation>,
    /// SOX compliance.
    pub sox_compliance: Option<SoxComplianceEvaluation>,
    /// PCAOB compliance.
    pub pcaob_compliance: Option<PcaobComplianceEvaluation>,
    /// Audit trail completeness.
    pub audit_trail: Option<AuditTrailEvaluation>,
    /// Overall pass/fail status.
    pub passes: bool,
    /// Summary of failures.
    pub failures: Vec<String>,
    /// Warnings (non-critical issues).
    pub warnings: Vec<String>,
}

impl StandardsComplianceEvaluation {
    /// Create a new empty evaluation.
    pub fn new() -> Self {
        Self {
            revenue_recognition: None,
            lease_accounting: None,
            fair_value: None,
            impairment: None,
            isa_compliance: None,
            sox_compliance: None,
            pcaob_compliance: None,
            audit_trail: None,
            passes: true,
            failures: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Check all evaluations against thresholds.
    pub fn check_thresholds(&mut self, thresholds: &StandardsThresholds) {
        self.failures.clear();
        self.warnings.clear();

        // Check revenue recognition
        if let Some(ref rev) = self.revenue_recognition {
            if rev.po_allocation_compliance < thresholds.min_po_allocation_compliance {
                self.failures.push(format!(
                    "PO allocation compliance {:.2}% < {:.2}% (threshold)",
                    rev.po_allocation_compliance * 100.0,
                    thresholds.min_po_allocation_compliance * 100.0
                ));
            }
            if rev.timing_compliance < thresholds.min_revenue_timing_compliance {
                self.failures.push(format!(
                    "Revenue timing compliance {:.2}% < {:.2}% (threshold)",
                    rev.timing_compliance * 100.0,
                    thresholds.min_revenue_timing_compliance * 100.0
                ));
            }
        }

        // Check lease accounting
        if let Some(ref lease) = self.lease_accounting {
            if lease.classification_accuracy < thresholds.min_lease_classification_accuracy {
                self.failures.push(format!(
                    "Lease classification accuracy {:.2}% < {:.2}% (threshold)",
                    lease.classification_accuracy * 100.0,
                    thresholds.min_lease_classification_accuracy * 100.0
                ));
            }
            if lease.rou_asset_accuracy < thresholds.min_rou_asset_accuracy {
                self.failures.push(format!(
                    "ROU asset calculation accuracy {:.2}% < {:.2}% (threshold)",
                    lease.rou_asset_accuracy * 100.0,
                    thresholds.min_rou_asset_accuracy * 100.0
                ));
            }
        }

        // Check fair value
        if let Some(ref fv) = self.fair_value {
            if fv.hierarchy_compliance < thresholds.min_fair_value_hierarchy_compliance {
                self.failures.push(format!(
                    "Fair value hierarchy compliance {:.2}% < {:.2}% (threshold)",
                    fv.hierarchy_compliance * 100.0,
                    thresholds.min_fair_value_hierarchy_compliance * 100.0
                ));
            }
        }

        // Check impairment
        if let Some(ref imp) = self.impairment {
            if imp.trigger_recognition_rate < thresholds.min_impairment_trigger_rate {
                self.warnings.push(format!(
                    "Impairment trigger recognition rate {:.2}% < {:.2}% (warning)",
                    imp.trigger_recognition_rate * 100.0,
                    thresholds.min_impairment_trigger_rate * 100.0
                ));
            }
        }

        // Check ISA compliance
        if let Some(ref isa) = self.isa_compliance {
            if isa.coverage_rate < thresholds.min_isa_coverage {
                self.failures.push(format!(
                    "ISA coverage rate {:.2}% < {:.2}% (threshold)",
                    isa.coverage_rate * 100.0,
                    thresholds.min_isa_coverage * 100.0
                ));
            }
        }

        // Check SOX compliance
        if let Some(ref sox) = self.sox_compliance {
            if sox.control_coverage < thresholds.min_sox_control_coverage {
                self.failures.push(format!(
                    "SOX control coverage {:.2}% < {:.2}% (threshold)",
                    sox.control_coverage * 100.0,
                    thresholds.min_sox_control_coverage * 100.0
                ));
            }
        }

        // Check audit trail
        if let Some(ref trail) = self.audit_trail {
            if trail.completeness < thresholds.min_audit_trail_completeness {
                self.failures.push(format!(
                    "Audit trail completeness {:.2}% < {:.2}% (threshold)",
                    trail.completeness * 100.0,
                    thresholds.min_audit_trail_completeness * 100.0
                ));
            }
        }

        self.passes = self.failures.is_empty();
    }
}

impl Default for StandardsComplianceEvaluation {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Revenue Recognition Evaluation (ASC 606 / IFRS 15)
// ============================================================================

/// Revenue recognition compliance evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueRecognitionEvaluation {
    /// Total contracts evaluated.
    pub total_contracts: usize,
    /// Contracts with valid structure.
    pub valid_contracts: usize,
    /// Performance obligation allocation compliance (0.0-1.0).
    pub po_allocation_compliance: f64,
    /// Revenue timing compliance (0.0-1.0).
    pub timing_compliance: f64,
    /// Variable consideration estimation compliance.
    pub variable_consideration_compliance: f64,
    /// Contract modification handling compliance.
    pub modification_compliance: f64,
    /// Framework-specific rule violations.
    pub framework_violations: Vec<FrameworkViolation>,
    /// Contracts with balanced revenue/deferred revenue.
    pub balanced_contracts: usize,
}

impl RevenueRecognitionEvaluation {
    /// Create a new evaluation.
    pub fn new() -> Self {
        Self {
            total_contracts: 0,
            valid_contracts: 0,
            po_allocation_compliance: 1.0,
            timing_compliance: 1.0,
            variable_consideration_compliance: 1.0,
            modification_compliance: 1.0,
            framework_violations: Vec::new(),
            balanced_contracts: 0,
        }
    }
}

impl Default for RevenueRecognitionEvaluation {
    fn default() -> Self {
        Self::new()
    }
}

/// Revenue recognition evaluator.
pub struct RevenueRecognitionEvaluator;

impl RevenueRecognitionEvaluator {
    /// Evaluate revenue recognition contracts.
    pub fn evaluate(contracts: &[RevenueContract]) -> RevenueRecognitionEvaluation {
        let mut eval = RevenueRecognitionEvaluation::new();
        eval.total_contracts = contracts.len();

        if contracts.is_empty() {
            return eval;
        }

        let mut valid_count = 0;
        let mut po_compliant = 0;
        let mut timing_compliant = 0;
        let mut vc_compliant = 0;
        let mut balanced = 0;

        for contract in contracts {
            // Check contract validity
            if contract.is_valid() {
                valid_count += 1;
            }

            // Check PO allocation (must sum to transaction price)
            if contract.check_po_allocation() {
                po_compliant += 1;
            } else {
                eval.framework_violations.push(FrameworkViolation {
                    standard: "ASC 606-10-32-28".to_string(),
                    description: format!(
                        "Contract {} PO allocation doesn't equal transaction price",
                        contract.contract_id
                    ),
                    severity: ViolationSeverity::Error,
                });
            }

            // Check revenue timing
            if contract.check_revenue_timing() {
                timing_compliant += 1;
            }

            // Check variable consideration
            if contract.variable_consideration.is_none() || contract.check_variable_consideration()
            {
                vc_compliant += 1;
            }

            // Check revenue/deferred balance
            if contract.check_balance() {
                balanced += 1;
            }
        }

        eval.valid_contracts = valid_count;
        eval.po_allocation_compliance = po_compliant as f64 / contracts.len() as f64;
        eval.timing_compliance = timing_compliant as f64 / contracts.len() as f64;
        eval.variable_consideration_compliance = vc_compliant as f64 / contracts.len() as f64;
        eval.balanced_contracts = balanced;

        eval
    }
}

/// Revenue contract for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueContract {
    /// Contract identifier.
    pub contract_id: String,
    /// Transaction price.
    pub transaction_price: Decimal,
    /// Performance obligations.
    pub performance_obligations: Vec<PerformanceObligation>,
    /// Variable consideration.
    pub variable_consideration: Option<VariableConsideration>,
    /// Total revenue recognized.
    pub revenue_recognized: Decimal,
    /// Total deferred revenue.
    pub deferred_revenue: Decimal,
}

impl RevenueContract {
    /// Check if contract structure is valid.
    pub fn is_valid(&self) -> bool {
        !self.performance_obligations.is_empty() && self.transaction_price > dec!(0)
    }

    /// Check PO allocation sums to transaction price.
    pub fn check_po_allocation(&self) -> bool {
        let allocated: Decimal = self
            .performance_obligations
            .iter()
            .map(|po| po.allocated_amount)
            .sum();
        (allocated - self.transaction_price).abs() < dec!(0.01)
    }

    /// Check revenue timing is consistent with satisfaction.
    pub fn check_revenue_timing(&self) -> bool {
        for po in &self.performance_obligations {
            let expected_revenue = po.allocated_amount * po.satisfaction_percent;
            let tolerance = po.allocated_amount * dec!(0.01); // 1% tolerance
            if (po.recognized_revenue - expected_revenue).abs() > tolerance {
                return false;
            }
        }
        true
    }

    /// Check variable consideration is within constraint.
    pub fn check_variable_consideration(&self) -> bool {
        if let Some(ref vc) = self.variable_consideration {
            // Variable consideration should be constrained (not overly optimistic)
            vc.constrained_amount <= vc.expected_amount
        } else {
            true
        }
    }

    /// Check revenue + deferred = transaction price.
    pub fn check_balance(&self) -> bool {
        let total = self.revenue_recognized + self.deferred_revenue;
        (total - self.transaction_price).abs() < dec!(0.01)
    }
}

/// Performance obligation for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceObligation {
    /// Obligation ID.
    pub obligation_id: String,
    /// Allocated amount.
    pub allocated_amount: Decimal,
    /// Satisfaction percentage (0.0-1.0 as Decimal).
    pub satisfaction_percent: Decimal,
    /// Revenue recognized.
    pub recognized_revenue: Decimal,
    /// Whether satisfaction is over time.
    pub over_time: bool,
}

/// Variable consideration for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableConsideration {
    /// Expected amount.
    pub expected_amount: Decimal,
    /// Constrained amount (after applying constraint).
    pub constrained_amount: Decimal,
}

// ============================================================================
// Lease Accounting Evaluation (ASC 842 / IFRS 16)
// ============================================================================

/// Lease accounting compliance evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseAccountingEvaluation {
    /// Total leases evaluated.
    pub total_leases: usize,
    /// Finance leases count.
    pub finance_leases: usize,
    /// Operating leases count.
    pub operating_leases: usize,
    /// Classification accuracy.
    pub classification_accuracy: f64,
    /// ROU asset calculation accuracy.
    pub rou_asset_accuracy: f64,
    /// Lease liability accuracy.
    pub lease_liability_accuracy: f64,
    /// Discount rate reasonableness score.
    pub discount_rate_reasonableness: f64,
    /// Framework violations.
    pub framework_violations: Vec<FrameworkViolation>,
}

impl LeaseAccountingEvaluation {
    /// Create a new evaluation.
    pub fn new() -> Self {
        Self {
            total_leases: 0,
            finance_leases: 0,
            operating_leases: 0,
            classification_accuracy: 1.0,
            rou_asset_accuracy: 1.0,
            lease_liability_accuracy: 1.0,
            discount_rate_reasonableness: 1.0,
            framework_violations: Vec::new(),
        }
    }
}

impl Default for LeaseAccountingEvaluation {
    fn default() -> Self {
        Self::new()
    }
}

/// Lease for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseEvaluation {
    /// Lease identifier.
    pub lease_id: String,
    /// Lease term in months.
    pub lease_term_months: u32,
    /// Asset useful life in months.
    pub asset_useful_life_months: u32,
    /// Present value ratio (PV / fair value).
    pub pv_ratio: f64,
    /// Classified as finance lease.
    pub is_finance: bool,
    /// Framework used.
    pub framework: String,
    /// ROU asset initial measurement.
    pub rou_asset_initial: Decimal,
    /// Lease liability initial measurement.
    pub lease_liability_initial: Decimal,
    /// Discount rate used.
    pub discount_rate: f64,
}

impl LeaseEvaluation {
    /// Check US GAAP classification (bright-line tests).
    pub fn check_us_gaap_classification(&self) -> bool {
        let term_ratio = self.lease_term_months as f64 / self.asset_useful_life_months as f64;

        // Bright-line tests
        let should_be_finance = term_ratio >= 0.75 || self.pv_ratio >= 0.90;

        self.is_finance == should_be_finance
    }

    /// Check IFRS classification (principles-based).
    pub fn check_ifrs_classification(&self) -> bool {
        // IFRS doesn't have bright-lines but similar indicators
        let term_ratio = self.lease_term_months as f64 / self.asset_useful_life_months as f64;

        // For evaluation, we use similar thresholds as indicators
        let indicators_suggest_finance = term_ratio >= 0.75 || self.pv_ratio >= 0.90;

        // IFRS is principles-based so some deviation is acceptable
        self.is_finance == indicators_suggest_finance
    }

    /// Check ROU asset equals lease liability (plus prepayments, less incentives).
    pub fn check_rou_equals_liability(&self) -> bool {
        // For simplicity, check they're approximately equal
        let diff = (self.rou_asset_initial - self.lease_liability_initial).abs();
        diff < self.lease_liability_initial * dec!(0.05) // 5% tolerance
    }
}

/// Lease accounting evaluator.
pub struct LeaseAccountingEvaluator;

impl LeaseAccountingEvaluator {
    /// Evaluate lease accounting compliance.
    pub fn evaluate(leases: &[LeaseEvaluation], framework: &str) -> LeaseAccountingEvaluation {
        let mut eval = LeaseAccountingEvaluation::new();
        eval.total_leases = leases.len();

        if leases.is_empty() {
            return eval;
        }

        let mut classification_correct = 0;
        let mut rou_accurate = 0;

        for lease in leases {
            if lease.is_finance {
                eval.finance_leases += 1;
            } else {
                eval.operating_leases += 1;
            }

            // Check classification
            let classification_ok = if framework == "us_gaap" {
                lease.check_us_gaap_classification()
            } else {
                lease.check_ifrs_classification()
            };

            if classification_ok {
                classification_correct += 1;
            } else {
                eval.framework_violations.push(FrameworkViolation {
                    standard: if framework == "us_gaap" {
                        "ASC 842-10-25-2".to_string()
                    } else {
                        "IFRS 16.63".to_string()
                    },
                    description: format!(
                        "Lease {} classification may be incorrect",
                        lease.lease_id
                    ),
                    severity: ViolationSeverity::Warning,
                });
            }

            // Check ROU/liability matching
            if lease.check_rou_equals_liability() {
                rou_accurate += 1;
            }

            // Check discount rate reasonableness (2-15% typical range)
            if lease.discount_rate < 0.02 || lease.discount_rate > 0.15 {
                eval.framework_violations.push(FrameworkViolation {
                    standard: "ASC 842-20-30-3".to_string(),
                    description: format!(
                        "Lease {} discount rate {:.2}% unusual",
                        lease.lease_id,
                        lease.discount_rate * 100.0
                    ),
                    severity: ViolationSeverity::Warning,
                });
            }
        }

        eval.classification_accuracy = classification_correct as f64 / leases.len() as f64;
        eval.rou_asset_accuracy = rou_accurate as f64 / leases.len() as f64;
        eval.lease_liability_accuracy = eval.rou_asset_accuracy; // Same calculation

        eval
    }
}

// ============================================================================
// Fair Value Measurement Evaluation (ASC 820 / IFRS 13)
// ============================================================================

/// Fair value measurement compliance evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairValueEvaluation {
    /// Total measurements evaluated.
    pub total_measurements: usize,
    /// Level 1 measurements (quoted prices).
    pub level_1_count: usize,
    /// Level 2 measurements (observable inputs).
    pub level_2_count: usize,
    /// Level 3 measurements (unobservable inputs).
    pub level_3_count: usize,
    /// Hierarchy compliance (0.0-1.0).
    pub hierarchy_compliance: f64,
    /// Valuation technique consistency.
    pub technique_consistency: f64,
    /// Framework violations.
    pub framework_violations: Vec<FrameworkViolation>,
}

impl FairValueEvaluation {
    /// Create a new evaluation.
    pub fn new() -> Self {
        Self {
            total_measurements: 0,
            level_1_count: 0,
            level_2_count: 0,
            level_3_count: 0,
            hierarchy_compliance: 1.0,
            technique_consistency: 1.0,
            framework_violations: Vec::new(),
        }
    }
}

impl Default for FairValueEvaluation {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Impairment Testing Evaluation (ASC 360 / IAS 36)
// ============================================================================

/// Impairment testing compliance evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpairmentEvaluation {
    /// Total tests evaluated.
    pub total_tests: usize,
    /// Tests with proper trigger recognition.
    pub triggered_properly: usize,
    /// Trigger recognition rate.
    pub trigger_recognition_rate: f64,
    /// Tests with valid recoverable amounts.
    pub valid_recoverable_amounts: usize,
    /// Impairment losses recognized.
    pub impairment_losses: usize,
    /// Reversals (IFRS only, disallowed for goodwill).
    pub reversals: usize,
    /// Invalid reversals (GAAP or goodwill).
    pub invalid_reversals: usize,
    /// Framework violations.
    pub framework_violations: Vec<FrameworkViolation>,
}

impl ImpairmentEvaluation {
    /// Create a new evaluation.
    pub fn new() -> Self {
        Self {
            total_tests: 0,
            triggered_properly: 0,
            trigger_recognition_rate: 1.0,
            valid_recoverable_amounts: 0,
            impairment_losses: 0,
            reversals: 0,
            invalid_reversals: 0,
            framework_violations: Vec::new(),
        }
    }
}

impl Default for ImpairmentEvaluation {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ISA Compliance Evaluation
// ============================================================================

/// ISA compliance evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsaComplianceEvaluation {
    /// Standards covered.
    pub standards_covered: HashSet<String>,
    /// Total requirements mapped.
    pub total_requirements: usize,
    /// Requirements addressed.
    pub requirements_addressed: usize,
    /// Coverage rate.
    pub coverage_rate: f64,
    /// Procedures with ISA mapping.
    pub procedures_mapped: usize,
    /// Unmapped procedures.
    pub procedures_unmapped: usize,
    /// Critical gaps identified.
    pub critical_gaps: Vec<String>,
}

impl IsaComplianceEvaluation {
    /// Create a new evaluation.
    pub fn new() -> Self {
        Self {
            standards_covered: HashSet::new(),
            total_requirements: 0,
            requirements_addressed: 0,
            coverage_rate: 1.0,
            procedures_mapped: 0,
            procedures_unmapped: 0,
            critical_gaps: Vec::new(),
        }
    }
}

impl Default for IsaComplianceEvaluation {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SOX Compliance Evaluation
// ============================================================================

/// SOX compliance evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoxComplianceEvaluation {
    /// Section 302 certifications present.
    pub section_302_certifications: usize,
    /// Section 404 assessments present.
    pub section_404_assessments: usize,
    /// Key controls identified.
    pub key_controls: usize,
    /// Key controls tested.
    pub controls_tested: usize,
    /// Control coverage (tested/total).
    pub control_coverage: f64,
    /// Material weaknesses identified.
    pub material_weaknesses: usize,
    /// Significant deficiencies identified.
    pub significant_deficiencies: usize,
    /// Deficiency classifications valid.
    pub valid_classifications: usize,
}

impl SoxComplianceEvaluation {
    /// Create a new evaluation.
    pub fn new() -> Self {
        Self {
            section_302_certifications: 0,
            section_404_assessments: 0,
            key_controls: 0,
            controls_tested: 0,
            control_coverage: 1.0,
            material_weaknesses: 0,
            significant_deficiencies: 0,
            valid_classifications: 0,
        }
    }
}

impl Default for SoxComplianceEvaluation {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PCAOB Compliance Evaluation
// ============================================================================

/// PCAOB compliance evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcaobComplianceEvaluation {
    /// Standards covered.
    pub standards_covered: HashSet<String>,
    /// Coverage rate.
    pub coverage_rate: f64,
    /// ICFR opinion present.
    pub icfr_opinion_present: bool,
    /// Critical audit matters documented.
    pub critical_audit_matters: usize,
}

impl PcaobComplianceEvaluation {
    /// Create a new evaluation.
    pub fn new() -> Self {
        Self {
            standards_covered: HashSet::new(),
            coverage_rate: 1.0,
            icfr_opinion_present: false,
            critical_audit_matters: 0,
        }
    }
}

impl Default for PcaobComplianceEvaluation {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Audit Trail Evaluation
// ============================================================================

/// Audit trail completeness evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailEvaluation {
    /// Total trail items.
    pub total_items: usize,
    /// Complete trail items.
    pub complete_items: usize,
    /// Completeness score.
    pub completeness: f64,
    /// Items with risk assessment linkage.
    pub risk_linked: usize,
    /// Items with evidence linkage.
    pub evidence_linked: usize,
    /// Items with conclusion.
    pub concluded: usize,
    /// Gaps identified.
    pub gaps: Vec<AuditTrailGap>,
}

impl AuditTrailEvaluation {
    /// Create a new evaluation.
    pub fn new() -> Self {
        Self {
            total_items: 0,
            complete_items: 0,
            completeness: 1.0,
            risk_linked: 0,
            evidence_linked: 0,
            concluded: 0,
            gaps: Vec::new(),
        }
    }
}

impl Default for AuditTrailEvaluation {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit trail gap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailGap {
    /// Trail item ID.
    pub trail_id: String,
    /// Gap type.
    pub gap_type: String,
    /// Description.
    pub description: String,
}

// ============================================================================
// Framework Violation
// ============================================================================

/// A violation of a specific standard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkViolation {
    /// Standard reference (e.g., "ASC 606-10-25-1").
    pub standard: String,
    /// Description of the violation.
    pub description: String,
    /// Severity level.
    pub severity: ViolationSeverity,
}

/// Violation severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    /// Informational - minor issue.
    Info,
    /// Warning - potential issue.
    Warning,
    /// Error - definite violation.
    Error,
    /// Critical - material violation.
    Critical,
}

// ============================================================================
// Thresholds
// ============================================================================

/// Thresholds for standards compliance evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardsThresholds {
    /// Minimum PO allocation compliance.
    pub min_po_allocation_compliance: f64,
    /// Minimum revenue timing compliance.
    pub min_revenue_timing_compliance: f64,
    /// Minimum lease classification accuracy.
    pub min_lease_classification_accuracy: f64,
    /// Minimum ROU asset accuracy.
    pub min_rou_asset_accuracy: f64,
    /// Minimum fair value hierarchy compliance.
    pub min_fair_value_hierarchy_compliance: f64,
    /// Minimum impairment trigger recognition rate.
    pub min_impairment_trigger_rate: f64,
    /// Minimum ISA coverage.
    pub min_isa_coverage: f64,
    /// Minimum SOX control coverage.
    pub min_sox_control_coverage: f64,
    /// Minimum audit trail completeness.
    pub min_audit_trail_completeness: f64,
}

impl Default for StandardsThresholds {
    fn default() -> Self {
        Self {
            min_po_allocation_compliance: 0.95,
            min_revenue_timing_compliance: 0.95,
            min_lease_classification_accuracy: 0.90,
            min_rou_asset_accuracy: 0.95,
            min_fair_value_hierarchy_compliance: 0.95,
            min_impairment_trigger_rate: 0.80,
            min_isa_coverage: 0.90,
            min_sox_control_coverage: 0.95,
            min_audit_trail_completeness: 0.90,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_standards_compliance_evaluation_new() {
        let eval = StandardsComplianceEvaluation::new();
        assert!(eval.passes);
        assert!(eval.failures.is_empty());
    }

    #[test]
    fn test_revenue_contract_validation() {
        let contract = RevenueContract {
            contract_id: "C001".to_string(),
            transaction_price: dec!(10000),
            performance_obligations: vec![
                PerformanceObligation {
                    obligation_id: "PO1".to_string(),
                    allocated_amount: dec!(6000),
                    satisfaction_percent: dec!(1.0),
                    recognized_revenue: dec!(6000),
                    over_time: false,
                },
                PerformanceObligation {
                    obligation_id: "PO2".to_string(),
                    allocated_amount: dec!(4000),
                    satisfaction_percent: dec!(0.5),
                    recognized_revenue: dec!(2000),
                    over_time: true,
                },
            ],
            variable_consideration: None,
            revenue_recognized: dec!(8000),
            deferred_revenue: dec!(2000),
        };

        assert!(contract.is_valid());
        assert!(contract.check_po_allocation());
        assert!(contract.check_revenue_timing());
        assert!(contract.check_balance());
    }

    #[test]
    fn test_revenue_contract_invalid_po_allocation() {
        let contract = RevenueContract {
            contract_id: "C002".to_string(),
            transaction_price: dec!(10000),
            performance_obligations: vec![PerformanceObligation {
                obligation_id: "PO1".to_string(),
                allocated_amount: dec!(5000), // Only half allocated
                satisfaction_percent: dec!(1.0),
                recognized_revenue: dec!(5000),
                over_time: false,
            }],
            variable_consideration: None,
            revenue_recognized: dec!(5000),
            deferred_revenue: dec!(5000),
        };

        assert!(!contract.check_po_allocation());
    }

    #[test]
    fn test_lease_us_gaap_classification() {
        // Finance lease (term >= 75% of useful life)
        let finance_lease = LeaseEvaluation {
            lease_id: "L001".to_string(),
            lease_term_months: 48,
            asset_useful_life_months: 60,
            pv_ratio: 0.85,
            is_finance: true,
            framework: "us_gaap".to_string(),
            rou_asset_initial: dec!(50000),
            lease_liability_initial: dec!(50000),
            discount_rate: 0.05,
        };
        assert!(finance_lease.check_us_gaap_classification());

        // Operating lease
        let operating_lease = LeaseEvaluation {
            lease_id: "L002".to_string(),
            lease_term_months: 24,
            asset_useful_life_months: 120,
            pv_ratio: 0.50,
            is_finance: false,
            framework: "us_gaap".to_string(),
            rou_asset_initial: dec!(20000),
            lease_liability_initial: dec!(20000),
            discount_rate: 0.05,
        };
        assert!(operating_lease.check_us_gaap_classification());
    }

    #[test]
    fn test_standards_thresholds_check() {
        let mut eval = StandardsComplianceEvaluation::new();
        eval.revenue_recognition = Some(RevenueRecognitionEvaluation {
            po_allocation_compliance: 0.90, // Below 0.95 threshold
            timing_compliance: 0.98,
            ..Default::default()
        });

        let thresholds = StandardsThresholds::default();
        eval.check_thresholds(&thresholds);

        assert!(!eval.passes);
        assert_eq!(eval.failures.len(), 1);
        assert!(eval.failures[0].contains("PO allocation"));
    }

    #[test]
    fn test_lease_accounting_evaluator() {
        let leases = vec![
            LeaseEvaluation {
                lease_id: "L001".to_string(),
                lease_term_months: 48,
                asset_useful_life_months: 60,
                pv_ratio: 0.85,
                is_finance: true,
                framework: "us_gaap".to_string(),
                rou_asset_initial: dec!(50000),
                lease_liability_initial: dec!(50000),
                discount_rate: 0.05,
            },
            LeaseEvaluation {
                lease_id: "L002".to_string(),
                lease_term_months: 24,
                asset_useful_life_months: 120,
                pv_ratio: 0.50,
                is_finance: false,
                framework: "us_gaap".to_string(),
                rou_asset_initial: dec!(20000),
                lease_liability_initial: dec!(20000),
                discount_rate: 0.05,
            },
        ];

        let eval = LeaseAccountingEvaluator::evaluate(&leases, "us_gaap");

        assert_eq!(eval.total_leases, 2);
        assert_eq!(eval.finance_leases, 1);
        assert_eq!(eval.operating_leases, 1);
        assert_eq!(eval.classification_accuracy, 1.0);
    }

    #[test]
    fn test_revenue_recognition_evaluator() {
        let contracts = vec![RevenueContract {
            contract_id: "C001".to_string(),
            transaction_price: dec!(10000),
            performance_obligations: vec![PerformanceObligation {
                obligation_id: "PO1".to_string(),
                allocated_amount: dec!(10000),
                satisfaction_percent: dec!(1.0),
                recognized_revenue: dec!(10000),
                over_time: false,
            }],
            variable_consideration: None,
            revenue_recognized: dec!(10000),
            deferred_revenue: dec!(0),
        }];

        let eval = RevenueRecognitionEvaluator::evaluate(&contracts);

        assert_eq!(eval.total_contracts, 1);
        assert_eq!(eval.valid_contracts, 1);
        assert_eq!(eval.po_allocation_compliance, 1.0);
        assert_eq!(eval.timing_compliance, 1.0);
    }
}
