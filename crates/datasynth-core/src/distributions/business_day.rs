//! Business day calculations for settlement dates and working day logic.
//!
//! Provides financial settlement conventions (T+N), business day arithmetic,
//! and half-day policy handling for enterprise accounting systems.

use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::holidays::HolidayCalendar;

/// Policy for handling half-day trading sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HalfDayPolicy {
    /// Treat half-days as full business days
    #[default]
    FullDay,
    /// Treat half-days as half business days (for counting purposes)
    HalfDay,
    /// Treat half-days as non-business days
    NonBusinessDay,
}

/// Convention for handling month-end settlement dates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MonthEndConvention {
    /// Move to the following business day; if that's in the next month, move to preceding
    #[default]
    ModifiedFollowing,
    /// Move to the preceding business day
    Preceding,
    /// Move to the following business day
    Following,
    /// Always use the last business day of the month
    EndOfMonth,
}

/// Settlement type specifying how many business days after trade date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettlementType {
    /// T+N settlement (N business days after trade date)
    TPlus(i32),
    /// Same-day settlement
    SameDay,
    /// Next business day settlement
    NextBusinessDay,
    /// End of month settlement
    MonthEnd,
}

impl Default for SettlementType {
    fn default() -> Self {
        Self::TPlus(2)
    }
}

/// Configuration for wire transfer settlement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireSettlementConfig {
    /// Cutoff time for same-day processing (e.g., "14:00")
    pub cutoff_time: NaiveTime,
    /// Settlement type when before cutoff
    pub before_cutoff: SettlementType,
    /// Settlement type when after cutoff
    pub after_cutoff: SettlementType,
}

impl Default for WireSettlementConfig {
    fn default() -> Self {
        Self {
            cutoff_time: NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
            before_cutoff: SettlementType::SameDay,
            after_cutoff: SettlementType::NextBusinessDay,
        }
    }
}

/// Standard settlement rules for different instrument types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementRules {
    /// Equity settlement (typically T+2)
    pub equity: SettlementType,
    /// Government bonds settlement (typically T+1)
    pub government_bonds: SettlementType,
    /// FX spot settlement (typically T+2)
    pub fx_spot: SettlementType,
    /// FX forward settlement (depends on tenor)
    pub fx_forward: SettlementType,
    /// Corporate bonds settlement (typically T+2)
    pub corporate_bonds: SettlementType,
    /// Wire transfer (domestic)
    pub wire_domestic: WireSettlementConfig,
    /// Wire transfer (international)
    pub wire_international: SettlementType,
    /// ACH transfers
    pub ach: SettlementType,
}

impl Default for SettlementRules {
    fn default() -> Self {
        Self {
            equity: SettlementType::TPlus(2),
            government_bonds: SettlementType::TPlus(1),
            fx_spot: SettlementType::TPlus(2),
            fx_forward: SettlementType::TPlus(2),
            corporate_bonds: SettlementType::TPlus(2),
            wire_domestic: WireSettlementConfig::default(),
            wire_international: SettlementType::TPlus(1),
            ach: SettlementType::TPlus(1),
        }
    }
}

/// Calculator for business day operations.
///
/// Handles business day arithmetic, settlement date calculation,
/// and respects regional holiday calendars.
#[derive(Debug, Clone)]
pub struct BusinessDayCalculator {
    /// Holiday calendar for the region
    calendar: HolidayCalendar,
    /// Days considered weekend (typically Sat/Sun, but Fri/Sat in some regions)
    weekend_days: HashSet<Weekday>,
    /// Policy for half-day sessions
    half_day_policy: HalfDayPolicy,
    /// Map of half-day dates (early close days)
    half_days: HashMap<NaiveDate, NaiveTime>,
    /// Settlement rules
    settlement_rules: SettlementRules,
    /// Month-end convention
    month_end_convention: MonthEndConvention,
}

impl BusinessDayCalculator {
    /// Create a new business day calculator with the given holiday calendar.
    pub fn new(calendar: HolidayCalendar) -> Self {
        let mut weekend_days = HashSet::new();
        weekend_days.insert(Weekday::Sat);
        weekend_days.insert(Weekday::Sun);

        Self {
            calendar,
            weekend_days,
            half_day_policy: HalfDayPolicy::default(),
            half_days: HashMap::new(),
            settlement_rules: SettlementRules::default(),
            month_end_convention: MonthEndConvention::default(),
        }
    }

    /// Create a calculator with custom weekend days (e.g., Fri/Sat for Middle East).
    pub fn with_weekend_days(mut self, weekend_days: HashSet<Weekday>) -> Self {
        self.weekend_days = weekend_days;
        self
    }

    /// Set the half-day policy.
    pub fn with_half_day_policy(mut self, policy: HalfDayPolicy) -> Self {
        self.half_day_policy = policy;
        self
    }

    /// Add a half-day (early close) date.
    pub fn add_half_day(&mut self, date: NaiveDate, close_time: NaiveTime) {
        self.half_days.insert(date, close_time);
    }

    /// Set the settlement rules.
    pub fn with_settlement_rules(mut self, rules: SettlementRules) -> Self {
        self.settlement_rules = rules;
        self
    }

    /// Set the month-end convention.
    pub fn with_month_end_convention(mut self, convention: MonthEndConvention) -> Self {
        self.month_end_convention = convention;
        self
    }

    /// Check if a date is a weekend day.
    pub fn is_weekend(&self, date: NaiveDate) -> bool {
        self.weekend_days.contains(&date.weekday())
    }

    /// Check if a date is a holiday.
    pub fn is_holiday(&self, date: NaiveDate) -> bool {
        self.calendar.is_holiday(date)
    }

    /// Check if a date is a half-day (early close).
    pub fn is_half_day(&self, date: NaiveDate) -> bool {
        self.half_days.contains_key(&date)
    }

    /// Get the early close time for a half-day, if applicable.
    pub fn get_half_day_close(&self, date: NaiveDate) -> Option<NaiveTime> {
        self.half_days.get(&date).copied()
    }

    /// Check if a date is a business day.
    ///
    /// A date is a business day if:
    /// - It is not a weekend day
    /// - It is not a bank holiday
    /// - It is not a half-day treated as non-business (per policy)
    pub fn is_business_day(&self, date: NaiveDate) -> bool {
        // Check weekend
        if self.is_weekend(date) {
            return false;
        }

        // Check holidays - bank holidays are not business days
        // Also treat very low multipliers (< 0.1) as non-business days
        if self.calendar.is_holiday(date) {
            let holidays = self.calendar.get_holidays(date);
            // If any holiday on this date is a bank holiday, it's not a business day
            if holidays.iter().any(|h| h.is_bank_holiday) {
                return false;
            }
            // Also check if multiplier is very low (effectively closed)
            let mult = self.calendar.get_multiplier(date);
            if mult < 0.1 {
                return false;
            }
        }

        // Check half-day policy
        if self.is_half_day(date) && self.half_day_policy == HalfDayPolicy::NonBusinessDay {
            return false;
        }

        true
    }

    /// Add N business days to a date.
    ///
    /// If N is positive, moves forward; if negative, moves backward.
    pub fn add_business_days(&self, date: NaiveDate, days: i32) -> NaiveDate {
        if days == 0 {
            return date;
        }

        let direction = if days > 0 { 1 } else { -1 };
        let mut remaining = days.abs();
        let mut current = date;

        while remaining > 0 {
            current += Duration::days(direction as i64);
            if self.is_business_day(current) {
                remaining -= 1;
            }
        }

        current
    }

    /// Subtract N business days from a date.
    pub fn sub_business_days(&self, date: NaiveDate, days: i32) -> NaiveDate {
        self.add_business_days(date, -days)
    }

    /// Get the next business day on or after the given date.
    ///
    /// If `inclusive` is true and the date is a business day, returns the date itself.
    /// Otherwise, returns the next business day.
    pub fn next_business_day(&self, date: NaiveDate, inclusive: bool) -> NaiveDate {
        let mut current = date;

        if inclusive && self.is_business_day(current) {
            return current;
        }

        loop {
            current += Duration::days(1);
            if self.is_business_day(current) {
                return current;
            }
        }
    }

    /// Get the previous business day on or before the given date.
    ///
    /// If `inclusive` is true and the date is a business day, returns the date itself.
    /// Otherwise, returns the previous business day.
    pub fn prev_business_day(&self, date: NaiveDate, inclusive: bool) -> NaiveDate {
        let mut current = date;

        if inclusive && self.is_business_day(current) {
            return current;
        }

        loop {
            current -= Duration::days(1);
            if self.is_business_day(current) {
                return current;
            }
        }
    }

    /// Count business days between two dates (exclusive of end date).
    ///
    /// Returns a positive count if end > start, negative if end < start.
    pub fn business_days_between(&self, start: NaiveDate, end: NaiveDate) -> i32 {
        if start == end {
            return 0;
        }

        let (earlier, later, sign) = if start < end {
            (start, end, 1)
        } else {
            (end, start, -1)
        };

        let mut count = 0;
        let mut current = earlier + Duration::days(1);

        while current < later {
            if self.is_business_day(current) {
                count += 1;
            }
            current += Duration::days(1);
        }

        count * sign
    }

    /// Calculate the settlement date for a trade.
    ///
    /// The trade date itself is day 0 (T+0). Settlement occurs on T+N
    /// where N is determined by the settlement type.
    pub fn settlement_date(&self, trade_date: NaiveDate, settlement: SettlementType) -> NaiveDate {
        match settlement {
            SettlementType::TPlus(days) => {
                // Start from trade date and add N business days
                self.add_business_days(trade_date, days)
            }
            SettlementType::SameDay => {
                // Same day if it's a business day, otherwise next
                self.next_business_day(trade_date, true)
            }
            SettlementType::NextBusinessDay => {
                // Next business day after trade date
                self.next_business_day(trade_date, false)
            }
            SettlementType::MonthEnd => {
                // Last business day of the month
                self.last_business_day_of_month(trade_date)
            }
        }
    }

    /// Calculate settlement date for a wire transfer.
    pub fn wire_settlement_date(
        &self,
        trade_date: NaiveDate,
        trade_time: NaiveTime,
        config: &WireSettlementConfig,
    ) -> NaiveDate {
        let settlement_type = if trade_time <= config.cutoff_time {
            config.before_cutoff
        } else {
            config.after_cutoff
        };

        self.settlement_date(trade_date, settlement_type)
    }

    /// Get the last business day of the month containing the given date.
    pub fn last_business_day_of_month(&self, date: NaiveDate) -> NaiveDate {
        let last_calendar_day = self.last_day_of_month(date);
        self.prev_business_day(last_calendar_day, true)
    }

    /// Get the first business day of the month containing the given date.
    pub fn first_business_day_of_month(&self, date: NaiveDate) -> NaiveDate {
        let first = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap();
        self.next_business_day(first, true)
    }

    /// Adjust a date according to the month-end convention.
    ///
    /// Used when a calculated date falls on a non-business day.
    pub fn adjust_for_business_day(&self, date: NaiveDate) -> NaiveDate {
        if self.is_business_day(date) {
            return date;
        }

        match self.month_end_convention {
            MonthEndConvention::Following => self.next_business_day(date, false),
            MonthEndConvention::Preceding => self.prev_business_day(date, false),
            MonthEndConvention::ModifiedFollowing => {
                let following = self.next_business_day(date, false);
                if following.month() != date.month() {
                    self.prev_business_day(date, false)
                } else {
                    following
                }
            }
            MonthEndConvention::EndOfMonth => self.last_business_day_of_month(date),
        }
    }

    /// Get settlement rules.
    pub fn settlement_rules(&self) -> &SettlementRules {
        &self.settlement_rules
    }

    /// Get the last day of the month for a given date.
    fn last_day_of_month(&self, date: NaiveDate) -> NaiveDate {
        let year = date.year();
        let month = date.month();

        if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - Duration::days(1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - Duration::days(1)
        }
    }

    /// Get business days in a month.
    pub fn business_days_in_month(&self, year: i32, month: u32) -> Vec<NaiveDate> {
        let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let last = self.last_day_of_month(first);

        let mut business_days = Vec::new();
        let mut current = first;

        while current <= last {
            if self.is_business_day(current) {
                business_days.push(current);
            }
            current += Duration::days(1);
        }

        business_days
    }

    /// Count business days in a month.
    pub fn count_business_days_in_month(&self, year: i32, month: u32) -> usize {
        self.business_days_in_month(year, month).len()
    }
}

/// Builder for creating a BusinessDayCalculator with common configurations.
pub struct BusinessDayCalculatorBuilder {
    calendar: HolidayCalendar,
    weekend_days: Option<HashSet<Weekday>>,
    half_day_policy: HalfDayPolicy,
    half_days: HashMap<NaiveDate, NaiveTime>,
    settlement_rules: SettlementRules,
    month_end_convention: MonthEndConvention,
}

impl BusinessDayCalculatorBuilder {
    /// Create a new builder with a holiday calendar.
    pub fn new(calendar: HolidayCalendar) -> Self {
        Self {
            calendar,
            weekend_days: None,
            half_day_policy: HalfDayPolicy::default(),
            half_days: HashMap::new(),
            settlement_rules: SettlementRules::default(),
            month_end_convention: MonthEndConvention::default(),
        }
    }

    /// Set weekend days (default is Saturday and Sunday).
    pub fn weekend_days(mut self, days: HashSet<Weekday>) -> Self {
        self.weekend_days = Some(days);
        self
    }

    /// Set to Middle East weekend (Friday and Saturday).
    pub fn middle_east_weekend(mut self) -> Self {
        let mut days = HashSet::new();
        days.insert(Weekday::Fri);
        days.insert(Weekday::Sat);
        self.weekend_days = Some(days);
        self
    }

    /// Set half-day policy.
    pub fn half_day_policy(mut self, policy: HalfDayPolicy) -> Self {
        self.half_day_policy = policy;
        self
    }

    /// Add a half-day.
    pub fn add_half_day(mut self, date: NaiveDate, close_time: NaiveTime) -> Self {
        self.half_days.insert(date, close_time);
        self
    }

    /// Add US stock market half-days for a year.
    ///
    /// Typically: day before Independence Day, Black Friday, Christmas Eve
    pub fn add_us_market_half_days(mut self, year: i32) -> Self {
        let close_time = NaiveTime::from_hms_opt(13, 0, 0).unwrap();

        // Day before Independence Day (if July 3 is a weekday)
        let july_3 = NaiveDate::from_ymd_opt(year, 7, 3).unwrap();
        if !matches!(july_3.weekday(), Weekday::Sat | Weekday::Sun) {
            self.half_days.insert(july_3, close_time);
        }

        // Black Friday (day after Thanksgiving - 4th Thursday of November)
        let first_nov = NaiveDate::from_ymd_opt(year, 11, 1).unwrap();
        let days_until_thu = (Weekday::Thu.num_days_from_monday() as i32
            - first_nov.weekday().num_days_from_monday() as i32
            + 7)
            % 7;
        let thanksgiving = first_nov + Duration::days(days_until_thu as i64 + 21); // 4th Thursday
        let black_friday = thanksgiving + Duration::days(1);
        self.half_days.insert(black_friday, close_time);

        // Christmas Eve (if December 24 is a weekday)
        let christmas_eve = NaiveDate::from_ymd_opt(year, 12, 24).unwrap();
        if !matches!(christmas_eve.weekday(), Weekday::Sat | Weekday::Sun) {
            self.half_days.insert(christmas_eve, close_time);
        }

        self
    }

    /// Set settlement rules.
    pub fn settlement_rules(mut self, rules: SettlementRules) -> Self {
        self.settlement_rules = rules;
        self
    }

    /// Set month-end convention.
    pub fn month_end_convention(mut self, convention: MonthEndConvention) -> Self {
        self.month_end_convention = convention;
        self
    }

    /// Build the BusinessDayCalculator.
    pub fn build(self) -> BusinessDayCalculator {
        let mut calc = BusinessDayCalculator::new(self.calendar);

        if let Some(weekend_days) = self.weekend_days {
            calc.weekend_days = weekend_days;
        }

        calc.half_day_policy = self.half_day_policy;
        calc.half_days = self.half_days;
        calc.settlement_rules = self.settlement_rules;
        calc.month_end_convention = self.month_end_convention;

        calc
    }
}

/// Configuration for business day settings (for YAML/JSON deserialization).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessDayConfig {
    /// Enable business day calculations
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Half-day policy
    #[serde(default)]
    pub half_day_policy: HalfDayPolicy,
    /// Settlement rules
    #[serde(default)]
    pub settlement_rules: SettlementRulesConfig,
    /// Month-end convention
    #[serde(default)]
    pub month_end_convention: MonthEndConvention,
    /// Weekend days (list of day names like "saturday", "sunday")
    #[serde(default)]
    pub weekend_days: Option<Vec<String>>,
}

fn default_true() -> bool {
    true
}

impl Default for BusinessDayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            half_day_policy: HalfDayPolicy::default(),
            settlement_rules: SettlementRulesConfig::default(),
            month_end_convention: MonthEndConvention::default(),
            weekend_days: None,
        }
    }
}

/// Settlement rules configuration for YAML/JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementRulesConfig {
    /// Equity settlement days (T+N)
    #[serde(default = "default_settlement_2")]
    pub equity_days: i32,
    /// Government bonds settlement days
    #[serde(default = "default_settlement_1")]
    pub government_bonds_days: i32,
    /// FX spot settlement days
    #[serde(default = "default_settlement_2")]
    pub fx_spot_days: i32,
    /// Corporate bonds settlement days
    #[serde(default = "default_settlement_2")]
    pub corporate_bonds_days: i32,
    /// Wire transfer cutoff time (HH:MM format)
    #[serde(default = "default_wire_cutoff")]
    pub wire_cutoff_time: String,
    /// International wire settlement days
    #[serde(default = "default_settlement_1")]
    pub wire_international_days: i32,
    /// ACH settlement days
    #[serde(default = "default_settlement_1")]
    pub ach_days: i32,
}

fn default_settlement_1() -> i32 {
    1
}

fn default_settlement_2() -> i32 {
    2
}

fn default_wire_cutoff() -> String {
    "14:00".to_string()
}

impl Default for SettlementRulesConfig {
    fn default() -> Self {
        Self {
            equity_days: 2,
            government_bonds_days: 1,
            fx_spot_days: 2,
            corporate_bonds_days: 2,
            wire_cutoff_time: "14:00".to_string(),
            wire_international_days: 1,
            ach_days: 1,
        }
    }
}

impl SettlementRulesConfig {
    /// Convert to SettlementRules.
    pub fn to_settlement_rules(&self) -> SettlementRules {
        let cutoff_time = NaiveTime::parse_from_str(&self.wire_cutoff_time, "%H:%M")
            .unwrap_or_else(|_| NaiveTime::from_hms_opt(14, 0, 0).unwrap());

        SettlementRules {
            equity: SettlementType::TPlus(self.equity_days),
            government_bonds: SettlementType::TPlus(self.government_bonds_days),
            fx_spot: SettlementType::TPlus(self.fx_spot_days),
            fx_forward: SettlementType::TPlus(self.fx_spot_days),
            corporate_bonds: SettlementType::TPlus(self.corporate_bonds_days),
            wire_domestic: WireSettlementConfig {
                cutoff_time,
                before_cutoff: SettlementType::SameDay,
                after_cutoff: SettlementType::NextBusinessDay,
            },
            wire_international: SettlementType::TPlus(self.wire_international_days),
            ach: SettlementType::TPlus(self.ach_days),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distributions::holidays::Region;

    fn test_calendar() -> HolidayCalendar {
        HolidayCalendar::for_region(Region::US, 2024)
    }

    #[test]
    fn test_is_business_day() {
        let calc = BusinessDayCalculator::new(test_calendar());

        // Regular weekday (Wednesday, no holiday)
        let wednesday = NaiveDate::from_ymd_opt(2024, 6, 12).unwrap();
        assert!(calc.is_business_day(wednesday));

        // Weekend (Saturday)
        let saturday = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert!(!calc.is_business_day(saturday));

        // Holiday (Christmas)
        let christmas = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
        assert!(!calc.is_business_day(christmas));
    }

    #[test]
    fn test_add_business_days() {
        let calc = BusinessDayCalculator::new(test_calendar());

        // Friday + 1 business day = Monday
        let friday = NaiveDate::from_ymd_opt(2024, 6, 14).unwrap();
        let next = calc.add_business_days(friday, 1);
        assert_eq!(next.weekday(), Weekday::Mon);
        assert_eq!(next, NaiveDate::from_ymd_opt(2024, 6, 17).unwrap());

        // Friday June 14 + 5 business days = Monday June 24
        // (Mon 17, Tue 18, skip Juneteenth Jun 19, Thu 20, Fri 21, Mon 24)
        let next_week = calc.add_business_days(friday, 5);
        assert_eq!(next_week.weekday(), Weekday::Mon);
        assert_eq!(next_week, NaiveDate::from_ymd_opt(2024, 6, 24).unwrap());
    }

    #[test]
    fn test_sub_business_days() {
        let calc = BusinessDayCalculator::new(test_calendar());

        // Monday - 1 business day = Friday
        let monday = NaiveDate::from_ymd_opt(2024, 6, 17).unwrap();
        let prev = calc.sub_business_days(monday, 1);
        assert_eq!(prev.weekday(), Weekday::Fri);
        assert_eq!(prev, NaiveDate::from_ymd_opt(2024, 6, 14).unwrap());
    }

    #[test]
    fn test_business_days_between() {
        let calc = BusinessDayCalculator::new(test_calendar());

        // Monday to Friday (same week) = 3 business days between (Tue, Wed, Thu)
        // Note: exclusive of both start and end dates
        let monday = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        let friday = NaiveDate::from_ymd_opt(2024, 6, 14).unwrap();
        assert_eq!(calc.business_days_between(monday, friday), 3);

        // Same day = 0
        assert_eq!(calc.business_days_between(monday, monday), 0);

        // Reverse direction
        assert_eq!(calc.business_days_between(friday, monday), -3);

        // Monday to next Monday (skipping weekend) = 4 business days
        // Tue, Wed, Thu, Fri between Mon-Mon
        let next_monday = NaiveDate::from_ymd_opt(2024, 6, 17).unwrap();
        assert_eq!(calc.business_days_between(monday, next_monday), 4);
    }

    #[test]
    fn test_settlement_t_plus_2() {
        let calc = BusinessDayCalculator::new(test_calendar());

        // Trade on Monday, settle on Wednesday (T+2)
        let monday = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        let settlement = calc.settlement_date(monday, SettlementType::TPlus(2));
        assert_eq!(settlement, NaiveDate::from_ymd_opt(2024, 6, 12).unwrap());

        // Trade on Thursday, settle on Monday (T+2 skips weekend)
        let thursday = NaiveDate::from_ymd_opt(2024, 6, 13).unwrap();
        let settlement = calc.settlement_date(thursday, SettlementType::TPlus(2));
        assert_eq!(settlement, NaiveDate::from_ymd_opt(2024, 6, 17).unwrap());
    }

    #[test]
    fn test_settlement_same_day() {
        let calc = BusinessDayCalculator::new(test_calendar());

        // Business day - same day
        let monday = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        assert_eq!(
            calc.settlement_date(monday, SettlementType::SameDay),
            monday
        );

        // Saturday - moves to Monday
        let saturday = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let settlement = calc.settlement_date(saturday, SettlementType::SameDay);
        assert_eq!(settlement, NaiveDate::from_ymd_opt(2024, 6, 17).unwrap());
    }

    #[test]
    fn test_next_business_day() {
        let calc = BusinessDayCalculator::new(test_calendar());

        // From Saturday, inclusive - moves to Monday
        let saturday = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let next = calc.next_business_day(saturday, true);
        assert_eq!(next, NaiveDate::from_ymd_opt(2024, 6, 17).unwrap());

        // From Monday, inclusive - stays Monday
        let monday = NaiveDate::from_ymd_opt(2024, 6, 17).unwrap();
        assert_eq!(calc.next_business_day(monday, true), monday);

        // From Monday, not inclusive - moves to Tuesday
        assert_eq!(
            calc.next_business_day(monday, false),
            NaiveDate::from_ymd_opt(2024, 6, 18).unwrap()
        );
    }

    #[test]
    fn test_prev_business_day() {
        let calc = BusinessDayCalculator::new(test_calendar());

        // From Sunday, inclusive - moves to Friday
        let sunday = NaiveDate::from_ymd_opt(2024, 6, 16).unwrap();
        let prev = calc.prev_business_day(sunday, true);
        assert_eq!(prev, NaiveDate::from_ymd_opt(2024, 6, 14).unwrap());
    }

    #[test]
    fn test_last_business_day_of_month() {
        let calc = BusinessDayCalculator::new(test_calendar());

        // June 2024 ends on Sunday, so last business day is Friday 28th
        let june = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let last = calc.last_business_day_of_month(june);
        assert_eq!(last, NaiveDate::from_ymd_opt(2024, 6, 28).unwrap());
    }

    #[test]
    fn test_modified_following_convention() {
        let calc = BusinessDayCalculator::new(test_calendar())
            .with_month_end_convention(MonthEndConvention::ModifiedFollowing);

        // June 30, 2024 is a Sunday
        // Following would give July 1, but that's next month
        // So Modified Following gives Friday June 28
        let june_30 = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        let adjusted = calc.adjust_for_business_day(june_30);
        assert_eq!(adjusted, NaiveDate::from_ymd_opt(2024, 6, 28).unwrap());
    }

    #[test]
    fn test_middle_east_weekend() {
        let calc = BusinessDayCalculatorBuilder::new(test_calendar())
            .middle_east_weekend()
            .build();

        // Friday should be weekend in Middle East
        let friday = NaiveDate::from_ymd_opt(2024, 6, 14).unwrap();
        assert!(!calc.is_business_day(friday));

        // Sunday should be a business day
        let sunday = NaiveDate::from_ymd_opt(2024, 6, 16).unwrap();
        assert!(calc.is_business_day(sunday));
    }

    #[test]
    fn test_half_day_policy() {
        // Use a date that's not a holiday in the US calendar - July 3, 2024 (Wednesday)
        let half_day = NaiveDate::from_ymd_opt(2024, 7, 3).unwrap();
        let close_time = NaiveTime::from_hms_opt(13, 0, 0).unwrap();

        // With HalfDay policy - still a business day
        let calc_half = BusinessDayCalculatorBuilder::new(test_calendar())
            .half_day_policy(HalfDayPolicy::HalfDay)
            .add_half_day(half_day, close_time)
            .build();
        assert!(calc_half.is_business_day(half_day));
        assert_eq!(calc_half.get_half_day_close(half_day), Some(close_time));

        // With NonBusinessDay policy - not a business day
        let calc_non = BusinessDayCalculatorBuilder::new(test_calendar())
            .half_day_policy(HalfDayPolicy::NonBusinessDay)
            .add_half_day(half_day, close_time)
            .build();
        assert!(!calc_non.is_business_day(half_day));
    }

    #[test]
    fn test_business_days_in_month() {
        let calc = BusinessDayCalculator::new(test_calendar());

        // June 2024 has 30 days, 8-10 weekend days, and Juneteenth (June 19)
        // So approximately 20 business days
        let days = calc.business_days_in_month(2024, 6);
        assert!(
            days.len() >= 18 && days.len() <= 22,
            "Expected 18-22 business days in June 2024, got {}",
            days.len()
        );

        // All returned days should be business days
        for day in &days {
            assert!(
                calc.is_business_day(*day),
                "{} should be a business day",
                day
            );
        }
    }

    #[test]
    fn test_wire_settlement() {
        let calc = BusinessDayCalculator::new(test_calendar());
        let config = WireSettlementConfig::default();

        let monday = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();

        // Before cutoff - same day
        let morning = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        assert_eq!(calc.wire_settlement_date(monday, morning, &config), monday);

        // After cutoff - next business day
        let evening = NaiveTime::from_hms_opt(16, 0, 0).unwrap();
        let next = calc.wire_settlement_date(monday, evening, &config);
        assert_eq!(next, NaiveDate::from_ymd_opt(2024, 6, 11).unwrap());
    }

    #[test]
    fn test_settlement_rules_config() {
        let config = SettlementRulesConfig {
            equity_days: 3,
            wire_cutoff_time: "15:30".to_string(),
            ..Default::default()
        };

        let rules = config.to_settlement_rules();
        assert_eq!(rules.equity, SettlementType::TPlus(3));
        assert_eq!(
            rules.wire_domestic.cutoff_time,
            NaiveTime::from_hms_opt(15, 30, 0).unwrap()
        );
    }
}
