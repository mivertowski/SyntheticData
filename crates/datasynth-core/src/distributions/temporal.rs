//! Temporal distribution samplers for realistic posting patterns.
//!
//! Implements seasonality, working hour patterns, and period-end spikes
//! commonly observed in enterprise accounting systems.

use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Weekday};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use super::holidays::HolidayCalendar;
use super::period_end::PeriodEndDynamics;
use super::seasonality::IndustrySeasonality;

/// Configuration for seasonality patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalityConfig {
    /// Enable month-end volume spikes
    pub month_end_spike: bool,
    /// Month-end spike multiplier (e.g., 2.5 = 2.5x normal volume)
    pub month_end_multiplier: f64,
    /// Days before month-end to start spike
    pub month_end_lead_days: u32,

    /// Enable quarter-end spikes
    pub quarter_end_spike: bool,
    /// Quarter-end spike multiplier
    pub quarter_end_multiplier: f64,

    /// Enable year-end spikes
    pub year_end_spike: bool,
    /// Year-end spike multiplier
    pub year_end_multiplier: f64,

    /// Activity level on weekends (0.0 = no activity, 1.0 = normal)
    pub weekend_activity: f64,
    /// Activity level on holidays
    pub holiday_activity: f64,

    /// Enable day-of-week patterns (Monday catch-up, Friday slowdown)
    pub day_of_week_patterns: bool,
    /// Monday activity multiplier (catch-up from weekend)
    pub monday_multiplier: f64,
    /// Tuesday activity multiplier
    pub tuesday_multiplier: f64,
    /// Wednesday activity multiplier
    pub wednesday_multiplier: f64,
    /// Thursday activity multiplier
    pub thursday_multiplier: f64,
    /// Friday activity multiplier (early departures)
    pub friday_multiplier: f64,
}

impl Default for SeasonalityConfig {
    fn default() -> Self {
        Self {
            month_end_spike: true,
            month_end_multiplier: 2.5,
            month_end_lead_days: 5,
            quarter_end_spike: true,
            quarter_end_multiplier: 4.0,
            year_end_spike: true,
            year_end_multiplier: 6.0,
            weekend_activity: 0.1,
            holiday_activity: 0.05,
            // Day-of-week patterns: humans work differently across the week
            day_of_week_patterns: true,
            monday_multiplier: 1.3,    // Catch-up from weekend backlog
            tuesday_multiplier: 1.1,   // Still catching up
            wednesday_multiplier: 1.0, // Midweek normal
            thursday_multiplier: 1.0,  // Midweek normal
            friday_multiplier: 0.85,   // Early departures, winding down
        }
    }
}

/// Configuration for working hours pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingHoursConfig {
    /// Start of working day (hour, 0-23)
    pub day_start: u8,
    /// End of working day (hour, 0-23)
    pub day_end: u8,
    /// Peak hours during the day
    pub peak_hours: Vec<u8>,
    /// Weight for peak hours (multiplier)
    pub peak_weight: f64,
    /// Probability of after-hours posting
    pub after_hours_probability: f64,
}

impl Default for WorkingHoursConfig {
    fn default() -> Self {
        Self {
            day_start: 8,
            day_end: 18,
            peak_hours: vec![9, 10, 11, 14, 15, 16],
            peak_weight: 1.5,
            after_hours_probability: 0.05,
        }
    }
}

/// Configuration for intra-day posting patterns.
///
/// Defines segments of the business day with different activity multipliers,
/// allowing for realistic modeling of morning spikes, lunch dips, and end-of-day rushes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntraDayPatterns {
    /// Whether intra-day patterns are enabled.
    pub enabled: bool,
    /// Time segments with activity multipliers.
    pub segments: Vec<IntraDaySegment>,
}

impl Default for IntraDayPatterns {
    fn default() -> Self {
        Self {
            enabled: true,
            segments: vec![
                IntraDaySegment {
                    name: "morning_spike".to_string(),
                    start: NaiveTime::from_hms_opt(8, 30, 0).expect("valid date/time components"),
                    end: NaiveTime::from_hms_opt(10, 0, 0).expect("valid date/time components"),
                    multiplier: 1.8,
                    posting_type: PostingType::Both,
                },
                IntraDaySegment {
                    name: "mid_morning".to_string(),
                    start: NaiveTime::from_hms_opt(10, 0, 0).expect("valid date/time components"),
                    end: NaiveTime::from_hms_opt(12, 0, 0).expect("valid date/time components"),
                    multiplier: 1.2,
                    posting_type: PostingType::Both,
                },
                IntraDaySegment {
                    name: "lunch_dip".to_string(),
                    start: NaiveTime::from_hms_opt(12, 0, 0).expect("valid date/time components"),
                    end: NaiveTime::from_hms_opt(13, 30, 0).expect("valid date/time components"),
                    multiplier: 0.4,
                    posting_type: PostingType::Human,
                },
                IntraDaySegment {
                    name: "afternoon".to_string(),
                    start: NaiveTime::from_hms_opt(13, 30, 0).expect("valid date/time components"),
                    end: NaiveTime::from_hms_opt(16, 0, 0).expect("valid date/time components"),
                    multiplier: 1.1,
                    posting_type: PostingType::Both,
                },
                IntraDaySegment {
                    name: "eod_rush".to_string(),
                    start: NaiveTime::from_hms_opt(16, 0, 0).expect("valid date/time components"),
                    end: NaiveTime::from_hms_opt(17, 30, 0).expect("valid date/time components"),
                    multiplier: 1.5,
                    posting_type: PostingType::Both,
                },
            ],
        }
    }
}

impl IntraDayPatterns {
    /// Creates intra-day patterns with no segments (disabled).
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            segments: Vec::new(),
        }
    }

    /// Creates patterns with custom segments.
    pub fn with_segments(segments: Vec<IntraDaySegment>) -> Self {
        Self {
            enabled: true,
            segments,
        }
    }

    /// Gets the multiplier for a given time based on posting type.
    pub fn get_multiplier(&self, time: NaiveTime, is_human: bool) -> f64 {
        if !self.enabled {
            return 1.0;
        }

        for segment in &self.segments {
            if time >= segment.start && time < segment.end {
                // Check if this segment applies to the posting type
                let applies = match segment.posting_type {
                    PostingType::Human => is_human,
                    PostingType::System => !is_human,
                    PostingType::Both => true,
                };
                if applies {
                    return segment.multiplier;
                }
            }
        }

        1.0 // Default multiplier if no segment matches
    }
}

/// A segment of the business day with specific activity patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntraDaySegment {
    /// Name of the segment (e.g., "morning_spike", "lunch_dip").
    pub name: String,
    /// Start time of the segment.
    pub start: NaiveTime,
    /// End time of the segment.
    pub end: NaiveTime,
    /// Activity multiplier for this segment (1.0 = normal).
    pub multiplier: f64,
    /// Type of postings this segment applies to.
    pub posting_type: PostingType,
}

/// Type of posting for intra-day pattern matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostingType {
    /// Human/manual postings only.
    Human,
    /// System/automated postings only.
    System,
    /// Both human and system postings.
    Both,
}

/// Sampler for temporal patterns in transaction generation.
pub struct TemporalSampler {
    rng: ChaCha8Rng,
    seasonality_config: SeasonalityConfig,
    working_hours_config: WorkingHoursConfig,
    /// List of holiday dates (legacy)
    holidays: Vec<NaiveDate>,
    /// Industry-specific seasonality patterns (optional).
    industry_seasonality: Option<IndustrySeasonality>,
    /// Regional holiday calendar (optional).
    holiday_calendar: Option<HolidayCalendar>,
    /// Period-end dynamics for decay curves (optional).
    period_end_dynamics: Option<PeriodEndDynamics>,
    /// Whether to use period-end dynamics instead of legacy flat multipliers.
    use_period_end_dynamics: bool,
    /// Intra-day patterns for time-of-day activity variation.
    intra_day_patterns: Option<IntraDayPatterns>,
}

impl TemporalSampler {
    /// Create a new temporal sampler.
    pub fn new(seed: u64) -> Self {
        Self::with_config(
            seed,
            SeasonalityConfig::default(),
            WorkingHoursConfig::default(),
            Vec::new(),
        )
    }

    /// Create a temporal sampler with custom configuration.
    pub fn with_config(
        seed: u64,
        seasonality_config: SeasonalityConfig,
        working_hours_config: WorkingHoursConfig,
        holidays: Vec<NaiveDate>,
    ) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            seasonality_config,
            working_hours_config,
            holidays,
            industry_seasonality: None,
            holiday_calendar: None,
            period_end_dynamics: None,
            use_period_end_dynamics: false,
            intra_day_patterns: None,
        }
    }

    /// Create a temporal sampler with full enhanced configuration.
    #[allow(clippy::too_many_arguments)]
    pub fn with_full_config(
        seed: u64,
        seasonality_config: SeasonalityConfig,
        working_hours_config: WorkingHoursConfig,
        holidays: Vec<NaiveDate>,
        industry_seasonality: Option<IndustrySeasonality>,
        holiday_calendar: Option<HolidayCalendar>,
    ) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            seasonality_config,
            working_hours_config,
            holidays,
            industry_seasonality,
            holiday_calendar,
            period_end_dynamics: None,
            use_period_end_dynamics: false,
            intra_day_patterns: None,
        }
    }

    /// Create a temporal sampler with period-end dynamics.
    #[allow(clippy::too_many_arguments)]
    pub fn with_period_end_dynamics(
        seed: u64,
        seasonality_config: SeasonalityConfig,
        working_hours_config: WorkingHoursConfig,
        holidays: Vec<NaiveDate>,
        industry_seasonality: Option<IndustrySeasonality>,
        holiday_calendar: Option<HolidayCalendar>,
        period_end_dynamics: PeriodEndDynamics,
    ) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            seasonality_config,
            working_hours_config,
            holidays,
            industry_seasonality,
            holiday_calendar,
            period_end_dynamics: Some(period_end_dynamics),
            use_period_end_dynamics: true,
            intra_day_patterns: None,
        }
    }

    /// Sets the intra-day patterns for time-of-day activity variation.
    pub fn set_intra_day_patterns(&mut self, patterns: IntraDayPatterns) {
        self.intra_day_patterns = Some(patterns);
    }

    /// Gets the intra-day multiplier for a given time.
    pub fn get_intra_day_multiplier(&self, time: NaiveTime, is_human: bool) -> f64 {
        self.intra_day_patterns
            .as_ref()
            .map(|p| p.get_multiplier(time, is_human))
            .unwrap_or(1.0)
    }

    /// Set industry-specific seasonality.
    pub fn with_industry_seasonality(mut self, seasonality: IndustrySeasonality) -> Self {
        self.industry_seasonality = Some(seasonality);
        self
    }

    /// Set regional holiday calendar.
    pub fn with_holiday_calendar(mut self, calendar: HolidayCalendar) -> Self {
        self.holiday_calendar = Some(calendar);
        self
    }

    /// Set industry seasonality (mutable reference version).
    pub fn set_industry_seasonality(&mut self, seasonality: IndustrySeasonality) {
        self.industry_seasonality = Some(seasonality);
    }

    /// Set holiday calendar (mutable reference version).
    pub fn set_holiday_calendar(&mut self, calendar: HolidayCalendar) {
        self.holiday_calendar = Some(calendar);
    }

    /// Set period-end dynamics.
    pub fn with_period_end(mut self, dynamics: PeriodEndDynamics) -> Self {
        self.period_end_dynamics = Some(dynamics);
        self.use_period_end_dynamics = true;
        self
    }

    /// Set period-end dynamics (mutable reference version).
    pub fn set_period_end_dynamics(&mut self, dynamics: PeriodEndDynamics) {
        self.period_end_dynamics = Some(dynamics);
        self.use_period_end_dynamics = true;
    }

    /// Get the period-end dynamics if set.
    pub fn period_end_dynamics(&self) -> Option<&PeriodEndDynamics> {
        self.period_end_dynamics.as_ref()
    }

    /// Enable or disable period-end dynamics usage.
    pub fn set_use_period_end_dynamics(&mut self, enabled: bool) {
        self.use_period_end_dynamics = enabled;
    }

    /// Get the industry seasonality if set.
    pub fn industry_seasonality(&self) -> Option<&IndustrySeasonality> {
        self.industry_seasonality.as_ref()
    }

    /// Get the holiday calendar if set.
    pub fn holiday_calendar(&self) -> Option<&HolidayCalendar> {
        self.holiday_calendar.as_ref()
    }

    /// Generate US federal holidays for a given year.
    pub fn generate_us_holidays(year: i32) -> Vec<NaiveDate> {
        let mut holidays = Vec::new();

        // New Year's Day
        holidays.push(NaiveDate::from_ymd_opt(year, 1, 1).expect("valid date/time components"));
        // Independence Day
        holidays.push(NaiveDate::from_ymd_opt(year, 7, 4).expect("valid date/time components"));
        // Christmas
        holidays.push(NaiveDate::from_ymd_opt(year, 12, 25).expect("valid date/time components"));
        // Thanksgiving (4th Thursday of November)
        let first_thursday = (1..=7)
            .map(|d| NaiveDate::from_ymd_opt(year, 11, d).expect("valid date/time components"))
            .find(|d| d.weekday() == Weekday::Thu)
            .expect("valid date/time components");
        let thanksgiving = first_thursday + Duration::weeks(3);
        holidays.push(thanksgiving);

        holidays
    }

    /// Check if a date is a weekend.
    pub fn is_weekend(&self, date: NaiveDate) -> bool {
        matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
    }

    /// Get the day-of-week activity multiplier.
    ///
    /// Returns a multiplier based on the day of the week:
    /// - Monday: Higher activity (catch-up from weekend)
    /// - Tuesday: Slightly elevated
    /// - Wednesday/Thursday: Normal
    /// - Friday: Reduced (early departures, winding down)
    /// - Saturday/Sunday: Uses weekend_activity setting
    pub fn get_day_of_week_multiplier(&self, date: NaiveDate) -> f64 {
        if !self.seasonality_config.day_of_week_patterns {
            return 1.0;
        }

        match date.weekday() {
            Weekday::Mon => self.seasonality_config.monday_multiplier,
            Weekday::Tue => self.seasonality_config.tuesday_multiplier,
            Weekday::Wed => self.seasonality_config.wednesday_multiplier,
            Weekday::Thu => self.seasonality_config.thursday_multiplier,
            Weekday::Fri => self.seasonality_config.friday_multiplier,
            Weekday::Sat | Weekday::Sun => 1.0, // Weekend activity handled separately
        }
    }

    /// Check if a date is a holiday.
    pub fn is_holiday(&self, date: NaiveDate) -> bool {
        // Check legacy holidays list
        if self.holidays.contains(&date) {
            return true;
        }

        // Check holiday calendar if available
        if let Some(ref calendar) = self.holiday_calendar {
            if calendar.is_holiday(date) {
                return true;
            }
        }

        false
    }

    /// Get the holiday activity multiplier for a date.
    fn get_holiday_multiplier(&self, date: NaiveDate) -> f64 {
        // Check holiday calendar first (more accurate)
        if let Some(ref calendar) = self.holiday_calendar {
            let mult = calendar.get_multiplier(date);
            if mult < 1.0 {
                return mult;
            }
        }

        // Fall back to legacy holidays with default multiplier
        if self.holidays.contains(&date) {
            return self.seasonality_config.holiday_activity;
        }

        1.0
    }

    /// Check if a date is month-end (last N days of month).
    pub fn is_month_end(&self, date: NaiveDate) -> bool {
        let last_day = Self::last_day_of_month(date);
        let days_until_end = (last_day - date).num_days();
        days_until_end >= 0 && days_until_end < self.seasonality_config.month_end_lead_days as i64
    }

    /// Check if a date is quarter-end.
    pub fn is_quarter_end(&self, date: NaiveDate) -> bool {
        let month = date.month();
        let is_quarter_end_month = matches!(month, 3 | 6 | 9 | 12);
        is_quarter_end_month && self.is_month_end(date)
    }

    /// Check if a date is year-end.
    pub fn is_year_end(&self, date: NaiveDate) -> bool {
        date.month() == 12 && self.is_month_end(date)
    }

    /// Get the last day of the month for a given date.
    pub fn last_day_of_month(date: NaiveDate) -> NaiveDate {
        let year = date.year();
        let month = date.month();

        if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("valid date/time components")
                - Duration::days(1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).expect("valid date/time components")
                - Duration::days(1)
        }
    }

    /// Get the activity multiplier for a specific date.
    ///
    /// Combines:
    /// - Base seasonality (month-end, quarter-end, year-end spikes)
    /// - Day-of-week patterns (Monday catch-up, Friday slowdown)
    /// - Weekend activity reduction
    /// - Holiday activity reduction (from calendar or legacy list)
    /// - Industry-specific seasonality (if configured)
    /// - Period-end dynamics (if configured, replaces legacy flat multipliers)
    pub fn get_date_multiplier(&self, date: NaiveDate) -> f64 {
        let mut multiplier = 1.0;

        // Weekend reduction
        if self.is_weekend(date) {
            multiplier *= self.seasonality_config.weekend_activity;
        } else {
            // Day-of-week patterns (only for weekdays)
            multiplier *= self.get_day_of_week_multiplier(date);
        }

        // Holiday reduction (using enhanced calendar if available)
        let holiday_mult = self.get_holiday_multiplier(date);
        if holiday_mult < 1.0 {
            multiplier *= holiday_mult;
        }

        // Period-end spikes - use dynamics if available, otherwise legacy flat multipliers
        if self.use_period_end_dynamics {
            if let Some(ref dynamics) = self.period_end_dynamics {
                let period_mult = dynamics.get_multiplier_for_date(date);
                multiplier *= period_mult;
            }
        } else {
            // Legacy flat multipliers (take the highest applicable)
            if self.seasonality_config.year_end_spike && self.is_year_end(date) {
                multiplier *= self.seasonality_config.year_end_multiplier;
            } else if self.seasonality_config.quarter_end_spike && self.is_quarter_end(date) {
                multiplier *= self.seasonality_config.quarter_end_multiplier;
            } else if self.seasonality_config.month_end_spike && self.is_month_end(date) {
                multiplier *= self.seasonality_config.month_end_multiplier;
            }
        }

        // Industry-specific seasonality
        if let Some(ref industry) = self.industry_seasonality {
            let industry_mult = industry.get_multiplier(date);
            // Industry multipliers are additive to base (they represent deviations from normal)
            // A multiplier > 1.0 increases activity, < 1.0 decreases it
            multiplier *= industry_mult;
        }

        multiplier
    }

    /// Get the period-end multiplier for a date.
    ///
    /// Returns the period-end component of the date multiplier,
    /// using dynamics if available, otherwise legacy flat multipliers.
    pub fn get_period_end_multiplier(&self, date: NaiveDate) -> f64 {
        if self.use_period_end_dynamics {
            if let Some(ref dynamics) = self.period_end_dynamics {
                return dynamics.get_multiplier_for_date(date);
            }
        }

        // Legacy flat multipliers
        if self.seasonality_config.year_end_spike && self.is_year_end(date) {
            self.seasonality_config.year_end_multiplier
        } else if self.seasonality_config.quarter_end_spike && self.is_quarter_end(date) {
            self.seasonality_config.quarter_end_multiplier
        } else if self.seasonality_config.month_end_spike && self.is_month_end(date) {
            self.seasonality_config.month_end_multiplier
        } else {
            1.0
        }
    }

    /// Get the base multiplier without industry seasonality.
    pub fn get_base_date_multiplier(&self, date: NaiveDate) -> f64 {
        let mut multiplier = 1.0;

        if self.is_weekend(date) {
            multiplier *= self.seasonality_config.weekend_activity;
        } else {
            // Day-of-week patterns (only for weekdays)
            multiplier *= self.get_day_of_week_multiplier(date);
        }

        let holiday_mult = self.get_holiday_multiplier(date);
        if holiday_mult < 1.0 {
            multiplier *= holiday_mult;
        }

        // Period-end spikes - use dynamics if available
        if self.use_period_end_dynamics {
            if let Some(ref dynamics) = self.period_end_dynamics {
                let period_mult = dynamics.get_multiplier_for_date(date);
                multiplier *= period_mult;
            }
        } else {
            // Legacy flat multipliers
            if self.seasonality_config.year_end_spike && self.is_year_end(date) {
                multiplier *= self.seasonality_config.year_end_multiplier;
            } else if self.seasonality_config.quarter_end_spike && self.is_quarter_end(date) {
                multiplier *= self.seasonality_config.quarter_end_multiplier;
            } else if self.seasonality_config.month_end_spike && self.is_month_end(date) {
                multiplier *= self.seasonality_config.month_end_multiplier;
            }
        }

        multiplier
    }

    /// Get only the industry seasonality multiplier for a date.
    pub fn get_industry_multiplier(&self, date: NaiveDate) -> f64 {
        self.industry_seasonality
            .as_ref()
            .map(|s| s.get_multiplier(date))
            .unwrap_or(1.0)
    }

    /// Sample a posting date within a range based on seasonality.
    pub fn sample_date(&mut self, start: NaiveDate, end: NaiveDate) -> NaiveDate {
        let days = (end - start).num_days() as usize;
        if days == 0 {
            return start;
        }

        // Build weighted distribution based on activity levels
        let mut weights: Vec<f64> = (0..=days)
            .map(|d| {
                let date = start + Duration::days(d as i64);
                self.get_date_multiplier(date)
            })
            .collect();

        // Normalize weights
        let total: f64 = weights.iter().sum();
        weights.iter_mut().for_each(|w| *w /= total);

        // Sample using weights
        let p: f64 = self.rng.gen();
        let mut cumulative = 0.0;
        for (i, weight) in weights.iter().enumerate() {
            cumulative += weight;
            if p < cumulative {
                return start + Duration::days(i as i64);
            }
        }

        end
    }

    /// Sample a posting time based on working hours.
    pub fn sample_time(&mut self, is_human: bool) -> NaiveTime {
        if !is_human {
            // Automated systems can post any time, but prefer off-hours
            let hour = if self.rng.gen::<f64>() < 0.7 {
                // 70% off-peak hours (night batch processing)
                self.rng.gen_range(22..=23).clamp(0, 23)
                    + if self.rng.gen_bool(0.5) {
                        0
                    } else {
                        self.rng.gen_range(0..=5)
                    }
            } else {
                self.rng.gen_range(0..24)
            };
            let minute = self.rng.gen_range(0..60);
            let second = self.rng.gen_range(0..60);
            return NaiveTime::from_hms_opt(hour.clamp(0, 23) as u32, minute, second)
                .expect("valid date/time components");
        }

        // Human users follow working hours
        let hour = if self.rng.gen::<f64>() < self.working_hours_config.after_hours_probability {
            // After hours
            if self.rng.gen_bool(0.5) {
                self.rng.gen_range(6..self.working_hours_config.day_start)
            } else {
                self.rng.gen_range(self.working_hours_config.day_end..22)
            }
        } else {
            // Normal working hours with peak weighting
            let is_peak = self.rng.gen::<f64>() < 0.6; // 60% during peak
            if is_peak && !self.working_hours_config.peak_hours.is_empty() {
                *self
                    .working_hours_config
                    .peak_hours
                    .choose(&mut self.rng)
                    .expect("valid date/time components")
            } else {
                self.rng.gen_range(
                    self.working_hours_config.day_start..self.working_hours_config.day_end,
                )
            }
        };

        let minute = self.rng.gen_range(0..60);
        let second = self.rng.gen_range(0..60);

        NaiveTime::from_hms_opt(hour as u32, minute, second).expect("valid date/time components")
    }

    /// Calculate expected transaction count for a date given daily average.
    pub fn expected_count_for_date(&self, date: NaiveDate, daily_average: f64) -> u64 {
        let multiplier = self.get_date_multiplier(date);
        (daily_average * multiplier).round() as u64
    }

    /// Reset the sampler with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
    }
}

/// Time period specification for generation.
#[derive(Debug, Clone)]
pub struct TimePeriod {
    /// Start date (inclusive)
    pub start_date: NaiveDate,
    /// End date (inclusive)
    pub end_date: NaiveDate,
    /// Fiscal year
    pub fiscal_year: u16,
    /// Fiscal periods covered
    pub fiscal_periods: Vec<u8>,
}

impl TimePeriod {
    /// Create a time period for a full fiscal year.
    pub fn fiscal_year(year: u16) -> Self {
        Self {
            start_date: NaiveDate::from_ymd_opt(year as i32, 1, 1)
                .expect("valid date/time components"),
            end_date: NaiveDate::from_ymd_opt(year as i32, 12, 31)
                .expect("valid date/time components"),
            fiscal_year: year,
            fiscal_periods: (1..=12).collect(),
        }
    }

    /// Create a time period for specific months.
    pub fn months(year: u16, start_month: u8, num_months: u8) -> Self {
        let start_date = NaiveDate::from_ymd_opt(year as i32, start_month as u32, 1)
            .expect("valid date/time components");
        let end_month = ((start_month - 1 + num_months - 1) % 12) + 1;
        let end_year = year + (start_month as u16 - 1 + num_months as u16 - 1) / 12;
        let end_date = TemporalSampler::last_day_of_month(
            NaiveDate::from_ymd_opt(end_year as i32, end_month as u32, 1)
                .expect("valid date/time components"),
        );

        Self {
            start_date,
            end_date,
            fiscal_year: year,
            fiscal_periods: (start_month..start_month + num_months).collect(),
        }
    }

    /// Get total days in the period.
    pub fn total_days(&self) -> i64 {
        (self.end_date - self.start_date).num_days() + 1
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_is_weekend() {
        let sampler = TemporalSampler::new(42);
        let saturday = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let sunday = NaiveDate::from_ymd_opt(2024, 6, 16).unwrap();
        let monday = NaiveDate::from_ymd_opt(2024, 6, 17).unwrap();

        assert!(sampler.is_weekend(saturday));
        assert!(sampler.is_weekend(sunday));
        assert!(!sampler.is_weekend(monday));
    }

    #[test]
    fn test_is_month_end() {
        let sampler = TemporalSampler::new(42);
        let month_end = NaiveDate::from_ymd_opt(2024, 6, 28).unwrap();
        let month_start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();

        assert!(sampler.is_month_end(month_end));
        assert!(!sampler.is_month_end(month_start));
    }

    #[test]
    fn test_date_multiplier() {
        let sampler = TemporalSampler::new(42);

        // Regular weekday (Wednesday = 1.0)
        let regular_day = NaiveDate::from_ymd_opt(2024, 6, 12).unwrap(); // Wednesday
        assert!((sampler.get_date_multiplier(regular_day) - 1.0).abs() < 0.01);

        // Weekend
        let weekend = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(); // Saturday
        assert!(sampler.get_date_multiplier(weekend) < 0.2);

        // Month end
        let month_end = NaiveDate::from_ymd_opt(2024, 6, 28).unwrap();
        assert!(sampler.get_date_multiplier(month_end) > 2.0);
    }

    #[test]
    fn test_day_of_week_patterns() {
        let sampler = TemporalSampler::new(42);

        // June 2024: 10=Mon, 11=Tue, 12=Wed, 13=Thu, 14=Fri
        let monday = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        let tuesday = NaiveDate::from_ymd_opt(2024, 6, 11).unwrap();
        let wednesday = NaiveDate::from_ymd_opt(2024, 6, 12).unwrap();
        let thursday = NaiveDate::from_ymd_opt(2024, 6, 13).unwrap();
        let friday = NaiveDate::from_ymd_opt(2024, 6, 14).unwrap();

        // Monday should have highest weekday multiplier (catch-up)
        let mon_mult = sampler.get_day_of_week_multiplier(monday);
        assert!((mon_mult - 1.3).abs() < 0.01);

        // Tuesday slightly elevated
        let tue_mult = sampler.get_day_of_week_multiplier(tuesday);
        assert!((tue_mult - 1.1).abs() < 0.01);

        // Wednesday/Thursday normal
        let wed_mult = sampler.get_day_of_week_multiplier(wednesday);
        let thu_mult = sampler.get_day_of_week_multiplier(thursday);
        assert!((wed_mult - 1.0).abs() < 0.01);
        assert!((thu_mult - 1.0).abs() < 0.01);

        // Friday reduced (winding down)
        let fri_mult = sampler.get_day_of_week_multiplier(friday);
        assert!((fri_mult - 0.85).abs() < 0.01);

        // Verify the pattern is applied in get_date_multiplier
        // (excluding period-end effects)
        assert!(sampler.get_date_multiplier(monday) > sampler.get_date_multiplier(friday));
    }

    #[test]
    fn test_sample_time_human() {
        let mut sampler = TemporalSampler::new(42);

        for _ in 0..100 {
            let time = sampler.sample_time(true);
            // Most times should be during working hours
            let hour = time.hour();
            // Just verify it's a valid time
            assert!(hour < 24);
        }
    }

    #[test]
    fn test_time_period() {
        let period = TimePeriod::fiscal_year(2024);
        assert_eq!(period.total_days(), 366); // 2024 is leap year

        let partial = TimePeriod::months(2024, 1, 6);
        assert!(partial.total_days() > 180);
        assert!(partial.total_days() < 185);
    }
}
