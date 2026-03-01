//! Integration tests for BenfordAnalyzer.
//!
//! Validates Benford's Law analysis across a range of input distributions
//! including log-normal (naturally conforming), uniform (non-conforming),
//! edge cases with empty/small samples, and inputs containing zeros/negatives.

use datasynth_eval::{BenfordAnalyzer, BenfordConformity, EvalError};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, LogNormal, Uniform};
use rust_decimal::Decimal;

/// Generate amounts from a log-normal distribution, which naturally follows
/// Benford's Law. Uses a deterministic seed for reproducibility.
fn generate_lognormal_amounts(count: usize, seed: u64) -> Vec<Decimal> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let ln = LogNormal::new(6.0, 1.5).expect("valid log-normal params");
    (0..count)
        .map(|_| {
            let value = ln.sample(&mut rng);
            // Round to 2 decimal places for realistic financial amounts
            Decimal::from_f64_retain(value)
                .unwrap_or(Decimal::ONE)
                .round_dp(2)
        })
        .collect()
}

/// Generate uniformly distributed amounts in [1, 1000], which should NOT
/// follow Benford's Law.
fn generate_uniform_amounts(count: usize, seed: u64) -> Vec<Decimal> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let uniform = Uniform::new(1.0_f64, 1000.0).expect("valid uniform params");
    (0..count)
        .map(|_| {
            let value = uniform.sample(&mut rng);
            Decimal::from_f64_retain(value)
                .unwrap_or(Decimal::ONE)
                .round_dp(2)
        })
        .collect()
}

#[test]
fn test_benford_conforming_data() {
    // Log-normal distribution naturally follows Benford's Law
    let amounts = generate_lognormal_amounts(1000, 42);
    let analyzer = BenfordAnalyzer::new(0.05);
    let result = analyzer
        .analyze(&amounts)
        .expect("analysis should succeed with 1000 log-normal samples");

    assert_eq!(result.sample_size, 1000);
    assert_eq!(result.degrees_of_freedom, 8);
    assert!(
        result.passes,
        "Log-normal data should pass Benford test (p_value={}, mad={})",
        result.p_value, result.mad
    );
    assert!(
        result.conformity == BenfordConformity::Close
            || result.conformity == BenfordConformity::Acceptable,
        "Expected Close or Acceptable conformity, got {:?} (mad={})",
        result.conformity,
        result.mad
    );
    // Verify observed frequencies sum to approximately 1.0
    let freq_sum: f64 = result.observed_frequencies.iter().sum();
    assert!(
        (freq_sum - 1.0).abs() < 0.001,
        "Observed frequencies should sum to ~1.0, got {}",
        freq_sum
    );
}

#[test]
fn test_benford_uniform_data_fails() {
    // Uniform distribution should NOT follow Benford's Law
    let amounts = generate_uniform_amounts(1000, 99);
    let analyzer = BenfordAnalyzer::new(0.05);
    let result = analyzer
        .analyze(&amounts)
        .expect("analysis should succeed with 1000 uniform samples");

    assert_eq!(result.sample_size, 1000);
    // Uniform data should fail or show non-conforming
    assert!(
        !result.passes || result.conformity == BenfordConformity::NonConforming,
        "Uniform data should fail Benford test or be NonConforming (passes={}, conformity={:?}, p_value={}, mad={})",
        result.passes,
        result.conformity,
        result.p_value,
        result.mad
    );
}

#[test]
fn test_benford_empty_data() {
    let amounts: Vec<Decimal> = Vec::new();
    let analyzer = BenfordAnalyzer::new(0.05);
    let result = analyzer.analyze(&amounts);

    assert!(result.is_err(), "Empty data should return an error");
    match result {
        Err(EvalError::InsufficientData { required, actual }) => {
            assert_eq!(required, 10);
            assert_eq!(actual, 0);
        }
        other => panic!("Expected InsufficientData error, got {:?}", other.err()),
    }
}

#[test]
fn test_benford_small_sample() {
    // 10 amounts -- exactly at the minimum threshold
    let amounts: Vec<Decimal> = (1..=10).map(|i| Decimal::new(i * 100 + 50, 2)).collect();
    let analyzer = BenfordAnalyzer::new(0.05);
    // With exactly 10 samples the analyzer requires >= 10 valid first digits.
    // All values 1.50 through 10.50 have first digits 1..9 plus one starting with 1 (10.50).
    // That gives 10 valid first digits, so analysis should succeed.
    let result = analyzer.analyze(&amounts);
    assert!(
        result.is_ok(),
        "10 non-zero amounts should not panic and should produce a result"
    );
    let analysis = result.expect("already checked ok");
    assert_eq!(analysis.sample_size, 10);
    assert_eq!(analysis.degrees_of_freedom, 8);
}

#[test]
fn test_benford_zero_and_negative_filtered() {
    // Build a dataset with zeros, negatives, and valid amounts mixed in.
    // The analyzer should silently filter out zeros (no first digit) and
    // use the absolute value of negatives.
    let mut amounts: Vec<Decimal> = Vec::new();

    // Add zeros
    for _ in 0..20 {
        amounts.push(Decimal::ZERO);
    }
    // Add negative amounts
    for i in 1..=50 {
        amounts.push(Decimal::new(-(i * 100 + 37), 2));
    }
    // Add positive log-normal-ish amounts to reach sufficient sample size
    let positive = generate_lognormal_amounts(200, 77);
    amounts.extend(positive);

    let analyzer = BenfordAnalyzer::new(0.05);
    let result = analyzer
        .analyze(&amounts)
        .expect("should handle zeros and negatives gracefully");

    // Zeros should be excluded; negatives should be included (via abs)
    // 50 negatives + 200 positives = 250 valid first digits
    assert!(
        result.sample_size >= 200,
        "Sample size should be at least 200 (zeros excluded), got {}",
        result.sample_size
    );
    // Verify the analysis ran to completion with reasonable values
    assert!(result.p_value >= 0.0 && result.p_value <= 1.0);
    assert!(result.mad >= 0.0);
    assert!(result.chi_squared >= 0.0);
}
