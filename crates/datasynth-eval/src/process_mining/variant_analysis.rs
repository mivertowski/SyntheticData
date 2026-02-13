//! Process variant distribution analysis.
//!
//! Validates that process variants have reasonable diversity and
//! are not all happy-path.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// Process variant data.
#[derive(Debug, Clone)]
pub struct VariantData {
    /// Variant identifier (typically the sequence of activities).
    pub variant_id: String,
    /// Number of cases following this variant.
    pub case_count: usize,
    /// Whether this is the happy/normal path.
    pub is_happy_path: bool,
}

/// Thresholds for variant analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantThresholds {
    /// Minimum entropy (Shannon entropy of variant distribution).
    pub min_entropy: f64,
    /// Maximum happy path concentration.
    pub max_happy_path_concentration: f64,
    /// Minimum number of distinct variants.
    pub min_variant_count: usize,
}

impl Default for VariantThresholds {
    fn default() -> Self {
        Self {
            min_entropy: 1.0,
            max_happy_path_concentration: 0.95,
            min_variant_count: 2,
        }
    }
}

/// Results of variant analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantAnalysis {
    /// Number of distinct variants.
    pub variant_count: usize,
    /// Total cases.
    pub total_cases: usize,
    /// Shannon entropy of variant distribution.
    pub variant_entropy: f64,
    /// Happy path concentration (fraction of cases on happy path).
    pub happy_path_concentration: f64,
    /// Top variant frequencies (variant_id, fraction).
    pub top_variants: Vec<(String, f64)>,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Analyzer for process variants.
pub struct VariantAnalyzer {
    thresholds: VariantThresholds,
}

impl VariantAnalyzer {
    /// Create a new analyzer with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: VariantThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: VariantThresholds) -> Self {
        Self { thresholds }
    }

    /// Analyze process variants.
    pub fn analyze(&self, variants: &[VariantData]) -> EvalResult<VariantAnalysis> {
        let mut issues = Vec::new();

        if variants.is_empty() {
            return Ok(VariantAnalysis {
                variant_count: 0,
                total_cases: 0,
                variant_entropy: 0.0,
                happy_path_concentration: 0.0,
                top_variants: Vec::new(),
                passes: true,
                issues: Vec::new(),
            });
        }

        let total_cases: usize = variants.iter().map(|v| v.case_count).sum();
        let variant_count = variants.len();

        // Shannon entropy
        let variant_entropy = if total_cases > 0 {
            let mut entropy = 0.0_f64;
            for v in variants {
                if v.case_count > 0 {
                    let p = v.case_count as f64 / total_cases as f64;
                    entropy -= p * p.ln();
                }
            }
            entropy
        } else {
            0.0
        };

        // Happy path concentration
        let happy_cases: usize = variants
            .iter()
            .filter(|v| v.is_happy_path)
            .map(|v| v.case_count)
            .sum();
        let happy_path_concentration = if total_cases > 0 {
            happy_cases as f64 / total_cases as f64
        } else {
            0.0
        };

        // Top variants
        let mut sorted: Vec<&VariantData> = variants.iter().collect();
        sorted.sort_by(|a, b| b.case_count.cmp(&a.case_count));
        let top_variants: Vec<(String, f64)> = sorted
            .iter()
            .take(5)
            .map(|v| {
                (
                    v.variant_id.clone(),
                    if total_cases > 0 {
                        v.case_count as f64 / total_cases as f64
                    } else {
                        0.0
                    },
                )
            })
            .collect();

        // Check thresholds
        if variant_count < self.thresholds.min_variant_count {
            issues.push(format!(
                "Only {} variants (minimum {})",
                variant_count, self.thresholds.min_variant_count
            ));
        }
        if variant_entropy < self.thresholds.min_entropy && variant_count > 1 {
            issues.push(format!(
                "Variant entropy {:.3} < {:.3}",
                variant_entropy, self.thresholds.min_entropy
            ));
        }
        if happy_path_concentration > self.thresholds.max_happy_path_concentration {
            issues.push(format!(
                "Happy path concentration {:.3} > {:.3}",
                happy_path_concentration, self.thresholds.max_happy_path_concentration
            ));
        }

        let passes = issues.is_empty();

        Ok(VariantAnalysis {
            variant_count,
            total_cases,
            variant_entropy,
            happy_path_concentration,
            top_variants,
            passes,
            issues,
        })
    }
}

impl Default for VariantAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_diverse_variants() {
        let analyzer = VariantAnalyzer::new();
        let variants = vec![
            VariantData {
                variant_id: "A->B->C".to_string(),
                case_count: 50,
                is_happy_path: true,
            },
            VariantData {
                variant_id: "A->B->D->C".to_string(),
                case_count: 30,
                is_happy_path: false,
            },
            VariantData {
                variant_id: "A->E->C".to_string(),
                case_count: 20,
                is_happy_path: false,
            },
        ];

        let result = analyzer.analyze(&variants).unwrap();
        assert!(result.passes);
        assert_eq!(result.variant_count, 3);
        assert!(result.variant_entropy > 0.0);
    }

    #[test]
    fn test_all_happy_path() {
        let analyzer = VariantAnalyzer::new();
        let variants = vec![
            VariantData {
                variant_id: "A->B->C".to_string(),
                case_count: 100,
                is_happy_path: true,
            },
            VariantData {
                variant_id: "A->B->D".to_string(),
                case_count: 1,
                is_happy_path: false,
            },
        ];

        let result = analyzer.analyze(&variants).unwrap();
        assert!(!result.passes);
        assert!(result.happy_path_concentration > 0.95);
    }

    #[test]
    fn test_single_variant() {
        let analyzer = VariantAnalyzer::new();
        let variants = vec![VariantData {
            variant_id: "A->B".to_string(),
            case_count: 100,
            is_happy_path: true,
        }];

        let result = analyzer.analyze(&variants).unwrap();
        assert!(!result.passes); // Too few variants
    }

    #[test]
    fn test_empty() {
        let analyzer = VariantAnalyzer::new();
        let result = analyzer.analyze(&[]).unwrap();
        assert!(result.passes);
    }
}
