//! Banking/KYC/AML evaluation module.
//!
//! Validates banking data including KYC profile completeness
//! and AML typology coherence and detectability.

pub mod aml_detectability;
pub mod kyc_completeness;

pub use aml_detectability::{
    AmlDetectabilityAnalysis, AmlDetectabilityAnalyzer, AmlTransactionData, TypologyData,
};
pub use kyc_completeness::{KycCompletenessAnalysis, KycCompletenessAnalyzer, KycProfileData};

use serde::{Deserialize, Serialize};

/// Combined banking evaluation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankingEvaluation {
    /// KYC completeness analysis.
    pub kyc: Option<KycCompletenessAnalysis>,
    /// AML detectability analysis.
    pub aml: Option<AmlDetectabilityAnalysis>,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

impl BankingEvaluation {
    /// Create a new empty evaluation.
    pub fn new() -> Self {
        Self {
            kyc: None,
            aml: None,
            passes: true,
            issues: Vec::new(),
        }
    }

    /// Check thresholds and update pass status.
    pub fn check_thresholds(&mut self) {
        self.issues.clear();
        if let Some(ref kyc) = self.kyc {
            if !kyc.passes {
                self.issues.extend(kyc.issues.clone());
            }
        }
        if let Some(ref aml) = self.aml {
            if !aml.passes {
                self.issues.extend(aml.issues.clone());
            }
        }
        self.passes = self.issues.is_empty();
    }
}

impl Default for BankingEvaluation {
    fn default() -> Self {
        Self::new()
    }
}
