//! Engagement context for FSM engine.

use std::collections::HashMap;

use chrono::NaiveDate;
use rust_decimal::Decimal;

/// Context from the broader generation run, passed to the FSM engine.
pub struct EngagementContext {
    pub company_code: String,
    pub company_name: String,
    pub fiscal_year: i32,
    pub currency: String,
    pub total_revenue: Decimal,
    pub total_assets: Decimal,
    pub engagement_start: NaiveDate,
    pub report_date: NaiveDate,

    // Financial data (for MaterialityGenerator, GoingConcernGenerator)
    /// Pre-tax income used as a materiality benchmark (ISA 320).
    pub pretax_income: Decimal,
    /// Total equity for ratio analysis.
    pub equity: Decimal,
    /// Gross profit for margin-based analytical procedures.
    pub gross_profit: Decimal,
    /// Working capital (current assets minus current liabilities).
    pub working_capital: Decimal,
    /// Operating cash flow for going-concern indicators.
    pub operating_cash_flow: Decimal,
    /// Total debt for leverage and going-concern analysis.
    pub total_debt: Decimal,

    // Team/personnel (for WorkpaperGenerator, EvidenceGenerator, FindingGenerator)
    /// IDs of team members available for assignment to workpapers and steps.
    pub team_member_ids: Vec<String>,
    /// Pairs of (member_id, display_name) for richer output.
    pub team_member_pairs: Vec<(String, String)>,

    // Reference data (for RiskAssessmentGenerator, AnalyticalProcedureGenerator)
    /// GL account codes available for risk assessment and sampling.
    pub accounts: Vec<String>,
    /// Vendor names for AP-related procedures.
    pub vendor_names: Vec<String>,
    /// Customer names for AR-related procedures.
    pub customer_names: Vec<String>,

    // Cross-referencing data (for artifact coherence)
    /// Sample journal entry IDs for evidence tracing (not full JEs -- just IDs and amounts).
    pub journal_entry_ids: Vec<String>,
    /// Account balances for risk weighting.
    pub account_balances: HashMap<String, f64>,
    /// Internal control IDs for finding-to-control linking.
    pub control_ids: Vec<String>,
    /// Injected anomaly references for finding-to-anomaly linking.
    pub anomaly_refs: Vec<String>,

    // Configuration flags
    /// Whether the entity is listed on a US exchange (triggers PCAOB/SOX paths).
    pub is_us_listed: bool,
    /// Entity codes participating in the engagement (e.g. group audit).
    pub entity_codes: Vec<String>,
}

impl EngagementContext {
    /// Create a test context with anomaly references for finding linkage testing.
    pub fn test_with_anomalies() -> Self {
        let mut ctx = Self::test_default();
        ctx.anomaly_refs = vec!["ANOM-001".into(), "ANOM-002".into()];
        ctx
    }

    /// Create a minimal test context with sensible defaults.
    pub fn test_default() -> Self {
        Self {
            company_code: "TEST01".into(),
            company_name: "Test Corp".into(),
            fiscal_year: 2025,
            currency: "USD".into(),
            total_revenue: Decimal::new(10_000_000, 0),
            total_assets: Decimal::new(50_000_000, 0),
            engagement_start: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            report_date: NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),

            // Financial data
            pretax_income: Decimal::ZERO,
            equity: Decimal::ZERO,
            gross_profit: Decimal::ZERO,
            working_capital: Decimal::ZERO,
            operating_cash_flow: Decimal::ZERO,
            total_debt: Decimal::ZERO,

            // Team — two members for test coverage
            team_member_ids: vec!["TM001".into(), "TM002".into()],
            team_member_pairs: vec![
                ("TM001".into(), "Alice Auditor".into()),
                ("TM002".into(), "Bob Reviewer".into()),
            ],

            // Reference data — a handful of accounts for testing
            accounts: vec![
                "1100".into(), // AR Control
                "2000".into(), // AP Control
                "4000".into(), // Revenue
                "5000".into(), // COGS
            ],
            vendor_names: Vec::new(),
            customer_names: Vec::new(),

            // Cross-referencing data
            journal_entry_ids: vec![
                "JE-2025-0001".into(),
                "JE-2025-0002".into(),
                "JE-2025-0003".into(),
                "JE-2025-0004".into(),
            ],
            account_balances: HashMap::from([
                ("1100".into(), 1_250_000.0), // AR Control
                ("2000".into(), 875_000.0),   // AP Control
                ("4000".into(), 5_000_000.0), // Revenue
                ("5000".into(), 3_200_000.0), // COGS
            ]),
            control_ids: vec![
                "C001".into(), // Three-way match
                "C010".into(), // JE approval
                "C020".into(), // Bank reconciliation
            ],
            anomaly_refs: Vec::new(),

            // Flags
            is_us_listed: false,
            entity_codes: vec!["TEST01".into()],
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_has_journal_entry_ids() {
        let ctx = EngagementContext::test_default();
        assert_eq!(ctx.journal_entry_ids.len(), 4);
        assert!(ctx.journal_entry_ids[0].starts_with("JE-"));
    }

    #[test]
    fn test_default_has_account_balances() {
        let ctx = EngagementContext::test_default();
        assert_eq!(ctx.account_balances.len(), 4);
        assert!(ctx.account_balances.contains_key("1100"));
        assert!(ctx.account_balances.contains_key("4000"));
        assert!(*ctx.account_balances.get("4000").unwrap() > 0.0);
    }

    #[test]
    fn test_default_has_control_ids() {
        let ctx = EngagementContext::test_default();
        assert_eq!(ctx.control_ids.len(), 3);
        assert!(ctx.control_ids.contains(&"C001".to_string()));
    }

    #[test]
    fn test_default_anomaly_refs_empty() {
        let ctx = EngagementContext::test_default();
        assert!(ctx.anomaly_refs.is_empty());
    }

    #[test]
    fn test_with_anomalies_has_refs() {
        let ctx = EngagementContext::test_with_anomalies();
        assert_eq!(ctx.anomaly_refs.len(), 2);
        assert_eq!(ctx.anomaly_refs[0], "ANOM-001");
    }
}
