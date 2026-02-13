//! Scheme detectability evaluation.
//!
//! Validates that injected fraud schemes follow an expected difficulty ordering:
//! trivial > easy > moderate > hard > expert in terms of detection score,
//! and computes Spearman rank correlation.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single scheme record with difficulty and detection score.
#[derive(Debug, Clone)]
pub struct SchemeRecord {
    /// Unique identifier for this scheme instance.
    pub scheme_id: String,
    /// Difficulty level (e.g. "trivial", "easy", "moderate", "hard", "expert").
    pub difficulty: String,
    /// Detection score: probability that the scheme is detected (0.0-1.0).
    pub detection_score: f64,
}

/// Thresholds for scheme detectability analysis.
#[derive(Debug, Clone)]
pub struct SchemeDetectabilityThresholds {
    /// Minimum Spearman correlation for detectability ordering.
    pub min_detectability_score: f64,
}

impl Default for SchemeDetectabilityThresholds {
    fn default() -> Self {
        Self {
            min_detectability_score: 0.60,
        }
    }
}

/// Results of scheme detectability analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemeDetectabilityAnalysis {
    /// Whether difficulty ordering is monotonically valid.
    pub difficulty_ordering_valid: bool,
    /// Spearman rank correlation between difficulty ordinal and detection rate.
    pub detectability_score: f64,
    /// Mean detection rate per difficulty level.
    pub per_difficulty_rates: Vec<(String, f64)>,
    /// Total number of scheme records analyzed.
    pub total_schemes: usize,
    /// Whether the analysis passes all thresholds.
    pub passes: bool,
    /// Issues found during analysis.
    pub issues: Vec<String>,
}

/// Analyzer for scheme detectability.
pub struct SchemeDetectabilityAnalyzer {
    thresholds: SchemeDetectabilityThresholds,
}

impl SchemeDetectabilityAnalyzer {
    /// Canonical difficulty ordering (from most detectable to least).
    const DIFFICULTY_ORDER: &'static [&'static str] =
        &["trivial", "easy", "moderate", "hard", "expert"];

    /// Create a new analyzer with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: SchemeDetectabilityThresholds::default(),
        }
    }

    /// Create an analyzer with custom thresholds.
    pub fn with_thresholds(thresholds: SchemeDetectabilityThresholds) -> Self {
        Self { thresholds }
    }

    /// Analyze scheme detectability.
    pub fn analyze(&self, records: &[SchemeRecord]) -> EvalResult<SchemeDetectabilityAnalysis> {
        let mut issues = Vec::new();
        let total_schemes = records.len();

        if records.is_empty() {
            return Ok(SchemeDetectabilityAnalysis {
                difficulty_ordering_valid: true,
                detectability_score: 0.0,
                per_difficulty_rates: Vec::new(),
                total_schemes: 0,
                passes: true,
                issues: vec!["No scheme records provided".to_string()],
            });
        }

        // Group by difficulty and compute mean detection score
        let mut groups: HashMap<String, Vec<f64>> = HashMap::new();
        for record in records {
            groups
                .entry(record.difficulty.clone())
                .or_default()
                .push(record.detection_score);
        }

        let per_difficulty_rates: Vec<(String, f64)> = Self::DIFFICULTY_ORDER
            .iter()
            .filter_map(|&d| {
                groups.get(d).map(|scores| {
                    let mean = scores.iter().sum::<f64>() / scores.len() as f64;
                    (d.to_string(), mean)
                })
            })
            .collect();

        // Check monotonic ordering: trivial should have highest detection rate
        let difficulty_ordering_valid = self.check_monotonic(&per_difficulty_rates);
        if !difficulty_ordering_valid {
            issues.push("Difficulty ordering is not monotonically decreasing".to_string());
        }

        // Compute Spearman rank correlation
        let detectability_score = self.compute_spearman(records);

        if detectability_score < self.thresholds.min_detectability_score {
            issues.push(format!(
                "Detectability score {:.4} < {:.4} (threshold)",
                detectability_score, self.thresholds.min_detectability_score
            ));
        }

        let passes = issues.is_empty();

        Ok(SchemeDetectabilityAnalysis {
            difficulty_ordering_valid,
            detectability_score,
            per_difficulty_rates,
            total_schemes,
            passes,
            issues,
        })
    }

    /// Check if per-difficulty rates are monotonically decreasing.
    fn check_monotonic(&self, rates: &[(String, f64)]) -> bool {
        if rates.len() < 2 {
            return true;
        }

        for i in 1..rates.len() {
            if rates[i].1 > rates[i - 1].1 {
                return false;
            }
        }

        true
    }

    /// Compute Spearman rank correlation between difficulty ordinal and detection rate.
    ///
    /// Assigns ordinal ranks to difficulties (trivial=1, easy=2, etc.) and
    /// correlates with detection scores. A positive correlation means
    /// easier schemes have higher detection rates (expected).
    fn compute_spearman(&self, records: &[SchemeRecord]) -> f64 {
        let ordinal_map: HashMap<&str, f64> = Self::DIFFICULTY_ORDER
            .iter()
            .enumerate()
            .map(|(i, &d)| (d, (i + 1) as f64))
            .collect();

        // Filter to records with known difficulty levels
        let pairs: Vec<(f64, f64)> = records
            .iter()
            .filter_map(|r| {
                ordinal_map
                    .get(r.difficulty.as_str())
                    .map(|&ordinal| (ordinal, r.detection_score))
            })
            .collect();

        if pairs.len() < 3 {
            return 0.0;
        }

        // Rank both columns
        let ordinals: Vec<f64> = pairs.iter().map(|(o, _)| *o).collect();
        let scores: Vec<f64> = pairs.iter().map(|(_, s)| *s).collect();

        let ranked_ord = compute_ranks(&ordinals);
        let ranked_scores = compute_ranks(&scores);

        // Pearson correlation on ranks
        let n = pairs.len() as f64;
        let mean_o = ranked_ord.iter().sum::<f64>() / n;
        let mean_s = ranked_scores.iter().sum::<f64>() / n;

        let mut cov = 0.0;
        let mut var_o = 0.0;
        let mut var_s = 0.0;

        for i in 0..pairs.len() {
            let do_ = ranked_ord[i] - mean_o;
            let ds = ranked_scores[i] - mean_s;
            cov += do_ * ds;
            var_o += do_ * do_;
            var_s += ds * ds;
        }

        let denom = (var_o * var_s).sqrt();
        if denom < 1e-12 {
            return 0.0;
        }

        // We expect negative correlation (higher ordinal = harder = lower detection),
        // so we negate to get a positive "detectability score"
        let rho = cov / denom;
        (-rho).clamp(0.0, 1.0)
    }
}

/// Compute ranks for a vector of values (average ranks for ties).
fn compute_ranks(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    let mut indexed: Vec<(usize, f64)> = values.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut ranks = vec![0.0; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        // Find all tied values
        while j < n && (indexed[j].1 - indexed[i].1).abs() < 1e-12 {
            j += 1;
        }
        // Average rank for the tie group
        let avg_rank = (i + j + 1) as f64 / 2.0; // 1-based
        for k in i..j {
            ranks[indexed[k].0] = avg_rank;
        }
        i = j;
    }

    ranks
}

impl Default for SchemeDetectabilityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ordering() {
        let records = vec![
            SchemeRecord {
                scheme_id: "s1".into(),
                difficulty: "trivial".into(),
                detection_score: 0.95,
            },
            SchemeRecord {
                scheme_id: "s2".into(),
                difficulty: "easy".into(),
                detection_score: 0.80,
            },
            SchemeRecord {
                scheme_id: "s3".into(),
                difficulty: "moderate".into(),
                detection_score: 0.60,
            },
            SchemeRecord {
                scheme_id: "s4".into(),
                difficulty: "hard".into(),
                detection_score: 0.35,
            },
            SchemeRecord {
                scheme_id: "s5".into(),
                difficulty: "expert".into(),
                detection_score: 0.10,
            },
        ];

        let analyzer = SchemeDetectabilityAnalyzer::new();
        let result = analyzer.analyze(&records).unwrap();

        assert!(result.difficulty_ordering_valid);
        assert!(result.detectability_score > 0.6);
        assert!(result.passes);
    }

    #[test]
    fn test_invalid_ordering() {
        // Inverted: trivial has lowest detection, expert has highest
        let records = vec![
            SchemeRecord {
                scheme_id: "s1".into(),
                difficulty: "trivial".into(),
                detection_score: 0.10,
            },
            SchemeRecord {
                scheme_id: "s2".into(),
                difficulty: "easy".into(),
                detection_score: 0.30,
            },
            SchemeRecord {
                scheme_id: "s3".into(),
                difficulty: "moderate".into(),
                detection_score: 0.50,
            },
            SchemeRecord {
                scheme_id: "s4".into(),
                difficulty: "hard".into(),
                detection_score: 0.70,
            },
            SchemeRecord {
                scheme_id: "s5".into(),
                difficulty: "expert".into(),
                detection_score: 0.90,
            },
        ];

        let analyzer = SchemeDetectabilityAnalyzer::new();
        let result = analyzer.analyze(&records).unwrap();

        assert!(!result.difficulty_ordering_valid);
        assert!(!result.passes);
    }

    #[test]
    fn test_empty_schemes() {
        let analyzer = SchemeDetectabilityAnalyzer::new();
        let result = analyzer.analyze(&[]).unwrap();

        assert_eq!(result.total_schemes, 0);
        assert!(result.difficulty_ordering_valid);
    }
}
