//! Country pack coherence evaluator.
//!
//! Validates country pack configuration data including tax rate ranges,
//! approval level ordering, holiday multiplier ranges, IBAN lengths,
//! and fiscal year configuration.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// Thresholds for country pack evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryPackThresholds {
    /// Minimum fraction of valid tax rates.
    pub min_rate_validity: f64,
    /// Minimum fraction of valid format/config fields.
    pub min_format_validity: f64,
}

impl Default for CountryPackThresholds {
    fn default() -> Self {
        Self {
            min_rate_validity: 0.99,
            min_format_validity: 0.99,
        }
    }
}

/// Tax rate data for range validation.
#[derive(Debug, Clone)]
pub struct TaxRateData {
    /// Rate name/description.
    pub rate_name: String,
    /// Tax rate value.
    pub rate: f64,
}

/// Approval level data for ordering validation.
#[derive(Debug, Clone)]
pub struct ApprovalLevelData {
    /// Approval level number.
    pub level: u32,
    /// Threshold amount for this level.
    pub threshold: f64,
}

/// Holiday data for multiplier validation.
#[derive(Debug, Clone)]
pub struct HolidayData {
    /// Holiday name.
    pub name: String,
    /// Activity multiplier (0.0 = no activity, 1.0 = normal activity).
    pub activity_multiplier: f64,
}

/// Country pack data combining all validation inputs.
#[derive(Debug, Clone)]
pub struct CountryPackData {
    /// ISO country code.
    pub country_code: String,
    /// Tax rates defined for this country.
    pub tax_rates: Vec<TaxRateData>,
    /// Approval levels defined for this country.
    pub approval_levels: Vec<ApprovalLevelData>,
    /// Holidays defined for this country.
    pub holidays: Vec<HolidayData>,
    /// IBAN length (if applicable).
    pub iban_length: Option<u32>,
    /// Fiscal year start month (1-12).
    pub fiscal_year_start_month: Option<u32>,
}

/// Results of country pack coherence evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryPackEvaluation {
    /// Fraction of tax rates in [0.0, 1.0].
    pub tax_rate_validity: f64,
    /// Fraction of country packs with correctly ordered approval levels.
    pub approval_order_validity: f64,
    /// Fraction of holiday multipliers in [0.0, 1.0].
    pub holiday_multiplier_validity: f64,
    /// Fraction of IBAN lengths in valid range [15, 34].
    pub iban_length_validity: f64,
    /// Fraction of fiscal year months in [1, 12].
    pub fiscal_year_validity: f64,
    /// Total country packs evaluated.
    pub total_packs: usize,
    /// Total tax rates evaluated.
    pub total_tax_rates: usize,
    /// Total holidays evaluated.
    pub total_holidays: usize,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Evaluator for country pack coherence.
pub struct CountryPackEvaluator {
    thresholds: CountryPackThresholds,
}

impl CountryPackEvaluator {
    /// Create a new evaluator with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: CountryPackThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: CountryPackThresholds) -> Self {
        Self { thresholds }
    }

    /// Evaluate country pack data coherence.
    pub fn evaluate(&self, packs: &[CountryPackData]) -> EvalResult<CountryPackEvaluation> {
        let mut issues = Vec::new();

        // 1. Tax rates in [0.0, 1.0]
        let all_rates: Vec<&TaxRateData> = packs.iter().flat_map(|p| p.tax_rates.iter()).collect();
        let rate_ok = all_rates
            .iter()
            .filter(|r| (0.0..=1.0).contains(&r.rate))
            .count();
        let tax_rate_validity = if all_rates.is_empty() {
            1.0
        } else {
            rate_ok as f64 / all_rates.len() as f64
        };

        // 2. Approval levels in ascending threshold order
        let order_ok = packs
            .iter()
            .filter(|p| {
                if p.approval_levels.len() <= 1 {
                    return true;
                }
                let mut sorted = p.approval_levels.clone();
                sorted.sort_by_key(|a| a.level);
                sorted.windows(2).all(|w| w[0].threshold <= w[1].threshold)
            })
            .count();
        let approval_order_validity = if packs.is_empty() {
            1.0
        } else {
            order_ok as f64 / packs.len() as f64
        };

        // 3. Holiday multipliers in [0.0, 1.0]
        let all_holidays: Vec<&HolidayData> =
            packs.iter().flat_map(|p| p.holidays.iter()).collect();
        let holiday_ok = all_holidays
            .iter()
            .filter(|h| (0.0..=1.0).contains(&h.activity_multiplier))
            .count();
        let holiday_multiplier_validity = if all_holidays.is_empty() {
            1.0
        } else {
            holiday_ok as f64 / all_holidays.len() as f64
        };

        // 4. IBAN length in [15, 34]
        let ibans: Vec<u32> = packs.iter().filter_map(|p| p.iban_length).collect();
        let iban_ok = ibans.iter().filter(|&&l| (15..=34).contains(&l)).count();
        let iban_length_validity = if ibans.is_empty() {
            1.0
        } else {
            iban_ok as f64 / ibans.len() as f64
        };

        // 5. Fiscal year start month in [1, 12]
        let fy_months: Vec<u32> = packs
            .iter()
            .filter_map(|p| p.fiscal_year_start_month)
            .collect();
        let fy_ok = fy_months.iter().filter(|&&m| (1..=12).contains(&m)).count();
        let fiscal_year_validity = if fy_months.is_empty() {
            1.0
        } else {
            fy_ok as f64 / fy_months.len() as f64
        };

        // Check thresholds
        if tax_rate_validity < self.thresholds.min_rate_validity {
            issues.push(format!(
                "Tax rate validity {:.4} < {:.4}",
                tax_rate_validity, self.thresholds.min_rate_validity
            ));
        }
        if approval_order_validity < self.thresholds.min_format_validity {
            issues.push(format!(
                "Approval level ordering validity {:.4} < {:.4}",
                approval_order_validity, self.thresholds.min_format_validity
            ));
        }
        if holiday_multiplier_validity < self.thresholds.min_rate_validity {
            issues.push(format!(
                "Holiday multiplier validity {:.4} < {:.4}",
                holiday_multiplier_validity, self.thresholds.min_rate_validity
            ));
        }
        if iban_length_validity < self.thresholds.min_format_validity {
            issues.push(format!(
                "IBAN length validity {:.4} < {:.4}",
                iban_length_validity, self.thresholds.min_format_validity
            ));
        }
        if fiscal_year_validity < self.thresholds.min_format_validity {
            issues.push(format!(
                "Fiscal year month validity {:.4} < {:.4}",
                fiscal_year_validity, self.thresholds.min_format_validity
            ));
        }

        let passes = issues.is_empty();

        Ok(CountryPackEvaluation {
            tax_rate_validity,
            approval_order_validity,
            holiday_multiplier_validity,
            iban_length_validity,
            fiscal_year_validity,
            total_packs: packs.len(),
            total_tax_rates: all_rates.len(),
            total_holidays: all_holidays.len(),
            passes,
            issues,
        })
    }
}

impl Default for CountryPackEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_country_pack() {
        let evaluator = CountryPackEvaluator::new();
        let packs = vec![CountryPackData {
            country_code: "DE".to_string(),
            tax_rates: vec![
                TaxRateData {
                    rate_name: "standard_vat".to_string(),
                    rate: 0.19,
                },
                TaxRateData {
                    rate_name: "reduced_vat".to_string(),
                    rate: 0.07,
                },
            ],
            approval_levels: vec![
                ApprovalLevelData {
                    level: 1,
                    threshold: 1000.0,
                },
                ApprovalLevelData {
                    level: 2,
                    threshold: 5000.0,
                },
                ApprovalLevelData {
                    level: 3,
                    threshold: 25000.0,
                },
            ],
            holidays: vec![
                HolidayData {
                    name: "New Year".to_string(),
                    activity_multiplier: 0.0,
                },
                HolidayData {
                    name: "Christmas Eve".to_string(),
                    activity_multiplier: 0.3,
                },
            ],
            iban_length: Some(22),
            fiscal_year_start_month: Some(1),
        }];

        let result = evaluator.evaluate(&packs).unwrap();
        assert!(result.passes);
        assert_eq!(result.total_packs, 1);
        assert_eq!(result.total_tax_rates, 2);
        assert_eq!(result.total_holidays, 2);
    }

    #[test]
    fn test_invalid_tax_rate() {
        let evaluator = CountryPackEvaluator::new();
        let packs = vec![CountryPackData {
            country_code: "XX".to_string(),
            tax_rates: vec![TaxRateData {
                rate_name: "bad_rate".to_string(),
                rate: 1.5, // Invalid: > 1.0
            }],
            approval_levels: vec![],
            holidays: vec![],
            iban_length: None,
            fiscal_year_start_month: None,
        }];

        let result = evaluator.evaluate(&packs).unwrap();
        assert!(!result.passes);
        assert!(result.issues.iter().any(|i| i.contains("Tax rate")));
    }

    #[test]
    fn test_unordered_approval_levels() {
        let evaluator = CountryPackEvaluator::new();
        let packs = vec![CountryPackData {
            country_code: "XX".to_string(),
            tax_rates: vec![],
            approval_levels: vec![
                ApprovalLevelData {
                    level: 1,
                    threshold: 5000.0,
                },
                ApprovalLevelData {
                    level: 2,
                    threshold: 1000.0, // Wrong: lower than level 1
                },
            ],
            holidays: vec![],
            iban_length: None,
            fiscal_year_start_month: None,
        }];

        let result = evaluator.evaluate(&packs).unwrap();
        assert!(!result.passes);
        assert!(result.issues.iter().any(|i| i.contains("Approval level")));
    }

    #[test]
    fn test_invalid_iban_length() {
        let evaluator = CountryPackEvaluator::new();
        let packs = vec![CountryPackData {
            country_code: "XX".to_string(),
            tax_rates: vec![],
            approval_levels: vec![],
            holidays: vec![],
            iban_length: Some(10), // Invalid: < 15
            fiscal_year_start_month: None,
        }];

        let result = evaluator.evaluate(&packs).unwrap();
        assert!(!result.passes);
        assert!(result.issues.iter().any(|i| i.contains("IBAN length")));
    }

    #[test]
    fn test_invalid_fiscal_year_month() {
        let evaluator = CountryPackEvaluator::new();
        let packs = vec![CountryPackData {
            country_code: "XX".to_string(),
            tax_rates: vec![],
            approval_levels: vec![],
            holidays: vec![],
            iban_length: None,
            fiscal_year_start_month: Some(13), // Invalid: > 12
        }];

        let result = evaluator.evaluate(&packs).unwrap();
        assert!(!result.passes);
        assert!(result.issues.iter().any(|i| i.contains("Fiscal year")));
    }

    #[test]
    fn test_invalid_holiday_multiplier() {
        let evaluator = CountryPackEvaluator::new();
        let packs = vec![CountryPackData {
            country_code: "XX".to_string(),
            tax_rates: vec![],
            approval_levels: vec![],
            holidays: vec![HolidayData {
                name: "Bad Holiday".to_string(),
                activity_multiplier: 1.5, // Invalid: > 1.0
            }],
            iban_length: None,
            fiscal_year_start_month: None,
        }];

        let result = evaluator.evaluate(&packs).unwrap();
        assert!(!result.passes);
        assert!(result
            .issues
            .iter()
            .any(|i| i.contains("Holiday multiplier")));
    }

    #[test]
    fn test_empty_data() {
        let evaluator = CountryPackEvaluator::new();
        let result = evaluator.evaluate(&[]).unwrap();
        assert!(result.passes);
    }
}
