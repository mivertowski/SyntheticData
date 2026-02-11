//! Regional holiday calendars for transaction generation.
//!
//! Supports holidays for US, DE (Germany), GB (UK), CN (China),
//! JP (Japan), and IN (India) with appropriate activity multipliers.

use chrono::{Datelike, Duration, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

/// Supported regions for holiday calendars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Region {
    /// United States
    US,
    /// Germany
    DE,
    /// United Kingdom
    GB,
    /// China
    CN,
    /// Japan
    JP,
    /// India
    IN,
    /// Brazil
    BR,
    /// Mexico
    MX,
    /// Australia
    AU,
    /// Singapore
    SG,
    /// South Korea
    KR,
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Region::US => write!(f, "United States"),
            Region::DE => write!(f, "Germany"),
            Region::GB => write!(f, "United Kingdom"),
            Region::CN => write!(f, "China"),
            Region::JP => write!(f, "Japan"),
            Region::IN => write!(f, "India"),
            Region::BR => write!(f, "Brazil"),
            Region::MX => write!(f, "Mexico"),
            Region::AU => write!(f, "Australia"),
            Region::SG => write!(f, "Singapore"),
            Region::KR => write!(f, "South Korea"),
        }
    }
}

/// A holiday with its associated activity multiplier.
#[derive(Debug, Clone)]
pub struct Holiday {
    /// Holiday name.
    pub name: String,
    /// Date of the holiday.
    pub date: NaiveDate,
    /// Activity multiplier (0.0 = completely closed, 1.0 = normal).
    pub activity_multiplier: f64,
    /// Whether this is a bank holiday (affects financial transactions).
    pub is_bank_holiday: bool,
}

impl Holiday {
    /// Create a new holiday.
    pub fn new(name: impl Into<String>, date: NaiveDate, multiplier: f64) -> Self {
        Self {
            name: name.into(),
            date,
            activity_multiplier: multiplier,
            is_bank_holiday: true,
        }
    }

    /// Set whether this is a bank holiday.
    pub fn with_bank_holiday(mut self, is_bank_holiday: bool) -> Self {
        self.is_bank_holiday = is_bank_holiday;
        self
    }
}

/// A calendar of holidays for a specific region and year.
#[derive(Debug, Clone)]
pub struct HolidayCalendar {
    /// Region for this calendar.
    pub region: Region,
    /// Year for this calendar.
    pub year: i32,
    /// List of holidays.
    pub holidays: Vec<Holiday>,
}

impl HolidayCalendar {
    /// Create a new empty holiday calendar.
    pub fn new(region: Region, year: i32) -> Self {
        Self {
            region,
            year,
            holidays: Vec::new(),
        }
    }

    /// Create a holiday calendar for a specific region and year.
    pub fn for_region(region: Region, year: i32) -> Self {
        match region {
            Region::US => Self::us_holidays(year),
            Region::DE => Self::de_holidays(year),
            Region::GB => Self::gb_holidays(year),
            Region::CN => Self::cn_holidays(year),
            Region::JP => Self::jp_holidays(year),
            Region::IN => Self::in_holidays(year),
            Region::BR => Self::br_holidays(year),
            Region::MX => Self::mx_holidays(year),
            Region::AU => Self::au_holidays(year),
            Region::SG => Self::sg_holidays(year),
            Region::KR => Self::kr_holidays(year),
        }
    }

    /// Check if a date is a holiday.
    pub fn is_holiday(&self, date: NaiveDate) -> bool {
        self.holidays.iter().any(|h| h.date == date)
    }

    /// Get the activity multiplier for a date.
    pub fn get_multiplier(&self, date: NaiveDate) -> f64 {
        self.holidays
            .iter()
            .find(|h| h.date == date)
            .map(|h| h.activity_multiplier)
            .unwrap_or(1.0)
    }

    /// Get all holidays for a date (may include multiple on same day).
    pub fn get_holidays(&self, date: NaiveDate) -> Vec<&Holiday> {
        self.holidays.iter().filter(|h| h.date == date).collect()
    }

    /// Add a holiday to the calendar.
    pub fn add_holiday(&mut self, holiday: Holiday) {
        self.holidays.push(holiday);
    }

    /// Get all dates in the calendar.
    pub fn all_dates(&self) -> Vec<NaiveDate> {
        self.holidays.iter().map(|h| h.date).collect()
    }

    /// US Federal Holidays.
    fn us_holidays(year: i32) -> Self {
        let mut cal = Self::new(Region::US, year);

        // New Year's Day - Jan 1 (observed)
        let new_years = NaiveDate::from_ymd_opt(year, 1, 1).expect("valid date components");
        cal.add_holiday(Holiday::new(
            "New Year's Day",
            Self::observe_weekend(new_years),
            0.02,
        ));

        // Martin Luther King Jr. Day - 3rd Monday of January
        let mlk = Self::nth_weekday_of_month(year, 1, Weekday::Mon, 3);
        cal.add_holiday(Holiday::new("Martin Luther King Jr. Day", mlk, 0.1));

        // Presidents' Day - 3rd Monday of February
        let presidents = Self::nth_weekday_of_month(year, 2, Weekday::Mon, 3);
        cal.add_holiday(Holiday::new("Presidents' Day", presidents, 0.1));

        // Memorial Day - Last Monday of May
        let memorial = Self::last_weekday_of_month(year, 5, Weekday::Mon);
        cal.add_holiday(Holiday::new("Memorial Day", memorial, 0.05));

        // Juneteenth - June 19
        let juneteenth = NaiveDate::from_ymd_opt(year, 6, 19).expect("valid date components");
        cal.add_holiday(Holiday::new(
            "Juneteenth",
            Self::observe_weekend(juneteenth),
            0.1,
        ));

        // Independence Day - July 4
        let independence = NaiveDate::from_ymd_opt(year, 7, 4).expect("valid date components");
        cal.add_holiday(Holiday::new(
            "Independence Day",
            Self::observe_weekend(independence),
            0.02,
        ));

        // Labor Day - 1st Monday of September
        let labor = Self::nth_weekday_of_month(year, 9, Weekday::Mon, 1);
        cal.add_holiday(Holiday::new("Labor Day", labor, 0.05));

        // Columbus Day - 2nd Monday of October
        let columbus = Self::nth_weekday_of_month(year, 10, Weekday::Mon, 2);
        cal.add_holiday(Holiday::new("Columbus Day", columbus, 0.2));

        // Veterans Day - November 11
        let veterans = NaiveDate::from_ymd_opt(year, 11, 11).expect("valid date components");
        cal.add_holiday(Holiday::new(
            "Veterans Day",
            Self::observe_weekend(veterans),
            0.1,
        ));

        // Thanksgiving - 4th Thursday of November
        let thanksgiving = Self::nth_weekday_of_month(year, 11, Weekday::Thu, 4);
        cal.add_holiday(Holiday::new("Thanksgiving", thanksgiving, 0.02));

        // Day after Thanksgiving
        cal.add_holiday(Holiday::new(
            "Day after Thanksgiving",
            thanksgiving + Duration::days(1),
            0.1,
        ));

        // Christmas Eve - December 24
        let christmas_eve = NaiveDate::from_ymd_opt(year, 12, 24).expect("valid date components");
        cal.add_holiday(Holiday::new("Christmas Eve", christmas_eve, 0.1));

        // Christmas Day - December 25
        let christmas = NaiveDate::from_ymd_opt(year, 12, 25).expect("valid date components");
        cal.add_holiday(Holiday::new(
            "Christmas Day",
            Self::observe_weekend(christmas),
            0.02,
        ));

        // New Year's Eve - December 31
        let new_years_eve = NaiveDate::from_ymd_opt(year, 12, 31).expect("valid date components");
        cal.add_holiday(Holiday::new("New Year's Eve", new_years_eve, 0.1));

        cal
    }

    /// German holidays (nationwide).
    fn de_holidays(year: i32) -> Self {
        let mut cal = Self::new(Region::DE, year);

        // Neujahr - January 1
        cal.add_holiday(Holiday::new(
            "Neujahr",
            NaiveDate::from_ymd_opt(year, 1, 1).expect("valid date components"),
            0.02,
        ));

        // Karfreitag - Good Friday (Easter - 2 days)
        let easter = Self::easter_date(year);
        cal.add_holiday(Holiday::new("Karfreitag", easter - Duration::days(2), 0.02));

        // Ostermontag - Easter Monday
        cal.add_holiday(Holiday::new(
            "Ostermontag",
            easter + Duration::days(1),
            0.02,
        ));

        // Tag der Arbeit - May 1
        cal.add_holiday(Holiday::new(
            "Tag der Arbeit",
            NaiveDate::from_ymd_opt(year, 5, 1).expect("valid date components"),
            0.02,
        ));

        // Christi Himmelfahrt - Ascension Day (Easter + 39 days)
        cal.add_holiday(Holiday::new(
            "Christi Himmelfahrt",
            easter + Duration::days(39),
            0.02,
        ));

        // Pfingstmontag - Whit Monday (Easter + 50 days)
        cal.add_holiday(Holiday::new(
            "Pfingstmontag",
            easter + Duration::days(50),
            0.02,
        ));

        // Tag der Deutschen Einheit - October 3
        cal.add_holiday(Holiday::new(
            "Tag der Deutschen Einheit",
            NaiveDate::from_ymd_opt(year, 10, 3).expect("valid date components"),
            0.02,
        ));

        // Weihnachten - December 25-26
        cal.add_holiday(Holiday::new(
            "1. Weihnachtstag",
            NaiveDate::from_ymd_opt(year, 12, 25).expect("valid date components"),
            0.02,
        ));
        cal.add_holiday(Holiday::new(
            "2. Weihnachtstag",
            NaiveDate::from_ymd_opt(year, 12, 26).expect("valid date components"),
            0.02,
        ));

        // Silvester - December 31
        cal.add_holiday(Holiday::new(
            "Silvester",
            NaiveDate::from_ymd_opt(year, 12, 31).expect("valid date components"),
            0.1,
        ));

        cal
    }

    /// UK bank holidays.
    fn gb_holidays(year: i32) -> Self {
        let mut cal = Self::new(Region::GB, year);

        // New Year's Day
        let new_years = NaiveDate::from_ymd_opt(year, 1, 1).expect("valid date components");
        cal.add_holiday(Holiday::new(
            "New Year's Day",
            Self::observe_weekend(new_years),
            0.02,
        ));

        // Good Friday
        let easter = Self::easter_date(year);
        cal.add_holiday(Holiday::new(
            "Good Friday",
            easter - Duration::days(2),
            0.02,
        ));

        // Easter Monday
        cal.add_holiday(Holiday::new(
            "Easter Monday",
            easter + Duration::days(1),
            0.02,
        ));

        // Early May Bank Holiday - 1st Monday of May
        let early_may = Self::nth_weekday_of_month(year, 5, Weekday::Mon, 1);
        cal.add_holiday(Holiday::new("Early May Bank Holiday", early_may, 0.02));

        // Spring Bank Holiday - Last Monday of May
        let spring = Self::last_weekday_of_month(year, 5, Weekday::Mon);
        cal.add_holiday(Holiday::new("Spring Bank Holiday", spring, 0.02));

        // Summer Bank Holiday - Last Monday of August
        let summer = Self::last_weekday_of_month(year, 8, Weekday::Mon);
        cal.add_holiday(Holiday::new("Summer Bank Holiday", summer, 0.02));

        // Christmas Day
        let christmas = NaiveDate::from_ymd_opt(year, 12, 25).expect("valid date components");
        cal.add_holiday(Holiday::new(
            "Christmas Day",
            Self::observe_weekend(christmas),
            0.02,
        ));

        // Boxing Day
        let boxing = NaiveDate::from_ymd_opt(year, 12, 26).expect("valid date components");
        cal.add_holiday(Holiday::new(
            "Boxing Day",
            Self::observe_weekend(boxing),
            0.02,
        ));

        cal
    }

    /// Chinese holidays (simplified - fixed dates only).
    fn cn_holidays(year: i32) -> Self {
        let mut cal = Self::new(Region::CN, year);

        // New Year's Day - January 1
        cal.add_holiday(Holiday::new(
            "New Year",
            NaiveDate::from_ymd_opt(year, 1, 1).expect("valid date components"),
            0.05,
        ));

        // Spring Festival (Chinese New Year) - approximate late Jan/early Feb
        // Using a simplified calculation - typically 7-day holiday
        let cny = Self::approximate_chinese_new_year(year);
        for i in 0..7 {
            cal.add_holiday(Holiday::new(
                if i == 0 {
                    "Spring Festival"
                } else {
                    "Spring Festival Holiday"
                },
                cny + Duration::days(i),
                0.02,
            ));
        }

        // Qingming Festival - April 4-6 (approximate)
        cal.add_holiday(Holiday::new(
            "Qingming Festival",
            NaiveDate::from_ymd_opt(year, 4, 5).expect("valid date components"),
            0.05,
        ));

        // Labor Day - May 1 (3-day holiday)
        for i in 0..3 {
            cal.add_holiday(Holiday::new(
                if i == 0 {
                    "Labor Day"
                } else {
                    "Labor Day Holiday"
                },
                NaiveDate::from_ymd_opt(year, 5, 1).expect("valid date components")
                    + Duration::days(i),
                0.05,
            ));
        }

        // Dragon Boat Festival - approximate early June
        cal.add_holiday(Holiday::new(
            "Dragon Boat Festival",
            NaiveDate::from_ymd_opt(year, 6, 10).expect("valid date components"),
            0.05,
        ));

        // Mid-Autumn Festival - approximate late September
        cal.add_holiday(Holiday::new(
            "Mid-Autumn Festival",
            NaiveDate::from_ymd_opt(year, 9, 15).expect("valid date components"),
            0.05,
        ));

        // National Day - October 1 (7-day holiday)
        for i in 0..7 {
            cal.add_holiday(Holiday::new(
                if i == 0 {
                    "National Day"
                } else {
                    "National Day Holiday"
                },
                NaiveDate::from_ymd_opt(year, 10, 1).expect("valid date components")
                    + Duration::days(i),
                0.02,
            ));
        }

        cal
    }

    /// Japanese holidays.
    fn jp_holidays(year: i32) -> Self {
        let mut cal = Self::new(Region::JP, year);

        // Ganjitsu - January 1
        cal.add_holiday(Holiday::new(
            "Ganjitsu (New Year)",
            NaiveDate::from_ymd_opt(year, 1, 1).expect("valid date components"),
            0.02,
        ));

        // New Year holidays - January 2-3
        cal.add_holiday(Holiday::new(
            "New Year Holiday",
            NaiveDate::from_ymd_opt(year, 1, 2).expect("valid date components"),
            0.05,
        ));
        cal.add_holiday(Holiday::new(
            "New Year Holiday",
            NaiveDate::from_ymd_opt(year, 1, 3).expect("valid date components"),
            0.05,
        ));

        // Seijin no Hi - Coming of Age Day - 2nd Monday of January
        let seijin = Self::nth_weekday_of_month(year, 1, Weekday::Mon, 2);
        cal.add_holiday(Holiday::new("Seijin no Hi", seijin, 0.05));

        // Kenkoku Kinen no Hi - National Foundation Day - February 11
        cal.add_holiday(Holiday::new(
            "Kenkoku Kinen no Hi",
            NaiveDate::from_ymd_opt(year, 2, 11).expect("valid date components"),
            0.02,
        ));

        // Tenno Tanjobi - Emperor's Birthday - February 23
        cal.add_holiday(Holiday::new(
            "Tenno Tanjobi",
            NaiveDate::from_ymd_opt(year, 2, 23).expect("valid date components"),
            0.02,
        ));

        // Shunbun no Hi - Vernal Equinox - around March 20-21
        cal.add_holiday(Holiday::new(
            "Shunbun no Hi",
            NaiveDate::from_ymd_opt(year, 3, 20).expect("valid date components"),
            0.02,
        ));

        // Showa no Hi - Showa Day - April 29
        cal.add_holiday(Holiday::new(
            "Showa no Hi",
            NaiveDate::from_ymd_opt(year, 4, 29).expect("valid date components"),
            0.02,
        ));

        // Golden Week - April 29 - May 5
        cal.add_holiday(Holiday::new(
            "Kenpo Kinenbi",
            NaiveDate::from_ymd_opt(year, 5, 3).expect("valid date components"),
            0.02,
        ));
        cal.add_holiday(Holiday::new(
            "Midori no Hi",
            NaiveDate::from_ymd_opt(year, 5, 4).expect("valid date components"),
            0.02,
        ));
        cal.add_holiday(Holiday::new(
            "Kodomo no Hi",
            NaiveDate::from_ymd_opt(year, 5, 5).expect("valid date components"),
            0.02,
        ));

        // Umi no Hi - Marine Day - 3rd Monday of July
        let umi = Self::nth_weekday_of_month(year, 7, Weekday::Mon, 3);
        cal.add_holiday(Holiday::new("Umi no Hi", umi, 0.05));

        // Yama no Hi - Mountain Day - August 11
        cal.add_holiday(Holiday::new(
            "Yama no Hi",
            NaiveDate::from_ymd_opt(year, 8, 11).expect("valid date components"),
            0.05,
        ));

        // Keiro no Hi - Respect for the Aged Day - 3rd Monday of September
        let keiro = Self::nth_weekday_of_month(year, 9, Weekday::Mon, 3);
        cal.add_holiday(Holiday::new("Keiro no Hi", keiro, 0.05));

        // Shubun no Hi - Autumnal Equinox - around September 22-23
        cal.add_holiday(Holiday::new(
            "Shubun no Hi",
            NaiveDate::from_ymd_opt(year, 9, 23).expect("valid date components"),
            0.02,
        ));

        // Sports Day - 2nd Monday of October
        let sports = Self::nth_weekday_of_month(year, 10, Weekday::Mon, 2);
        cal.add_holiday(Holiday::new("Sports Day", sports, 0.05));

        // Bunka no Hi - Culture Day - November 3
        cal.add_holiday(Holiday::new(
            "Bunka no Hi",
            NaiveDate::from_ymd_opt(year, 11, 3).expect("valid date components"),
            0.02,
        ));

        // Kinro Kansha no Hi - Labor Thanksgiving Day - November 23
        cal.add_holiday(Holiday::new(
            "Kinro Kansha no Hi",
            NaiveDate::from_ymd_opt(year, 11, 23).expect("valid date components"),
            0.02,
        ));

        cal
    }

    /// Indian holidays (national holidays).
    fn in_holidays(year: i32) -> Self {
        let mut cal = Self::new(Region::IN, year);

        // Republic Day - January 26
        cal.add_holiday(Holiday::new(
            "Republic Day",
            NaiveDate::from_ymd_opt(year, 1, 26).expect("valid date components"),
            0.02,
        ));

        // Holi - approximate March (lunar calendar)
        cal.add_holiday(Holiday::new(
            "Holi",
            NaiveDate::from_ymd_opt(year, 3, 10).expect("valid date components"),
            0.05,
        ));

        // Good Friday
        let easter = Self::easter_date(year);
        cal.add_holiday(Holiday::new(
            "Good Friday",
            easter - Duration::days(2),
            0.05,
        ));

        // Independence Day - August 15
        cal.add_holiday(Holiday::new(
            "Independence Day",
            NaiveDate::from_ymd_opt(year, 8, 15).expect("valid date components"),
            0.02,
        ));

        // Gandhi Jayanti - October 2
        cal.add_holiday(Holiday::new(
            "Gandhi Jayanti",
            NaiveDate::from_ymd_opt(year, 10, 2).expect("valid date components"),
            0.02,
        ));

        // Dussehra - approximate October (lunar calendar)
        cal.add_holiday(Holiday::new(
            "Dussehra",
            NaiveDate::from_ymd_opt(year, 10, 15).expect("valid date components"),
            0.05,
        ));

        // Diwali - approximate October/November (5-day festival)
        let diwali = Self::approximate_diwali(year);
        for i in 0..5 {
            cal.add_holiday(Holiday::new(
                match i {
                    0 => "Dhanteras",
                    1 => "Naraka Chaturdashi",
                    2 => "Diwali",
                    3 => "Govardhan Puja",
                    _ => "Bhai Dooj",
                },
                diwali + Duration::days(i),
                if i == 2 { 0.02 } else { 0.1 },
            ));
        }

        // Christmas - December 25
        cal.add_holiday(Holiday::new(
            "Christmas",
            NaiveDate::from_ymd_opt(year, 12, 25).expect("valid date components"),
            0.1,
        ));

        cal
    }

    /// Brazilian holidays (national holidays).
    fn br_holidays(year: i32) -> Self {
        let mut cal = Self::new(Region::BR, year);

        // Confraternização Universal - January 1
        cal.add_holiday(Holiday::new(
            "Confraternização Universal",
            NaiveDate::from_ymd_opt(year, 1, 1).expect("valid date components"),
            0.02,
        ));

        // Carnaval - Tuesday before Ash Wednesday (47 days before Easter)
        let easter = Self::easter_date(year);
        let carnival_tuesday = easter - Duration::days(47);
        let carnival_monday = carnival_tuesday - Duration::days(1);
        cal.add_holiday(Holiday::new("Carnaval (Segunda)", carnival_monday, 0.02));
        cal.add_holiday(Holiday::new("Carnaval (Terça)", carnival_tuesday, 0.02));

        // Sexta-feira Santa - Good Friday
        cal.add_holiday(Holiday::new(
            "Sexta-feira Santa",
            easter - Duration::days(2),
            0.02,
        ));

        // Tiradentes - April 21
        cal.add_holiday(Holiday::new(
            "Tiradentes",
            NaiveDate::from_ymd_opt(year, 4, 21).expect("valid date components"),
            0.02,
        ));

        // Dia do Trabalho - May 1
        cal.add_holiday(Holiday::new(
            "Dia do Trabalho",
            NaiveDate::from_ymd_opt(year, 5, 1).expect("valid date components"),
            0.02,
        ));

        // Corpus Christi - 60 days after Easter
        cal.add_holiday(Holiday::new(
            "Corpus Christi",
            easter + Duration::days(60),
            0.05,
        ));

        // Independência do Brasil - September 7
        cal.add_holiday(Holiday::new(
            "Independência do Brasil",
            NaiveDate::from_ymd_opt(year, 9, 7).expect("valid date components"),
            0.02,
        ));

        // Nossa Senhora Aparecida - October 12
        cal.add_holiday(Holiday::new(
            "Nossa Senhora Aparecida",
            NaiveDate::from_ymd_opt(year, 10, 12).expect("valid date components"),
            0.02,
        ));

        // Finados - November 2
        cal.add_holiday(Holiday::new(
            "Finados",
            NaiveDate::from_ymd_opt(year, 11, 2).expect("valid date components"),
            0.02,
        ));

        // Proclamação da República - November 15
        cal.add_holiday(Holiday::new(
            "Proclamação da República",
            NaiveDate::from_ymd_opt(year, 11, 15).expect("valid date components"),
            0.02,
        ));

        // Natal - December 25
        cal.add_holiday(Holiday::new(
            "Natal",
            NaiveDate::from_ymd_opt(year, 12, 25).expect("valid date components"),
            0.02,
        ));

        cal
    }

    /// Mexican holidays (national holidays).
    fn mx_holidays(year: i32) -> Self {
        let mut cal = Self::new(Region::MX, year);

        // Año Nuevo - January 1
        cal.add_holiday(Holiday::new(
            "Año Nuevo",
            NaiveDate::from_ymd_opt(year, 1, 1).expect("valid date components"),
            0.02,
        ));

        // Día de la Constitución - First Monday of February
        let constitution = Self::nth_weekday_of_month(year, 2, Weekday::Mon, 1);
        cal.add_holiday(Holiday::new("Día de la Constitución", constitution, 0.02));

        // Natalicio de Benito Juárez - Third Monday of March
        let juarez = Self::nth_weekday_of_month(year, 3, Weekday::Mon, 3);
        cal.add_holiday(Holiday::new("Natalicio de Benito Juárez", juarez, 0.02));

        // Semana Santa - Holy Thursday and Good Friday
        let easter = Self::easter_date(year);
        cal.add_holiday(Holiday::new(
            "Jueves Santo",
            easter - Duration::days(3),
            0.05,
        ));
        cal.add_holiday(Holiday::new(
            "Viernes Santo",
            easter - Duration::days(2),
            0.02,
        ));

        // Día del Trabajo - May 1
        cal.add_holiday(Holiday::new(
            "Día del Trabajo",
            NaiveDate::from_ymd_opt(year, 5, 1).expect("valid date components"),
            0.02,
        ));

        // Día de la Independencia - September 16
        cal.add_holiday(Holiday::new(
            "Día de la Independencia",
            NaiveDate::from_ymd_opt(year, 9, 16).expect("valid date components"),
            0.02,
        ));

        // Día de la Revolución - Third Monday of November
        let revolution = Self::nth_weekday_of_month(year, 11, Weekday::Mon, 3);
        cal.add_holiday(Holiday::new("Día de la Revolución", revolution, 0.02));

        // Día de Muertos - November 1-2 (not official but widely observed)
        cal.add_holiday(Holiday::new(
            "Día de Muertos",
            NaiveDate::from_ymd_opt(year, 11, 1).expect("valid date components"),
            0.1,
        ));
        cal.add_holiday(Holiday::new(
            "Día de Muertos",
            NaiveDate::from_ymd_opt(year, 11, 2).expect("valid date components"),
            0.1,
        ));

        // Navidad - December 25
        cal.add_holiday(Holiday::new(
            "Navidad",
            NaiveDate::from_ymd_opt(year, 12, 25).expect("valid date components"),
            0.02,
        ));

        cal
    }

    /// Australian holidays (national holidays).
    fn au_holidays(year: i32) -> Self {
        let mut cal = Self::new(Region::AU, year);

        // New Year's Day - January 1
        let new_years = NaiveDate::from_ymd_opt(year, 1, 1).expect("valid date components");
        cal.add_holiday(Holiday::new(
            "New Year's Day",
            Self::observe_weekend(new_years),
            0.02,
        ));

        // Australia Day - January 26 (observed)
        let australia_day = NaiveDate::from_ymd_opt(year, 1, 26).expect("valid date components");
        cal.add_holiday(Holiday::new(
            "Australia Day",
            Self::observe_weekend(australia_day),
            0.02,
        ));

        // Good Friday
        let easter = Self::easter_date(year);
        cal.add_holiday(Holiday::new(
            "Good Friday",
            easter - Duration::days(2),
            0.02,
        ));

        // Easter Saturday
        cal.add_holiday(Holiday::new(
            "Easter Saturday",
            easter - Duration::days(1),
            0.02,
        ));

        // Easter Monday
        cal.add_holiday(Holiday::new(
            "Easter Monday",
            easter + Duration::days(1),
            0.02,
        ));

        // ANZAC Day - April 25
        let anzac = NaiveDate::from_ymd_opt(year, 4, 25).expect("valid date components");
        cal.add_holiday(Holiday::new("ANZAC Day", anzac, 0.02));

        // Queen's Birthday - Second Monday of June (varies by state, using NSW)
        let queens_birthday = Self::nth_weekday_of_month(year, 6, Weekday::Mon, 2);
        cal.add_holiday(Holiday::new("Queen's Birthday", queens_birthday, 0.02));

        // Christmas Day
        let christmas = NaiveDate::from_ymd_opt(year, 12, 25).expect("valid date components");
        cal.add_holiday(Holiday::new(
            "Christmas Day",
            Self::observe_weekend(christmas),
            0.02,
        ));

        // Boxing Day - December 26
        let boxing = NaiveDate::from_ymd_opt(year, 12, 26).expect("valid date components");
        cal.add_holiday(Holiday::new(
            "Boxing Day",
            Self::observe_weekend(boxing),
            0.02,
        ));

        cal
    }

    /// Singaporean holidays (national holidays).
    fn sg_holidays(year: i32) -> Self {
        let mut cal = Self::new(Region::SG, year);

        // New Year's Day - January 1
        cal.add_holiday(Holiday::new(
            "New Year's Day",
            NaiveDate::from_ymd_opt(year, 1, 1).expect("valid date components"),
            0.02,
        ));

        // Chinese New Year (2 days) - approximate
        let cny = Self::approximate_chinese_new_year(year);
        cal.add_holiday(Holiday::new("Chinese New Year", cny, 0.02));
        cal.add_holiday(Holiday::new(
            "Chinese New Year (Day 2)",
            cny + Duration::days(1),
            0.02,
        ));

        // Good Friday
        let easter = Self::easter_date(year);
        cal.add_holiday(Holiday::new(
            "Good Friday",
            easter - Duration::days(2),
            0.02,
        ));

        // Labour Day - May 1
        cal.add_holiday(Holiday::new(
            "Labour Day",
            NaiveDate::from_ymd_opt(year, 5, 1).expect("valid date components"),
            0.02,
        ));

        // Vesak Day - approximate (full moon in May)
        let vesak = Self::approximate_vesak(year);
        cal.add_holiday(Holiday::new("Vesak Day", vesak, 0.02));

        // Hari Raya Puasa - approximate (end of Ramadan)
        let hari_raya_puasa = Self::approximate_hari_raya_puasa(year);
        cal.add_holiday(Holiday::new("Hari Raya Puasa", hari_raya_puasa, 0.02));

        // Hari Raya Haji - approximate (Festival of Sacrifice)
        let hari_raya_haji = Self::approximate_hari_raya_haji(year);
        cal.add_holiday(Holiday::new("Hari Raya Haji", hari_raya_haji, 0.02));

        // National Day - August 9
        cal.add_holiday(Holiday::new(
            "National Day",
            NaiveDate::from_ymd_opt(year, 8, 9).expect("valid date components"),
            0.02,
        ));

        // Deepavali - approximate (October/November)
        let deepavali = Self::approximate_deepavali(year);
        cal.add_holiday(Holiday::new("Deepavali", deepavali, 0.02));

        // Christmas Day
        cal.add_holiday(Holiday::new(
            "Christmas Day",
            NaiveDate::from_ymd_opt(year, 12, 25).expect("valid date components"),
            0.02,
        ));

        cal
    }

    /// South Korean holidays (national holidays).
    fn kr_holidays(year: i32) -> Self {
        let mut cal = Self::new(Region::KR, year);

        // New Year's Day - January 1
        cal.add_holiday(Holiday::new(
            "Sinjeong",
            NaiveDate::from_ymd_opt(year, 1, 1).expect("valid date components"),
            0.02,
        ));

        // Seollal (Korean New Year) - 3 days around lunar new year
        let seollal = Self::approximate_korean_new_year(year);
        cal.add_holiday(Holiday::new(
            "Seollal (Eve)",
            seollal - Duration::days(1),
            0.02,
        ));
        cal.add_holiday(Holiday::new("Seollal", seollal, 0.02));
        cal.add_holiday(Holiday::new(
            "Seollal (Day 2)",
            seollal + Duration::days(1),
            0.02,
        ));

        // Independence Movement Day - March 1
        cal.add_holiday(Holiday::new(
            "Samiljeol",
            NaiveDate::from_ymd_opt(year, 3, 1).expect("valid date components"),
            0.02,
        ));

        // Children's Day - May 5
        cal.add_holiday(Holiday::new(
            "Eorininal",
            NaiveDate::from_ymd_opt(year, 5, 5).expect("valid date components"),
            0.02,
        ));

        // Buddha's Birthday - approximate (8th day of 4th lunar month)
        let buddha_birthday = Self::approximate_korean_buddha_birthday(year);
        cal.add_holiday(Holiday::new("Seokgatansinil", buddha_birthday, 0.02));

        // Memorial Day - June 6
        cal.add_holiday(Holiday::new(
            "Hyeonchungil",
            NaiveDate::from_ymd_opt(year, 6, 6).expect("valid date components"),
            0.02,
        ));

        // Liberation Day - August 15
        cal.add_holiday(Holiday::new(
            "Gwangbokjeol",
            NaiveDate::from_ymd_opt(year, 8, 15).expect("valid date components"),
            0.02,
        ));

        // Chuseok (Korean Thanksgiving) - 3 days around harvest moon
        let chuseok = Self::approximate_chuseok(year);
        cal.add_holiday(Holiday::new(
            "Chuseok (Eve)",
            chuseok - Duration::days(1),
            0.02,
        ));
        cal.add_holiday(Holiday::new("Chuseok", chuseok, 0.02));
        cal.add_holiday(Holiday::new(
            "Chuseok (Day 2)",
            chuseok + Duration::days(1),
            0.02,
        ));

        // National Foundation Day - October 3
        cal.add_holiday(Holiday::new(
            "Gaecheonjeol",
            NaiveDate::from_ymd_opt(year, 10, 3).expect("valid date components"),
            0.02,
        ));

        // Hangul Day - October 9
        cal.add_holiday(Holiday::new(
            "Hangullal",
            NaiveDate::from_ymd_opt(year, 10, 9).expect("valid date components"),
            0.02,
        ));

        // Christmas - December 25
        cal.add_holiday(Holiday::new(
            "Seongtanjeol",
            NaiveDate::from_ymd_opt(year, 12, 25).expect("valid date components"),
            0.02,
        ));

        cal
    }

    /// Calculate Easter date using the anonymous Gregorian algorithm.
    fn easter_date(year: i32) -> NaiveDate {
        let a = year % 19;
        let b = year / 100;
        let c = year % 100;
        let d = b / 4;
        let e = b % 4;
        let f = (b + 8) / 25;
        let g = (b - f + 1) / 3;
        let h = (19 * a + b - d - g + 15) % 30;
        let i = c / 4;
        let k = c % 4;
        let l = (32 + 2 * e + 2 * i - h - k) % 7;
        let m = (a + 11 * h + 22 * l) / 451;
        let month = (h + l - 7 * m + 114) / 31;
        let day = ((h + l - 7 * m + 114) % 31) + 1;

        NaiveDate::from_ymd_opt(year, month as u32, day as u32).expect("valid date components")
    }

    /// Get nth weekday of a month (e.g., 3rd Monday of January).
    fn nth_weekday_of_month(year: i32, month: u32, weekday: Weekday, n: u32) -> NaiveDate {
        let first = NaiveDate::from_ymd_opt(year, month, 1).expect("valid date components");
        let first_weekday = first.weekday();

        let days_until = (weekday.num_days_from_monday() as i64
            - first_weekday.num_days_from_monday() as i64
            + 7)
            % 7;

        first + Duration::days(days_until + (n - 1) as i64 * 7)
    }

    /// Get last weekday of a month (e.g., last Monday of May).
    fn last_weekday_of_month(year: i32, month: u32, weekday: Weekday) -> NaiveDate {
        let last = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("valid date components")
                - Duration::days(1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).expect("valid date components")
                - Duration::days(1)
        };

        let last_weekday = last.weekday();
        let days_back = (last_weekday.num_days_from_monday() as i64
            - weekday.num_days_from_monday() as i64
            + 7)
            % 7;

        last - Duration::days(days_back)
    }

    /// Observe weekend holidays on nearest weekday.
    fn observe_weekend(date: NaiveDate) -> NaiveDate {
        match date.weekday() {
            Weekday::Sat => date - Duration::days(1), // Friday
            Weekday::Sun => date + Duration::days(1), // Monday
            _ => date,
        }
    }

    /// Approximate Chinese New Year date (simplified calculation).
    fn approximate_chinese_new_year(year: i32) -> NaiveDate {
        // Chinese New Year falls between Jan 21 and Feb 20
        // This is a simplified approximation
        let base_year = 2000;
        let cny_2000 = NaiveDate::from_ymd_opt(2000, 2, 5).expect("valid date components");

        let years_diff = year - base_year;
        let lunar_cycle = 29.5306; // days per lunar month
        let days_offset = (years_diff as f64 * 12.0 * lunar_cycle) % 365.25;

        let mut result = cny_2000 + Duration::days(days_offset as i64);

        // Ensure it falls in Jan-Feb range
        while result.month() > 2 || (result.month() == 2 && result.day() > 20) {
            result -= Duration::days(29);
        }
        while result.month() < 1 || (result.month() == 1 && result.day() < 21) {
            result += Duration::days(29);
        }

        // Adjust year if needed
        if result.year() != year {
            result = NaiveDate::from_ymd_opt(year, result.month(), result.day().min(28))
                .unwrap_or_else(|| {
                    NaiveDate::from_ymd_opt(year, result.month(), 28)
                        .expect("valid date components")
                });
        }

        result
    }

    /// Approximate Diwali date (simplified calculation).
    fn approximate_diwali(year: i32) -> NaiveDate {
        // Diwali typically falls in October-November
        // This is a simplified approximation
        match year % 4 {
            0 => NaiveDate::from_ymd_opt(year, 11, 1).expect("valid date components"),
            1 => NaiveDate::from_ymd_opt(year, 10, 24).expect("valid date components"),
            2 => NaiveDate::from_ymd_opt(year, 11, 12).expect("valid date components"),
            _ => NaiveDate::from_ymd_opt(year, 11, 4).expect("valid date components"),
        }
    }

    /// Approximate Vesak Day (Buddha's Birthday in Theravada tradition).
    /// Falls on the full moon of the 4th lunar month (usually May).
    fn approximate_vesak(year: i32) -> NaiveDate {
        // Vesak is typically in May
        // Using approximate lunar cycle calculation
        let base = match year % 19 {
            0 => 18,
            1 => 7,
            2 => 26,
            3 => 15,
            4 => 5,
            5 => 24,
            6 => 13,
            7 => 2,
            8 => 22,
            9 => 11,
            10 => 30,
            11 => 19,
            12 => 8,
            13 => 27,
            14 => 17,
            15 => 6,
            16 => 25,
            17 => 14,
            _ => 3,
        };
        let month = if base > 20 { 4 } else { 5 };
        let day = if base > 20 { base - 10 } else { base };
        NaiveDate::from_ymd_opt(year, month, day.clamp(1, 28) as u32)
            .expect("valid date components")
    }

    /// Approximate Hari Raya Puasa (Eid al-Fitr).
    /// Based on Islamic lunar calendar (moves ~11 days earlier each year).
    fn approximate_hari_raya_puasa(year: i32) -> NaiveDate {
        // Islamic calendar moves about 11 days earlier each year
        // Base: 2024 Eid al-Fitr was approximately April 10
        let base_year = 2024;
        let base_date = NaiveDate::from_ymd_opt(2024, 4, 10).expect("valid date components");
        let years_diff = year - base_year;
        let days_shift = (years_diff as f64 * -10.63) as i64;
        let mut result = base_date + Duration::days(days_shift);

        // Wrap around to stay in valid range
        while result.year() != year {
            if result.year() > year {
                result -= Duration::days(354); // Islamic lunar year
            } else {
                result += Duration::days(354);
            }
        }
        result
    }

    /// Approximate Hari Raya Haji (Eid al-Adha).
    /// Approximately 70 days after Hari Raya Puasa.
    fn approximate_hari_raya_haji(year: i32) -> NaiveDate {
        Self::approximate_hari_raya_puasa(year) + Duration::days(70)
    }

    /// Approximate Deepavali date (same as Diwali).
    fn approximate_deepavali(year: i32) -> NaiveDate {
        Self::approximate_diwali(year)
    }

    /// Approximate Korean New Year (Seollal).
    /// Similar to Chinese New Year but may differ by a day.
    fn approximate_korean_new_year(year: i32) -> NaiveDate {
        Self::approximate_chinese_new_year(year)
    }

    /// Approximate Korean Buddha's Birthday.
    /// 8th day of the 4th lunar month.
    fn approximate_korean_buddha_birthday(year: i32) -> NaiveDate {
        // Typically falls in late April to late May
        match year % 19 {
            0 => NaiveDate::from_ymd_opt(year, 5, 15).expect("valid date components"),
            1 => NaiveDate::from_ymd_opt(year, 5, 4).expect("valid date components"),
            2 => NaiveDate::from_ymd_opt(year, 5, 23).expect("valid date components"),
            3 => NaiveDate::from_ymd_opt(year, 5, 12).expect("valid date components"),
            4 => NaiveDate::from_ymd_opt(year, 5, 1).expect("valid date components"),
            5 => NaiveDate::from_ymd_opt(year, 5, 20).expect("valid date components"),
            6 => NaiveDate::from_ymd_opt(year, 5, 10).expect("valid date components"),
            7 => NaiveDate::from_ymd_opt(year, 4, 29).expect("valid date components"),
            8 => NaiveDate::from_ymd_opt(year, 5, 18).expect("valid date components"),
            9 => NaiveDate::from_ymd_opt(year, 5, 7).expect("valid date components"),
            10 => NaiveDate::from_ymd_opt(year, 5, 26).expect("valid date components"),
            11 => NaiveDate::from_ymd_opt(year, 5, 15).expect("valid date components"),
            12 => NaiveDate::from_ymd_opt(year, 5, 4).expect("valid date components"),
            13 => NaiveDate::from_ymd_opt(year, 5, 24).expect("valid date components"),
            14 => NaiveDate::from_ymd_opt(year, 5, 13).expect("valid date components"),
            15 => NaiveDate::from_ymd_opt(year, 5, 2).expect("valid date components"),
            16 => NaiveDate::from_ymd_opt(year, 5, 21).expect("valid date components"),
            17 => NaiveDate::from_ymd_opt(year, 5, 10).expect("valid date components"),
            _ => NaiveDate::from_ymd_opt(year, 4, 30).expect("valid date components"),
        }
    }

    /// Approximate Chuseok (Korean Thanksgiving).
    /// 15th day of the 8th lunar month (harvest moon).
    fn approximate_chuseok(year: i32) -> NaiveDate {
        // Chuseok typically falls in September or early October
        match year % 19 {
            0 => NaiveDate::from_ymd_opt(year, 9, 17).expect("valid date components"),
            1 => NaiveDate::from_ymd_opt(year, 10, 6).expect("valid date components"),
            2 => NaiveDate::from_ymd_opt(year, 9, 25).expect("valid date components"),
            3 => NaiveDate::from_ymd_opt(year, 9, 14).expect("valid date components"),
            4 => NaiveDate::from_ymd_opt(year, 10, 3).expect("valid date components"),
            5 => NaiveDate::from_ymd_opt(year, 9, 22).expect("valid date components"),
            6 => NaiveDate::from_ymd_opt(year, 9, 11).expect("valid date components"),
            7 => NaiveDate::from_ymd_opt(year, 9, 30).expect("valid date components"),
            8 => NaiveDate::from_ymd_opt(year, 9, 19).expect("valid date components"),
            9 => NaiveDate::from_ymd_opt(year, 10, 9).expect("valid date components"),
            10 => NaiveDate::from_ymd_opt(year, 9, 28).expect("valid date components"),
            11 => NaiveDate::from_ymd_opt(year, 9, 17).expect("valid date components"),
            12 => NaiveDate::from_ymd_opt(year, 10, 6).expect("valid date components"),
            13 => NaiveDate::from_ymd_opt(year, 9, 25).expect("valid date components"),
            14 => NaiveDate::from_ymd_opt(year, 9, 14).expect("valid date components"),
            15 => NaiveDate::from_ymd_opt(year, 10, 4).expect("valid date components"),
            16 => NaiveDate::from_ymd_opt(year, 9, 22).expect("valid date components"),
            17 => NaiveDate::from_ymd_opt(year, 9, 12).expect("valid date components"),
            _ => NaiveDate::from_ymd_opt(year, 10, 1).expect("valid date components"),
        }
    }
}

/// Custom holiday configuration for YAML/JSON input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomHolidayConfig {
    /// Holiday name.
    pub name: String,
    /// Month (1-12).
    pub month: u8,
    /// Day of month.
    pub day: u8,
    /// Activity multiplier (optional, defaults to 0.05).
    #[serde(default = "default_holiday_multiplier")]
    pub activity_multiplier: f64,
}

fn default_holiday_multiplier() -> f64 {
    0.05
}

impl CustomHolidayConfig {
    /// Convert to a Holiday for a specific year.
    pub fn to_holiday(&self, year: i32) -> Holiday {
        Holiday::new(
            &self.name,
            NaiveDate::from_ymd_opt(year, self.month as u32, self.day as u32)
                .expect("valid date components"),
            self.activity_multiplier,
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_us_holidays() {
        let cal = HolidayCalendar::for_region(Region::US, 2024);

        // Check some specific holidays exist
        let christmas = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
        assert!(cal.is_holiday(christmas));

        // Independence Day (observed on Friday since July 4 is Thursday in 2024)
        let independence = NaiveDate::from_ymd_opt(2024, 7, 4).unwrap();
        assert!(cal.is_holiday(independence));
    }

    #[test]
    fn test_german_holidays() {
        let cal = HolidayCalendar::for_region(Region::DE, 2024);

        // Tag der Deutschen Einheit - October 3
        let unity = NaiveDate::from_ymd_opt(2024, 10, 3).unwrap();
        assert!(cal.is_holiday(unity));
    }

    #[test]
    fn test_easter_calculation() {
        // Known Easter dates
        assert_eq!(
            HolidayCalendar::easter_date(2024),
            NaiveDate::from_ymd_opt(2024, 3, 31).unwrap()
        );
        assert_eq!(
            HolidayCalendar::easter_date(2025),
            NaiveDate::from_ymd_opt(2025, 4, 20).unwrap()
        );
    }

    #[test]
    fn test_nth_weekday() {
        // 3rd Monday of January 2024
        let mlk = HolidayCalendar::nth_weekday_of_month(2024, 1, Weekday::Mon, 3);
        assert_eq!(mlk, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

        // 4th Thursday of November 2024 (Thanksgiving)
        let thanksgiving = HolidayCalendar::nth_weekday_of_month(2024, 11, Weekday::Thu, 4);
        assert_eq!(thanksgiving, NaiveDate::from_ymd_opt(2024, 11, 28).unwrap());
    }

    #[test]
    fn test_last_weekday() {
        // Last Monday of May 2024 (Memorial Day)
        let memorial = HolidayCalendar::last_weekday_of_month(2024, 5, Weekday::Mon);
        assert_eq!(memorial, NaiveDate::from_ymd_opt(2024, 5, 27).unwrap());
    }

    #[test]
    fn test_activity_multiplier() {
        let cal = HolidayCalendar::for_region(Region::US, 2024);

        // Holiday should have low multiplier
        let christmas = NaiveDate::from_ymd_opt(2024, 12, 25).unwrap();
        assert!(cal.get_multiplier(christmas) < 0.1);

        // Regular day should be 1.0
        let regular = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert!((cal.get_multiplier(regular) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_all_regions_have_holidays() {
        let regions = [
            Region::US,
            Region::DE,
            Region::GB,
            Region::CN,
            Region::JP,
            Region::IN,
            Region::BR,
            Region::MX,
            Region::AU,
            Region::SG,
            Region::KR,
        ];

        for region in regions {
            let cal = HolidayCalendar::for_region(region, 2024);
            assert!(
                !cal.holidays.is_empty(),
                "Region {:?} should have holidays",
                region
            );
        }
    }

    #[test]
    fn test_brazilian_holidays() {
        let cal = HolidayCalendar::for_region(Region::BR, 2024);

        // Independência do Brasil - September 7
        let independence = NaiveDate::from_ymd_opt(2024, 9, 7).unwrap();
        assert!(cal.is_holiday(independence));

        // Tiradentes - April 21
        let tiradentes = NaiveDate::from_ymd_opt(2024, 4, 21).unwrap();
        assert!(cal.is_holiday(tiradentes));
    }

    #[test]
    fn test_mexican_holidays() {
        let cal = HolidayCalendar::for_region(Region::MX, 2024);

        // Día de la Independencia - September 16
        let independence = NaiveDate::from_ymd_opt(2024, 9, 16).unwrap();
        assert!(cal.is_holiday(independence));
    }

    #[test]
    fn test_australian_holidays() {
        let cal = HolidayCalendar::for_region(Region::AU, 2024);

        // ANZAC Day - April 25
        let anzac = NaiveDate::from_ymd_opt(2024, 4, 25).unwrap();
        assert!(cal.is_holiday(anzac));

        // Australia Day - January 26
        let australia_day = NaiveDate::from_ymd_opt(2024, 1, 26).unwrap();
        assert!(cal.is_holiday(australia_day));
    }

    #[test]
    fn test_singapore_holidays() {
        let cal = HolidayCalendar::for_region(Region::SG, 2024);

        // National Day - August 9
        let national = NaiveDate::from_ymd_opt(2024, 8, 9).unwrap();
        assert!(cal.is_holiday(national));
    }

    #[test]
    fn test_korean_holidays() {
        let cal = HolidayCalendar::for_region(Region::KR, 2024);

        // Liberation Day - August 15
        let liberation = NaiveDate::from_ymd_opt(2024, 8, 15).unwrap();
        assert!(cal.is_holiday(liberation));

        // Hangul Day - October 9
        let hangul = NaiveDate::from_ymd_opt(2024, 10, 9).unwrap();
        assert!(cal.is_holiday(hangul));
    }

    #[test]
    fn test_chinese_holidays() {
        let cal = HolidayCalendar::for_region(Region::CN, 2024);

        // National Day - October 1
        let national = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
        assert!(cal.is_holiday(national));
    }

    #[test]
    fn test_japanese_golden_week() {
        let cal = HolidayCalendar::for_region(Region::JP, 2024);

        // Check Golden Week holidays
        let kodomo = NaiveDate::from_ymd_opt(2024, 5, 5).unwrap();
        assert!(cal.is_holiday(kodomo));
    }
}
