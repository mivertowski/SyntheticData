//! Temporal versioning for compliance standards.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Impact level of a standard change on generated data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeImpact {
    /// Cosmetic or disclosure-only changes
    Low,
    /// Changes to recognition or measurement
    Medium,
    /// Fundamental restructuring (e.g., IAS 39 → IFRS 9)
    High,
    /// Complete replacement of a standard
    Replacement,
}

impl ChangeImpact {
    /// Returns a numeric score for ML features.
    pub fn score(&self) -> f64 {
        match self {
            Self::Low => 0.25,
            Self::Medium => 0.50,
            Self::High => 0.75,
            Self::Replacement => 1.0,
        }
    }
}

/// A specific version of a standard with temporal bounds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalVersion {
    /// Version identifier (e.g., "2018", "2020-amended", "2023-revised")
    pub version_id: String,
    /// Date this version was issued/published
    pub issued_date: Option<NaiveDate>,
    /// Date this version becomes available for early adoption
    pub early_adoption_from: Option<NaiveDate>,
    /// Date this version becomes mandatory (global default)
    pub effective_from: NaiveDate,
    /// Date this version is superseded (None = currently active)
    pub superseded_at: Option<NaiveDate>,
    /// Per-jurisdiction effective date overrides
    pub jurisdiction_overrides: HashMap<String, NaiveDate>,
    /// Key changes from the previous version
    pub change_summary: Vec<String>,
    /// Impact level on generated data
    pub impact: ChangeImpact,
}

impl TemporalVersion {
    /// Creates a new temporal version with required fields.
    pub fn new(
        version_id: impl Into<String>,
        effective_from: NaiveDate,
        impact: ChangeImpact,
    ) -> Self {
        Self {
            version_id: version_id.into(),
            issued_date: None,
            early_adoption_from: None,
            effective_from,
            superseded_at: None,
            jurisdiction_overrides: HashMap::new(),
            change_summary: Vec::new(),
            impact,
        }
    }

    /// Sets the issued date.
    pub fn with_issued_date(mut self, date: NaiveDate) -> Self {
        self.issued_date = Some(date);
        self
    }

    /// Sets the early adoption date.
    pub fn with_early_adoption(mut self, date: NaiveDate) -> Self {
        self.early_adoption_from = Some(date);
        self
    }

    /// Sets the superseded date.
    pub fn superseded_at(mut self, date: NaiveDate) -> Self {
        self.superseded_at = Some(date);
        self
    }

    /// Adds a jurisdiction-specific effective date override.
    pub fn with_jurisdiction_override(mut self, country: &str, date: NaiveDate) -> Self {
        self.jurisdiction_overrides
            .insert(country.to_string(), date);
        self
    }

    /// Adds a change summary item.
    pub fn with_change(mut self, summary: impl Into<String>) -> Self {
        self.change_summary.push(summary.into());
        self
    }

    /// Returns whether this version is active at a given date (global, ignoring jurisdiction overrides).
    pub fn is_active_at(&self, date: NaiveDate) -> bool {
        date >= self.effective_from && self.superseded_at.is_none_or(|sup| date < sup)
    }

    /// Returns whether this version is active at a given date for a specific jurisdiction.
    pub fn is_active_at_in(&self, date: NaiveDate, country: &str) -> bool {
        let effective = self
            .jurisdiction_overrides
            .get(country)
            .copied()
            .unwrap_or(self.effective_from);
        date >= effective && self.superseded_at.is_none_or(|sup| date < sup)
    }

    /// Returns the effective date for a specific jurisdiction.
    pub fn effective_date_for(&self, country: &str) -> NaiveDate {
        self.jurisdiction_overrides
            .get(country)
            .copied()
            .unwrap_or(self.effective_from)
    }

    /// Returns the number of days this version has been active as of a given date.
    pub fn days_active_at(&self, date: NaiveDate) -> Option<i64> {
        if self.is_active_at(date) {
            Some((date - self.effective_from).num_days())
        } else {
            None
        }
    }
}

/// A resolved standard pinned to a specific version for generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedStandard {
    /// The standard identifier
    pub id: super::StandardId,
    /// The resolved version
    pub version: TemporalVersion,
    /// Local designation (e.g., "Ind AS 116" for IFRS 16 in India)
    pub local_designation: Option<String>,
    /// Entity codes this standard applies to
    pub applicable_entities: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    #[test]
    fn test_version_active_at() {
        let v = TemporalVersion::new("2019", date(2019, 1, 1), ChangeImpact::High);
        assert!(!v.is_active_at(date(2018, 12, 31)));
        assert!(v.is_active_at(date(2019, 1, 1)));
        assert!(v.is_active_at(date(2025, 6, 30)));
    }

    #[test]
    fn test_version_superseded() {
        let v = TemporalVersion::new("2019", date(2019, 1, 1), ChangeImpact::High)
            .superseded_at(date(2023, 1, 1));
        assert!(v.is_active_at(date(2022, 12, 31)));
        assert!(!v.is_active_at(date(2023, 1, 1)));
    }

    #[test]
    fn test_jurisdiction_override() {
        let v = TemporalVersion::new("2019", date(2019, 1, 1), ChangeImpact::High)
            .with_jurisdiction_override("IN", date(2020, 4, 1));

        // Globally active from 2019
        assert!(v.is_active_at_in(date(2019, 6, 1), "US"));
        // India delayed to April 2020
        assert!(!v.is_active_at_in(date(2019, 6, 1), "IN"));
        assert!(v.is_active_at_in(date(2020, 6, 1), "IN"));
    }

    #[test]
    fn test_change_impact_score() {
        assert!((ChangeImpact::Low.score() - 0.25).abs() < f64::EPSILON);
        assert!((ChangeImpact::Replacement.score() - 1.0).abs() < f64::EPSILON);
    }
}
