# Spec 08: Enhanced Realism Validation & Metrics

**Status**: Draft
**Priority**: High
**Depends on**: Specs 01-07 (all external realism modules)
**Extends**: `datasynth-eval/src/` (evaluation framework)

---

## 1. Problem Statement

The existing evaluation module (`datasynth-eval`) validates statistical properties
(Benford's Law, distributions, temporal patterns), coherence (balance validation,
document chains), and data quality. However, with the introduction of external
realism factors, we need **new validation dimensions** that assess whether the
generated data exhibits realistic responses to macroeconomic conditions, shock
events, and cross-factor dynamics. Without these, we cannot quantify whether the
external realism enhancements actually improve synthetic data quality.

## 2. Scientific Foundation

### 2.1 Synthetic Data Evaluation Framework (Three Pillars)

Modern synthetic data evaluation assesses three dimensions (Alaa et al., 2022):

1. **Fidelity** — Statistical similarity to real data (marginals, joints, temporal)
2. **Utility** — Downstream task performance (ML model accuracy on synthetic vs. real)
3. **Privacy** — Resistance to re-identification attacks

For enterprise financial data, we add a fourth dimension:
4. **Coherence** — Internal consistency of accounting/financial relationships

### 2.2 Distributional Distance Metrics

| Metric | Formula | Properties | Use Case |
|--------|---------|-----------|----------|
| **Jensen-Shannon Divergence** | JSD(P,Q) = ½KL(P‖M) + ½KL(Q‖M) | Symmetric, bounded [0, ln2] | Discrete/categorical columns |
| **Wasserstein Distance** | W_p(P,Q) = inf E[‖X-Y‖^p]^{1/p} | True metric, captures geometry | Continuous distributions |
| **Kolmogorov-Smirnov** | D = sup_x ‖F_n(x) - F(x)‖ | Non-parametric, distribution-free | Goodness-of-fit |
| **Maximum Mean Discrepancy** | MMD = sup_{f∈H} (E_P[f] - E_Q[f]) | Kernel-based, captures all moments | Non-parametric two-sample test |
| **Anderson-Darling** | A² = -n - Σ[(2i-1)/n][ln F(Y_i) + ln(1-F(Y_{n+1-i}))] | Weights tails more than K-S | Tail behavior validation |

**Reference**: Alaa, A., Van Breugel, B., Saveliev, E., & van der Schaar, M. (2022).
"How Faithful is your Synthetic Data?" *ICML 2022*.

### 2.3 Classifier Two-Sample Test (C2ST)

Train a binary classifier to distinguish real from synthetic data. If accuracy ≈ 0.5,
the synthetic data is indistinguishable. If accuracy > 0.7, systematic differences exist.

```
C2ST = Accuracy(classifier: real vs. synthetic)
```

**Interpretation**:
- 0.50: Perfect synthetic data (indistinguishable)
- 0.50-0.60: Good synthetic data
- 0.60-0.70: Moderate differences detectable
- 0.70+: Significant systematic differences

**Reference**: Lopez-Paz, D. & Oquab, M. (2017). "Revisiting Classifier Two-Sample
Tests." *ICLR 2017*.

### 2.4 Benford's Law Compliance (Financial-Specific)

For financial transaction data, Benford's Law conformance is a critical realism test.

**Expected first-digit distribution**: P(d) = log₁₀(1 + 1/d)

| Digit | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
|-------|------|------|------|------|------|------|------|------|------|
| P(d) | 30.1% | 17.6% | 12.5% | 9.7% | 7.9% | 6.7% | 5.8% | 5.1% | 4.6% |

**Conformity thresholds** (MAD = Mean Absolute Deviation):
- MAD < 0.006: Close conformity
- 0.006-0.012: Acceptable conformity
- 0.012-0.015: Marginal conformity
- MAD > 0.015: Non-conformity

**Reference**: Nigrini, M.J. (2012). *Benford's Law: Applications for Forensic
Accounting, Auditing, and Fraud Detection*. Wiley.

### 2.5 Temporal Realism Metrics

Financial time series have characteristic patterns that synthetic data must replicate:

1. **Autocorrelation structure**: Transaction volumes exhibit strong weekly/monthly
   autocorrelation (ACF decays slowly)
2. **Volatility clustering**: Periods of high variance cluster together (GARCH effects)
3. **Regime persistence**: Economic regimes last months-years, not days
4. **Structural breaks**: Shocks create detectable change points
5. **Seasonality**: Month-end, quarter-end, year-end spikes must match real patterns
6. **Fat tails**: Financial returns exhibit excess kurtosis (kurtosis > 3)

## 3. Proposed Design

### 3.1 Realism Metric Categories

```rust
/// Complete realism assessment of generated synthetic data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealismReport {
    /// Overall realism score (0.0-1.0)
    pub overall_score: f64,
    /// Category-level scores
    pub categories: RealismCategories,
    /// Individual metric results
    pub metrics: Vec<MetricResult>,
    /// Identified weaknesses
    pub weaknesses: Vec<RealismWeakness>,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealismCategories {
    /// Statistical fidelity (distributions, correlations)
    pub statistical_fidelity: CategoryScore,
    /// Temporal realism (patterns, seasonality, volatility clustering)
    pub temporal_realism: CategoryScore,
    /// Financial coherence (accounting rules, balance constraints)
    pub financial_coherence: CategoryScore,
    /// Macroeconomic realism (factor co-movement, regime dynamics)
    pub macroeconomic_realism: CategoryScore,
    /// Shock response realism (impact shapes, recovery patterns)
    pub shock_response_realism: CategoryScore,
    /// Behavioral realism (entity-level behavioral patterns)
    pub behavioral_realism: CategoryScore,
    /// Cross-factor coherence (structural relationships hold)
    pub cross_factor_coherence: CategoryScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScore {
    pub score: f64,
    pub weight: f64,
    pub metric_count: usize,
    pub pass_count: usize,
    pub details: Vec<MetricResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricResult {
    pub name: String,
    pub category: MetricCategory,
    pub value: f64,
    pub threshold: f64,
    pub passed: bool,
    pub details: String,
}
```

### 3.2 Statistical Fidelity Metrics

```rust
pub struct StatisticalFidelityValidator {
    /// Reference distribution parameters (if available)
    reference: Option<ReferenceDistributions>,
}

impl StatisticalFidelityValidator {
    /// Run all statistical fidelity checks
    pub fn validate(&self, data: &GeneratedData) -> Vec<MetricResult> {
        let mut results = Vec::new();

        // 1. Benford's Law (first digit)
        results.push(self.benford_first_digit(&data.transaction_amounts()));

        // 2. Benford's Law (second digit)
        results.push(self.benford_second_digit(&data.transaction_amounts()));

        // 3. Distribution fit (amounts should be lognormal)
        results.push(self.distribution_fit_test(
            &data.transaction_amounts(),
            DistributionType::LogNormal,
        ));

        // 4. Correlation matrix preservation
        results.push(self.correlation_preservation(&data));

        // 5. Marginal distribution comparison (per column)
        for column in data.numeric_columns() {
            results.push(self.marginal_distribution_test(&column));
        }

        // 6. Joint distribution test (2D)
        results.push(self.joint_distribution_test(
            &data.amount_column(),
            &data.line_items_column(),
        ));

        // 7. Tail behavior (excess kurtosis)
        results.push(self.tail_behavior_test(&data.transaction_amounts()));

        // 8. Zero-inflation check (certain fields should have excess zeros)
        results.push(self.zero_inflation_check(&data));

        results
    }

    fn benford_first_digit(&self, amounts: &[f64]) -> MetricResult {
        let observed = compute_first_digit_frequencies(amounts);
        let expected = benford_expected_frequencies();
        let mad = mean_absolute_deviation(&observed, &expected);
        let chi2 = chi_squared_statistic(&observed, &expected, amounts.len());
        let chi2_critical = 15.507; // χ²(8, 0.05)

        MetricResult {
            name: "Benford First Digit".into(),
            category: MetricCategory::StatisticalFidelity,
            value: mad,
            threshold: 0.015,
            passed: mad < 0.015 && chi2 < chi2_critical,
            details: format!(
                "MAD={:.4} (threshold <0.015), χ²={:.2} (critical {:.2})",
                mad, chi2, chi2_critical
            ),
        }
    }

    fn correlation_preservation(&self, data: &GeneratedData) -> MetricResult {
        let realized = data.compute_correlation_matrix();
        let target = self.reference.as_ref()
            .map(|r| r.correlation_matrix.clone())
            .unwrap_or_else(|| realized.clone()); // Self-consistency if no reference

        let frobenius = frobenius_norm_difference(&realized, &target);

        MetricResult {
            name: "Correlation Matrix Preservation".into(),
            category: MetricCategory::StatisticalFidelity,
            value: frobenius,
            threshold: 0.20,
            passed: frobenius < 0.20,
            details: format!("Frobenius norm difference: {:.4}", frobenius),
        }
    }

    fn tail_behavior_test(&self, amounts: &[f64]) -> MetricResult {
        let kurtosis = compute_excess_kurtosis(amounts);
        // Financial data typically has excess kurtosis > 0 (leptokurtic)
        MetricResult {
            name: "Excess Kurtosis (Fat Tails)".into(),
            category: MetricCategory::StatisticalFidelity,
            value: kurtosis,
            threshold: 0.0,
            passed: kurtosis > 0.0, // Should be positive for financial data
            details: format!("Excess kurtosis: {:.2} (expected >0 for financial data)", kurtosis),
        }
    }
}
```

### 3.3 Temporal Realism Metrics

```rust
pub struct TemporalRealismValidator;

impl TemporalRealismValidator {
    pub fn validate(&self, data: &GeneratedData) -> Vec<MetricResult> {
        let mut results = Vec::new();
        let time_series = data.as_time_series();

        // 1. Autocorrelation structure
        results.push(self.autocorrelation_check(&time_series));

        // 2. Volatility clustering (ARCH effects)
        results.push(self.volatility_clustering(&time_series));

        // 3. Month-end spike detection
        results.push(self.month_end_spike(&time_series));

        // 4. Quarter-end spike detection
        results.push(self.quarter_end_spike(&time_series));

        // 5. Weekend/holiday suppression
        results.push(self.weekend_holiday_check(&time_series));

        // 6. Structural break detection
        results.push(self.structural_break_detection(&time_series));

        // 7. Regime persistence
        results.push(self.regime_persistence_check(&time_series));

        // 8. Intraday pattern (if timestamps available)
        if time_series.has_intraday() {
            results.push(self.intraday_pattern_check(&time_series));
        }

        results
    }

    fn volatility_clustering(&self, ts: &TimeSeries) -> MetricResult {
        // Test for ARCH effects: autocorrelation of squared returns
        let returns = ts.compute_returns();
        let sq_returns: Vec<f64> = returns.iter().map(|r| r * r).collect();
        let acf_sq = autocorrelation(&sq_returns, 5);

        // Ljung-Box test on squared returns
        let lb_statistic = ljung_box_statistic(&sq_returns, 5, sq_returns.len());
        let critical_value = 11.07; // χ²(5, 0.05)

        MetricResult {
            name: "Volatility Clustering (ARCH)".into(),
            category: MetricCategory::TemporalRealism,
            value: lb_statistic,
            threshold: critical_value,
            passed: true, // Both presence and absence can be valid
            details: format!(
                "Ljung-Box Q={:.2} (critical {:.2}), ACF(1) of squared returns: {:.3}",
                lb_statistic, critical_value, acf_sq[0]
            ),
        }
    }

    fn month_end_spike(&self, ts: &TimeSeries) -> MetricResult {
        let month_end_volume = ts.volume_near_month_end(5); // Last 5 days
        let mid_month_volume = ts.volume_mid_month();
        let ratio = month_end_volume / mid_month_volume;

        // Real financial data typically shows 1.5-4.0x month-end spikes
        MetricResult {
            name: "Month-End Volume Spike".into(),
            category: MetricCategory::TemporalRealism,
            value: ratio,
            threshold: 1.3,
            passed: ratio > 1.3 && ratio < 8.0,
            details: format!(
                "Month-end/mid-month ratio: {:.2}x (expected 1.5-4.0x)",
                ratio
            ),
        }
    }

    fn structural_break_detection(&self, ts: &TimeSeries) -> MetricResult {
        // Detect structural breaks using CUSUM or Bai-Perron methodology
        let breaks = cusum_test(&ts.values, 0.95);
        let expected_breaks = ts.expected_shock_periods(); // From scenario config

        let detected_ratio = if expected_breaks.is_empty() {
            1.0 // No shocks configured, any result is valid
        } else {
            let detected = breaks.iter()
                .filter(|b| expected_breaks.iter().any(|e| (b.period as i32 - *e as i32).unsigned_abs() <= 2))
                .count();
            detected as f64 / expected_breaks.len() as f64
        };

        MetricResult {
            name: "Structural Break Detection".into(),
            category: MetricCategory::TemporalRealism,
            value: detected_ratio,
            threshold: 0.70,
            passed: detected_ratio >= 0.70,
            details: format!(
                "Detected {}/{} expected structural breaks ({:.0}%)",
                (detected_ratio * expected_breaks.len() as f64) as usize,
                expected_breaks.len(),
                detected_ratio * 100.0
            ),
        }
    }
}
```

### 3.4 Macroeconomic Realism Metrics

```rust
pub struct MacroRealismValidator;

impl MacroRealismValidator {
    pub fn validate(
        &self,
        data: &GeneratedData,
        macro_factors: &[MacroFactors],
    ) -> Vec<MetricResult> {
        let mut results = Vec::new();

        // 1. GDP-Volume correlation
        results.push(self.gdp_volume_correlation(data, macro_factors));

        // 2. Inflation-Pricing correlation
        results.push(self.inflation_pricing_correlation(data, macro_factors));

        // 3. Credit spread-Default correlation
        results.push(self.credit_default_correlation(data, macro_factors));

        // 4. Recession impact asymmetry (declines sharper than recoveries)
        results.push(self.recession_asymmetry(data, macro_factors));

        // 5. Sector differentiation (different sectors respond differently)
        results.push(self.sector_differentiation(data, macro_factors));

        // 6. Regime transition smoothness
        results.push(self.regime_transition_smoothness(macro_factors));

        // 7. Factor autocorrelation (macro factors are persistent)
        results.push(self.factor_autocorrelation(macro_factors));

        // 8. Yield curve shape realism
        if let Some(yc) = macro_factors.first().map(|f| &f.yield_curve) {
            results.push(self.yield_curve_realism(macro_factors));
        }

        results
    }

    fn gdp_volume_correlation(
        &self,
        data: &GeneratedData,
        factors: &[MacroFactors],
    ) -> MetricResult {
        let gdp_series: Vec<f64> = factors.iter().map(|f| f.gdp_growth).collect();
        let volume_series = data.transaction_volume_by_period();

        let correlation = pearson_correlation(&gdp_series, &volume_series);

        // GDP and transaction volume should be positively correlated
        MetricResult {
            name: "GDP-Volume Correlation".into(),
            category: MetricCategory::MacroRealism,
            value: correlation,
            threshold: 0.3,
            passed: correlation > 0.3,
            details: format!(
                "Correlation between GDP growth and transaction volume: {:.3} (expected >0.3)",
                correlation
            ),
        }
    }

    fn recession_asymmetry(
        &self,
        data: &GeneratedData,
        factors: &[MacroFactors],
    ) -> MetricResult {
        // Real recessions are sharper than recoveries (asymmetric)
        let recession_periods: Vec<usize> = factors.iter()
            .enumerate()
            .filter(|(_, f)| matches!(f.regime, EconomicRegime::Contraction))
            .map(|(i, _)| i)
            .collect();

        if recession_periods.is_empty() {
            return MetricResult {
                name: "Recession Asymmetry".into(),
                category: MetricCategory::MacroRealism,
                value: 1.0,
                threshold: 0.0,
                passed: true,
                details: "No recession in scenario; check skipped".into(),
            };
        }

        let volumes = data.transaction_volume_by_period();
        let decline_speed = compute_decline_speed(&volumes, &recession_periods);
        let recovery_speed = compute_recovery_speed(&volumes, &recession_periods);

        // Decline should be faster than recovery
        let asymmetry_ratio = decline_speed / recovery_speed.max(0.001);

        MetricResult {
            name: "Recession Asymmetry".into(),
            category: MetricCategory::MacroRealism,
            value: asymmetry_ratio,
            threshold: 1.0,
            passed: asymmetry_ratio > 1.0, // Decline faster than recovery
            details: format!(
                "Decline speed / recovery speed = {:.2} (expected >1.0, real data ~1.5-2.5)",
                asymmetry_ratio
            ),
        }
    }

    fn sector_differentiation(
        &self,
        data: &GeneratedData,
        factors: &[MacroFactors],
    ) -> MetricResult {
        // Different sectors should respond differently to the same macro conditions
        let sector_responses = data.compute_sector_gdp_betas(factors);

        if sector_responses.len() < 2 {
            return MetricResult {
                name: "Sector Differentiation".into(),
                category: MetricCategory::MacroRealism,
                value: 0.0,
                threshold: 0.0,
                passed: true,
                details: "Fewer than 2 sectors; check skipped".into(),
            };
        }

        let betas: Vec<f64> = sector_responses.values().cloned().collect();
        let spread = betas.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
            - betas.iter().cloned().fold(f64::INFINITY, f64::min);

        MetricResult {
            name: "Sector Differentiation".into(),
            category: MetricCategory::MacroRealism,
            value: spread,
            threshold: 0.3,
            passed: spread > 0.3,
            details: format!(
                "GDP β spread across sectors: {:.2} (expected >0.3)",
                spread
            ),
        }
    }
}
```

### 3.5 Shock Response Metrics

```rust
pub struct ShockResponseValidator;

impl ShockResponseValidator {
    pub fn validate(
        &self,
        data: &GeneratedData,
        shocks: &[ExternalShock],
    ) -> Vec<MetricResult> {
        let mut results = Vec::new();

        for shock in shocks {
            // 1. Impact detection — can we detect the shock in the data?
            results.push(self.impact_detection(data, shock));

            // 2. Recovery shape — does recovery match configured shape?
            results.push(self.recovery_shape_match(data, shock));

            // 3. Impact magnitude — is the magnitude approximately correct?
            results.push(self.impact_magnitude(data, shock));

            // 4. Channel specificity — do affected channels show impact?
            results.push(self.channel_specificity(data, shock));

            // 5. Lag compliance — do lagged channels activate after the delay?
            results.push(self.lag_compliance(data, shock));
        }

        // Cross-shock metrics
        if shocks.len() >= 2 {
            results.push(self.shock_interaction_realism(data, shocks));
        }

        results
    }

    fn impact_detection(
        &self,
        data: &GeneratedData,
        shock: &ExternalShock,
    ) -> MetricResult {
        let volumes = data.transaction_volume_by_period();
        let onset = shock.onset_period;

        // Compare pre-shock mean to post-shock minimum
        let pre_mean = volumes[..onset].iter().sum::<f64>() / onset as f64;
        let post_window = &volumes[onset..onset.min(volumes.len()).max(onset) + 6];
        let post_min = post_window.iter().cloned().fold(f64::INFINITY, f64::min);

        let impact_detected = (pre_mean - post_min) / pre_mean > 0.05;

        MetricResult {
            name: format!("Shock Detection: {}", shock.id),
            category: MetricCategory::ShockResponse,
            value: (pre_mean - post_min) / pre_mean,
            threshold: 0.05,
            passed: impact_detected,
            details: format!(
                "Pre-shock mean: {:.0}, post-shock min: {:.0}, decline: {:.1}%",
                pre_mean, post_min, (pre_mean - post_min) / pre_mean * 100.0
            ),
        }
    }

    fn recovery_shape_match(
        &self,
        data: &GeneratedData,
        shock: &ExternalShock,
    ) -> MetricResult {
        let volumes = data.transaction_volume_by_period();
        let onset = shock.onset_period;
        let recovery_start = onset + shock.lifecycle.ramp_up_periods
            + shock.lifecycle.peak_duration_periods;

        if recovery_start >= volumes.len() {
            return MetricResult {
                name: format!("Recovery Shape: {}", shock.id),
                category: MetricCategory::ShockResponse,
                value: 0.0,
                threshold: 0.0,
                passed: true,
                details: "Recovery period outside data range".into(),
            };
        }

        let recovery_data = &volumes[recovery_start..];
        let expected_shape = &shock.recovery.shape;

        // Compute shape match score (0-1)
        let shape_score = match expected_shape {
            RecoveryShape::V => self.v_shape_score(recovery_data),
            RecoveryShape::U => self.u_shape_score(recovery_data),
            RecoveryShape::L => self.l_shape_score(recovery_data),
            RecoveryShape::Swoosh => self.swoosh_shape_score(recovery_data),
            _ => 0.5, // Default pass for other shapes
        };

        MetricResult {
            name: format!("Recovery Shape ({:?}): {}", expected_shape, shock.id),
            category: MetricCategory::ShockResponse,
            value: shape_score,
            threshold: 0.5,
            passed: shape_score > 0.5,
            details: format!(
                "Recovery shape match score: {:.2} (expected >0.5 for {:?})",
                shape_score, expected_shape
            ),
        }
    }
}
```

### 3.6 Classifier Two-Sample Test

```rust
pub struct ClassifierTwoSampleTest {
    /// Number of features to use
    n_features: usize,
    /// Train/test split ratio
    test_ratio: f64,
    /// Number of cross-validation folds
    cv_folds: usize,
}

impl ClassifierTwoSampleTest {
    /// Run C2ST comparing synthetic data against reference statistics
    pub fn run(
        &self,
        synthetic: &GeneratedData,
        reference_stats: &ReferenceStatistics,
    ) -> MetricResult {
        // Extract feature vectors from synthetic data
        let synthetic_features = self.extract_features(synthetic);

        // Generate reference features from reference statistics
        let reference_features = reference_stats.sample_features(
            synthetic_features.len(),
        );

        // Label: 0 = reference, 1 = synthetic
        let labels: Vec<u8> = std::iter::repeat(0)
            .take(reference_features.len())
            .chain(std::iter::repeat(1).take(synthetic_features.len()))
            .collect();

        let features: Vec<Vec<f64>> = reference_features.into_iter()
            .chain(synthetic_features.into_iter())
            .collect();

        // Train simple logistic regression classifier with CV
        let accuracy = self.cross_validated_accuracy(&features, &labels);

        MetricResult {
            name: "Classifier Two-Sample Test (C2ST)".into(),
            category: MetricCategory::StatisticalFidelity,
            value: accuracy,
            threshold: 0.60,
            passed: accuracy < 0.60, // Lower is better — closer to 0.5 = indistinguishable
            details: format!(
                "C2ST accuracy: {:.3} (ideal ≈0.50, threshold <0.60)",
                accuracy
            ),
        }
    }
}
```

### 3.7 Comprehensive Realism Scorer

```rust
impl RealismReport {
    /// Compute overall realism score as weighted average of categories
    pub fn compute_overall_score(&mut self) {
        let weights = [
            (&self.categories.statistical_fidelity, 0.20),
            (&self.categories.temporal_realism, 0.15),
            (&self.categories.financial_coherence, 0.20),
            (&self.categories.macroeconomic_realism, 0.15),
            (&self.categories.shock_response_realism, 0.10),
            (&self.categories.behavioral_realism, 0.10),
            (&self.categories.cross_factor_coherence, 0.10),
        ];

        let weighted_sum: f64 = weights.iter()
            .map(|(cat, w)| cat.score * w)
            .sum();
        let total_weight: f64 = weights.iter().map(|(_, w)| w).sum();

        self.overall_score = weighted_sum / total_weight;
    }

    /// Generate human-readable summary
    pub fn summary(&self) -> String {
        let grade = match self.overall_score {
            s if s >= 0.90 => "Excellent",
            s if s >= 0.80 => "Good",
            s if s >= 0.70 => "Acceptable",
            s if s >= 0.60 => "Fair",
            _ => "Needs Improvement",
        };

        format!(
            "Realism Score: {:.1}% ({})\n\
             Statistical Fidelity: {:.1}%\n\
             Temporal Realism: {:.1}%\n\
             Financial Coherence: {:.1}%\n\
             Macro Realism: {:.1}%\n\
             Shock Response: {:.1}%\n\
             Behavioral: {:.1}%\n\
             Cross-Factor: {:.1}%\n\
             Violations: {}, Warnings: {}",
            self.overall_score * 100.0,
            grade,
            self.categories.statistical_fidelity.score * 100.0,
            self.categories.temporal_realism.score * 100.0,
            self.categories.financial_coherence.score * 100.0,
            self.categories.macroeconomic_realism.score * 100.0,
            self.categories.shock_response_realism.score * 100.0,
            self.categories.behavioral_realism.score * 100.0,
            self.categories.cross_factor_coherence.score * 100.0,
            self.metrics.iter().filter(|m| !m.passed).count(),
            self.weaknesses.len(),
        )
    }
}
```

## 4. Configuration Schema

```yaml
external_realism:
  realism_validation:
    enabled: true

    # Which validation categories to run
    categories:
      statistical_fidelity:
        enabled: true
        weight: 0.20
        tests:
          - benford_first_digit:
              threshold_mad: 0.015
          - benford_second_digit:
              threshold_mad: 0.020
          - distribution_fit:
              target: lognormal
              significance: 0.05
          - correlation_preservation:
              threshold_frobenius: 0.20
          - tail_behavior:
              min_excess_kurtosis: 0.0
          - classifier_two_sample:
              threshold_accuracy: 0.60
              cv_folds: 5

      temporal_realism:
        enabled: true
        weight: 0.15
        tests:
          - autocorrelation:
              min_lag1: 0.3
          - volatility_clustering:
              significance: 0.05
          - month_end_spike:
              min_ratio: 1.3
              max_ratio: 8.0
          - weekend_suppression:
              max_weekend_ratio: 0.15
          - structural_breaks:
              detection_rate: 0.70

      financial_coherence:
        enabled: true
        weight: 0.20
        tests:
          - balance_equation:
              tolerance: 0.01
          - document_chain_integrity:
              missing_reference_rate: 0.02
          - trial_balance:
              tolerance: 0.01

      macroeconomic_realism:
        enabled: true
        weight: 0.15
        tests:
          - gdp_volume_correlation:
              min_correlation: 0.3
          - inflation_pricing_correlation:
              min_correlation: 0.2
          - recession_asymmetry:
              min_ratio: 1.0
          - sector_differentiation:
              min_beta_spread: 0.3
          - factor_autocorrelation:
              min_lag1: 0.5

      shock_response:
        enabled: true
        weight: 0.10
        tests:
          - impact_detection:
              min_decline: 0.05
          - recovery_shape_match:
              min_score: 0.5
          - channel_specificity:
              min_affected_ratio: 0.7
          - lag_compliance:
              tolerance_periods: 1

      behavioral_realism:
        enabled: true
        weight: 0.10
        tests:
          - payment_term_drift:
              expected_direction: positive_in_recession
          - approval_pattern_realism:
              eom_spike_expected: true

      cross_factor_coherence:
        enabled: true
        weight: 0.10
        tests:
          - sign_consistency:
              pass_rate: 0.95
          - taylor_rule_deviation:
              max_deviation_bps: 100
          - correlation_matrix:
              max_element_deviation: 0.15

    # Action on validation failure
    fail_on_score_below: null             # Optional: fail if score < threshold
    warn_on_score_below: 0.70

    # Reference data (if available for comparison)
    reference_data: null                  # Path to reference dataset
    reference_statistics: null            # Path to pre-computed reference stats

    # Output
    export_report: true                   # Export full realism report
    export_format: json                   # json | csv | html
    export_visualizations: true           # Generate charts (requires gnuplot)
```

## 5. Report Output Format

```json
{
  "overall_score": 0.847,
  "grade": "Good",
  "timestamp": "2026-03-01T12:00:00Z",
  "scenario": "2008_financial_crisis",
  "categories": {
    "statistical_fidelity": {
      "score": 0.92,
      "tests": [
        { "name": "Benford First Digit", "value": 0.0042, "threshold": 0.015, "passed": true },
        { "name": "Distribution Fit (LogNormal)", "value": 0.034, "threshold": 0.05, "passed": true },
        { "name": "Correlation Preservation", "value": 0.14, "threshold": 0.20, "passed": true },
        { "name": "C2ST", "value": 0.54, "threshold": 0.60, "passed": true }
      ]
    },
    "macroeconomic_realism": {
      "score": 0.81,
      "tests": [
        { "name": "GDP-Volume Correlation", "value": 0.67, "threshold": 0.30, "passed": true },
        { "name": "Recession Asymmetry", "value": 1.8, "threshold": 1.0, "passed": true },
        { "name": "Sector Differentiation", "value": 0.45, "threshold": 0.30, "passed": true }
      ]
    }
  },
  "weaknesses": [
    {
      "category": "temporal_realism",
      "metric": "Month-End Spike",
      "description": "Month-end spike ratio (1.2x) below expected range (1.3-8.0x)",
      "recommendation": "Increase period_end.month_end.peak_multiplier to at least 2.0"
    }
  ]
}
```

## 6. Integration with Auto-Tuner

The realism report feeds into the existing `datasynth-eval` auto-tuner to automatically
adjust configuration parameters:

```rust
impl AutoTuner {
    /// Generate config patches from realism report weaknesses
    pub fn patches_from_realism(
        &self,
        report: &RealismReport,
    ) -> Vec<ConfigPatch> {
        report.weaknesses.iter()
            .filter_map(|weakness| {
                match weakness.category.as_str() {
                    "temporal_realism" => self.temporal_patch(weakness),
                    "macroeconomic_realism" => self.macro_patch(weakness),
                    "statistical_fidelity" => self.statistical_patch(weakness),
                    _ => None,
                }
            })
            .collect()
    }
}
```

## 7. Testing Strategy

- **Score stability**: Same data + config produces same realism score
- **Known-good detection**: Well-configured generation scores >0.80
- **Known-bad detection**: Intentionally broken config scores <0.50
- **Metric independence**: Failing one metric doesn't artificially lower unrelated categories
- **Threshold calibration**: Thresholds produce <5% false positive rate on valid data
- **Auto-tuner integration**: Patches improve realism score on re-generation

## References

1. Alaa, A., Van Breugel, B., Saveliev, E., & van der Schaar, M. (2022). "How Faithful is your Synthetic Data?" *ICML 2022*.
2. Lopez-Paz, D. & Oquab, M. (2017). "Revisiting Classifier Two-Sample Tests." *ICLR 2017*.
3. Nigrini, M.J. (2012). *Benford's Law*. Wiley.
4. Assefa, S.A., et al. (2020). "Generating Synthetic Data in Finance." Alan Turing Institute.
5. Park, N., et al. (2018). "Data Synthesis based on Generative Adversarial Networks." *PVLDB*, 11(10).
6. Dankar, F.K. & Ibrahim, M. (2021). "Fake It Till You Make It: Guidelines for Effective Synthetic Data Generation." *Applied Sciences*, 11(5), 2158.
7. Chundawat, N.S., et al. (2022). "A Survey on Evaluation Metrics for Synthetic Data Generation." *ACM Computing Surveys*.
