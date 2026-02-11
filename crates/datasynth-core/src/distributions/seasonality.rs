//! Industry-specific seasonality patterns for transaction generation.
//!
//! Defines seasonal events and multipliers for different industries,
//! enabling realistic volume variations throughout the year.

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::models::IndustrySector;

/// A seasonal event that affects transaction volume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalEvent {
    /// Name of the event.
    pub name: String,
    /// Start month (1-12).
    pub start_month: u8,
    /// Start day of the month.
    pub start_day: u8,
    /// End month (1-12).
    pub end_month: u8,
    /// End day of the month.
    pub end_day: u8,
    /// Volume multiplier during this event.
    pub multiplier: f64,
    /// Priority for overlapping events (higher = takes precedence).
    pub priority: u8,
}

impl SeasonalEvent {
    /// Create a new seasonal event.
    pub fn new(
        name: impl Into<String>,
        start_month: u8,
        start_day: u8,
        end_month: u8,
        end_day: u8,
        multiplier: f64,
    ) -> Self {
        Self {
            name: name.into(),
            start_month,
            start_day,
            end_month,
            end_day,
            multiplier,
            priority: 0,
        }
    }

    /// Set the priority for this event.
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Check if this event is active on a given date.
    pub fn is_active(&self, date: NaiveDate) -> bool {
        let month = date.month() as u8;
        let day = date.day() as u8;

        // Handle events that span year boundary (e.g., Dec 20 - Jan 5)
        if self.start_month > self.end_month {
            // Event spans year boundary
            if month > self.start_month || month < self.end_month {
                return true;
            }
            if month == self.start_month && day >= self.start_day {
                return true;
            }
            if month == self.end_month && day <= self.end_day {
                return true;
            }
            return false;
        }

        // Normal case: event within same year
        if month < self.start_month || month > self.end_month {
            return false;
        }

        if month == self.start_month && day < self.start_day {
            return false;
        }

        if month == self.end_month && day > self.end_day {
            return false;
        }

        true
    }
}

/// Industry-specific seasonality patterns.
#[derive(Debug, Clone)]
pub struct IndustrySeasonality {
    /// The industry sector.
    pub industry: IndustrySector,
    /// Seasonal events for this industry.
    pub events: Vec<SeasonalEvent>,
}

impl IndustrySeasonality {
    /// Create a new industry seasonality definition.
    pub fn new(industry: IndustrySector) -> Self {
        Self {
            industry,
            events: Vec::new(),
        }
    }

    /// Create seasonality patterns for a specific industry.
    pub fn for_industry(industry: IndustrySector) -> Self {
        match industry {
            IndustrySector::Retail => Self::retail(),
            IndustrySector::Manufacturing => Self::manufacturing(),
            IndustrySector::FinancialServices => Self::financial_services(),
            IndustrySector::Healthcare => Self::healthcare(),
            IndustrySector::Technology => Self::technology(),
            IndustrySector::ProfessionalServices => Self::professional_services(),
            IndustrySector::Energy => Self::energy(),
            IndustrySector::Transportation => Self::transportation(),
            IndustrySector::RealEstate => Self::real_estate(),
            IndustrySector::Telecommunications => Self::telecommunications(),
        }
    }

    /// Get the volume multiplier for a specific date.
    ///
    /// If multiple events are active, returns the one with highest priority.
    /// If no events are active, returns 1.0.
    pub fn get_multiplier(&self, date: NaiveDate) -> f64 {
        let active_events: Vec<&SeasonalEvent> =
            self.events.iter().filter(|e| e.is_active(date)).collect();

        if active_events.is_empty() {
            return 1.0;
        }

        // Return the multiplier from the highest priority active event
        active_events
            .into_iter()
            .max_by_key(|e| e.priority)
            .map(|e| e.multiplier)
            .unwrap_or(1.0)
    }

    /// Add a seasonal event.
    pub fn add_event(&mut self, event: SeasonalEvent) {
        self.events.push(event);
    }

    /// Retail industry seasonality patterns.
    fn retail() -> Self {
        let mut s = Self::new(IndustrySector::Retail);

        // Black Friday / Cyber Monday (Nov 20-30): 8x
        s.add_event(
            SeasonalEvent::new("Black Friday/Cyber Monday", 11, 20, 11, 30, 8.0).with_priority(10),
        );

        // Christmas rush (Dec 15-24): 6x
        s.add_event(SeasonalEvent::new("Christmas Rush", 12, 15, 12, 24, 6.0).with_priority(9));

        // Post-holiday returns (Jan 1-15): 3x
        s.add_event(SeasonalEvent::new("Post-Holiday Returns", 1, 1, 1, 15, 3.0).with_priority(7));

        // Back-to-school (Aug 1-31): 2x
        s.add_event(SeasonalEvent::new("Back-to-School", 8, 1, 8, 31, 2.0).with_priority(5));

        // Valentine's Day surge (Feb 7-14): 1.8x
        s.add_event(SeasonalEvent::new("Valentine's Day", 2, 7, 2, 14, 1.8).with_priority(4));

        // Easter season (late March - mid April): 1.5x
        s.add_event(SeasonalEvent::new("Easter Season", 3, 20, 4, 15, 1.5).with_priority(3));

        // Summer slowdown (Jun-Jul): 0.7x
        s.add_event(SeasonalEvent::new("Summer Slowdown", 6, 1, 7, 31, 0.7).with_priority(2));

        s
    }

    /// Manufacturing industry seasonality patterns.
    fn manufacturing() -> Self {
        let mut s = Self::new(IndustrySector::Manufacturing);

        // Year-end close (Dec 20-31): 4x
        s.add_event(SeasonalEvent::new("Year-End Close", 12, 20, 12, 31, 4.0).with_priority(10));

        // Q4 inventory buildup (Oct-Nov): 2x
        s.add_event(
            SeasonalEvent::new("Q4 Inventory Buildup", 10, 1, 11, 30, 2.0).with_priority(6),
        );

        // Model year transitions (Sep): 1.5x
        s.add_event(SeasonalEvent::new("Model Year Transition", 9, 1, 9, 30, 1.5).with_priority(5));

        // Spring production ramp (Mar-Apr): 1.3x
        s.add_event(
            SeasonalEvent::new("Spring Production Ramp", 3, 1, 4, 30, 1.3).with_priority(3),
        );

        // Summer slowdown/maintenance (Jul): 0.6x
        s.add_event(SeasonalEvent::new("Summer Shutdown", 7, 1, 7, 31, 0.6).with_priority(4));

        // Holiday shutdown (Dec 24-26): 0.2x
        s.add_event(SeasonalEvent::new("Holiday Shutdown", 12, 24, 12, 26, 0.2).with_priority(11));

        s
    }

    /// Financial services industry seasonality patterns.
    fn financial_services() -> Self {
        let mut s = Self::new(IndustrySector::FinancialServices);

        // Year-end (Dec 15-31): 8x
        s.add_event(SeasonalEvent::new("Year-End", 12, 15, 12, 31, 8.0).with_priority(10));

        // Q1 end (Mar 26-31): 5x
        s.add_event(SeasonalEvent::new("Q1 Close", 3, 26, 3, 31, 5.0).with_priority(9));

        // Q2 end (Jun 25-30): 5x
        s.add_event(SeasonalEvent::new("Q2 Close", 6, 25, 6, 30, 5.0).with_priority(9));

        // Q3 end (Sep 25-30): 5x
        s.add_event(SeasonalEvent::new("Q3 Close", 9, 25, 9, 30, 5.0).with_priority(9));

        // Tax deadline (Apr 10-20): 3x
        s.add_event(SeasonalEvent::new("Tax Deadline", 4, 10, 4, 20, 3.0).with_priority(7));

        // Annual audit season (Jan 15 - Feb 28): 2.5x
        s.add_event(SeasonalEvent::new("Audit Season", 1, 15, 2, 28, 2.5).with_priority(6));

        // SEC/Regulatory filing periods (Feb 1-28): 2x
        s.add_event(SeasonalEvent::new("Regulatory Filing", 2, 1, 2, 28, 2.0).with_priority(5));

        s
    }

    /// Healthcare industry seasonality patterns.
    fn healthcare() -> Self {
        let mut s = Self::new(IndustrySector::Healthcare);

        // Year-end (Dec 15-31): 3x
        s.add_event(SeasonalEvent::new("Year-End", 12, 15, 12, 31, 3.0).with_priority(10));

        // Insurance renewal/benefits enrollment (Jan 1-31): 2x
        s.add_event(SeasonalEvent::new("Insurance Enrollment", 1, 1, 1, 31, 2.0).with_priority(8));

        // Flu season (Oct-Feb): 1.5x
        s.add_event(SeasonalEvent::new("Flu Season", 10, 1, 10, 31, 1.5).with_priority(4));
        s.add_event(SeasonalEvent::new("Flu Season Extended", 11, 1, 2, 28, 1.5).with_priority(4));

        // Open enrollment period (Nov): 1.8x
        s.add_event(SeasonalEvent::new("Open Enrollment", 11, 1, 11, 30, 1.8).with_priority(6));

        // Summer elective procedure slowdown (Jun-Aug): 0.8x
        s.add_event(SeasonalEvent::new("Summer Slowdown", 6, 15, 8, 15, 0.8).with_priority(3));

        s
    }

    /// Technology industry seasonality patterns.
    fn technology() -> Self {
        let mut s = Self::new(IndustrySector::Technology);

        // Q4 enterprise deals (Dec): 4x
        s.add_event(
            SeasonalEvent::new("Q4 Enterprise Deals", 12, 1, 12, 31, 4.0).with_priority(10),
        );

        // Black Friday/holiday sales (Nov 15-30): 2x
        s.add_event(SeasonalEvent::new("Holiday Sales", 11, 15, 11, 30, 2.0).with_priority(8));

        // Back-to-school (Aug-Sep): 1.5x
        s.add_event(SeasonalEvent::new("Back-to-School", 8, 1, 9, 15, 1.5).with_priority(5));

        // Product launch seasons (Mar, Sep): 1.8x
        s.add_event(SeasonalEvent::new("Spring Launches", 3, 1, 3, 31, 1.8).with_priority(6));
        s.add_event(SeasonalEvent::new("Fall Launches", 9, 1, 9, 30, 1.8).with_priority(6));

        // Summer slowdown (Jul-Aug): 0.7x
        s.add_event(SeasonalEvent::new("Summer Slowdown", 7, 1, 8, 15, 0.7).with_priority(3));

        s
    }

    /// Professional services industry seasonality patterns.
    fn professional_services() -> Self {
        let mut s = Self::new(IndustrySector::ProfessionalServices);

        // Year-end (Dec): 3x
        s.add_event(SeasonalEvent::new("Year-End", 12, 10, 12, 31, 3.0).with_priority(10));

        // Tax season (Feb-Apr): 2.5x for accounting firms
        s.add_event(SeasonalEvent::new("Tax Season", 2, 1, 4, 15, 2.5).with_priority(8));

        // Budget season (Oct-Nov): 1.8x
        s.add_event(SeasonalEvent::new("Budget Season", 10, 1, 11, 30, 1.8).with_priority(6));

        // Summer slowdown (Jul-Aug): 0.75x
        s.add_event(SeasonalEvent::new("Summer Slowdown", 7, 1, 8, 31, 0.75).with_priority(3));

        // Holiday period (Dec 23-Jan 2): 0.3x
        s.add_event(SeasonalEvent::new("Holiday Period", 12, 23, 12, 26, 0.3).with_priority(11));

        s
    }

    /// Energy industry seasonality patterns.
    fn energy() -> Self {
        let mut s = Self::new(IndustrySector::Energy);

        // Winter heating season (Nov-Feb): 1.8x
        s.add_event(
            SeasonalEvent::new("Winter Heating Season", 11, 1, 2, 28, 1.8).with_priority(6),
        );

        // Summer cooling season (Jun-Aug): 1.5x
        s.add_event(SeasonalEvent::new("Summer Cooling Season", 6, 1, 8, 31, 1.5).with_priority(5));

        // Year-end reconciliation (Dec 15-31): 3x
        s.add_event(SeasonalEvent::new("Year-End", 12, 15, 12, 31, 3.0).with_priority(10));

        // Spring/Fall shoulder seasons: 0.8x
        s.add_event(SeasonalEvent::new("Spring Shoulder", 3, 15, 5, 15, 0.8).with_priority(3));
        s.add_event(SeasonalEvent::new("Fall Shoulder", 9, 15, 10, 15, 0.8).with_priority(3));

        s
    }

    /// Transportation industry seasonality patterns.
    fn transportation() -> Self {
        let mut s = Self::new(IndustrySector::Transportation);

        // Holiday shipping season (Nov 15 - Dec 24): 4x
        s.add_event(SeasonalEvent::new("Holiday Shipping", 11, 15, 12, 24, 4.0).with_priority(10));

        // Back-to-school (Aug): 1.5x
        s.add_event(SeasonalEvent::new("Back-to-School", 8, 1, 8, 31, 1.5).with_priority(5));

        // Summer travel season (Jun-Aug): 1.3x for passenger
        s.add_event(SeasonalEvent::new("Summer Travel", 6, 15, 8, 15, 1.3).with_priority(4));

        // Post-holiday slowdown (Jan): 0.7x
        s.add_event(SeasonalEvent::new("January Slowdown", 1, 5, 1, 31, 0.7).with_priority(3));

        s
    }

    /// Real estate industry seasonality patterns.
    fn real_estate() -> Self {
        let mut s = Self::new(IndustrySector::RealEstate);

        // Spring buying season (Mar-Jun): 2x
        s.add_event(SeasonalEvent::new("Spring Buying Season", 3, 1, 6, 30, 2.0).with_priority(6));

        // Year-end closings (Dec): 2.5x
        s.add_event(SeasonalEvent::new("Year-End Closings", 12, 1, 12, 31, 2.5).with_priority(8));

        // Summer moving season (Jun-Aug): 1.8x
        s.add_event(SeasonalEvent::new("Summer Moving", 6, 1, 8, 31, 1.8).with_priority(5));

        // Winter slowdown (Jan-Feb): 0.6x
        s.add_event(SeasonalEvent::new("Winter Slowdown", 1, 1, 2, 28, 0.6).with_priority(3));

        s
    }

    /// Telecommunications industry seasonality patterns.
    fn telecommunications() -> Self {
        let mut s = Self::new(IndustrySector::Telecommunications);

        // Holiday activations (Nov-Dec): 2x
        s.add_event(
            SeasonalEvent::new("Holiday Activations", 11, 15, 12, 31, 2.0).with_priority(8),
        );

        // Back-to-school (Aug-Sep): 1.5x
        s.add_event(SeasonalEvent::new("Back-to-School", 8, 1, 9, 15, 1.5).with_priority(5));

        // Year-end billing (Dec 15-31): 1.8x
        s.add_event(SeasonalEvent::new("Year-End Billing", 12, 15, 12, 31, 1.8).with_priority(7));

        // Q1 slowdown (Jan-Feb): 0.8x
        s.add_event(SeasonalEvent::new("Q1 Slowdown", 1, 15, 2, 28, 0.8).with_priority(3));

        s
    }
}

/// Custom seasonal event configuration for YAML/JSON input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSeasonalEventConfig {
    /// Event name.
    pub name: String,
    /// Start month (1-12).
    pub start_month: u8,
    /// Start day of month.
    pub start_day: u8,
    /// End month (1-12).
    pub end_month: u8,
    /// End day of month.
    pub end_day: u8,
    /// Volume multiplier.
    pub multiplier: f64,
    /// Priority (optional, defaults to 5).
    #[serde(default = "default_priority")]
    pub priority: u8,
}

fn default_priority() -> u8 {
    5
}

impl From<CustomSeasonalEventConfig> for SeasonalEvent {
    fn from(config: CustomSeasonalEventConfig) -> Self {
        SeasonalEvent::new(
            config.name,
            config.start_month,
            config.start_day,
            config.end_month,
            config.end_day,
            config.multiplier,
        )
        .with_priority(config.priority)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_seasonal_event_active() {
        let event = SeasonalEvent::new("Test Event", 11, 20, 11, 30, 2.0);

        // Active dates
        assert!(event.is_active(NaiveDate::from_ymd_opt(2024, 11, 20).unwrap()));
        assert!(event.is_active(NaiveDate::from_ymd_opt(2024, 11, 25).unwrap()));
        assert!(event.is_active(NaiveDate::from_ymd_opt(2024, 11, 30).unwrap()));

        // Inactive dates
        assert!(!event.is_active(NaiveDate::from_ymd_opt(2024, 11, 19).unwrap()));
        assert!(!event.is_active(NaiveDate::from_ymd_opt(2024, 12, 1).unwrap()));
        assert!(!event.is_active(NaiveDate::from_ymd_opt(2024, 10, 25).unwrap()));
    }

    #[test]
    fn test_year_spanning_event() {
        let event = SeasonalEvent::new("Holiday Period", 12, 20, 1, 5, 0.3);

        // Active in December
        assert!(event.is_active(NaiveDate::from_ymd_opt(2024, 12, 20).unwrap()));
        assert!(event.is_active(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()));

        // Active in January
        assert!(event.is_active(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
        assert!(event.is_active(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()));

        // Inactive
        assert!(!event.is_active(NaiveDate::from_ymd_opt(2024, 12, 19).unwrap()));
        assert!(!event.is_active(NaiveDate::from_ymd_opt(2024, 1, 6).unwrap()));
        assert!(!event.is_active(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()));
    }

    #[test]
    fn test_retail_seasonality() {
        let seasonality = IndustrySeasonality::for_industry(IndustrySector::Retail);

        // Black Friday should be 8x
        let black_friday = NaiveDate::from_ymd_opt(2024, 11, 25).unwrap();
        assert!((seasonality.get_multiplier(black_friday) - 8.0).abs() < 0.01);

        // Regular day should be 1x
        let regular_day = NaiveDate::from_ymd_opt(2024, 5, 15).unwrap();
        assert!((seasonality.get_multiplier(regular_day) - 1.0).abs() < 0.01);

        // Summer slowdown should be 0.7x
        let summer = NaiveDate::from_ymd_opt(2024, 7, 15).unwrap();
        assert!((seasonality.get_multiplier(summer) - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_financial_services_seasonality() {
        let seasonality = IndustrySeasonality::for_industry(IndustrySector::FinancialServices);

        // Year-end should be 8x
        let year_end = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        assert!((seasonality.get_multiplier(year_end) - 8.0).abs() < 0.01);

        // Quarter-end should be 5x
        let q1_end = NaiveDate::from_ymd_opt(2024, 3, 28).unwrap();
        assert!((seasonality.get_multiplier(q1_end) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_priority_handling() {
        let mut s = IndustrySeasonality::new(IndustrySector::Retail);

        // Add two overlapping events
        s.add_event(SeasonalEvent::new("Low Priority", 12, 1, 12, 31, 2.0).with_priority(1));
        s.add_event(SeasonalEvent::new("High Priority", 12, 15, 12, 25, 5.0).with_priority(10));

        // In overlap period, high priority should win
        let overlap = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        assert!((s.get_multiplier(overlap) - 5.0).abs() < 0.01);

        // Outside overlap, low priority applies
        let low_only = NaiveDate::from_ymd_opt(2024, 12, 5).unwrap();
        assert!((s.get_multiplier(low_only) - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_all_industries_have_events() {
        let industries = [
            IndustrySector::Retail,
            IndustrySector::Manufacturing,
            IndustrySector::FinancialServices,
            IndustrySector::Healthcare,
            IndustrySector::Technology,
            IndustrySector::ProfessionalServices,
            IndustrySector::Energy,
            IndustrySector::Transportation,
            IndustrySector::RealEstate,
            IndustrySector::Telecommunications,
        ];

        for industry in industries {
            let s = IndustrySeasonality::for_industry(industry);
            assert!(
                !s.events.is_empty(),
                "Industry {:?} should have seasonal events",
                industry
            );
        }
    }
}
