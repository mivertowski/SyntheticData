//! Built-in quality gate profiles.
//!
//! Provides strict, default, and lenient profiles with pre-configured thresholds.

use super::engine::{GateProfile, QualityGate, QualityMetric};

/// Strict profile — tight thresholds for production-quality data.
///
/// - Benford MAD < 0.01
/// - Balance coherence >= 0.999
/// - Document chain integrity >= 0.95
/// - Completion rate >= 0.99
/// - Duplicate rate <= 0.001
/// - Referential integrity >= 0.999
/// - IC match rate >= 0.99
/// - Privacy MIA AUC <= 0.55
pub fn strict_profile() -> GateProfile {
    GateProfile::new(
        "strict",
        vec![
            QualityGate::lte("benford_compliance", QualityMetric::BenfordMad, 0.01),
            QualityGate::gte("balance_coherence", QualityMetric::BalanceCoherence, 0.999),
            QualityGate::gte(
                "document_chain_integrity",
                QualityMetric::DocumentChainIntegrity,
                0.95,
            ),
            QualityGate::gte("temporal_consistency", QualityMetric::TemporalConsistency, 0.90),
            QualityGate::gte("completion_rate", QualityMetric::CompletionRate, 0.99),
            QualityGate::lte("duplicate_rate", QualityMetric::DuplicateRate, 0.001),
            QualityGate::gte(
                "referential_integrity",
                QualityMetric::ReferentialIntegrity,
                0.999,
            ),
            QualityGate::gte("ic_match_rate", QualityMetric::IcMatchRate, 0.99),
            QualityGate::lte("privacy_mia_auc", QualityMetric::PrivacyMiaAuc, 0.55),
        ],
    )
}

/// Default profile — balanced thresholds suitable for most use cases.
///
/// - Benford MAD < 0.015
/// - Balance coherence >= 0.99
/// - Document chain integrity >= 0.90
/// - Completion rate >= 0.95
/// - Duplicate rate <= 0.01
/// - Referential integrity >= 0.99
/// - IC match rate >= 0.95
/// - Privacy MIA AUC <= 0.60
pub fn default_profile() -> GateProfile {
    GateProfile::new(
        "default",
        vec![
            QualityGate::lte("benford_compliance", QualityMetric::BenfordMad, 0.015),
            QualityGate::gte("balance_coherence", QualityMetric::BalanceCoherence, 0.99),
            QualityGate::gte(
                "document_chain_integrity",
                QualityMetric::DocumentChainIntegrity,
                0.90,
            ),
            QualityGate::gte("temporal_consistency", QualityMetric::TemporalConsistency, 0.80),
            QualityGate::gte("completion_rate", QualityMetric::CompletionRate, 0.95),
            QualityGate::lte("duplicate_rate", QualityMetric::DuplicateRate, 0.01),
            QualityGate::gte(
                "referential_integrity",
                QualityMetric::ReferentialIntegrity,
                0.99,
            ),
            QualityGate::gte("ic_match_rate", QualityMetric::IcMatchRate, 0.95),
            QualityGate::lte("privacy_mia_auc", QualityMetric::PrivacyMiaAuc, 0.60),
        ],
    )
}

/// Lenient profile — relaxed thresholds for exploratory or development use.
///
/// - Benford MAD < 0.03
/// - Balance coherence >= 0.95
/// - Document chain integrity >= 0.80
/// - Completion rate >= 0.90
/// - Duplicate rate <= 0.05
/// - Referential integrity >= 0.95
/// - IC match rate >= 0.85
/// - Privacy MIA AUC <= 0.70
pub fn lenient_profile() -> GateProfile {
    GateProfile::new(
        "lenient",
        vec![
            QualityGate::lte("benford_compliance", QualityMetric::BenfordMad, 0.03),
            QualityGate::gte("balance_coherence", QualityMetric::BalanceCoherence, 0.95),
            QualityGate::gte(
                "document_chain_integrity",
                QualityMetric::DocumentChainIntegrity,
                0.80,
            ),
            QualityGate::gte("temporal_consistency", QualityMetric::TemporalConsistency, 0.60),
            QualityGate::gte("completion_rate", QualityMetric::CompletionRate, 0.90),
            QualityGate::lte("duplicate_rate", QualityMetric::DuplicateRate, 0.05),
            QualityGate::gte(
                "referential_integrity",
                QualityMetric::ReferentialIntegrity,
                0.95,
            ),
            QualityGate::gte("ic_match_rate", QualityMetric::IcMatchRate, 0.85),
            QualityGate::lte("privacy_mia_auc", QualityMetric::PrivacyMiaAuc, 0.70),
        ],
    )
}

/// Get a profile by name.
///
/// Returns `None` for unrecognized names.
pub fn get_profile(name: &str) -> Option<GateProfile> {
    match name {
        "strict" => Some(strict_profile()),
        "default" => Some(default_profile()),
        "lenient" => Some(lenient_profile()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strict_profile_has_gates() {
        let profile = strict_profile();
        assert_eq!(profile.name, "strict");
        assert!(!profile.gates.is_empty());
        assert!(profile.gates.len() >= 5);
    }

    #[test]
    fn test_default_profile_has_gates() {
        let profile = default_profile();
        assert_eq!(profile.name, "default");
        assert!(!profile.gates.is_empty());
    }

    #[test]
    fn test_lenient_profile_has_gates() {
        let profile = lenient_profile();
        assert_eq!(profile.name, "lenient");
        assert!(!profile.gates.is_empty());
    }

    #[test]
    fn test_strict_thresholds_tighter_than_lenient() {
        let strict = strict_profile();
        let lenient = lenient_profile();

        // Find Benford MAD gate in both
        let strict_benford = strict.gates.iter().find(|g| g.metric == QualityMetric::BenfordMad);
        let lenient_benford = lenient.gates.iter().find(|g| g.metric == QualityMetric::BenfordMad);

        if let (Some(s), Some(l)) = (strict_benford, lenient_benford) {
            // Strict should have a lower (tighter) MAD threshold
            assert!(s.threshold < l.threshold, "strict MAD ({}) should be < lenient MAD ({})", s.threshold, l.threshold);
        }
    }

    #[test]
    fn test_get_profile_by_name() {
        assert!(get_profile("strict").is_some());
        assert!(get_profile("default").is_some());
        assert!(get_profile("lenient").is_some());
        assert!(get_profile("nonexistent").is_none());
    }

    #[test]
    fn test_profile_serialization_roundtrip() {
        for name in &["strict", "default", "lenient"] {
            let profile = get_profile(name).expect("profile should exist");
            let json = serde_json::to_string(&profile).expect("should serialize");
            let deser: GateProfile = serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(deser.name, *name);
            assert_eq!(deser.gates.len(), profile.gates.len());
        }
    }
}
