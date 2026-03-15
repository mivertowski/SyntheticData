//! Finding generator for audit engagements.
//!
//! Generates audit findings with condition/criteria/cause/effect structure,
//! remediation plans, and cross-references per ISA 265.

use chrono::Duration;
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

use datasynth_core::models::audit::{
    Assertion, AuditEngagement, AuditFinding, FindingSeverity, FindingStatus, FindingType,
    MilestoneStatus, RemediationPlan, RemediationStatus, Workpaper,
};

/// A control available for linkage during finding generation.
#[derive(Debug, Clone)]
pub struct AvailableControl {
    /// Control ID (external reference, e.g., "CTRL-001")
    pub control_id: String,
    /// Assertions this control addresses
    pub assertions: Vec<Assertion>,
    /// Process areas this control covers
    pub process_areas: Vec<String>,
}

/// A risk assessment available for linkage during finding generation.
#[derive(Debug, Clone)]
pub struct AvailableRisk {
    /// Risk ID (external reference, e.g., "RISK-001")
    pub risk_id: String,
    /// Engagement ID this risk belongs to
    pub engagement_id: uuid::Uuid,
    /// Account or process being assessed
    pub account_or_process: String,
}

/// Configuration for finding generation.
#[derive(Debug, Clone)]
pub struct FindingGeneratorConfig {
    /// Number of findings per engagement (min, max)
    pub findings_per_engagement: (u32, u32),
    /// Probability of material weakness
    pub material_weakness_probability: f64,
    /// Probability of significant deficiency
    pub significant_deficiency_probability: f64,
    /// Probability of misstatement finding
    pub misstatement_probability: f64,
    /// Probability of remediation plan
    pub remediation_plan_probability: f64,
    /// Probability of management agreement
    pub management_agrees_probability: f64,
    /// Misstatement amount range (min, max)
    pub misstatement_range: (i64, i64),
}

impl Default for FindingGeneratorConfig {
    fn default() -> Self {
        Self {
            findings_per_engagement: (3, 12),
            material_weakness_probability: 0.05,
            significant_deficiency_probability: 0.15,
            misstatement_probability: 0.30,
            remediation_plan_probability: 0.70,
            management_agrees_probability: 0.85,
            misstatement_range: (1_000, 500_000),
        }
    }
}

/// Generator for audit findings.
pub struct FindingGenerator {
    rng: ChaCha8Rng,
    config: FindingGeneratorConfig,
    finding_counter: u32,
    fiscal_year: u16,
}

impl FindingGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config: FindingGeneratorConfig::default(),
            finding_counter: 0,
            fiscal_year: 2025,
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: FindingGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
            finding_counter: 0,
            fiscal_year: 2025,
        }
    }

    /// Generate findings for an engagement.
    pub fn generate_findings_for_engagement(
        &mut self,
        engagement: &AuditEngagement,
        workpapers: &[Workpaper],
        team_members: &[String],
    ) -> Vec<AuditFinding> {
        self.fiscal_year = engagement.fiscal_year;

        let count = self.rng.random_range(
            self.config.findings_per_engagement.0..=self.config.findings_per_engagement.1,
        );

        let mut findings = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let finding = self.generate_finding(engagement, workpapers, team_members);
            findings.push(finding);
        }

        findings
    }

    /// Generate a single finding.
    pub fn generate_finding(
        &mut self,
        engagement: &AuditEngagement,
        workpapers: &[Workpaper],
        team_members: &[String],
    ) -> AuditFinding {
        self.finding_counter += 1;

        let finding_type = self.select_finding_type();
        let (title, account) = self.generate_finding_title(finding_type);

        let mut finding = AuditFinding::new(engagement.engagement_id, finding_type, &title);

        finding.finding_ref = format!("FIND-{}-{:03}", self.fiscal_year, self.finding_counter);

        // Generate condition, criteria, cause, effect
        let (condition, criteria, cause, effect) = self.generate_ccce(finding_type, &account);
        finding = finding.with_details(&condition, &criteria, &cause, &effect);

        // Generate recommendation
        let recommendation = self.generate_recommendation(finding_type, &account);
        finding = finding.with_recommendation(&recommendation);

        // Set severity based on type
        finding.severity = self.determine_severity(finding_type, &finding);

        // Add monetary impact for misstatements
        if self.is_misstatement_type(finding_type) {
            let (factual, projected, judgmental) = self.generate_misstatement_amounts();
            finding = finding.with_misstatement(factual, projected, judgmental);

            if let Some(f) = factual {
                finding = finding.with_monetary_impact(f);
            }
        }

        // Add assertions and accounts
        finding.assertions_affected = self.select_assertions(finding_type);
        finding.accounts_affected = vec![account.clone()];
        finding.process_areas = self.select_process_areas(&account);

        // Link to workpapers
        if !workpapers.is_empty() {
            let wp_count = self.rng.random_range(1..=3.min(workpapers.len()));
            for _ in 0..wp_count {
                let idx = self.rng.random_range(0..workpapers.len());
                finding.workpaper_refs.push(workpapers[idx].workpaper_id);
            }
            // Set the primary workpaper_id to the first linked workpaper
            if let Some(first_wp) = finding.workpaper_refs.first() {
                finding.workpaper_id = Some(first_wp.to_string());
            }
        }

        // Set identified by
        let identifier = self.select_team_member(team_members, "senior");
        finding.identified_by = identifier;
        finding.identified_date =
            engagement.fieldwork_start + Duration::days(self.rng.random_range(7..30));

        // Maybe add review
        if self.rng.random::<f64>() < 0.8 {
            finding.reviewed_by = Some(self.select_team_member(team_members, "manager"));
            finding.review_date =
                Some(finding.identified_date + Duration::days(self.rng.random_range(3..10)));
            finding.status = FindingStatus::PendingReview;
        }

        // Determine reporting requirements
        finding.mark_for_reporting(
            finding.finding_type.requires_sox_reporting() || finding.severity.score() >= 3,
            finding.requires_governance_communication(),
        );

        // Maybe add management response
        if self.rng.random::<f64>() < 0.7 {
            let response_date =
                finding.identified_date + Duration::days(self.rng.random_range(7..21));
            let agrees = self.rng.random::<f64>() < self.config.management_agrees_probability;
            let response = self.generate_management_response(finding_type, agrees);
            finding.add_management_response(&response, agrees, response_date);

            // Maybe add remediation plan
            if agrees && self.rng.random::<f64>() < self.config.remediation_plan_probability {
                let plan = self.generate_remediation_plan(&finding, &account);
                finding.with_remediation_plan(plan);
            }
        }

        finding
    }

    /// Generate findings for an engagement with control and risk context.
    ///
    /// This extended version populates `related_control_ids` and `related_risk_id`
    /// by matching finding assertions/process areas against available controls and risks.
    pub fn generate_findings_with_context(
        &mut self,
        engagement: &AuditEngagement,
        workpapers: &[Workpaper],
        team_members: &[String],
        controls: &[AvailableControl],
        risks: &[AvailableRisk],
    ) -> Vec<AuditFinding> {
        let mut findings =
            self.generate_findings_for_engagement(engagement, workpapers, team_members);

        for finding in &mut findings {
            self.link_controls_and_risks(finding, controls, risks);
        }

        findings
    }

    /// Link a finding to related controls and risks based on assertion/process overlap.
    fn link_controls_and_risks(
        &mut self,
        finding: &mut AuditFinding,
        controls: &[AvailableControl],
        risks: &[AvailableRisk],
    ) {
        // Link controls: find controls whose assertions overlap with this finding's assertions
        let finding_assertions = &finding.assertions_affected;
        let finding_process_areas = &finding.process_areas;

        let mut matched_controls: Vec<&AvailableControl> = controls
            .iter()
            .filter(|ctrl| {
                // Match if any assertion overlaps
                let assertion_match = ctrl
                    .assertions
                    .iter()
                    .any(|a| finding_assertions.contains(a));
                // Or if any process area overlaps (case-insensitive)
                let process_match = ctrl.process_areas.iter().any(|pa| {
                    finding_process_areas
                        .iter()
                        .any(|fp| fp.to_lowercase().contains(&pa.to_lowercase()))
                });
                assertion_match || process_match
            })
            .collect();

        // If no matches, pick 1-2 random controls (findings should still link to something)
        if matched_controls.is_empty() && !controls.is_empty() {
            let count = self.rng.random_range(1..=2.min(controls.len()));
            for _ in 0..count {
                let idx = self.rng.random_range(0..controls.len());
                matched_controls.push(&controls[idx]);
            }
        }

        // Cap at 3 related controls
        if matched_controls.len() > 3 {
            matched_controls.truncate(3);
        }

        finding.related_control_ids = matched_controls
            .iter()
            .map(|c| c.control_id.clone())
            .collect();

        // Link risk: find a risk in the same engagement whose account/process matches
        let engagement_risks: Vec<&AvailableRisk> = risks
            .iter()
            .filter(|r| r.engagement_id == finding.engagement_id)
            .collect();

        if !engagement_risks.is_empty() {
            // Try to find a risk whose account_or_process matches one of our affected accounts
            let matching_risk = engagement_risks.iter().find(|r| {
                finding.accounts_affected.iter().any(|a| {
                    r.account_or_process
                        .to_lowercase()
                        .contains(&a.to_lowercase())
                })
            });

            if let Some(risk) = matching_risk {
                finding.related_risk_id = Some(risk.risk_id.clone());
            } else {
                // Fall back to a random risk from the same engagement
                let idx = self.rng.random_range(0..engagement_risks.len());
                finding.related_risk_id = Some(engagement_risks[idx].risk_id.clone());
            }
        }
    }

    /// Select finding type based on probabilities.
    fn select_finding_type(&mut self) -> FindingType {
        let r: f64 = self.rng.random();

        if r < self.config.material_weakness_probability {
            FindingType::MaterialWeakness
        } else if r < self.config.material_weakness_probability
            + self.config.significant_deficiency_probability
        {
            FindingType::SignificantDeficiency
        } else if r < self.config.material_weakness_probability
            + self.config.significant_deficiency_probability
            + self.config.misstatement_probability
        {
            if self.rng.random::<f64>() < 0.3 {
                FindingType::MaterialMisstatement
            } else {
                FindingType::ImmaterialMisstatement
            }
        } else {
            let other_types = [
                FindingType::ControlDeficiency,
                FindingType::ComplianceException,
                FindingType::OtherMatter,
                FindingType::ItDeficiency,
                FindingType::ProcessImprovement,
            ];
            let idx = self.rng.random_range(0..other_types.len());
            other_types[idx]
        }
    }

    /// Generate finding title and related account.
    fn generate_finding_title(&mut self, finding_type: FindingType) -> (String, String) {
        match finding_type {
            FindingType::MaterialWeakness => {
                let titles = [
                    (
                        "Inadequate segregation of duties in revenue cycle",
                        "Revenue",
                    ),
                    (
                        "Lack of effective review of journal entries",
                        "General Ledger",
                    ),
                    (
                        "Insufficient IT general controls over financial applications",
                        "IT Controls",
                    ),
                    (
                        "Inadequate controls over financial close process",
                        "Financial Close",
                    ),
                ];
                let idx = self.rng.random_range(0..titles.len());
                (titles[idx].0.into(), titles[idx].1.into())
            }
            FindingType::SignificantDeficiency => {
                let titles = [
                    (
                        "Inadequate documentation of account reconciliations",
                        "Accounts Receivable",
                    ),
                    (
                        "Untimely review of vendor master file changes",
                        "Accounts Payable",
                    ),
                    ("Incomplete fixed asset physical inventory", "Fixed Assets"),
                    (
                        "Lack of formal approval for manual journal entries",
                        "General Ledger",
                    ),
                ];
                let idx = self.rng.random_range(0..titles.len());
                (titles[idx].0.into(), titles[idx].1.into())
            }
            FindingType::ControlDeficiency => {
                let titles = [
                    (
                        "Missing secondary approval on expense reports",
                        "Operating Expenses",
                    ),
                    ("Incomplete access review documentation", "IT Controls"),
                    ("Delayed bank reconciliation preparation", "Cash"),
                    ("Inconsistent inventory count procedures", "Inventory"),
                ];
                let idx = self.rng.random_range(0..titles.len());
                (titles[idx].0.into(), titles[idx].1.into())
            }
            FindingType::MaterialMisstatement | FindingType::ImmaterialMisstatement => {
                let titles = [
                    ("Revenue cutoff error", "Revenue"),
                    ("Inventory valuation adjustment", "Inventory"),
                    (
                        "Accounts receivable allowance understatement",
                        "Accounts Receivable",
                    ),
                    ("Accrued liabilities understatement", "Accrued Liabilities"),
                    ("Fixed asset depreciation calculation error", "Fixed Assets"),
                ];
                let idx = self.rng.random_range(0..titles.len());
                (titles[idx].0.into(), titles[idx].1.into())
            }
            FindingType::ComplianceException => {
                let titles = [
                    ("Late filing of sales tax returns", "Tax"),
                    ("Incomplete Form 1099 reporting", "Tax"),
                    ("Non-compliance with debt covenant reporting", "Debt"),
                ];
                let idx = self.rng.random_range(0..titles.len());
                (titles[idx].0.into(), titles[idx].1.into())
            }
            FindingType::ItDeficiency => {
                let titles = [
                    ("Excessive user access privileges", "IT Controls"),
                    ("Inadequate password policy enforcement", "IT Controls"),
                    ("Missing change management documentation", "IT Controls"),
                    ("Incomplete disaster recovery testing", "IT Controls"),
                ];
                let idx = self.rng.random_range(0..titles.len());
                (titles[idx].0.into(), titles[idx].1.into())
            }
            FindingType::OtherMatter | FindingType::ProcessImprovement => {
                let titles = [
                    (
                        "Opportunity to improve month-end close efficiency",
                        "Financial Close",
                    ),
                    (
                        "Enhancement to vendor onboarding process",
                        "Accounts Payable",
                    ),
                    (
                        "Automation opportunity in reconciliation process",
                        "General Ledger",
                    ),
                ];
                let idx = self.rng.random_range(0..titles.len());
                (titles[idx].0.into(), titles[idx].1.into())
            }
        }
    }

    /// Generate condition, criteria, cause, effect.
    fn generate_ccce(
        &mut self,
        finding_type: FindingType,
        account: &str,
    ) -> (String, String, String, String) {
        match finding_type {
            FindingType::MaterialWeakness
            | FindingType::SignificantDeficiency
            | FindingType::ControlDeficiency => {
                let condition = format!(
                    "During our testing of {} controls, we noted that the control was not operating effectively. \
                    Specifically, {} of {} items tested did not have evidence of the required control activity.",
                    account,
                    self.rng.random_range(2..8),
                    self.rng.random_range(20..40)
                );
                let criteria = format!(
                    "Company policy and SOX requirements mandate that all {account} transactions receive appropriate \
                    review and approval prior to processing."
                );
                let cause = "Staffing constraints and competing priorities resulted in reduced focus on control execution.".into();
                let effect = format!(
                    "Transactions may be processed without appropriate oversight, increasing the risk of errors \
                    or fraud in the {account} balance."
                );
                (condition, criteria, cause, effect)
            }
            FindingType::MaterialMisstatement | FindingType::ImmaterialMisstatement => {
                let amount = self.rng.random_range(
                    self.config.misstatement_range.0..self.config.misstatement_range.1,
                );
                let condition = format!(
                    "Our testing identified a misstatement in {account} of approximately ${amount}. \
                    The error resulted from incorrect application of accounting standards."
                );
                let criteria = "US GAAP and company accounting policy require accurate recording of all transactions.".into();
                let cause =
                    "Manual calculation error combined with inadequate review procedures.".into();
                let effect = format!(
                    "The {} balance was {} by ${}, which {}.",
                    account,
                    if self.rng.random::<bool>() {
                        "overstated"
                    } else {
                        "understated"
                    },
                    amount,
                    if finding_type == FindingType::MaterialMisstatement {
                        "represents a material misstatement"
                    } else {
                        "is below materiality but has been communicated to management"
                    }
                );
                (condition, criteria, cause, effect)
            }
            FindingType::ComplianceException => {
                let condition = format!(
                    "The Company did not comply with {account} regulatory requirements during the period under audit."
                );
                let criteria =
                    "Applicable laws and regulations require timely and accurate compliance."
                        .into();
                let cause = "Lack of monitoring procedures to track compliance deadlines.".into();
                let effect =
                    "The Company may be subject to penalties or regulatory scrutiny.".into();
                (condition, criteria, cause, effect)
            }
            _ => {
                let condition =
                    format!("We identified an opportunity to enhance the {account} process.");
                let criteria =
                    "Industry best practices suggest continuous improvement in control processes."
                        .into();
                let cause =
                    "Current processes have not been updated to reflect operational changes."
                        .into();
                let effect =
                    "Operational efficiency could be improved with process enhancements.".into();
                (condition, criteria, cause, effect)
            }
        }
    }

    /// Generate recommendation.
    fn generate_recommendation(&mut self, finding_type: FindingType, account: &str) -> String {
        match finding_type {
            FindingType::MaterialWeakness | FindingType::SignificantDeficiency => {
                format!(
                    "We recommend that management: (1) Implement additional review procedures for {account} transactions, \
                    (2) Document all control activities contemporaneously, and \
                    (3) Provide additional training to personnel responsible for control execution."
                )
            }
            FindingType::ControlDeficiency => {
                format!(
                    "We recommend that management strengthen the {account} control by ensuring timely execution \
                    and documentation of all required review activities."
                )
            }
            FindingType::MaterialMisstatement | FindingType::ImmaterialMisstatement => {
                "We recommend that management record the proposed adjusting entry and implement \
                additional review procedures to prevent similar errors in future periods.".into()
            }
            FindingType::ComplianceException => {
                "We recommend that management implement a compliance calendar with automated reminders \
                and establish monitoring procedures to ensure timely compliance.".into()
            }
            FindingType::ItDeficiency => {
                "We recommend that IT management review and remediate the identified access control \
                weaknesses and implement periodic access certification procedures.".into()
            }
            _ => {
                format!(
                    "We recommend that management evaluate the {account} process for potential \
                    efficiency improvements and implement changes as appropriate."
                )
            }
        }
    }

    /// Determine severity based on finding type and other factors.
    fn determine_severity(
        &mut self,
        finding_type: FindingType,
        _finding: &AuditFinding,
    ) -> FindingSeverity {
        let base_severity = finding_type.default_severity();

        // Maybe adjust severity
        if self.rng.random::<f64>() < 0.2 {
            match base_severity {
                FindingSeverity::Critical => FindingSeverity::High,
                FindingSeverity::High => {
                    if self.rng.random::<bool>() {
                        FindingSeverity::Critical
                    } else {
                        FindingSeverity::Medium
                    }
                }
                FindingSeverity::Medium => {
                    if self.rng.random::<bool>() {
                        FindingSeverity::High
                    } else {
                        FindingSeverity::Low
                    }
                }
                FindingSeverity::Low => FindingSeverity::Medium,
                FindingSeverity::Informational => FindingSeverity::Low,
            }
        } else {
            base_severity
        }
    }

    /// Check if finding type is a misstatement.
    fn is_misstatement_type(&self, finding_type: FindingType) -> bool {
        matches!(
            finding_type,
            FindingType::MaterialMisstatement | FindingType::ImmaterialMisstatement
        )
    }

    /// Generate misstatement amounts.
    fn generate_misstatement_amounts(
        &mut self,
    ) -> (Option<Decimal>, Option<Decimal>, Option<Decimal>) {
        let factual = Decimal::new(
            self.rng
                .random_range(self.config.misstatement_range.0..self.config.misstatement_range.1),
            0,
        );

        let projected = if self.rng.random::<f64>() < 0.5 {
            Some(Decimal::new(
                self.rng
                    .random_range(0..self.config.misstatement_range.1 / 2),
                0,
            ))
        } else {
            None
        };

        let judgmental = if self.rng.random::<f64>() < 0.3 {
            Some(Decimal::new(
                self.rng
                    .random_range(0..self.config.misstatement_range.1 / 4),
                0,
            ))
        } else {
            None
        };

        (Some(factual), projected, judgmental)
    }

    /// Select assertions affected.
    fn select_assertions(&mut self, finding_type: FindingType) -> Vec<Assertion> {
        let mut assertions = Vec::new();

        match finding_type {
            FindingType::MaterialMisstatement | FindingType::ImmaterialMisstatement => {
                assertions.push(Assertion::Accuracy);
                if self.rng.random::<bool>() {
                    assertions.push(Assertion::ValuationAndAllocation);
                }
            }
            FindingType::MaterialWeakness
            | FindingType::SignificantDeficiency
            | FindingType::ControlDeficiency => {
                let possible = [
                    Assertion::Occurrence,
                    Assertion::Completeness,
                    Assertion::Accuracy,
                    Assertion::Classification,
                ];
                let count = self.rng.random_range(1..=3);
                for _ in 0..count {
                    let idx = self.rng.random_range(0..possible.len());
                    if !assertions.contains(&possible[idx]) {
                        assertions.push(possible[idx]);
                    }
                }
            }
            _ => {
                assertions.push(Assertion::PresentationAndDisclosure);
            }
        }

        assertions
    }

    /// Select process areas.
    fn select_process_areas(&mut self, account: &str) -> Vec<String> {
        let account_lower = account.to_lowercase();

        if account_lower.contains("revenue") || account_lower.contains("receivable") {
            vec!["Order to Cash".into(), "Revenue Recognition".into()]
        } else if account_lower.contains("payable") || account_lower.contains("expense") {
            vec!["Procure to Pay".into(), "Expense Management".into()]
        } else if account_lower.contains("inventory") {
            vec!["Inventory Management".into(), "Cost of Goods Sold".into()]
        } else if account_lower.contains("fixed asset") {
            vec!["Capital Asset Management".into()]
        } else if account_lower.contains("it") {
            vec![
                "IT General Controls".into(),
                "IT Application Controls".into(),
            ]
        } else if account_lower.contains("payroll") {
            vec!["Hire to Retire".into(), "Payroll Processing".into()]
        } else {
            vec!["Financial Close".into()]
        }
    }

    /// Generate management response.
    fn generate_management_response(&mut self, finding_type: FindingType, agrees: bool) -> String {
        if agrees {
            match finding_type {
                FindingType::MaterialWeakness | FindingType::SignificantDeficiency => {
                    "Management agrees with the finding and has initiated a remediation plan to \
                    address the identified control deficiency. We expect to complete remediation \
                    prior to the next audit cycle."
                        .into()
                }
                FindingType::MaterialMisstatement | FindingType::ImmaterialMisstatement => {
                    "Management agrees with the proposed adjustment and will record the entry. \
                    We have implemented additional review procedures to prevent similar errors."
                        .into()
                }
                _ => "Management agrees with the observation and will implement the recommended \
                    improvements as resources permit."
                    .into(),
            }
        } else {
            "Management respectfully disagrees with the finding. We believe that existing \
            controls are adequate and operating effectively. We will provide additional \
            documentation to support our position."
                .into()
        }
    }

    /// Generate remediation plan.
    fn generate_remediation_plan(
        &mut self,
        finding: &AuditFinding,
        account: &str,
    ) -> RemediationPlan {
        let target_date = finding.identified_date + Duration::days(self.rng.random_range(60..180));

        let description = format!(
            "Implement enhanced controls and monitoring procedures for {account} to address \
            the identified deficiency. This includes updated policies, additional training, \
            and implementation of automated controls where feasible."
        );

        let responsible_party = format!(
            "{} Manager",
            if account.to_lowercase().contains("it") {
                "IT"
            } else {
                "Controller"
            }
        );

        let mut plan = RemediationPlan::new(
            finding.finding_id,
            &description,
            &responsible_party,
            target_date,
        );

        plan.validation_approach =
            "Auditor will test remediated controls during the next audit cycle.".into();

        // Add milestones
        let milestone_dates = [
            (
                finding.identified_date + Duration::days(30),
                "Complete root cause analysis",
            ),
            (
                finding.identified_date + Duration::days(60),
                "Document updated control procedures",
            ),
            (
                finding.identified_date + Duration::days(90),
                "Implement control changes",
            ),
            (target_date, "Complete testing and validation"),
        ];

        for (date, desc) in milestone_dates {
            plan.add_milestone(desc, date);
        }

        // Maybe mark some progress
        if self.rng.random::<f64>() < 0.3 {
            plan.status = RemediationStatus::InProgress;
            if !plan.milestones.is_empty() {
                plan.milestones[0].status = MilestoneStatus::Complete;
                plan.milestones[0].completion_date = Some(plan.milestones[0].target_date);
            }
        }

        plan
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
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::audit::test_helpers::create_test_engagement;

    #[test]
    fn test_finding_generation() {
        let mut generator = FindingGenerator::new(42);
        let engagement = create_test_engagement();
        let team = vec!["STAFF001".into(), "SENIOR001".into(), "MANAGER001".into()];

        let findings = generator.generate_findings_for_engagement(&engagement, &[], &team);

        assert!(!findings.is_empty());
        for finding in &findings {
            assert!(!finding.condition.is_empty());
            assert!(!finding.criteria.is_empty());
            assert!(!finding.recommendation.is_empty());
        }
    }

    #[test]
    fn test_finding_types_distribution() {
        let mut generator = FindingGenerator::new(42);
        let engagement = create_test_engagement();
        let team = vec!["STAFF001".into()];

        // Generate many findings to check distribution
        let config = FindingGeneratorConfig {
            findings_per_engagement: (50, 50),
            ..Default::default()
        };
        generator.config = config;

        let findings = generator.generate_findings_for_engagement(&engagement, &[], &team);

        let material_weaknesses = findings
            .iter()
            .filter(|f| f.finding_type == FindingType::MaterialWeakness)
            .count();
        let significant_deficiencies = findings
            .iter()
            .filter(|f| f.finding_type == FindingType::SignificantDeficiency)
            .count();

        // Material weaknesses should be rare
        assert!(material_weaknesses < 10);
        // Significant deficiencies should be more common than material weaknesses
        assert!(significant_deficiencies > material_weaknesses);
    }

    #[test]
    fn test_misstatement_finding() {
        let config = FindingGeneratorConfig {
            misstatement_probability: 1.0,
            material_weakness_probability: 0.0,
            significant_deficiency_probability: 0.0,
            ..Default::default()
        };
        let mut generator = FindingGenerator::with_config(42, config);
        let engagement = create_test_engagement();

        let finding = generator.generate_finding(&engagement, &[], &["STAFF001".into()]);

        assert!(finding.is_misstatement);
        assert!(finding.factual_misstatement.is_some() || finding.projected_misstatement.is_some());
    }

    #[test]
    fn test_remediation_plan() {
        let config = FindingGeneratorConfig {
            remediation_plan_probability: 1.0,
            management_agrees_probability: 1.0,
            ..Default::default()
        };
        let mut generator = FindingGenerator::with_config(42, config);
        let engagement = create_test_engagement();

        let findings =
            generator.generate_findings_for_engagement(&engagement, &[], &["STAFF001".into()]);

        // At least some findings should have remediation plans
        let with_plans = findings
            .iter()
            .filter(|f| f.remediation_plan.is_some())
            .count();
        assert!(with_plans > 0);

        for finding in findings.iter().filter(|f| f.remediation_plan.is_some()) {
            let plan = finding.remediation_plan.as_ref().unwrap();
            assert!(!plan.description.is_empty());
            assert!(!plan.milestones.is_empty());
        }
    }

    #[test]
    fn test_governance_communication() {
        let config = FindingGeneratorConfig {
            material_weakness_probability: 1.0,
            ..Default::default()
        };
        let mut generator = FindingGenerator::with_config(42, config);
        let engagement = create_test_engagement();

        let finding = generator.generate_finding(&engagement, &[], &["STAFF001".into()]);

        assert!(finding.report_to_governance);
        assert!(finding.include_in_management_letter);
    }

    #[test]
    fn test_generate_findings_with_context_links_controls_and_risks() {
        use datasynth_core::models::audit::Assertion;

        let mut generator = FindingGenerator::new(42);
        let engagement = create_test_engagement();
        let team = vec!["STAFF001".into(), "SENIOR001".into(), "MANAGER001".into()];

        let controls = vec![
            AvailableControl {
                control_id: "CTRL-001".into(),
                assertions: vec![Assertion::Accuracy, Assertion::Completeness],
                process_areas: vec!["Revenue Recognition".into()],
            },
            AvailableControl {
                control_id: "CTRL-002".into(),
                assertions: vec![Assertion::Occurrence],
                process_areas: vec!["Procure to Pay".into()],
            },
        ];

        let risks = vec![AvailableRisk {
            risk_id: "RISK-001".into(),
            engagement_id: engagement.engagement_id,
            account_or_process: "Revenue".into(),
        }];

        let findings =
            generator.generate_findings_with_context(&engagement, &[], &team, &controls, &risks);

        assert!(!findings.is_empty());

        // Every finding should have at least one related control or risk
        for finding in &findings {
            let has_controls = !finding.related_control_ids.is_empty();
            let has_risk = finding.related_risk_id.is_some();
            assert!(
                has_controls || has_risk,
                "Finding {} should have related controls or risk",
                finding.finding_ref
            );
        }

        // At least one finding should have a related risk from same engagement
        let with_risk = findings
            .iter()
            .filter(|f| f.related_risk_id.is_some())
            .count();
        assert!(with_risk > 0, "At least one finding should link to a risk");
    }

    #[test]
    fn test_generate_findings_with_context_caps_controls_at_three() {
        use datasynth_core::models::audit::Assertion;

        let config = FindingGeneratorConfig {
            findings_per_engagement: (5, 5),
            ..Default::default()
        };
        let mut generator = FindingGenerator::with_config(42, config);
        let engagement = create_test_engagement();
        let team = vec!["STAFF001".into()];

        // Create many controls covering all assertions
        let controls: Vec<AvailableControl> = (0..10)
            .map(|i| AvailableControl {
                control_id: format!("CTRL-{:03}", i),
                assertions: vec![
                    Assertion::Accuracy,
                    Assertion::Completeness,
                    Assertion::Occurrence,
                    Assertion::Classification,
                ],
                process_areas: vec![
                    "Revenue Recognition".into(),
                    "Procure to Pay".into(),
                    "Financial Close".into(),
                ],
            })
            .collect();

        let findings =
            generator.generate_findings_with_context(&engagement, &[], &team, &controls, &[]);

        for finding in &findings {
            assert!(
                finding.related_control_ids.len() <= 3,
                "Finding {} has {} controls, expected max 3",
                finding.finding_ref,
                finding.related_control_ids.len()
            );
        }
    }

    #[test]
    fn test_workpaper_id_populated_from_workpaper_refs() {
        let mut generator = FindingGenerator::new(42);
        let engagement = create_test_engagement();
        let team = vec!["STAFF001".into()];

        // Create a workpaper to reference
        use datasynth_core::models::audit::{Workpaper, WorkpaperSection};
        let workpaper = Workpaper::new(
            engagement.engagement_id,
            "WP-001",
            "Test Workpaper",
            WorkpaperSection::ControlTesting,
        );

        let findings = generator.generate_findings_for_engagement(&engagement, &[workpaper], &team);

        // All findings should have at least one workpaper ref, and workpaper_id should be set
        for finding in &findings {
            assert!(
                !finding.workpaper_refs.is_empty(),
                "Finding should have workpaper refs when workpapers provided"
            );
            assert!(
                finding.workpaper_id.is_some(),
                "Finding should have workpaper_id set when workpaper_refs is populated"
            );
        }
    }
}
