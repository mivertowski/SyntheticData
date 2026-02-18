//! Shared generator utilities.

use rand::Rng;

/// Select from weighted options. Weights don't need to sum to 1.0.
pub fn weighted_select<'a, T, R: Rng>(rng: &mut R, options: &'a [(T, f64)]) -> &'a T {
    let total: f64 = options.iter().map(|(_, w)| w).sum();
    let mut roll = rng.gen::<f64>() * total;
    for (item, weight) in options {
        roll -= weight;
        if roll <= 0.0 {
            return item;
        }
    }
    &options
        .last()
        .expect("weighted_select called with empty options")
        .0
}

/// Sample a Decimal in a range using the RNG.
pub fn sample_decimal_range<R: Rng>(
    rng: &mut R,
    min: rust_decimal::Decimal,
    max: rust_decimal::Decimal,
) -> rust_decimal::Decimal {
    use rust_decimal::prelude::ToPrimitive;
    let min_f = min.to_f64().unwrap_or(0.0);
    let max_f = max.to_f64().unwrap_or(min_f + 1.0);
    let val = rng.gen_range(min_f..=max_f);
    rust_decimal::Decimal::from_f64_retain(val).unwrap_or(min)
}

/// Create a seeded RNG for a generator, with an optional discriminator for sub-generators.
pub fn seeded_rng(seed: u64, discriminator: u64) -> rand_chacha::ChaCha8Rng {
    use rand::SeedableRng;
    rand_chacha::ChaCha8Rng::seed_from_u64(seed.wrapping_add(discriminator))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_weighted_select_distribution() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let options = vec![("a", 0.9), ("b", 0.1)];
        let mut a_count = 0;
        for _ in 0..100 {
            if *weighted_select(&mut rng, &options) == "a" {
                a_count += 1;
            }
        }
        assert!(a_count > 70, "Expected ~90% 'a', got {}", a_count);
    }

    #[test]
    fn test_weighted_select_single_option() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let options = vec![("only", 1.0)];
        assert_eq!(*weighted_select(&mut rng, &options), "only");
    }

    #[test]
    fn test_sample_decimal_range() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let min = rust_decimal::Decimal::new(100, 0);
        let max = rust_decimal::Decimal::new(200, 0);
        for _ in 0..100 {
            let val = sample_decimal_range(&mut rng, min, max);
            assert!(
                val >= min && val <= max,
                "Value {} outside [{}, {}]",
                val,
                min,
                max
            );
        }
    }

    #[test]
    fn test_seeded_rng_deterministic() {
        let rng1 = seeded_rng(42, 100);
        let rng2 = seeded_rng(42, 100);
        // Same seed + discriminator should produce same state
        assert_eq!(format!("{:?}", rng1), format!("{:?}", rng2));
    }

    #[test]
    fn test_seeded_rng_different_discriminators() {
        let mut rng1 = seeded_rng(42, 0);
        let mut rng2 = seeded_rng(42, 1);
        let val1: f64 = rng1.gen();
        let val2: f64 = rng2.gen();
        assert_ne!(
            val1, val2,
            "Different discriminators should produce different values"
        );
    }
}
