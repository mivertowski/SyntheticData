//! Shared test helpers for audit generator tests.
//!
//! Provides common test fixtures to avoid duplication across
//! finding, workpaper, risk, and judgment generator tests.

use chrono::NaiveDate;
use rust_decimal::Decimal;

use datasynth_core::models::audit::{AuditEngagement, EngagementType};

/// Create a standard test engagement for audit generator unit tests.
///
/// Returns an `AuditEngagement` for entity "ENTITY001" / "Test Company Inc."
/// with a fiscal year of 2025, materiality of $1,000,000 based on Total Revenue,
/// and a standard audit timeline (Oct 2025 through Mar 2026).
pub(crate) fn create_test_engagement() -> AuditEngagement {
    AuditEngagement::new(
        "ENTITY001",
        "Test Company Inc.",
        EngagementType::AnnualAudit,
        2025,
        NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
    )
    .with_materiality(
        Decimal::new(1_000_000, 0),
        0.75,
        0.05,
        "Total Revenue",
        0.005,
    )
    .with_timeline(
        NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 10, 31).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
        NaiveDate::from_ymd_opt(2026, 2, 15).unwrap(),
        NaiveDate::from_ymd_opt(2026, 2, 16).unwrap(),
        NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
    )
    .with_team(
        "PARTNER001",
        "John Partner",
        "MANAGER001",
        "Jane Manager",
        vec!["SENIOR001".into(), "STAFF001".into()],
    )
}
