//! Integration tests for the IFRS 8 / ASC 280 segment reporting generator.

use datasynth_generators::{SegmentGenerator, SegmentSeed};
use rust_decimal::Decimal;

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

// ============================================================================
// Revenue reconciliation
// ============================================================================

#[test]
fn segment_external_rev_sums_to_consolidated() {
    let mut gen = SegmentGenerator::new(42);
    let seeds = make_seeds(&["North America", "Europe", "APAC"]);
    let rev = Decimal::from(10_000_000);
    let profit = Decimal::from(1_500_000);
    let assets = Decimal::from(40_000_000);

    let (segments, recon) = gen.generate("GROUP", "2024-03", rev, profit, assets, &seeds);

    // Sum of external revenues must equal the consolidated revenue passed in
    let sum_ext: Decimal = segments.iter().map(|s| s.revenue_external).sum();
    assert_eq!(
        sum_ext, rev,
        "Σ external revenue ({sum_ext}) ≠ consolidated_revenue ({rev})"
    );

    // Reconciliation: segment_revenue_total + eliminations = consolidated_revenue
    let recon_computed = recon.segment_revenue_total + recon.intersegment_eliminations;
    assert_eq!(
        recon_computed, recon.consolidated_revenue,
        "segment_revenue_total ({}) + eliminations ({}) ≠ consolidated_revenue ({})",
        recon.segment_revenue_total, recon.intersegment_eliminations, recon.consolidated_revenue
    );

    // consolidated_revenue on the reconciliation must match what we passed in
    assert_eq!(recon.consolidated_revenue, rev);
}

// ============================================================================
// Profit reconciliation
// ============================================================================

#[test]
fn segment_profit_reconciliation_identity() {
    let mut gen = SegmentGenerator::new(77);
    let seeds = make_seeds(&["Consumer", "Enterprise"]);
    let rev = Decimal::from(5_000_000);
    let profit = Decimal::from(750_000);
    let assets = Decimal::from(15_000_000);

    let (_, recon) = gen.generate("CORP", "2024-06", rev, profit, assets, &seeds);

    // consolidated_profit = segment_profit_total + corporate_overhead
    assert_eq!(
        recon.consolidated_profit,
        recon.segment_profit_total + recon.corporate_overhead,
        "Profit reconciliation identity failed: {} ≠ {} + {}",
        recon.consolidated_profit,
        recon.segment_profit_total,
        recon.corporate_overhead
    );

    // Corporate overhead should be non-positive (it is a cost centre)
    assert!(
        recon.corporate_overhead <= Decimal::ZERO,
        "corporate_overhead should be ≤ 0, got {}",
        recon.corporate_overhead
    );
}

// ============================================================================
// Asset reconciliation
// ============================================================================

#[test]
fn segment_asset_reconciliation_identity() {
    let mut gen = SegmentGenerator::new(13);
    let seeds = make_seeds(&["Hardware", "Software", "Services"]);
    let rev = Decimal::from(8_000_000);
    let profit = Decimal::from(1_200_000);
    let assets = Decimal::from(25_000_000);

    let (_, recon) = gen.generate("C001", "2024-12", rev, profit, assets, &seeds);

    // consolidated_assets = segment_assets_total + unallocated_assets
    assert_eq!(
        recon.consolidated_assets,
        recon.segment_assets_total + recon.unallocated_assets,
        "Asset reconciliation identity failed: {} ≠ {} + {}",
        recon.consolidated_assets,
        recon.segment_assets_total,
        recon.unallocated_assets
    );

    // Unallocated assets should be non-negative
    assert!(
        recon.unallocated_assets >= Decimal::ZERO,
        "unallocated_assets should be ≥ 0, got {}",
        recon.unallocated_assets
    );
}

// ============================================================================
// Each segment must have non-negative external revenue
// ============================================================================

#[test]
fn each_segment_has_nonnegative_external_revenue() {
    let mut gen = SegmentGenerator::new(99);
    let seeds = make_seeds(&["SegA", "SegB", "SegC", "SegD"]);
    let rev = Decimal::from(20_000_000);
    let profit = Decimal::from(3_000_000);
    let assets = Decimal::from(60_000_000);

    let (segments, _) = gen.generate("GRP", "2024-09", rev, profit, assets, &seeds);

    assert!(
        !segments.is_empty(),
        "Should have produced at least one segment"
    );

    for seg in &segments {
        assert!(
            seg.revenue_external >= Decimal::ZERO,
            "Segment '{}' has negative external revenue: {}",
            seg.name,
            seg.revenue_external
        );
    }
}

// ============================================================================
// Single-entity → product line segments
// ============================================================================

#[test]
fn single_entity_generates_product_line_segments() {
    use datasynth_core::models::SegmentType;

    let mut gen = SegmentGenerator::new(1000);
    let seeds = make_seeds(&["AcmeCorp"]);
    let rev = Decimal::from(2_000_000);
    let profit = Decimal::from(250_000);
    let assets = Decimal::from(8_000_000);

    let (segments, _) = gen.generate("C001", "2024-03", rev, profit, assets, &seeds);

    // Single seed → product-line segments (≥ 2)
    assert!(
        segments.len() >= 2,
        "Expected ≥ 2 product-line segments, got {}",
        segments.len()
    );

    for seg in &segments {
        assert_eq!(
            seg.segment_type,
            SegmentType::ProductLine,
            "Expected ProductLine segment type"
        );
    }
}

// ============================================================================
// Multi-entity → geographic segments
// ============================================================================

#[test]
fn multi_entity_generates_geographic_segments() {
    use datasynth_core::models::SegmentType;

    let mut gen = SegmentGenerator::new(2000);
    let seeds = make_seeds(&["US", "DE", "SG"]);
    let rev = Decimal::from(15_000_000);
    let profit = Decimal::from(2_250_000);
    let assets = Decimal::from(50_000_000);

    let (segments, _) = gen.generate("GROUP", "2024-06", rev, profit, assets, &seeds);

    assert_eq!(
        segments.len(),
        3,
        "Expected 3 geographic segments (one per entity)"
    );

    for seg in &segments {
        assert_eq!(
            seg.segment_type,
            SegmentType::Geographic,
            "Expected Geographic segment type"
        );
    }
}

// ============================================================================
// Determinism
// ============================================================================

#[test]
fn segment_generation_is_deterministic() {
    let seeds = make_seeds(&["Alpha", "Beta"]);
    let rev = Decimal::from(5_000_000);
    let profit = Decimal::from(500_000);
    let assets = Decimal::from(20_000_000);

    let (segs1, recon1) =
        SegmentGenerator::new(42).generate("G", "2024-01", rev, profit, assets, &seeds);
    let (segs2, recon2) =
        SegmentGenerator::new(42).generate("G", "2024-01", rev, profit, assets, &seeds);

    assert_eq!(segs1.len(), segs2.len(), "Segment count should be the same");

    for (a, b) in segs1.iter().zip(segs2.iter()) {
        assert_eq!(
            a.segment_id, b.segment_id,
            "segment_id must be deterministic"
        );
        assert_eq!(
            a.revenue_external, b.revenue_external,
            "revenue_external must be deterministic"
        );
        assert_eq!(
            a.total_assets, b.total_assets,
            "total_assets must be deterministic"
        );
    }

    assert_eq!(
        recon1.segment_revenue_total, recon2.segment_revenue_total,
        "Reconciliation revenue total must be deterministic"
    );
    assert_eq!(
        recon1.consolidated_profit, recon2.consolidated_profit,
        "Reconciliation profit must be deterministic"
    );
}

// ============================================================================
// Segment IDs are unique within a single generate() call
// ============================================================================

#[test]
fn segment_ids_are_unique() {
    let mut gen = SegmentGenerator::new(55);
    let seeds = make_seeds(&["X", "Y", "Z"]);

    let (segments, _) = gen.generate(
        "G",
        "2024-01",
        Decimal::from(3_000_000),
        Decimal::from(300_000),
        Decimal::from(10_000_000),
        &seeds,
    );

    let mut ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for seg in &segments {
        assert!(
            ids.insert(seg.segment_id.clone()),
            "Duplicate segment_id found: {}",
            seg.segment_id
        );
    }
}

// ============================================================================
// Period label is preserved on all outputs
// ============================================================================

#[test]
fn period_label_propagated_correctly() {
    let mut gen = SegmentGenerator::new(7);
    let seeds = make_seeds(&["EU", "US"]);
    let period = "2025-06";

    let (segments, recon) = gen.generate(
        "GRP",
        period,
        Decimal::from(1_000_000),
        Decimal::from(100_000),
        Decimal::from(5_000_000),
        &seeds,
    );

    for seg in &segments {
        assert_eq!(seg.period, period, "Segment period label mismatch");
        assert_eq!(seg.company_code, "GRP", "Company code mismatch");
    }
    assert_eq!(recon.period, period, "Reconciliation period label mismatch");
    assert_eq!(
        recon.company_code, "GRP",
        "Reconciliation company code mismatch"
    );
}
