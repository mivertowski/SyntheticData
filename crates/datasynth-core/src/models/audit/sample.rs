//! Audit sample models per ISA 530.
//!
//! Provides structures for documenting audit sampling — the items selected,
//! misstatements found, and the projected conclusion against tolerable error.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::SamplingMethod;

/// Result for a single sampled item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SampleItemResult {
    /// No misstatement or deviation — item is correct
    #[default]
    Correct,
    /// A misstatement was identified
    Misstatement,
    /// A deviation / exception was noted
    Exception,
}

/// Overall conclusion for the audit sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SampleConclusion {
    /// Projected misstatement is at or below tolerable misstatement
    ProjectedBelowTolerable,
    /// Projected misstatement exceeds tolerable misstatement
    ProjectedExceedsTolerable,
    /// Insufficient information to reach a conclusion
    #[default]
    InsufficientEvidence,
}

/// A single item selected for testing within an audit sample.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleItem {
    /// Unique item ID
    pub item_id: Uuid,
    /// Document or transaction reference
    pub document_ref: String,
    /// Book value of the item
    pub book_value: Decimal,
    /// Audited (corrected) value, if tested
    pub audited_value: Option<Decimal>,
    /// Misstatement amount (book minus audited), if any
    pub misstatement: Option<Decimal>,
    /// Result for this item
    pub result: SampleItemResult,
}

impl SampleItem {
    /// Create a new sample item with only a document reference and book value.
    pub fn new(document_ref: impl Into<String>, book_value: Decimal) -> Self {
        Self {
            item_id: Uuid::new_v4(),
            document_ref: document_ref.into(),
            book_value,
            audited_value: None,
            misstatement: None,
            result: SampleItemResult::Correct,
        }
    }
}

/// A documented audit sample per ISA 530.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSample {
    /// Unique sample ID
    pub sample_id: Uuid,
    /// Sample reference code, e.g. "SAMP-a1b2c3d4"
    pub sample_ref: String,
    /// Workpaper this sample is part of
    pub workpaper_id: Uuid,
    /// Engagement this sample belongs to
    pub engagement_id: Uuid,
    /// Description of the population tested
    pub population_description: String,
    /// Total number of items in the population
    pub population_size: u64,
    /// Total monetary value of the population (for MUS / projection)
    pub population_value: Option<Decimal>,
    /// Sampling methodology used
    pub sampling_method: SamplingMethod,
    /// Planned / actual number of items selected
    pub sample_size: u32,
    /// Sampling interval (used for systematic / MUS selection)
    pub sampling_interval: Option<Decimal>,
    /// Confidence level (e.g. 0.95 for 95 %)
    pub confidence_level: f64,
    /// Tolerable misstatement threshold
    pub tolerable_misstatement: Option<Decimal>,
    /// Expected misstatement used in sample size determination
    pub expected_misstatement: Option<Decimal>,
    /// Individual items tested
    pub items: Vec<SampleItem>,
    /// Cumulative misstatement found across all items
    pub total_misstatement_found: Decimal,
    /// Projected population misstatement
    pub projected_misstatement: Option<Decimal>,
    /// Conclusion reached
    pub conclusion: Option<SampleConclusion>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last-modified timestamp
    pub updated_at: DateTime<Utc>,
}

impl AuditSample {
    /// Create a new audit sample.
    pub fn new(
        workpaper_id: Uuid,
        engagement_id: Uuid,
        population_description: impl Into<String>,
        population_size: u64,
        sampling_method: SamplingMethod,
        sample_size: u32,
    ) -> Self {
        let now = Utc::now();
        let sample_ref = format!("SAMP-{}", &workpaper_id.to_string()[..8]);
        Self {
            sample_id: Uuid::new_v4(),
            sample_ref,
            workpaper_id,
            engagement_id,
            population_description: population_description.into(),
            population_size,
            population_value: None,
            sampling_method,
            sample_size,
            sampling_interval: None,
            confidence_level: 0.95,
            tolerable_misstatement: None,
            expected_misstatement: None,
            items: Vec::new(),
            total_misstatement_found: Decimal::ZERO,
            projected_misstatement: None,
            conclusion: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a tested item to the sample and accumulate total misstatement.
    pub fn add_item(&mut self, item: SampleItem) {
        if let Some(m) = item.misstatement {
            self.total_misstatement_found += m.abs();
        }
        self.items.push(item);
        self.updated_at = Utc::now();
    }

    /// Compute projected population misstatement based on sample results.
    ///
    /// Formula: `(total_misstatement / sample_value) × population_value`
    ///
    /// Falls back to `0` when there are no items or the sample value is zero.
    pub fn compute_projected_misstatement(&mut self) {
        if self.items.is_empty() {
            self.projected_misstatement = Some(Decimal::ZERO);
            return;
        }

        let sample_value: Decimal = self.items.iter().map(|i| i.book_value).sum();
        if sample_value == Decimal::ZERO {
            self.projected_misstatement = Some(Decimal::ZERO);
            return;
        }

        let projected = match self.population_value {
            Some(pop_val) => {
                // (total_misstatement / sample_value) * population_value
                let rate = self.total_misstatement_found / sample_value;
                rate * pop_val
            }
            None => {
                // No population value — scale by population count / sample count
                let pop_count =
                    Decimal::from(self.population_size);
                let samp_count = Decimal::from(self.items.len() as u64);
                if samp_count == Decimal::ZERO {
                    Decimal::ZERO
                } else {
                    let rate = self.total_misstatement_found / sample_value;
                    // Approximate: rate × (pop_count / samp_count) × average_book_value
                    let avg_book = sample_value / samp_count;
                    rate * avg_book * pop_count
                }
            }
        };

        self.projected_misstatement = Some(projected);
        self.updated_at = Utc::now();
    }

    /// Compute projection and reach a conclusion against tolerable misstatement.
    pub fn conclude(&mut self) {
        self.compute_projected_misstatement();
        let projected = self.projected_misstatement.unwrap_or(Decimal::ZERO);

        self.conclusion = Some(match self.tolerable_misstatement {
            Some(tolerable) => {
                if projected <= tolerable {
                    SampleConclusion::ProjectedBelowTolerable
                } else {
                    SampleConclusion::ProjectedExceedsTolerable
                }
            }
            None => SampleConclusion::InsufficientEvidence,
        });
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn make_sample() -> AuditSample {
        AuditSample::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Accounts receivable invoices over $1,000",
            500,
            SamplingMethod::MonetaryUnit,
            50,
        )
    }

    #[test]
    fn test_new_sample() {
        let s = make_sample();
        assert_eq!(s.sample_size, 50);
        assert_eq!(s.population_size, 500);
        assert_eq!(s.confidence_level, 0.95);
        assert_eq!(s.total_misstatement_found, Decimal::ZERO);
        assert!(s.conclusion.is_none());
        assert!(s.sample_ref.starts_with("SAMP-"));
    }

    #[test]
    fn test_add_item_accumulates_misstatement() {
        let mut s = make_sample();

        let mut item1 = SampleItem::new("INV-001", dec!(1000));
        item1.misstatement = Some(dec!(50));
        item1.result = SampleItemResult::Misstatement;

        let mut item2 = SampleItem::new("INV-002", dec!(2000));
        item2.misstatement = Some(dec!(-30)); // negative misstatement — abs is taken
        item2.result = SampleItemResult::Misstatement;

        s.add_item(item1);
        s.add_item(item2);

        assert_eq!(s.total_misstatement_found, dec!(80)); // 50 + 30
        assert_eq!(s.items.len(), 2);
    }

    #[test]
    fn test_compute_projected_zero_items() {
        let mut s = make_sample();
        s.compute_projected_misstatement();
        assert_eq!(s.projected_misstatement, Some(Decimal::ZERO));
    }

    #[test]
    fn test_compute_projected_zero_sample_value() {
        let mut s = make_sample();
        // Item with zero book value
        s.add_item(SampleItem::new("INV-000", dec!(0)));
        s.compute_projected_misstatement();
        assert_eq!(s.projected_misstatement, Some(Decimal::ZERO));
    }

    #[test]
    fn test_compute_projected_normal() {
        let mut s = make_sample();
        s.population_value = Some(dec!(100_000));

        let mut item = SampleItem::new("INV-001", dec!(5_000));
        item.misstatement = Some(dec!(500));
        s.add_item(item);

        s.compute_projected_misstatement();
        // rate = 500/5000 = 0.1; projected = 0.1 * 100_000 = 10_000
        assert_eq!(s.projected_misstatement, Some(dec!(10_000)));
    }

    #[test]
    fn test_conclude_below_tolerable() {
        let mut s = make_sample();
        s.population_value = Some(dec!(100_000));
        s.tolerable_misstatement = Some(dec!(15_000));

        let mut item = SampleItem::new("INV-001", dec!(5_000));
        item.misstatement = Some(dec!(500)); // projected = 10_000 < 15_000
        s.add_item(item);

        s.conclude();
        assert_eq!(s.conclusion, Some(SampleConclusion::ProjectedBelowTolerable));
    }

    #[test]
    fn test_conclude_exceeds_tolerable() {
        let mut s = make_sample();
        s.population_value = Some(dec!(100_000));
        s.tolerable_misstatement = Some(dec!(5_000));

        let mut item = SampleItem::new("INV-001", dec!(5_000));
        item.misstatement = Some(dec!(500)); // projected = 10_000 > 5_000
        s.add_item(item);

        s.conclude();
        assert_eq!(s.conclusion, Some(SampleConclusion::ProjectedExceedsTolerable));
    }

    #[test]
    fn test_conclude_no_tolerable() {
        let mut s = make_sample();
        // no tolerable_misstatement set
        s.conclude();
        assert_eq!(s.conclusion, Some(SampleConclusion::InsufficientEvidence));
    }

    #[test]
    fn test_sampling_method_serde() {
        let methods = [
            SamplingMethod::StatisticalRandom,
            SamplingMethod::MonetaryUnit,
            SamplingMethod::Judgmental,
            SamplingMethod::Haphazard,
            SamplingMethod::Block,
            SamplingMethod::AllItems,
        ];
        for m in &methods {
            let json = serde_json::to_string(m).unwrap();
            let back: SamplingMethod = serde_json::from_str(&json).unwrap();
            assert_eq!(back, *m);
        }
    }

    #[test]
    fn test_sample_conclusion_serde() {
        let conclusions = [
            SampleConclusion::ProjectedBelowTolerable,
            SampleConclusion::ProjectedExceedsTolerable,
            SampleConclusion::InsufficientEvidence,
        ];
        for c in &conclusions {
            let json = serde_json::to_string(c).unwrap();
            let back: SampleConclusion = serde_json::from_str(&json).unwrap();
            assert_eq!(back, *c);
        }
    }
}
