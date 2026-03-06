//! Cross-field consistency evaluation.
//!
//! Validates consistency rules across related fields within records.

use crate::error::EvalResult;
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Results of consistency analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyAnalysis {
    /// Total records analyzed.
    pub total_records: usize,
    /// Per-rule results.
    pub rule_results: Vec<RuleResult>,
    /// Overall pass rate (0.0-1.0).
    pub pass_rate: f64,
    /// Total violations.
    pub total_violations: usize,
    /// Violations by rule type.
    pub violations_by_type: HashMap<String, usize>,
}

/// Result for a single consistency rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResult {
    /// Rule name.
    pub rule_name: String,
    /// Rule description.
    pub description: String,
    /// Number of records checked.
    pub records_checked: usize,
    /// Number of records passed.
    pub records_passed: usize,
    /// Pass rate for this rule.
    pub pass_rate: f64,
    /// Example violations.
    pub example_violations: Vec<String>,
}

/// A consistency rule definition.
#[derive(Debug, Clone)]
pub struct ConsistencyRule {
    /// Rule name.
    pub name: String,
    /// Rule description.
    pub description: String,
    /// Rule type.
    pub rule_type: RuleType,
}

/// Type of consistency rule.
pub enum RuleType {
    /// Date ordering (e.g., document_date <= posting_date).
    DateOrdering {
        earlier_field: String,
        later_field: String,
    },
    /// Mutual exclusion (e.g., debit XOR credit).
    MutualExclusion { field1: String, field2: String },
    /// Fiscal period matches date.
    FiscalPeriodDateAlignment {
        date_field: String,
        period_field: String,
        year_field: String,
    },
    /// Amount sign consistency.
    AmountSign {
        amount_field: String,
        indicator_field: String,
        positive_indicator: String,
    },
    /// Required if present (if field A has value, field B must too).
    RequiredIfPresent {
        trigger_field: String,
        required_field: String,
    },
    /// Value range (field must be within range).
    ValueRange {
        field: String,
        min: Option<Decimal>,
        max: Option<Decimal>,
    },
    /// Custom rule with closure.
    Custom {
        checker: Arc<dyn Fn(&ConsistencyRecord) -> bool + Send + Sync>,
    },
}

impl Clone for RuleType {
    fn clone(&self) -> Self {
        match self {
            RuleType::DateOrdering {
                earlier_field,
                later_field,
            } => RuleType::DateOrdering {
                earlier_field: earlier_field.clone(),
                later_field: later_field.clone(),
            },
            RuleType::MutualExclusion { field1, field2 } => RuleType::MutualExclusion {
                field1: field1.clone(),
                field2: field2.clone(),
            },
            RuleType::FiscalPeriodDateAlignment {
                date_field,
                period_field,
                year_field,
            } => RuleType::FiscalPeriodDateAlignment {
                date_field: date_field.clone(),
                period_field: period_field.clone(),
                year_field: year_field.clone(),
            },
            RuleType::AmountSign {
                amount_field,
                indicator_field,
                positive_indicator,
            } => RuleType::AmountSign {
                amount_field: amount_field.clone(),
                indicator_field: indicator_field.clone(),
                positive_indicator: positive_indicator.clone(),
            },
            RuleType::RequiredIfPresent {
                trigger_field,
                required_field,
            } => RuleType::RequiredIfPresent {
                trigger_field: trigger_field.clone(),
                required_field: required_field.clone(),
            },
            RuleType::ValueRange { field, min, max } => RuleType::ValueRange {
                field: field.clone(),
                min: *min,
                max: *max,
            },
            RuleType::Custom { checker } => RuleType::Custom {
                checker: Arc::clone(checker),
            },
        }
    }
}

impl std::fmt::Debug for RuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleType::DateOrdering {
                earlier_field,
                later_field,
            } => f
                .debug_struct("DateOrdering")
                .field("earlier_field", earlier_field)
                .field("later_field", later_field)
                .finish(),
            RuleType::MutualExclusion { field1, field2 } => f
                .debug_struct("MutualExclusion")
                .field("field1", field1)
                .field("field2", field2)
                .finish(),
            RuleType::FiscalPeriodDateAlignment {
                date_field,
                period_field,
                year_field,
            } => f
                .debug_struct("FiscalPeriodDateAlignment")
                .field("date_field", date_field)
                .field("period_field", period_field)
                .field("year_field", year_field)
                .finish(),
            RuleType::AmountSign {
                amount_field,
                indicator_field,
                positive_indicator,
            } => f
                .debug_struct("AmountSign")
                .field("amount_field", amount_field)
                .field("indicator_field", indicator_field)
                .field("positive_indicator", positive_indicator)
                .finish(),
            RuleType::RequiredIfPresent {
                trigger_field,
                required_field,
            } => f
                .debug_struct("RequiredIfPresent")
                .field("trigger_field", trigger_field)
                .field("required_field", required_field)
                .finish(),
            RuleType::ValueRange { field, min, max } => f
                .debug_struct("ValueRange")
                .field("field", field)
                .field("min", min)
                .field("max", max)
                .finish(),
            RuleType::Custom { .. } => f
                .debug_struct("Custom")
                .field("checker", &"<custom_fn>")
                .finish(),
        }
    }
}

/// A record for consistency checking.
#[derive(Debug, Clone, Default)]
pub struct ConsistencyRecord {
    /// String field values.
    pub string_fields: HashMap<String, String>,
    /// Decimal field values.
    pub decimal_fields: HashMap<String, Decimal>,
    /// Date field values.
    pub date_fields: HashMap<String, NaiveDate>,
    /// Integer field values.
    pub integer_fields: HashMap<String, i64>,
    /// Boolean field values.
    pub boolean_fields: HashMap<String, bool>,
}

/// Analyzer for cross-field consistency.
pub struct ConsistencyAnalyzer {
    /// Rules to check.
    rules: Vec<ConsistencyRule>,
    /// Maximum example violations to collect per rule.
    max_examples: usize,
}

impl ConsistencyAnalyzer {
    /// Create a new analyzer with specified rules.
    pub fn new(rules: Vec<ConsistencyRule>) -> Self {
        Self {
            rules,
            max_examples: 5,
        }
    }

    /// Create with default accounting rules.
    pub fn with_default_rules() -> Self {
        let rules = vec![
            ConsistencyRule {
                name: "date_ordering".to_string(),
                description: "Document date must be on or before posting date".to_string(),
                rule_type: RuleType::DateOrdering {
                    earlier_field: "document_date".to_string(),
                    later_field: "posting_date".to_string(),
                },
            },
            ConsistencyRule {
                name: "debit_credit_exclusion".to_string(),
                description: "Each line must have either debit or credit, not both".to_string(),
                rule_type: RuleType::MutualExclusion {
                    field1: "debit_amount".to_string(),
                    field2: "credit_amount".to_string(),
                },
            },
            ConsistencyRule {
                name: "fiscal_period_alignment".to_string(),
                description: "Fiscal period must match posting date".to_string(),
                rule_type: RuleType::FiscalPeriodDateAlignment {
                    date_field: "posting_date".to_string(),
                    period_field: "fiscal_period".to_string(),
                    year_field: "fiscal_year".to_string(),
                },
            },
        ];

        Self::new(rules)
    }

    /// Analyze consistency of records.
    pub fn analyze(&self, records: &[ConsistencyRecord]) -> EvalResult<ConsistencyAnalysis> {
        let total_records = records.len();
        let mut rule_results = Vec::new();
        let mut total_violations = 0;
        let mut violations_by_type: HashMap<String, usize> = HashMap::new();

        for rule in &self.rules {
            let mut records_checked = 0;
            let mut records_passed = 0;
            let mut example_violations = Vec::new();

            for (idx, record) in records.iter().enumerate() {
                let applicable = self.is_rule_applicable(rule, record);
                if !applicable {
                    continue;
                }

                records_checked += 1;
                let passed = self.check_rule(rule, record);

                if passed {
                    records_passed += 1;
                } else {
                    total_violations += 1;
                    *violations_by_type.entry(rule.name.clone()).or_insert(0) += 1;

                    if example_violations.len() < self.max_examples {
                        example_violations.push(format!("Record {idx}: {record:?}"));
                    }
                }
            }

            let pass_rate = if records_checked > 0 {
                records_passed as f64 / records_checked as f64
            } else {
                1.0
            };

            rule_results.push(RuleResult {
                rule_name: rule.name.clone(),
                description: rule.description.clone(),
                records_checked,
                records_passed,
                pass_rate,
                example_violations,
            });
        }

        let total_checked: usize = rule_results.iter().map(|r| r.records_checked).sum();
        let total_passed: usize = rule_results.iter().map(|r| r.records_passed).sum();
        let pass_rate = if total_checked > 0 {
            total_passed as f64 / total_checked as f64
        } else {
            1.0
        };

        Ok(ConsistencyAnalysis {
            total_records,
            rule_results,
            pass_rate,
            total_violations,
            violations_by_type,
        })
    }

    /// Check if rule is applicable to record (has required fields).
    fn is_rule_applicable(&self, rule: &ConsistencyRule, record: &ConsistencyRecord) -> bool {
        match &rule.rule_type {
            RuleType::DateOrdering {
                earlier_field,
                later_field,
            } => {
                record.date_fields.contains_key(earlier_field)
                    && record.date_fields.contains_key(later_field)
            }
            RuleType::MutualExclusion { field1, field2 } => {
                record.decimal_fields.contains_key(field1)
                    || record.decimal_fields.contains_key(field2)
            }
            RuleType::FiscalPeriodDateAlignment {
                date_field,
                period_field,
                year_field,
            } => {
                record.date_fields.contains_key(date_field)
                    && record.integer_fields.contains_key(period_field)
                    && record.integer_fields.contains_key(year_field)
            }
            RuleType::AmountSign {
                amount_field,
                indicator_field,
                ..
            } => {
                record.decimal_fields.contains_key(amount_field)
                    && record.string_fields.contains_key(indicator_field)
            }
            RuleType::RequiredIfPresent { trigger_field, .. } => {
                record.string_fields.contains_key(trigger_field)
                    || record.decimal_fields.contains_key(trigger_field)
            }
            RuleType::ValueRange { field, .. } => record.decimal_fields.contains_key(field),
            RuleType::Custom { .. } => true,
        }
    }

    /// Check if record passes rule.
    fn check_rule(&self, rule: &ConsistencyRule, record: &ConsistencyRecord) -> bool {
        match &rule.rule_type {
            RuleType::DateOrdering {
                earlier_field,
                later_field,
            } => {
                let earlier = record.date_fields.get(earlier_field);
                let later = record.date_fields.get(later_field);
                match (earlier, later) {
                    (Some(e), Some(l)) => e <= l,
                    _ => true,
                }
            }
            RuleType::MutualExclusion { field1, field2 } => {
                let val1 = record
                    .decimal_fields
                    .get(field1)
                    .map(|v| *v != Decimal::ZERO)
                    .unwrap_or(false);
                let val2 = record
                    .decimal_fields
                    .get(field2)
                    .map(|v| *v != Decimal::ZERO)
                    .unwrap_or(false);
                // XOR: exactly one should be non-zero
                val1 != val2
            }
            RuleType::FiscalPeriodDateAlignment {
                date_field,
                period_field,
                year_field,
            } => {
                let date = record.date_fields.get(date_field);
                let period = record.integer_fields.get(period_field);
                let year = record.integer_fields.get(year_field);

                match (date, period, year) {
                    (Some(d), Some(p), Some(y)) => d.month() as i64 == *p && d.year() as i64 == *y,
                    _ => true,
                }
            }
            RuleType::AmountSign {
                amount_field,
                indicator_field,
                positive_indicator,
            } => {
                let amount = record.decimal_fields.get(amount_field);
                let indicator = record.string_fields.get(indicator_field);

                match (amount, indicator) {
                    (Some(a), Some(i)) => {
                        let should_be_positive = i == positive_indicator;
                        let is_positive = *a >= Decimal::ZERO;
                        should_be_positive == is_positive
                    }
                    _ => true,
                }
            }
            RuleType::RequiredIfPresent {
                trigger_field,
                required_field,
            } => {
                let has_trigger = record.string_fields.contains_key(trigger_field)
                    || record.decimal_fields.contains_key(trigger_field);

                if !has_trigger {
                    return true;
                }

                record.string_fields.contains_key(required_field)
                    || record.decimal_fields.contains_key(required_field)
            }
            RuleType::ValueRange { field, min, max } => {
                let value = record.decimal_fields.get(field);
                match value {
                    Some(v) => {
                        let above_min = min.map(|m| *v >= m).unwrap_or(true);
                        let below_max = max.map(|m| *v <= m).unwrap_or(true);
                        above_min && below_max
                    }
                    None => true,
                }
            }
            RuleType::Custom { checker } => checker(record),
        }
    }
}

impl Default for ConsistencyAnalyzer {
    fn default() -> Self {
        Self::with_default_rules()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_date_ordering_pass() {
        let mut record = ConsistencyRecord::default();
        record.date_fields.insert(
            "document_date".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );
        record.date_fields.insert(
            "posting_date".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        let analyzer = ConsistencyAnalyzer::with_default_rules();
        let result = analyzer.analyze(&[record]).unwrap();

        let date_rule = result
            .rule_results
            .iter()
            .find(|r| r.rule_name == "date_ordering")
            .unwrap();
        assert_eq!(date_rule.pass_rate, 1.0);
    }

    #[test]
    fn test_date_ordering_fail() {
        let mut record = ConsistencyRecord::default();
        record.date_fields.insert(
            "document_date".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
        );
        record.date_fields.insert(
            "posting_date".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        let analyzer = ConsistencyAnalyzer::with_default_rules();
        let result = analyzer.analyze(&[record]).unwrap();

        let date_rule = result
            .rule_results
            .iter()
            .find(|r| r.rule_name == "date_ordering")
            .unwrap();
        assert_eq!(date_rule.pass_rate, 0.0);
    }

    #[test]
    fn test_mutual_exclusion() {
        let mut record = ConsistencyRecord::default();
        record
            .decimal_fields
            .insert("debit_amount".to_string(), Decimal::new(100, 0));
        record
            .decimal_fields
            .insert("credit_amount".to_string(), Decimal::ZERO);

        let analyzer = ConsistencyAnalyzer::with_default_rules();
        let result = analyzer.analyze(&[record]).unwrap();

        let excl_rule = result
            .rule_results
            .iter()
            .find(|r| r.rule_name == "debit_credit_exclusion")
            .unwrap();
        assert_eq!(excl_rule.pass_rate, 1.0);
    }

    #[test]
    fn test_mutual_exclusion_fail_both_nonzero() {
        let mut record = ConsistencyRecord::default();
        record
            .decimal_fields
            .insert("debit_amount".to_string(), Decimal::new(100, 0));
        record
            .decimal_fields
            .insert("credit_amount".to_string(), Decimal::new(50, 0));

        let analyzer = ConsistencyAnalyzer::with_default_rules();
        let result = analyzer.analyze(&[record]).unwrap();

        let excl_rule = result
            .rule_results
            .iter()
            .find(|r| r.rule_name == "debit_credit_exclusion")
            .unwrap();
        assert_eq!(excl_rule.pass_rate, 0.0);
    }
}
