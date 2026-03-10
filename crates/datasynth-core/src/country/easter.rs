//! Easter date computation (Anonymous Gregorian algorithm).
//!
//! Extracted from `distributions/holidays.rs` for reuse by the country-pack
//! holiday resolver.

use chrono::NaiveDate;

/// Compute the date of Easter Sunday for a given year using the
/// Anonymous Gregorian algorithm.
///
/// Returns `None` if the computed month/day is somehow invalid (should not
/// happen for years in the range 1583–9999).
pub fn compute_easter(year: i32) -> Option<NaiveDate> {
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

    NaiveDate::from_ymd_opt(year, month as u32, day as u32)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_easter_2024() {
        // Easter 2024 is March 31
        let date = compute_easter(2024).expect("valid date");
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 3, 31).unwrap());
    }

    #[test]
    fn test_easter_2025() {
        // Easter 2025 is April 20
        let date = compute_easter(2025).expect("valid date");
        assert_eq!(date, NaiveDate::from_ymd_opt(2025, 4, 20).unwrap());
    }

    #[test]
    fn test_easter_2000() {
        // Easter 2000 is April 23
        let date = compute_easter(2000).expect("valid date");
        assert_eq!(date, NaiveDate::from_ymd_opt(2000, 4, 23).unwrap());
    }
}
