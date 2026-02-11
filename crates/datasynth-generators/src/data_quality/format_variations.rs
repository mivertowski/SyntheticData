//! Format variations for data quality simulation.
//!
//! Simulates realistic format inconsistencies including:
//! - Date formats (ISO, US, EU, various separators)
//! - Amount formats (decimal separators, thousand separators, currency symbols)
//! - Identifier formats (padding, prefixes, case variations)
//! - Text formats (case, whitespace, encoding)

use chrono::NaiveDate;
use rand::Rng;
use rust_decimal::Decimal;

/// Date format variations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DateFormat {
    /// ISO 8601: 2024-01-15
    ISO,
    /// US format: 01/15/2024
    US,
    /// US with dashes: 01-15-2024
    USDash,
    /// European: 15/01/2024
    EU,
    /// European with dashes: 15-01-2024
    EUDash,
    /// European with dots: 15.01.2024
    EUDot,
    /// Long format: January 15, 2024
    Long,
    /// Short year: 01/15/24
    ShortYear,
    /// Compact: 20240115
    Compact,
    /// Unix timestamp
    Unix,
    /// Excel serial number
    ExcelSerial,
}

impl DateFormat {
    /// Returns all date formats.
    pub fn all() -> Vec<Self> {
        vec![
            DateFormat::ISO,
            DateFormat::US,
            DateFormat::USDash,
            DateFormat::EU,
            DateFormat::EUDash,
            DateFormat::EUDot,
            DateFormat::Long,
            DateFormat::ShortYear,
            DateFormat::Compact,
        ]
    }

    /// Formats a date using this format.
    pub fn format(&self, date: NaiveDate) -> String {
        match self {
            DateFormat::ISO => date.format("%Y-%m-%d").to_string(),
            DateFormat::US => date.format("%m/%d/%Y").to_string(),
            DateFormat::USDash => date.format("%m-%d-%Y").to_string(),
            DateFormat::EU => date.format("%d/%m/%Y").to_string(),
            DateFormat::EUDash => date.format("%d-%m-%Y").to_string(),
            DateFormat::EUDot => date.format("%d.%m.%Y").to_string(),
            DateFormat::Long => date.format("%B %d, %Y").to_string(),
            DateFormat::ShortYear => date.format("%m/%d/%y").to_string(),
            DateFormat::Compact => date.format("%Y%m%d").to_string(),
            DateFormat::Unix => {
                let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).expect("valid unix epoch date");
                let days = (date - epoch).num_days();
                (days * 86400).to_string()
            }
            DateFormat::ExcelSerial => {
                // Excel epoch is December 30, 1899
                let epoch = NaiveDate::from_ymd_opt(1899, 12, 30).expect("valid excel epoch date");
                let days = (date - epoch).num_days();
                days.to_string()
            }
        }
    }
}

/// Amount format variations.
#[derive(Debug, Clone, PartialEq)]
pub enum AmountFormat {
    /// Plain number: 1234.56
    Plain,
    /// With thousand separator (comma): 1,234.56
    USComma,
    /// European (dot thousand, comma decimal): 1.234,56
    EUFormat,
    /// Space thousand separator: 1 234.56
    SpaceSeparator,
    /// With currency prefix: $1,234.56
    CurrencyPrefix(String),
    /// With currency suffix: 1,234.56 USD
    CurrencySuffix(String),
    /// Accounting format (parentheses for negative): (1,234.56)
    Accounting,
    /// Scientific notation: 1.23456E+03
    Scientific,
    /// No decimal places: 1235
    NoDecimals,
    /// Four decimal places: 1234.5600
    FourDecimals,
}

impl AmountFormat {
    /// Returns common amount formats.
    pub fn common() -> Vec<Self> {
        vec![
            AmountFormat::Plain,
            AmountFormat::USComma,
            AmountFormat::EUFormat,
            AmountFormat::SpaceSeparator,
            AmountFormat::CurrencyPrefix("$".to_string()),
            AmountFormat::CurrencySuffix("USD".to_string()),
            AmountFormat::Accounting,
            AmountFormat::NoDecimals,
        ]
    }

    /// Formats a decimal using this format.
    pub fn format(&self, amount: Decimal) -> String {
        let is_negative = amount < Decimal::ZERO;
        let abs_amount = amount.abs();
        let amount_f64: f64 = abs_amount.try_into().unwrap_or(0.0);

        match self {
            AmountFormat::Plain => {
                if is_negative {
                    format!("-{:.2}", amount_f64)
                } else {
                    format!("{:.2}", amount_f64)
                }
            }
            AmountFormat::USComma => {
                let formatted = format_with_thousands(amount_f64, ',', '.');
                if is_negative {
                    format!("-{}", formatted)
                } else {
                    formatted
                }
            }
            AmountFormat::EUFormat => {
                let formatted = format_with_thousands(amount_f64, '.', ',');
                if is_negative {
                    format!("-{}", formatted)
                } else {
                    formatted
                }
            }
            AmountFormat::SpaceSeparator => {
                let formatted = format_with_thousands(amount_f64, ' ', '.');
                if is_negative {
                    format!("-{}", formatted)
                } else {
                    formatted
                }
            }
            AmountFormat::CurrencyPrefix(symbol) => {
                let formatted = format_with_thousands(amount_f64, ',', '.');
                if is_negative {
                    format!("-{}{}", symbol, formatted)
                } else {
                    format!("{}{}", symbol, formatted)
                }
            }
            AmountFormat::CurrencySuffix(code) => {
                let formatted = format_with_thousands(amount_f64, ',', '.');
                if is_negative {
                    format!("-{} {}", formatted, code)
                } else {
                    format!("{} {}", formatted, code)
                }
            }
            AmountFormat::Accounting => {
                let formatted = format_with_thousands(amount_f64, ',', '.');
                if is_negative {
                    format!("({})", formatted)
                } else {
                    formatted
                }
            }
            AmountFormat::Scientific => {
                if is_negative {
                    format!("-{:.5E}", amount_f64)
                } else {
                    format!("{:.5E}", amount_f64)
                }
            }
            AmountFormat::NoDecimals => {
                let rounded = amount_f64.round() as i64;
                if is_negative {
                    format!("-{}", rounded.abs())
                } else {
                    rounded.to_string()
                }
            }
            AmountFormat::FourDecimals => {
                if is_negative {
                    format!("-{:.4}", amount_f64)
                } else {
                    format!("{:.4}", amount_f64)
                }
            }
        }
    }
}

/// Formats a number with thousand separators.
fn format_with_thousands(value: f64, thousand_sep: char, decimal_sep: char) -> String {
    let integer_part = value.trunc() as i64;
    let decimal_part = ((value.fract() * 100.0).round() as i64).abs();

    let integer_str = integer_part.abs().to_string();
    let mut result = String::new();

    for (i, c) in integer_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(thousand_sep);
        }
        result.push(c);
    }

    let integer_formatted: String = result.chars().rev().collect();
    format!("{}{}{:02}", integer_formatted, decimal_sep, decimal_part)
}

/// Identifier format variations.
#[derive(Debug, Clone)]
pub enum IdentifierFormat {
    /// Original case.
    Original,
    /// Uppercase.
    Upper,
    /// Lowercase.
    Lower,
    /// With prefix.
    WithPrefix(String),
    /// With suffix.
    WithSuffix(String),
    /// Zero-padded to length.
    ZeroPadded(usize),
    /// Space-padded to length.
    SpacePadded(usize),
    /// With separator.
    WithSeparator { separator: char, interval: usize },
}

impl IdentifierFormat {
    /// Formats an identifier using this format.
    pub fn format(&self, id: &str) -> String {
        match self {
            IdentifierFormat::Original => id.to_string(),
            IdentifierFormat::Upper => id.to_uppercase(),
            IdentifierFormat::Lower => id.to_lowercase(),
            IdentifierFormat::WithPrefix(prefix) => format!("{}{}", prefix, id),
            IdentifierFormat::WithSuffix(suffix) => format!("{}{}", id, suffix),
            IdentifierFormat::ZeroPadded(len) => {
                if id.len() >= *len {
                    id.to_string()
                } else {
                    format!("{:0>width$}", id, width = len)
                }
            }
            IdentifierFormat::SpacePadded(len) => {
                if id.len() >= *len {
                    id.to_string()
                } else {
                    format!("{:>width$}", id, width = len)
                }
            }
            IdentifierFormat::WithSeparator {
                separator,
                interval,
            } => {
                let mut result = String::new();
                for (i, c) in id.chars().enumerate() {
                    if i > 0 && i % interval == 0 {
                        result.push(*separator);
                    }
                    result.push(c);
                }
                result
            }
        }
    }
}

/// Text format variations.
#[derive(Debug, Clone)]
pub enum TextFormat {
    /// Original text.
    Original,
    /// Uppercase.
    Upper,
    /// Lowercase.
    Lower,
    /// Title case.
    Title,
    /// With leading whitespace.
    LeadingWhitespace(usize),
    /// With trailing whitespace.
    TrailingWhitespace(usize),
    /// With extra internal spaces.
    ExtraSpaces,
    /// Trimmed.
    Trimmed,
    /// With non-breaking spaces.
    NonBreakingSpaces,
}

impl TextFormat {
    /// Formats text using this format.
    pub fn format(&self, text: &str) -> String {
        match self {
            TextFormat::Original => text.to_string(),
            TextFormat::Upper => text.to_uppercase(),
            TextFormat::Lower => text.to_lowercase(),
            TextFormat::Title => text
                .split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => {
                            first.to_uppercase().to_string()
                                + chars.as_str().to_lowercase().as_str()
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join(" "),
            TextFormat::LeadingWhitespace(n) => {
                format!("{}{}", " ".repeat(*n), text)
            }
            TextFormat::TrailingWhitespace(n) => {
                format!("{}{}", text, " ".repeat(*n))
            }
            TextFormat::ExtraSpaces => text.split_whitespace().collect::<Vec<_>>().join("  "),
            TextFormat::Trimmed => text.trim().to_string(),
            TextFormat::NonBreakingSpaces => text.replace(' ', "\u{00A0}"),
        }
    }
}

/// Configuration for format variations.
#[derive(Debug, Clone)]
pub struct FormatVariationConfig {
    /// Probability of applying date format variation.
    pub date_variation_rate: f64,
    /// Probability of applying amount format variation.
    pub amount_variation_rate: f64,
    /// Probability of applying identifier format variation.
    pub identifier_variation_rate: f64,
    /// Probability of applying text format variation.
    pub text_variation_rate: f64,
    /// Allowed date formats.
    pub allowed_date_formats: Vec<DateFormat>,
    /// Allowed amount formats.
    pub allowed_amount_formats: Vec<AmountFormat>,
}

impl Default for FormatVariationConfig {
    fn default() -> Self {
        Self {
            date_variation_rate: 0.05,
            amount_variation_rate: 0.03,
            identifier_variation_rate: 0.02,
            text_variation_rate: 0.05,
            allowed_date_formats: DateFormat::all(),
            allowed_amount_formats: AmountFormat::common(),
        }
    }
}

/// Format variation injector.
pub struct FormatVariationInjector {
    config: FormatVariationConfig,
    stats: FormatVariationStats,
}

/// Statistics for format variations.
#[derive(Debug, Clone, Default)]
pub struct FormatVariationStats {
    pub date_variations: usize,
    pub amount_variations: usize,
    pub identifier_variations: usize,
    pub text_variations: usize,
    pub total_processed: usize,
}

impl FormatVariationInjector {
    /// Creates a new format variation injector.
    pub fn new(config: FormatVariationConfig) -> Self {
        Self {
            config,
            stats: FormatVariationStats::default(),
        }
    }

    /// Potentially applies a date format variation.
    pub fn vary_date<R: Rng>(&mut self, date: NaiveDate, rng: &mut R) -> String {
        self.stats.total_processed += 1;

        if rng.gen::<f64>() < self.config.date_variation_rate {
            self.stats.date_variations += 1;
            let format = &self.config.allowed_date_formats
                [rng.gen_range(0..self.config.allowed_date_formats.len())];
            format.format(date)
        } else {
            DateFormat::ISO.format(date)
        }
    }

    /// Potentially applies an amount format variation.
    pub fn vary_amount<R: Rng>(&mut self, amount: Decimal, rng: &mut R) -> String {
        self.stats.total_processed += 1;

        if rng.gen::<f64>() < self.config.amount_variation_rate {
            self.stats.amount_variations += 1;
            let format = &self.config.allowed_amount_formats
                [rng.gen_range(0..self.config.allowed_amount_formats.len())];
            format.format(amount)
        } else {
            AmountFormat::Plain.format(amount)
        }
    }

    /// Potentially applies an identifier format variation.
    pub fn vary_identifier<R: Rng>(&mut self, id: &str, rng: &mut R) -> String {
        self.stats.total_processed += 1;

        if rng.gen::<f64>() < self.config.identifier_variation_rate {
            self.stats.identifier_variations += 1;

            let variations = [
                IdentifierFormat::Upper,
                IdentifierFormat::Lower,
                IdentifierFormat::ZeroPadded(10),
                IdentifierFormat::WithPrefix(" ".to_string()),
                IdentifierFormat::WithSuffix(" ".to_string()),
            ];

            let format = &variations[rng.gen_range(0..variations.len())];
            format.format(id)
        } else {
            id.to_string()
        }
    }

    /// Potentially applies a text format variation.
    pub fn vary_text<R: Rng>(&mut self, text: &str, rng: &mut R) -> String {
        self.stats.total_processed += 1;

        if rng.gen::<f64>() < self.config.text_variation_rate {
            self.stats.text_variations += 1;

            let variations = [
                TextFormat::Upper,
                TextFormat::Lower,
                TextFormat::Title,
                TextFormat::LeadingWhitespace(1),
                TextFormat::TrailingWhitespace(1),
                TextFormat::ExtraSpaces,
            ];

            let format = &variations[rng.gen_range(0..variations.len())];
            format.format(text)
        } else {
            text.to_string()
        }
    }

    /// Returns statistics.
    pub fn stats(&self) -> &FormatVariationStats {
        &self.stats
    }

    /// Resets statistics.
    pub fn reset_stats(&mut self) {
        self.stats = FormatVariationStats::default();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_date_formats() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        assert_eq!(DateFormat::ISO.format(date), "2024-01-15");
        assert_eq!(DateFormat::US.format(date), "01/15/2024");
        assert_eq!(DateFormat::EU.format(date), "15/01/2024");
        assert_eq!(DateFormat::Compact.format(date), "20240115");
    }

    #[test]
    fn test_amount_formats() {
        let amount = dec!(1234567.89);

        assert_eq!(AmountFormat::Plain.format(amount), "1234567.89");
        assert_eq!(AmountFormat::USComma.format(amount), "1,234,567.89");
        assert_eq!(AmountFormat::EUFormat.format(amount), "1.234.567,89");
        assert_eq!(AmountFormat::NoDecimals.format(amount), "1234568");
    }

    #[test]
    fn test_negative_amounts() {
        let amount = dec!(-1234.56);

        assert_eq!(AmountFormat::Plain.format(amount), "-1234.56");
        assert_eq!(AmountFormat::Accounting.format(amount), "(1,234.56)");
    }

    #[test]
    fn test_identifier_formats() {
        let id = "abc123";

        assert_eq!(IdentifierFormat::Upper.format(id), "ABC123");
        assert_eq!(IdentifierFormat::ZeroPadded(10).format(id), "0000abc123");
    }

    #[test]
    fn test_text_formats() {
        let text = "hello world";

        assert_eq!(TextFormat::Upper.format(text), "HELLO WORLD");
        assert_eq!(TextFormat::Title.format(text), "Hello World");
        assert_eq!(TextFormat::ExtraSpaces.format(text), "hello  world");
    }

    #[test]
    fn test_format_injector() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        let config = FormatVariationConfig {
            date_variation_rate: 1.0, // Always vary for testing
            ..Default::default()
        };

        let mut injector = FormatVariationInjector::new(config);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let formatted = injector.vary_date(date, &mut rng);

        // Formatted date should not be empty and stats should be updated
        assert!(!formatted.is_empty());
        assert_eq!(injector.stats().date_variations, 1);
    }
}
