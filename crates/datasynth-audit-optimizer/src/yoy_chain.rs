//! Year-over-year engagement chains.
//!
//! Simulates sequential audit engagements for the same entity across multiple
//! fiscal years, carrying forward a configurable fraction of prior-year findings
//! and tracking trends in duration and finding counts.

use std::path::PathBuf;

use datasynth_audit_fsm::context::EngagementContext;
use datasynth_audit_fsm::engine::AuditFsmEngine;
use datasynth_audit_fsm::error::AuditFsmError;
use datasynth_audit_fsm::loader::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Configuration for a year-over-year engagement chain.
#[derive(Debug, Clone)]
pub struct YoyChainConfig {
    /// Identifier for the entity being audited.
    pub entity_id: String,
    /// Blueprint selector (e.g. `"fsa"`, `"ia"`, or a file path).
    pub blueprint: String,
    /// Overlay selector (e.g. `"default"`, `"thorough"`, or a file path).
    pub overlay: String,
    /// Number of years to simulate (2..=10).
    pub years: usize,
    /// Base RNG seed; year `i` uses `base_seed + i as u64`.
    pub base_seed: u64,
    /// Fraction of findings that persist year-over-year (0.0..=1.0, default 0.3).
    pub finding_carry_rate: f64,
}

impl Default for YoyChainConfig {
    fn default() -> Self {
        Self {
            entity_id: "ENTITY_01".into(),
            blueprint: "fsa".into(),
            overlay: "default".into(),
            years: 3,
            base_seed: 42,
            finding_carry_rate: 0.3,
        }
    }
}

// ---------------------------------------------------------------------------
// Report types
// ---------------------------------------------------------------------------

/// Per-year result within a YoY chain.
#[derive(Debug, Clone, Serialize)]
pub struct YoyEngagementResult {
    /// Fiscal year simulated.
    pub year: i32,
    /// Total FSM events emitted.
    pub events: usize,
    /// Total typed artifacts generated.
    pub artifacts: usize,
    /// Findings generated anew during this year.
    pub findings_new: usize,
    /// Findings carried forward from the prior year.
    pub findings_carried: usize,
    /// Fraction of procedures that reached a terminal state.
    pub completion_rate: f64,
    /// Engagement duration in hours.
    pub duration_hours: f64,
}

/// Consolidated report for an entire YoY chain.
#[derive(Debug, Clone, Serialize)]
pub struct YoyChainReport {
    /// Entity identifier.
    pub entity_id: String,
    /// Per-year results in chronological order.
    pub years: Vec<YoyEngagementResult>,
    /// Finding trend: (year, total_findings) per year.
    pub finding_trend: Vec<(i32, usize)>,
    /// Duration trend: (year, hours) per year.
    pub duration_trend: Vec<(i32, f64)>,
}

// ---------------------------------------------------------------------------
// Blueprint / overlay resolution (shared pattern with portfolio)
// ---------------------------------------------------------------------------

fn resolve_blueprint(name: &str) -> Result<BlueprintWithPreconditions, AuditFsmError> {
    match name {
        "fsa" | "builtin:fsa" => BlueprintWithPreconditions::load_builtin_fsa(),
        "ia" | "builtin:ia" => BlueprintWithPreconditions::load_builtin_ia(),
        "kpmg" | "builtin:kpmg" => BlueprintWithPreconditions::load_builtin_kpmg(),
        "pwc" | "builtin:pwc" => BlueprintWithPreconditions::load_builtin_pwc(),
        "deloitte" | "builtin:deloitte" => BlueprintWithPreconditions::load_builtin_deloitte(),
        "ey_gam_lite" | "builtin:ey_gam_lite" => {
            BlueprintWithPreconditions::load_builtin_ey_gam_lite()
        }
        path => BlueprintWithPreconditions::load_from_file(PathBuf::from(path)),
    }
}

fn resolve_overlay(
    name: &str,
) -> Result<datasynth_audit_fsm::schema::GenerationOverlay, AuditFsmError> {
    match name {
        "default" | "builtin:default" => {
            load_overlay(&OverlaySource::Builtin(BuiltinOverlay::Default))
        }
        "thorough" | "builtin:thorough" => {
            load_overlay(&OverlaySource::Builtin(BuiltinOverlay::Thorough))
        }
        "rushed" | "builtin:rushed" => {
            load_overlay(&OverlaySource::Builtin(BuiltinOverlay::Rushed))
        }
        "retail" | "builtin:retail" => {
            load_overlay(&OverlaySource::Builtin(BuiltinOverlay::IndustryRetail))
        }
        "manufacturing" | "builtin:manufacturing" => load_overlay(&OverlaySource::Builtin(
            BuiltinOverlay::IndustryManufacturing,
        )),
        "financial_services" | "builtin:financial_services" => load_overlay(
            &OverlaySource::Builtin(BuiltinOverlay::IndustryFinancialServices),
        ),
        path => load_overlay(&OverlaySource::Custom(PathBuf::from(path))),
    }
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/// Run a year-over-year engagement chain.
///
/// For each simulated year the engine runs a fresh engagement with a
/// deterministically-derived seed.  A configurable fraction of the prior
/// year's findings are carried forward as "anomaly context", increasing
/// the carried-finding count for the subsequent year.
///
/// # Errors
///
/// Returns an error if the blueprint or overlay cannot be loaded, or if
/// the engine fails for any simulated year.
pub fn run_yoy_chain(config: &YoyChainConfig) -> Result<YoyChainReport, AuditFsmError> {
    assert!(
        config.years >= 2 && config.years <= 10,
        "years must be in 2..=10, got {}",
        config.years
    );

    let bwp = resolve_blueprint(&config.blueprint)?;
    let overlay = resolve_overlay(&config.overlay)?;

    let base_year = 2025_i32.saturating_sub(config.years as i32);
    let mut year_results = Vec::with_capacity(config.years);
    let mut prior_findings: usize = 0;

    for i in 0..config.years {
        let year = base_year + i as i32 + 1;
        let seed = config.base_seed.wrapping_add(i as u64);
        let rng = ChaCha8Rng::seed_from_u64(seed);

        let mut engine = AuditFsmEngine::new(bwp.clone(), overlay.clone(), rng);

        // Build context with the appropriate fiscal year.
        let mut ctx = EngagementContext::test_default();
        ctx.fiscal_year = year;
        ctx.company_code = config.entity_id.clone();

        // Carry forward anomaly refs from prior year (simulating persistent findings).
        let carried = ((prior_findings as f64) * config.finding_carry_rate).round() as usize;
        if carried > 0 {
            ctx.anomaly_refs = (0..carried)
                .map(|j| format!("CARRY-{}-{:03}", year - 1, j + 1))
                .collect();
        }

        let result = engine.run_engagement(&ctx)?;

        let new_findings = result.artifacts.findings.len();
        let total_procs = result.procedure_states.len();
        let completed = result
            .procedure_states
            .values()
            .filter(|s| s.as_str() == "completed" || s.as_str() == "closed")
            .count();

        year_results.push(YoyEngagementResult {
            year,
            events: result.event_log.len(),
            artifacts: result.artifacts.total_artifacts(),
            findings_new: new_findings,
            findings_carried: carried,
            completion_rate: if total_procs > 0 {
                completed as f64 / total_procs as f64
            } else {
                0.0
            },
            duration_hours: result.total_duration_hours,
        });

        // The total findings for carry-forward purposes is new + carried.
        prior_findings = new_findings + carried;
    }

    let finding_trend: Vec<(i32, usize)> = year_results
        .iter()
        .map(|r| (r.year, r.findings_new + r.findings_carried))
        .collect();

    let duration_trend: Vec<(i32, f64)> = year_results
        .iter()
        .map(|r| (r.year, r.duration_hours))
        .collect();

    Ok(YoyChainReport {
        entity_id: config.entity_id.clone(),
        years: year_results,
        finding_trend,
        duration_trend,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_produces_n_years() {
        let config = YoyChainConfig {
            entity_id: "TEST_ENTITY".into(),
            blueprint: "fsa".into(),
            overlay: "default".into(),
            years: 3,
            base_seed: 42,
            finding_carry_rate: 0.3,
        };
        let report = run_yoy_chain(&config).unwrap();

        assert_eq!(report.years.len(), 3, "expected 3 year results");
        assert_eq!(report.entity_id, "TEST_ENTITY");
        assert_eq!(report.finding_trend.len(), 3);
        assert_eq!(report.duration_trend.len(), 3);

        // Years should be sequential.
        for window in report.years.windows(2) {
            assert_eq!(
                window[1].year,
                window[0].year + 1,
                "years should be sequential"
            );
        }

        // Each year should have produced events and artifacts.
        for yr in &report.years {
            assert!(yr.events > 0, "year {} should have events", yr.year);
            assert!(yr.artifacts > 0, "year {} should have artifacts", yr.year);
        }
    }

    #[test]
    fn test_findings_carry_forward() {
        let config = YoyChainConfig {
            entity_id: "CARRY_TEST".into(),
            blueprint: "fsa".into(),
            overlay: "default".into(),
            years: 4,
            base_seed: 99,
            finding_carry_rate: 1.0, // 100% carry to make effect visible
        };
        let report = run_yoy_chain(&config).unwrap();

        // First year cannot have carried findings.
        assert_eq!(
            report.years[0].findings_carried, 0,
            "year 1 should have 0 carried findings"
        );

        // If year 1 produced any findings, subsequent years should carry them.
        let y1_new = report.years[0].findings_new;
        if y1_new > 0 {
            assert!(
                report.years[1].findings_carried > 0,
                "with 100% carry rate and {} new findings in year 1, year 2 should carry some",
                y1_new
            );
        }
    }

    #[test]
    fn test_trends_are_computed() {
        let config = YoyChainConfig {
            entity_id: "TREND_TEST".into(),
            blueprint: "fsa".into(),
            overlay: "default".into(),
            years: 2,
            base_seed: 77,
            finding_carry_rate: 0.5,
        };
        let report = run_yoy_chain(&config).unwrap();

        // Finding trend entries should match year results.
        assert_eq!(report.finding_trend.len(), report.years.len());
        for (i, (year, total)) in report.finding_trend.iter().enumerate() {
            assert_eq!(*year, report.years[i].year);
            assert_eq!(
                *total,
                report.years[i].findings_new + report.years[i].findings_carried
            );
        }

        // Duration trend should match.
        assert_eq!(report.duration_trend.len(), report.years.len());
        for (i, (year, hours)) in report.duration_trend.iter().enumerate() {
            assert_eq!(*year, report.years[i].year);
            assert!(
                (*hours - report.years[i].duration_hours).abs() < 0.001,
                "duration mismatch for year {}",
                year
            );
        }
    }
}
