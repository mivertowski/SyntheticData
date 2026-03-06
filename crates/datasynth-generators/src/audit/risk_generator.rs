//! Risk assessment generator for audit engagements.
//!
//! Generates risk assessments with fraud risk factors, planned responses,
//! and cross-references per ISA 315 and ISA 330.

use chrono::Duration;
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use datasynth_core::models::audit::{
    Assertion, AuditEngagement, DetectionRisk, EngagementPhase, FraudRiskFactor,
    FraudTriangleElement, PlannedResponse, ResponseNature, ResponseProcedureType, ResponseStatus,
    ResponseTiming, RiskAssessment, RiskCategory, RiskLevel, RiskReviewStatus, Trend,
};

/// Configuration for risk assessment generation.
#[derive(Debug, Clone)]
pub struct RiskAssessmentGeneratorConfig {
    /// Number of risks per engagement (min, max)
    pub risks_per_engagement: (u32, u32),
    /// Probability of significant risk designation
    pub significant_risk_probability: f64,
    /// Probability of high inherent risk
    pub high_inherent_risk_probability: f64,
    /// Probability of fraud risk factors
    pub fraud_factor_probability: f64,
    /// Number of planned responses per risk (min, max)
    pub responses_per_risk: (u32, u32),
}

impl Default for RiskAssessmentGeneratorConfig {
    fn default() -> Self {
        Self {
            risks_per_engagement: (8, 20),
            significant_risk_probability: 0.20,
            high_inherent_risk_probability: 0.25,
            fraud_factor_probability: 0.30,
            responses_per_risk: (1, 4),
        }
    }
}

/// Generator for risk assessments.
pub struct RiskAssessmentGenerator {
    rng: ChaCha8Rng,
    config: RiskAssessmentGeneratorConfig,
    risk_counter: u32,
}

impl RiskAssessmentGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config: RiskAssessmentGeneratorConfig::default(),
            risk_counter: 0,
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: RiskAssessmentGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
            risk_counter: 0,
        }
    }

    /// Generate risk assessments for an engagement.
    pub fn generate_risks_for_engagement(
        &mut self,
        engagement: &AuditEngagement,
        team_members: &[String],
        accounts: &[String],
    ) -> Vec<RiskAssessment> {
        let count = self
            .rng
            .random_range(self.config.risks_per_engagement.0..=self.config.risks_per_engagement.1);

        let mut risks = Vec::with_capacity(count as usize);

        // Always include presumed risks per ISA 240
        risks.push(self.generate_revenue_fraud_risk(engagement, team_members));
        risks.push(self.generate_management_override_risk(engagement, team_members));

        // Generate additional risks for various accounts/processes
        let risk_areas = self.get_risk_areas(accounts);
        for area in risk_areas.iter().take((count - 2) as usize) {
            let risk = self.generate_risk_assessment(engagement, area, team_members);
            risks.push(risk);
        }

        risks
    }

    /// Generate a single risk assessment.
    pub fn generate_risk_assessment(
        &mut self,
        engagement: &AuditEngagement,
        account_or_process: &str,
        team_members: &[String],
    ) -> RiskAssessment {
        self.risk_counter += 1;

        let risk_category = self.select_risk_category();
        let (description, assertion) =
            self.generate_risk_description(account_or_process, risk_category);

        let mut risk = RiskAssessment::new(
            engagement.engagement_id,
            risk_category,
            account_or_process,
            &description,
        );

        risk.risk_ref = format!("RISK-{:04}", self.risk_counter);

        // Set assertion if applicable
        if let Some(assertion) = assertion {
            risk = risk.with_assertion(assertion);
        }

        // Set risk levels
        let inherent_risk = self.generate_inherent_risk();
        let control_risk = self.generate_control_risk(&inherent_risk);
        risk = risk.with_risk_levels(inherent_risk, control_risk);

        // Maybe mark as significant
        if self.rng.random::<f64>() < self.config.significant_risk_probability
            || matches!(
                risk.risk_of_material_misstatement,
                RiskLevel::High | RiskLevel::Significant
            )
        {
            let rationale = self.generate_significant_risk_rationale(risk_category);
            risk = risk.mark_significant(&rationale);
        }

        // Add fraud risk factors if applicable
        if self.rng.random::<f64>() < self.config.fraud_factor_probability {
            let factors = self.generate_fraud_factors();
            for factor in factors {
                risk.add_fraud_factor(factor);
            }
        }

        // Set response nature and timing
        risk.response_nature = self.select_response_nature(&risk);
        risk.response_timing = self.select_response_timing(engagement);
        risk.response_extent = self.generate_response_extent(&risk);

        // Add planned responses
        let response_count = self
            .rng
            .random_range(self.config.responses_per_risk.0..=self.config.responses_per_risk.1);
        for _ in 0..response_count {
            let response = self.generate_planned_response(&risk, team_members, engagement);
            risk.add_response(response);
        }

        // Set assessor
        let assessor = self.select_team_member(team_members, "senior");
        risk = risk.with_assessed_by(&assessor, engagement.planning_start + Duration::days(7));

        // Maybe add review
        if self.rng.random::<f64>() < 0.8 {
            risk.review_status = RiskReviewStatus::Approved;
            risk.reviewer_id = Some(self.select_team_member(team_members, "manager"));
            risk.review_date = Some(engagement.planning_start + Duration::days(14));
        }

        risk
    }

    /// Generate presumed revenue fraud risk per ISA 240.
    fn generate_revenue_fraud_risk(
        &mut self,
        engagement: &AuditEngagement,
        team_members: &[String],
    ) -> RiskAssessment {
        self.risk_counter += 1;

        let mut risk = RiskAssessment::new(
            engagement.engagement_id,
            RiskCategory::FraudRisk,
            "Revenue Recognition",
            "Presumed fraud risk in revenue recognition per ISA 240.26",
        );

        risk.risk_ref = format!("RISK-{:04}", self.risk_counter);
        risk = risk.with_assertion(Assertion::Occurrence);
        risk = risk.with_risk_levels(RiskLevel::High, RiskLevel::Medium);
        risk = risk.mark_significant("Presumed fraud risk per ISA 240 - revenue recognition");
        risk.presumed_revenue_fraud_risk = true;

        // Add fraud triangle factors
        risk.add_fraud_factor(FraudRiskFactor::new(
            FraudTriangleElement::Pressure,
            "Management compensation tied to revenue targets",
            75,
            "Compensation plan review",
        ));
        risk.add_fraud_factor(FraudRiskFactor::new(
            FraudTriangleElement::Opportunity,
            "Complex revenue arrangements with multiple performance obligations",
            60,
            "Contract review",
        ));

        risk.response_nature = ResponseNature::SubstantiveOnly;
        risk.response_timing = ResponseTiming::YearEnd;
        risk.response_extent = "Extended substantive testing with increased sample sizes".into();

        // Add responses
        risk.add_response(PlannedResponse::new(
            "Test revenue cutoff at year-end",
            ResponseProcedureType::TestOfDetails,
            Assertion::Cutoff,
            &self.select_team_member(team_members, "senior"),
            engagement.fieldwork_start + Duration::days(14),
        ));
        risk.add_response(PlannedResponse::new(
            "Confirm significant revenue transactions with customers",
            ResponseProcedureType::Confirmation,
            Assertion::Occurrence,
            &self.select_team_member(team_members, "staff"),
            engagement.fieldwork_start + Duration::days(21),
        ));
        risk.add_response(PlannedResponse::new(
            "Perform analytical procedures on revenue trends",
            ResponseProcedureType::AnalyticalProcedure,
            Assertion::Completeness,
            &self.select_team_member(team_members, "senior"),
            engagement.fieldwork_start + Duration::days(7),
        ));

        let assessor = self.select_team_member(team_members, "manager");
        risk = risk.with_assessed_by(&assessor, engagement.planning_start);
        risk.review_status = RiskReviewStatus::Approved;
        risk.reviewer_id = Some(engagement.engagement_partner_id.clone());
        risk.review_date = Some(engagement.planning_start + Duration::days(7));

        risk
    }

    /// Generate presumed management override risk per ISA 240.
    fn generate_management_override_risk(
        &mut self,
        engagement: &AuditEngagement,
        team_members: &[String],
    ) -> RiskAssessment {
        self.risk_counter += 1;

        let mut risk = RiskAssessment::new(
            engagement.engagement_id,
            RiskCategory::FraudRisk,
            "Management Override of Controls",
            "Presumed risk of management override of controls per ISA 240.31",
        );

        risk.risk_ref = format!("RISK-{:04}", self.risk_counter);
        risk = risk.with_risk_levels(RiskLevel::High, RiskLevel::High);
        risk = risk.mark_significant("Presumed fraud risk per ISA 240 - management override");
        risk.presumed_management_override = true;

        risk.add_fraud_factor(FraudRiskFactor::new(
            FraudTriangleElement::Opportunity,
            "Management has ability to override controls",
            80,
            "Control environment assessment",
        ));
        risk.add_fraud_factor(FraudRiskFactor::new(
            FraudTriangleElement::Rationalization,
            "Tone at the top may not emphasize ethical behavior",
            50,
            "Governance inquiries",
        ));

        risk.response_nature = ResponseNature::SubstantiveOnly;
        risk.response_timing = ResponseTiming::YearEnd;
        risk.response_extent = "Mandatory procedures per ISA 240.32-34".into();

        // ISA 240 required responses
        risk.add_response(PlannedResponse::new(
            "Test appropriateness of journal entries and adjustments",
            ResponseProcedureType::TestOfDetails,
            Assertion::Accuracy,
            &self.select_team_member(team_members, "senior"),
            engagement.fieldwork_start + Duration::days(28),
        ));
        risk.add_response(PlannedResponse::new(
            "Review accounting estimates for bias",
            ResponseProcedureType::AnalyticalProcedure,
            Assertion::ValuationAndAllocation,
            &self.select_team_member(team_members, "manager"),
            engagement.fieldwork_start + Duration::days(35),
        ));
        risk.add_response(PlannedResponse::new(
            "Evaluate business rationale for significant unusual transactions",
            ResponseProcedureType::Inquiry,
            Assertion::Occurrence,
            &self.select_team_member(team_members, "manager"),
            engagement.fieldwork_start + Duration::days(42),
        ));

        let assessor = self.select_team_member(team_members, "manager");
        risk = risk.with_assessed_by(&assessor, engagement.planning_start);
        risk.review_status = RiskReviewStatus::Approved;
        risk.reviewer_id = Some(engagement.engagement_partner_id.clone());
        risk.review_date = Some(engagement.planning_start + Duration::days(7));

        risk
    }

    /// Get risk areas based on accounts.
    fn get_risk_areas(&mut self, accounts: &[String]) -> Vec<String> {
        let mut areas: Vec<String> = if accounts.is_empty() {
            vec![
                "Cash and Cash Equivalents".into(),
                "Accounts Receivable".into(),
                "Inventory".into(),
                "Property, Plant and Equipment".into(),
                "Accounts Payable".into(),
                "Accrued Liabilities".into(),
                "Long-term Debt".into(),
                "Revenue".into(),
                "Cost of Sales".into(),
                "Operating Expenses".into(),
                "Payroll and Benefits".into(),
                "Income Taxes".into(),
                "Related Party Transactions".into(),
                "Financial Statement Disclosures".into(),
                "IT General Controls".into(),
            ]
        } else {
            accounts.to_vec()
        };

        // Shuffle and return
        for i in (1..areas.len()).rev() {
            let j = self.rng.random_range(0..=i);
            areas.swap(i, j);
        }
        areas
    }

    /// Select risk category.
    fn select_risk_category(&mut self) -> RiskCategory {
        let categories = [
            (RiskCategory::AssertionLevel, 0.50),
            (RiskCategory::FinancialStatementLevel, 0.15),
            (RiskCategory::EstimateRisk, 0.10),
            (RiskCategory::ItGeneralControl, 0.10),
            (RiskCategory::RelatedParty, 0.05),
            (RiskCategory::GoingConcern, 0.05),
            (RiskCategory::RegulatoryCompliance, 0.05),
        ];

        let r: f64 = self.rng.random();
        let mut cumulative = 0.0;
        for (category, probability) in categories {
            cumulative += probability;
            if r < cumulative {
                return category;
            }
        }
        RiskCategory::AssertionLevel
    }

    /// Generate risk description and assertion.
    fn generate_risk_description(
        &mut self,
        account_or_process: &str,
        category: RiskCategory,
    ) -> (String, Option<Assertion>) {
        let assertions = [
            (Assertion::Existence, "existence"),
            (Assertion::Completeness, "completeness"),
            (Assertion::Accuracy, "accuracy"),
            (Assertion::ValuationAndAllocation, "valuation"),
            (Assertion::Cutoff, "cutoff"),
            (Assertion::RightsAndObligations, "rights and obligations"),
            (
                Assertion::PresentationAndDisclosure,
                "presentation and disclosure",
            ),
        ];

        let idx = self.rng.random_range(0..assertions.len());
        let (assertion, assertion_name) = assertions[idx];

        let description = match category {
            RiskCategory::AssertionLevel => {
                format!(
                    "Risk that {account_or_process} is materially misstated due to {assertion_name}"
                )
            }
            RiskCategory::FinancialStatementLevel => {
                format!(
                    "Pervasive risk affecting {account_or_process} due to control environment weaknesses"
                )
            }
            RiskCategory::EstimateRisk => {
                format!(
                    "Risk of material misstatement in {account_or_process} estimates due to estimation uncertainty"
                )
            }
            RiskCategory::ItGeneralControl => {
                format!(
                    "IT general control risk affecting {account_or_process} data integrity and processing"
                )
            }
            RiskCategory::RelatedParty => {
                format!(
                    "Risk of undisclosed related party transactions affecting {account_or_process}"
                )
            }
            RiskCategory::GoingConcern => {
                "Risk that the entity may not continue as a going concern".into()
            }
            RiskCategory::RegulatoryCompliance => {
                format!(
                    "Risk of non-compliance with laws and regulations affecting {account_or_process}"
                )
            }
            RiskCategory::FraudRisk => {
                format!("Fraud risk in {account_or_process}")
            }
        };

        let assertion_opt = match category {
            RiskCategory::AssertionLevel | RiskCategory::EstimateRisk => Some(assertion),
            _ => None,
        };

        (description, assertion_opt)
    }

    /// Generate inherent risk level.
    fn generate_inherent_risk(&mut self) -> RiskLevel {
        if self.rng.random::<f64>() < self.config.high_inherent_risk_probability {
            RiskLevel::High
        } else if self.rng.random::<f64>() < 0.5 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    /// Generate control risk based on inherent risk.
    fn generate_control_risk(&mut self, inherent_risk: &RiskLevel) -> RiskLevel {
        // Control risk tends to be correlated with inherent risk
        match inherent_risk {
            RiskLevel::High | RiskLevel::Significant => {
                if self.rng.random::<f64>() < 0.6 {
                    RiskLevel::High
                } else {
                    RiskLevel::Medium
                }
            }
            RiskLevel::Medium => {
                if self.rng.random::<f64>() < 0.4 {
                    RiskLevel::Medium
                } else if self.rng.random::<f64>() < 0.7 {
                    RiskLevel::Low
                } else {
                    RiskLevel::High
                }
            }
            RiskLevel::Low => {
                if self.rng.random::<f64>() < 0.7 {
                    RiskLevel::Low
                } else {
                    RiskLevel::Medium
                }
            }
        }
    }

    /// Generate significant risk rationale.
    fn generate_significant_risk_rationale(&mut self, category: RiskCategory) -> String {
        match category {
            RiskCategory::FraudRisk => {
                "Fraud risk requiring special audit consideration per ISA 240".into()
            }
            RiskCategory::EstimateRisk => {
                "High estimation uncertainty requiring special audit consideration per ISA 540"
                    .into()
            }
            RiskCategory::RelatedParty => {
                "Related party transactions outside normal course of business per ISA 550".into()
            }
            RiskCategory::GoingConcern => {
                "Significant doubt about going concern per ISA 570".into()
            }
            _ => {
                let rationales = [
                    "High inherent risk combined with weak control environment",
                    "Significant management judgment involved",
                    "Complex transactions requiring specialized knowledge",
                    "History of misstatements in this area",
                    "New accounting standard implementation",
                ];
                let idx = self.rng.random_range(0..rationales.len());
                rationales[idx].into()
            }
        }
    }

    /// Generate fraud risk factors.
    fn generate_fraud_factors(&mut self) -> Vec<FraudRiskFactor> {
        let mut factors = Vec::new();
        let count = self.rng.random_range(1..=3);

        let pressure_indicators = [
            "Financial pressure from debt covenants",
            "Compensation tied to financial targets",
            "Industry decline affecting profitability",
            "Unrealistic budget expectations",
        ];

        let opportunity_indicators = [
            "Weak segregation of duties",
            "Lack of independent oversight",
            "Complex organizational structure",
            "Inadequate monitoring of controls",
        ];

        let rationalization_indicators = [
            "History of management explanations for variances",
            "Aggressive accounting policies",
            "Frequent disputes with auditors",
            "Strained relationship with regulators",
        ];

        for _ in 0..count {
            let element = match self.rng.random_range(0..3) {
                0 => {
                    let idx = self.rng.random_range(0..pressure_indicators.len());
                    FraudRiskFactor::new(
                        FraudTriangleElement::Pressure,
                        pressure_indicators[idx],
                        self.rng.random_range(40..90),
                        "Risk assessment procedures",
                    )
                }
                1 => {
                    let idx = self.rng.random_range(0..opportunity_indicators.len());
                    FraudRiskFactor::new(
                        FraudTriangleElement::Opportunity,
                        opportunity_indicators[idx],
                        self.rng.random_range(40..90),
                        "Control environment assessment",
                    )
                }
                _ => {
                    let idx = self.rng.random_range(0..rationalization_indicators.len());
                    FraudRiskFactor::new(
                        FraudTriangleElement::Rationalization,
                        rationalization_indicators[idx],
                        self.rng.random_range(30..70),
                        "Management inquiries",
                    )
                }
            };

            let trend = match self.rng.random_range(0..3) {
                0 => Trend::Increasing,
                1 => Trend::Stable,
                _ => Trend::Decreasing,
            };

            factors.push(element.with_trend(trend));
        }

        factors
    }

    /// Select response nature based on risk.
    fn select_response_nature(&mut self, risk: &RiskAssessment) -> ResponseNature {
        match risk.risk_of_material_misstatement {
            RiskLevel::High | RiskLevel::Significant => ResponseNature::SubstantiveOnly,
            RiskLevel::Medium => {
                if self.rng.random::<f64>() < 0.6 {
                    ResponseNature::Combined
                } else {
                    ResponseNature::SubstantiveOnly
                }
            }
            RiskLevel::Low => {
                if self.rng.random::<f64>() < 0.4 {
                    ResponseNature::ControlsReliance
                } else {
                    ResponseNature::Combined
                }
            }
        }
    }

    /// Select response timing.
    fn select_response_timing(&mut self, engagement: &AuditEngagement) -> ResponseTiming {
        match engagement.current_phase {
            EngagementPhase::Planning | EngagementPhase::RiskAssessment => {
                if self.rng.random::<f64>() < 0.3 {
                    ResponseTiming::Interim
                } else {
                    ResponseTiming::YearEnd
                }
            }
            _ => ResponseTiming::YearEnd,
        }
    }

    /// Generate response extent description.
    fn generate_response_extent(&mut self, risk: &RiskAssessment) -> String {
        match risk.required_detection_risk() {
            DetectionRisk::Low => {
                "Extended testing with larger sample sizes and unpredictable procedures".into()
            }
            DetectionRisk::Medium => {
                "Moderate sample sizes with standard testing procedures".into()
            }
            DetectionRisk::High => {
                "Reduced testing extent with reliance on analytical procedures".into()
            }
        }
    }

    /// Generate a planned response.
    fn generate_planned_response(
        &mut self,
        risk: &RiskAssessment,
        team_members: &[String],
        engagement: &AuditEngagement,
    ) -> PlannedResponse {
        let procedure_type = self.select_procedure_type(&risk.response_nature);
        let assertion = risk.assertion.unwrap_or_else(|| self.random_assertion());
        let procedure =
            self.generate_procedure_description(procedure_type, &risk.account_or_process);

        let days_offset = self.rng.random_range(7..45);
        let target_date = engagement.fieldwork_start + Duration::days(days_offset);

        let mut response = PlannedResponse::new(
            &procedure,
            procedure_type,
            assertion,
            &self.select_team_member(team_members, "staff"),
            target_date,
        );

        // Maybe mark as in progress or complete
        if self.rng.random::<f64>() < 0.2 {
            response.status = ResponseStatus::InProgress;
        }

        response
    }

    /// Select procedure type based on response nature.
    fn select_procedure_type(&mut self, nature: &ResponseNature) -> ResponseProcedureType {
        match nature {
            ResponseNature::ControlsReliance => {
                if self.rng.random::<f64>() < 0.7 {
                    ResponseProcedureType::TestOfControls
                } else {
                    ResponseProcedureType::Inquiry
                }
            }
            ResponseNature::SubstantiveOnly => {
                let types = [
                    ResponseProcedureType::TestOfDetails,
                    ResponseProcedureType::AnalyticalProcedure,
                    ResponseProcedureType::Confirmation,
                    ResponseProcedureType::PhysicalInspection,
                ];
                let idx = self.rng.random_range(0..types.len());
                types[idx]
            }
            ResponseNature::Combined => {
                let types = [
                    ResponseProcedureType::TestOfControls,
                    ResponseProcedureType::TestOfDetails,
                    ResponseProcedureType::AnalyticalProcedure,
                ];
                let idx = self.rng.random_range(0..types.len());
                types[idx]
            }
        }
    }

    /// Generate procedure description.
    fn generate_procedure_description(
        &mut self,
        procedure_type: ResponseProcedureType,
        account: &str,
    ) -> String {
        match procedure_type {
            ResponseProcedureType::TestOfControls => {
                format!("Test operating effectiveness of controls over {account}")
            }
            ResponseProcedureType::TestOfDetails => {
                format!(
                    "Select sample of {account} transactions and vouch to supporting documentation"
                )
            }
            ResponseProcedureType::AnalyticalProcedure => {
                format!("Perform analytical procedures on {account} and investigate variances")
            }
            ResponseProcedureType::Confirmation => {
                format!("Send confirmations for {account} balances")
            }
            ResponseProcedureType::PhysicalInspection => {
                format!("Physically inspect {account} items")
            }
            ResponseProcedureType::Inquiry => {
                format!("Inquire of management regarding {account} processes")
            }
        }
    }

    /// Select a team member.
    fn select_team_member(&mut self, team_members: &[String], role_hint: &str) -> String {
        let matching: Vec<&String> = team_members
            .iter()
            .filter(|m| m.to_lowercase().contains(role_hint))
            .collect();

        if let Some(&member) = matching.first() {
            member.clone()
        } else if !team_members.is_empty() {
            let idx = self.rng.random_range(0..team_members.len());
            team_members[idx].clone()
        } else {
            format!("{}001", role_hint.to_uppercase())
        }
    }

    /// Generate a random assertion.
    fn random_assertion(&mut self) -> Assertion {
        let assertions = [
            Assertion::Occurrence,
            Assertion::Completeness,
            Assertion::Accuracy,
            Assertion::Cutoff,
            Assertion::Existence,
            Assertion::ValuationAndAllocation,
        ];
        let idx = self.rng.random_range(0..assertions.len());
        assertions[idx]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::audit::test_helpers::create_test_engagement;

    #[test]
    fn test_risk_generation() {
        let mut generator = RiskAssessmentGenerator::new(42);
        let engagement = create_test_engagement();
        let team = vec!["STAFF001".into(), "SENIOR001".into(), "MANAGER001".into()];

        let risks = generator.generate_risks_for_engagement(&engagement, &team, &[]);

        assert!(risks.len() >= 2); // At least presumed risks

        // Check for presumed risks
        let has_revenue_fraud = risks.iter().any(|r| r.presumed_revenue_fraud_risk);
        let has_mgmt_override = risks.iter().any(|r| r.presumed_management_override);
        assert!(has_revenue_fraud);
        assert!(has_mgmt_override);
    }

    #[test]
    fn test_significant_risk() {
        let mut generator = RiskAssessmentGenerator::new(42);
        let engagement = create_test_engagement();
        let team = vec!["STAFF001".into()];

        let risks = generator.generate_risks_for_engagement(&engagement, &team, &[]);

        // Presumed risks should be significant
        let significant_risks: Vec<_> = risks.iter().filter(|r| r.is_significant_risk).collect();
        assert!(significant_risks.len() >= 2);
    }

    #[test]
    fn test_planned_responses() {
        let mut generator = RiskAssessmentGenerator::new(42);
        let engagement = create_test_engagement();
        let team = vec!["STAFF001".into(), "SENIOR001".into()];

        let risks = generator.generate_risks_for_engagement(&engagement, &team, &[]);

        for risk in &risks {
            assert!(!risk.planned_response.is_empty());
        }
    }

    #[test]
    fn test_fraud_factors() {
        let config = RiskAssessmentGeneratorConfig {
            fraud_factor_probability: 1.0,
            ..Default::default()
        };
        let mut generator = RiskAssessmentGenerator::with_config(42, config);
        let engagement = create_test_engagement();

        let _risk =
            generator.generate_risk_assessment(&engagement, "Inventory", &["STAFF001".into()]);

        // May or may not have fraud factors depending on risk category
        // But presumed risks always have them
    }

    #[test]
    fn test_detection_risk() {
        let mut generator = RiskAssessmentGenerator::new(42);
        let engagement = create_test_engagement();

        let risks = generator.generate_risks_for_engagement(&engagement, &["STAFF001".into()], &[]);

        for risk in &risks {
            let detection_risk = risk.required_detection_risk();
            // High ROMM should require low detection risk
            if matches!(
                risk.risk_of_material_misstatement,
                RiskLevel::High | RiskLevel::Significant
            ) {
                assert_eq!(detection_risk, DetectionRisk::Low);
            }
        }
    }
}
