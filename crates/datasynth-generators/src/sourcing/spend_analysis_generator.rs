//! Spend analysis generator.
//!
//! Analyzes vendor spend to identify sourcing opportunities.

use datasynth_config::schema::SpendAnalysisConfig;
use datasynth_core::models::sourcing::{SpendAnalysis, VendorSpendShare};
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

/// Generates spend analysis records from vendor pool and transaction data.
pub struct SpendAnalysisGenerator {
    rng: ChaCha8Rng,
    config: SpendAnalysisConfig,
}

impl SpendAnalysisGenerator {
    /// Create a new spend analysis generator.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config: SpendAnalysisConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(seed: u64, config: SpendAnalysisConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
        }
    }

    /// Generate spend analysis for a set of vendor-category pairs.
    ///
    /// # Arguments
    /// * `company_code` - Company code
    /// * `vendor_ids` - Available vendor IDs
    /// * `categories` - Spend categories (category_id, category_name)
    /// * `fiscal_year` - Analysis period
    pub fn generate(
        &mut self,
        company_code: &str,
        vendor_ids: &[String],
        categories: &[(String, String)],
        fiscal_year: u16,
    ) -> Vec<SpendAnalysis> {
        let mut analyses = Vec::new();

        for (cat_id, cat_name) in categories {
            // Assign random vendors to this category
            let vendor_count = self.rng.gen_range(3..=vendor_ids.len().min(15));
            let mut cat_vendors: Vec<&String> = vendor_ids
                .choose_multiple(&mut self.rng, vendor_count)
                .collect();
            cat_vendors.shuffle(&mut self.rng);

            // Generate spend shares using Pareto-like distribution
            let mut raw_shares: Vec<f64> = (0..cat_vendors.len())
                .map(|i| 1.0 / ((i as f64 + 1.0).powf(0.8)))
                .collect();
            let total: f64 = raw_shares.iter().sum();
            for s in &mut raw_shares {
                *s /= total;
            }

            let total_spend = Decimal::from(self.rng.gen_range(100_000i64..=5_000_000));
            let transaction_count = self.rng.gen_range(50..=2000);

            // Calculate HHI
            let hhi: f64 = raw_shares.iter().map(|s| (s * 100.0).powi(2)).sum();

            let contract_coverage = self.rng.gen_range(0.3..=0.95);
            let preferred_coverage = contract_coverage * self.rng.gen_range(0.7..=1.0);

            let vendor_shares: Vec<VendorSpendShare> = cat_vendors
                .iter()
                .zip(raw_shares.iter())
                .map(|(vid, share)| VendorSpendShare {
                    vendor_id: vid.to_string(),
                    vendor_name: format!("Vendor {}", vid),
                    spend_amount: Decimal::from_f64_retain(
                        total_spend.to_string().parse::<f64>().unwrap_or(0.0) * share,
                    )
                    .unwrap_or(Decimal::ZERO),
                    share: *share,
                    is_preferred: *share > 0.15 && self.rng.gen_bool(preferred_coverage),
                })
                .collect();

            analyses.push(SpendAnalysis {
                category_id: cat_id.clone(),
                category_name: cat_name.clone(),
                company_code: company_code.to_string(),
                total_spend,
                vendor_count: cat_vendors.len() as u32,
                transaction_count,
                hhi_index: hhi,
                vendor_shares,
                contract_coverage,
                preferred_vendor_coverage: preferred_coverage,
                price_trend_pct: self.rng.gen_range(-0.05..=0.10),
                fiscal_year,
            });
        }

        analyses
    }

    /// Get the HHI threshold from config.
    pub fn hhi_threshold(&self) -> f64 {
        self.config.hhi_threshold
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_vendor_ids() -> Vec<String> {
        (1..=10).map(|i| format!("V{:04}", i)).collect()
    }

    fn test_categories() -> Vec<(String, String)> {
        vec![
            ("CAT-001".to_string(), "Office Supplies".to_string()),
            ("CAT-002".to_string(), "IT Equipment".to_string()),
        ]
    }

    #[test]
    fn test_basic_generation() {
        let mut gen = SpendAnalysisGenerator::new(42);
        let results = gen.generate("C001", &test_vendor_ids(), &test_categories(), 2024);

        assert_eq!(results.len(), 2);
        for analysis in &results {
            assert_eq!(analysis.company_code, "C001");
            assert_eq!(analysis.fiscal_year, 2024);
            assert!(!analysis.category_id.is_empty());
            assert!(!analysis.category_name.is_empty());
            assert!(analysis.vendor_count > 0);
            assert!(analysis.transaction_count > 0);
            assert!(analysis.total_spend > Decimal::ZERO);
            assert!(analysis.hhi_index > 0.0);
            assert!(!analysis.vendor_shares.is_empty());
        }
    }

    #[test]
    fn test_deterministic() {
        let mut gen1 = SpendAnalysisGenerator::new(42);
        let mut gen2 = SpendAnalysisGenerator::new(42);
        let vendors = test_vendor_ids();
        let cats = test_categories();

        let r1 = gen1.generate("C001", &vendors, &cats, 2024);
        let r2 = gen2.generate("C001", &vendors, &cats, 2024);

        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a.category_id, b.category_id);
            assert_eq!(a.total_spend, b.total_spend);
            assert_eq!(a.vendor_count, b.vendor_count);
            assert_eq!(a.transaction_count, b.transaction_count);
        }
    }

    #[test]
    fn test_field_constraints() {
        let mut gen = SpendAnalysisGenerator::new(99);
        let results = gen.generate("C001", &test_vendor_ids(), &test_categories(), 2024);

        for analysis in &results {
            // Shares should sum to approximately 1.0
            let share_sum: f64 = analysis.vendor_shares.iter().map(|s| s.share).sum();
            assert!(
                (share_sum - 1.0).abs() < 0.01,
                "shares should sum to ~1.0, got {}",
                share_sum
            );

            // Contract coverage and price trend should be in valid range
            assert!(analysis.contract_coverage >= 0.0 && analysis.contract_coverage <= 1.0);
            assert!(
                analysis.preferred_vendor_coverage >= 0.0
                    && analysis.preferred_vendor_coverage <= 1.0
            );
            assert!(analysis.price_trend_pct >= -0.05 && analysis.price_trend_pct <= 0.10);

            // Each vendor share should have a non-empty vendor_id
            for vs in &analysis.vendor_shares {
                assert!(!vs.vendor_id.is_empty());
            }
        }
    }

    #[test]
    fn test_hhi_threshold() {
        let gen = SpendAnalysisGenerator::new(42);
        assert_eq!(gen.hhi_threshold(), 2500.0);
    }
}
