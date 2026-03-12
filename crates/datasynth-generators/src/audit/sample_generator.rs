//! Audit sample generator per ISA 530.
//!
//! Generates `AuditSample` records with realistic item distributions for
//! workpapers that use statistical sampling.  Workpapers with `SamplingMethod::Judgmental`
//! and `population_size == 0` are treated as non-sampling procedures and receive
//! no sample (returns `None`).

use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use uuid::Uuid;

use datasynth_core::models::audit::{
	AuditSample, SampleItem, SampleItemResult, SamplingMethod, Workpaper,
};

/// Configuration for the sample generator (ISA 530).
#[derive(Debug, Clone)]
pub struct SampleGeneratorConfig {
	/// Number of items to include in each sample (min, max)
	pub items_per_sample: (u32, u32),
	/// Fraction of items that are correct (no misstatement)
	pub correct_ratio: f64,
	/// Fraction of items that have a misstatement
	pub misstatement_ratio: f64,
	/// Fraction of items that have a deviation / exception
	pub exception_ratio: f64,
	/// Only generate samples for workpapers with statistical sampling methods;
	/// when `false`, any workpaper with `population_size > 0` receives a sample.
	pub generate_for_non_sampling: bool,
}

impl Default for SampleGeneratorConfig {
	fn default() -> Self {
		Self {
			items_per_sample: (15, 60),
			correct_ratio: 0.90,
			misstatement_ratio: 0.07,
			exception_ratio: 0.03,
			generate_for_non_sampling: false,
		}
	}
}

/// Generator for `AuditSample` records per ISA 530.
pub struct SampleGenerator {
	/// Seeded random number generator
	rng: ChaCha8Rng,
	/// Configuration
	config: SampleGeneratorConfig,
	/// Counter for human-readable document references
	item_counter: u64,
}

impl SampleGenerator {
	/// Create a new generator with the given seed and default configuration.
	pub fn new(seed: u64) -> Self {
		Self {
			rng: seeded_rng(seed, 0),
			config: SampleGeneratorConfig::default(),
			item_counter: 0,
		}
	}

	/// Create a new generator with custom configuration.
	pub fn with_config(seed: u64, config: SampleGeneratorConfig) -> Self {
		Self { rng: seeded_rng(seed, 0), config, item_counter: 0 }
	}

	/// Generate an `AuditSample` for a workpaper, or `None` if sampling is not applicable.
	///
	/// A sample is generated when:
	/// - The workpaper's `sampling_method` is one of the statistical methods
	///   (`StatisticalRandom`, `MonetaryUnit`), **or**
	/// - `config.generate_for_non_sampling` is `true` and `workpaper.population_size > 0`.
	///
	/// # Arguments
	/// * `workpaper`     — The workpaper to create the sample for.
	/// * `engagement_id` — The engagement UUID (must match `workpaper.engagement_id`).
	pub fn generate_sample(
		&mut self,
		workpaper: &Workpaper,
		engagement_id: Uuid,
	) -> Option<AuditSample> {
		// Decide whether to generate a sample for this workpaper.
		let is_statistical = matches!(
			workpaper.sampling_method,
			SamplingMethod::StatisticalRandom | SamplingMethod::MonetaryUnit
		);
		let has_population = workpaper.population_size > 0;

		let should_generate = is_statistical || (self.config.generate_for_non_sampling && has_population);
		if !should_generate {
			return None;
		}

		let sample_count = self
			.rng
			.random_range(self.config.items_per_sample.0..=self.config.items_per_sample.1);

		// Population description derived from workpaper title.
		let pop_description = format!("{} — sampled population", workpaper.title);

		let mut sample = AuditSample::new(
			workpaper.workpaper_id,
			engagement_id,
			pop_description,
			workpaper.population_size.max(sample_count as u64),
			workpaper.sampling_method,
			sample_count,
		);

		// Set population value (rough estimate: average item ~$50k × population size).
		let pop_value_units: i64 =
			(workpaper.population_size as i64).saturating_mul(50_000_i64).max(100_000);
		sample.population_value = Some(Decimal::new(pop_value_units, 0));

		// Tolerable misstatement ≈ 5% of population value.
		sample.tolerable_misstatement = sample.population_value.map(|v| v / Decimal::from(20));

		// Generate the individual items.
		for _ in 0..sample_count {
			self.item_counter += 1;
			let doc_ref = format!("DOC-{:06}", self.item_counter);

			// Book value: $1k – $500k
			let book_units: i64 = self.rng.random_range(1_000_i64..=500_000_i64);
			let book_value = Decimal::new(book_units, 0);

			let roll: f64 = self.rng.random();
			let misstatement_cutoff = self.config.misstatement_ratio;
			let exception_cutoff = misstatement_cutoff + self.config.exception_ratio;

			let mut item = SampleItem::new(&doc_ref, book_value);

			if roll < misstatement_cutoff {
				// Misstatement: audited value differs by 1–15% of book.
				let pct: f64 = self.rng.random_range(0.01..0.15);
				let diff_units = (book_units as f64 * pct).round() as i64;
				let diff = Decimal::new(diff_units.max(1), 0);
				// Randomly overstate or understate.
				let audited = if self.rng.random::<bool>() {
					book_value + diff
				} else {
					(book_value - diff).max(Decimal::ZERO)
				};
				let misstatement = book_value - audited;

				item.audited_value = Some(audited);
				item.misstatement = Some(misstatement);
				item.result = SampleItemResult::Misstatement;
			} else if roll < exception_cutoff {
				// Exception / deviation: audited value differs by 5–20%.
				let pct: f64 = self.rng.random_range(0.05..0.20);
				let diff_units = (book_units as f64 * pct).round() as i64;
				let diff = Decimal::new(diff_units.max(1), 0);
				let audited = (book_value - diff).max(Decimal::ZERO);
				let misstatement = book_value - audited;

				item.audited_value = Some(audited);
				item.misstatement = Some(misstatement);
				item.result = SampleItemResult::Exception;
			} else {
				// Correct: audited value equals book value.
				item.audited_value = Some(book_value);
				item.result = SampleItemResult::Correct;
			}

			sample.add_item(item);
		}

		// Compute projection and reach a conclusion.
		sample.conclude();

		// Upgrade to InsufficientEvidence → use actual projection.
		// (conclude() handles this already; no override needed.)

		Some(sample)
	}
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
	use super::*;
	use datasynth_core::models::audit::{
		ProcedureType, SampleConclusion, Workpaper, WorkpaperScope, WorkpaperSection,
	};

	fn make_gen(seed: u64) -> SampleGenerator {
		SampleGenerator::new(seed)
	}

	/// Build a workpaper that will receive a sample (statistical method + population).
	fn sampling_workpaper(method: SamplingMethod) -> Workpaper {
		Workpaper::new(
			Uuid::new_v4(),
			"D-100",
			"Accounts Receivable Testing",
			WorkpaperSection::SubstantiveTesting,
		)
		.with_procedure("Test AR balances", ProcedureType::SubstantiveTest)
		.with_scope(WorkpaperScope::default(), 1_000, 50, method)
	}

	fn non_sampling_workpaper() -> Workpaper {
		Workpaper::new(
			Uuid::new_v4(),
			"C-100",
			"Controls Walk-through",
			WorkpaperSection::ControlTesting,
		)
		.with_scope(WorkpaperScope::default(), 0, 0, SamplingMethod::Judgmental)
	}

	// -------------------------------------------------------------------------

	/// A statistical-method workpaper produces a sample with items in range.
	#[test]
	fn test_generates_sample() {
		let wp = sampling_workpaper(SamplingMethod::StatisticalRandom);
		let eng_id = wp.engagement_id;
		let mut gen = make_gen(42);
		let sample = gen.generate_sample(&wp, eng_id).unwrap();

		let cfg = SampleGeneratorConfig::default();
		let min = cfg.items_per_sample.0 as usize;
		let max = cfg.items_per_sample.1 as usize;
		assert!(
			sample.items.len() >= min && sample.items.len() <= max,
			"expected {min}..={max} items, got {}",
			sample.items.len()
		);
		assert!(sample.conclusion.is_some(), "sample should have a conclusion");
	}

	/// A non-sampling workpaper with population_size == 0 returns None.
	#[test]
	fn test_no_sample_for_non_sampling() {
		let wp = non_sampling_workpaper();
		let eng_id = wp.engagement_id;
		let mut gen = make_gen(99);
		let result = gen.generate_sample(&wp, eng_id);
		assert!(result.is_none(), "expected None for non-sampling workpaper");
	}

	/// With a large count, the item result distribution should roughly match the config.
	#[test]
	fn test_item_distribution() {
		let wp = sampling_workpaper(SamplingMethod::MonetaryUnit);
		let eng_id = wp.engagement_id;
		let config = SampleGeneratorConfig {
			items_per_sample: (300, 300),
			correct_ratio: 0.90,
			misstatement_ratio: 0.07,
			exception_ratio: 0.03,
			generate_for_non_sampling: false,
		};
		let mut gen = SampleGenerator::with_config(77, config);
		let sample = gen.generate_sample(&wp, eng_id).unwrap();

		let total = sample.items.len() as f64;
		let correct_count = sample
			.items
			.iter()
			.filter(|i| i.result == SampleItemResult::Correct)
			.count() as f64;

		// Correct ratio should be within ±15% of 90%.
		let ratio = correct_count / total;
		assert!(
			(0.75..=1.00).contains(&ratio),
			"correct ratio {ratio:.2} outside expected 75–100%"
		);
	}

	/// Same seed produces identical output.
	#[test]
	fn test_deterministic() {
		let wp = sampling_workpaper(SamplingMethod::StatisticalRandom);
		let eng_id = wp.engagement_id;

		let sample_a = SampleGenerator::new(1234).generate_sample(&wp, eng_id).unwrap();
		let sample_b = SampleGenerator::new(1234).generate_sample(&wp, eng_id).unwrap();

		assert_eq!(sample_a.items.len(), sample_b.items.len());
		for (a, b) in sample_a.items.iter().zip(sample_b.items.iter()) {
			assert_eq!(a.document_ref, b.document_ref);
			assert_eq!(a.book_value, b.book_value);
			assert_eq!(a.result, b.result);
		}
		assert_eq!(sample_a.conclusion, sample_b.conclusion);
	}

	/// `generate_for_non_sampling = true` causes a Judgmental-method workpaper
	/// with population_size > 0 to receive a sample.
	#[test]
	fn test_generate_for_non_sampling_flag() {
		let mut wp = non_sampling_workpaper();
		wp.population_size = 500; // non-zero population
		let eng_id = wp.engagement_id;

		let config = SampleGeneratorConfig {
			generate_for_non_sampling: true,
			..Default::default()
		};
		let mut gen = SampleGenerator::with_config(55, config);
		let result = gen.generate_sample(&wp, eng_id);
		assert!(result.is_some(), "expected Some when generate_for_non_sampling = true");
	}

	/// Misstatement items should have a non-zero misstatement amount.
	#[test]
	fn test_misstatement_items_have_amounts() {
		let wp = sampling_workpaper(SamplingMethod::StatisticalRandom);
		let eng_id = wp.engagement_id;
		let config = SampleGeneratorConfig {
			items_per_sample: (200, 200),
			misstatement_ratio: 0.50, // inflate so we always get some
			exception_ratio: 0.10,
			correct_ratio: 0.40,
			generate_for_non_sampling: false,
		};
		let mut gen = SampleGenerator::with_config(33, config);
		let sample = gen.generate_sample(&wp, eng_id).unwrap();

		let mist_items: Vec<_> = sample
			.items
			.iter()
			.filter(|i| i.result == SampleItemResult::Misstatement)
			.collect();

		assert!(!mist_items.is_empty(), "expected some misstatement items");
		for item in mist_items {
			assert!(
				item.misstatement.is_some(),
				"misstatement item should have a misstatement amount"
			);
			// misstatement amount should be non-zero
			assert_ne!(
				item.misstatement.unwrap(),
				Decimal::ZERO,
				"misstatement amount should not be zero"
			);
		}
	}

	/// The sample conclusion should be a valid `SampleConclusion` variant.
	#[test]
	fn test_conclusion_is_set() {
		let wp = sampling_workpaper(SamplingMethod::MonetaryUnit);
		let eng_id = wp.engagement_id;
		let mut gen = make_gen(12);
		let sample = gen.generate_sample(&wp, eng_id).unwrap();

		let conclusion = sample.conclusion.unwrap();
		let valid = matches!(
			conclusion,
			SampleConclusion::ProjectedBelowTolerable
				| SampleConclusion::ProjectedExceedsTolerable
				| SampleConclusion::InsufficientEvidence
		);
		assert!(valid, "unexpected SampleConclusion variant");
	}
}
