//! Engagement context for FSM engine.

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

    // Configuration flags
    /// Whether the entity is listed on a US exchange (triggers PCAOB/SOX paths).
    pub is_us_listed: bool,
    /// Entity codes participating in the engagement (e.g. group audit).
    pub entity_codes: Vec<String>,
}

impl EngagementContext {
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

            // Flags
            is_us_listed: false,
            entity_codes: vec!["TEST01".into()],
        }
    }
}
