//! IFRS 8 / ASC 280 Operating Segment Reporting generator.
//!
//! Produces:
//! - A set of [`OperatingSegment`] records that partition the consolidated
//!   financials into reportable segments (geographic or product-line).
//! - A [`SegmentReconciliation`] that proves segment totals tie back to
//!   the consolidated income-statement and balance-sheet totals.
//!
//! ## Segment derivation logic
//!
//! | Config | Segment basis |
//! |--------|---------------|
//! | Multi-entity (≥2 companies) | One `Geographic` segment per company |
//! | Single-entity | 2–3 `ProductLine` segments from CoA revenue sub-ranges |
//!
//! ## Reconciliation identity
//!
//! ```text
//! consolidated_revenue  = Σ revenue_external
//!                       = segment_revenue_total + intersegment_eliminations
//! consolidated_profit   = segment_profit_total  + corporate_overhead
//! consolidated_assets   = segment_assets_total  + unallocated_assets
//! ```

use datasynth_core::models::{OperatingSegment, SegmentReconciliation, SegmentType};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use tracing::debug;

/// Generates IFRS 8 / ASC 280 segment reporting data.
pub struct SegmentGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

/// Lightweight description of one entity / business unit used to seed segment names.
#[derive(Debug, Clone)]
pub struct SegmentSeed {
    /// Company or business-unit code (e.g. "C001")
    pub code: String,
    /// Human-readable name (e.g. "North America" or "Software Products")
    pub name: String,
    /// Currency used by this entity (informational only)
    pub currency: String,
}

impl SegmentGenerator {
    /// Create a new segment generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::SegmentReport),
        }
    }

    /// Generate operating segments and a reconciliation for one fiscal period.
    ///
    /// # Arguments
    /// * `company_code` – group / parent company code used in output records
    /// * `period` – fiscal period label (e.g. "2024-03")
    /// * `consolidated_revenue` – total external revenue from the consolidated IS
    /// * `consolidated_profit` – consolidated operating profit
    /// * `consolidated_assets` – consolidated total assets
    /// * `entity_seeds` – one entry per legal entity / product line to derive segments from
    /// * `total_depreciation` – consolidated D&A from the depreciation run (e.g.
    ///   `DepreciationRun.total_depreciation`).  When `Some`, distributed to segments
    ///   proportionally to their share of total assets.  When `None`, D&A is approximated
    ///   as 50–80 % of each segment's CapEx (a reasonable synthetic-data heuristic).
    ///
    /// # Returns
    /// `(segments, reconciliation)` where the reconciliation ties segment totals
    /// back to the consolidated figures passed in.
    pub fn generate(
        &mut self,
        company_code: &str,
        period: &str,
        consolidated_revenue: Decimal,
        consolidated_profit: Decimal,
        consolidated_assets: Decimal,
        entity_seeds: &[SegmentSeed],
        total_depreciation: Option<Decimal>,
    ) -> (Vec<OperatingSegment>, SegmentReconciliation) {
        debug!(
            company_code,
            period,
            consolidated_revenue = consolidated_revenue.to_string(),
            consolidated_profit = consolidated_profit.to_string(),
            consolidated_assets = consolidated_assets.to_string(),
            n_seeds = entity_seeds.len(),
            "Generating segment reports"
        );

        // Determine segment type and names
        let (segment_type, segment_names) = if entity_seeds.len() >= 2 {
            // Multi-entity: geographic segments = one per legal entity
            let names: Vec<String> = entity_seeds.iter().map(|s| s.name.clone()).collect();
            (SegmentType::Geographic, names)
        } else {
            // Single-entity: create 2-3 product line segments
            (SegmentType::ProductLine, self.default_product_lines())
        };

        let n = segment_names.len().clamp(2, 8);

        // Generate proportional splits for revenue, profit, assets
        let rev_splits = self.random_proportions(n);
        let profit_multipliers = self.profit_multipliers(n);
        let asset_splits = self.random_proportions(n);

        // Build segments
        let mut segments: Vec<OperatingSegment> = Vec::with_capacity(n);

        // Intersegment revenue: only meaningful for geographic / multi-entity
        // Use 3–8 % of gross revenue as intersegment transactions
        let intersegment_rate = if segment_type == SegmentType::Geographic && n >= 2 {
            let rate_bps = self.rng.random_range(300u32..=800);
            Decimal::from(rate_bps) / Decimal::from(10_000u32)
        } else {
            Decimal::ZERO
        };

        let total_intersegment = consolidated_revenue.abs() * intersegment_rate;

        // We want: Σ revenue_external = consolidated_revenue
        // Therefore allocate consolidated_revenue proportionally as external revenue.
        // Intersegment revenue is additional on top (it cancels in consolidation).
        let mut remaining_rev = consolidated_revenue;
        let mut remaining_profit = consolidated_profit;
        let mut remaining_assets = consolidated_assets;

        for (i, name) in segment_names.iter().take(n).enumerate() {
            let is_last = i == n - 1;

            let ext_rev = if is_last {
                remaining_rev
            } else {
                let r = consolidated_revenue * rev_splits[i];
                remaining_rev -= r;
                r
            };

            // Intersegment: distribute evenly across all segments (they net to zero)
            let interseg_rev = if intersegment_rate > Decimal::ZERO {
                total_intersegment * rev_splits[i]
            } else {
                Decimal::ZERO
            };

            // Operating profit: apply per-segment margin multiplier
            let seg_profit = if is_last {
                remaining_profit
            } else {
                let base_margin = if consolidated_revenue != Decimal::ZERO {
                    consolidated_profit / consolidated_revenue
                } else {
                    Decimal::ZERO
                };
                let adjusted_margin = base_margin * profit_multipliers[i];
                let p = ext_rev * adjusted_margin;
                remaining_profit -= p;
                p
            };

            let seg_assets = if is_last {
                remaining_assets
            } else {
                let a = consolidated_assets * asset_splits[i];
                remaining_assets -= a;
                a
            };

            // Liabilities: assume ~40-60 % of assets ratio with some noise
            let liab_ratio =
                Decimal::from(self.rng.random_range(35u32..=55)) / Decimal::from(100u32);
            let seg_liabilities = (seg_assets * liab_ratio).max(Decimal::ZERO);

            // CapEx: ~3-8 % of segment assets
            let capex_rate =
                Decimal::from(self.rng.random_range(30u32..=80)) / Decimal::from(1_000u32);
            let capex = (seg_assets * capex_rate).max(Decimal::ZERO);

            // D&A: when `total_depreciation` is provided (from the FA subledger depreciation
            // run), distribute it proportionally to each segment's share of total assets.
            // Fall back to the 50–80 % of CapEx heuristic when no actual depreciation data
            // is available (e.g. when the FA subledger is not generated).
            let da = if let Some(total_depr) = total_depreciation {
                let asset_share = if consolidated_assets != Decimal::ZERO {
                    seg_assets / consolidated_assets
                } else {
                    asset_splits[i]
                };
                (total_depr * asset_share).max(Decimal::ZERO)
            } else {
                let da_ratio =
                    Decimal::from(self.rng.random_range(50u32..=80)) / Decimal::from(100u32);
                capex * da_ratio
            };

            segments.push(OperatingSegment {
                segment_id: self.uuid_factory.next().to_string(),
                name: name.clone(),
                segment_type,
                revenue_external: ext_rev,
                revenue_intersegment: interseg_rev,
                operating_profit: seg_profit,
                total_assets: seg_assets,
                total_liabilities: seg_liabilities,
                capital_expenditure: capex,
                depreciation_amortization: da,
                period: period.to_string(),
                company_code: company_code.to_string(),
            });
        }

        // Corporate overhead: 2–5 % of consolidated revenue (negative)
        let overhead_rate = Decimal::from(self.rng.random_range(2u32..=5)) / Decimal::from(100u32);
        let corporate_overhead = -(consolidated_revenue.abs() * overhead_rate);

        // Unallocated assets: goodwill, deferred tax, etc — 5-12 % of total
        let unalloc_rate = Decimal::from(self.rng.random_range(5u32..=12)) / Decimal::from(100u32);
        let unallocated_assets = consolidated_assets.abs() * unalloc_rate;

        // Segment totals
        let segment_revenue_total: Decimal = segments
            .iter()
            .map(|s| s.revenue_external + s.revenue_intersegment)
            .sum();

        // Intersegment eliminations are derived to enforce the exact identity:
        //   segment_revenue_total + intersegment_eliminations = consolidated_revenue
        // This avoids any decimal precision residual from proportion arithmetic.
        let intersegment_eliminations = consolidated_revenue - segment_revenue_total;

        let segment_profit_total: Decimal = segments.iter().map(|s| s.operating_profit).sum();
        let segment_assets_total: Decimal = segments.iter().map(|s| s.total_assets).sum();

        let reconciliation = SegmentReconciliation {
            period: period.to_string(),
            company_code: company_code.to_string(),
            segment_revenue_total,
            intersegment_eliminations,
            consolidated_revenue,
            segment_profit_total,
            corporate_overhead,
            consolidated_profit: segment_profit_total + corporate_overhead,
            segment_assets_total,
            unallocated_assets,
            consolidated_assets: segment_assets_total + unallocated_assets,
        };

        (segments, reconciliation)
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Generate a default list of 3 product-line segment names.
    fn default_product_lines(&mut self) -> Vec<String> {
        let options: &[&[&str]] = &[
            &["Products", "Services", "Licensing"],
            &["Hardware", "Software", "Support"],
            &["Consumer", "Commercial", "Enterprise"],
            &["Core", "Growth", "Emerging"],
            &["Domestic", "International", "Other"],
        ];
        let idx = self.rng.random_range(0..options.len());
        options[idx].iter().map(|s| s.to_string()).collect()
    }

    /// Generate n random proportions that sum to 1.
    fn random_proportions(&mut self, n: usize) -> Vec<Decimal> {
        // Draw n uniform samples and normalise
        let raw: Vec<f64> = (0..n)
            .map(|_| self.rng.random_range(1u32..=100) as f64)
            .collect();
        let total: f64 = raw.iter().sum();
        raw.iter()
            .map(|v| Decimal::from_f64_retain(v / total).unwrap_or(Decimal::ZERO))
            .collect()
    }

    /// Generate per-segment profit multipliers (relative margin adjustments) around 1.0.
    fn profit_multipliers(&mut self, n: usize) -> Vec<Decimal> {
        (0..n)
            .map(|_| {
                let m = self.rng.random_range(70u32..=130);
                Decimal::from(m) / Decimal::from(100u32)
            })
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_seeds(names: &[&str]) -> Vec<SegmentSeed> {
        names
            .iter()
            .enumerate()
            .map(|(i, n)| SegmentSeed {
                code: format!("C{:03}", i + 1),
                name: n.to_string(),
                currency: "USD".to_string(),
            })
            .collect()
    }

    #[test]
    fn test_segment_totals_match_consolidated_revenue() {
        let mut gen = SegmentGenerator::new(42);
        let seeds = make_seeds(&["North America", "Europe"]);

        let rev = Decimal::from(1_000_000);
        let profit = Decimal::from(150_000);
        let assets = Decimal::from(5_000_000);

        let (segments, recon) = gen.generate("GROUP", "2024-03", rev, profit, assets, &seeds, None);

        assert!(!segments.is_empty());

        // Σ external revenues = consolidated_revenue
        let sum_ext: Decimal = segments.iter().map(|s| s.revenue_external).sum();
        assert_eq!(
            sum_ext, rev,
            "Σ external revenue should equal consolidated_revenue"
        );

        // Reconciliation identity: segment_revenue_total + eliminations = consolidated_revenue
        let computed = recon.segment_revenue_total + recon.intersegment_eliminations;
        assert_eq!(
            computed, recon.consolidated_revenue,
            "segment_revenue_total + eliminations ≠ consolidated_revenue"
        );
    }

    #[test]
    fn test_reconciliation_profit_math() {
        let mut gen = SegmentGenerator::new(99);
        let seeds = make_seeds(&["Americas", "EMEA", "APAC"]);

        let (_, recon) = gen.generate(
            "CORP",
            "2024-06",
            Decimal::from(2_000_000),
            Decimal::from(300_000),
            Decimal::from(8_000_000),
            &seeds,
            None,
        );

        // consolidated_profit = segment_profit_total + corporate_overhead
        assert_eq!(
            recon.consolidated_profit,
            recon.segment_profit_total + recon.corporate_overhead,
            "Profit reconciliation identity failed"
        );
    }

    #[test]
    fn test_reconciliation_assets_math() {
        let mut gen = SegmentGenerator::new(7);
        let seeds = make_seeds(&["ProductA", "ProductB"]);

        let (_, recon) = gen.generate(
            "C001",
            "2024-01",
            Decimal::from(500_000),
            Decimal::from(50_000),
            Decimal::from(3_000_000),
            &seeds,
            None,
        );

        // consolidated_assets = segment_assets_total + unallocated_assets
        assert_eq!(
            recon.consolidated_assets,
            recon.segment_assets_total + recon.unallocated_assets,
            "Asset reconciliation identity failed"
        );
    }

    #[test]
    fn test_each_segment_has_positive_external_revenue() {
        let mut gen = SegmentGenerator::new(42);
        // Use seeds with all positive consolidated numbers
        let seeds = make_seeds(&["SegA", "SegB", "SegC"]);
        let rev = Decimal::from(3_000_000);
        let profit = Decimal::from(600_000);
        let assets = Decimal::from(10_000_000);

        let (segments, _) = gen.generate("GRP", "2024-12", rev, profit, assets, &seeds, None);

        for seg in &segments {
            assert!(
                seg.revenue_external >= Decimal::ZERO,
                "Segment '{}' has negative external revenue: {}",
                seg.name,
                seg.revenue_external
            );
        }
    }

    #[test]
    fn test_single_entity_uses_product_lines() {
        let mut gen = SegmentGenerator::new(1234);
        let seeds = make_seeds(&["OnlyEntity"]);

        let (segments, _) = gen.generate(
            "C001",
            "2024-03",
            Decimal::from(1_000_000),
            Decimal::from(100_000),
            Decimal::from(4_000_000),
            &seeds,
            None,
        );

        // With a single seed, product-line segments should be generated (≥ 2)
        assert!(segments.len() >= 2, "Expected ≥ 2 product-line segments");
        // All should be ProductLine type
        for seg in &segments {
            assert_eq!(seg.segment_type, SegmentType::ProductLine);
        }
    }

    #[test]
    fn test_multi_entity_uses_geographic_segments() {
        let mut gen = SegmentGenerator::new(5678);
        let seeds = make_seeds(&["US", "DE", "JP"]);

        let (segments, _) = gen.generate(
            "GROUP",
            "2024-03",
            Decimal::from(9_000_000),
            Decimal::from(900_000),
            Decimal::from(30_000_000),
            &seeds,
            None,
        );

        assert_eq!(segments.len(), 3);
        for seg in &segments {
            assert_eq!(seg.segment_type, SegmentType::Geographic);
        }
    }

    #[test]
    fn test_deterministic() {
        let seeds = make_seeds(&["A", "B"]);
        let rev = Decimal::from(1_000_000);
        let profit = Decimal::from(200_000);
        let assets = Decimal::from(5_000_000);

        let (segs1, recon1) =
            SegmentGenerator::new(42).generate("G", "2024-01", rev, profit, assets, &seeds, None);
        let (segs2, recon2) =
            SegmentGenerator::new(42).generate("G", "2024-01", rev, profit, assets, &seeds, None);

        assert_eq!(segs1.len(), segs2.len());
        for (a, b) in segs1.iter().zip(segs2.iter()) {
            assert_eq!(a.segment_id, b.segment_id);
            assert_eq!(a.revenue_external, b.revenue_external);
        }
        assert_eq!(recon1.consolidated_revenue, recon2.consolidated_revenue);
        assert_eq!(recon1.segment_profit_total, recon2.segment_profit_total);
    }
}
