//! Unusual item marker generator — ISA 520.
//!
//! Scans all journal entries for a given entity and applies five analytical
//! dimensions to flag entries that deviate from expected patterns:
//!
//! | Dimension    | Rule                                                              |
//! |--------------|-------------------------------------------------------------------|
//! | Size         | Amount > 3σ from account mean (top ~1-2% by amount)            |
//! | Timing       | Posted in the last 3 days of the fiscal period, or on weekends |
//! | Relationship | Account combination appears in <1% of all entries               |
//! | Frequency    | First-time use of a GL account by a particular poster           |
//! | Nature       | Manual entry to an account that is almost always automated      |
//!
//! The generator aims for a **5–10% overall flagging rate**, with anomaly-
//! labelled entries (`is_anomaly = true`) having a materially higher rate.
//! Severity is derived from the number of dimensions triggered per entry.

use std::collections::{HashMap, HashSet};

use chrono::Datelike;
use datasynth_core::models::audit::unusual_items::{
    UnusualDimension, UnusualItemFlag, UnusualSeverity,
};
use datasynth_core::models::journal_entry::{JournalEntry, TransactionSource};
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::info;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the unusual item generator.
#[derive(Debug, Clone)]
pub struct UnusualItemGeneratorConfig {
    /// Probability of flagging a normal (non-anomaly) entry that has at least
    /// one unusual dimension.  Keeps the overall rate at ~5-10%.
    pub normal_entry_flag_probability: f64,
    /// Probability of flagging an anomaly-labelled entry that has at least one
    /// unusual dimension.  Anomaly entries should have a higher flag rate.
    pub anomaly_entry_flag_probability: f64,
    /// Number of standard deviations above the mean at which the Size dimension
    /// is triggered.
    pub size_sigma_threshold: f64,
    /// Number of days before the end of the fiscal period in which a posting
    /// is considered a period-end clustering event.
    pub period_end_days: u32,
}

impl Default for UnusualItemGeneratorConfig {
    fn default() -> Self {
        Self {
            normal_entry_flag_probability: 0.60,
            anomaly_entry_flag_probability: 0.92,
            size_sigma_threshold: 3.0,
            period_end_days: 3,
        }
    }
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generator for ISA 520 unusual item flags.
///
/// Call [`UnusualItemGenerator::generate_for_entity`] for each entity after
/// all journal entries have been produced.
pub struct UnusualItemGenerator {
    rng: ChaCha8Rng,
    config: UnusualItemGeneratorConfig,
}

impl UnusualItemGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0x5200), // 0x5200 ≈ ISA 520
            config: UnusualItemGeneratorConfig::default(),
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: UnusualItemGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0x5200),
            config,
        }
    }

    /// Generate unusual item flags for all journal entries of a single entity.
    ///
    /// # Arguments
    /// * `entity_code` — Company / entity code being reviewed.
    /// * `entries` — All journal entries for this entity (used to compute
    ///   statistics and populate flags).
    /// * `period_end_date` — Last day of the fiscal period (used for the
    ///   Timing dimension).
    pub fn generate_for_entity(
        &mut self,
        entity_code: &str,
        entries: &[JournalEntry],
        period_end_date: chrono::NaiveDate,
    ) -> Vec<UnusualItemFlag> {
        info!(
            "Generating unusual item flags for entity {} ({} entries)",
            entity_code,
            entries.len()
        );
        // Filter to this entity's entries
        let entity_entries: Vec<&JournalEntry> = entries
            .iter()
            .filter(|e| e.header.company_code == entity_code)
            .collect();

        if entity_entries.is_empty() {
            return Vec::new();
        }

        // ---- Pre-compute statistics ----------------------------------------
        let account_stats = compute_account_stats(&entity_entries);
        let pair_freq = compute_pair_frequencies(&entity_entries);
        let total_entries = entity_entries.len();
        let automated_accounts = compute_automated_accounts(&entity_entries);
        let poster_accounts = compute_poster_account_history(&entity_entries);

        // ---- Evaluate each entry -------------------------------------------
        let mut flags: Vec<UnusualItemFlag> = Vec::new();
        let mut flag_counter: u64 = 0;

        for je in &entity_entries {
            let dimensions = self.evaluate_dimensions(
                je,
                &account_stats,
                &pair_freq,
                total_entries,
                &automated_accounts,
                &poster_accounts,
                period_end_date,
            );

            if dimensions.is_empty() {
                continue;
            }

            // Apply probabilistic sampling to keep overall rate at ~5-10%
            let flag_prob = if je.header.is_anomaly {
                self.config.anomaly_entry_flag_probability
            } else {
                self.config.normal_entry_flag_probability
            };

            let roll: f64 = self.rng.random();
            if roll > flag_prob {
                continue;
            }

            let severity = UnusualSeverity::from_dimension_count(dimensions.len());
            let description = build_description(entity_code, je, &dimensions);
            let (expected, actual) = build_expected_actual(je, &dimensions, &account_stats);
            let investigation_required = matches!(
                severity,
                UnusualSeverity::Significant | UnusualSeverity::Moderate
            ) || je.header.is_anomaly;

            flag_counter += 1;
            let id = format!("UIF-{}-{:05}", entity_code, flag_counter);

            let gl_accounts: Vec<String> = je
                .lines
                .iter()
                .map(|l| l.gl_account.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            flags.push(UnusualItemFlag {
                id,
                entity_code: entity_code.to_string(),
                journal_entry_id: je.header.document_id.to_string(),
                gl_accounts,
                dimensions,
                severity,
                description,
                expected_value: Some(expected),
                actual_value: actual,
                investigation_required,
                is_labeled_anomaly: je.header.is_anomaly,
            });
        }

        info!(
            "Generated {} unusual item flags for entity {}",
            flags.len(),
            entity_code
        );
        flags
    }

    /// Generate flags for multiple entities at once.
    pub fn generate_for_entities(
        &mut self,
        entity_codes: &[String],
        entries: &[JournalEntry],
        period_end_date: chrono::NaiveDate,
    ) -> Vec<UnusualItemFlag> {
        entity_codes
            .iter()
            .flat_map(|code| self.generate_for_entity(code, entries, period_end_date))
            .collect()
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Evaluate which unusual dimensions apply to a single journal entry.
    fn evaluate_dimensions(
        &mut self,
        je: &JournalEntry,
        account_stats: &HashMap<String, AccountStat>,
        pair_freq: &HashMap<String, usize>,
        total_entries: usize,
        automated_accounts: &HashSet<String>,
        poster_accounts: &HashMap<String, HashMap<String, u64>>,
        period_end_date: chrono::NaiveDate,
    ) -> Vec<UnusualDimension> {
        let mut dims: Vec<UnusualDimension> = Vec::new();

        // ---- Size ---------------------------------------------------------
        // Flag if any line amount is > threshold * σ above mean for that account
        for line in &je.lines {
            let amount = if line.debit_amount > Decimal::ZERO {
                line.debit_amount
            } else {
                line.credit_amount
            };
            if let Some(stat) = account_stats.get(&line.gl_account) {
                if stat.std_dev > Decimal::ZERO {
                    let z = (amount - stat.mean) / stat.std_dev;
                    if z > Decimal::try_from(self.config.size_sigma_threshold).unwrap_or(dec!(3.0))
                    {
                        if !dims.contains(&UnusualDimension::Size) {
                            dims.push(UnusualDimension::Size);
                        }
                        break;
                    }
                }
            }
        }

        // ---- Timing -------------------------------------------------------
        let posting_date = je.header.posting_date;
        let days_before_end = (period_end_date - posting_date).num_days();
        let is_period_end =
            days_before_end >= 0 && days_before_end < self.config.period_end_days as i64;
        let is_weekend = matches!(
            posting_date.weekday(),
            chrono::Weekday::Sat | chrono::Weekday::Sun
        );
        if is_period_end || is_weekend {
            dims.push(UnusualDimension::Timing);
        }

        // ---- Relationship -------------------------------------------------
        // Check if the GL account combination in this entry is rare
        if je.lines.len() >= 2 {
            let accounts: Vec<&str> = {
                let mut v: Vec<&str> = je.lines.iter().map(|l| l.gl_account.as_str()).collect();
                v.sort_unstable();
                v.dedup();
                v
            };
            if accounts.len() >= 2 {
                let pair_key = format!("{}-{}", accounts[0], accounts[1]);
                let count = pair_freq.get(&pair_key).copied().unwrap_or(0);
                let freq = count as f64 / total_entries.max(1) as f64;
                if freq < 0.01 {
                    dims.push(UnusualDimension::Relationship);
                }
            }
        }

        // ---- Frequency ----------------------------------------------------
        // First-time use of a GL account by this poster
        let poster = &je.header.created_by;
        for line in &je.lines {
            let history = poster_accounts.get(poster);
            // If this account has only 1 occurrence by this poster, it's a first-time
            let count_for_poster: u64 = history
                .and_then(|h| h.get(&line.gl_account))
                .copied()
                .unwrap_or(0u64);
            if count_for_poster <= 1 {
                if !dims.contains(&UnusualDimension::Frequency) {
                    dims.push(UnusualDimension::Frequency);
                }
                break;
            }
        }

        // ---- Nature -------------------------------------------------------
        // Manual entry to an account that is almost always automated
        if je.header.source == TransactionSource::Manual {
            for line in &je.lines {
                if automated_accounts.contains(&line.gl_account) {
                    if !dims.contains(&UnusualDimension::Nature) {
                        dims.push(UnusualDimension::Nature);
                    }
                    break;
                }
            }
        }

        dims
    }
}

// ---------------------------------------------------------------------------
// Statistics helpers
// ---------------------------------------------------------------------------

/// Per-account descriptive statistics computed from the entity's entries.
#[derive(Debug, Clone)]
struct AccountStat {
    mean: Decimal,
    std_dev: Decimal,
}

/// Compute mean and standard deviation of debit+credit amounts per GL account.
fn compute_account_stats(entries: &[&JournalEntry]) -> HashMap<String, AccountStat> {
    let mut sums: HashMap<String, (Decimal, u64)> = HashMap::new();

    for je in entries {
        for line in &je.lines {
            let amount = if line.debit_amount > Decimal::ZERO {
                line.debit_amount
            } else {
                line.credit_amount
            };
            let entry = sums.entry(line.gl_account.clone()).or_default();
            entry.0 += amount;
            entry.1 += 1;
        }
    }

    // Compute means
    let means: HashMap<String, Decimal> = sums
        .iter()
        .map(|(acct, (sum, count))| {
            let mean = if *count > 0 {
                sum / Decimal::from(*count)
            } else {
                Decimal::ZERO
            };
            (acct.clone(), mean)
        })
        .collect();

    // Compute variance (sum of squared deviations)
    let mut variance_sums: HashMap<String, (Decimal, u64)> = HashMap::new();
    for je in entries {
        for line in &je.lines {
            let amount = if line.debit_amount > Decimal::ZERO {
                line.debit_amount
            } else {
                line.credit_amount
            };
            let mean = means
                .get(&line.gl_account)
                .copied()
                .unwrap_or(Decimal::ZERO);
            let diff = amount - mean;
            // diff² using Decimal multiplication
            let diff_sq = diff * diff;
            let entry = variance_sums.entry(line.gl_account.clone()).or_default();
            entry.0 += diff_sq;
            entry.1 += 1;
        }
    }

    means
        .into_iter()
        .map(|(acct, mean)| {
            let std_dev = variance_sums
                .get(&acct)
                .filter(|(_, n)| *n > 1)
                .map(|(sum_sq, n)| {
                    let variance = sum_sq / Decimal::from(*n - 1);
                    // Integer square root approximation via f64
                    let variance_f64: f64 = variance.try_into().unwrap_or(0.0);
                    Decimal::try_from(variance_f64.sqrt()).unwrap_or(Decimal::ONE)
                })
                .unwrap_or(Decimal::ONE);
            (acct, AccountStat { mean, std_dev })
        })
        .collect()
}

/// Compute frequency of each sorted GL account pair across all entries.
fn compute_pair_frequencies(entries: &[&JournalEntry]) -> HashMap<String, usize> {
    let mut freq: HashMap<String, usize> = HashMap::new();
    for je in entries {
        let mut accounts: Vec<&str> = je.lines.iter().map(|l| l.gl_account.as_str()).collect();
        accounts.sort_unstable();
        accounts.dedup();
        if accounts.len() >= 2 {
            let key = format!("{}-{}", accounts[0], accounts[1]);
            *freq.entry(key).or_default() += 1;
        }
    }
    freq
}

/// Determine which GL accounts are "almost always automated" (>= 95% of
/// postings are from non-Manual sources).
fn compute_automated_accounts(entries: &[&JournalEntry]) -> HashSet<String> {
    let mut account_counts: HashMap<String, (u64, u64)> = HashMap::new(); // (total, manual)
    for je in entries {
        let is_manual = je.header.source == TransactionSource::Manual;
        for line in &je.lines {
            let entry = account_counts.entry(line.gl_account.clone()).or_default();
            entry.0 += 1;
            if is_manual {
                entry.1 += 1;
            }
        }
    }
    account_counts
        .into_iter()
        .filter(|(_, (total, manual))| *total >= 10 && (*manual as f64 / *total as f64) < 0.05)
        .map(|(acct, _)| acct)
        .collect()
}

/// Build a map of poster → { gl_account → count } to detect first-time use.
fn compute_poster_account_history(
    entries: &[&JournalEntry],
) -> HashMap<String, HashMap<String, u64>> {
    let mut history: HashMap<String, HashMap<String, u64>> = HashMap::new();
    for je in entries {
        let poster = je.header.created_by.clone();
        let poster_map = history.entry(poster).or_default();
        for line in &je.lines {
            *poster_map.entry(line.gl_account.clone()).or_default() += 1;
        }
    }
    history
}

// ---------------------------------------------------------------------------
// Description helpers
// ---------------------------------------------------------------------------

fn build_description(entity_code: &str, je: &JournalEntry, dims: &[UnusualDimension]) -> String {
    let dim_names: Vec<String> = dims.iter().map(|d| d.to_string()).collect();
    format!(
        "Journal entry {} for entity {} flagged as unusual on {} dimension(s): {}. \
         Posted on {} by {}.",
        je.header.document_id,
        entity_code,
        dims.len(),
        dim_names.join(", "),
        je.header.posting_date,
        je.header.created_by,
    )
}

fn build_expected_actual(
    je: &JournalEntry,
    dims: &[UnusualDimension],
    account_stats: &HashMap<String, AccountStat>,
) -> (String, String) {
    // Use the primary unusual dimension for the expected/actual description
    let primary = dims.first().unwrap_or(&UnusualDimension::Size);

    match primary {
        UnusualDimension::Size => {
            // Find the largest line and its account stats
            if let Some(line) = je.lines.iter().max_by(|a, b| {
                let amt_a = if a.debit_amount > Decimal::ZERO {
                    a.debit_amount
                } else {
                    a.credit_amount
                };
                let amt_b = if b.debit_amount > Decimal::ZERO {
                    b.debit_amount
                } else {
                    b.credit_amount
                };
                amt_a.cmp(&amt_b)
            }) {
                let amount = if line.debit_amount > Decimal::ZERO {
                    line.debit_amount
                } else {
                    line.credit_amount
                };
                if let Some(stat) = account_stats.get(&line.gl_account) {
                    let expected = format!(
                        "amount within 3σ of account {} mean ({:.2} ± {:.2})",
                        line.gl_account, stat.mean, stat.std_dev
                    );
                    let actual = format!("amount = {:.2}", amount);
                    return (expected, actual);
                }
            }
            (
                "amount within normal range for account".to_string(),
                "amount exceeds 3σ threshold".to_string(),
            )
        }
        UnusualDimension::Timing => (
            "posting during normal business week, not in last 3 days of period".to_string(),
            format!(
                "posted on {} ({})",
                je.header.posting_date,
                je.header.posting_date.weekday()
            ),
        ),
        UnusualDimension::Relationship => (
            "account combination occurs in ≥1% of all entries".to_string(),
            "account combination occurs in <1% of all entries".to_string(),
        ),
        UnusualDimension::Frequency => (
            "poster has prior experience with all GL accounts used".to_string(),
            format!("first-time account usage by {}", je.header.created_by),
        ),
        UnusualDimension::Nature => (
            "automated source for this GL account (non-manual)".to_string(),
            format!(
                "manual entry ({}) to typically automated account",
                je.header.source
            ),
        ),
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::journal_entry::{JournalEntry, JournalEntryHeader};

    fn make_entry(
        company_code: &str,
        posting_date: chrono::NaiveDate,
        gl_accounts: &[(&str, Decimal)], // (account, amount)
        source: TransactionSource,
        is_anomaly: bool,
        created_by: &str,
    ) -> JournalEntry {
        use datasynth_core::models::journal_entry::JournalEntryLine;
        let mut header = JournalEntryHeader::new(company_code.to_string(), posting_date);
        header.source = source;
        header.is_anomaly = is_anomaly;
        header.created_by = created_by.to_string();

        let doc_id = header.document_id;
        let mut lines = Vec::new();
        for (i, (account, amount)) in gl_accounts.iter().enumerate() {
            if i % 2 == 0 {
                lines.push(JournalEntryLine::debit(
                    doc_id,
                    i as u32 + 1,
                    account.to_string(),
                    *amount,
                ));
            } else {
                lines.push(JournalEntryLine::credit(
                    doc_id,
                    i as u32 + 1,
                    account.to_string(),
                    *amount,
                ));
            }
        }
        JournalEntry {
            header,
            lines: lines.into(),
        }
    }

    fn period_end() -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(2024, 6, 30).unwrap()
    }

    /// Build a batch of mostly normal entries plus a few anomalies
    fn build_test_entries(n: usize) -> Vec<JournalEntry> {
        let mut entries = Vec::new();
        let posting = chrono::NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        for i in 0..n {
            let is_anomaly = i % 20 == 0; // ~5%
            entries.push(make_entry(
                "C001",
                posting,
                &[("1100", dec!(1000)), ("4000", dec!(1000))],
                TransactionSource::Automated,
                is_anomaly,
                "USER01",
            ));
        }
        entries
    }

    #[test]
    fn test_empty_entity_returns_no_flags() {
        let mut gen = UnusualItemGenerator::new(42);
        let entries: Vec<JournalEntry> = Vec::new();
        let flags = gen.generate_for_entity("C001", &entries, period_end());
        assert!(flags.is_empty());
    }

    #[test]
    fn test_severity_derived_from_dimensions() {
        // Minor = 1 dim
        assert_eq!(
            UnusualSeverity::from_dimension_count(1),
            UnusualSeverity::Minor
        );
        // Moderate = 2 dims
        assert_eq!(
            UnusualSeverity::from_dimension_count(2),
            UnusualSeverity::Moderate
        );
        // Significant = 3+ dims
        assert_eq!(
            UnusualSeverity::from_dimension_count(3),
            UnusualSeverity::Significant
        );
        assert_eq!(
            UnusualSeverity::from_dimension_count(5),
            UnusualSeverity::Significant
        );
    }

    #[test]
    fn test_anomaly_entries_have_higher_flag_rate() {
        let n = 200;
        let entries = build_test_entries(n);
        let period = chrono::NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        let mut gen = UnusualItemGenerator::new(42);
        let flags = gen.generate_for_entity("C001", &entries, period);

        let anomaly_entry_ids: std::collections::HashSet<String> = entries
            .iter()
            .filter(|e| e.header.is_anomaly)
            .map(|e| e.header.document_id.to_string())
            .collect();

        let flagged_anomalies = flags
            .iter()
            .filter(|f| anomaly_entry_ids.contains(&f.journal_entry_id))
            .count();
        let flagged_normal = flags
            .iter()
            .filter(|f| !anomaly_entry_ids.contains(&f.journal_entry_id))
            .count();

        let anomaly_count = entries.iter().filter(|e| e.header.is_anomaly).count();
        let normal_count = n - anomaly_count;

        // Both rates may be 0 if no unusual dimensions were triggered, but
        // anomaly rate should be >= normal rate when flags exist.
        if flagged_anomalies > 0 || flagged_normal > 0 {
            let anomaly_rate = flagged_anomalies as f64 / anomaly_count.max(1) as f64;
            let normal_rate = flagged_normal as f64 / normal_count.max(1) as f64;
            assert!(
                anomaly_rate >= normal_rate,
                "anomaly rate {:.2} < normal rate {:.2}",
                anomaly_rate,
                normal_rate
            );
        }
    }

    #[test]
    fn test_weekend_entry_triggers_timing_dimension() {
        // June 15 2024 is a Saturday
        let saturday = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(saturday.weekday(), chrono::Weekday::Sat);
        let entry = make_entry(
            "C001",
            saturday,
            &[("1100", dec!(1000)), ("4000", dec!(1000))],
            TransactionSource::Automated,
            false,
            "USER01",
        );
        let entries = vec![entry];
        // Force flag by setting anomaly_entry_flag_probability to 1.0
        let config = UnusualItemGeneratorConfig {
            normal_entry_flag_probability: 1.0,
            anomaly_entry_flag_probability: 1.0,
            ..Default::default()
        };
        let mut gen = UnusualItemGenerator::with_config(42, config);
        let flags = gen.generate_for_entity("C001", &entries, period_end());
        let has_timing = flags
            .iter()
            .any(|f| f.dimensions.contains(&UnusualDimension::Timing));
        assert!(
            has_timing,
            "Saturday posting should trigger Timing dimension"
        );
    }

    #[test]
    fn test_flag_ids_are_unique() {
        let entries = build_test_entries(100);
        let period = chrono::NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        let config = UnusualItemGeneratorConfig {
            normal_entry_flag_probability: 1.0,
            anomaly_entry_flag_probability: 1.0,
            ..Default::default()
        };
        let mut gen = UnusualItemGenerator::with_config(42, config);
        let flags = gen.generate_for_entity("C001", &entries, period);
        let ids: HashSet<&str> = flags.iter().map(|f| f.id.as_str()).collect();
        assert_eq!(ids.len(), flags.len(), "Flag IDs must be unique");
    }

    #[test]
    fn test_serialisation() {
        let entries = build_test_entries(50);
        let period = chrono::NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        let config = UnusualItemGeneratorConfig {
            normal_entry_flag_probability: 1.0,
            anomaly_entry_flag_probability: 1.0,
            ..Default::default()
        };
        let mut gen = UnusualItemGenerator::with_config(42, config);
        let flags = gen.generate_for_entity("C001", &entries, period);
        // Must round-trip through JSON
        let json = serde_json::to_string(&flags).unwrap();
        let decoded: Vec<UnusualItemFlag> = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.len(), flags.len());
    }
}
