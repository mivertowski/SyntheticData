//! Significant Classes of Transactions (SCOTS) generator per ISA 315.
//!
//! Generates the standard set of SCOTs based on the business processes
//! configured.  Volume and monetary value are derived from actual JE data
//! (count and aggregate amount by business process / account area).
//!
//! # Standard SCOTs generated
//!
//! | SCOT name                     | Process | Type       | Significance |
//! |-------------------------------|---------|------------|--------------|
//! | Revenue — Product Sales       | O2C     | Routine    | High         |
//! | Purchases — Procurement       | P2P     | Routine    | High         |
//! | Payroll                       | H2R     | Routine    | Medium       |
//! | Fixed Asset Additions         | R2R     | Non-Routine| Medium       |
//! | Depreciation                  | R2R     | Estimation | Medium       |
//! | Tax Provision                 | R2R     | Estimation | High         |
//! | ECL / Bad Debt Provision      | R2R     | Estimation | High         |
//! | Period-End Adjustments        | R2R     | Non-Routine| Medium       |
//! | Intercompany Transactions     | IC      | Routine    | High         |
//!
//! Intercompany SCOT is only generated when IC is enabled.

use datasynth_core::models::audit::scots::{
    CriticalPathStage, EstimationComplexity, ProcessingMethod, ScotSignificance,
    ScotTransactionType, SignificantClassOfTransactions,
};
use datasynth_core::models::JournalEntry;
use datasynth_core::utils::seeded_rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ---------------------------------------------------------------------------
// SCOT specification (static template)
// ---------------------------------------------------------------------------

/// Internal specification for a standard SCOT template.
#[derive(Debug, Clone)]
struct ScotSpec {
    scot_name: &'static str,
    business_process: &'static str,
    significance_level: ScotSignificance,
    transaction_type: ScotTransactionType,
    processing_method: ProcessingMethod,
    /// GL account prefixes used to extract volume/value from JEs.
    account_prefixes: &'static [&'static str],
    relevant_assertions: &'static [&'static str],
    related_account_areas: &'static [&'static str],
    estimation_complexity: Option<EstimationComplexity>,
    /// Critical path stages: (name, description, is_automated, control_id_suffix)
    stages: &'static [(&'static str, &'static str, bool, Option<&'static str>)],
    /// Whether this SCOT requires IC to be enabled.
    requires_ic: bool,
}

/// Standard SCOTS per ISA 315 / typical audit scope.
static STANDARD_SCOTS: &[ScotSpec] = &[
    ScotSpec {
        scot_name: "Revenue — Product Sales",
        business_process: "O2C",
        significance_level: ScotSignificance::High,
        transaction_type: ScotTransactionType::Routine,
        processing_method: ProcessingMethod::SemiAutomated,
        account_prefixes: &["4"],
        relevant_assertions: &["Occurrence", "Accuracy", "Cutoff"],
        related_account_areas: &["Revenue", "Trade Receivables"],
        estimation_complexity: None,
        stages: &[
            ("Initiation", "Sales order created by customer or internal sales team", false, Some("C001")),
            ("Recording", "System records SO upon credit check approval and customer master validation", true, Some("C002")),
            ("Processing", "Automated posting to revenue accounts upon goods delivery confirmation", true, Some("C003")),
            ("Reporting", "Revenue aggregated into income statement via automated GL summarisation", true, None),
        ],
        requires_ic: false,
    },
    ScotSpec {
        scot_name: "Purchases — Procurement",
        business_process: "P2P",
        significance_level: ScotSignificance::High,
        transaction_type: ScotTransactionType::Routine,
        processing_method: ProcessingMethod::SemiAutomated,
        account_prefixes: &["5", "6", "2"],
        relevant_assertions: &["Occurrence", "Completeness", "Accuracy"],
        related_account_areas: &["Cost of Sales", "Trade Payables", "Inventory"],
        estimation_complexity: None,
        stages: &[
            ("Initiation", "Purchase requisition raised by department, approved per authority matrix", false, Some("C010")),
            ("Recording", "System generates purchase order from approved requisition", true, Some("C011")),
            ("Processing", "Three-way match (PO / GR / invoice) with system tolerance checks", true, Some("C012")),
            ("Reporting", "Accounts payable and cost postings flow to trial balance automatically", true, None),
        ],
        requires_ic: false,
    },
    ScotSpec {
        scot_name: "Payroll",
        business_process: "H2R",
        significance_level: ScotSignificance::Medium,
        transaction_type: ScotTransactionType::Routine,
        processing_method: ProcessingMethod::SemiAutomated,
        account_prefixes: &["5", "6"],
        relevant_assertions: &["Occurrence", "Accuracy", "Completeness"],
        related_account_areas: &["Cost of Sales", "Accruals"],
        estimation_complexity: None,
        stages: &[
            ("Initiation", "HR confirms headcount and compensation data for the period", false, Some("C020")),
            ("Recording", "Payroll system calculates gross pay, deductions, and net pay per employee", true, Some("C021")),
            ("Processing", "Payroll journal entries posted to GL; bank file generated for payment", true, Some("C022")),
            ("Reporting", "Payroll costs aggregated by cost centre into management and financial reports", true, None),
        ],
        requires_ic: false,
    },
    ScotSpec {
        scot_name: "Fixed Asset Additions",
        business_process: "R2R",
        significance_level: ScotSignificance::Medium,
        transaction_type: ScotTransactionType::NonRoutine,
        processing_method: ProcessingMethod::SemiAutomated,
        account_prefixes: &["1"],
        relevant_assertions: &["Existence", "Rights & Obligations", "Accuracy"],
        related_account_areas: &["Fixed Assets"],
        estimation_complexity: None,
        stages: &[
            ("Initiation", "Capital expenditure request raised, approved per capital authorisation policy", false, Some("C030")),
            ("Recording", "Asset created in fixed asset register with cost, category, and useful life", false, Some("C031")),
            ("Processing", "Capitalisation journal entry posted; asset available for depreciation", true, None),
            ("Reporting", "Fixed assets reported on balance sheet net of accumulated depreciation", true, None),
        ],
        requires_ic: false,
    },
    ScotSpec {
        scot_name: "Depreciation",
        business_process: "R2R",
        significance_level: ScotSignificance::Medium,
        transaction_type: ScotTransactionType::Estimation,
        processing_method: ProcessingMethod::FullyAutomated,
        account_prefixes: &["1", "5", "6"],
        relevant_assertions: &["Accuracy", "Valuation & Allocation"],
        related_account_areas: &["Fixed Assets", "Cost of Sales"],
        estimation_complexity: Some(EstimationComplexity::Simple),
        stages: &[
            ("Initiation", "Period-end close triggers automated depreciation run in asset module", true, Some("C040")),
            ("Recording", "System calculates depreciation per asset based on cost, method, and useful life", true, Some("C041")),
            ("Processing", "Depreciation journal entry posted to GL (Dr: Dep Expense / Cr: Accum Dep)", true, None),
            ("Reporting", "Depreciation charge flows to income statement; net book value updated on balance sheet", true, None),
        ],
        requires_ic: false,
    },
    ScotSpec {
        scot_name: "Tax Provision",
        business_process: "R2R",
        significance_level: ScotSignificance::High,
        transaction_type: ScotTransactionType::Estimation,
        processing_method: ProcessingMethod::Manual,
        account_prefixes: &["3", "2"],
        relevant_assertions: &["Accuracy", "Valuation & Allocation", "Completeness (Balance)"],
        related_account_areas: &["Tax", "Equity"],
        estimation_complexity: Some(EstimationComplexity::Complex),
        stages: &[
            ("Initiation", "Tax team prepares provision calculation based on pre-tax income and timing differences", false, Some("C050")),
            ("Recording", "Current and deferred tax spreadsheet reviewed and approved by tax director", false, Some("C051")),
            ("Processing", "Manual journal entry posted for current tax payable and deferred tax asset/liability", false, Some("C052")),
            ("Reporting", "Tax charge reported in income statement; deferred tax balance on balance sheet", true, None),
        ],
        requires_ic: false,
    },
    ScotSpec {
        scot_name: "ECL / Bad Debt Provision",
        business_process: "R2R",
        significance_level: ScotSignificance::High,
        transaction_type: ScotTransactionType::Estimation,
        processing_method: ProcessingMethod::Manual,
        account_prefixes: &["1"],
        relevant_assertions: &["Valuation & Allocation", "Completeness (Balance)"],
        related_account_areas: &["Trade Receivables", "Provisions"],
        estimation_complexity: Some(EstimationComplexity::Moderate),
        stages: &[
            ("Initiation", "Finance team reviews AR aging and customer credit risk at period end", false, Some("C060")),
            ("Recording", "ECL / provision matrix applied; individual customer assessments for significant debtors", false, Some("C061")),
            ("Processing", "Provision journal entry posted (Dr: Bad Debt Expense / Cr: Provision for Doubtful Debts)", false, None),
            ("Reporting", "Net receivables (after provision) reported on balance sheet; bad debt expense in P&L", true, None),
        ],
        requires_ic: false,
    },
    ScotSpec {
        scot_name: "Period-End Adjustments",
        business_process: "R2R",
        significance_level: ScotSignificance::Medium,
        transaction_type: ScotTransactionType::NonRoutine,
        processing_method: ProcessingMethod::Manual,
        account_prefixes: &["3", "4", "5", "6"],
        relevant_assertions: &["Accuracy", "Cutoff", "Occurrence"],
        related_account_areas: &["Accruals", "Revenue", "Cost of Sales"],
        estimation_complexity: None,
        stages: &[
            ("Initiation", "Close checklist triggers accrual and prepayment review at period end", false, None),
            ("Recording", "Preparer calculates and documents accruals based on invoices received / services incurred", false, Some("C070")),
            ("Processing", "Manual journal entries posted and reviewed by controller before period close", false, Some("C071")),
            ("Reporting", "Accruals and prepayments reported in financial statements per cut-off policy", true, None),
        ],
        requires_ic: false,
    },
    ScotSpec {
        scot_name: "Intercompany Transactions",
        business_process: "IC",
        significance_level: ScotSignificance::High,
        transaction_type: ScotTransactionType::Routine,
        processing_method: ProcessingMethod::SemiAutomated,
        account_prefixes: &["1", "2", "3", "4"],
        relevant_assertions: &["Occurrence", "Accuracy", "Completeness"],
        related_account_areas: &["Related Parties", "Revenue", "Cost of Sales"],
        estimation_complexity: None,
        stages: &[
            ("Initiation", "IC transactions initiated by business units per transfer pricing agreements", false, Some("C080")),
            ("Recording", "IC netting system captures matching transactions across entities", true, Some("C081")),
            ("Processing", "Automated matching engine reconciles IC balances; unmatched items flagged for resolution", true, Some("C082")),
            ("Reporting", "IC balances eliminated on consolidation; residual differences reported", true, None),
        ],
        requires_ic: true,
    },
];

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the SCOTS generator.
#[derive(Debug, Clone)]
pub struct ScotsGeneratorConfig {
    /// Whether intercompany transactions are in scope (generates IC SCOT).
    pub intercompany_enabled: bool,
    /// Minimum synthetic volume for SCOTs when no JE data is available.
    pub min_volume: usize,
    /// Maximum synthetic volume.
    pub max_volume: usize,
}

impl Default for ScotsGeneratorConfig {
    fn default() -> Self {
        Self {
            intercompany_enabled: false,
            min_volume: 50,
            max_volume: 10_000,
        }
    }
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generator for ISA 315 Significant Classes of Transactions.
pub struct ScotsGenerator {
    rng: ChaCha8Rng,
    config: ScotsGeneratorConfig,
}

impl ScotsGenerator {
    /// Create a new generator with default configuration.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0x315A), // discriminator for ISA 315 SCOTS
            config: ScotsGeneratorConfig::default(),
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: ScotsGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0x315A),
            config,
        }
    }

    /// Generate standard SCOTs for a single entity.
    ///
    /// Volume and monetary values are derived from the supplied JE slice.
    /// When no JEs are present for a particular SCOT, synthetic estimates are used.
    pub fn generate_for_entity(
        &mut self,
        entity_code: &str,
        entries: &[JournalEntry],
    ) -> Vec<SignificantClassOfTransactions> {
        let mut scots = Vec::new();

        for spec in STANDARD_SCOTS {
            if spec.requires_ic && !self.config.intercompany_enabled {
                continue;
            }

            let scot = self.build_scot(entity_code, spec, entries);
            scots.push(scot);
        }

        scots
    }

    /// Build a single SCOT from a specification and JE data.
    fn build_scot(
        &mut self,
        entity_code: &str,
        spec: &ScotSpec,
        entries: &[JournalEntry],
    ) -> SignificantClassOfTransactions {
        let (volume, monetary_value) = self.extract_volume_and_value(entity_code, spec, entries);

        let id = format!(
            "SCOT-{}-{}",
            entity_code,
            spec.scot_name.replace([' ', '—', '-', '/'], "_").to_uppercase(),
        );

        let critical_path = spec
            .stages
            .iter()
            .map(|(name, desc, is_auto, ctrl_id)| CriticalPathStage {
                stage_name: name.to_string(),
                description: desc.to_string(),
                is_automated: *is_auto,
                key_control_id: ctrl_id.map(|c| format!("{entity_code}-{c}")),
            })
            .collect();

        SignificantClassOfTransactions {
            id,
            entity_code: entity_code.to_string(),
            scot_name: spec.scot_name.to_string(),
            business_process: spec.business_process.to_string(),
            significance_level: spec.significance_level,
            transaction_type: spec.transaction_type,
            processing_method: spec.processing_method,
            volume,
            monetary_value,
            critical_path,
            relevant_assertions: spec
                .relevant_assertions
                .iter()
                .map(|s| s.to_string())
                .collect(),
            related_account_areas: spec
                .related_account_areas
                .iter()
                .map(|s| s.to_string())
                .collect(),
            estimation_complexity: spec.estimation_complexity,
        }
    }

    /// Derive volume (count) and monetary value from JEs for the given SCOT.
    ///
    /// Matches JE lines by account prefix.  Falls back to synthetic values
    /// when no matching entries are found.
    fn extract_volume_and_value(
        &mut self,
        entity_code: &str,
        spec: &ScotSpec,
        entries: &[JournalEntry],
    ) -> (usize, Decimal) {
        use rand::Rng;

        // Count JEs and sum their debit amounts for matching accounts
        let matching_entries: Vec<&JournalEntry> = entries
            .iter()
            .filter(|e| e.company_code() == entity_code)
            .filter(|e| {
                e.lines.iter().any(|l| {
                    spec.account_prefixes
                        .iter()
                        .any(|&p| l.account_code.starts_with(p))
                })
            })
            .collect();

        if !matching_entries.is_empty() {
            let volume = matching_entries.len();
            let value: Decimal = matching_entries
                .iter()
                .flat_map(|e| e.lines.iter())
                .filter(|l| {
                    spec.account_prefixes
                        .iter()
                        .any(|&p| l.account_code.starts_with(p))
                })
                .map(|l| l.debit_amount + l.credit_amount)
                .sum::<Decimal>()
                / dec!(2); // avoid double-counting debit+credit
            (volume.max(1), value.max(dec!(1)))
        } else {
            // Synthetic fallback
            let volume = self.rng.random_range(self.config.min_volume..=self.config.max_volume);
            let avg_txn = Decimal::from(self.rng.random_range(1_000_i64..=50_000_i64));
            let value = (Decimal::from(volume as i64) * avg_txn).round_dp(0);
            (volume, value.max(dec!(1)))
        }
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
    fn generates_standard_scots_without_ic() {
        let mut gen = ScotsGenerator::new(42);
        let scots = gen.generate_for_entity("C001", &[]);
        // All non-IC SCOTs should be present (8 without IC)
        assert_eq!(scots.len(), 8, "Expected 8 non-IC SCOTs, got {}", scots.len());
    }

    #[test]
    fn ic_scot_added_when_enabled() {
        let config = ScotsGeneratorConfig {
            intercompany_enabled: true,
            ..ScotsGeneratorConfig::default()
        };
        let mut gen = ScotsGenerator::with_config(42, config);
        let scots = gen.generate_for_entity("C001", &[]);
        assert_eq!(scots.len(), 9, "Expected 9 SCOTs including IC");

        let ic_scot = scots.iter().find(|s| s.business_process == "IC");
        assert!(ic_scot.is_some(), "IC SCOT should be present when IC is enabled");
    }

    #[test]
    fn estimation_scots_have_complexity() {
        let mut gen = ScotsGenerator::new(42);
        let scots = gen.generate_for_entity("C001", &[]);

        let estimation_scots: Vec<_> = scots
            .iter()
            .filter(|s| s.transaction_type == ScotTransactionType::Estimation)
            .collect();

        assert!(!estimation_scots.is_empty(), "Should have estimation SCOTs");
        for s in &estimation_scots {
            assert!(
                s.estimation_complexity.is_some(),
                "Estimation SCOT '{}' must have estimation_complexity",
                s.scot_name
            );
        }
    }

    #[test]
    fn non_estimation_scots_have_no_complexity() {
        let mut gen = ScotsGenerator::new(42);
        let scots = gen.generate_for_entity("C001", &[]);

        for s in &scots {
            if s.transaction_type != ScotTransactionType::Estimation {
                assert!(
                    s.estimation_complexity.is_none(),
                    "Non-estimation SCOT '{}' should not have estimation_complexity",
                    s.scot_name
                );
            }
        }
    }

    #[test]
    fn all_scots_have_four_critical_path_stages() {
        let mut gen = ScotsGenerator::new(42);
        let scots = gen.generate_for_entity("C001", &[]);

        for s in &scots {
            assert_eq!(
                s.critical_path.len(),
                4,
                "SCOT '{}' should have exactly 4 critical path stages",
                s.scot_name
            );
        }
    }

    #[test]
    fn scot_ids_are_unique() {
        let mut gen = ScotsGenerator::new(42);
        let scots = gen.generate_for_entity("C001", &[]);

        let ids: std::collections::HashSet<&str> = scots.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids.len(), scots.len(), "SCOT IDs should be unique");
    }

    #[test]
    fn volume_and_value_are_positive() {
        let mut gen = ScotsGenerator::new(42);
        let scots = gen.generate_for_entity("C001", &[]);

        for s in &scots {
            assert!(s.volume > 0, "SCOT '{}' volume must be > 0", s.scot_name);
            assert!(
                s.monetary_value > Decimal::ZERO,
                "SCOT '{}' monetary_value must be > 0",
                s.scot_name
            );
        }
    }

    #[test]
    fn tax_provision_is_high_significance_estimation() {
        let mut gen = ScotsGenerator::new(42);
        let scots = gen.generate_for_entity("C001", &[]);

        let tax = scots.iter().find(|s| s.scot_name == "Tax Provision").unwrap();
        assert_eq!(tax.significance_level, ScotSignificance::High);
        assert_eq!(tax.transaction_type, ScotTransactionType::Estimation);
        assert_eq!(
            tax.estimation_complexity,
            Some(EstimationComplexity::Complex)
        );
    }

    #[test]
    fn revenue_scot_is_o2c_routine_high() {
        let mut gen = ScotsGenerator::new(42);
        let scots = gen.generate_for_entity("C001", &[]);

        let rev = scots
            .iter()
            .find(|s| s.scot_name == "Revenue — Product Sales")
            .unwrap();
        assert_eq!(rev.business_process, "O2C");
        assert_eq!(rev.transaction_type, ScotTransactionType::Routine);
        assert_eq!(rev.significance_level, ScotSignificance::High);
    }
}
