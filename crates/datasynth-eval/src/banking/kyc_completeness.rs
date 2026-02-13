//! KYC profile completeness evaluator.
//!
//! Validates that KYC profiles have required fields populated
//! and beneficial owner coverage meets standards.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// KYC profile data for validation.
#[derive(Debug, Clone)]
pub struct KycProfileData {
    /// Profile identifier.
    pub profile_id: String,
    /// Whether customer name is populated.
    pub has_name: bool,
    /// Whether date of birth / incorporation date is populated.
    pub has_dob: bool,
    /// Whether address is populated.
    pub has_address: bool,
    /// Whether ID document is populated.
    pub has_id_document: bool,
    /// Whether risk rating is assigned.
    pub has_risk_rating: bool,
    /// Whether beneficial owner information is populated (for entities).
    pub has_beneficial_owner: bool,
    /// Whether the profile is for an entity (vs individual).
    pub is_entity: bool,
    /// Whether the profile has been reviewed/verified.
    pub is_verified: bool,
}

/// Thresholds for KYC completeness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycCompletenessThresholds {
    /// Minimum rate for core fields (name, DOB, address, ID).
    pub min_core_field_rate: f64,
    /// Minimum beneficial owner coverage for entities.
    pub min_beneficial_owner_rate: f64,
    /// Minimum risk rating coverage.
    pub min_risk_rating_rate: f64,
}

impl Default for KycCompletenessThresholds {
    fn default() -> Self {
        Self {
            min_core_field_rate: 0.95,
            min_beneficial_owner_rate: 0.90,
            min_risk_rating_rate: 0.95,
        }
    }
}

/// Results of KYC completeness analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycCompletenessAnalysis {
    /// Core field completeness rate (name + DOB + address + ID all present).
    pub core_field_rate: f64,
    /// Individual field rates.
    pub name_rate: f64,
    /// DOB/incorporation date rate.
    pub dob_rate: f64,
    /// Address rate.
    pub address_rate: f64,
    /// ID document rate.
    pub id_document_rate: f64,
    /// Risk rating coverage.
    pub risk_rating_rate: f64,
    /// Beneficial owner rate for entities.
    pub beneficial_owner_rate: f64,
    /// Verification rate.
    pub verification_rate: f64,
    /// Total profiles evaluated.
    pub total_profiles: usize,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Analyzer for KYC completeness.
pub struct KycCompletenessAnalyzer {
    thresholds: KycCompletenessThresholds,
}

impl KycCompletenessAnalyzer {
    /// Create a new analyzer with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: KycCompletenessThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: KycCompletenessThresholds) -> Self {
        Self { thresholds }
    }

    /// Analyze KYC profiles.
    pub fn analyze(&self, profiles: &[KycProfileData]) -> EvalResult<KycCompletenessAnalysis> {
        let mut issues = Vec::new();
        let total = profiles.len();

        if total == 0 {
            return Ok(KycCompletenessAnalysis {
                core_field_rate: 1.0,
                name_rate: 1.0,
                dob_rate: 1.0,
                address_rate: 1.0,
                id_document_rate: 1.0,
                risk_rating_rate: 1.0,
                beneficial_owner_rate: 1.0,
                verification_rate: 1.0,
                total_profiles: 0,
                passes: true,
                issues: Vec::new(),
            });
        }

        let name_count = profiles.iter().filter(|p| p.has_name).count();
        let dob_count = profiles.iter().filter(|p| p.has_dob).count();
        let address_count = profiles.iter().filter(|p| p.has_address).count();
        let id_count = profiles.iter().filter(|p| p.has_id_document).count();
        let risk_count = profiles.iter().filter(|p| p.has_risk_rating).count();
        let verified_count = profiles.iter().filter(|p| p.is_verified).count();

        let core_complete = profiles
            .iter()
            .filter(|p| p.has_name && p.has_dob && p.has_address && p.has_id_document)
            .count();

        let entities: Vec<&KycProfileData> = profiles.iter().filter(|p| p.is_entity).collect();
        let bo_count = entities.iter().filter(|p| p.has_beneficial_owner).count();

        let core_field_rate = core_complete as f64 / total as f64;
        let name_rate = name_count as f64 / total as f64;
        let dob_rate = dob_count as f64 / total as f64;
        let address_rate = address_count as f64 / total as f64;
        let id_document_rate = id_count as f64 / total as f64;
        let risk_rating_rate = risk_count as f64 / total as f64;
        let verification_rate = verified_count as f64 / total as f64;
        let beneficial_owner_rate = if entities.is_empty() {
            1.0
        } else {
            bo_count as f64 / entities.len() as f64
        };

        if core_field_rate < self.thresholds.min_core_field_rate {
            issues.push(format!(
                "Core field rate {:.3} < {:.3}",
                core_field_rate, self.thresholds.min_core_field_rate
            ));
        }
        if beneficial_owner_rate < self.thresholds.min_beneficial_owner_rate {
            issues.push(format!(
                "Beneficial owner rate {:.3} < {:.3}",
                beneficial_owner_rate, self.thresholds.min_beneficial_owner_rate
            ));
        }
        if risk_rating_rate < self.thresholds.min_risk_rating_rate {
            issues.push(format!(
                "Risk rating rate {:.3} < {:.3}",
                risk_rating_rate, self.thresholds.min_risk_rating_rate
            ));
        }

        let passes = issues.is_empty();

        Ok(KycCompletenessAnalysis {
            core_field_rate,
            name_rate,
            dob_rate,
            address_rate,
            id_document_rate,
            risk_rating_rate,
            beneficial_owner_rate,
            verification_rate,
            total_profiles: total,
            passes,
            issues,
        })
    }
}

impl Default for KycCompletenessAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn complete_profile() -> KycProfileData {
        KycProfileData {
            profile_id: "KYC001".to_string(),
            has_name: true,
            has_dob: true,
            has_address: true,
            has_id_document: true,
            has_risk_rating: true,
            has_beneficial_owner: true,
            is_entity: true,
            is_verified: true,
        }
    }

    #[test]
    fn test_complete_profiles() {
        let analyzer = KycCompletenessAnalyzer::new();
        let result = analyzer.analyze(&[complete_profile()]).unwrap();
        assert!(result.passes);
        assert_eq!(result.core_field_rate, 1.0);
    }

    #[test]
    fn test_incomplete_profiles() {
        let analyzer = KycCompletenessAnalyzer::new();
        let mut profile = complete_profile();
        profile.has_name = false;
        profile.has_risk_rating = false;

        let result = analyzer.analyze(&[profile]).unwrap();
        assert!(!result.passes);
        assert_eq!(result.core_field_rate, 0.0);
    }

    #[test]
    fn test_empty() {
        let analyzer = KycCompletenessAnalyzer::new();
        let result = analyzer.analyze(&[]).unwrap();
        assert!(result.passes);
    }
}
