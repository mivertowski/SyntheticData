//! Service organization and SOC report generator per ISA 402.
//!
//! Generates 1–3 service organizations per entity and produces SOC 1 Type II
//! reports with 3–8 control objectives and 0–2 exceptions per report.
//! User entity controls are generated mapping back to SOC objectives.

use chrono::{Duration, NaiveDate};
use datasynth_core::models::audit::service_organization::{
    ControlEffectiveness, ControlObjective, ServiceOrganization, ServiceType, SocException,
    SocOpinionType, SocReport, SocReportType, UserEntityControl,
};
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

/// Configuration for service organization generation.
#[derive(Debug, Clone)]
pub struct ServiceOrgGeneratorConfig {
    /// Number of service organizations per entity (min, max)
    pub service_orgs_per_entity: (usize, usize),
    /// Number of control objectives per SOC report (min, max)
    pub objectives_per_report: (usize, usize),
    /// Number of exceptions per report (min, max)
    pub exceptions_per_report: (usize, usize),
    /// Probability of a qualified opinion (vs unmodified)
    pub qualified_opinion_probability: f64,
    /// Number of user entity controls per SOC report (min, max)
    pub user_controls_per_report: (usize, usize),
}

impl Default for ServiceOrgGeneratorConfig {
    fn default() -> Self {
        Self {
            service_orgs_per_entity: (1, 3),
            objectives_per_report: (3, 8),
            exceptions_per_report: (0, 2),
            qualified_opinion_probability: 0.10,
            user_controls_per_report: (1, 4),
        }
    }
}

/// Result of generating service organization data for a set of entities.
#[derive(Debug, Clone, Default)]
pub struct ServiceOrgSnapshot {
    /// Service organizations identified
    pub service_organizations: Vec<ServiceOrganization>,
    /// SOC reports obtained
    pub soc_reports: Vec<SocReport>,
    /// User entity controls documented
    pub user_entity_controls: Vec<UserEntityControl>,
}

/// Generator for ISA 402 service organization controls.
pub struct ServiceOrgGenerator {
    rng: ChaCha8Rng,
    config: ServiceOrgGeneratorConfig,
}

impl ServiceOrgGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0x402),
            config: ServiceOrgGeneratorConfig::default(),
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: ServiceOrgGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0x402),
            config,
        }
    }

    /// Generate service organizations and SOC reports for a list of entities.
    pub fn generate(
        &mut self,
        entity_codes: &[String],
        period_end_date: NaiveDate,
    ) -> ServiceOrgSnapshot {
        if entity_codes.is_empty() {
            return ServiceOrgSnapshot::default();
        }

        let mut snapshot = ServiceOrgSnapshot::default();

        // Pool of service type templates to draw from
        let service_type_pool = [
            ServiceType::PayrollProcessor,
            ServiceType::CloudHosting,
            ServiceType::PaymentProcessor,
            ServiceType::ItManagedServices,
            ServiceType::DataCentre,
        ];

        for entity_code in entity_codes {
            let org_count = self.rng.random_range(
                self.config.service_orgs_per_entity.0..=self.config.service_orgs_per_entity.1,
            );

            for i in 0..org_count {
                let service_type = service_type_pool[i % service_type_pool.len()];
                let org_name = self.org_name(service_type, i);

                // Check if a matching service org already exists (reuse across entities)
                let org_id = if let Some(existing) = snapshot
                    .service_organizations
                    .iter_mut()
                    .find(|o| o.service_type == service_type && o.name == org_name)
                {
                    existing.entities_served.push(entity_code.clone());
                    existing.id.clone()
                } else {
                    let org = ServiceOrganization::new(
                        org_name,
                        service_type,
                        vec![entity_code.clone()],
                    );
                    let id = org.id.clone();
                    snapshot.service_organizations.push(org);
                    id
                };

                // Generate a SOC 1 Type II report for this org/entity pair
                let report = self.generate_soc_report(&org_id, period_end_date);
                let report_id = report.id.clone();
                let objective_ids: Vec<String> = report
                    .control_objectives
                    .iter()
                    .map(|o| o.id.clone())
                    .collect();
                snapshot.soc_reports.push(report);

                // Generate user entity controls for the report
                let user_controls =
                    self.generate_user_controls(&report_id, &objective_ids, entity_code);
                snapshot.user_entity_controls.extend(user_controls);
            }
        }

        snapshot
    }

    fn generate_soc_report(&mut self, service_org_id: &str, period_end_date: NaiveDate) -> SocReport {
        let objectives_count = self.rng.random_range(
            self.config.objectives_per_report.0..=self.config.objectives_per_report.1,
        );
        let exceptions_count = self.rng.random_range(
            self.config.exceptions_per_report.0..=self.config.exceptions_per_report.1,
        );

        let has_exceptions = exceptions_count > 0;
        let opinion_type = if has_exceptions
            && self.rng.random::<f64>() < self.config.qualified_opinion_probability
        {
            SocOpinionType::Qualified
        } else {
            SocOpinionType::Unmodified
        };

        // SOC report covers the 12 months ending at period-end
        let report_period_start = period_end_date - Duration::days(365);
        let report_period_end = period_end_date;

        let mut report = SocReport::new(
            service_org_id,
            SocReportType::Soc1Type2,
            report_period_start,
            report_period_end,
            opinion_type,
        );

        // Generate control objectives
        for j in 0..objectives_count {
            let controls_tested = self.rng.random_range(3u32..=12);
            // Objectives with exceptions may have ineffective controls
            let controls_effective = !(has_exceptions && j < exceptions_count);
            let description = self.objective_description(j);
            let objective = ControlObjective::new(description, controls_tested, controls_effective);
            report.control_objectives.push(objective);
        }

        // Generate exceptions for objectives that have failures
        let ineffective_objectives: Vec<String> = report
            .control_objectives
            .iter()
            .filter(|o| !o.controls_effective)
            .map(|o| o.id.clone())
            .collect();

        for obj_id in &ineffective_objectives {
            let exception = SocException {
                control_objective_id: obj_id.clone(),
                description: "A sample of transactions tested revealed that the control did not \
                               operate as designed during the period."
                    .to_string(),
                management_response: "Management has implemented enhanced monitoring procedures \
                                      to address the identified control deficiency."
                    .to_string(),
                user_entity_impact: "User entities should consider compensating controls to \
                                     address the risk arising from this exception."
                    .to_string(),
            };
            report.exceptions_noted.push(exception);
        }

        report
    }

    fn generate_user_controls(
        &mut self,
        soc_report_id: &str,
        objective_ids: &[String],
        _entity_code: &str,
    ) -> Vec<UserEntityControl> {
        if objective_ids.is_empty() {
            return Vec::new();
        }

        let count = self.rng.random_range(
            self.config.user_controls_per_report.0..=self.config.user_controls_per_report.1,
        );

        let mut controls = Vec::with_capacity(count);
        for i in 0..count {
            let mapped_objective = &objective_ids[i % objective_ids.len()];
            let implemented = self.rng.random::<f64>() < 0.90;
            let effectiveness = if implemented {
                if self.rng.random::<f64>() < 0.80 {
                    ControlEffectiveness::Effective
                } else {
                    ControlEffectiveness::EffectiveWithExceptions
                }
            } else {
                ControlEffectiveness::NotTested
            };

            let description = self.user_control_description(i);
            let control = UserEntityControl::new(
                soc_report_id,
                description,
                mapped_objective,
                implemented,
                effectiveness,
            );
            controls.push(control);
        }

        controls
    }

    fn org_name(&self, service_type: ServiceType, index: usize) -> String {
        let names_by_type: &[&str] = match service_type {
            ServiceType::PayrollProcessor => &[
                "Ceridian HCM Inc.",
                "ADP Employer Services",
                "Paychex Inc.",
                "Workday Payroll Ltd.",
            ],
            ServiceType::CloudHosting => &[
                "Amazon Web Services Inc.",
                "Microsoft Azure Cloud",
                "Google Cloud Platform",
                "IBM Cloud Services",
            ],
            ServiceType::PaymentProcessor => &[
                "Stripe Inc.",
                "PayPal Holdings Inc.",
                "Worldpay Group Ltd.",
                "Adyen N.V.",
            ],
            ServiceType::ItManagedServices => &[
                "DXC Technology Co.",
                "Unisys Corporation",
                "Cognizant IT Solutions",
                "Infosys BPM Ltd.",
            ],
            ServiceType::DataCentre => &[
                "Equinix Inc.",
                "Digital Realty Trust",
                "CyrusOne LLC",
                "Iron Mountain Data Centres",
            ],
        };
        names_by_type[index % names_by_type.len()].to_string()
    }

    fn objective_description(&self, index: usize) -> String {
        let objectives = [
            "Logical access controls over applications and data are designed and operating effectively.",
            "Change management procedures ensure that programme changes are authorised, tested, and approved.",
            "Computer operations controls ensure that processing is complete, accurate, and timely.",
            "Data backup and recovery controls ensure data integrity and availability.",
            "Network and security controls protect systems from unauthorised access.",
            "Incident management controls ensure that security incidents are identified and resolved.",
            "Vendor management controls ensure that third-party risks are assessed and monitored.",
            "Physical security controls restrict access to data processing facilities.",
        ];
        objectives[index % objectives.len()].to_string()
    }

    fn user_control_description(&self, index: usize) -> String {
        let descriptions = [
            "Review of user access rights at least annually and removal of access for terminated employees.",
            "Reconciliation of payroll data transmitted to the service organization and results received.",
            "Monitoring of service organization performance metrics and escalation of issues.",
            "Review and approval of changes to master data transmitted to the service organization.",
            "Periodic review of SOC reports and assessment of exceptions on user entity operations.",
        ];
        descriptions[index % descriptions.len()].to_string()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn period_end() -> NaiveDate {
        NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()
    }

    fn entity_codes(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("C{i:03}")).collect()
    }

    #[test]
    fn test_service_orgs_within_bounds() {
        let mut gen = ServiceOrgGenerator::new(42);
        let snapshot = gen.generate(&entity_codes(1), period_end());
        assert!(
            snapshot.service_organizations.len() >= 1
                && snapshot.service_organizations.len() <= 3,
            "expected 1-3 service orgs, got {}",
            snapshot.service_organizations.len()
        );
    }

    #[test]
    fn test_soc_reports_have_objectives_in_range() {
        let mut gen = ServiceOrgGenerator::new(42);
        let snapshot = gen.generate(&entity_codes(2), period_end());
        for report in &snapshot.soc_reports {
            assert!(
                report.control_objectives.len() >= 3 && report.control_objectives.len() <= 8,
                "expected 3-8 control objectives, got {}",
                report.control_objectives.len()
            );
        }
    }

    #[test]
    fn test_exceptions_within_bounds() {
        let mut gen = ServiceOrgGenerator::new(42);
        let snapshot = gen.generate(&entity_codes(3), period_end());
        for report in &snapshot.soc_reports {
            assert!(
                report.exceptions_noted.len() <= 2,
                "expected 0-2 exceptions, got {}",
                report.exceptions_noted.len()
            );
        }
    }

    #[test]
    fn test_user_entity_controls_reference_valid_reports() {
        use std::collections::HashSet;
        let mut gen = ServiceOrgGenerator::new(42);
        let snapshot = gen.generate(&entity_codes(2), period_end());

        let report_ids: HashSet<String> =
            snapshot.soc_reports.iter().map(|r| r.id.clone()).collect();

        for ctrl in &snapshot.user_entity_controls {
            assert!(
                report_ids.contains(&ctrl.soc_report_id),
                "UserEntityControl references unknown soc_report_id '{}'",
                ctrl.soc_report_id
            );
        }
    }

    #[test]
    fn test_empty_entities_returns_empty_snapshot() {
        let mut gen = ServiceOrgGenerator::new(42);
        let snapshot = gen.generate(&[], period_end());
        assert!(snapshot.service_organizations.is_empty());
        assert!(snapshot.soc_reports.is_empty());
        assert!(snapshot.user_entity_controls.is_empty());
    }
}
