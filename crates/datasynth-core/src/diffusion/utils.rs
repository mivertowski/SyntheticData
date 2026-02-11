use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal};

/// Add Gaussian noise to each element of x with given variance.
pub fn add_gaussian_noise(x: &[f64], variance: f64, rng: &mut ChaCha8Rng) -> Vec<f64> {
    let std_dev = variance.sqrt();
    if let Ok(normal) = Normal::new(0.0, std_dev) {
        x.iter().map(|&v| v + normal.sample(rng)).collect()
    } else {
        x.to_vec()
    }
}

/// Normalize features to zero mean and unit variance.
/// Returns (normalized_data, means, stds).
pub fn normalize_features(data: &[Vec<f64>]) -> (Vec<Vec<f64>>, Vec<f64>, Vec<f64>) {
    if data.is_empty() {
        return (vec![], vec![], vec![]);
    }

    let n_features = data[0].len();
    let n_samples = data.len() as f64;

    // Compute means
    let mut means = vec![0.0; n_features];
    for row in data {
        for (j, &val) in row.iter().enumerate() {
            if j < n_features {
                means[j] += val;
            }
        }
    }
    for m in &mut means {
        *m /= n_samples;
    }

    // Compute standard deviations
    let mut stds = vec![0.0; n_features];
    for row in data {
        for (j, &val) in row.iter().enumerate() {
            if j < n_features {
                stds[j] += (val - means[j]).powi(2);
            }
        }
    }
    for s in &mut stds {
        *s = (*s / n_samples).sqrt().max(1e-8); // Avoid division by zero
    }

    // Normalize
    let normalized: Vec<Vec<f64>> = data
        .iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(j, &val)| {
                    if j < n_features {
                        (val - means[j]) / stds[j]
                    } else {
                        val
                    }
                })
                .collect()
        })
        .collect();

    (normalized, means, stds)
}

/// Denormalize features back to original scale.
pub fn denormalize_features(data: &[Vec<f64>], means: &[f64], stds: &[f64]) -> Vec<Vec<f64>> {
    data.iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(j, &val)| {
                    if j < means.len() && j < stds.len() {
                        val * stds[j] + means[j]
                    } else {
                        val
                    }
                })
                .collect()
        })
        .collect()
}

/// Clip values to [min, max] range.
pub fn clip_values(data: &mut [Vec<f64>], min: f64, max: f64) {
    for row in data.iter_mut() {
        for val in row.iter_mut() {
            *val = val.clamp(min, max);
        }
    }
}

/// Generate standard normal noise matrix.
pub fn generate_noise(n_samples: usize, n_features: usize, seed: u64) -> Vec<Vec<f64>> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    if let Ok(normal) = Normal::new(0.0, 1.0) {
        (0..n_samples)
            .map(|_| (0..n_features).map(|_| normal.sample(&mut rng)).collect())
            .collect()
    } else {
        vec![vec![0.0; n_features]; n_samples]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_add_gaussian_noise() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let x = vec![1.0, 2.0, 3.0];
        let noised = add_gaussian_noise(&x, 0.01, &mut rng);
        assert_eq!(noised.len(), 3);
        // Noise should be small
        for (orig, noised) in x.iter().zip(noised.iter()) {
            assert!((orig - noised).abs() < 1.0);
        }
    }

    #[test]
    fn test_normalize_denormalize_roundtrip() {
        let data = vec![vec![10.0, 20.0], vec![12.0, 22.0], vec![14.0, 24.0]];
        let (normalized, means, stds) = normalize_features(&data);
        let recovered = denormalize_features(&normalized, &means, &stds);

        for (orig, rec) in data.iter().zip(recovered.iter()) {
            for (o, r) in orig.iter().zip(rec.iter()) {
                assert!((o - r).abs() < 1e-10, "Roundtrip failed: {} vs {}", o, r);
            }
        }
    }

    #[test]
    fn test_normalize_zero_mean() {
        let data = vec![vec![10.0, 20.0], vec![20.0, 40.0]];
        let (normalized, _, _) = normalize_features(&data);
        let mean: f64 = normalized.iter().map(|r| r[0]).sum::<f64>() / normalized.len() as f64;
        assert!(
            mean.abs() < 1e-10,
            "Normalized mean should be ~0, got {}",
            mean
        );
    }

    #[test]
    fn test_clip_values() {
        let mut data = vec![vec![-5.0, 10.0, 0.5]];
        clip_values(&mut data, 0.0, 1.0);
        assert_eq!(data[0], vec![0.0, 1.0, 0.5]);
    }

    #[test]
    fn test_generate_noise_shape() {
        let noise = generate_noise(100, 5, 42);
        assert_eq!(noise.len(), 100);
        assert_eq!(noise[0].len(), 5);
    }

    #[test]
    fn test_normalize_empty() {
        let (data, means, stds) = normalize_features(&[]);
        assert!(data.is_empty());
        assert!(means.is_empty());
        assert!(stds.is_empty());
    }
}
