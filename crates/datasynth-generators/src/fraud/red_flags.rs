//! Red flag generation with correlation probabilities.
//!
//! This module generates fraud indicators (red flags) with appropriate
//! correlation probabilities for both fraudulent and legitimate transactions.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Strength of a red flag indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RedFlagStrength {
    /// Strong correlation with fraud (>60% fraud probability).
    Strong,
    /// Moderate correlation (30-60% fraud probability).
    Moderate,
    /// Weak correlation (<30% fraud probability).
    Weak,
}

impl RedFlagStrength {
    /// Returns the fraud probability range for this strength.
    pub fn fraud_probability_range(&self) -> (f64, f64) {
        match self {
            RedFlagStrength::Strong => (0.60, 0.90),
            RedFlagStrength::Moderate => (0.30, 0.60),
            RedFlagStrength::Weak => (0.10, 0.30),
        }
    }
}

/// Category of red flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RedFlagCategory {
    /// Vendor-related flags.
    Vendor,
    /// Transaction pattern flags.
    Transaction,
    /// Employee behavior flags.
    Employee,
    /// Document-related flags.
    Document,
    /// Timing-related flags.
    Timing,
    /// Account-related flags.
    Account,
}

/// A red flag pattern definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedFlagPattern {
    /// Unique name of the pattern.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Category of the flag.
    pub category: RedFlagCategory,
    /// Strength of the flag.
    pub strength: RedFlagStrength,
    /// Base probability that this flag indicates fraud.
    pub fraud_probability: f64,
    /// Probability of flag appearing when fraud is present: P(flag | fraud).
    pub inject_with_fraud: f64,
    /// Probability of flag appearing in legitimate transactions: P(flag | not fraud).
    pub inject_without_fraud: f64,
    /// Detection methods effective for this flag.
    pub detection_methods: Vec<String>,
    /// Related fraud schemes.
    pub related_schemes: Vec<String>,
}

impl RedFlagPattern {
    /// Creates a new red flag pattern.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        category: RedFlagCategory,
        strength: RedFlagStrength,
        fraud_probability: f64,
        inject_with_fraud: f64,
        inject_without_fraud: f64,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            category,
            strength,
            fraud_probability,
            inject_with_fraud,
            inject_without_fraud,
            detection_methods: Vec::new(),
            related_schemes: Vec::new(),
        }
    }

    /// Adds detection methods.
    pub fn with_detection_methods(mut self, methods: Vec<impl Into<String>>) -> Self {
        self.detection_methods = methods.into_iter().map(Into::into).collect();
        self
    }

    /// Adds related fraud schemes.
    pub fn with_related_schemes(mut self, schemes: Vec<impl Into<String>>) -> Self {
        self.related_schemes = schemes.into_iter().map(Into::into).collect();
        self
    }

    /// Calculates the lift (how much more likely fraud is when flag is present).
    pub fn lift(&self) -> f64 {
        if self.inject_without_fraud > 0.0 {
            self.inject_with_fraud / self.inject_without_fraud
        } else {
            f64::INFINITY
        }
    }
}

/// An instantiated red flag on a specific transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedFlag {
    /// Reference to the pattern name.
    pub pattern_name: String,
    /// Document ID where flag was detected.
    pub document_id: String,
    /// Category of the flag.
    pub category: RedFlagCategory,
    /// Strength of the flag.
    pub strength: RedFlagStrength,
    /// Specific details about the flag instance.
    pub details: HashMap<String, String>,
    /// Whether this flag is actually associated with fraud.
    pub is_fraudulent: bool,
    /// Confidence score (0.0-1.0).
    pub confidence: f64,
}

impl RedFlag {
    /// Creates a new red flag instance.
    pub fn new(
        pattern_name: impl Into<String>,
        document_id: impl Into<String>,
        category: RedFlagCategory,
        strength: RedFlagStrength,
        is_fraudulent: bool,
    ) -> Self {
        Self {
            pattern_name: pattern_name.into(),
            document_id: document_id.into(),
            category,
            strength,
            details: HashMap::new(),
            is_fraudulent,
            confidence: 1.0,
        }
    }

    /// Adds a detail to the flag.
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }

    /// Sets the confidence score.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
}

/// Generator for red flags.
#[derive(Debug, Clone)]
pub struct RedFlagGenerator {
    /// Strong red flag patterns.
    pub strong_flags: Vec<RedFlagPattern>,
    /// Moderate red flag patterns.
    pub moderate_flags: Vec<RedFlagPattern>,
    /// Weak red flag patterns.
    pub weak_flags: Vec<RedFlagPattern>,
}

impl Default for RedFlagGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RedFlagGenerator {
    /// Creates a new red flag generator with default patterns.
    pub fn new() -> Self {
        Self {
            strong_flags: Self::default_strong_flags(),
            moderate_flags: Self::default_moderate_flags(),
            weak_flags: Self::default_weak_flags(),
        }
    }

    /// Returns all patterns.
    pub fn all_patterns(&self) -> Vec<&RedFlagPattern> {
        let mut patterns: Vec<&RedFlagPattern> = Vec::new();
        patterns.extend(self.strong_flags.iter());
        patterns.extend(self.moderate_flags.iter());
        patterns.extend(self.weak_flags.iter());
        patterns
    }

    /// Generates red flags for a transaction.
    pub fn inject_flags<R: Rng>(
        &self,
        document_id: &str,
        is_fraud: bool,
        rng: &mut R,
    ) -> Vec<RedFlag> {
        let mut flags = Vec::new();

        // Process strong flags
        for pattern in &self.strong_flags {
            let prob = if is_fraud {
                pattern.inject_with_fraud
            } else {
                pattern.inject_without_fraud
            };
            if rng.gen::<f64>() < prob {
                flags.push(self.create_flag(document_id, pattern, is_fraud));
            }
        }

        // Process moderate flags
        for pattern in &self.moderate_flags {
            let prob = if is_fraud {
                pattern.inject_with_fraud
            } else {
                pattern.inject_without_fraud
            };
            if rng.gen::<f64>() < prob {
                flags.push(self.create_flag(document_id, pattern, is_fraud));
            }
        }

        // Process weak flags
        for pattern in &self.weak_flags {
            let prob = if is_fraud {
                pattern.inject_with_fraud
            } else {
                pattern.inject_without_fraud
            };
            if rng.gen::<f64>() < prob {
                flags.push(self.create_flag(document_id, pattern, is_fraud));
            }
        }

        flags
    }

    /// Creates a red flag instance from a pattern.
    fn create_flag(&self, document_id: &str, pattern: &RedFlagPattern, is_fraud: bool) -> RedFlag {
        RedFlag::new(
            &pattern.name,
            document_id,
            pattern.category,
            pattern.strength,
            is_fraud,
        )
        .with_confidence(pattern.fraud_probability)
    }

    /// Adds a custom pattern.
    pub fn add_pattern(&mut self, pattern: RedFlagPattern) {
        match pattern.strength {
            RedFlagStrength::Strong => self.strong_flags.push(pattern),
            RedFlagStrength::Moderate => self.moderate_flags.push(pattern),
            RedFlagStrength::Weak => self.weak_flags.push(pattern),
        }
    }

    /// Default strong red flag patterns.
    fn default_strong_flags() -> Vec<RedFlagPattern> {
        vec![
            RedFlagPattern::new(
                "matched_address_vendor_employee",
                "Vendor address matches an employee's home address",
                RedFlagCategory::Vendor,
                RedFlagStrength::Strong,
                0.85,
                0.90,
                0.001,
            )
            .with_related_schemes(vec!["shell_company", "fictitious_vendor"]),
            RedFlagPattern::new(
                "sequential_check_numbers_same_vendor",
                "Sequential check numbers paid to the same vendor",
                RedFlagCategory::Transaction,
                RedFlagStrength::Strong,
                0.70,
                0.80,
                0.01,
            )
            .with_related_schemes(vec!["duplicate_payment", "check_tampering"]),
            RedFlagPattern::new(
                "po_box_only_vendor",
                "Vendor has only PO Box address, no physical address",
                RedFlagCategory::Vendor,
                RedFlagStrength::Strong,
                0.60,
                0.75,
                0.02,
            )
            .with_related_schemes(vec!["fictitious_vendor", "shell_company"]),
            RedFlagPattern::new(
                "vendor_bank_matches_employee",
                "Vendor bank account matches employee's account",
                RedFlagCategory::Vendor,
                RedFlagStrength::Strong,
                0.90,
                0.95,
                0.0005,
            )
            .with_related_schemes(vec!["fictitious_vendor", "personal_purchases"]),
            RedFlagPattern::new(
                "approver_processor_same_person",
                "Same person created and approved the transaction",
                RedFlagCategory::Employee,
                RedFlagStrength::Strong,
                0.65,
                0.85,
                0.015,
            )
            .with_related_schemes(vec!["self_approval", "segregation_violation"]),
        ]
    }

    /// Default moderate red flag patterns.
    fn default_moderate_flags() -> Vec<RedFlagPattern> {
        vec![
            RedFlagPattern::new(
                "vendor_no_physical_address",
                "Vendor has no verified physical address on file",
                RedFlagCategory::Vendor,
                RedFlagStrength::Moderate,
                0.40,
                0.60,
                0.05,
            ),
            RedFlagPattern::new(
                "amount_just_below_threshold",
                "Amount is just below approval threshold",
                RedFlagCategory::Transaction,
                RedFlagStrength::Moderate,
                0.35,
                0.70,
                0.10,
            )
            .with_related_schemes(vec!["threshold_avoidance", "split_transaction"]),
            RedFlagPattern::new(
                "unusual_vendor_payment_pattern",
                "Payment pattern to vendor differs from historical norm",
                RedFlagCategory::Vendor,
                RedFlagStrength::Moderate,
                0.30,
                0.55,
                0.08,
            ),
            RedFlagPattern::new(
                "new_vendor_large_first_payment",
                "New vendor receives unusually large first payment",
                RedFlagCategory::Vendor,
                RedFlagStrength::Moderate,
                0.40,
                0.65,
                0.06,
            )
            .with_related_schemes(vec!["shell_company", "kickback"]),
            RedFlagPattern::new(
                "missing_supporting_documentation",
                "Transaction lacks required supporting documentation",
                RedFlagCategory::Document,
                RedFlagStrength::Moderate,
                0.35,
                0.60,
                0.08,
            ),
            RedFlagPattern::new(
                "employee_vacation_fraud_pattern",
                "Suspicious transactions only when specific employee present",
                RedFlagCategory::Employee,
                RedFlagStrength::Moderate,
                0.45,
                0.70,
                0.05,
            ),
            RedFlagPattern::new(
                "dormant_vendor_reactivation",
                "Previously dormant vendor suddenly receives payments",
                RedFlagCategory::Vendor,
                RedFlagStrength::Moderate,
                0.35,
                0.50,
                0.07,
            ),
            RedFlagPattern::new(
                "invoice_without_purchase_order",
                "Invoice paid without corresponding purchase order",
                RedFlagCategory::Document,
                RedFlagStrength::Moderate,
                0.30,
                0.55,
                0.12,
            ),
        ]
    }

    /// Default weak red flag patterns.
    fn default_weak_flags() -> Vec<RedFlagPattern> {
        vec![
            RedFlagPattern::new(
                "round_dollar_amount",
                "Transaction amount is a round number",
                RedFlagCategory::Transaction,
                RedFlagStrength::Weak,
                0.15,
                0.40,
                0.20,
            ),
            RedFlagPattern::new(
                "month_end_timing",
                "Transaction posted at month/quarter/year end",
                RedFlagCategory::Timing,
                RedFlagStrength::Weak,
                0.10,
                0.50,
                0.30,
            ),
            RedFlagPattern::new(
                "benford_first_digit_deviation",
                "First digit distribution deviates from Benford's Law",
                RedFlagCategory::Transaction,
                RedFlagStrength::Weak,
                0.12,
                0.35,
                0.15,
            ),
            RedFlagPattern::new(
                "after_hours_posting",
                "Transaction posted outside normal business hours",
                RedFlagCategory::Timing,
                RedFlagStrength::Weak,
                0.15,
                0.45,
                0.18,
            ),
            RedFlagPattern::new(
                "unusual_account_combination",
                "Debit/credit account combination is unusual",
                RedFlagCategory::Account,
                RedFlagStrength::Weak,
                0.20,
                0.40,
                0.12,
            ),
            RedFlagPattern::new(
                "repeat_amount_pattern",
                "Same exact amount appears multiple times",
                RedFlagCategory::Transaction,
                RedFlagStrength::Weak,
                0.18,
                0.45,
                0.15,
            ),
            RedFlagPattern::new(
                "weekend_transaction",
                "Transaction recorded on weekend",
                RedFlagCategory::Timing,
                RedFlagStrength::Weak,
                0.12,
                0.35,
                0.15,
            ),
            RedFlagPattern::new(
                "vague_description",
                "Transaction description is vague or missing",
                RedFlagCategory::Document,
                RedFlagStrength::Weak,
                0.15,
                0.40,
                0.18,
            ),
        ]
    }
}

/// Statistics about generated red flags.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RedFlagStatistics {
    /// Total flags generated.
    pub total_flags: usize,
    /// Flags on fraudulent transactions.
    pub flags_with_fraud: usize,
    /// Flags on legitimate transactions (false positives).
    pub flags_without_fraud: usize,
    /// Breakdown by strength.
    pub by_strength: HashMap<String, usize>,
    /// Breakdown by category.
    pub by_category: HashMap<String, usize>,
    /// Breakdown by pattern name.
    pub by_pattern: HashMap<String, usize>,
}

impl RedFlagStatistics {
    /// Creates statistics from a list of flags.
    #[allow(clippy::field_reassign_with_default)]
    pub fn from_flags(flags: &[RedFlag]) -> Self {
        let mut stats = Self::default();
        stats.total_flags = flags.len();

        for flag in flags {
            if flag.is_fraudulent {
                stats.flags_with_fraud += 1;
            } else {
                stats.flags_without_fraud += 1;
            }

            *stats
                .by_strength
                .entry(format!("{:?}", flag.strength))
                .or_insert(0) += 1;

            *stats
                .by_category
                .entry(format!("{:?}", flag.category))
                .or_insert(0) += 1;

            *stats
                .by_pattern
                .entry(flag.pattern_name.clone())
                .or_insert(0) += 1;
        }

        stats
    }

    /// Returns the precision (true positive rate among flagged transactions).
    pub fn precision(&self) -> f64 {
        if self.total_flags > 0 {
            self.flags_with_fraud as f64 / self.total_flags as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_red_flag_pattern() {
        let pattern = RedFlagPattern::new(
            "test_pattern",
            "Test pattern description",
            RedFlagCategory::Vendor,
            RedFlagStrength::Strong,
            0.80,
            0.90,
            0.05,
        )
        .with_related_schemes(vec!["shell_company"]);

        assert_eq!(pattern.name, "test_pattern");
        assert_eq!(pattern.strength, RedFlagStrength::Strong);
        assert!((pattern.lift() - 18.0).abs() < 0.01); // 0.90 / 0.05 = 18
    }

    #[test]
    fn test_red_flag() {
        let flag = RedFlag::new(
            "matched_address",
            "INV001",
            RedFlagCategory::Vendor,
            RedFlagStrength::Strong,
            true,
        )
        .with_detail("vendor_id", "V001")
        .with_confidence(0.85);

        assert_eq!(flag.document_id, "INV001");
        assert!(flag.is_fraudulent);
        assert_eq!(flag.confidence, 0.85);
        assert_eq!(flag.details.get("vendor_id"), Some(&"V001".to_string()));
    }

    #[test]
    fn test_red_flag_generator() {
        let generator = RedFlagGenerator::new();

        assert!(!generator.strong_flags.is_empty());
        assert!(!generator.moderate_flags.is_empty());
        assert!(!generator.weak_flags.is_empty());

        let all_patterns = generator.all_patterns();
        assert!(all_patterns.len() > 15);
    }

    #[test]
    fn test_inject_flags_fraud() {
        let generator = RedFlagGenerator::new();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // Test with fraud - should have higher flag rate
        let fraud_flags: Vec<RedFlag> = (0..100)
            .flat_map(|i| generator.inject_flags(&format!("DOC{:03}", i), true, &mut rng))
            .collect();

        // Test without fraud - should have lower flag rate
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);
        let legit_flags: Vec<RedFlag> = (0..100)
            .flat_map(|i| generator.inject_flags(&format!("DOC{:03}", i), false, &mut rng2))
            .collect();

        // Fraud should generate more flags on average
        assert!(fraud_flags.len() > legit_flags.len());
    }

    #[test]
    fn test_red_flag_statistics() {
        let flags = vec![
            RedFlag::new(
                "pattern1",
                "DOC1",
                RedFlagCategory::Vendor,
                RedFlagStrength::Strong,
                true,
            ),
            RedFlag::new(
                "pattern2",
                "DOC2",
                RedFlagCategory::Transaction,
                RedFlagStrength::Moderate,
                true,
            ),
            RedFlag::new(
                "pattern3",
                "DOC3",
                RedFlagCategory::Timing,
                RedFlagStrength::Weak,
                false,
            ),
        ];

        let stats = RedFlagStatistics::from_flags(&flags);

        assert_eq!(stats.total_flags, 3);
        assert_eq!(stats.flags_with_fraud, 2);
        assert_eq!(stats.flags_without_fraud, 1);
        assert!((stats.precision() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_default_patterns_have_correct_properties() {
        let generator = RedFlagGenerator::new();

        // Strong flags should have high fraud probability
        for pattern in &generator.strong_flags {
            assert!(pattern.fraud_probability >= 0.60);
            assert!(pattern.inject_with_fraud > pattern.inject_without_fraud);
        }

        // Weak flags should have low fraud probability
        for pattern in &generator.weak_flags {
            assert!(pattern.fraud_probability < 0.30);
        }
    }

    #[test]
    fn test_add_custom_pattern() {
        let mut generator = RedFlagGenerator::new();
        let initial_strong = generator.strong_flags.len();

        generator.add_pattern(RedFlagPattern::new(
            "custom_pattern",
            "Custom test pattern",
            RedFlagCategory::Account,
            RedFlagStrength::Strong,
            0.75,
            0.85,
            0.03,
        ));

        assert_eq!(generator.strong_flags.len(), initial_strong + 1);
    }
}
