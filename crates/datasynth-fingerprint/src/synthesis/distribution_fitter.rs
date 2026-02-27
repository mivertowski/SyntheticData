//! Distribution fitting utilities.

use crate::models::{DistributionParams, DistributionType, NumericStats};

/// Fit a distribution to observed statistics.
pub fn fit_to_stats(stats: &NumericStats) -> (DistributionType, DistributionParams) {
    // If already fitted, return existing
    if stats.distribution != DistributionType::Unknown {
        return (stats.distribution, stats.distribution_params.clone());
    }

    // Attempt to fit based on characteristics
    let mean = stats.mean;
    let std_dev = stats.std_dev;
    let min = stats.min;
    let max = stats.max;

    // Check for uniform distribution
    let range = max - min;
    if range > 0.0 {
        let expected_std_uniform = range / (12.0_f64).sqrt();
        if (std_dev - expected_std_uniform).abs() / expected_std_uniform < 0.15 {
            return (
                DistributionType::Uniform,
                DistributionParams::uniform(min, max),
            );
        }
    }

    // Check for exponential (mean ≈ std_dev for exponential)
    if mean > 0.0 && min >= 0.0 && (std_dev / mean - 1.0).abs() < 0.2 {
        return (
            DistributionType::Exponential,
            DistributionParams::exponential(1.0 / mean),
        );
    }

    // For positive data, prefer log-normal
    if min > 0.0 && mean > 0.0 {
        let log_values_mean = mean.ln();
        let cv = std_dev / mean; // Coefficient of variation
        let sigma = (1.0 + cv.powi(2)).ln().sqrt();
        let mu = log_values_mean - sigma.powi(2) / 2.0;

        return (
            DistributionType::LogNormal,
            DistributionParams::log_normal(mu, sigma),
        );
    }

    // Default to normal
    (
        DistributionType::Normal,
        DistributionParams::normal(mean, std_dev),
    )
}

/// Estimate log-normal parameters using method of moments.
pub fn estimate_lognormal_params(mean: f64, variance: f64) -> (f64, f64) {
    // mu = ln(mean^2 / sqrt(variance + mean^2))
    // sigma^2 = ln(1 + variance / mean^2)
    if mean <= 0.0 {
        return (0.0, 1.0);
    }

    let sigma_sq = (1.0 + variance / mean.powi(2)).ln();
    let mu = mean.ln() - sigma_sq / 2.0;

    (mu, sigma_sq.sqrt())
}

/// Estimate normal parameters (trivial).
pub fn estimate_normal_params(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 1.0);
    }

    let n = values.len() as f64;
    let mean: f64 = values.iter().sum::<f64>() / n;
    let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;

    (mean, variance.sqrt())
}

/// Goodness of fit test (simplified KS-like).
pub fn goodness_of_fit(
    observed: &[f64],
    dist_type: DistributionType,
    params: &DistributionParams,
) -> f64 {
    // Returns a score 0-1 where 1 is perfect fit
    if observed.is_empty() {
        return 0.0;
    }

    let mut sorted = observed.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    let n = sorted.len();
    let mut max_diff = 0.0;

    for (i, &x) in sorted.iter().enumerate() {
        let empirical_cdf = (i + 1) as f64 / n as f64;
        let theoretical_cdf = theoretical_cdf(x, dist_type, params);
        let diff = (empirical_cdf - theoretical_cdf).abs();
        if diff > max_diff {
            max_diff = diff;
        }
    }

    // Convert to 0-1 score (lower KS statistic = better fit)
    1.0 - max_diff.min(1.0)
}

/// Theoretical CDF for a distribution.
fn theoretical_cdf(x: f64, dist_type: DistributionType, params: &DistributionParams) -> f64 {
    match dist_type {
        DistributionType::Normal => {
            let mean = params.param1.unwrap_or(0.0);
            let std_dev = params.param2.unwrap_or(1.0);
            normal_cdf(x, mean, std_dev)
        }
        DistributionType::LogNormal => {
            if x <= 0.0 {
                return 0.0;
            }
            let mu = params.param1.unwrap_or(0.0);
            let sigma = params.param2.unwrap_or(1.0);
            normal_cdf(x.ln(), mu, sigma)
        }
        DistributionType::Uniform => {
            let a = params.param1.unwrap_or(0.0);
            let b = params.param2.unwrap_or(1.0);
            if x < a {
                0.0
            } else if x > b {
                1.0
            } else {
                (x - a) / (b - a)
            }
        }
        DistributionType::Exponential => {
            let rate = params.param1.unwrap_or(1.0);
            if x < 0.0 {
                0.0
            } else {
                1.0 - (-rate * x).exp()
            }
        }
        DistributionType::Gamma => {
            // Gamma CDF = regularized incomplete gamma function P(a, x/b)
            // param1 = shape (alpha/k), param2 = rate (beta) or scale (theta)
            // Convention: param1 = shape, param2 = rate (1/scale)
            let shape = params.param1.unwrap_or(1.0);
            let rate = params.param2.unwrap_or(1.0);
            if x <= 0.0 || shape <= 0.0 || rate <= 0.0 {
                0.0
            } else {
                regularized_gamma_p(shape, rate * x)
            }
        }
        DistributionType::Pareto => {
            // Pareto CDF = 1 - (x_m / x)^alpha for x >= x_m
            // param1 = x_m (scale/minimum), param2 = alpha (shape)
            let x_m = params.param1.unwrap_or(1.0);
            let alpha = params.param2.unwrap_or(1.0);
            if x < x_m {
                0.0
            } else {
                1.0 - (x_m / x).powf(alpha)
            }
        }
        DistributionType::PointMass => {
            // PointMass CDF: 0 for x < point, 1 for x >= point
            let point = params.param1.unwrap_or(0.0);
            if x < point {
                0.0
            } else {
                1.0
            }
        }
        DistributionType::Mixture => {
            // Mixture CDF = weighted sum of component CDFs
            if let Some(ref components) = params.mixture_components {
                let mut cdf_val = 0.0;
                for comp in components {
                    cdf_val += comp.weight * theoretical_cdf(x, comp.distribution, &comp.params);
                }
                cdf_val
            } else {
                0.5
            }
        }
        _ => 0.5, // Empirical/Unknown: no parametric CDF available
    }
}

/// Normal CDF approximation.
fn normal_cdf(x: f64, mean: f64, std_dev: f64) -> f64 {
    if std_dev == 0.0 {
        return if x >= mean { 1.0 } else { 0.0 };
    }

    let z = (x - mean) / std_dev;
    0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2))
}

/// Error function approximation.
fn erf(x: f64) -> f64 {
    // Approximation from Abramowitz and Stegun
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

/// Natural logarithm of the Gamma function using the Lanczos approximation.
///
/// Uses g=7 with 9 coefficients, accurate to ~15 significant digits.
fn ln_gamma(x: f64) -> f64 {
    // For negative non-integer x, use the reflection formula
    if x < 0.5 {
        // Reflection formula: Gamma(x) * Gamma(1-x) = pi / sin(pi*x)
        let reflected = ln_gamma(1.0 - x);
        return (std::f64::consts::PI / (std::f64::consts::PI * x).sin()).ln() - reflected;
    }

    // Lanczos coefficients for g=7, n=9
    const COEFFICIENTS: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_9,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    const G: f64 = 7.0;

    let x = x - 1.0;
    let mut sum = COEFFICIENTS[0];
    for (i, &coeff) in COEFFICIENTS.iter().enumerate().skip(1) {
        sum += coeff / (x + i as f64);
    }

    let t = x + G + 0.5;
    0.5 * (2.0 * std::f64::consts::PI).ln() + (t.ln() * (x + 0.5)) - t + sum.ln()
}

/// Regularized lower incomplete gamma function P(a, x) using series expansion.
///
/// P(a, x) = gamma(a, x) / Gamma(a)
///
/// For small x relative to a, uses the series:
///   P(a, x) = e^{-x} * x^a * sum_{n=0}^{inf} x^n / (a * (a+1) * ... * (a+n))
///
/// For large x relative to a, uses Q(a, x) = 1 - P(a, x) via continued fraction.
fn regularized_gamma_p(a: f64, x: f64) -> f64 {
    if x < 0.0 {
        return 0.0;
    }
    if x == 0.0 {
        return 0.0;
    }
    if a <= 0.0 {
        return 1.0;
    }

    // Use continued fraction for large x (more stable)
    if x > a + 1.0 {
        return 1.0 - regularized_gamma_q_cf(a, x);
    }

    // Series expansion for P(a, x)
    regularized_gamma_p_series(a, x)
}

/// Series expansion for the regularized lower incomplete gamma function P(a, x).
///
/// P(a, x) = e^{-x} * x^a / Gamma(a) * sum_{n=0}^{inf} x^n / (a*(a+1)*...*(a+n))
fn regularized_gamma_p_series(a: f64, x: f64) -> f64 {
    let max_iterations = 200;
    let epsilon = 1e-14;

    let ln_prefix = a * x.ln() - x - ln_gamma(a);

    // Guard against overflow/underflow
    if ln_prefix < -700.0 {
        return 0.0;
    }

    let prefix = ln_prefix.exp();

    let mut sum = 1.0 / a;
    let mut term = 1.0 / a;

    for n in 1..max_iterations {
        term *= x / (a + n as f64);
        sum += term;
        if term.abs() < epsilon * sum.abs() {
            break;
        }
    }

    (prefix * sum).clamp(0.0, 1.0)
}

/// Regularized upper incomplete gamma function Q(a, x) using modified Lentz's
/// continued fraction algorithm.
///
/// Q(a, x) = 1 - P(a, x) = e^{-x} * x^a / Gamma(a) * CF
///
/// The continued fraction is:
///   CF = 1 / (x + 1 - a + K)
/// where K is the continued fraction with terms:
///   a_n = n * (a - n), b_n = x + 2n + 1 - a
fn regularized_gamma_q_cf(a: f64, x: f64) -> f64 {
    let max_iterations = 200;
    let epsilon = 1e-14;
    let tiny = 1e-30;

    let ln_prefix = a * x.ln() - x - ln_gamma(a);

    // Guard against overflow/underflow
    if ln_prefix < -700.0 {
        return 1.0; // Q(a,x) -> 1 when prefix is tiny (x very small)
    }

    let prefix = ln_prefix.exp();

    // Modified Lentz's algorithm for the continued fraction
    // CF = 1/(b_0+) a_1/(b_1+) a_2/(b_2+) ...
    // where b_0 = x + 1 - a, and for n >= 1:
    //   a_n = n * (a - n)
    //   b_n = x + 2n + 1 - a

    let b0 = x + 1.0 - a;
    let mut f = if b0.abs() < tiny { tiny } else { b0 };
    let mut c = f;
    let mut d = 0.0;

    for n in 1..max_iterations {
        let an = n as f64 * (a - n as f64);
        let bn = x + 2.0 * n as f64 + 1.0 - a;

        d = bn + an * d;
        if d.abs() < tiny {
            d = tiny;
        }
        d = 1.0 / d;

        c = bn + an / c;
        if c.abs() < tiny {
            c = tiny;
        }

        let delta = c * d;
        f *= delta;

        if (delta - 1.0).abs() < epsilon {
            break;
        }
    }

    let result = prefix / f;
    result.clamp(0.0, 1.0)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::MixtureComponent;

    #[test]
    fn test_estimate_lognormal() {
        let (mu, sigma) = estimate_lognormal_params(100.0, 2500.0);
        assert!(mu > 0.0);
        assert!(sigma > 0.0);
    }

    #[test]
    fn test_normal_cdf() {
        assert!((normal_cdf(0.0, 0.0, 1.0) - 0.5).abs() < 0.01);
        assert!(normal_cdf(3.0, 0.0, 1.0) > 0.99);
        assert!(normal_cdf(-3.0, 0.0, 1.0) < 0.01);
    }

    // --- ln_gamma tests ---

    #[test]
    fn test_ln_gamma_known_values() {
        // Gamma(1) = 1, ln(1) = 0
        assert!(
            ln_gamma(1.0).abs() < 1e-10,
            "ln_gamma(1) should be 0, got {}",
            ln_gamma(1.0)
        );

        // Gamma(2) = 1, ln(1) = 0
        assert!(
            ln_gamma(2.0).abs() < 1e-10,
            "ln_gamma(2) should be 0, got {}",
            ln_gamma(2.0)
        );

        // Gamma(3) = 2, ln(2) ~ 0.6931
        let expected = 2.0_f64.ln();
        assert!(
            (ln_gamma(3.0) - expected).abs() < 1e-8,
            "ln_gamma(3) should be {}, got {}",
            expected,
            ln_gamma(3.0)
        );

        // Gamma(5) = 24, ln(24) ~ 3.1781
        let expected = 24.0_f64.ln();
        assert!(
            (ln_gamma(5.0) - expected).abs() < 1e-8,
            "ln_gamma(5) should be {}, got {}",
            expected,
            ln_gamma(5.0)
        );

        // Gamma(0.5) = sqrt(pi), ln(sqrt(pi)) ~ 0.5724
        let expected = (std::f64::consts::PI).sqrt().ln();
        assert!(
            (ln_gamma(0.5) - expected).abs() < 1e-8,
            "ln_gamma(0.5) should be {}, got {}",
            expected,
            ln_gamma(0.5)
        );
    }

    #[test]
    fn test_ln_gamma_large_values() {
        // Gamma(10) = 362880
        let expected = 362880.0_f64.ln();
        assert!(
            (ln_gamma(10.0) - expected).abs() < 1e-6,
            "ln_gamma(10) should be ~{}, got {}",
            expected,
            ln_gamma(10.0)
        );
    }

    // --- Gamma CDF tests ---

    #[test]
    fn test_gamma_cdf_exponential_special_case() {
        // Gamma(1, rate) = Exponential(rate)
        // CDF = 1 - exp(-rate * x)
        let params = DistributionParams {
            param1: Some(1.0), // shape = 1
            param2: Some(2.0), // rate = 2
            ..DistributionParams::empty()
        };

        // At x=0, CDF should be 0
        let cdf0 = theoretical_cdf(0.0, DistributionType::Gamma, &params);
        assert!(
            cdf0.abs() < 1e-10,
            "Gamma CDF at 0 should be 0, got {}",
            cdf0
        );

        // At x=0.5, exp CDF = 1 - exp(-2*0.5) = 1 - exp(-1) ~ 0.6321
        let expected = 1.0 - (-1.0_f64).exp();
        let cdf_half = theoretical_cdf(0.5, DistributionType::Gamma, &params);
        assert!(
            (cdf_half - expected).abs() < 0.01,
            "Gamma(1,2) CDF at 0.5 should be ~{}, got {}",
            expected,
            cdf_half
        );

        // At large x, CDF should approach 1
        let cdf_large = theoretical_cdf(10.0, DistributionType::Gamma, &params);
        assert!(
            cdf_large > 0.99,
            "Gamma CDF at large x should be ~1, got {}",
            cdf_large
        );
    }

    #[test]
    fn test_gamma_cdf_shape_2() {
        // Gamma(2, 1): CDF at x = 1 - (1 + x)*exp(-x)
        let params = DistributionParams {
            param1: Some(2.0), // shape = 2
            param2: Some(1.0), // rate = 1
            ..DistributionParams::empty()
        };

        // At x=1: CDF = 1 - 2*exp(-1) ~ 0.2642
        let expected = 1.0 - 2.0 * (-1.0_f64).exp();
        let cdf_val = theoretical_cdf(1.0, DistributionType::Gamma, &params);
        assert!(
            (cdf_val - expected).abs() < 0.01,
            "Gamma(2,1) CDF at 1.0 should be ~{}, got {}",
            expected,
            cdf_val
        );

        // At x=3: CDF = 1 - 4*exp(-3) ~ 0.8009
        let expected3 = 1.0 - 4.0 * (-3.0_f64).exp();
        let cdf_val3 = theoretical_cdf(3.0, DistributionType::Gamma, &params);
        assert!(
            (cdf_val3 - expected3).abs() < 0.01,
            "Gamma(2,1) CDF at 3.0 should be ~{}, got {}",
            expected3,
            cdf_val3
        );
    }

    #[test]
    fn test_gamma_cdf_negative_x() {
        let params = DistributionParams {
            param1: Some(2.0),
            param2: Some(1.0),
            ..DistributionParams::empty()
        };
        let cdf_neg = theoretical_cdf(-1.0, DistributionType::Gamma, &params);
        assert!(
            cdf_neg.abs() < 1e-10,
            "Gamma CDF for negative x should be 0, got {}",
            cdf_neg
        );
    }

    #[test]
    fn test_gamma_cdf_monotonically_increasing() {
        let params = DistributionParams {
            param1: Some(3.0),
            param2: Some(0.5),
            ..DistributionParams::empty()
        };
        let mut prev = 0.0;
        for i in 0..=20 {
            let x = i as f64 * 0.5;
            let cdf_val = theoretical_cdf(x, DistributionType::Gamma, &params);
            assert!(
                cdf_val >= prev - 1e-10,
                "Gamma CDF should be monotonically increasing: at x={}, cdf={} < prev={}",
                x,
                cdf_val,
                prev
            );
            prev = cdf_val;
        }
    }

    // --- Pareto CDF tests ---

    #[test]
    fn test_pareto_cdf_basic() {
        // Pareto CDF = 1 - (x_m/x)^alpha for x >= x_m
        let params = DistributionParams {
            param1: Some(1.0), // x_m = 1
            param2: Some(2.0), // alpha = 2
            ..DistributionParams::empty()
        };

        // At x < x_m, CDF should be 0
        let cdf_below = theoretical_cdf(0.5, DistributionType::Pareto, &params);
        assert!(
            cdf_below.abs() < 1e-10,
            "Pareto CDF below x_m should be 0, got {}",
            cdf_below
        );

        // At x = x_m, CDF should be 0
        let cdf_at = theoretical_cdf(1.0, DistributionType::Pareto, &params);
        assert!(
            cdf_at.abs() < 1e-10,
            "Pareto CDF at x_m should be 0, got {}",
            cdf_at
        );

        // At x = 2, CDF = 1 - (1/2)^2 = 0.75
        let cdf_2 = theoretical_cdf(2.0, DistributionType::Pareto, &params);
        assert!(
            (cdf_2 - 0.75).abs() < 1e-10,
            "Pareto(1,2) CDF at 2.0 should be 0.75, got {}",
            cdf_2
        );

        // At x = 4, CDF = 1 - (1/4)^2 = 0.9375
        let cdf_4 = theoretical_cdf(4.0, DistributionType::Pareto, &params);
        assert!(
            (cdf_4 - 0.9375).abs() < 1e-10,
            "Pareto(1,2) CDF at 4.0 should be 0.9375, got {}",
            cdf_4
        );
    }

    #[test]
    fn test_pareto_cdf_different_params() {
        // x_m = 5, alpha = 3
        let params = DistributionParams {
            param1: Some(5.0),
            param2: Some(3.0),
            ..DistributionParams::empty()
        };

        // At x = 10, CDF = 1 - (5/10)^3 = 1 - 0.125 = 0.875
        let cdf_10 = theoretical_cdf(10.0, DistributionType::Pareto, &params);
        assert!(
            (cdf_10 - 0.875).abs() < 1e-10,
            "Pareto(5,3) CDF at 10.0 should be 0.875, got {}",
            cdf_10
        );
    }

    #[test]
    fn test_pareto_cdf_monotonically_increasing() {
        let params = DistributionParams {
            param1: Some(1.0),
            param2: Some(1.5),
            ..DistributionParams::empty()
        };
        let mut prev = 0.0;
        for i in 1..=20 {
            let x = i as f64;
            let cdf_val = theoretical_cdf(x, DistributionType::Pareto, &params);
            assert!(
                cdf_val >= prev - 1e-10,
                "Pareto CDF should be monotonically increasing"
            );
            prev = cdf_val;
        }
    }

    // --- PointMass CDF tests ---

    #[test]
    fn test_point_mass_cdf() {
        let params = DistributionParams {
            param1: Some(5.0), // point mass at 5
            ..DistributionParams::empty()
        };

        // Below the point: CDF = 0
        let cdf_below = theoretical_cdf(4.99, DistributionType::PointMass, &params);
        assert!(
            cdf_below.abs() < 1e-10,
            "PointMass CDF below point should be 0, got {}",
            cdf_below
        );

        // At the point: CDF = 1
        let cdf_at = theoretical_cdf(5.0, DistributionType::PointMass, &params);
        assert!(
            (cdf_at - 1.0).abs() < 1e-10,
            "PointMass CDF at point should be 1, got {}",
            cdf_at
        );

        // Above the point: CDF = 1
        let cdf_above = theoretical_cdf(5.01, DistributionType::PointMass, &params);
        assert!(
            (cdf_above - 1.0).abs() < 1e-10,
            "PointMass CDF above point should be 1, got {}",
            cdf_above
        );
    }

    #[test]
    fn test_point_mass_cdf_at_zero() {
        let params = DistributionParams {
            param1: Some(0.0),
            ..DistributionParams::empty()
        };

        let cdf_neg = theoretical_cdf(-0.001, DistributionType::PointMass, &params);
        assert!(cdf_neg.abs() < 1e-10);

        let cdf_zero = theoretical_cdf(0.0, DistributionType::PointMass, &params);
        assert!((cdf_zero - 1.0).abs() < 1e-10);
    }

    // --- Mixture CDF tests ---

    #[test]
    fn test_mixture_cdf_single_component() {
        // Mixture with a single Normal(0,1) component should equal Normal CDF
        let params = DistributionParams {
            mixture_components: Some(vec![MixtureComponent {
                weight: 1.0,
                distribution: DistributionType::Normal,
                params: DistributionParams::normal(0.0, 1.0),
            }]),
            ..DistributionParams::empty()
        };

        let mix_cdf = theoretical_cdf(0.0, DistributionType::Mixture, &params);
        let normal_cdf_val = normal_cdf(0.0, 0.0, 1.0);
        assert!(
            (mix_cdf - normal_cdf_val).abs() < 1e-10,
            "Single-component mixture should equal component CDF: {} vs {}",
            mix_cdf,
            normal_cdf_val
        );
    }

    #[test]
    fn test_mixture_cdf_two_components() {
        // 50/50 mixture of Normal(0,1) and Normal(5,1)
        let params = DistributionParams {
            mixture_components: Some(vec![
                MixtureComponent {
                    weight: 0.5,
                    distribution: DistributionType::Normal,
                    params: DistributionParams::normal(0.0, 1.0),
                },
                MixtureComponent {
                    weight: 0.5,
                    distribution: DistributionType::Normal,
                    params: DistributionParams::normal(5.0, 1.0),
                },
            ]),
            ..DistributionParams::empty()
        };

        // At x=0: 0.5 * Phi(0) + 0.5 * Phi(-5) ~ 0.5 * 0.5 + 0.5 * ~0 ~ 0.25
        let mix_cdf_0 = theoretical_cdf(0.0, DistributionType::Mixture, &params);
        assert!(
            (mix_cdf_0 - 0.25).abs() < 0.01,
            "Mixture CDF at 0 should be ~0.25, got {}",
            mix_cdf_0
        );

        // At x=5: 0.5 * Phi(5) + 0.5 * Phi(0) ~ 0.5 * ~1 + 0.5 * 0.5 ~ 0.75
        let mix_cdf_5 = theoretical_cdf(5.0, DistributionType::Mixture, &params);
        assert!(
            (mix_cdf_5 - 0.75).abs() < 0.01,
            "Mixture CDF at 5 should be ~0.75, got {}",
            mix_cdf_5
        );
    }

    #[test]
    fn test_mixture_cdf_monotonically_increasing() {
        let params = DistributionParams {
            mixture_components: Some(vec![
                MixtureComponent {
                    weight: 0.3,
                    distribution: DistributionType::Normal,
                    params: DistributionParams::normal(-2.0, 1.0),
                },
                MixtureComponent {
                    weight: 0.7,
                    distribution: DistributionType::Normal,
                    params: DistributionParams::normal(3.0, 2.0),
                },
            ]),
            ..DistributionParams::empty()
        };

        let mut prev = 0.0;
        for i in -50..=50 {
            let x = i as f64 * 0.2;
            let cdf_val = theoretical_cdf(x, DistributionType::Mixture, &params);
            assert!(
                cdf_val >= prev - 1e-10,
                "Mixture CDF should be monotonically increasing: at x={}, cdf={} < prev={}",
                x,
                cdf_val,
                prev
            );
            prev = cdf_val;
        }
    }

    #[test]
    fn test_mixture_cdf_no_components_fallback() {
        // If no mixture components are provided, should return 0.5
        let params = DistributionParams::empty();
        let cdf_val = theoretical_cdf(0.0, DistributionType::Mixture, &params);
        assert!(
            (cdf_val - 0.5).abs() < 1e-10,
            "Mixture with no components should return 0.5, got {}",
            cdf_val
        );
    }

    // --- Goodness of fit with new CDFs ---

    #[test]
    fn test_goodness_of_fit_gamma() {
        // Generate some data roughly from Gamma(2, 1) distribution
        // Using the fact that Gamma(2,1) has mean=2, variance=2
        let data: Vec<f64> = (1..=100).map(|i| (i as f64) * 0.1).collect();
        let params = DistributionParams {
            param1: Some(2.0),
            param2: Some(1.0),
            ..DistributionParams::empty()
        };
        let score = goodness_of_fit(&data, DistributionType::Gamma, &params);
        // Just verify it returns a valid score
        assert!(
            (0.0..=1.0).contains(&score),
            "Score should be in [0,1], got {}",
            score
        );
    }

    #[test]
    fn test_goodness_of_fit_pareto() {
        // Data that roughly follows Pareto(1, 2)
        let data: Vec<f64> = (1..=50).map(|i| 1.0 + i as f64 * 0.1).collect();
        let params = DistributionParams {
            param1: Some(1.0),
            param2: Some(2.0),
            ..DistributionParams::empty()
        };
        let score = goodness_of_fit(&data, DistributionType::Pareto, &params);
        assert!(
            (0.0..=1.0).contains(&score),
            "Score should be in [0,1], got {}",
            score
        );
    }

    // --- Regularized gamma function tests ---

    #[test]
    fn test_regularized_gamma_p_known_values() {
        // P(1, x) = 1 - exp(-x) (because Gamma(1,1) = Exponential(1))
        let p_1_1 = regularized_gamma_p(1.0, 1.0);
        let expected = 1.0 - (-1.0_f64).exp();
        assert!(
            (p_1_1 - expected).abs() < 1e-8,
            "P(1,1) should be ~{}, got {}",
            expected,
            p_1_1
        );

        // P(1, 0) = 0
        let p_1_0 = regularized_gamma_p(1.0, 0.0);
        assert!(p_1_0.abs() < 1e-10, "P(1,0) should be 0, got {}", p_1_0);
    }

    #[test]
    fn test_regularized_gamma_p_large_x() {
        // For large x, P(a, x) should approach 1
        let p = regularized_gamma_p(2.0, 50.0);
        assert!((p - 1.0).abs() < 1e-6, "P(2, 50) should be ~1.0, got {}", p);
    }

    #[test]
    fn test_regularized_gamma_p_bounds() {
        // P should always be in [0, 1]
        for &a in &[0.5, 1.0, 2.0, 5.0, 10.0] {
            for &x in &[0.0, 0.1, 1.0, 5.0, 20.0, 100.0] {
                let p = regularized_gamma_p(a, x);
                assert!(
                    (0.0..=1.0).contains(&p),
                    "P({}, {}) = {} out of bounds",
                    a,
                    x,
                    p
                );
            }
        }
    }

    #[test]
    fn test_regularized_gamma_p_monotonic_in_x() {
        // P(a, x) should be monotonically increasing in x for fixed a
        let a = 3.0;
        let mut prev = 0.0;
        for i in 0..=50 {
            let x = i as f64 * 0.5;
            let p = regularized_gamma_p(a, x);
            assert!(
                p >= prev - 1e-10,
                "P({}, {}) = {} should be >= prev {}",
                a,
                x,
                p,
                prev
            );
            prev = p;
        }
    }
}
