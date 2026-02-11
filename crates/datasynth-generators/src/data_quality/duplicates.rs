//! Duplicate record generation for data quality simulation.
//!
//! Simulates realistic duplicate scenarios:
//! - Exact duplicates (complete record duplication)
//! - Near duplicates (minor variations)
//! - Fuzzy duplicates (similar but not identical)
//! - Cross-system duplicates (different identifiers, same entity)

use chrono::{Duration, NaiveDate};
use rand::Rng;
use rust_decimal::Decimal;

/// Type of duplicate.
#[derive(Debug, Clone, PartialEq)]
pub enum DuplicateType {
    /// Complete exact duplicate.
    Exact,
    /// Near duplicate with minor variations.
    Near {
        /// Fields that vary.
        varying_fields: Vec<String>,
    },
    /// Fuzzy duplicate with significant but recognizable differences.
    Fuzzy {
        /// Similarity threshold (0.0 - 1.0).
        similarity: f64,
    },
    /// Cross-system duplicate (same entity, different identifiers).
    CrossSystem {
        /// Source system identifier.
        source_system: String,
        /// Target system identifier.
        target_system: String,
    },
}

/// Configuration for duplicate generation.
#[derive(Debug, Clone)]
pub struct DuplicateConfig {
    /// Overall duplicate rate.
    pub duplicate_rate: f64,
    /// Exact duplicate rate (of duplicates).
    pub exact_rate: f64,
    /// Near duplicate rate (of duplicates).
    pub near_rate: f64,
    /// Fuzzy duplicate rate (of duplicates).
    pub fuzzy_rate: f64,
    /// Maximum days between duplicate entries.
    pub max_date_offset_days: i64,
    /// Fields that commonly vary in near duplicates.
    pub varying_fields: Vec<String>,
    /// Amount variance for near duplicates (percentage).
    pub amount_variance: f64,
}

impl Default for DuplicateConfig {
    fn default() -> Self {
        Self {
            duplicate_rate: 0.005, // 0.5% of records get duplicated
            exact_rate: 0.3,       // 30% of duplicates are exact
            near_rate: 0.5,        // 50% are near duplicates
            fuzzy_rate: 0.2,       // 20% are fuzzy
            max_date_offset_days: 5,
            varying_fields: vec![
                "entry_date".to_string(),
                "created_by".to_string(),
                "description".to_string(),
            ],
            amount_variance: 0.01, // 1% variance
        }
    }
}

/// A duplicate record with metadata.
#[derive(Debug, Clone)]
pub struct DuplicateRecord<T: Clone> {
    /// The original record.
    pub original: T,
    /// The duplicate record.
    pub duplicate: T,
    /// Type of duplicate.
    pub duplicate_type: DuplicateType,
    /// Fields that differ.
    pub differing_fields: Vec<String>,
    /// Duplicate ID for tracking.
    pub duplicate_id: String,
}

/// Trait for records that can be duplicated.
pub trait Duplicatable: Clone {
    /// Returns the record's unique identifier.
    fn get_id(&self) -> String;

    /// Sets a new identifier.
    fn set_id(&mut self, id: String);

    /// Gets a field value by name.
    fn get_field(&self, field: &str) -> Option<String>;

    /// Sets a field value by name.
    fn set_field(&mut self, field: &str, value: &str);

    /// Gets the amount (for amount-bearing records).
    fn get_amount(&self) -> Option<Decimal>;

    /// Sets the amount.
    fn set_amount(&mut self, amount: Decimal);

    /// Gets the date.
    fn get_date(&self) -> Option<NaiveDate>;

    /// Sets the date.
    fn set_date(&mut self, date: NaiveDate);
}

/// Duplicate generator.
pub struct DuplicateGenerator {
    config: DuplicateConfig,
    stats: DuplicateStats,
    next_duplicate_id: u64,
}

/// Statistics for duplicate generation.
#[derive(Debug, Clone, Default)]
pub struct DuplicateStats {
    /// Total records processed.
    pub total_processed: usize,
    /// Total duplicates created.
    pub total_duplicates: usize,
    /// Exact duplicates.
    pub exact_duplicates: usize,
    /// Near duplicates.
    pub near_duplicates: usize,
    /// Fuzzy duplicates.
    pub fuzzy_duplicates: usize,
    /// Cross-system duplicates.
    pub cross_system_duplicates: usize,
}

impl DuplicateGenerator {
    /// Creates a new duplicate generator.
    pub fn new(config: DuplicateConfig) -> Self {
        Self {
            config,
            stats: DuplicateStats::default(),
            next_duplicate_id: 1,
        }
    }

    /// Determines if a record should be duplicated.
    pub fn should_duplicate<R: Rng>(&self, rng: &mut R) -> bool {
        rng.gen::<f64>() < self.config.duplicate_rate
    }

    /// Creates a duplicate of a record.
    pub fn create_duplicate<T: Duplicatable, R: Rng>(
        &mut self,
        record: &T,
        rng: &mut R,
    ) -> DuplicateRecord<T> {
        self.stats.total_processed += 1;
        self.stats.total_duplicates += 1;

        let duplicate_type = self.select_duplicate_type(rng);
        let mut duplicate = record.clone();
        let mut differing_fields = Vec::new();

        // Generate new ID
        let new_id = format!("{}-DUP{}", record.get_id(), self.next_duplicate_id);
        self.next_duplicate_id += 1;
        duplicate.set_id(new_id);
        differing_fields.push("id".to_string());

        match &duplicate_type {
            DuplicateType::Exact => {
                self.stats.exact_duplicates += 1;
                // No other changes needed
            }
            DuplicateType::Near { varying_fields } => {
                self.stats.near_duplicates += 1;
                self.apply_near_duplicate_variations(&mut duplicate, varying_fields, rng);
                differing_fields.extend(varying_fields.clone());
            }
            DuplicateType::Fuzzy { similarity } => {
                self.stats.fuzzy_duplicates += 1;
                let varied = self.apply_fuzzy_variations(&mut duplicate, *similarity, rng);
                differing_fields.extend(varied);
            }
            DuplicateType::CrossSystem {
                source_system: _,
                target_system,
            } => {
                self.stats.cross_system_duplicates += 1;
                // Change system identifier
                if let Some(_current_id) = duplicate.get_field("system_id") {
                    duplicate.set_field("system_id", target_system);
                    differing_fields.push("system_id".to_string());
                }
            }
        }

        let duplicate_id = format!("DUP{:08}", self.stats.total_duplicates);

        DuplicateRecord {
            original: record.clone(),
            duplicate,
            duplicate_type,
            differing_fields,
            duplicate_id,
        }
    }

    /// Selects the type of duplicate to create.
    fn select_duplicate_type<R: Rng>(&self, rng: &mut R) -> DuplicateType {
        let r = rng.gen::<f64>();

        if r < self.config.exact_rate {
            DuplicateType::Exact
        } else if r < self.config.exact_rate + self.config.near_rate {
            DuplicateType::Near {
                varying_fields: self.config.varying_fields.clone(),
            }
        } else {
            DuplicateType::Fuzzy {
                similarity: rng.gen_range(0.8..0.95),
            }
        }
    }

    /// Applies near-duplicate variations.
    fn apply_near_duplicate_variations<T: Duplicatable, R: Rng>(
        &self,
        record: &mut T,
        varying_fields: &[String],
        rng: &mut R,
    ) {
        for field in varying_fields {
            match field.as_str() {
                "entry_date" | "date" => {
                    if let Some(date) = record.get_date() {
                        let offset = rng.gen_range(
                            -self.config.max_date_offset_days..=self.config.max_date_offset_days,
                        );
                        record.set_date(date + Duration::days(offset));
                    }
                }
                "amount" | "debit_amount" | "credit_amount" => {
                    if let Some(amount) = record.get_amount() {
                        let variance = 1.0
                            + rng.gen_range(
                                -self.config.amount_variance..self.config.amount_variance,
                            );
                        let new_amount =
                            amount * Decimal::from_f64_retain(variance).unwrap_or(Decimal::ONE);
                        record.set_amount(new_amount.round_dp(2));
                    }
                }
                "description" => {
                    if let Some(desc) = record.get_field("description") {
                        // Add minor variation
                        let variations = [
                            format!("{} ", desc),
                            format!(" {}", desc),
                            desc.to_uppercase(),
                            desc.to_lowercase(),
                        ];
                        let variation = &variations[rng.gen_range(0..variations.len())];
                        record.set_field("description", variation);
                    }
                }
                _ => {
                    // Generic variation: add whitespace
                    if let Some(value) = record.get_field(field) {
                        record.set_field(field, &format!("{} ", value));
                    }
                }
            }
        }
    }

    /// Applies fuzzy variations (more significant changes).
    fn apply_fuzzy_variations<T: Duplicatable, R: Rng>(
        &self,
        record: &mut T,
        similarity: f64,
        rng: &mut R,
    ) -> Vec<String> {
        let mut varied_fields = Vec::new();
        let change_probability = 1.0 - similarity;

        // Amount variation
        if rng.gen::<f64>() < change_probability {
            if let Some(amount) = record.get_amount() {
                let variance = 1.0 + rng.gen_range(-0.1..0.1); // Up to 10% variation
                let new_amount =
                    amount * Decimal::from_f64_retain(variance).unwrap_or(Decimal::ONE);
                record.set_amount(new_amount.round_dp(2));
                varied_fields.push("amount".to_string());
            }
        }

        // Date variation
        if rng.gen::<f64>() < change_probability {
            if let Some(date) = record.get_date() {
                let offset = rng.gen_range(-30..=30);
                record.set_date(date + Duration::days(offset));
                varied_fields.push("date".to_string());
            }
        }

        // Description variation
        if rng.gen::<f64>() < change_probability {
            if let Some(desc) = record.get_field("description") {
                // Introduce typos or abbreviations
                let abbreviated = abbreviate_text(&desc);
                record.set_field("description", &abbreviated);
                varied_fields.push("description".to_string());
            }
        }

        varied_fields
    }

    /// Returns statistics.
    pub fn stats(&self) -> &DuplicateStats {
        &self.stats
    }

    /// Resets statistics.
    pub fn reset_stats(&mut self) {
        self.stats = DuplicateStats::default();
    }
}

/// Abbreviates text by replacing common words.
fn abbreviate_text(text: &str) -> String {
    let abbreviations = [
        ("Account", "Acct"),
        ("Payment", "Pmt"),
        ("Invoice", "Inv"),
        ("Number", "No"),
        ("Department", "Dept"),
        ("Company", "Co"),
        ("Corporation", "Corp"),
        ("International", "Intl"),
        ("Management", "Mgmt"),
        ("Reference", "Ref"),
    ];

    let mut result = text.to_string();
    for (full, abbr) in abbreviations {
        result = result.replace(full, abbr);
    }
    result
}

/// Detects potential duplicates in a dataset.
pub struct DuplicateDetector {
    /// Similarity threshold for fuzzy matching.
    similarity_threshold: f64,
    /// Fields to compare.
    comparison_fields: Vec<String>,
}

impl DuplicateDetector {
    /// Creates a new duplicate detector.
    pub fn new(similarity_threshold: f64, comparison_fields: Vec<String>) -> Self {
        Self {
            similarity_threshold,
            comparison_fields,
        }
    }

    /// Calculates similarity between two strings (Jaccard similarity).
    pub fn string_similarity(&self, a: &str, b: &str) -> f64 {
        if a == b {
            return 1.0;
        }

        let a_chars: std::collections::HashSet<char> = a.chars().collect();
        let b_chars: std::collections::HashSet<char> = b.chars().collect();

        let intersection = a_chars.intersection(&b_chars).count();
        let union = a_chars.union(&b_chars).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Checks if two records are potential duplicates.
    pub fn are_duplicates<T: Duplicatable>(&self, a: &T, b: &T) -> bool {
        let mut total_similarity = 0.0;
        let mut field_count = 0;

        for field in &self.comparison_fields {
            if let (Some(val_a), Some(val_b)) = (a.get_field(field), b.get_field(field)) {
                total_similarity += self.string_similarity(&val_a, &val_b);
                field_count += 1;
            }
        }

        // Also compare amounts if available
        if let (Some(amt_a), Some(amt_b)) = (a.get_amount(), b.get_amount()) {
            let amt_a_f64: f64 = amt_a.try_into().unwrap_or(0.0);
            let amt_b_f64: f64 = amt_b.try_into().unwrap_or(0.0);

            if amt_a_f64.abs() > 0.0 {
                let ratio = (amt_a_f64 - amt_b_f64).abs() / amt_a_f64.abs();
                total_similarity += 1.0 - ratio.min(1.0);
                field_count += 1;
            }
        }

        if field_count == 0 {
            return false;
        }

        let avg_similarity = total_similarity / field_count as f64;
        avg_similarity >= self.similarity_threshold
    }

    /// Finds all duplicate pairs in a collection.
    pub fn find_duplicates<T: Duplicatable>(&self, records: &[T]) -> Vec<(usize, usize, f64)> {
        let mut duplicates = Vec::new();

        for i in 0..records.len() {
            for j in (i + 1)..records.len() {
                if self.are_duplicates(&records[i], &records[j]) {
                    let mut similarity = 0.0;
                    let mut count = 0;

                    for field in &self.comparison_fields {
                        if let (Some(a), Some(b)) =
                            (records[i].get_field(field), records[j].get_field(field))
                        {
                            similarity += self.string_similarity(&a, &b);
                            count += 1;
                        }
                    }

                    if count > 0 {
                        duplicates.push((i, j, similarity / count as f64));
                    }
                }
            }
        }

        duplicates
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Simple test struct implementing Duplicatable
    #[derive(Clone)]
    struct TestRecord {
        id: String,
        description: String,
        amount: Decimal,
        date: NaiveDate,
    }

    impl Duplicatable for TestRecord {
        fn get_id(&self) -> String {
            self.id.clone()
        }

        fn set_id(&mut self, id: String) {
            self.id = id;
        }

        fn get_field(&self, field: &str) -> Option<String> {
            match field {
                "description" => Some(self.description.clone()),
                "id" => Some(self.id.clone()),
                _ => None,
            }
        }

        fn set_field(&mut self, field: &str, value: &str) {
            if field == "description" {
                self.description = value.to_string();
            }
        }

        fn get_amount(&self) -> Option<Decimal> {
            Some(self.amount)
        }

        fn set_amount(&mut self, amount: Decimal) {
            self.amount = amount;
        }

        fn get_date(&self) -> Option<NaiveDate> {
            Some(self.date)
        }

        fn set_date(&mut self, date: NaiveDate) {
            self.date = date;
        }
    }

    #[test]
    fn test_duplicate_generation() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        use rust_decimal_macros::dec;

        let config = DuplicateConfig::default();
        let mut generator = DuplicateGenerator::new(config);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let record = TestRecord {
            id: "JE001".to_string(),
            description: "Test Entry".to_string(),
            amount: dec!(1000),
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        };

        let duplicate = generator.create_duplicate(&record, &mut rng);

        assert_ne!(duplicate.duplicate.get_id(), record.get_id());
        assert_eq!(generator.stats().total_duplicates, 1);
    }

    #[test]
    fn test_string_similarity() {
        let detector = DuplicateDetector::new(0.8, vec!["description".to_string()]);

        assert_eq!(detector.string_similarity("hello", "hello"), 1.0);
        assert!(detector.string_similarity("hello", "helo") > 0.8);
        assert!(detector.string_similarity("abc", "xyz") < 0.5);
    }

    #[test]
    fn test_abbreviate_text() {
        let text = "Account Payment Invoice";
        let abbreviated = abbreviate_text(text);
        assert_eq!(abbreviated, "Acct Pmt Inv");
    }
}
