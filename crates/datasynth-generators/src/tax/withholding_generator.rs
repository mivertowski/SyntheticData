//! Withholding Tax Generator.
//!
//! Generates [`WithholdingTaxRecord`]s for cross-border payments, applying
//! treaty rates when a bilateral tax treaty exists between the source country
//! and vendor country, or falling back to a configurable default withholding
//! rate.

use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

use datasynth_core::models::{WithholdingTaxRecord, WithholdingType};

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates withholding tax records for cross-border vendor payments.
///
/// For each payment where `vendor_country != source_country`:
/// - Looks up the treaty rate for the `(source_country, vendor_country)` pair.
/// - If a treaty exists, applies the treaty rate.
/// - If no treaty exists, applies the default withholding rate.
/// - Computes `withheld_amount = base_amount * applied_rate`.
///
/// Domestic payments (where `vendor_country == source_country`) are excluded
/// from withholding and produce no records.
///
/// # Standard Treaty Rates
///
/// The [`with_standard_treaties`](WithholdingGenerator::with_standard_treaties)
/// method loads a US-centric treaty network with service withholding rates for
/// major trading partners (GB, DE, JP, FR, SG, IN, BR).
pub struct WithholdingGenerator {
    rng: ChaCha8Rng,
    /// Treaty rates indexed by `(source_country, vendor_country)`.
    treaty_rates: HashMap<(String, String), Decimal>,
    /// Default withholding rate when no treaty applies.
    default_rate: Decimal,
    counter: u64,
}

impl WithholdingGenerator {
    /// Creates a new withholding generator with the given seed and default rate.
    ///
    /// The default rate is applied when no treaty rate exists for a given
    /// country pair. A common value is `0.30` (30%).
    pub fn new(seed: u64, default_rate: Decimal) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            treaty_rates: HashMap::new(),
            default_rate,
            counter: 0,
        }
    }

    /// Adds a treaty rate for a specific country pair.
    ///
    /// The rate is stored for the `(source_country, vendor_country)` direction.
    /// Treaty rates are directional: a US-DE treaty rate applies when the US is
    /// the source and DE is the vendor, but not vice versa.
    pub fn add_treaty_rate(&mut self, source_country: &str, vendor_country: &str, rate: Decimal) {
        self.treaty_rates.insert(
            (source_country.to_string(), vendor_country.to_string()),
            rate,
        );
    }

    /// Loads the standard US treaty network for service withholding rates.
    ///
    /// Treaty rates (service withholding, US perspective):
    /// - US-GB: 0% (services)
    /// - US-DE: 0% (services)
    /// - US-JP: 0% (services)
    /// - US-FR: 0% (services)
    /// - US-SG: 0% (services)
    /// - US-IN: 15% (services)
    /// - US-BR: 15% (services)
    pub fn with_standard_treaties(mut self) -> Self {
        let treaties = [
            ("US", "GB", dec!(0.00)),
            ("US", "DE", dec!(0.00)),
            ("US", "JP", dec!(0.00)),
            ("US", "FR", dec!(0.00)),
            ("US", "SG", dec!(0.00)),
            ("US", "IN", dec!(0.15)),
            ("US", "BR", dec!(0.15)),
        ];

        for (source, vendor, rate) in &treaties {
            self.treaty_rates
                .insert((source.to_string(), vendor.to_string()), *rate);
        }

        self
    }

    /// Generate withholding records for cross-border payments.
    ///
    /// Each payment is a tuple of `(payment_id, vendor_id, vendor_country, amount)`.
    /// Domestic payments (where `vendor_country == source_country`) are excluded.
    ///
    /// For each cross-border payment:
    /// - If a treaty rate exists for `(source_country, vendor_country)`, the
    ///   treaty rate is applied.
    /// - Otherwise the `default_rate` is applied.
    /// - `withheld_amount = base_amount * applied_rate`.
    pub fn generate(
        &mut self,
        payments: &[(String, String, String, Decimal)],
        source_country: &str,
    ) -> Vec<WithholdingTaxRecord> {
        let mut records = Vec::new();

        for (payment_id, vendor_id, vendor_country, amount) in payments {
            // Skip domestic payments
            if vendor_country == source_country {
                continue;
            }

            let key = (source_country.to_string(), vendor_country.clone());
            let (applied_rate, treaty_rate) = match self.treaty_rates.get(&key) {
                Some(&rate) => (rate, Some(rate)),
                None => (self.default_rate, None),
            };

            self.counter += 1;
            let record_id = format!("WHT-{:06}", self.counter);

            // Generate a certificate number with some randomness
            let cert_suffix: u32 = self.rng.random_range(100_000..999_999);
            let cert_number = format!("CERT-{}-{cert_suffix}", &record_id);

            let mut record = WithholdingTaxRecord::new(
                record_id,
                payment_id,
                vendor_id,
                WithholdingType::ServiceWithholding,
                self.default_rate,
                applied_rate,
                *amount,
            )
            .with_certificate_number(cert_number);

            if let Some(rate) = treaty_rate {
                record = record.with_treaty_rate(rate);
            }

            records.push(record);
        }

        records
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn payment(
        id: &str,
        vendor_id: &str,
        vendor_country: &str,
        amount: Decimal,
    ) -> (String, String, String, Decimal) {
        (
            id.to_string(),
            vendor_id.to_string(),
            vendor_country.to_string(),
            amount,
        )
    }

    #[test]
    fn test_with_treaty_rate() {
        let mut gen = WithholdingGenerator::new(42, dec!(0.30)).with_standard_treaties();

        let payments = vec![payment("PAY-001", "V-GB-01", "GB", dec!(100000))];

        let records = gen.generate(&payments, "US");

        assert_eq!(records.len(), 1);
        let rec = &records[0];
        assert_eq!(rec.vendor_id, "V-GB-01");
        assert_eq!(rec.applied_rate, dec!(0.00));
        assert_eq!(rec.treaty_rate, Some(dec!(0.00)));
        assert_eq!(rec.withheld_amount, dec!(0.00));
        assert_eq!(rec.statutory_rate, dec!(0.30));
        assert!(rec.has_treaty_benefit());
    }

    #[test]
    fn test_without_treaty() {
        let mut gen = WithholdingGenerator::new(42, dec!(0.30)).with_standard_treaties();

        // ZZ is not in the treaty network
        let payments = vec![payment("PAY-002", "V-ZZ-01", "ZZ", dec!(50000))];

        let records = gen.generate(&payments, "US");

        assert_eq!(records.len(), 1);
        let rec = &records[0];
        assert_eq!(rec.applied_rate, dec!(0.30));
        assert_eq!(rec.treaty_rate, None);
        assert_eq!(rec.withheld_amount, dec!(15000.00));
        assert!(!rec.has_treaty_benefit());
    }

    #[test]
    fn test_standard_treaties() {
        let gen = WithholdingGenerator::new(42, dec!(0.30)).with_standard_treaties();

        // Verify all standard treaty rates are loaded
        assert_eq!(gen.treaty_rates.len(), 7);

        assert_eq!(
            gen.treaty_rates.get(&("US".to_string(), "GB".to_string())),
            Some(&dec!(0.00))
        );
        assert_eq!(
            gen.treaty_rates.get(&("US".to_string(), "DE".to_string())),
            Some(&dec!(0.00))
        );
        assert_eq!(
            gen.treaty_rates.get(&("US".to_string(), "JP".to_string())),
            Some(&dec!(0.00))
        );
        assert_eq!(
            gen.treaty_rates.get(&("US".to_string(), "FR".to_string())),
            Some(&dec!(0.00))
        );
        assert_eq!(
            gen.treaty_rates.get(&("US".to_string(), "SG".to_string())),
            Some(&dec!(0.00))
        );
        assert_eq!(
            gen.treaty_rates.get(&("US".to_string(), "IN".to_string())),
            Some(&dec!(0.15))
        );
        assert_eq!(
            gen.treaty_rates.get(&("US".to_string(), "BR".to_string())),
            Some(&dec!(0.15))
        );
    }

    #[test]
    fn test_domestic_excluded() {
        let mut gen = WithholdingGenerator::new(42, dec!(0.30)).with_standard_treaties();

        let payments = vec![
            payment("PAY-DOM", "V-US-01", "US", dec!(100000)),
            payment("PAY-XB", "V-GB-01", "GB", dec!(50000)),
        ];

        let records = gen.generate(&payments, "US");

        // Only the cross-border payment should produce a record
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].payment_id, "PAY-XB");
    }

    #[test]
    fn test_deterministic() {
        let payments = vec![
            payment("PAY-001", "V-GB-01", "GB", dec!(100000)),
            payment("PAY-002", "V-IN-01", "IN", dec!(50000)),
            payment("PAY-003", "V-ZZ-01", "ZZ", dec!(25000)),
        ];

        let mut gen1 = WithholdingGenerator::new(12345, dec!(0.30)).with_standard_treaties();
        let records1 = gen1.generate(&payments, "US");

        let mut gen2 = WithholdingGenerator::new(12345, dec!(0.30)).with_standard_treaties();
        let records2 = gen2.generate(&payments, "US");

        assert_eq!(records1.len(), records2.len());
        for (r1, r2) in records1.iter().zip(records2.iter()) {
            assert_eq!(r1.id, r2.id);
            assert_eq!(r1.payment_id, r2.payment_id);
            assert_eq!(r1.vendor_id, r2.vendor_id);
            assert_eq!(r1.applied_rate, r2.applied_rate);
            assert_eq!(r1.treaty_rate, r2.treaty_rate);
            assert_eq!(r1.withheld_amount, r2.withheld_amount);
            assert_eq!(r1.certificate_number, r2.certificate_number);
        }
    }

    #[test]
    fn test_treaty_with_nonzero_rate() {
        let mut gen = WithholdingGenerator::new(42, dec!(0.30)).with_standard_treaties();

        // India has a 15% treaty rate for services
        let payments = vec![payment("PAY-IN", "V-IN-01", "IN", dec!(100000))];

        let records = gen.generate(&payments, "US");

        assert_eq!(records.len(), 1);
        let rec = &records[0];
        assert_eq!(rec.applied_rate, dec!(0.15));
        assert_eq!(rec.treaty_rate, Some(dec!(0.15)));
        assert_eq!(rec.withheld_amount, dec!(15000.00));
        assert_eq!(rec.statutory_rate, dec!(0.30));
        assert!(
            rec.has_treaty_benefit(),
            "15% treaty rate is less than 30% statutory"
        );
        assert_eq!(rec.treaty_savings(), dec!(15000.00));
    }

    #[test]
    fn test_custom_treaty_rate() {
        let mut gen = WithholdingGenerator::new(42, dec!(0.25));
        gen.add_treaty_rate("DE", "US", dec!(0.05));

        let payments = vec![payment("PAY-001", "V-US-01", "US", dec!(200000))];

        let records = gen.generate(&payments, "DE");

        assert_eq!(records.len(), 1);
        let rec = &records[0];
        assert_eq!(rec.applied_rate, dec!(0.05));
        assert_eq!(rec.treaty_rate, Some(dec!(0.05)));
        assert_eq!(rec.withheld_amount, dec!(10000.00));
        assert_eq!(rec.statutory_rate, dec!(0.25));
    }
}
