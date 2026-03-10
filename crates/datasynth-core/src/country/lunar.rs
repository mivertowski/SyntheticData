//! Lunar and Islamic calendar approximation functions.
//!
//! Extracted from `distributions/holidays.rs` for reuse by the country-pack
//! holiday resolver. Each function returns `Option<NaiveDate>` instead of
//! panicking on invalid dates.

use chrono::{Datelike, Duration, NaiveDate};

/// Dispatch a lunar holiday algorithm by name.
///
/// Returns a vector of dates (one per day of the holiday) or `None` if the
/// algorithm name is unrecognised.
pub fn resolve_lunar_holiday(
    algorithm: &str,
    year: i32,
    duration_days: u32,
) -> Option<Vec<NaiveDate>> {
    let base = match algorithm {
        "chinese_new_year" => approximate_chinese_new_year(year),
        "diwali" => approximate_diwali(year),
        "vesak" => approximate_vesak(year),
        "hari_raya_puasa" => approximate_hari_raya_puasa(year),
        "hari_raya_haji" => approximate_hari_raya_haji(year),
        "deepavali" => approximate_deepavali(year),
        "korean_new_year" => approximate_korean_new_year(year),
        "korean_buddha_birthday" => approximate_korean_buddha_birthday(year),
        "chuseok" => approximate_chuseok(year),
        _ => return None,
    }?;

    let mut dates = Vec::with_capacity(duration_days as usize);
    for offset in 0..duration_days {
        dates.push(base + Duration::days(offset as i64));
    }
    Some(dates)
}

/// Approximate Chinese New Year date (simplified calculation).
pub fn approximate_chinese_new_year(year: i32) -> Option<NaiveDate> {
    let base_year = 2000;
    let cny_2000 = NaiveDate::from_ymd_opt(2000, 2, 5)?;

    let years_diff = year - base_year;
    let lunar_cycle = 29.5306_f64;
    let days_offset = (years_diff as f64 * 12.0 * lunar_cycle) % 365.25;

    let mut result = cny_2000 + Duration::days(days_offset as i64);

    // Ensure it falls in Jan-Feb range
    while result.month() > 2 || (result.month() == 2 && result.day() > 20) {
        result -= Duration::days(29);
    }
    while result.month() == 1 && result.day() < 21 {
        result += Duration::days(29);
    }

    // Adjust year if needed
    if result.year() != year {
        let day = result.day().min(28);
        result = NaiveDate::from_ymd_opt(year, result.month(), day)
            .or_else(|| NaiveDate::from_ymd_opt(year, result.month(), 28))?;
    }

    Some(result)
}

/// Approximate Diwali date (simplified calculation).
pub fn approximate_diwali(year: i32) -> Option<NaiveDate> {
    match year % 4 {
        0 => NaiveDate::from_ymd_opt(year, 11, 1),
        1 => NaiveDate::from_ymd_opt(year, 10, 24),
        2 => NaiveDate::from_ymd_opt(year, 11, 12),
        _ => NaiveDate::from_ymd_opt(year, 11, 4),
    }
}

/// Approximate Vesak Day (Buddha's Birthday in Theravada tradition).
pub fn approximate_vesak(year: i32) -> Option<NaiveDate> {
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
    NaiveDate::from_ymd_opt(year, month, (day as u32).clamp(1, 28))
}

/// Approximate Hari Raya Puasa (Eid al-Fitr).
pub fn approximate_hari_raya_puasa(year: i32) -> Option<NaiveDate> {
    let base_year = 2024;
    let base_date = NaiveDate::from_ymd_opt(2024, 4, 10)?;
    let years_diff = year - base_year;
    let days_shift = (years_diff as f64 * -10.63) as i64;
    let mut result = base_date + Duration::days(days_shift);

    while result.year() != year {
        if result.year() > year {
            result -= Duration::days(354);
        } else {
            result += Duration::days(354);
        }
    }
    Some(result)
}

/// Approximate Hari Raya Haji (Eid al-Adha) — ~70 days after Puasa.
pub fn approximate_hari_raya_haji(year: i32) -> Option<NaiveDate> {
    approximate_hari_raya_puasa(year).map(|d| d + Duration::days(70))
}

/// Approximate Deepavali date (same as Diwali).
pub fn approximate_deepavali(year: i32) -> Option<NaiveDate> {
    approximate_diwali(year)
}

/// Approximate Korean New Year (Seollal) — similar to Chinese New Year.
pub fn approximate_korean_new_year(year: i32) -> Option<NaiveDate> {
    approximate_chinese_new_year(year)
}

/// Approximate Korean Buddha's Birthday (8th day of the 4th lunar month).
pub fn approximate_korean_buddha_birthday(year: i32) -> Option<NaiveDate> {
    match year % 19 {
        0 => NaiveDate::from_ymd_opt(year, 5, 15),
        1 => NaiveDate::from_ymd_opt(year, 5, 4),
        2 => NaiveDate::from_ymd_opt(year, 5, 23),
        3 => NaiveDate::from_ymd_opt(year, 5, 12),
        4 => NaiveDate::from_ymd_opt(year, 5, 1),
        5 => NaiveDate::from_ymd_opt(year, 5, 20),
        6 => NaiveDate::from_ymd_opt(year, 5, 10),
        7 => NaiveDate::from_ymd_opt(year, 4, 29),
        8 => NaiveDate::from_ymd_opt(year, 5, 18),
        9 => NaiveDate::from_ymd_opt(year, 5, 7),
        10 => NaiveDate::from_ymd_opt(year, 5, 26),
        11 => NaiveDate::from_ymd_opt(year, 5, 15),
        12 => NaiveDate::from_ymd_opt(year, 5, 4),
        13 => NaiveDate::from_ymd_opt(year, 5, 24),
        14 => NaiveDate::from_ymd_opt(year, 5, 13),
        15 => NaiveDate::from_ymd_opt(year, 5, 2),
        16 => NaiveDate::from_ymd_opt(year, 5, 21),
        17 => NaiveDate::from_ymd_opt(year, 5, 10),
        _ => NaiveDate::from_ymd_opt(year, 4, 30),
    }
}

/// Approximate Chuseok (Korean Thanksgiving, 15th day of 8th lunar month).
pub fn approximate_chuseok(year: i32) -> Option<NaiveDate> {
    match year % 19 {
        0 => NaiveDate::from_ymd_opt(year, 9, 17),
        1 => NaiveDate::from_ymd_opt(year, 10, 6),
        2 => NaiveDate::from_ymd_opt(year, 9, 25),
        3 => NaiveDate::from_ymd_opt(year, 9, 14),
        4 => NaiveDate::from_ymd_opt(year, 10, 3),
        5 => NaiveDate::from_ymd_opt(year, 9, 22),
        6 => NaiveDate::from_ymd_opt(year, 9, 11),
        7 => NaiveDate::from_ymd_opt(year, 9, 30),
        8 => NaiveDate::from_ymd_opt(year, 9, 19),
        9 => NaiveDate::from_ymd_opt(year, 10, 9),
        10 => NaiveDate::from_ymd_opt(year, 9, 28),
        11 => NaiveDate::from_ymd_opt(year, 9, 17),
        12 => NaiveDate::from_ymd_opt(year, 10, 6),
        13 => NaiveDate::from_ymd_opt(year, 9, 25),
        14 => NaiveDate::from_ymd_opt(year, 9, 14),
        15 => NaiveDate::from_ymd_opt(year, 10, 4),
        16 => NaiveDate::from_ymd_opt(year, 9, 22),
        17 => NaiveDate::from_ymd_opt(year, 9, 12),
        _ => NaiveDate::from_ymd_opt(year, 10, 1),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_chinese_new_year() {
        let dates = resolve_lunar_holiday("chinese_new_year", 2024, 1);
        assert!(dates.is_some());
        let dates = dates.unwrap();
        assert_eq!(dates.len(), 1);
        let d = dates[0];
        assert_eq!(d.year(), 2024);
        // Should be in Jan-Feb range
        assert!(d.month() <= 2);
    }

    #[test]
    fn test_resolve_multi_day() {
        let dates = resolve_lunar_holiday("chinese_new_year", 2024, 7);
        assert!(dates.is_some());
        assert_eq!(dates.unwrap().len(), 7);
    }

    #[test]
    fn test_resolve_unknown_algorithm() {
        assert!(resolve_lunar_holiday("unknown_algo", 2024, 1).is_none());
    }

    #[test]
    fn test_diwali_range() {
        for year in 2020..2030 {
            let d = approximate_diwali(year).expect("valid date");
            assert!(d.month() == 10 || d.month() == 11);
        }
    }

    #[test]
    fn test_vesak_range() {
        for year in 2020..2030 {
            let d = approximate_vesak(year).expect("valid date");
            assert!(d.month() == 4 || d.month() == 5);
        }
    }

    #[test]
    fn test_hari_raya_haji_after_puasa() {
        let puasa = approximate_hari_raya_puasa(2024).expect("valid");
        let haji = approximate_hari_raya_haji(2024).expect("valid");
        assert_eq!(haji - puasa, Duration::days(70));
    }

    #[test]
    fn test_chuseok_range() {
        for year in 2020..2030 {
            let d = approximate_chuseok(year).expect("valid date");
            assert!(d.month() == 9 || d.month() == 10);
        }
    }
}
