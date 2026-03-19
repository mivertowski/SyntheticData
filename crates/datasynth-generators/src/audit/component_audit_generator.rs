//! ISA 600 Component Audit Generator.
//!
//! Generates group audit artefacts following ISA 600 (Special Considerations —
//! Audits of Group Financial Statements):
//!
//! - Component auditor records (one per jurisdiction)
//! - Group audit plan with materiality allocations
//! - Component instructions (one per entity)
//! - Component auditor reports (one per entity, including misstatements)

use std::collections::HashMap;

use chrono::{Duration, NaiveDate};
use datasynth_config::schema::CompanyConfig;
use datasynth_core::models::audit::component_audit::{
    AllocationBasis, CompetenceLevel, ComponentAuditSnapshot, ComponentAuditor,
    ComponentAuditorReport, ComponentInstruction, ComponentMaterialityAllocation, ComponentScope,
    GroupAuditPlan, GroupRiskLevel, Misstatement, MisstatementType,
};
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

/// Generates ISA 600 group audit artefacts.
pub struct ComponentAuditGenerator {
    rng: ChaCha8Rng,
}

impl ComponentAuditGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0x600),
        }
    }

    /// Generate a full `ComponentAuditSnapshot` for a group of companies.
    ///
    /// # Arguments
    /// * `companies` – all companies in the configuration
    /// * `group_materiality` – group-level materiality from the engagement
    /// * `engagement_id` – ID of the parent audit engagement
    /// * `period_end` – period end date (component deadlines are derived from this)
    pub fn generate(
        &mut self,
        companies: &[CompanyConfig],
        group_materiality: Decimal,
        engagement_id: &str,
        period_end: NaiveDate,
    ) -> ComponentAuditSnapshot {
        if companies.is_empty() {
            return ComponentAuditSnapshot::default();
        }

        // ----------------------------------------------------------------
        // 1. Group companies by country → one auditor per jurisdiction
        // ----------------------------------------------------------------
        // Map: country → list of entity codes
        let mut jurisdiction_map: HashMap<String, Vec<String>> = HashMap::new();
        for company in companies {
            jurisdiction_map
                .entry(company.country.clone())
                .or_default()
                .push(company.code.clone());
        }

        // Build a stable, sorted list of jurisdictions for deterministic output.
        let mut jurisdictions: Vec<String> = jurisdiction_map.keys().cloned().collect();
        jurisdictions.sort();

        // ----------------------------------------------------------------
        // 2. Create one ComponentAuditor per jurisdiction
        // ----------------------------------------------------------------
        let mut auditor_id_counter: u32 = 0;
        // Map: country → auditor_id (for instruction assignment)
        let mut country_to_auditor_id: HashMap<String, String> = HashMap::new();
        let mut component_auditors: Vec<ComponentAuditor> = Vec::new();

        for country in &jurisdictions {
            auditor_id_counter += 1;
            let auditor_id = format!("CA-{country}-{auditor_id_counter:04}");

            let firm_name = format!("Audit Firm {country}");

            // Competence: 90% satisfactory, 8% requires supervision, 2% unsatisfactory
            let competence = {
                let r: f64 = self.rng.random();
                if r < 0.90 {
                    CompetenceLevel::Satisfactory
                } else if r < 0.98 {
                    CompetenceLevel::RequiresSupervision
                } else {
                    CompetenceLevel::Unsatisfactory
                }
            };

            let assigned_entities = jurisdiction_map.get(country).cloned().unwrap_or_default();

            country_to_auditor_id.insert(country.clone(), auditor_id.clone());

            component_auditors.push(ComponentAuditor {
                id: auditor_id,
                firm_name,
                jurisdiction: country.clone(),
                independence_confirmed: self.rng.random::<f64>() > 0.02, // 98% confirmed
                competence_assessment: competence,
                assigned_entities,
            });
        }

        // ----------------------------------------------------------------
        // 3. Determine entity weights (proxy: index + 1 to avoid zero)
        // ----------------------------------------------------------------
        // Weight for company[i] = (n - i) so the first companies are "larger"
        let n = companies.len();
        let weights: Vec<f64> = (0..n).map(|i| (n - i) as f64).collect();
        let total_weight: f64 = weights.iter().sum();

        // ----------------------------------------------------------------
        // 4. Build materiality allocations per entity
        // ----------------------------------------------------------------
        let mut component_allocations: Vec<ComponentMaterialityAllocation> = Vec::new();
        let mut significant_components: Vec<String> = Vec::new();
        let group_mat_f64 = group_materiality
            .to_string()
            .parse::<f64>()
            .unwrap_or(1_000_000.0);

        for (i, company) in companies.iter().enumerate() {
            let entity_share = weights[i] / total_weight;

            // component_materiality = group_materiality * entity_share * 0.75
            let cm_f64 = group_mat_f64 * entity_share * 0.75;
            let component_materiality =
                Decimal::from_f64_retain(cm_f64).unwrap_or(Decimal::new(100_000, 2));
            let clearly_trivial =
                Decimal::from_f64_retain(cm_f64 * 0.05).unwrap_or(Decimal::new(5_000, 2));

            let allocation_basis = if entity_share >= 0.05 {
                // Both significant (≥15%) and mid-size (5–15%) entities use
                // revenue-proportional allocation; small entities use risk-based.
                AllocationBasis::RevenueProportional
            } else {
                AllocationBasis::RiskBased
            };

            if entity_share >= 0.15 {
                significant_components.push(company.code.clone());
            }

            component_allocations.push(ComponentMaterialityAllocation {
                entity_code: company.code.clone(),
                component_materiality,
                clearly_trivial,
                allocation_basis,
            });
        }

        // ----------------------------------------------------------------
        // 5. Aggregation risk — driven by number of components
        // ----------------------------------------------------------------
        let aggregation_risk = if n <= 2 {
            GroupRiskLevel::Low
        } else if n <= 5 {
            GroupRiskLevel::Medium
        } else {
            GroupRiskLevel::High
        };

        // ----------------------------------------------------------------
        // 6. Build GroupAuditPlan
        // ----------------------------------------------------------------
        let consolidation_procedures = vec![
            "Review intercompany eliminations for completeness".to_string(),
            "Agree component trial balances to consolidation working papers".to_string(),
            "Test goodwill impairment at group level".to_string(),
            "Review consolidation journal entries for unusual items".to_string(),
            "Assess appropriateness of accounting policies across components".to_string(),
        ];

        let group_audit_plan = GroupAuditPlan {
            engagement_id: engagement_id.to_string(),
            group_materiality,
            component_allocations: component_allocations.clone(),
            aggregation_risk,
            significant_components: significant_components.clone(),
            consolidation_audit_procedures: consolidation_procedures,
        };

        // ----------------------------------------------------------------
        // 7. Build ComponentInstruction per entity
        // ----------------------------------------------------------------
        let reporting_deadline = period_end + Duration::days(60);
        let mut instruction_id_counter: u32 = 0;
        let mut instructions: Vec<ComponentInstruction> = Vec::new();

        for (i, company) in companies.iter().enumerate() {
            instruction_id_counter += 1;
            let entity_share = weights[i] / total_weight;
            let auditor_id = company_to_auditor_id(&company.country, &country_to_auditor_id);

            let alloc = &component_allocations[i];

            // Scope determined by entity share
            let scope = if entity_share >= 0.15 {
                ComponentScope::FullScope
            } else if entity_share >= 0.05 {
                ComponentScope::SpecificScope {
                    account_areas: vec![
                        "Revenue".to_string(),
                        "Receivables".to_string(),
                        "Inventory".to_string(),
                    ],
                }
            } else {
                ComponentScope::AnalyticalOnly
            };

            let specific_procedures = self.build_procedures(&scope, company);
            let areas_of_focus = self.build_areas_of_focus(&scope);

            instructions.push(ComponentInstruction {
                id: format!("CI-{instruction_id_counter:06}"),
                component_auditor_id: auditor_id,
                entity_code: company.code.clone(),
                scope,
                materiality_allocated: alloc.component_materiality,
                reporting_deadline,
                specific_procedures,
                areas_of_focus,
            });
        }

        // ----------------------------------------------------------------
        // 8. Build ComponentAuditorReport per entity
        // ----------------------------------------------------------------
        let mut report_id_counter: u32 = 0;
        let mut reports: Vec<ComponentAuditorReport> = Vec::new();

        for (i, company) in companies.iter().enumerate() {
            report_id_counter += 1;
            let entity_share = weights[i] / total_weight;
            let instruction = &instructions[i];
            let alloc = &component_allocations[i];
            let auditor_id = company_to_auditor_id(&company.country, &country_to_auditor_id);

            // Number of misstatements: 0-3, proportional to entity size
            let max_misstatements = if entity_share >= 0.15 {
                3usize
            } else if entity_share >= 0.05 {
                2
            } else {
                1
            };
            let misstatement_count = self.rng.random_range(0..=max_misstatements);

            let mut misstatements: Vec<Misstatement> = Vec::new();
            for _ in 0..misstatement_count {
                misstatements.push(self.generate_misstatement(alloc.component_materiality));
            }

            // Scope limitations: rare (5% chance)
            let scope_limitations: Vec<String> = if self.rng.random::<f64>() < 0.05 {
                vec!["Limited access to subsidiary records for inventory count".to_string()]
            } else {
                vec![]
            };

            // Significant findings
            let significant_findings: Vec<String> = misstatements
                .iter()
                .filter(|m| !m.corrected)
                .map(|m| {
                    format!(
                        "{}: {} {} ({})",
                        m.account_area,
                        m.description,
                        m.amount,
                        format!("{:?}", m.classification).to_lowercase()
                    )
                })
                .collect();

            let conclusion = if misstatements.iter().all(|m| m.corrected)
                && scope_limitations.is_empty()
            {
                format!(
                    "No uncorrected misstatements identified in {} that exceed component materiality.",
                    company.name
                )
            } else {
                format!(
                    "Uncorrected misstatements or limitations noted in {}. See significant findings.",
                    company.name
                )
            };

            reports.push(ComponentAuditorReport {
                id: format!("CR-{report_id_counter:06}"),
                instruction_id: instruction.id.clone(),
                component_auditor_id: auditor_id,
                entity_code: company.code.clone(),
                misstatements_identified: misstatements,
                scope_limitations,
                significant_findings,
                conclusion,
            });
        }

        ComponentAuditSnapshot {
            component_auditors,
            group_audit_plan: Some(group_audit_plan),
            component_instructions: instructions,
            component_reports: reports,
        }
    }

    // ----------------------------------------------------------------
    // Internal helpers
    // ----------------------------------------------------------------

    fn generate_misstatement(&mut self, component_materiality: Decimal) -> Misstatement {
        let account_areas = [
            "Revenue",
            "Receivables",
            "Inventory",
            "Fixed Assets",
            "Payables",
            "Accruals",
            "Provisions",
        ];
        let area_idx = self.rng.random_range(0..account_areas.len());
        let area = account_areas[area_idx].to_string();

        let types = [
            MisstatementType::Factual,
            MisstatementType::Judgmental,
            MisstatementType::Projected,
        ];
        let type_idx = self.rng.random_range(0..types.len());
        let classification = types[type_idx].clone();

        // Amount: 1% – 80% of component materiality (random)
        let cm_f64 = component_materiality
            .to_string()
            .parse::<f64>()
            .unwrap_or(100_000.0);
        let pct: f64 = self.rng.random_range(0.01..=0.80);
        let amount = Decimal::from_f64_retain(cm_f64 * pct).unwrap_or(Decimal::new(1_000, 0));

        let corrected = self.rng.random::<f64>() > 0.40; // 60% corrected

        let description = match &classification {
            MisstatementType::Factual => format!("Factual misstatement in {area}"),
            MisstatementType::Judgmental => format!("Judgmental difference in {area} estimate"),
            MisstatementType::Projected => format!("Projected error in {area} population"),
        };

        Misstatement {
            description,
            amount,
            classification,
            account_area: area,
            corrected,
        }
    }

    fn build_procedures(&mut self, scope: &ComponentScope, company: &CompanyConfig) -> Vec<String> {
        match scope {
            ComponentScope::FullScope => vec![
                format!(
                    "Perform full audit of {} financial statements",
                    company.name
                ),
                "Test internal controls over financial reporting".to_string(),
                "Perform substantive testing on all material account balances".to_string(),
                "Attend physical inventory count".to_string(),
                "Confirm significant balances with third parties".to_string(),
                "Review subsequent events through reporting deadline".to_string(),
            ],
            ComponentScope::SpecificScope { account_areas } => {
                let mut procs =
                    vec!["Perform substantive procedures on specified account areas".to_string()];
                for area in account_areas {
                    procs.push(format!("Obtain audit evidence for {area} balance"));
                }
                procs
            }
            ComponentScope::LimitedProcedures => vec![
                "Perform agreed-upon procedures as specified in instruction".to_string(),
                "Report all factual findings without expressing an opinion".to_string(),
            ],
            ComponentScope::AnalyticalOnly => vec![
                "Perform analytical procedures on key account balances".to_string(),
                "Investigate significant fluctuations exceeding component materiality".to_string(),
                "Obtain management explanations for unusual movements".to_string(),
            ],
        }
    }

    fn build_areas_of_focus(&self, scope: &ComponentScope) -> Vec<String> {
        match scope {
            ComponentScope::FullScope => vec![
                "Revenue recognition".to_string(),
                "Going concern assessment".to_string(),
                "Related party transactions".to_string(),
                "Significant estimates and judgments".to_string(),
            ],
            ComponentScope::SpecificScope { account_areas } => account_areas.clone(),
            ComponentScope::LimitedProcedures => vec!["As agreed in instruction".to_string()],
            ComponentScope::AnalyticalOnly => vec![
                "Year-on-year variance analysis".to_string(),
                "Budget vs actual comparison".to_string(),
            ],
        }
    }
}

/// Look up the auditor ID for a given country, falling back to a generic ID.
fn company_to_auditor_id(country: &str, country_to_auditor_id: &HashMap<String, String>) -> String {
    country_to_auditor_id
        .get(country)
        .cloned()
        .unwrap_or_else(|| format!("CA-{country}-0001"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_config::schema::{CompanyConfig, TransactionVolume};

    fn make_company(code: &str, name: &str, country: &str) -> CompanyConfig {
        CompanyConfig {
            code: code.to_string(),
            name: name.to_string(),
            currency: "USD".to_string(),
            functional_currency: None,
            country: country.to_string(),
            fiscal_year_variant: "K4".to_string(),
            annual_transaction_volume: TransactionVolume::TenK,
            volume_weight: 1.0,
        }
    }

    #[test]
    fn test_single_entity_produces_one_auditor_instruction_report() {
        let companies = vec![make_company("C001", "Alpha Inc", "US")];
        let mut gen = ComponentAuditGenerator::new(42);
        let period_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        let group_mat = Decimal::new(1_000_000, 0);

        let snapshot = gen.generate(&companies, group_mat, "ENG-001", period_end);

        assert_eq!(
            snapshot.component_auditors.len(),
            1,
            "one auditor per jurisdiction"
        );
        assert_eq!(
            snapshot.component_instructions.len(),
            1,
            "one instruction per entity"
        );
        assert_eq!(snapshot.component_reports.len(), 1, "one report per entity");
        assert!(
            snapshot.group_audit_plan.is_some(),
            "group plan should be present"
        );
    }

    #[test]
    fn test_multi_entity_two_jurisdictions_two_auditors() {
        let companies = vec![
            make_company("C001", "Alpha Inc", "US"),
            make_company("C002", "Beta GmbH", "DE"),
            make_company("C003", "Gamma LLC", "US"),
        ];
        let mut gen = ComponentAuditGenerator::new(42);
        let period_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        let group_mat = Decimal::new(5_000_000, 0);

        let snapshot = gen.generate(&companies, group_mat, "ENG-002", period_end);

        assert_eq!(
            snapshot.component_auditors.len(),
            2,
            "US and DE → 2 auditors"
        );
        assert_eq!(snapshot.component_instructions.len(), 3, "one per entity");
        assert_eq!(snapshot.component_reports.len(), 3, "one per entity");
    }

    #[test]
    fn test_scope_thresholds_with_large_group() {
        // 7 equal-weight companies → each has 1/7 ≈ 14.3% share → SpecificScope or AnalyticalOnly
        // To get FullScope, we need ≥ 15% → need fewer or heavier first entity.
        // With n=2 companies: first has weight 2/3 ≈ 66.7% → FullScope
        //                     second has weight 1/3 ≈ 33.3% → FullScope
        let companies = vec![
            make_company("C001", "BigCo", "US"),
            make_company("C002", "SmallCo", "US"),
        ];
        let mut gen = ComponentAuditGenerator::new(42);
        let period_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        let group_mat = Decimal::new(10_000_000, 0);

        let snapshot = gen.generate(&companies, group_mat, "ENG-003", period_end);

        // With 2 companies: weights = [2, 1], total = 3
        // C001 share = 2/3 ≈ 66.7% → FullScope + significant
        // C002 share = 1/3 ≈ 33.3% → FullScope + significant
        let plan = snapshot.group_audit_plan.as_ref().unwrap();
        assert!(plan.significant_components.contains(&"C001".to_string()));
        assert!(plan.significant_components.contains(&"C002".to_string()));

        let c001_inst = snapshot
            .component_instructions
            .iter()
            .find(|i| i.entity_code == "C001")
            .unwrap();
        assert_eq!(c001_inst.scope, ComponentScope::FullScope);
    }

    #[test]
    fn test_scope_analytical_only_for_small_entity() {
        // 10 equal-ish companies: last entity has smallest weight (1/55 ≈ 1.8%) → AnalyticalOnly
        let companies: Vec<CompanyConfig> = (1..=10)
            .map(|i| make_company(&format!("C{i:03}"), &format!("Company {i}"), "US"))
            .collect();
        let mut gen = ComponentAuditGenerator::new(42);
        let period_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        let group_mat = Decimal::new(10_000_000, 0);

        let snapshot = gen.generate(&companies, group_mat, "ENG-004", period_end);

        // Last company (C010) has weight 1 out of total 55 → share ≈ 1.8% → AnalyticalOnly
        let last_inst = snapshot
            .component_instructions
            .iter()
            .find(|i| i.entity_code == "C010")
            .unwrap();
        assert_eq!(last_inst.scope, ComponentScope::AnalyticalOnly);
    }

    #[test]
    fn test_sum_of_component_materialities_le_group_materiality() {
        let companies: Vec<CompanyConfig> = (1..=5)
            .map(|i| make_company(&format!("C{i:03}"), &format!("Firm {i}"), "US"))
            .collect();
        let mut gen = ComponentAuditGenerator::new(99);
        let period_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        let group_mat = Decimal::new(2_000_000, 0);

        let snapshot = gen.generate(&companies, group_mat, "ENG-005", period_end);

        let plan = snapshot.group_audit_plan.as_ref().unwrap();
        let total_component_mat: Decimal = plan
            .component_allocations
            .iter()
            .map(|a| a.component_materiality)
            .sum();

        assert!(
            total_component_mat <= group_mat,
            "sum of component mats {total_component_mat} should be <= group mat {group_mat}"
        );
    }

    #[test]
    fn test_all_entities_covered_by_exactly_one_instruction() {
        let companies = vec![
            make_company("C001", "Alpha", "US"),
            make_company("C002", "Beta", "DE"),
            make_company("C003", "Gamma", "FR"),
        ];
        let mut gen = ComponentAuditGenerator::new(7);
        let period_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        let group_mat = Decimal::new(3_000_000, 0);

        let snapshot = gen.generate(&companies, group_mat, "ENG-006", period_end);

        for company in &companies {
            let count = snapshot
                .component_instructions
                .iter()
                .filter(|i| i.entity_code == company.code)
                .count();
            assert_eq!(
                count, 1,
                "entity {} should have exactly 1 instruction",
                company.code
            );
        }
    }

    #[test]
    fn test_all_reports_reference_valid_instruction_ids() {
        let companies = vec![
            make_company("C001", "Alpha", "US"),
            make_company("C002", "Beta", "GB"),
        ];
        let mut gen = ComponentAuditGenerator::new(123);
        let period_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        let group_mat = Decimal::new(1_500_000, 0);

        let snapshot = gen.generate(&companies, group_mat, "ENG-007", period_end);

        let instruction_ids: std::collections::HashSet<String> = snapshot
            .component_instructions
            .iter()
            .map(|i| i.id.clone())
            .collect();

        for report in &snapshot.component_reports {
            assert!(
                instruction_ids.contains(&report.instruction_id),
                "report {} references unknown instruction {}",
                report.id,
                report.instruction_id
            );
        }
    }
}
