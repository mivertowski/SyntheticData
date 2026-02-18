//! Sales quote generator.
//!
//! Generates realistic sales quotations with line items, following the
//! Quote-to-Cash lifecycle from draft through win/loss/expiry.

use chrono::{Datelike, NaiveDate};
use datasynth_config::schema::SalesQuoteConfig;
use datasynth_core::models::{QuoteLineItem, QuoteStatus, SalesQuote};
use datasynth_core::utils::{sample_decimal_range, seeded_rng};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

/// Generates [`SalesQuote`] instances with realistic line items,
/// status distribution, and customer/material assignments.
pub struct SalesQuoteGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    // Reserved for deterministic line-item IDs when QuoteLineItem gains a UUID field.
    #[allow(dead_code)]
    item_uuid_factory: DeterministicUuidFactory,
}

impl SalesQuoteGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::SalesQuote),
            item_uuid_factory: DeterministicUuidFactory::with_sub_discriminator(
                seed,
                GeneratorType::SalesQuote,
                1,
            ),
        }
    }

    /// Generate sales quotes for the given period and configuration.
    ///
    /// # Arguments
    ///
    /// * `company_code` - The company code issuing quotes.
    /// * `customer_ids` - Slice of (customer_id, customer_name) tuples.
    /// * `material_ids` - Slice of (material_id, description) tuples.
    /// * `period_start` - Start of the generation period (inclusive).
    /// * `period_end` - End of the generation period (inclusive).
    /// * `config` - Sales quote configuration knobs.
    pub fn generate(
        &mut self,
        company_code: &str,
        customer_ids: &[(String, String)],
        material_ids: &[(String, String)],
        period_start: NaiveDate,
        period_end: NaiveDate,
        config: &SalesQuoteConfig,
    ) -> Vec<SalesQuote> {
        if customer_ids.is_empty() || material_ids.is_empty() {
            return Vec::new();
        }

        let mut quotes = Vec::new();

        // Iterate over each month in the period
        let mut year = period_start.year();
        let mut month = period_start.month();
        let end_year = period_end.year();
        let end_month = period_end.month();

        loop {
            // Generate quotes_per_month quotes for this month
            for _ in 0..config.quotes_per_month {
                let quote = self.generate_single_quote(
                    company_code,
                    customer_ids,
                    material_ids,
                    year,
                    month,
                    config,
                );
                quotes.push(quote);
            }

            // Advance to next month
            if year == end_year && month == end_month {
                break;
            }
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }

        quotes
    }

    /// Generate a single sales quote for a given month.
    fn generate_single_quote(
        &mut self,
        company_code: &str,
        customer_ids: &[(String, String)],
        material_ids: &[(String, String)],
        year: i32,
        month: u32,
        config: &SalesQuoteConfig,
    ) -> SalesQuote {
        let quote_id = self.uuid_factory.next().to_string();

        // Random customer
        let customer_idx = self.rng.gen_range(0..customer_ids.len());
        let (customer_id, customer_name) = &customer_ids[customer_idx];

        // Random quote date within the month
        let last_day = last_day_of_month(year, month);
        let day = self.rng.gen_range(1..=last_day);
        let quote_date = NaiveDate::from_ymd_opt(year, month, day).unwrap_or_else(|| {
            NaiveDate::from_ymd_opt(year, month, 1)
                .unwrap_or(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap_or_default())
        });
        let valid_until = quote_date + chrono::Duration::days(config.validity_days as i64);

        // Generate 1-5 line items
        let item_count = self.rng.gen_range(1..=5usize);
        let mut line_items = Vec::with_capacity(item_count);
        let mut total_amount = Decimal::ZERO;

        for item_num in 1..=item_count {
            let mat_idx = self.rng.gen_range(0..material_ids.len());
            let (material_id, description) = &material_ids[mat_idx];

            let unit_price =
                sample_decimal_range(&mut self.rng, Decimal::from(50), Decimal::from(5000))
                    .round_dp(2);
            let quantity =
                sample_decimal_range(&mut self.rng, Decimal::ONE, Decimal::from(100)).round_dp(0);
            let line_amount = (unit_price * quantity).round_dp(2);
            total_amount += line_amount;

            line_items.push(QuoteLineItem {
                item_number: item_num as u32,
                material_id: material_id.clone(),
                description: description.clone(),
                quantity,
                unit_price,
                line_amount,
            });
        }

        // Discount: 20% of quotes get 5-20% discount
        let (discount_percent, discount_amount) = if self.rng.gen::<f64>() < 0.20 {
            let pct = self.rng.gen_range(0.05..0.20);
            let disc_amount =
                (Decimal::from_f64_retain(pct).unwrap_or(Decimal::ZERO) * total_amount).round_dp(2);
            (pct, disc_amount)
        } else {
            (0.0, Decimal::ZERO)
        };

        // Status distribution:
        // win_rate fraction Won, 25% Lost, 15% Expired, 10% Sent, 5% Draft, rest Negotiating
        let status_roll: f64 = self.rng.gen();
        let won_threshold = config.win_rate;
        let lost_threshold = won_threshold + 0.25;
        let expired_threshold = lost_threshold + 0.15;
        let sent_threshold = expired_threshold + 0.10;
        let draft_threshold = sent_threshold + 0.05;

        let status = if status_roll < won_threshold {
            QuoteStatus::Won
        } else if status_roll < lost_threshold {
            QuoteStatus::Lost
        } else if status_roll < expired_threshold {
            QuoteStatus::Expired
        } else if status_roll < sent_threshold {
            QuoteStatus::Sent
        } else if status_roll < draft_threshold {
            QuoteStatus::Draft
        } else {
            QuoteStatus::Negotiating
        };

        // Won quotes get a linked sales order ID
        let sales_order_id = if status == QuoteStatus::Won {
            let so_num = self.rng.gen_range(1..=999999u32);
            Some(format!("SO-{:06}", so_num))
        } else {
            None
        };

        // Lost quotes get a reason
        let lost_reasons = [
            "Price too high",
            "Competitor won",
            "Budget constraints",
            "Requirements changed",
            "Timing not right",
        ];
        let lost_reason = if status == QuoteStatus::Lost {
            let idx = self.rng.gen_range(0..lost_reasons.len());
            Some(lost_reasons[idx].to_string())
        } else {
            None
        };

        // Sales rep: "SR-{01-20}"
        let rep_num = self.rng.gen_range(1..=20u32);
        let sales_rep_id = Some(format!("SR-{:02}", rep_num));

        SalesQuote {
            quote_id,
            company_code: company_code.to_string(),
            customer_id: customer_id.clone(),
            customer_name: customer_name.clone(),
            quote_date,
            valid_until,
            status,
            line_items,
            total_amount,
            currency: "USD".to_string(),
            discount_percent,
            discount_amount,
            sales_rep_id,
            sales_order_id,
            lost_reason,
            notes: None,
        }
    }
}

/// Return the last day of the given month/year.
fn last_day_of_month(year: i32, month: u32) -> u32 {
    // The first day of the next month, minus one day
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .unwrap_or(28)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sample_customers() -> Vec<(String, String)> {
        vec![
            ("CUST-001".to_string(), "Acme Corp".to_string()),
            ("CUST-002".to_string(), "Globex Inc".to_string()),
            ("CUST-003".to_string(), "Initech LLC".to_string()),
        ]
    }

    fn sample_materials() -> Vec<(String, String)> {
        vec![
            ("MAT-001".to_string(), "Widget A".to_string()),
            ("MAT-002".to_string(), "Widget B".to_string()),
            ("MAT-003".to_string(), "Gadget X".to_string()),
            ("MAT-004".to_string(), "Component Y".to_string()),
        ]
    }

    fn default_config() -> SalesQuoteConfig {
        SalesQuoteConfig {
            enabled: true,
            quotes_per_month: 30,
            win_rate: 0.35,
            validity_days: 30,
        }
    }

    #[test]
    fn test_basic_generation_produces_expected_count() {
        let mut gen = SalesQuoteGenerator::new(42);
        let customers = sample_customers();
        let materials = sample_materials();
        let config = default_config();

        let period_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

        let quotes = gen.generate(
            "C001",
            &customers,
            &materials,
            period_start,
            period_end,
            &config,
        );

        // 3 months * 30 quotes_per_month = 90
        assert_eq!(quotes.len(), 90);

        // All quotes should have at least one line item
        for q in &quotes {
            assert!(!q.line_items.is_empty());
            assert!(q.line_items.len() <= 5);
            assert!(q.total_amount > Decimal::ZERO);
            assert!(!q.quote_id.is_empty());
            assert_eq!(q.company_code, "C001");
            assert!(q.sales_rep_id.is_some());
        }

        // Won quotes should have sales_order_id
        for q in quotes.iter().filter(|q| q.status == QuoteStatus::Won) {
            assert!(
                q.sales_order_id.is_some(),
                "Won quotes must have a sales order ID"
            );
        }

        // Lost quotes should have a reason
        for q in quotes.iter().filter(|q| q.status == QuoteStatus::Lost) {
            assert!(
                q.lost_reason.is_some(),
                "Lost quotes must have a lost reason"
            );
        }
    }

    #[test]
    fn test_deterministic_output_with_same_seed() {
        let customers = sample_customers();
        let materials = sample_materials();
        let config = default_config();

        let period_start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();

        let mut gen1 = SalesQuoteGenerator::new(12345);
        let quotes1 = gen1.generate(
            "C001",
            &customers,
            &materials,
            period_start,
            period_end,
            &config,
        );

        let mut gen2 = SalesQuoteGenerator::new(12345);
        let quotes2 = gen2.generate(
            "C001",
            &customers,
            &materials,
            period_start,
            period_end,
            &config,
        );

        assert_eq!(quotes1.len(), quotes2.len());
        for (q1, q2) in quotes1.iter().zip(quotes2.iter()) {
            assert_eq!(q1.quote_id, q2.quote_id);
            assert_eq!(q1.customer_id, q2.customer_id);
            assert_eq!(q1.total_amount, q2.total_amount);
            assert_eq!(q1.status, q2.status);
        }
    }

    #[test]
    fn test_status_distribution_within_range() {
        let mut gen = SalesQuoteGenerator::new(999);
        let customers = sample_customers();
        let materials = sample_materials();
        let config = SalesQuoteConfig {
            enabled: true,
            quotes_per_month: 100,
            win_rate: 0.35,
            validity_days: 30,
        };

        let period_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let quotes = gen.generate(
            "C001",
            &customers,
            &materials,
            period_start,
            period_end,
            &config,
        );
        let total = quotes.len() as f64;

        let won_count = quotes
            .iter()
            .filter(|q| q.status == QuoteStatus::Won)
            .count() as f64;
        let lost_count = quotes
            .iter()
            .filter(|q| q.status == QuoteStatus::Lost)
            .count() as f64;

        // Win rate should be roughly 35% (allow 20-50% range for randomness)
        let win_rate = won_count / total;
        assert!(
            win_rate > 0.20 && win_rate < 0.50,
            "Win rate {} should be roughly 35%",
            win_rate
        );

        // Lost rate should be roughly 25% (allow 15-35% range)
        let lost_rate = lost_count / total;
        assert!(
            lost_rate > 0.15 && lost_rate < 0.35,
            "Lost rate {} should be roughly 25%",
            lost_rate
        );
    }
}
