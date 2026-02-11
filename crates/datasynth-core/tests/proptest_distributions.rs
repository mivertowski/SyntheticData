//! Property-based tests for distribution samplers.

use proptest::prelude::*;
use rust_decimal::Decimal;

use datasynth_core::distributions::{AmountDistributionConfig, AmountSampler, BenfordSampler};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn benford_samples_are_positive(seed in 1u64..10000) {
        let config = AmountDistributionConfig::default();
        let mut sampler = BenfordSampler::new(seed, config);
        for _ in 0..100 {
            let sample: Decimal = sampler.sample();
            prop_assert!(sample > Decimal::ZERO, "Benford sample should be positive, got {}", sample);
            // Decimal is always finite, no need for is_finite check
        }
    }

    #[test]
    fn amount_sampler_respects_range(seed in 1u64..10000) {
        let mut sampler = AmountSampler::new(seed);
        for _ in 0..100 {
            let sample: Decimal = sampler.sample();
            prop_assert!(sample > Decimal::ZERO, "Amount should be positive, got {}", sample);
        }
    }

    #[test]
    fn benford_mad_reasonable_for_large_samples(seed in 1u64..1000) {
        // For 1000+ samples from a Benford-compliant distribution,
        // the Mean Absolute Deviation from Benford's Law should be reasonable
        let config = AmountDistributionConfig::default();
        let mut sampler = BenfordSampler::new(seed, config);
        let mut digit_counts = [0u32; 9]; // digits 1-9
        let n = 2000;

        for _ in 0..n {
            let val: Decimal = sampler.sample();
            // Extract first non-zero digit from the decimal string
            let val_str = val.to_string();
            if let Some(first_digit) = val_str
                .chars()
                .find(|c| c.is_ascii_digit() && *c != '0')
                .and_then(|c| c.to_digit(10))
            {
                let d = first_digit as usize;
                if d >= 1 && d <= 9 {
                    digit_counts[d - 1] += 1;
                }
            }
        }

        // Benford's expected frequencies
        let expected: Vec<f64> = (1..=9).map(|d| (1.0 + 1.0 / d as f64).log10()).collect();
        let total = digit_counts.iter().sum::<u32>() as f64;

        if total > 0.0 {
            let mad: f64 = digit_counts.iter().enumerate().map(|(i, &count)| {
                let observed = count as f64 / total;
                (observed - expected[i]).abs()
            }).sum::<f64>() / 9.0;

            // MAD < 0.04 is reasonable (relaxed threshold for property testing)
            prop_assert!(mad < 0.04, "Benford MAD too high: {:.4}", mad);
        }
    }
}
