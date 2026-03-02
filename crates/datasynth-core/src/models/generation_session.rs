//! Models for the unified generation pipeline's session state.
//!
//! These types track the state of a multi-period generation run,
//! including fiscal period decomposition, balance carry-forward,
//! document ID sequencing, and deterministic seed advancement.

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// ---------------------------------------------------------------------------
// GenerationPeriod — one slice of the total generation time span
// ---------------------------------------------------------------------------

/// A single period within a generation run.
///
/// The unified pipeline decomposes the total requested time span into
/// fiscal-year-aligned periods. Each period is generated independently
/// with its own RNG seed derived from the master seed.
///
/// Named `GenerationPeriod` to avoid collision with the accounting-level
/// [`FiscalPeriod`](super::FiscalPeriod) in `period_close`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenerationPeriod {
    /// Zero-based index of this period in the run.
    pub index: usize,
    /// Human-readable label, e.g. "FY2024" or "FY2024-H1".
    pub label: String,
    /// First calendar date of the period (inclusive).
    pub start_date: NaiveDate,
    /// Last calendar date of the period (inclusive).
    pub end_date: NaiveDate,
    /// Number of months covered by this period.
    pub months: u32,
}

impl GenerationPeriod {
    /// Decompose a total generation span into fiscal-year-aligned periods.
    ///
    /// # Arguments
    /// * `start_date`  — first day of the generation window
    /// * `total_months` — total months requested (e.g. 36 for 3 years)
    /// * `fiscal_year_months` — months per fiscal year (typically 12)
    ///
    /// # Returns
    /// A `Vec<GenerationPeriod>` covering the entire span. The last period
    /// may be shorter than `fiscal_year_months` if `total_months` is not
    /// evenly divisible.
    pub fn compute_periods(
        start_date: NaiveDate,
        total_months: u32,
        fiscal_year_months: u32,
    ) -> Vec<GenerationPeriod> {
        assert!(fiscal_year_months > 0, "fiscal_year_months must be > 0");
        assert!(total_months > 0, "total_months must be > 0");

        let mut periods = Vec::new();
        let mut remaining = total_months;
        let mut cursor = start_date;
        let mut index: usize = 0;

        while remaining > 0 {
            let months = remaining.min(fiscal_year_months);
            let end = add_months(cursor, months)
                .pred_opt()
                .expect("valid predecessor date");
            let label = format!("FY{}", cursor.year());

            periods.push(GenerationPeriod {
                index,
                label,
                start_date: cursor,
                end_date: end,
                months,
            });

            cursor = add_months(cursor, months);
            remaining -= months;
            index += 1;
        }

        periods
    }
}

// ---------------------------------------------------------------------------
// SessionState — mutable state carried across periods
// ---------------------------------------------------------------------------

/// Accumulated state for a multi-period generation session.
///
/// This struct is serializable so it can be checkpointed to disk and
/// resumed later (e.g. after a crash or for incremental generation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Master RNG seed for the entire run.
    pub rng_seed: u64,
    /// Index of the *next* period to generate (0 = fresh run).
    pub period_cursor: usize,
    /// GL and sub-ledger balances carried forward.
    pub balance_state: BalanceState,
    /// Next sequential document IDs.
    pub document_id_state: DocumentIdState,
    /// Counts of master data entities generated so far.
    pub entity_counts: EntityCounts,
    /// Per-period generation log (one entry per completed period).
    pub generation_log: Vec<PeriodLog>,
    /// SHA-256 hash of the config that created this session, used to
    /// detect config drift on resume.
    pub config_hash: String,
}

impl SessionState {
    /// Create a fresh session state for a new generation run.
    pub fn new(rng_seed: u64, config_hash: String) -> Self {
        Self {
            rng_seed,
            period_cursor: 0,
            balance_state: BalanceState::default(),
            document_id_state: DocumentIdState::default(),
            entity_counts: EntityCounts::default(),
            generation_log: Vec::new(),
            config_hash,
        }
    }
}

// ---------------------------------------------------------------------------
// BalanceState
// ---------------------------------------------------------------------------

/// GL and sub-ledger balances carried forward between periods.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BalanceState {
    /// Per-GL-account running balance (account_id -> balance).
    pub gl_balances: HashMap<String, f64>,
    /// Total accounts-receivable balance.
    pub ar_total: f64,
    /// Total accounts-payable balance.
    pub ap_total: f64,
    /// Net book value of all fixed assets.
    pub fa_net_book_value: f64,
    /// Retained earnings balance.
    pub retained_earnings: f64,
}

// ---------------------------------------------------------------------------
// DocumentIdState
// ---------------------------------------------------------------------------

/// Sequential document-ID counters so IDs never collide across periods.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentIdState {
    /// Next purchase-order number.
    pub next_po_number: u64,
    /// Next sales-order number.
    pub next_so_number: u64,
    /// Next journal-entry number.
    pub next_je_number: u64,
    /// Next invoice number.
    pub next_invoice_number: u64,
    /// Next payment number.
    pub next_payment_number: u64,
    /// Next goods-receipt number.
    pub next_gr_number: u64,
}

// ---------------------------------------------------------------------------
// EntityCounts
// ---------------------------------------------------------------------------

/// Counts of master-data entities generated so far.
///
/// Used to avoid regenerating master data in subsequent periods and to
/// allocate additional entities if growth is configured.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityCounts {
    /// Number of vendor master records.
    pub vendors: usize,
    /// Number of customer master records.
    pub customers: usize,
    /// Number of employee master records.
    pub employees: usize,
    /// Number of material master records.
    pub materials: usize,
    /// Number of fixed-asset master records.
    pub fixed_assets: usize,
}

// ---------------------------------------------------------------------------
// PeriodLog
// ---------------------------------------------------------------------------

/// Summary of what was generated in a single period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodLog {
    /// Label of the period (e.g. "FY2024").
    pub period_label: String,
    /// Number of journal entries generated.
    pub journal_entries: usize,
    /// Number of documents generated (PO, SO, GR, invoices, etc.).
    pub documents: usize,
    /// Number of anomalies injected.
    pub anomalies: usize,
    /// Wall-clock duration of the period generation in seconds.
    pub duration_secs: f64,
}

// ---------------------------------------------------------------------------
// Free functions
// ---------------------------------------------------------------------------

/// Derive a deterministic per-period RNG seed from the master seed.
///
/// Uses `DefaultHasher` (SipHash) to mix the seed with the period index,
/// producing a well-distributed child seed.
pub fn advance_seed(seed: u64, period_index: usize) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    seed.hash(&mut hasher);
    period_index.hash(&mut hasher);
    hasher.finish()
}

/// Add `months` calendar months to a `NaiveDate`, clamping the day to the
/// last valid day of the target month.
///
/// # Examples
/// ```
/// use chrono::NaiveDate;
/// use datasynth_core::models::generation_session::add_months;
///
/// let d = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
/// // Jan 31 + 1 month → Feb 29 (2024 is a leap year)
/// assert_eq!(add_months(d, 1), NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
/// ```
pub fn add_months(date: NaiveDate, months: u32) -> NaiveDate {
    let total_months = date.year() as i64 * 12 + (date.month() as i64 - 1) + months as i64;
    let target_year = (total_months / 12) as i32;
    let target_month = (total_months % 12) as u32 + 1;

    // Clamp day to last valid day of target month
    let max_day = last_day_of_month(target_year, target_month);
    let day = date.day().min(max_day);

    NaiveDate::from_ymd_opt(target_year, target_month, day).expect("valid date after add_months")
}

/// Return the last day of the given year/month.
fn last_day_of_month(year: i32, month: u32) -> u32 {
    if month == 12 {
        31
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
            .expect("valid next-month date")
            .pred_opt()
            .expect("valid predecessor")
            .day()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn test_compute_periods_single_year() {
        let periods = GenerationPeriod::compute_periods(date(2024, 1, 1), 12, 12);
        assert_eq!(periods.len(), 1);
        assert_eq!(periods[0].index, 0);
        assert_eq!(periods[0].label, "FY2024");
        assert_eq!(periods[0].start_date, date(2024, 1, 1));
        assert_eq!(periods[0].end_date, date(2024, 12, 31));
        assert_eq!(periods[0].months, 12);
    }

    #[test]
    fn test_compute_periods_three_years() {
        let periods = GenerationPeriod::compute_periods(date(2022, 1, 1), 36, 12);
        assert_eq!(periods.len(), 3);

        assert_eq!(periods[0].label, "FY2022");
        assert_eq!(periods[0].start_date, date(2022, 1, 1));
        assert_eq!(periods[0].end_date, date(2022, 12, 31));
        assert_eq!(periods[0].months, 12);

        assert_eq!(periods[1].label, "FY2023");
        assert_eq!(periods[1].start_date, date(2023, 1, 1));
        assert_eq!(periods[1].end_date, date(2023, 12, 31));
        assert_eq!(periods[1].months, 12);

        assert_eq!(periods[2].label, "FY2024");
        assert_eq!(periods[2].start_date, date(2024, 1, 1));
        assert_eq!(periods[2].end_date, date(2024, 12, 31));
        assert_eq!(periods[2].months, 12);
    }

    #[test]
    fn test_compute_periods_partial() {
        let periods = GenerationPeriod::compute_periods(date(2022, 1, 1), 18, 12);
        assert_eq!(periods.len(), 2);

        assert_eq!(periods[0].label, "FY2022");
        assert_eq!(periods[0].months, 12);
        assert_eq!(periods[0].end_date, date(2022, 12, 31));

        assert_eq!(periods[1].label, "FY2023");
        assert_eq!(periods[1].months, 6);
        assert_eq!(periods[1].start_date, date(2023, 1, 1));
        assert_eq!(periods[1].end_date, date(2023, 6, 30));
    }

    #[test]
    fn test_advance_seed_deterministic() {
        let a = advance_seed(42, 0);
        let b = advance_seed(42, 0);
        assert_eq!(a, b, "same inputs must produce same seed");
    }

    #[test]
    fn test_advance_seed_differs_by_index() {
        let a = advance_seed(42, 0);
        let b = advance_seed(42, 1);
        assert_ne!(a, b, "different indices must produce different seeds");
    }

    #[test]
    fn test_session_state_serde_roundtrip() {
        let mut state = SessionState::new(12345, "abc123hash".to_string());
        state.period_cursor = 2;
        state.balance_state.ar_total = 50_000.0;
        state.balance_state.retained_earnings = 100_000.0;
        state
            .balance_state
            .gl_balances
            .insert("1100".to_string(), 50_000.0);
        state.document_id_state.next_je_number = 500;
        state.entity_counts.vendors = 42;
        state.generation_log.push(PeriodLog {
            period_label: "FY2024".to_string(),
            journal_entries: 1000,
            documents: 2500,
            anomalies: 25,
            duration_secs: 3.14,
        });

        let json = serde_json::to_string(&state).expect("serialize");
        let restored: SessionState = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.rng_seed, 12345);
        assert_eq!(restored.period_cursor, 2);
        assert_eq!(restored.balance_state.ar_total, 50_000.0);
        assert_eq!(restored.balance_state.retained_earnings, 100_000.0);
        assert_eq!(
            restored.balance_state.gl_balances.get("1100"),
            Some(&50_000.0)
        );
        assert_eq!(restored.document_id_state.next_je_number, 500);
        assert_eq!(restored.entity_counts.vendors, 42);
        assert_eq!(restored.generation_log.len(), 1);
        assert_eq!(restored.generation_log[0].journal_entries, 1000);
        assert_eq!(restored.config_hash, "abc123hash");
    }

    #[test]
    fn test_balance_state_default() {
        let bs = BalanceState::default();
        assert!(bs.gl_balances.is_empty());
        assert_eq!(bs.ar_total, 0.0);
        assert_eq!(bs.ap_total, 0.0);
        assert_eq!(bs.fa_net_book_value, 0.0);
        assert_eq!(bs.retained_earnings, 0.0);
    }

    #[test]
    fn test_add_months_basic() {
        assert_eq!(add_months(date(2024, 1, 1), 1), date(2024, 2, 1));
        assert_eq!(add_months(date(2024, 1, 1), 12), date(2025, 1, 1));
        assert_eq!(add_months(date(2024, 11, 1), 2), date(2025, 1, 1));
    }

    #[test]
    fn test_add_months_day_clamping() {
        // Jan 31 + 1 month → Feb 29 (leap year 2024)
        assert_eq!(add_months(date(2024, 1, 31), 1), date(2024, 2, 29));
        // Jan 31 + 1 month → Feb 28 (non-leap year 2023)
        assert_eq!(add_months(date(2023, 1, 31), 1), date(2023, 2, 28));
    }

    #[test]
    fn test_document_id_state_default() {
        let d = DocumentIdState::default();
        assert_eq!(d.next_po_number, 0);
        assert_eq!(d.next_so_number, 0);
        assert_eq!(d.next_je_number, 0);
        assert_eq!(d.next_invoice_number, 0);
        assert_eq!(d.next_payment_number, 0);
        assert_eq!(d.next_gr_number, 0);
    }

    #[test]
    fn test_entity_counts_default() {
        let e = EntityCounts::default();
        assert_eq!(e.vendors, 0);
        assert_eq!(e.customers, 0);
        assert_eq!(e.employees, 0);
        assert_eq!(e.materials, 0);
        assert_eq!(e.fixed_assets, 0);
    }
}
