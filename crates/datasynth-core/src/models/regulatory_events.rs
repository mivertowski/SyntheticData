//! Regulatory event models for pattern drift simulation.
//!
//! Provides comprehensive regulatory change modeling including:
//! - Accounting standard adoptions (ASC 842, IFRS 16, etc.)
//! - Tax rate changes
//! - New compliance requirements
//! - Industry-specific regulations
//! - Audit focus shifts

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Regulatory drift event with impact details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryDriftEvent {
    /// Event identifier.
    pub event_id: String,
    /// Type of regulation.
    pub regulation_type: RegulationType,
    /// Effective date of the regulation.
    pub effective_date: NaiveDate,
    /// Announcement date (when known about).
    #[serde(default)]
    pub announcement_date: Option<NaiveDate>,
    /// Transition period in days (for phased adoption).
    #[serde(default)]
    pub transition_period_days: u32,
    /// Affected accounts.
    #[serde(default)]
    pub affected_accounts: Vec<String>,
    /// Impacts of this regulatory change.
    #[serde(default)]
    pub impacts: Vec<RegulatoryImpact>,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
}

impl RegulatoryDriftEvent {
    /// Create a new regulatory drift event.
    pub fn new(
        event_id: impl Into<String>,
        regulation_type: RegulationType,
        effective_date: NaiveDate,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            regulation_type,
            effective_date,
            announcement_date: None,
            transition_period_days: 0,
            affected_accounts: Vec::new(),
            impacts: Vec::new(),
            description: None,
        }
    }

    /// Check if the regulation is active at a given date.
    pub fn is_active_at(&self, date: NaiveDate) -> bool {
        date >= self.effective_date
    }

    /// Check if in transition period at a given date.
    pub fn is_in_transition_at(&self, date: NaiveDate) -> bool {
        if date < self.effective_date {
            return false;
        }
        let transition_end =
            self.effective_date + chrono::Duration::days(self.transition_period_days as i64);
        date < transition_end
    }

    /// Get the transition progress (0.0 to 1.0).
    pub fn transition_progress_at(&self, date: NaiveDate) -> f64 {
        if date < self.effective_date {
            return 0.0;
        }
        if self.transition_period_days == 0 {
            return 1.0;
        }
        let days_elapsed = (date - self.effective_date).num_days() as f64;
        (days_elapsed / self.transition_period_days as f64).min(1.0)
    }
}

/// Type of regulatory change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RegulationType {
    /// Accounting standard adoption.
    AccountingStandardAdoption {
        /// Standard name (e.g., "ASC 842", "IFRS 16").
        standard: String,
        /// Framework (US GAAP, IFRS).
        framework: AccountingFramework,
        /// Topic or area (e.g., "leases", "revenue").
        topic: String,
    },
    /// Tax rate change.
    TaxRateChange {
        /// Tax type (e.g., "corporate_income", "vat", "sales").
        tax_type: String,
        /// Old rate.
        #[serde(with = "rust_decimal::serde::str")]
        old_rate: Decimal,
        /// New rate.
        #[serde(with = "rust_decimal::serde::str")]
        new_rate: Decimal,
        /// Jurisdiction.
        jurisdiction: String,
    },
    /// New compliance requirement.
    NewComplianceRequirement {
        /// Requirement name.
        requirement: String,
        /// Compliance framework.
        framework: String,
        /// Severity level.
        severity: ComplianceSeverity,
    },
    /// Industry-specific regulation.
    IndustryRegulation {
        /// Industry affected.
        industry: String,
        /// Regulation name.
        regulation: String,
        /// Regulatory body.
        regulatory_body: String,
    },
}

impl RegulationType {
    /// Get the regulation type name.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::AccountingStandardAdoption { .. } => "accounting_standard_adoption",
            Self::TaxRateChange { .. } => "tax_rate_change",
            Self::NewComplianceRequirement { .. } => "new_compliance_requirement",
            Self::IndustryRegulation { .. } => "industry_regulation",
        }
    }

    /// Get the standard or regulation name.
    pub fn name(&self) -> &str {
        match self {
            Self::AccountingStandardAdoption { standard, .. } => standard,
            Self::TaxRateChange { tax_type, .. } => tax_type,
            Self::NewComplianceRequirement { requirement, .. } => requirement,
            Self::IndustryRegulation { regulation, .. } => regulation,
        }
    }
}

/// Accounting framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AccountingFramework {
    /// US Generally Accepted Accounting Principles.
    #[default]
    UsGaap,
    /// International Financial Reporting Standards.
    Ifrs,
    /// Dual reporting (both frameworks).
    DualReporting,
}

/// Compliance severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceSeverity {
    /// Minor compliance requirement.
    Minor,
    /// Standard compliance requirement.
    #[default]
    Standard,
    /// Major compliance requirement.
    Major,
    /// Critical compliance requirement.
    Critical,
}

impl ComplianceSeverity {
    /// Get the effort multiplier for compliance.
    pub fn effort_multiplier(&self) -> f64 {
        match self {
            Self::Minor => 1.1,
            Self::Standard => 1.3,
            Self::Major => 1.6,
            Self::Critical => 2.0,
        }
    }
}

/// Impact of a regulatory change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryImpact {
    /// Impact area.
    pub area: ImpactArea,
    /// Impact description.
    pub description: String,
    /// Magnitude of impact (0.0 to 1.0).
    pub magnitude: f64,
    /// Affected metrics.
    #[serde(default)]
    pub affected_metrics: Vec<String>,
}

/// Impact area for regulatory changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactArea {
    /// Balance sheet impact.
    BalanceSheet,
    /// Income statement impact.
    IncomeStatement,
    /// Cash flow impact.
    CashFlow,
    /// Disclosure requirements.
    Disclosures,
    /// Internal controls.
    InternalControls,
    /// IT systems.
    Systems,
    /// Processes.
    Processes,
    /// Audit procedures.
    AuditProcedures,
}

// =============================================================================
// Audit Focus Events
// =============================================================================

/// Audit focus shift event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFocusEvent {
    /// Event identifier.
    pub event_id: String,
    /// Focus type.
    pub focus_type: AuditFocusType,
    /// Effective date.
    pub effective_date: NaiveDate,
    /// Priority level (1 = highest).
    #[serde(default = "default_priority")]
    pub priority_level: u8,
    /// Risk areas affected.
    #[serde(default)]
    pub risk_areas: Vec<String>,
    /// Accounts requiring additional procedures.
    #[serde(default)]
    pub accounts_with_additional_procedures: Vec<String>,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
}

fn default_priority() -> u8 {
    3
}

impl AuditFocusEvent {
    /// Create a new audit focus event.
    pub fn new(
        event_id: impl Into<String>,
        focus_type: AuditFocusType,
        effective_date: NaiveDate,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            focus_type,
            effective_date,
            priority_level: 3,
            risk_areas: Vec::new(),
            accounts_with_additional_procedures: Vec::new(),
            description: None,
        }
    }

    /// Check if the focus is active at a given date.
    pub fn is_active_at(&self, date: NaiveDate) -> bool {
        date >= self.effective_date
    }
}

/// Type of audit focus shift.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditFocusType {
    /// Risk-based shift in audit focus.
    RiskBasedShift {
        /// Trigger for the shift.
        trigger: String,
        /// New focus areas.
        new_focus_areas: Vec<String>,
    },
    /// Response to industry trend.
    IndustryTrendResponse {
        /// Industry trend.
        trend: String,
        /// Affected procedures.
        affected_procedures: Vec<String>,
    },
    /// Follow-up on prior year findings.
    PriorYearFindingFollowUp {
        /// Finding IDs being followed up.
        finding_ids: Vec<String>,
    },
    /// Regulatory-driven focus.
    RegulatoryDrivenFocus {
        /// Regulation driving the focus.
        regulation: String,
        /// Required procedures.
        required_procedures: Vec<String>,
    },
    /// Fraud risk response.
    FraudRiskResponse {
        /// Risk indicators identified.
        risk_indicators: Vec<String>,
        /// Response procedures.
        response_procedures: Vec<String>,
    },
}

impl AuditFocusType {
    /// Get the focus type name.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::RiskBasedShift { .. } => "risk_based_shift",
            Self::IndustryTrendResponse { .. } => "industry_trend_response",
            Self::PriorYearFindingFollowUp { .. } => "prior_year_finding_followup",
            Self::RegulatoryDrivenFocus { .. } => "regulatory_driven_focus",
            Self::FraudRiskResponse { .. } => "fraud_risk_response",
        }
    }
}

// =============================================================================
// Regulatory Calendar
// =============================================================================

/// A calendar of scheduled regulatory events.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegulatoryCalendar {
    /// Regulatory events.
    #[serde(default)]
    pub regulatory_events: Vec<RegulatoryDriftEvent>,
    /// Audit focus events.
    #[serde(default)]
    pub audit_focus_events: Vec<AuditFocusEvent>,
}

impl RegulatoryCalendar {
    /// Create a new empty calendar.
    pub fn new() -> Self {
        Self {
            regulatory_events: Vec::new(),
            audit_focus_events: Vec::new(),
        }
    }

    /// Create a US GAAP 2024 preset calendar.
    pub fn us_gaap_2024() -> Self {
        Self {
            regulatory_events: vec![
                // ASC 842 is already effective, but can model ongoing implementation
                RegulatoryDriftEvent {
                    event_id: "REG-842-2024".to_string(),
                    regulation_type: RegulationType::AccountingStandardAdoption {
                        standard: "ASC 842".to_string(),
                        framework: AccountingFramework::UsGaap,
                        topic: "leases".to_string(),
                    },
                    effective_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    announcement_date: None,
                    transition_period_days: 0,
                    affected_accounts: vec![
                        "1600".to_string(), // ROU Assets
                        "2300".to_string(), // Lease Liabilities
                    ],
                    impacts: vec![RegulatoryImpact {
                        area: ImpactArea::BalanceSheet,
                        description: "Recognition of ROU assets and lease liabilities".to_string(),
                        magnitude: 0.3,
                        affected_metrics: vec![
                            "total_assets".to_string(),
                            "total_liabilities".to_string(),
                        ],
                    }],
                    description: Some("ASC 842 Leases implementation".to_string()),
                },
            ],
            audit_focus_events: vec![AuditFocusEvent {
                event_id: "AF-CYBER-2024".to_string(),
                focus_type: AuditFocusType::IndustryTrendResponse {
                    trend: "Increased cybersecurity risks".to_string(),
                    affected_procedures: vec![
                        "IT general controls testing".to_string(),
                        "Cybersecurity disclosure review".to_string(),
                    ],
                },
                effective_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                priority_level: 2,
                risk_areas: vec!["IT controls".to_string(), "Data security".to_string()],
                accounts_with_additional_procedures: Vec::new(),
                description: Some("Enhanced focus on cybersecurity".to_string()),
            }],
        }
    }

    /// Create an IFRS 2024 preset calendar.
    pub fn ifrs_2024() -> Self {
        Self {
            regulatory_events: vec![RegulatoryDriftEvent {
                event_id: "REG-IFRS16-2024".to_string(),
                regulation_type: RegulationType::AccountingStandardAdoption {
                    standard: "IFRS 16".to_string(),
                    framework: AccountingFramework::Ifrs,
                    topic: "leases".to_string(),
                },
                effective_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                announcement_date: None,
                transition_period_days: 0,
                affected_accounts: vec!["1600".to_string(), "2300".to_string()],
                impacts: vec![],
                description: Some("IFRS 16 Leases implementation".to_string()),
            }],
            audit_focus_events: Vec::new(),
        }
    }

    /// Add a regulatory event.
    pub fn add_regulatory_event(&mut self, event: RegulatoryDriftEvent) {
        self.regulatory_events.push(event);
    }

    /// Add an audit focus event.
    pub fn add_audit_focus_event(&mut self, event: AuditFocusEvent) {
        self.audit_focus_events.push(event);
    }

    /// Get regulatory events effective on a given date.
    pub fn regulatory_events_on_date(&self, date: NaiveDate) -> Vec<&RegulatoryDriftEvent> {
        self.regulatory_events
            .iter()
            .filter(|e| e.effective_date == date)
            .collect()
    }

    /// Get regulatory events active at a given date.
    pub fn active_regulatory_events_at(&self, date: NaiveDate) -> Vec<&RegulatoryDriftEvent> {
        self.regulatory_events
            .iter()
            .filter(|e| e.is_active_at(date))
            .collect()
    }

    /// Get audit focus events active at a given date.
    pub fn active_audit_focus_at(&self, date: NaiveDate) -> Vec<&AuditFocusEvent> {
        self.audit_focus_events
            .iter()
            .filter(|e| e.is_active_at(date))
            .collect()
    }

    /// Get events in transition at a given date.
    pub fn events_in_transition_at(&self, date: NaiveDate) -> Vec<&RegulatoryDriftEvent> {
        self.regulatory_events
            .iter()
            .filter(|e| e.is_in_transition_at(date))
            .collect()
    }

    /// Get all affected accounts at a given date.
    pub fn affected_accounts_at(&self, date: NaiveDate) -> Vec<String> {
        let mut accounts: Vec<String> = self
            .regulatory_events
            .iter()
            .filter(|e| e.is_active_at(date))
            .flat_map(|e| e.affected_accounts.iter().cloned())
            .collect();
        accounts.sort();
        accounts.dedup();
        accounts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regulatory_event_creation() {
        let event = RegulatoryDriftEvent::new(
            "REG-001",
            RegulationType::AccountingStandardAdoption {
                standard: "ASC 842".to_string(),
                framework: AccountingFramework::UsGaap,
                topic: "leases".to_string(),
            },
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert_eq!(event.event_id, "REG-001");
        assert_eq!(
            event.regulation_type.type_name(),
            "accounting_standard_adoption"
        );
    }

    #[test]
    fn test_regulatory_event_active() {
        let event = RegulatoryDriftEvent {
            event_id: "REG-001".to_string(),
            regulation_type: RegulationType::TaxRateChange {
                tax_type: "corporate_income".to_string(),
                old_rate: Decimal::from_str_exact("0.21").unwrap(),
                new_rate: Decimal::from_str_exact("0.25").unwrap(),
                jurisdiction: "US".to_string(),
            },
            effective_date: NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            announcement_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            transition_period_days: 30,
            affected_accounts: vec!["5100".to_string()],
            impacts: Vec::new(),
            description: None,
        };

        assert!(!event.is_active_at(NaiveDate::from_ymd_opt(2024, 5, 31).unwrap()));
        assert!(event.is_active_at(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()));
        assert!(event.is_in_transition_at(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()));
        assert!(!event.is_in_transition_at(NaiveDate::from_ymd_opt(2024, 7, 15).unwrap()));
    }

    #[test]
    fn test_audit_focus_event() {
        let event = AuditFocusEvent::new(
            "AF-001",
            AuditFocusType::FraudRiskResponse {
                risk_indicators: vec!["unusual_transactions".to_string()],
                response_procedures: vec!["extended_testing".to_string()],
            },
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
        );

        assert!(!event.is_active_at(NaiveDate::from_ymd_opt(2024, 2, 28).unwrap()));
        assert!(event.is_active_at(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()));
    }

    #[test]
    fn test_regulatory_calendar_preset() {
        let calendar = RegulatoryCalendar::us_gaap_2024();

        assert!(!calendar.regulatory_events.is_empty());
        assert!(!calendar.audit_focus_events.is_empty());

        let active =
            calendar.active_regulatory_events_at(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap());
        assert!(!active.is_empty());
    }
}
