//! Multi-engagement portfolio simulation.
//!
//! Runs multiple audit engagements with shared resources, correlated findings,
//! and consolidated reporting.  The [`simulate_portfolio`] function accepts a
//! [`PortfolioConfig`] describing the engagements, a shared [`ResourcePool`],
//! and correlation parameters, then returns a [`PortfolioReport`] with per-
//! engagement summaries, scheduling conflicts, systemic findings, and a risk
//! heatmap.

use std::collections::HashMap;
use std::path::PathBuf;

use chrono::Datelike;
use datasynth_audit_fsm::context::EngagementContext;
use datasynth_audit_fsm::engine::AuditFsmEngine;
use datasynth_audit_fsm::error::AuditFsmError;
use datasynth_audit_fsm::loader::*;
use datasynth_audit_fsm::schema::GenerationOverlay;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Config types
// ---------------------------------------------------------------------------

/// Top-level configuration for a portfolio simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioConfig {
    /// Specifications for each engagement to simulate.
    pub engagements: Vec<EngagementSpec>,
    /// Shared resource pool across all engagements.
    pub shared_resources: ResourcePool,
    /// Correlation settings for cross-engagement finding propagation.
    pub correlation: CorrelationConfig,
}

/// Specification of a single engagement within the portfolio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementSpec {
    /// Identifier for the entity being audited.
    pub entity_id: String,
    /// Blueprint selector: `"fsa"`, `"ia"`, `"builtin:fsa"`, `"builtin:ia"`,
    /// or a file path.
    pub blueprint: String,
    /// Overlay selector: `"default"`, `"builtin:default"`, `"thorough"`,
    /// `"rushed"`, or a file path.
    pub overlay: String,
    /// Industry classification for cross-engagement correlation.
    pub industry: String,
    /// Risk profile of the entity.
    pub risk_profile: RiskProfile,
    /// Deterministic RNG seed for this engagement.
    pub seed: u64,
}

/// Risk profile for an entity.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskProfile {
    High,
    Medium,
    Low,
}

/// Pool of shared resources across all engagements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    /// Slots keyed by role name.
    pub roles: HashMap<String, ResourceSlot>,
}

/// A single resource slot (headcount + hours per person).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSlot {
    /// Number of people available in this role.
    pub count: usize,
    /// Annual hours available per person.
    pub hours_per_person: f64,
    /// Unavailable date ranges as `(start_date, end_date)` pairs (ISO 8601 strings).
    /// Each pair reduces available hours by the number of business days in the range
    /// multiplied by 8 hours per day per person.
    pub unavailable_periods: Vec<(String, String)>,
}

impl ResourceSlot {
    /// Compute effective hours per person after subtracting unavailable periods.
    pub fn effective_hours_per_person(&self) -> f64 {
        let unavailable_hours: f64 = self
            .unavailable_periods
            .iter()
            .map(|(start, end)| {
                let start_date = chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d");
                let end_date = chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d");
                match (start_date, end_date) {
                    (Ok(s), Ok(e)) => {
                        // Count business days in range.
                        let mut days = 0;
                        let mut d = s;
                        while d <= e {
                            let wd = d.weekday();
                            if wd != chrono::Weekday::Sat && wd != chrono::Weekday::Sun {
                                days += 1;
                            }
                            d += chrono::Duration::days(1);
                        }
                        days as f64 * 8.0 // 8 hours per business day
                    }
                    _ => 0.0,
                }
            })
            .sum();
        (self.hours_per_person - unavailable_hours).max(0.0)
    }
}

impl ResourcePool {
    /// Total hours available for a given role (accounting for unavailable periods).
    pub fn total_hours(&self, role: &str) -> f64 {
        self.roles
            .get(role)
            .map(|s| s.count as f64 * s.effective_hours_per_person())
            .unwrap_or(0.0)
    }
}

/// Correlation parameters governing how findings propagate across engagements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationConfig {
    /// Probability that a finding shared by 2+ entities in the same industry
    /// is flagged as systemic.
    pub systemic_finding_probability: f64,
    /// Strength of industry correlation (reserved for future use).
    pub industry_correlation: f64,
}

impl Default for CorrelationConfig {
    fn default() -> Self {
        Self {
            systemic_finding_probability: 0.3,
            industry_correlation: 0.5,
        }
    }
}

// ---------------------------------------------------------------------------
// Report types
// ---------------------------------------------------------------------------

/// Consolidated output of a portfolio simulation.
#[derive(Debug, Clone, Serialize)]
pub struct PortfolioReport {
    /// Per-engagement summaries.
    pub engagement_summaries: Vec<EngagementSummary>,
    /// Total hours consumed across all engagements.
    pub total_hours: f64,
    /// Total monetary cost across all engagements.
    pub total_cost: f64,
    /// Resource utilization ratio per role (required / available).
    pub resource_utilization: HashMap<String, f64>,
    /// Roles where demand exceeds supply.
    pub scheduling_conflicts: Vec<SchedulingConflict>,
    /// Findings that appear across multiple entities in the same industry.
    pub systemic_findings: Vec<SystemicFinding>,
    /// Per-entity risk heat-map entries.
    pub risk_heatmap: Vec<RiskHeatmapEntry>,
}

/// Summary of a single engagement within the portfolio.
#[derive(Debug, Clone, Serialize)]
pub struct EngagementSummary {
    /// Entity identifier.
    pub entity_id: String,
    /// Blueprint used.
    pub blueprint: String,
    /// Number of FSM events emitted.
    pub events: usize,
    /// Number of typed artifacts generated.
    pub artifacts: usize,
    /// Estimated hours consumed.
    pub hours: f64,
    /// Estimated monetary cost.
    pub cost: f64,
    /// Number of audit findings generated.
    pub findings_count: usize,
    /// Fraction of procedures that reached a terminal state.
    pub completion_rate: f64,
}

/// A scheduling conflict where demand for a role exceeds supply.
#[derive(Debug, Clone, Serialize)]
pub struct SchedulingConflict {
    /// Role that is over-subscribed.
    pub role: String,
    /// Total hours required across all engagements.
    pub required_hours: f64,
    /// Total hours available in the pool.
    pub available_hours: f64,
    /// Entity IDs affected by this conflict.
    pub engagements_affected: Vec<String>,
}

/// A finding that appears systemically across an industry.
#[derive(Debug, Clone, Serialize)]
pub struct SystemicFinding {
    /// Classification of the finding.
    pub finding_type: String,
    /// Industry in which the finding was observed.
    pub industry: String,
    /// Entities affected.
    pub affected_entities: Vec<String>,
}

/// A single entry in the risk heat-map.
#[derive(Debug, Clone, Serialize)]
pub struct RiskHeatmapEntry {
    /// Entity identifier.
    pub entity_id: String,
    /// Risk category (industry).
    pub category: String,
    /// Numeric risk score in [0, 1].
    pub score: f64,
}

// ---------------------------------------------------------------------------
// Blueprint / overlay resolution helpers
// ---------------------------------------------------------------------------

fn resolve_blueprint(name: &str) -> Result<BlueprintWithPreconditions, AuditFsmError> {
    match name {
        "fsa" | "builtin:fsa" => BlueprintWithPreconditions::load_builtin_fsa(),
        "ia" | "builtin:ia" => BlueprintWithPreconditions::load_builtin_ia(),
        path => BlueprintWithPreconditions::load_from_file(PathBuf::from(path)),
    }
}

fn resolve_overlay(name: &str) -> Result<GenerationOverlay, AuditFsmError> {
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
// Main simulation function
// ---------------------------------------------------------------------------

/// Run a portfolio simulation over all configured engagements.
///
/// Each engagement is executed sequentially with its own deterministic RNG seed.
/// After all engagements complete, scheduling conflicts are detected, systemic
/// findings are propagated, and a consolidated report is returned.
pub fn simulate_portfolio(config: &PortfolioConfig) -> Result<PortfolioReport, AuditFsmError> {
    let mut summaries = Vec::new();
    let mut total_role_hours: HashMap<String, f64> = HashMap::new();
    // industry -> [(entity_id, finding_type)]
    let mut findings_by_industry: HashMap<String, Vec<(String, String)>> = HashMap::new();

    // 1. Run each engagement.
    for spec in &config.engagements {
        let bwp = resolve_blueprint(&spec.blueprint)?;
        let overlay = resolve_overlay(&spec.overlay)?;
        let rng = ChaCha8Rng::seed_from_u64(spec.seed);
        let mut engine = AuditFsmEngine::new(bwp.clone(), overlay.clone(), rng);
        let ctx = EngagementContext::demo();

        let result = engine.run_engagement(&ctx)?;

        // Compute hours and cost from blueprint procedures.
        let mut eng_hours = 0.0;
        let mut eng_cost = 0.0;
        for phase in &bwp.blueprint.phases {
            for proc in &phase.procedures {
                if result.procedure_states.contains_key(&proc.id) {
                    let h = overlay.resource_costs.effective_hours(proc);
                    eng_hours += h;
                    eng_cost += overlay.resource_costs.procedure_cost(proc);
                    // Track per-role hours.
                    let role = proc
                        .required_roles
                        .first()
                        .map(|r| r.as_str())
                        .unwrap_or("audit_staff");
                    *total_role_hours.entry(role.to_string()).or_default() += h;
                }
            }
        }

        // Track findings for cross-engagement correlation.
        let findings_count = result.artifacts.findings.len();
        if findings_count > 0 {
            // Extract actual finding types from the generated findings,
            // deduplicating per-entity to avoid inflating the count.
            let mut seen_types = std::collections::HashSet::new();
            for finding in &result.artifacts.findings {
                let finding_type = format!("{:?}", finding.finding_type)
                    .to_lowercase()
                    .replace(' ', "_");
                if seen_types.insert(finding_type.clone()) {
                    findings_by_industry
                        .entry(spec.industry.clone())
                        .or_default()
                        .push((spec.entity_id.clone(), finding_type));
                }
            }
        }

        let completed = result
            .procedure_states
            .values()
            .filter(|s| s.as_str() == "completed" || s.as_str() == "closed")
            .count();
        let total_procs = result.procedure_states.len();

        summaries.push(EngagementSummary {
            entity_id: spec.entity_id.clone(),
            blueprint: spec.blueprint.clone(),
            events: result.event_log.len(),
            artifacts: result.artifacts.total_artifacts(),
            hours: eng_hours,
            cost: eng_cost,
            findings_count,
            completion_rate: if total_procs > 0 {
                completed as f64 / total_procs as f64
            } else {
                0.0
            },
        });
    }

    // 2. Detect scheduling conflicts.
    let mut conflicts = Vec::new();
    for (role, required) in &total_role_hours {
        let available = config.shared_resources.total_hours(role);
        if available > 0.0 && *required > available {
            conflicts.push(SchedulingConflict {
                role: role.clone(),
                required_hours: *required,
                available_hours: available,
                engagements_affected: summaries.iter().map(|s| s.entity_id.clone()).collect(),
            });
        }
    }

    // 3. Propagate systemic findings.
    let mut systemic = Vec::new();
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    for (industry, findings) in &findings_by_industry {
        if findings.len() >= 2 {
            let roll: f64 = rand::Rng::random(&mut rng);
            if roll < config.correlation.systemic_finding_probability {
                systemic.push(SystemicFinding {
                    finding_type: "systemic_control_deficiency".to_string(),
                    industry: industry.clone(),
                    affected_entities: findings.iter().map(|(e, _)| e.clone()).collect(),
                });
            }
        }
    }

    // 4. Build risk heat-map.
    let mut heatmap = Vec::new();
    for spec in &config.engagements {
        let risk_score = match spec.risk_profile {
            RiskProfile::High => 0.9,
            RiskProfile::Medium => 0.5,
            RiskProfile::Low => 0.2,
        };
        heatmap.push(RiskHeatmapEntry {
            entity_id: spec.entity_id.clone(),
            category: spec.industry.clone(),
            score: risk_score,
        });
    }

    // 5. Resource utilization.
    let mut utilization = HashMap::new();
    for (role, required) in &total_role_hours {
        let available = config.shared_resources.total_hours(role);
        if available > 0.0 {
            utilization.insert(role.clone(), *required / available);
        }
    }

    let total_hours = summaries.iter().map(|s| s.hours).sum();
    let total_cost = summaries.iter().map(|s| s.cost).sum();

    Ok(PortfolioReport {
        engagement_summaries: summaries,
        total_hours,
        total_cost,
        resource_utilization: utilization,
        scheduling_conflicts: conflicts,
        systemic_findings: systemic,
        risk_heatmap: heatmap,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn default_pool() -> ResourcePool {
        let mut roles = HashMap::new();
        roles.insert(
            "engagement_partner".into(),
            ResourceSlot {
                count: 2,
                hours_per_person: 2000.0,
                unavailable_periods: vec![],
            },
        );
        roles.insert(
            "audit_manager".into(),
            ResourceSlot {
                count: 3,
                hours_per_person: 1800.0,
                unavailable_periods: vec![],
            },
        );
        roles.insert(
            "audit_senior".into(),
            ResourceSlot {
                count: 5,
                hours_per_person: 1600.0,
                unavailable_periods: vec![],
            },
        );
        roles.insert(
            "audit_staff".into(),
            ResourceSlot {
                count: 8,
                hours_per_person: 1600.0,
                unavailable_periods: vec![],
            },
        );
        ResourcePool { roles }
    }

    fn fsa_spec(entity: &str, seed: u64) -> EngagementSpec {
        EngagementSpec {
            entity_id: entity.into(),
            blueprint: "fsa".into(),
            overlay: "default".into(),
            industry: "financial_services".into(),
            risk_profile: RiskProfile::Medium,
            seed,
        }
    }

    #[test]
    fn test_single_engagement_portfolio() {
        let config = PortfolioConfig {
            engagements: vec![fsa_spec("ENTITY_A", 42)],
            shared_resources: default_pool(),
            correlation: CorrelationConfig::default(),
        };
        let report = simulate_portfolio(&config).unwrap();
        assert_eq!(report.engagement_summaries.len(), 1);
        let summary = &report.engagement_summaries[0];
        assert_eq!(summary.entity_id, "ENTITY_A");
        assert_eq!(summary.blueprint, "fsa");
        assert!(summary.events > 0);
        assert!(summary.hours > 0.0);
        assert!(summary.cost > 0.0);
        assert!(report.total_hours > 0.0);
        assert!(report.total_cost > 0.0);
    }

    #[test]
    fn test_two_fsa_engagements() {
        let config = PortfolioConfig {
            engagements: vec![fsa_spec("ENTITY_A", 42), fsa_spec("ENTITY_B", 99)],
            shared_resources: default_pool(),
            correlation: CorrelationConfig::default(),
        };
        let report = simulate_portfolio(&config).unwrap();
        assert_eq!(report.engagement_summaries.len(), 2);
        let ids: Vec<&str> = report
            .engagement_summaries
            .iter()
            .map(|s| s.entity_id.as_str())
            .collect();
        assert!(ids.contains(&"ENTITY_A"));
        assert!(ids.contains(&"ENTITY_B"));
        // Total hours should be the sum of individual hours.
        let sum_hours: f64 = report.engagement_summaries.iter().map(|s| s.hours).sum();
        assert!((report.total_hours - sum_hours).abs() < 0.01);
    }

    #[test]
    fn test_mixed_fsa_ia_portfolio() {
        let config = PortfolioConfig {
            engagements: vec![
                fsa_spec("FSA_ENTITY", 42),
                EngagementSpec {
                    entity_id: "IA_ENTITY".into(),
                    blueprint: "ia".into(),
                    overlay: "default".into(),
                    industry: "manufacturing".into(),
                    risk_profile: RiskProfile::High,
                    seed: 77,
                },
            ],
            shared_resources: default_pool(),
            correlation: CorrelationConfig::default(),
        };
        let report = simulate_portfolio(&config).unwrap();
        assert_eq!(report.engagement_summaries.len(), 2);
        let blueprints: Vec<&str> = report
            .engagement_summaries
            .iter()
            .map(|s| s.blueprint.as_str())
            .collect();
        assert!(blueprints.contains(&"fsa"));
        assert!(blueprints.contains(&"ia"));
    }

    #[test]
    fn test_resource_utilization_computed() {
        let config = PortfolioConfig {
            engagements: vec![fsa_spec("ENTITY_A", 42)],
            shared_resources: default_pool(),
            correlation: CorrelationConfig::default(),
        };
        let report = simulate_portfolio(&config).unwrap();
        // At least one role should have non-zero utilization.
        assert!(
            !report.resource_utilization.is_empty(),
            "expected non-empty resource utilization"
        );
        for (_role, util) in &report.resource_utilization {
            assert!(*util > 0.0, "utilization should be positive");
        }
    }

    #[test]
    fn test_portfolio_deterministic() {
        let config = PortfolioConfig {
            engagements: vec![fsa_spec("ENTITY_A", 42), fsa_spec("ENTITY_B", 99)],
            shared_resources: default_pool(),
            correlation: CorrelationConfig::default(),
        };
        let report1 = simulate_portfolio(&config).unwrap();
        let report2 = simulate_portfolio(&config).unwrap();

        assert_eq!(
            report1.engagement_summaries.len(),
            report2.engagement_summaries.len()
        );
        for (s1, s2) in report1
            .engagement_summaries
            .iter()
            .zip(report2.engagement_summaries.iter())
        {
            assert_eq!(s1.entity_id, s2.entity_id);
            assert_eq!(s1.events, s2.events);
            assert!((s1.hours - s2.hours).abs() < 0.01);
            assert!((s1.cost - s2.cost).abs() < 0.01);
            assert_eq!(s1.findings_count, s2.findings_count);
        }
        assert!((report1.total_hours - report2.total_hours).abs() < 0.01);
        assert!((report1.total_cost - report2.total_cost).abs() < 0.01);
    }

    #[test]
    fn test_risk_heatmap_populated() {
        let config = PortfolioConfig {
            engagements: vec![fsa_spec("ENTITY_A", 42), fsa_spec("ENTITY_B", 99)],
            shared_resources: default_pool(),
            correlation: CorrelationConfig::default(),
        };
        let report = simulate_portfolio(&config).unwrap();
        assert_eq!(
            report.risk_heatmap.len(),
            config.engagements.len(),
            "heatmap entries should match engagement count"
        );
        for entry in &report.risk_heatmap {
            assert!(
                entry.score > 0.0 && entry.score <= 1.0,
                "risk score should be in (0, 1]"
            );
        }
    }

    #[test]
    fn test_portfolio_report_serializes() {
        let config = PortfolioConfig {
            engagements: vec![fsa_spec("ENTITY_A", 42)],
            shared_resources: default_pool(),
            correlation: CorrelationConfig::default(),
        };
        let report = simulate_portfolio(&config).unwrap();
        let json = serde_json::to_string_pretty(&report).unwrap();
        assert!(json.contains("ENTITY_A"));
        assert!(json.contains("total_hours"));
        assert!(json.contains("risk_heatmap"));
        // Roundtrip: deserialize back.
        let _parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_unavailable_periods_reduce_hours() {
        let slot = ResourceSlot {
            count: 1,
            hours_per_person: 2000.0,
            // 5 business days (Mon-Fri) of unavailability = 40 hours.
            unavailable_periods: vec![("2025-01-06".to_string(), "2025-01-10".to_string())],
        };
        let effective = slot.effective_hours_per_person();
        assert!(
            (effective - 1960.0).abs() < 0.01,
            "Expected 1960.0 effective hours (2000 - 5*8), got {}",
            effective
        );
    }

    #[test]
    fn test_unavailable_periods_weekend_excluded() {
        let slot = ResourceSlot {
            count: 1,
            hours_per_person: 2000.0,
            // Range includes weekend: 2025-01-10 (Fri) through 2025-01-12 (Sun) = 1 business day
            unavailable_periods: vec![("2025-01-10".to_string(), "2025-01-12".to_string())],
        };
        let effective = slot.effective_hours_per_person();
        assert!(
            (effective - 1992.0).abs() < 0.01,
            "Expected 1992.0 effective hours (2000 - 1*8), got {}",
            effective
        );
    }

    #[test]
    fn test_pool_total_hours_with_unavailability() {
        let mut roles = HashMap::new();
        roles.insert(
            "audit_staff".into(),
            ResourceSlot {
                count: 2,
                hours_per_person: 1600.0,
                // 10 business days per person
                unavailable_periods: vec![("2025-01-06".to_string(), "2025-01-17".to_string())],
            },
        );
        let pool = ResourcePool { roles };
        // 10 business days * 8h = 80h subtracted per person; 2 people.
        let total = pool.total_hours("audit_staff");
        let expected = 2.0 * (1600.0 - 80.0);
        assert!(
            (total - expected).abs() < 0.01,
            "Expected {expected}, got {total}"
        );
    }
}
