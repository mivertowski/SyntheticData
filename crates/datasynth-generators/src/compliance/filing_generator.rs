//! Regulatory filing generator.
//!
//! Generates regulatory filing records for each company and jurisdiction,
//! with status progression and deadline tracking.

use chrono::{Datelike, Duration, NaiveDate};
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use datasynth_core::models::compliance::{FilingFrequency, FilingType, RegulatoryFiling};
use datasynth_core::utils::seeded_rng;

/// Configuration for filing generation.
#[derive(Debug, Clone)]
pub struct FilingGeneratorConfig {
    /// Filing types to include (empty = all applicable).
    pub filing_types: Vec<String>,
    /// Whether to generate status progression.
    pub generate_status_progression: bool,
}

impl Default for FilingGeneratorConfig {
    fn default() -> Self {
        Self {
            filing_types: Vec::new(),
            generate_status_progression: true,
        }
    }
}

/// Default filing requirements per jurisdiction.
struct FilingTemplate {
    filing_type: FilingType,
    frequency: FilingFrequency,
    regulator: &'static str,
    jurisdiction: &'static str,
    deadline_days: u32,
}

const FILING_TEMPLATES: &[FilingTemplate] = &[
    FilingTemplate {
        filing_type: FilingType::Form10K,
        frequency: FilingFrequency::Annual,
        regulator: "SEC",
        jurisdiction: "US",
        deadline_days: 60,
    },
    FilingTemplate {
        filing_type: FilingType::Form10Q,
        frequency: FilingFrequency::Quarterly,
        regulator: "SEC",
        jurisdiction: "US",
        deadline_days: 40,
    },
    FilingTemplate {
        filing_type: FilingType::Jahresabschluss,
        frequency: FilingFrequency::Annual,
        regulator: "Bundesanzeiger",
        jurisdiction: "DE",
        deadline_days: 365,
    },
    FilingTemplate {
        filing_type: FilingType::EBilanz,
        frequency: FilingFrequency::Annual,
        regulator: "Finanzamt",
        jurisdiction: "DE",
        deadline_days: 210,
    },
    FilingTemplate {
        filing_type: FilingType::LiasseFiscale,
        frequency: FilingFrequency::Annual,
        regulator: "DGFiP",
        jurisdiction: "FR",
        deadline_days: 120,
    },
    FilingTemplate {
        filing_type: FilingType::UkAnnualReturn,
        frequency: FilingFrequency::Annual,
        regulator: "Companies House",
        jurisdiction: "GB",
        deadline_days: 270,
    },
    FilingTemplate {
        filing_type: FilingType::Ct600,
        frequency: FilingFrequency::Annual,
        regulator: "HMRC",
        jurisdiction: "GB",
        deadline_days: 365,
    },
    FilingTemplate {
        filing_type: FilingType::YukaShokenHokokusho,
        frequency: FilingFrequency::Annual,
        regulator: "FSA",
        jurisdiction: "JP",
        deadline_days: 90,
    },
];

/// Generator for regulatory filing records.
pub struct FilingGenerator {
    rng: ChaCha8Rng,
    config: FilingGeneratorConfig,
}

impl FilingGenerator {
    /// Creates a new generator.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config: FilingGeneratorConfig::default(),
        }
    }

    /// Creates a generator with custom configuration.
    pub fn with_config(seed: u64, config: FilingGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
        }
    }

    /// Generates filing records for companies in specified jurisdictions.
    pub fn generate_filings(
        &mut self,
        company_codes: &[String],
        jurisdictions: &[String],
        start_date: NaiveDate,
        period_months: u32,
    ) -> Vec<RegulatoryFiling> {
        let mut filings = Vec::new();

        for company_code in company_codes {
            for jurisdiction in jurisdictions {
                let templates: Vec<&FilingTemplate> = FILING_TEMPLATES
                    .iter()
                    .filter(|t| t.jurisdiction == jurisdiction)
                    .filter(|t| {
                        self.config.filing_types.is_empty()
                            || self
                                .config
                                .filing_types
                                .iter()
                                .any(|ft| format!("{}", t.filing_type) == *ft)
                    })
                    .collect();

                for template in &templates {
                    let period_ends =
                        self.compute_period_ends(template.frequency, start_date, period_months);

                    for period_end in period_ends {
                        let deadline = period_end + Duration::days(template.deadline_days as i64);

                        let mut filing = RegulatoryFiling::new(
                            template.filing_type.clone(),
                            company_code.as_str(),
                            jurisdiction.as_str(),
                            period_end,
                            deadline,
                            template.regulator,
                        );

                        if self.config.generate_status_progression {
                            // Simulate filing date
                            let days_before_deadline =
                                self.rng.random_range(1i64..template.deadline_days as i64);
                            let filing_date = deadline - Duration::days(days_before_deadline);

                            // Small chance of late filing
                            let filing_date = if self.rng.random::<f64>() < 0.05 {
                                deadline + Duration::days(self.rng.random_range(1i64..30i64))
                            } else {
                                filing_date
                            };

                            filing = filing.filed_on(filing_date);
                            filing.filing_reference = Some(format!(
                                "{}-{}-{}-{}",
                                jurisdiction,
                                company_code,
                                period_end.format("%Y"),
                                template.filing_type
                            ));
                        }

                        filings.push(filing);
                    }
                }
            }
        }

        filings
    }

    fn compute_period_ends(
        &self,
        frequency: FilingFrequency,
        start_date: NaiveDate,
        period_months: u32,
    ) -> Vec<NaiveDate> {
        let mut ends = Vec::new();
        let interval_months: u32 = match frequency {
            FilingFrequency::Annual => 12,
            FilingFrequency::SemiAnnual => 6,
            FilingFrequency::Quarterly => 3,
            FilingFrequency::Monthly => 1,
            FilingFrequency::EventDriven => return ends,
        };

        let mut current_month = start_date.month();
        let mut current_year = start_date.year();
        let mut months_elapsed = 0u32;

        while months_elapsed < period_months {
            // Advance by interval
            months_elapsed += interval_months;
            if months_elapsed > period_months {
                break;
            }

            current_month += interval_months;
            while current_month > 12 {
                current_month -= 12;
                current_year += 1;
            }

            // Period end is last day of the month
            let next_month = if current_month == 12 {
                NaiveDate::from_ymd_opt(current_year + 1, 1, 1)
            } else {
                NaiveDate::from_ymd_opt(current_year, current_month + 1, 1)
            };
            if let Some(nm) = next_month {
                let period_end = nm - Duration::days(1);
                ends.push(period_end);
            }
        }

        ends
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_us_filings() {
        let mut gen = FilingGenerator::new(42);
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let filings = gen.generate_filings(&["C001".to_string()], &["US".to_string()], start, 12);
        // Should have 10-K (1 annual) + 10-Q (4 quarterly, but 3 within 12 months after offset)
        assert!(!filings.is_empty(), "Should generate US filings");

        for f in &filings {
            assert_eq!(f.company_code, "C001");
            assert_eq!(f.jurisdiction, "US");
        }
    }

    #[test]
    fn test_generate_multi_jurisdiction_filings() {
        let mut gen = FilingGenerator::new(42);
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let filings = gen.generate_filings(
            &["C001".to_string()],
            &["US".to_string(), "DE".to_string(), "GB".to_string()],
            start,
            12,
        );
        assert!(!filings.is_empty());

        let jurisdictions: std::collections::HashSet<&str> =
            filings.iter().map(|f| f.jurisdiction.as_str()).collect();
        assert!(jurisdictions.contains("US"));
        assert!(jurisdictions.contains("DE"));
        assert!(jurisdictions.contains("GB"));
    }
}
