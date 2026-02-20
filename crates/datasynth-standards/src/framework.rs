//! Accounting Framework Selection and Configuration.
//!
//! Provides the core types for selecting between US GAAP and IFRS accounting
//! frameworks, along with framework-specific settings that control how
//! accounting standards are applied during synthetic data generation.

use serde::{Deserialize, Serialize};

/// Primary accounting framework selection.
///
/// Determines which set of accounting standards governs the generation
/// of financial data, affecting everything from revenue recognition
/// timing to lease classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AccountingFramework {
    /// United States Generally Accepted Accounting Principles.
    ///
    /// Key characteristics:
    /// - Rules-based approach
    /// - LIFO inventory permitted
    /// - No revaluation of PPE above cost
    /// - No reversal of impairment losses (except for certain assets)
    /// - Bright-line tests for lease classification
    #[default]
    UsGaap,

    /// International Financial Reporting Standards.
    ///
    /// Key characteristics:
    /// - Principles-based approach
    /// - LIFO inventory prohibited
    /// - Revaluation model permitted for PPE
    /// - Reversal of impairment losses permitted (except goodwill)
    /// - Principles-based lease classification
    Ifrs,

    /// Dual Reporting under both US GAAP and IFRS.
    ///
    /// Generates reconciliation data showing differences between
    /// the two frameworks for the same underlying transactions.
    DualReporting,

    /// French GAAP (Plan Comptable Général – PCG).
    ///
    /// French statutory accounting framework:
    /// - PCG chart of accounts (classes 1–9)
    /// - LIFO prohibited (like IFRS)
    /// - Impairment reversal permitted under French rules
    /// - Principles-based lease classification (convergent with IFRS 16 for many entities)
    FrenchGaap,
}

impl AccountingFramework {
    /// Returns the standard name for revenue recognition.
    pub fn revenue_standard(&self) -> &'static str {
        match self {
            Self::UsGaap => "ASC 606",
            Self::Ifrs => "IFRS 15",
            Self::DualReporting => "ASC 606 / IFRS 15",
            Self::FrenchGaap => "PCG / ANC (IFRS 15 aligned)",
        }
    }

    /// Returns the standard name for lease accounting.
    pub fn lease_standard(&self) -> &'static str {
        match self {
            Self::UsGaap => "ASC 842",
            Self::Ifrs => "IFRS 16",
            Self::DualReporting => "ASC 842 / IFRS 16",
            Self::FrenchGaap => "PCG / ANC (IFRS 16 aligned)",
        }
    }

    /// Returns the standard name for fair value measurement.
    pub fn fair_value_standard(&self) -> &'static str {
        match self {
            Self::UsGaap => "ASC 820",
            Self::Ifrs => "IFRS 13",
            Self::DualReporting => "ASC 820 / IFRS 13",
            Self::FrenchGaap => "PCG / ANC (IFRS 13 aligned)",
        }
    }

    /// Returns the standard name for impairment.
    pub fn impairment_standard(&self) -> &'static str {
        match self {
            Self::UsGaap => "ASC 360",
            Self::Ifrs => "IAS 36",
            Self::DualReporting => "ASC 360 / IAS 36",
            Self::FrenchGaap => "PCG / ANC (IAS 36 aligned)",
        }
    }

    /// Returns whether LIFO inventory costing is permitted.
    pub fn allows_lifo(&self) -> bool {
        matches!(self, Self::UsGaap)
    }

    /// Returns whether development cost capitalization is required.
    pub fn requires_development_capitalization(&self) -> bool {
        matches!(self, Self::Ifrs | Self::DualReporting | Self::FrenchGaap)
    }

    /// Returns whether PPE revaluation above cost is permitted.
    pub fn allows_ppe_revaluation(&self) -> bool {
        matches!(self, Self::Ifrs | Self::DualReporting)
    }

    /// Returns whether impairment loss reversal is permitted.
    pub fn allows_impairment_reversal(&self) -> bool {
        matches!(self, Self::Ifrs | Self::DualReporting | Self::FrenchGaap)
    }

    /// Returns whether this framework uses bright-line lease tests.
    pub fn uses_brightline_lease_tests(&self) -> bool {
        matches!(self, Self::UsGaap)
    }
}

impl std::fmt::Display for AccountingFramework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UsGaap => write!(f, "US GAAP"),
            Self::Ifrs => write!(f, "IFRS"),
            Self::DualReporting => write!(f, "Dual Reporting (US GAAP & IFRS)"),
            Self::FrenchGaap => write!(f, "French GAAP (PCG)"),
        }
    }
}

/// Framework-specific settings that control accounting treatment options.
///
/// These settings allow fine-grained control over framework-specific
/// accounting policies within the selected framework.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkSettings {
    /// The primary accounting framework.
    pub framework: AccountingFramework,

    /// Whether to use LIFO inventory costing (US GAAP only).
    ///
    /// Default: false (use FIFO/weighted average)
    #[serde(default)]
    pub use_lifo_inventory: bool,

    /// Whether to capitalize development costs (IFRS requirement, US GAAP option).
    ///
    /// Under IFRS, development costs must be capitalized when criteria are met.
    /// Under US GAAP, most development costs are expensed.
    #[serde(default)]
    pub capitalize_development_costs: bool,

    /// Whether to use revaluation model for PPE (IFRS option).
    ///
    /// Under IFRS, entities can choose between cost model and revaluation model.
    /// Under US GAAP, revaluation above cost is not permitted.
    #[serde(default)]
    pub use_ppe_revaluation: bool,

    /// Whether to reverse impairment losses when conditions improve (IFRS option).
    ///
    /// Under IFRS, impairment losses (except for goodwill) can be reversed.
    /// Under US GAAP, impairment losses generally cannot be reversed.
    #[serde(default)]
    pub allow_impairment_reversal: bool,

    /// Threshold percentage for lease term test (US GAAP: 75%).
    ///
    /// A lease is classified as finance/capital if the lease term is >= this
    /// percentage of the asset's economic life.
    #[serde(default = "default_lease_term_threshold")]
    pub lease_term_threshold: f64,

    /// Threshold percentage for present value test (US GAAP: 90%).
    ///
    /// A lease is classified as finance/capital if the present value of lease
    /// payments is >= this percentage of the asset's fair value.
    #[serde(default = "default_lease_pv_threshold")]
    pub lease_pv_threshold: f64,

    /// Default incremental borrowing rate for lease calculations.
    #[serde(default = "default_incremental_borrowing_rate")]
    pub default_incremental_borrowing_rate: f64,

    /// Revenue recognition constraint for variable consideration.
    ///
    /// Under both frameworks, variable consideration is constrained to the
    /// amount that is highly probable (IFRS) or probable (US GAAP) not to
    /// result in a significant revenue reversal.
    #[serde(default = "default_variable_consideration_constraint")]
    pub variable_consideration_constraint: f64,
}

fn default_lease_term_threshold() -> f64 {
    0.75
}

fn default_lease_pv_threshold() -> f64 {
    0.90
}

fn default_incremental_borrowing_rate() -> f64 {
    0.05
}

fn default_variable_consideration_constraint() -> f64 {
    0.80
}

impl Default for FrameworkSettings {
    fn default() -> Self {
        Self {
            framework: AccountingFramework::default(),
            use_lifo_inventory: false,
            capitalize_development_costs: false,
            use_ppe_revaluation: false,
            allow_impairment_reversal: false,
            lease_term_threshold: default_lease_term_threshold(),
            lease_pv_threshold: default_lease_pv_threshold(),
            default_incremental_borrowing_rate: default_incremental_borrowing_rate(),
            variable_consideration_constraint: default_variable_consideration_constraint(),
        }
    }
}

impl FrameworkSettings {
    /// Create settings for US GAAP with typical US company policies.
    pub fn us_gaap() -> Self {
        Self {
            framework: AccountingFramework::UsGaap,
            use_lifo_inventory: false, // Most companies use FIFO
            capitalize_development_costs: false,
            use_ppe_revaluation: false,
            allow_impairment_reversal: false,
            ..Default::default()
        }
    }

    /// Create settings for IFRS with typical international policies.
    pub fn ifrs() -> Self {
        Self {
            framework: AccountingFramework::Ifrs,
            use_lifo_inventory: false,          // LIFO prohibited
            capitalize_development_costs: true, // Required when criteria met
            use_ppe_revaluation: false,         // Optional, most use cost model
            allow_impairment_reversal: true,    // Permitted under IFRS
            ..Default::default()
        }
    }

    /// Create settings for dual reporting.
    pub fn dual_reporting() -> Self {
        Self {
            framework: AccountingFramework::DualReporting,
            use_lifo_inventory: false,
            capitalize_development_costs: true,
            use_ppe_revaluation: false,
            allow_impairment_reversal: true,
            ..Default::default()
        }
    }

    /// Create settings for French GAAP (Plan Comptable Général).
    pub fn french_gaap() -> Self {
        Self {
            framework: AccountingFramework::FrenchGaap,
            use_lifo_inventory: false, // LIFO prohibited under French GAAP
            capitalize_development_costs: true, // Permitted when criteria met
            use_ppe_revaluation: false, // Cost model typical
            allow_impairment_reversal: true, // Permitted under French rules
            ..Default::default()
        }
    }

    /// Validate settings are consistent with the selected framework.
    pub fn validate(&self) -> Result<(), FrameworkValidationError> {
        // LIFO is only permitted under US GAAP (prohibited under IFRS and French GAAP)
        if self.use_lifo_inventory
            && matches!(
                self.framework,
                AccountingFramework::Ifrs | AccountingFramework::FrenchGaap
            )
        {
            return Err(FrameworkValidationError::LifoNotPermittedUnderIfrs);
        }

        // PPE revaluation is not permitted under US GAAP
        if self.use_ppe_revaluation && self.framework == AccountingFramework::UsGaap {
            return Err(FrameworkValidationError::RevaluationNotPermittedUnderUsGaap);
        }

        // Impairment reversal is not permitted under US GAAP
        if self.allow_impairment_reversal && self.framework == AccountingFramework::UsGaap {
            return Err(FrameworkValidationError::ImpairmentReversalNotPermittedUnderUsGaap);
        }

        // Validate thresholds
        if !(0.0..=1.0).contains(&self.lease_term_threshold) {
            return Err(FrameworkValidationError::InvalidThreshold(
                "lease_term_threshold".to_string(),
            ));
        }

        if !(0.0..=1.0).contains(&self.lease_pv_threshold) {
            return Err(FrameworkValidationError::InvalidThreshold(
                "lease_pv_threshold".to_string(),
            ));
        }

        Ok(())
    }
}

/// Errors that can occur during framework settings validation.
#[derive(Debug, Clone, thiserror::Error)]
pub enum FrameworkValidationError {
    #[error("LIFO inventory costing is not permitted under IFRS or French GAAP")]
    LifoNotPermittedUnderIfrs,

    #[error("PPE revaluation above cost is not permitted under US GAAP")]
    RevaluationNotPermittedUnderUsGaap,

    #[error("Reversal of impairment losses is not permitted under US GAAP")]
    ImpairmentReversalNotPermittedUnderUsGaap,

    #[error("Invalid threshold value for {0}: must be between 0.0 and 1.0")]
    InvalidThreshold(String),
}

/// Key differences between US GAAP and IFRS for a specific area.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkDifference {
    /// Area of accounting (e.g., "Revenue Recognition", "Lease Classification").
    pub area: String,

    /// US GAAP treatment description.
    pub us_gaap_treatment: String,

    /// IFRS treatment description.
    pub ifrs_treatment: String,

    /// Whether this difference typically results in material differences.
    pub typically_material: bool,

    /// Relevant US GAAP codification reference.
    pub us_gaap_reference: String,

    /// Relevant IFRS standard reference.
    pub ifrs_reference: String,
}

impl FrameworkDifference {
    /// Returns common framework differences for educational/documentation purposes.
    pub fn common_differences() -> Vec<Self> {
        vec![
            Self {
                area: "Inventory Costing".to_string(),
                us_gaap_treatment: "LIFO, FIFO, and weighted average permitted".to_string(),
                ifrs_treatment: "LIFO prohibited; FIFO and weighted average permitted".to_string(),
                typically_material: true,
                us_gaap_reference: "ASC 330".to_string(),
                ifrs_reference: "IAS 2".to_string(),
            },
            Self {
                area: "Development Costs".to_string(),
                us_gaap_treatment: "Generally expensed as incurred".to_string(),
                ifrs_treatment: "Capitalized when specified criteria are met".to_string(),
                typically_material: true,
                us_gaap_reference: "ASC 730".to_string(),
                ifrs_reference: "IAS 38".to_string(),
            },
            Self {
                area: "Property, Plant & Equipment".to_string(),
                us_gaap_treatment: "Cost model only; no revaluation above cost".to_string(),
                ifrs_treatment: "Cost model or revaluation model permitted".to_string(),
                typically_material: true,
                us_gaap_reference: "ASC 360".to_string(),
                ifrs_reference: "IAS 16".to_string(),
            },
            Self {
                area: "Impairment Reversal".to_string(),
                us_gaap_treatment: "Not permitted for most assets".to_string(),
                ifrs_treatment: "Permitted except for goodwill".to_string(),
                typically_material: true,
                us_gaap_reference: "ASC 360".to_string(),
                ifrs_reference: "IAS 36".to_string(),
            },
            Self {
                area: "Lease Classification".to_string(),
                us_gaap_treatment: "Bright-line tests (75% term, 90% PV)".to_string(),
                ifrs_treatment: "Principles-based; transfer of risks and rewards".to_string(),
                typically_material: false,
                us_gaap_reference: "ASC 842".to_string(),
                ifrs_reference: "IFRS 16".to_string(),
            },
            Self {
                area: "Contingent Liabilities".to_string(),
                us_gaap_treatment: "Recognized when probable (>75%) and estimable".to_string(),
                ifrs_treatment: "Recognized when probable (>50%) and estimable".to_string(),
                typically_material: true,
                us_gaap_reference: "ASC 450".to_string(),
                ifrs_reference: "IAS 37".to_string(),
            },
        ]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_defaults() {
        let framework = AccountingFramework::default();
        assert_eq!(framework, AccountingFramework::UsGaap);
    }

    #[test]
    fn test_framework_standards() {
        assert_eq!(AccountingFramework::UsGaap.revenue_standard(), "ASC 606");
        assert_eq!(AccountingFramework::Ifrs.revenue_standard(), "IFRS 15");
        assert_eq!(AccountingFramework::UsGaap.lease_standard(), "ASC 842");
        assert_eq!(AccountingFramework::Ifrs.lease_standard(), "IFRS 16");
        assert!(AccountingFramework::FrenchGaap
            .revenue_standard()
            .contains("PCG"));
    }

    #[test]
    fn test_framework_features() {
        assert!(AccountingFramework::UsGaap.allows_lifo());
        assert!(!AccountingFramework::Ifrs.allows_lifo());
        assert!(!AccountingFramework::FrenchGaap.allows_lifo());

        assert!(!AccountingFramework::UsGaap.allows_ppe_revaluation());
        assert!(AccountingFramework::Ifrs.allows_ppe_revaluation());

        assert!(!AccountingFramework::UsGaap.allows_impairment_reversal());
        assert!(AccountingFramework::Ifrs.allows_impairment_reversal());
        assert!(AccountingFramework::FrenchGaap.allows_impairment_reversal());
    }

    #[test]
    fn test_french_gaap_settings() {
        let settings = FrameworkSettings::french_gaap();
        assert!(settings.validate().is_ok());
        assert_eq!(settings.framework, AccountingFramework::FrenchGaap);
    }

    #[test]
    fn test_settings_validation_us_gaap() {
        let settings = FrameworkSettings::us_gaap();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_settings_validation_ifrs() {
        let settings = FrameworkSettings::ifrs();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_settings_validation_lifo_under_ifrs() {
        let mut settings = FrameworkSettings::ifrs();
        settings.use_lifo_inventory = true;
        assert!(matches!(
            settings.validate(),
            Err(FrameworkValidationError::LifoNotPermittedUnderIfrs)
        ));
    }

    #[test]
    fn test_settings_validation_revaluation_under_us_gaap() {
        let mut settings = FrameworkSettings::us_gaap();
        settings.use_ppe_revaluation = true;
        assert!(matches!(
            settings.validate(),
            Err(FrameworkValidationError::RevaluationNotPermittedUnderUsGaap)
        ));
    }

    #[test]
    fn test_common_differences() {
        let differences = FrameworkDifference::common_differences();
        assert!(!differences.is_empty());
        assert!(differences.iter().any(|d| d.area == "Inventory Costing"));
    }

    #[test]
    fn test_serde_roundtrip() {
        let framework = AccountingFramework::Ifrs;
        let json = serde_json::to_string(&framework).unwrap();
        let deserialized: AccountingFramework = serde_json::from_str(&json).unwrap();
        assert_eq!(framework, deserialized);

        let settings = FrameworkSettings::ifrs();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: FrameworkSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings.framework, deserialized.framework);
    }
}
