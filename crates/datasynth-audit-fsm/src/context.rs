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
}

impl EngagementContext {
    /// Create a minimal test context.
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
        }
    }
}
