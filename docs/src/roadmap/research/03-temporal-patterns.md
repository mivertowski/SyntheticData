# Research: Temporal Patterns and Distributions

## Current State Analysis

### Existing Temporal Infrastructure

| Component | Lines | Functionality |
|-----------|-------|---------------|
| `TemporalSampler` | 632 | Date/time sampling with seasonality |
| `IndustrySeasonality` | 538 | 10 industry profiles |
| `HolidayCalendar` | 852 | 6 regional calendars |
| `DriftController` | 373 | Gradual/sudden drift |
| `FiscalPeriod` | 849 | Period close mechanics |
| `BiTemporal` | 449 | Audit trail versioning |

### Current Capabilities

1. **Period-end spikes**: Month-end (2.5x), Quarter-end (4.0x), Year-end (6.0x)
2. **Day-of-week patterns**: Monday catch-up (1.3x), Friday wind-down (0.85x)
3. **Holiday handling**: 6 regions with ~15 holidays each
4. **Working hours**: 8-18 business hours with peak weighting
5. **Industry seasonality**: Black Friday, tax season, etc.

### Current Gaps

1. **No business day calculation** - T+1, T+2 settlement not supported
2. **No fiscal calendar alternatives** - Only calendar year supported
3. **Limited regional coverage** - Missing LATAM, more APAC
4. **No half-day handling** - Early closes before holidays
5. **Static spike multipliers** - No decay curves toward period-end
6. **No timezone awareness** - All times in single timezone
7. **Missing lunar calendars** - Approximate Chinese New Year, Diwali

---

## Improvement Recommendations

### 1. Business Day Calculations

#### 1.1 Core Business Day Engine

```rust
pub struct BusinessDayCalculator {
    calendar: HolidayCalendar,
    weekend_days: HashSet<Weekday>,
    half_day_handling: HalfDayPolicy,
}

pub enum HalfDayPolicy {
    FullDay,           // Count as full business day
    HalfDay,           // Count as 0.5 business days
    NonBusinessDay,    // Treat as holiday
}

impl BusinessDayCalculator {
    /// Add N business days to a date
    pub fn add_business_days(&self, date: NaiveDate, days: i32) -> NaiveDate;

    /// Subtract N business days from a date
    pub fn sub_business_days(&self, date: NaiveDate, days: i32) -> NaiveDate;

    /// Count business days between two dates
    pub fn business_days_between(&self, start: NaiveDate, end: NaiveDate) -> i32;

    /// Get the next business day (inclusive or exclusive)
    pub fn next_business_day(&self, date: NaiveDate, inclusive: bool) -> NaiveDate;

    /// Get the previous business day
    pub fn prev_business_day(&self, date: NaiveDate, inclusive: bool) -> NaiveDate;

    /// Is this date a business day?
    pub fn is_business_day(&self, date: NaiveDate) -> bool;
}
```

#### 1.2 Settlement Date Logic

```yaml
settlement_rules:
  enabled: true
  conventions:
    # Standard equity settlement
    equity:
      type: T_plus_N
      days: 2
      calendar: exchange

    # Government bonds
    government_bonds:
      type: T_plus_N
      days: 1
      calendar: federal

    # Corporate bonds
    corporate_bonds:
      type: T_plus_N
      days: 2
      calendar: combined

    # FX spot
    fx_spot:
      type: T_plus_N
      days: 2
      calendar: both_currencies

    # Wire transfers
    wire_domestic:
      type: same_day_or_next
      cutoff_time: "14:00"
      calendar: federal

    # ACH
    ach:
      type: T_plus_N
      days: 1-3
      distribution: { 1: 0.6, 2: 0.3, 3: 0.1 }
```

#### 1.3 Month-End Conventions

```yaml
month_end_conventions:
  # Modified Following
  modified_following:
    if_holiday: next_business_day
    if_crosses_month: previous_business_day

  # Preceding
  preceding:
    if_holiday: previous_business_day

  # Following
  following:
    if_holiday: next_business_day

  # End of Month
  end_of_month:
    if_start_is_eom: end_stays_eom
```

---

### 2. Expanded Regional Calendars

#### 2.1 Additional Regions

**Latin America**:
```yaml
calendars:
  brazil:
    holidays:
      - name: "Carnival"
        type: floating
        rule: "easter - 47 days"
        duration_days: 2
        activity_multiplier: 0.05

      - name: "Tiradentes Day"
        type: fixed
        month: 4
        day: 21

      - name: "Independence Day"
        type: fixed
        month: 9
        day: 7

      - name: "Republic Day"
        type: fixed
        month: 11
        day: 15

  mexico:
    holidays:
      - name: "Constitution Day"
        type: floating
        rule: "first monday of february"

      - name: "Benito Juárez Birthday"
        type: floating
        rule: "third monday of march"

      - name: "Labor Day"
        type: fixed
        month: 5
        day: 1

      - name: "Independence Day"
        type: fixed
        month: 9
        day: 16

      - name: "Revolution Day"
        type: floating
        rule: "third monday of november"

      - name: "Day of the Dead"
        type: fixed
        month: 11
        day: 2
        activity_multiplier: 0.3
```

**Asia-Pacific Expansion**:
```yaml
  australia:
    holidays:
      - name: "Australia Day"
        type: fixed
        month: 1
        day: 26
        observance: "next_monday_if_weekend"

      - name: "ANZAC Day"
        type: fixed
        month: 4
        day: 25

      - name: "Queen's Birthday"
        type: floating
        rule: "second monday of june"
        regional_variation: true  # Different dates by state

  singapore:
    holidays:
      - name: "Chinese New Year"
        type: lunar
        duration_days: 2

      - name: "Vesak Day"
        type: lunar

      - name: "Hari Raya Puasa"
        type: islamic
        rule: "end of ramadan"

      - name: "Deepavali"
        type: lunar
        calendar: hindu

  south_korea:
    holidays:
      - name: "Seollal"
        type: lunar
        calendar: korean
        duration_days: 3

      - name: "Chuseok"
        type: lunar
        calendar: korean
        duration_days: 3
```

#### 2.2 Lunar Calendar Implementation

```rust
/// Accurate lunar calendar calculations
pub struct LunarCalendar {
    calendar_type: LunarCalendarType,
    cache: HashMap<i32, Vec<LunarDate>>,
}

pub enum LunarCalendarType {
    Chinese,    // Chinese lunisolar
    Islamic,    // Hijri calendar
    Hebrew,     // Jewish calendar
    Hindu,      // Vikram Samvat
    Korean,     // Dangun calendar
}

impl LunarCalendar {
    /// Convert Gregorian date to lunar date
    pub fn to_lunar(&self, date: NaiveDate) -> LunarDate;

    /// Convert lunar date to Gregorian
    pub fn to_gregorian(&self, lunar: LunarDate) -> NaiveDate;

    /// Get Chinese New Year date for a given Gregorian year
    pub fn chinese_new_year(&self, year: i32) -> NaiveDate;

    /// Get Ramadan start date for a given Gregorian year
    pub fn ramadan_start(&self, year: i32) -> NaiveDate;

    /// Get Diwali date (new moon in Kartik)
    pub fn diwali(&self, year: i32) -> NaiveDate;
}
```

---

### 3. Period-End Dynamics

#### 3.1 Decay Curves Instead of Static Multipliers

Replace flat multipliers with realistic acceleration curves:

```yaml
period_end_dynamics:
  enabled: true

  month_end:
    model: exponential_acceleration
    parameters:
      start_day: -10          # 10 days before month-end
      base_multiplier: 1.0
      peak_multiplier: 3.5
      decay_rate: 0.3         # Exponential decay parameter

    # Activity profile by days-to-close
    daily_profile:
      -10: 1.0
      -7: 1.2
      -5: 1.5
      -3: 2.0
      -2: 2.5
      -1: 3.0                 # Day before close
      0: 3.5                  # Close day

  quarter_end:
    model: stepped_exponential
    inherit_from: month_end
    additional_multiplier: 1.5

  year_end:
    model: extended_crunch
    parameters:
      start_day: -15
      sustained_high_days: 5
      peak_multiplier: 6.0

    # Year-end specific activities
    activities:
      - type: "audit_adjustments"
        days: [-3, -2, -1, 0]
        multiplier: 2.0
      - type: "tax_provisions"
        days: [-5, -4, -3]
        multiplier: 1.5
      - type: "impairment_reviews"
        days: [-10, -9, -8]
        multiplier: 1.3
```

#### 3.2 Intra-Day Patterns

```yaml
intraday_patterns:
  # Morning rush
  morning_spike:
    start: "08:30"
    end: "10:00"
    multiplier: 1.8

  # Pre-lunch activity
  late_morning:
    start: "10:00"
    end: "12:00"
    multiplier: 1.2

  # Lunch lull
  lunch_dip:
    start: "12:00"
    end: "13:30"
    multiplier: 0.4

  # Afternoon steady
  afternoon:
    start: "13:30"
    end: "16:00"
    multiplier: 1.0

  # End-of-day push
  eod_rush:
    start: "16:00"
    end: "17:30"
    multiplier: 1.5

  # After hours (manual only)
  after_hours:
    start: "17:30"
    end: "20:00"
    multiplier: 0.15
    type: manual_only
```

#### 3.3 Time Zone Handling

```yaml
timezones:
  enabled: true

  company_timezones:
    default: "America/New_York"
    by_entity:
      - entity_pattern: "EU_*"
        timezone: "Europe/London"
      - entity_pattern: "DE_*"
        timezone: "Europe/Berlin"
      - entity_pattern: "APAC_*"
        timezone: "Asia/Singapore"
      - entity_pattern: "JP_*"
        timezone: "Asia/Tokyo"

  posting_behavior:
    # Consolidation timing
    consolidation:
      coordinator_timezone: "America/New_York"
      cutoff_time: "18:00"

    # Intercompany coordination
    intercompany:
      settlement_timezone: "UTC"
      matching_window_hours: 24
```

---

### 4. Fiscal Calendar Alternatives

#### 4.1 Non-Calendar Year Support

```yaml
fiscal_calendar:
  type: custom
  year_start:
    month: 7
    day: 1
  # Fiscal year 2024 = July 1, 2024 - June 30, 2025

  period_naming:
    format: "FY{year}P{period:02}"
    # FY2024P01 = July 2024
```

#### 4.2 4-4-5 Calendar

```yaml
fiscal_calendar:
  type: 4-4-5
  year_start:
    anchor: first_sunday_of_february
    # Or: last_saturday_of_january

  periods:
    Q1:
      - weeks: 4
      - weeks: 4
      - weeks: 5
    Q2:
      - weeks: 4
      - weeks: 4
      - weeks: 5
    Q3:
      - weeks: 4
      - weeks: 4
      - weeks: 5
    Q4:
      - weeks: 4
      - weeks: 4
      - weeks: 5

  # 53rd week handling (every 5-6 years)
  leap_week:
    occurrence: calculated
    placement: Q4_P3  # Added to last period
```

#### 4.3 13-Period Calendar

```yaml
fiscal_calendar:
  type: 13_period
  weeks_per_period: 4
  year_start:
    anchor: first_monday_of_january

  # 53rd week handling
  extra_week_period: 13
```

---

### 5. Advanced Seasonality

#### 5.1 Multi-Factor Seasonality

```yaml
seasonality:
  factors:
    # Annual cycle
    annual:
      type: fourier
      harmonics: 3
      coefficients:
        cos1: 0.15
        sin1: 0.08
        cos2: 0.05
        sin2: 0.03
        cos3: 0.02
        sin3: 0.01

    # Weekly cycle
    weekly:
      type: categorical
      values:
        monday: 1.25
        tuesday: 1.10
        wednesday: 1.00
        thursday: 1.00
        friday: 0.90
        saturday: 0.15
        sunday: 0.05

    # Monthly cycle (within month)
    monthly:
      type: piecewise
      segments:
        - days: [1, 5]
          multiplier: 1.3
          label: "month_start"
        - days: [6, 20]
          multiplier: 0.9
          label: "mid_month"
        - days: [21, 31]
          multiplier: 1.4
          label: "month_end"

  # Interaction effects
  interactions:
    - factors: [annual, weekly]
      type: multiplicative
    - factors: [monthly, weekly]
      type: additive
```

#### 5.2 Weather-Driven Seasonality

For relevant industries:

```yaml
weather_seasonality:
  enabled: true
  industries: [retail, utilities, agriculture, construction]

  patterns:
    temperature:
      cold_threshold: 32  # Fahrenheit
      hot_threshold: 85
      effects:
        cold:
          utilities: 1.8
          construction: 0.5
          retail_outdoor: 0.3
        hot:
          utilities: 1.5
          construction: 0.8
          retail_outdoor: 1.3

    precipitation:
      effects:
        rain:
          construction: 0.6
          retail_brick_mortar: 0.8
          retail_online: 1.2

  # Regional weather profiles
  regional_profiles:
    northeast_us:
      winter_severity: high
      summer_humidity: medium
    southwest_us:
      winter_severity: low
      summer_heat: extreme
    pacific_northwest:
      precipitation_days: high
      temperature_variance: low
```

---

### 6. Transaction Timing Realism

#### 6.1 Processing Lag Modeling

```yaml
processing_lags:
  # Time between event and posting
  event_to_posting:
    distribution: lognormal
    parameters:
      sales_order:
        mu: 0.5    # ~1.6 hours median
        sigma: 0.8
      goods_receipt:
        mu: 1.5    # ~4.5 hours median
        sigma: 0.5
      invoice_receipt:
        mu: 2.0    # ~7.4 hours median
        sigma: 0.6
      payment:
        mu: 0.2    # ~1.2 hours median
        sigma: 0.3

  # Day-boundary crossing
  cross_day_posting:
    enabled: true
    probability_by_hour:
      "17:00": 0.7   # 70% post next day if after 5pm
      "19:00": 0.9
      "21:00": 0.99

  # Batch processing delays
  batch_delays:
    enabled: true
    schedules:
      nightly_batch:
        run_time: "02:00"
        affects: [bank_transactions, interfaces]
      hourly_sync:
        interval_minutes: 60
        affects: [inventory_movements]
```

#### 6.2 Human vs. System Posting Patterns

```yaml
posting_patterns:
  human:
    # Working hours focus
    primary_hours: [9, 10, 11, 14, 15, 16]
    probability: 0.8

    # Occasional overtime
    extended_hours: [8, 17, 18, 19]
    probability: 0.15

    # Rare late night
    late_hours: [20, 21, 22]
    probability: 0.05

    # Keystroke timing (for detailed simulation)
    entry_duration:
      simple_je:
        mean_seconds: 45
        std_seconds: 15
      complex_je:
        mean_seconds: 180
        std_seconds: 60

  system:
    # Interface postings
    interface:
      typical_times: ["01:00", "05:00", "13:00"]
      duration_minutes: 15-45
      burst_rate: 100-500  # Records per minute

    # Automated recurring
    recurring:
      time: "00:30"
      day: first_business_day

    # Real-time integrations
    realtime:
      latency_ms: 100-500
      batch_size: 1
```

---

### 7. Period Close Orchestration

#### 7.1 Close Calendar Generation

```yaml
close_calendar:
  enabled: true

  # Standard close schedule
  monthly:
    soft_close:
      day: 2        # 2nd business day
      activities: [preliminary_review, initial_accruals]
    hard_close:
      day: 5        # 5th business day
      activities: [final_adjustments, lock_period]
    reporting:
      day: 7        # 7th business day
      activities: [management_reports, variance_analysis]

  quarterly:
    extended_close:
      additional_days: 3
    activities:
      - quarter_end_reserves
      - intercompany_reconciliation
      - consolidation

  annual:
    extended_close:
      additional_days: 10
    activities:
      - audit_adjustments
      - tax_provisions
      - impairment_testing
      - goodwill_analysis
      - segment_reporting
```

#### 7.2 Late Posting Behavior

```yaml
late_postings:
  enabled: true

  # Probability of late posting by days after close
  probability_curve:
    day_1: 0.08    # 8% of transactions post 1 day late
    day_2: 0.03
    day_3: 0.01
    day_4: 0.005
    day_5+: 0.002

  # Characteristics of late postings
  characteristics:
    # More likely to be corrections
    correction_probability: 0.4
    # Higher average amount
    amount_multiplier: 1.5
    # Require special approval
    approval_required: true
    # Must reference original period
    period_reference: required
```

---

### 8. Implementation Priority

| Enhancement | Complexity | Impact | Priority |
|-------------|------------|--------|----------|
| Business day calculator | Medium | Critical | P1 |
| Additional regional calendars | Medium | High | P1 |
| Decay curves for period-end | Low | High | P1 |
| Non-calendar fiscal years | Medium | Medium | P2 |
| 4-4-5 calendar support | High | Medium | P2 |
| Timezone handling | Medium | Medium | P2 |
| Lunar calendar accuracy | High | Medium | P3 |
| Weather seasonality | Medium | Low | P3 |
| Intra-day patterns | Low | Medium | P2 |
| Processing lag modeling | Medium | High | P1 |

---

### 9. Validation Metrics

```yaml
temporal_validation:
  metrics:
    # Period-end spike ratios
    period_end_spikes:
      month_end_ratio:
        expected: 2.0-3.0
        tolerance: 0.5
      quarter_end_ratio:
        expected: 3.5-4.5
        tolerance: 0.5
      year_end_ratio:
        expected: 5.0-7.0
        tolerance: 1.0

    # Day-of-week distribution
    dow_distribution:
      test: chi_squared
      expected_weights: [1.3, 1.1, 1.0, 1.0, 0.85, 0.1, 0.05]
      significance: 0.05

    # Holiday compliance
    holiday_activity:
      max_activity_on_holiday: 0.1
      allow_exceptions: ["bank_settlement"]

    # Business hours
    business_hours:
      human_transactions:
        in_hours_rate: 0.85-0.95
      system_transactions:
        off_hours_allowed: true

    # Late posting rate
    late_postings:
      max_rate: 0.15
      concentration_test: true  # Should not cluster
```

---

*See also*: [04-interconnectivity.md](04-interconnectivity.md) for relationship modeling
