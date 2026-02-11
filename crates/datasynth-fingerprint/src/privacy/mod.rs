//! Privacy mechanisms for fingerprint extraction.
//!
//! This module provides:
//! - **Differential Privacy**: Laplace noise with epsilon budgeting
//! - **K-Anonymity**: Suppression of rare categorical values
//! - **Audit Trail**: Complete logging of privacy decisions
//!
//! # Overview
//!
//! The [`PrivacyEngine`] applies privacy protections during fingerprint extraction,
//! ensuring that the extracted statistics cannot be used to identify individuals.
//!
//! # Privacy Levels
//!
//! Four pre-configured privacy levels are available:
//!
//! | Level | Epsilon | K | Use Case |
//! |-------|---------|---|----------|
//! | Minimal | 5.0 | 3 | Low privacy requirements |
//! | Standard | 1.0 | 5 | Balanced (default) |
//! | High | 0.5 | 10 | Sensitive data |
//! | Maximum | 0.1 | 20 | Highly sensitive data |
//!
//! # Usage
//!
//! ```ignore
//! use datasynth_fingerprint::privacy::{PrivacyEngine, PrivacyConfig};
//! use datasynth_fingerprint::models::PrivacyLevel;
//!
//! // Create engine with standard privacy
//! let mut engine = PrivacyEngine::from_level(PrivacyLevel::Standard);
//!
//! // Add noise to a numeric statistic
//! let noised_mean = engine.add_noise(100.5, 1.0, "table.amount.mean")?;
//!
//! // Filter categories by k-anonymity
//! let frequencies = vec![
//!     ("USA".to_string(), 1000),
//!     ("UK".to_string(), 500),
//!     ("Rare".to_string(), 2),  // Will be suppressed (< k=5)
//! ];
//! let filtered = engine.filter_categories(frequencies, 1502, "table.country");
//!
//! // Get the audit trail
//! let audit = engine.audit();
//! println!("Epsilon spent: {}", audit.total_epsilon_spent);
//! println!("Actions: {}", audit.actions.len());
//! ```
//!
//! # Differential Privacy
//!
//! The [`LaplaceMechanism`] adds calibrated noise to numeric statistics:
//!
//! ```ignore
//! let mechanism = LaplaceMechanism::new(epsilon);
//! let noised = mechanism.add_noise(value, sensitivity, epsilon_per_query);
//! ```
//!
//! The noise is calibrated based on:
//! - **Sensitivity**: How much a single record can change the statistic
//! - **Epsilon**: Privacy budget (lower = more privacy, more noise)
//!
//! # K-Anonymity
//!
//! The [`KAnonymity`] mechanism suppresses rare categorical values:
//!
//! ```ignore
//! let kanon = KAnonymity::new(k, min_occurrence);
//! let (kept, suppressed_count) = kanon.filter_frequencies(frequencies, total);
//! ```
//!
//! Values appearing fewer than k times are replaced with an "Other" category.
//!
//! # Audit Trail
//!
//! Every privacy decision is recorded in the [`PrivacyAudit`]:
//!
//! - Noise additions with epsilon spent
//! - Value suppressions
//! - Generalizations
//! - Winsorization of outliers
//!
//! The audit is included in the fingerprint file for transparency.
//!
//! [`PrivacyEngine`]: PrivacyEngine
//! [`LaplaceMechanism`]: differential::LaplaceMechanism
//! [`KAnonymity`]: kanonymity::KAnonymity
//! [`PrivacyAudit`]: crate::models::PrivacyAudit

mod audit;
pub mod budget;
pub mod composition;
mod differential;
mod kanonymity;
pub mod pareto;

pub use audit::*;
pub use composition::{
    create_accountant, CompositionMethod, MechanismRecord, NaiveAccountant, PrivacyAccountant,
    RenyiDPAccountant, ZeroCDPAccountant,
};
pub use differential::*;
pub use kanonymity::*;

use crate::error::{FingerprintError, FingerprintResult};
use crate::models::{
    PrivacyAction, PrivacyActionType, PrivacyAudit, PrivacyLevel, PrivacyMetadata,
};

/// Configuration for privacy mechanisms.
#[derive(Debug, Clone)]
pub struct PrivacyConfig {
    /// Privacy level.
    pub level: PrivacyLevel,
    /// Differential privacy epsilon budget.
    pub epsilon: f64,
    /// K-anonymity threshold.
    pub k_anonymity: u32,
    /// Outlier percentile for winsorization.
    pub outlier_percentile: f64,
    /// Minimum occurrence for categorical values.
    pub min_occurrence: u32,
    /// Fields to always suppress.
    pub suppressed_fields: Vec<String>,
    /// Composition method for privacy budget accounting.
    /// Defaults to `Naive` for backward compatibility.
    pub composition_method: CompositionMethod,
}

impl PrivacyConfig {
    /// Create from privacy level.
    pub fn from_level(level: PrivacyLevel) -> Self {
        let metadata = PrivacyMetadata::from_level(level);
        Self {
            level,
            epsilon: metadata.epsilon,
            k_anonymity: metadata.k_anonymity,
            outlier_percentile: metadata.outlier_percentile,
            min_occurrence: metadata.min_occurrence,
            suppressed_fields: metadata.suppressed_fields,
            composition_method: CompositionMethod::Naive,
        }
    }

    /// Create custom configuration.
    pub fn custom(epsilon: f64, k_anonymity: u32) -> Self {
        Self {
            level: PrivacyLevel::Custom,
            epsilon,
            k_anonymity,
            outlier_percentile: 95.0,
            min_occurrence: k_anonymity,
            suppressed_fields: Vec::new(),
            composition_method: CompositionMethod::Naive,
        }
    }

    /// Create custom configuration with delta and composition method.
    ///
    /// Use this for advanced composition (RDP/zCDP) where delta is meaningful.
    ///
    /// Note: The `delta` parameter is not stored in `PrivacyConfig` directly;
    /// it is managed by the accountant created in `PrivacyEngine::new()`.
    /// The delta value is captured in `PrivacyMetadata` for the manifest via
    /// the accountant's `target_delta()` method.
    pub fn custom_with_delta(
        epsilon: f64,
        _delta: f64,
        k_anonymity: u32,
        composition_method: CompositionMethod,
    ) -> Self {
        Self {
            level: PrivacyLevel::Custom,
            epsilon,
            k_anonymity,
            outlier_percentile: 95.0,
            min_occurrence: k_anonymity,
            suppressed_fields: Vec::new(),
            composition_method,
        }
    }
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self::from_level(PrivacyLevel::Standard)
    }
}

/// Privacy engine that applies privacy mechanisms during extraction.
///
/// The engine coordinates differential privacy (Laplace noise), k-anonymity,
/// and privacy budget accounting via a pluggable [`PrivacyAccountant`].
///
/// When using advanced composition methods (RDP or zCDP), the accountant
/// provides tighter effective epsilon bounds than naive summation, allowing
/// more queries within the same total privacy budget.
pub struct PrivacyEngine {
    config: PrivacyConfig,
    audit: PrivacyAudit,
    laplace: LaplaceMechanism,
    kanon: KAnonymity,
    accountant: Box<dyn PrivacyAccountant>,
}

impl PrivacyEngine {
    /// Create a new privacy engine.
    ///
    /// The appropriate [`PrivacyAccountant`] is created based on
    /// `config.composition_method`. For `Naive` (the default), this is
    /// fully backward compatible with the original behavior.
    pub fn new(config: PrivacyConfig) -> Self {
        let accountant = create_accountant(config.composition_method, config.epsilon);
        Self {
            audit: PrivacyAudit::new(config.epsilon, config.k_anonymity),
            laplace: LaplaceMechanism::new(config.epsilon),
            kanon: KAnonymity::new(config.k_anonymity, config.min_occurrence),
            accountant,
            config,
        }
    }

    /// Create from privacy level.
    pub fn from_level(level: PrivacyLevel) -> Self {
        Self::new(PrivacyConfig::from_level(level))
    }

    /// Check if budget allows spending epsilon.
    ///
    /// For naive composition, this checks the audit trail's simple sum.
    /// For advanced composition (RDP/zCDP), this uses the accountant's
    /// tighter effective epsilon calculation.
    pub fn can_spend(&self, epsilon: f64) -> bool {
        match self.config.composition_method {
            CompositionMethod::Naive | CompositionMethod::Advanced => {
                // Backward compatible: use the audit trail's simple sum
                self.audit.remaining_budget() >= epsilon
            }
            CompositionMethod::RenyiDP | CompositionMethod::ZeroCDP => {
                // Use the accountant's tighter bound
                self.accountant.remaining_budget() >= epsilon
            }
        }
    }

    /// Add noise to a numeric value.
    ///
    /// Each call consumes `epsilon / 100.0` of the privacy budget.
    /// The mechanism is recorded with both the audit trail (for logging)
    /// and the accountant (for composition-aware budget tracking).
    pub fn add_noise(
        &mut self,
        value: f64,
        sensitivity: f64,
        target: &str,
    ) -> FingerprintResult<f64> {
        let epsilon_per_query = self.config.epsilon / 100.0; // Budget across many queries

        if !self.can_spend(epsilon_per_query) {
            return Err(FingerprintError::PrivacyBudgetExhausted {
                spent: self.audit.total_epsilon_spent,
                limit: self.config.epsilon,
            });
        }

        let noised = self
            .laplace
            .add_noise(value, sensitivity, epsilon_per_query);

        // Record with the accountant for composition-aware tracking
        let mechanism_record = MechanismRecord::new(
            epsilon_per_query,
            format!("Laplace noise on {} (sensitivity={})", target, sensitivity),
        );
        self.accountant.record_mechanism(mechanism_record);

        let action = PrivacyAction::new(
            PrivacyActionType::LaplaceNoise,
            target,
            format!(
                "Added Laplace noise with sensitivity={}, epsilon={}",
                sensitivity, epsilon_per_query
            ),
            "Differential privacy protection",
        )
        .with_epsilon(epsilon_per_query);

        self.audit.record_action(action);
        Ok(noised)
    }

    /// Add noise to a count.
    pub fn add_noise_to_count(&mut self, count: u64, target: &str) -> FingerprintResult<u64> {
        let noised = self.add_noise(count as f64, 1.0, target)?;
        Ok(noised.max(0.0).round() as u64)
    }

    /// Filter categorical frequencies by k-anonymity.
    pub fn filter_categories(
        &mut self,
        frequencies: Vec<(String, u64)>,
        total: u64,
        target: &str,
    ) -> Vec<(String, f64)> {
        let (kept, suppressed) = self.kanon.filter_frequencies(frequencies, total);

        if suppressed > 0 {
            let action = PrivacyAction::new(
                PrivacyActionType::Suppression,
                target,
                format!(
                    "Suppressed {} rare categories below k={}",
                    suppressed, self.config.k_anonymity
                ),
                "K-anonymity protection",
            );
            self.audit.record_action(action);
        }

        kept
    }

    /// Winsorize outliers in a sorted list.
    pub fn winsorize(&mut self, values: &mut [f64], target: &str) {
        let percentile = self.config.outlier_percentile;
        let (low_count, high_count) = winsorize_values(values, percentile);

        if low_count > 0 || high_count > 0 {
            let action = PrivacyAction::new(
                PrivacyActionType::Winsorization,
                target,
                format!(
                    "Winsorized {} low and {} high outliers at {}th percentile",
                    low_count, high_count, percentile
                ),
                "Outlier protection",
            );
            self.audit.record_action(action);
        }
    }

    /// Check if a field should be suppressed.
    pub fn should_suppress_field(&self, field: &str) -> bool {
        self.config.suppressed_fields.iter().any(|f| f == field)
    }

    /// Record a custom privacy action.
    pub fn record_action(&mut self, action: PrivacyAction) {
        self.audit.record_action(action);
    }

    /// Get the privacy audit.
    pub fn audit(&self) -> &PrivacyAudit {
        &self.audit
    }

    /// Consume and return the privacy audit, populated with composition metadata.
    ///
    /// When using advanced composition (RDP/zCDP), the audit's `composition_method`
    /// and `rdp_alpha_effective` fields are populated from the accountant.
    pub fn into_audit(mut self) -> PrivacyAudit {
        // Populate composition metadata on the audit
        self.audit.composition_method = Some(self.accountant.method().to_string());

        // For RDP, record the optimal alpha order
        if let Some(alpha) = self.accountant.optimal_alpha() {
            self.audit.rdp_alpha_effective = Some(alpha);
        }

        self.audit
    }

    /// Get remaining epsilon budget.
    pub fn remaining_budget(&self) -> f64 {
        self.audit.remaining_budget()
    }

    /// Get the effective epsilon as computed by the accountant.
    ///
    /// For naive composition, this equals the sum of per-query epsilons.
    /// For RDP/zCDP, this is the tighter composed epsilon after conversion
    /// to (epsilon, delta)-DP.
    pub fn effective_epsilon(&self) -> f64 {
        self.accountant.effective_epsilon()
    }

    /// Get the accountant's remaining budget (composition-aware).
    ///
    /// For naive composition, this is the same as `remaining_budget()`.
    /// For RDP/zCDP, this may be larger due to tighter composition bounds.
    pub fn accountant_remaining_budget(&self) -> f64 {
        self.accountant.remaining_budget()
    }

    /// Build [`PrivacyMetadata`] from the current engine state.
    ///
    /// This populates the `delta` and `composition_method` fields based on
    /// the accountant configuration.
    pub fn build_privacy_metadata(&self) -> PrivacyMetadata {
        let mut meta = PrivacyMetadata::from_level(self.config.level);
        meta.epsilon = self.config.epsilon;
        meta.k_anonymity = self.config.k_anonymity;
        meta.composition_method = Some(self.accountant.method().to_string());
        meta.delta = self.accountant.target_delta();
        meta
    }

    /// Get a reference to the composition method in use.
    pub fn composition_method(&self) -> CompositionMethod {
        self.config.composition_method
    }
}

/// Winsorize values at given percentile.
fn winsorize_values(values: &mut [f64], percentile: f64) -> (usize, usize) {
    if values.is_empty() {
        return (0, 0);
    }

    let n = values.len();
    let low_idx = ((100.0 - percentile) / 100.0 * n as f64).floor() as usize;
    let high_idx = (percentile / 100.0 * n as f64).ceil() as usize;

    // Sort to find percentile values
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    let low_threshold = sorted.get(low_idx).copied().unwrap_or(f64::MIN);
    let high_threshold = sorted.get(high_idx.min(n - 1)).copied().unwrap_or(f64::MAX);

    let mut low_count = 0;
    let mut high_count = 0;

    for v in values.iter_mut() {
        if *v < low_threshold {
            *v = low_threshold;
            low_count += 1;
        } else if *v > high_threshold {
            *v = high_threshold;
            high_count += 1;
        }
    }

    (low_count, high_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Backward compatibility: Standard level with Naive composition unchanged
    // -----------------------------------------------------------------------

    #[test]
    fn test_standard_level_backward_compat_unchanged() {
        // The default behavior (Naive composition) must be identical to pre-PR behavior.
        let mut engine = PrivacyEngine::from_level(PrivacyLevel::Standard);

        // Verify config defaults
        assert_eq!(engine.composition_method(), CompositionMethod::Naive);
        assert!((engine.remaining_budget() - 1.0).abs() < 1e-10);

        // Add noise 50 times (each costing epsilon/100 = 0.01)
        for i in 0..50 {
            engine.add_noise(100.0, 1.0, &format!("col_{}", i)).unwrap();
        }

        // With naive composition, total spent = 50 * 0.01 = 0.50
        let audit = engine.audit();
        assert!((audit.total_epsilon_spent - 0.50).abs() < 1e-10);
        assert_eq!(audit.actions.len(), 50);

        // can_spend should use audit-based remaining budget
        let remaining = 1.0 - 0.50;
        assert!((engine.remaining_budget() - remaining).abs() < 1e-10);
    }

    #[test]
    fn test_naive_engine_into_audit_populates_composition_method() {
        let mut engine = PrivacyEngine::from_level(PrivacyLevel::Standard);
        engine.add_noise(42.0, 1.0, "test.col").unwrap();

        let audit = engine.into_audit();
        assert_eq!(audit.composition_method, Some("naive".to_string()));
        assert_eq!(audit.rdp_alpha_effective, None);
    }

    // -----------------------------------------------------------------------
    // RDP composition: tighter effective epsilon than naive for same operations
    // -----------------------------------------------------------------------

    #[test]
    fn test_rdp_tighter_effective_epsilon_than_naive() {
        // Create two engines with the same budget, one naive and one RDP
        let epsilon = 5.0;
        let n_queries = 50;

        let mut naive_config = PrivacyConfig::custom(epsilon, 5);
        naive_config.composition_method = CompositionMethod::Naive;
        let mut naive_engine = PrivacyEngine::new(naive_config);

        let mut rdp_config = PrivacyConfig::custom(epsilon, 5);
        rdp_config.composition_method = CompositionMethod::RenyiDP;
        let mut rdp_engine = PrivacyEngine::new(rdp_config);

        // Apply the same queries to both
        for i in 0..n_queries {
            let target = format!("col_{}", i);
            naive_engine.add_noise(100.0, 1.0, &target).unwrap();
            rdp_engine.add_noise(100.0, 1.0, &target).unwrap();
        }

        // Naive effective epsilon = sum of per-query epsilons = n * (epsilon/100)
        let naive_effective = naive_engine.effective_epsilon();
        let rdp_effective = rdp_engine.effective_epsilon();

        // RDP should give a tighter (lower) effective epsilon for many queries
        assert!(
            rdp_effective < naive_effective,
            "RDP effective epsilon ({:.6}) should be less than naive ({:.6})",
            rdp_effective,
            naive_effective
        );

        // Both should report the same number of actions in the audit
        assert_eq!(naive_engine.audit().actions.len(), n_queries);
        assert_eq!(rdp_engine.audit().actions.len(), n_queries);
    }

    #[test]
    fn test_rdp_engine_into_audit_populates_fields() {
        let mut config = PrivacyConfig::custom(5.0, 5);
        config.composition_method = CompositionMethod::RenyiDP;
        let mut engine = PrivacyEngine::new(config);

        engine.add_noise(42.0, 1.0, "test.col").unwrap();

        let audit = engine.into_audit();
        assert_eq!(audit.composition_method, Some("renyi_dp".to_string()));
        assert!(
            audit.rdp_alpha_effective.is_some(),
            "RDP audit should have optimal alpha set"
        );
    }

    // -----------------------------------------------------------------------
    // zCDP composition
    // -----------------------------------------------------------------------

    #[test]
    fn test_zcdp_tighter_effective_epsilon_than_naive() {
        let epsilon = 5.0;
        let n_queries = 50;

        let mut naive_config = PrivacyConfig::custom(epsilon, 5);
        naive_config.composition_method = CompositionMethod::Naive;
        let mut naive_engine = PrivacyEngine::new(naive_config);

        let mut zcdp_config = PrivacyConfig::custom(epsilon, 5);
        zcdp_config.composition_method = CompositionMethod::ZeroCDP;
        let mut zcdp_engine = PrivacyEngine::new(zcdp_config);

        for i in 0..n_queries {
            let target = format!("col_{}", i);
            naive_engine.add_noise(100.0, 1.0, &target).unwrap();
            zcdp_engine.add_noise(100.0, 1.0, &target).unwrap();
        }

        let naive_effective = naive_engine.effective_epsilon();
        let zcdp_effective = zcdp_engine.effective_epsilon();

        assert!(
            zcdp_effective < naive_effective,
            "zCDP effective epsilon ({:.6}) should be less than naive ({:.6})",
            zcdp_effective,
            naive_effective
        );
    }

    #[test]
    fn test_zcdp_engine_into_audit_populates_fields() {
        let mut config = PrivacyConfig::custom(5.0, 5);
        config.composition_method = CompositionMethod::ZeroCDP;
        let mut engine = PrivacyEngine::new(config);

        engine.add_noise(42.0, 1.0, "test.col").unwrap();

        let audit = engine.into_audit();
        assert_eq!(audit.composition_method, Some("zcdp".to_string()));
        // zCDP does not set rdp_alpha_effective
        assert_eq!(audit.rdp_alpha_effective, None);
    }

    // -----------------------------------------------------------------------
    // Budget exhaustion with composition
    // -----------------------------------------------------------------------

    #[test]
    fn test_naive_budget_exhaustion() {
        // With Naive composition and epsilon=1.0, each query costs epsilon/100 = 0.01.
        // Due to floating-point accumulation, we may not get exactly 100 queries.
        // The key property: the engine MUST eventually refuse queries.
        let mut engine = PrivacyEngine::from_level(PrivacyLevel::Standard);

        let mut succeeded = 0;
        for i in 0..110 {
            match engine.add_noise(1.0, 1.0, &format!("q_{}", i)) {
                Ok(_) => succeeded += 1,
                Err(_) => break,
            }
        }

        // Should get close to 100 queries (floating-point may cause slight variance)
        assert!(
            succeeded >= 99 && succeeded <= 100,
            "Expected ~100 successful queries, got {}",
            succeeded
        );

        // After exhaustion, the next query must fail
        let result = engine.add_noise(1.0, 1.0, "q_overflow");
        assert!(
            result.is_err(),
            "Should fail after exhausting budget with naive composition"
        );
    }

    #[test]
    fn test_rdp_budget_allows_more_queries_than_naive_before_exhaustion() {
        // RDP with composition should allow more queries before the
        // effective epsilon reaches the budget, compared to naive.
        //
        // With epsilon=1.0 and naive: exactly 100 queries at 0.01 each.
        // With RDP: effective_epsilon grows sub-linearly, so more queries fit.
        let mut rdp_config = PrivacyConfig::custom(1.0, 5);
        rdp_config.composition_method = CompositionMethod::RenyiDP;
        let mut rdp_engine = PrivacyEngine::new(rdp_config);

        let mut count = 0;
        for i in 0..500 {
            // Try up to 500, should get more than 100
            let result = rdp_engine.add_noise(1.0, 1.0, &format!("q_{}", i));
            if result.is_err() {
                break;
            }
            count += 1;
        }

        assert!(
            count > 100,
            "RDP engine should allow more than 100 queries (got {}), since \
             effective epsilon grows sub-linearly with composition",
            count
        );
    }

    // -----------------------------------------------------------------------
    // PrivacyMetadata builder
    // -----------------------------------------------------------------------

    #[test]
    fn test_build_privacy_metadata_naive() {
        let engine = PrivacyEngine::from_level(PrivacyLevel::Standard);
        let meta = engine.build_privacy_metadata();

        assert_eq!(meta.composition_method, Some("naive".to_string()));
        assert_eq!(meta.delta, None);
        assert!((meta.epsilon - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_build_privacy_metadata_rdp() {
        let mut config = PrivacyConfig::custom(2.0, 5);
        config.composition_method = CompositionMethod::RenyiDP;
        let engine = PrivacyEngine::new(config);
        let meta = engine.build_privacy_metadata();

        assert_eq!(meta.composition_method, Some("renyi_dp".to_string()));
        assert!(meta.delta.is_some(), "RDP metadata should include delta");
        assert!((meta.delta.unwrap() - 1e-5).abs() < 1e-15);
        assert!((meta.epsilon - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_build_privacy_metadata_zcdp() {
        let mut config = PrivacyConfig::custom(2.0, 5);
        config.composition_method = CompositionMethod::ZeroCDP;
        let engine = PrivacyEngine::new(config);
        let meta = engine.build_privacy_metadata();

        assert_eq!(meta.composition_method, Some("zcdp".to_string()));
        assert!(meta.delta.is_some(), "zCDP metadata should include delta");
        assert!((meta.delta.unwrap() - 1e-5).abs() < 1e-15);
    }

    // -----------------------------------------------------------------------
    // Accountant remaining budget vs audit remaining budget
    // -----------------------------------------------------------------------

    #[test]
    fn test_accountant_remaining_budget_tighter_for_rdp() {
        let mut config = PrivacyConfig::custom(5.0, 5);
        config.composition_method = CompositionMethod::RenyiDP;
        let mut engine = PrivacyEngine::new(config);

        // Add 50 queries
        for i in 0..50 {
            engine.add_noise(1.0, 1.0, &format!("q_{}", i)).unwrap();
        }

        let audit_remaining = engine.remaining_budget();
        let accountant_remaining = engine.accountant_remaining_budget();

        // Audit remaining = total - sum(per_query_eps) = 5.0 - 50*0.05 = 2.5
        assert!((audit_remaining - 2.5).abs() < 1e-10);

        // Accountant remaining should be larger (because effective eps < sum)
        assert!(
            accountant_remaining > audit_remaining,
            "Accountant remaining ({:.4}) should be greater than audit remaining ({:.4}) for RDP",
            accountant_remaining,
            audit_remaining
        );
    }
}
