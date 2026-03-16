//! Business Combination Generator (IFRS 3 / ASC 805).
//!
//! Generates synthetic business combinations with:
//! - Realistic consideration amounts (1M – 50M based on company size)
//! - Purchase price allocation with 4–6 fair value adjustments
//! - Goodwill computation (or bargain purchase gain)
//! - Day 1 journal entries recording all acquired assets/liabilities
//! - Subsequent amortization JEs for finite-lived acquired intangibles

use chrono::{Datelike, NaiveDate};
use datasynth_core::accounts::{
    cash_accounts::OPERATING_CASH, control_accounts::FIXED_ASSETS, intangible_accounts::*,
};
use datasynth_core::models::{
    business_combination::{
        AcquisitionConsideration, AcquisitionFvAdjustment, AcquisitionPpa, BusinessCombination,
    },
    journal_entry::{JournalEntry, JournalEntryHeader, JournalEntryLine, TransactionSource},
};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand_distr::LogNormal;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ============================================================================
// Constants
// ============================================================================

/// Acquiree company names used for generated acquisitions.
const ACQUIREE_NAMES: &[&str] = &[
    "Apex Innovations Ltd",
    "BlueCrest Technologies Inc",
    "Cascade Manufacturing Co",
    "Deltron Systems GmbH",
    "Elevate Software Corp",
    "FusionTech Solutions",
    "GlobalEdge Partners",
    "Harbinger Analytics Inc",
    "IronBridge Industries",
    "Jetstream Logistics Ltd",
    "Keystone Digital GmbH",
    "Lighthouse Pharma Corp",
    "Meridian Energy Solutions",
    "NovaTrend Consulting",
    "Oceanic Data Systems",
    "Pinnacle Biotech AG",
    "Quickstep Retail Group",
    "Redwood Semiconductor",
    "Silverline Communications",
    "TrueVision AI Corp",
];

// ============================================================================
// Output snapshot
// ============================================================================

/// All output from one run of the business combination generator.
#[derive(Debug, Default)]
pub struct BusinessCombinationSnapshot {
    /// Business combination records.
    pub combinations: Vec<BusinessCombination>,
    /// All generated journal entries (Day 1 + amortization).
    pub journal_entries: Vec<JournalEntry>,
}

// ============================================================================
// Generator
// ============================================================================

/// Generates synthetic business combinations with purchase price allocation,
/// goodwill computation, Day 1 journal entries, and amortization schedules.
pub struct BusinessCombinationGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl BusinessCombinationGenerator {
    /// Create a new generator with a deterministic seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::BusinessCombination),
        }
    }

    /// Generate business combinations for a company.
    ///
    /// # Arguments
    /// * `company_code` – Acquirer company code
    /// * `currency` – Transaction currency (ISO 4217)
    /// * `start_date` – Start of the generation period
    /// * `end_date` – End of the generation period
    /// * `acquisition_count` – How many acquisitions to generate (1-5)
    /// * `framework` – "IFRS" or "US_GAAP"
    pub fn generate(
        &mut self,
        company_code: &str,
        currency: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        acquisition_count: usize,
        framework: &str,
    ) -> BusinessCombinationSnapshot {
        if acquisition_count == 0 {
            return BusinessCombinationSnapshot::default();
        }

        let count = acquisition_count.min(5);
        let mut snapshot = BusinessCombinationSnapshot::default();

        for i in 0..count {
            let combination =
                self.generate_one(company_code, currency, start_date, end_date, i, framework);

            // Day 1 JEs
            let day1_jes = self.generate_day1_journal_entries(company_code, currency, &combination);
            snapshot.journal_entries.extend(day1_jes);

            // Amortization JEs for finite-lived intangibles
            let amort_jes = self.generate_amortization_journal_entries(
                company_code,
                currency,
                &combination,
                start_date,
                end_date,
            );
            snapshot.journal_entries.extend(amort_jes);

            snapshot.combinations.push(combination);
        }

        snapshot
    }

    // =========================================================================
    // Private helpers
    // =========================================================================

    fn generate_one(
        &mut self,
        company_code: &str,
        currency: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        index: usize,
        framework: &str,
    ) -> BusinessCombination {
        let id = format!(
            "BC-{}-{:04}",
            company_code,
            self.rng.random_range(1u32..=9999u32)
        );

        let acquiree_name = ACQUIREE_NAMES[index % ACQUIREE_NAMES.len()].to_string();

        // Acquisition date: random day within [start_date + 30 days, end_date - 30 days]
        let acquisition_date = self.random_date_in_period(start_date, end_date);

        // --- Consideration ---
        let total_consideration = self.sample_consideration_amount();
        let consideration = self.build_consideration(total_consideration);

        // --- PPA ---
        let ppa = self.build_ppa(total_consideration, currency);

        // --- Goodwill ---
        let raw_goodwill = total_consideration - ppa.net_identifiable_assets_fv;
        let goodwill = if raw_goodwill > Decimal::ZERO {
            raw_goodwill
        } else {
            // Bargain purchase: IFRS 3/ASC 805 require gain recognition; goodwill = 0
            Decimal::ZERO
        };

        BusinessCombination {
            id,
            acquirer_entity: company_code.to_string(),
            acquiree_name,
            acquisition_date,
            consideration,
            purchase_price_allocation: ppa,
            goodwill,
            framework: framework.to_string(),
        }
    }

    /// Draw a random consideration amount between ~1M and ~50M (log-normal).
    fn sample_consideration_amount(&mut self) -> Decimal {
        // Log-normal centered around ln(10M) ≈ 16.1 with σ = 1.0
        let mu = 16.1_f64;
        let sigma = 1.0_f64;
        let log_normal = LogNormal::new(mu, sigma).expect("valid log-normal params");
        let raw: f64 = log_normal.sample(&mut self.rng);
        // Clamp to [1M, 50M]
        let clamped = raw.clamp(1_000_000.0, 50_000_000.0);
        // Round to nearest 1000
        let rounded = (clamped / 1_000.0).round() * 1_000.0;
        Decimal::from_f64_retain(rounded).unwrap_or(Decimal::from(10_000_000u64))
    }

    /// Build the consideration breakdown: 60-90% cash, remainder shares/contingent.
    fn build_consideration(&mut self, total: Decimal) -> AcquisitionConsideration {
        let cash_pct = self.rng.random_range(0.60_f64..=0.90_f64);
        let cash_pct_dec = Decimal::from_f64_retain(cash_pct).unwrap_or(dec!(0.75));
        let cash = (total * cash_pct_dec).round_dp(2);

        let remainder = total - cash;

        // 40% chance of contingent consideration from remaining balance
        let contingent = if self.rng.random_bool(0.40) {
            let contingent_pct = self.rng.random_range(0.30_f64..=0.60_f64);
            let contingent_pct_dec = Decimal::from_f64_retain(contingent_pct).unwrap_or(dec!(0.40));
            let c = (remainder * contingent_pct_dec).round_dp(2);
            Some(c)
        } else {
            None
        };

        let shares_issued_value = if remainder > Decimal::ZERO {
            let shares = remainder - contingent.unwrap_or(Decimal::ZERO);
            if shares > Decimal::ZERO {
                Some(shares.round_dp(2))
            } else {
                None
            }
        } else {
            None
        };

        AcquisitionConsideration {
            cash,
            shares_issued_value,
            contingent_consideration: contingent,
            total,
        }
    }

    /// Build the purchase price allocation with 4-6 asset/liability line items.
    fn build_ppa(&mut self, total_consideration: Decimal, _currency: &str) -> AcquisitionPpa {
        let mut assets: Vec<AcquisitionFvAdjustment> = Vec::new();
        let mut liabilities: Vec<AcquisitionFvAdjustment> = Vec::new();

        // 1. PP&E – step-up 10-25% of book value
        let ppe_book = self.pct_of(total_consideration, 0.25_f64, 0.45_f64);
        let ppe_stepup_pct = self.rng.random_range(0.10_f64..=0.25_f64);
        let ppe_fv = self.apply_step_up(ppe_book, ppe_stepup_pct);
        assets.push(AcquisitionFvAdjustment {
            asset_or_liability: "Property, Plant & Equipment".to_string(),
            book_value: ppe_book,
            fair_value: ppe_fv,
            step_up: ppe_fv - ppe_book,
            useful_life_years: None, // PP&E amortized separately
        });

        // 2. Customer Relationships – new intangible, 15-25% of total consideration
        let cr_fv = self.pct_of(total_consideration, 0.15_f64, 0.25_f64);
        let cr_life = self.rng.random_range(10u32..=15u32);
        assets.push(AcquisitionFvAdjustment {
            asset_or_liability: "Customer Relationships".to_string(),
            book_value: Decimal::ZERO,
            fair_value: cr_fv,
            step_up: cr_fv,
            useful_life_years: Some(cr_life),
        });

        // 3. Trade Name – 5-10% of consideration
        let tn_fv = self.pct_of(total_consideration, 0.05_f64, 0.10_f64);
        let tn_life = self.rng.random_range(15u32..=20u32);
        assets.push(AcquisitionFvAdjustment {
            asset_or_liability: "Trade Name".to_string(),
            book_value: Decimal::ZERO,
            fair_value: tn_fv,
            step_up: tn_fv,
            useful_life_years: Some(tn_life),
        });

        // 4. Technology / Developed Software – 5-15% of consideration
        let tech_fv = self.pct_of(total_consideration, 0.05_f64, 0.15_f64);
        let tech_life = self.rng.random_range(5u32..=8u32);
        assets.push(AcquisitionFvAdjustment {
            asset_or_liability: "Developed Technology".to_string(),
            book_value: Decimal::ZERO,
            fair_value: tech_fv,
            step_up: tech_fv,
            useful_life_years: Some(tech_life),
        });

        // 5. Inventory – step-up 3-8% of book value
        let inv_book = self.pct_of(total_consideration, 0.10_f64, 0.20_f64);
        let inv_stepup_pct = self.rng.random_range(0.03_f64..=0.08_f64);
        let inv_fv = self.apply_step_up(inv_book, inv_stepup_pct);
        assets.push(AcquisitionFvAdjustment {
            asset_or_liability: "Inventory".to_string(),
            book_value: inv_book,
            fair_value: inv_fv,
            step_up: inv_fv - inv_book,
            useful_life_years: None,
        });

        // 6. (optional) AR – at book value
        if self.rng.random_bool(0.70) {
            let ar_book = self.pct_of(total_consideration, 0.05_f64, 0.15_f64);
            assets.push(AcquisitionFvAdjustment {
                asset_or_liability: "Accounts Receivable".to_string(),
                book_value: ar_book,
                fair_value: ar_book, // typically at book for collectible AR
                step_up: Decimal::ZERO,
                useful_life_years: None,
            });
        }

        // Liabilities assumed
        // Accounts Payable
        let ap_book = self.pct_of(total_consideration, 0.08_f64, 0.18_f64);
        liabilities.push(AcquisitionFvAdjustment {
            asset_or_liability: "Accounts Payable".to_string(),
            book_value: ap_book,
            fair_value: ap_book,
            step_up: Decimal::ZERO,
            useful_life_years: None,
        });

        // Long-term debt (70% chance)
        if self.rng.random_bool(0.70) {
            let debt_book = self.pct_of(total_consideration, 0.10_f64, 0.25_f64);
            // Debt FV may differ slightly from book value when interest rates have moved
            let debt_fv_adj = self.rng.random_range(-0.05_f64..=0.05_f64);
            let debt_fv = self.apply_step_up(debt_book, debt_fv_adj);
            liabilities.push(AcquisitionFvAdjustment {
                asset_or_liability: "Long-term Debt".to_string(),
                book_value: debt_book,
                fair_value: debt_fv,
                step_up: debt_fv - debt_book,
                useful_life_years: None,
            });
        }

        // Deferred Revenue (if any)
        if self.rng.random_bool(0.40) {
            let def_rev = self.pct_of(total_consideration, 0.02_f64, 0.06_f64);
            liabilities.push(AcquisitionFvAdjustment {
                asset_or_liability: "Deferred Revenue".to_string(),
                book_value: def_rev,
                fair_value: def_rev,
                step_up: Decimal::ZERO,
                useful_life_years: None,
            });
        }

        // Compute net identifiable assets at FV
        let total_asset_fv: Decimal = assets.iter().map(|a| a.fair_value).sum();
        let total_liability_fv: Decimal = liabilities.iter().map(|l| l.fair_value).sum();
        let net_identifiable_assets_fv = total_asset_fv - total_liability_fv;

        AcquisitionPpa {
            identifiable_assets: assets,
            identifiable_liabilities: liabilities,
            net_identifiable_assets_fv,
        }
    }

    /// Generate the Day 1 acquisition journal entry:
    ///   DR acquired assets at fair value
    ///   DR Goodwill
    ///   CR acquired liabilities at fair value
    ///   CR Cash / Consideration
    fn generate_day1_journal_entries(
        &mut self,
        company_code: &str,
        currency: &str,
        bc: &BusinessCombination,
    ) -> Vec<JournalEntry> {
        let doc_id = self.uuid_factory.next();
        let mut header = JournalEntryHeader::with_deterministic_id(
            company_code.to_string(),
            bc.acquisition_date,
            doc_id,
        );
        header.document_type = "BC".to_string();
        header.currency = currency.to_string();
        header.source = TransactionSource::Manual;
        header.header_text = Some(format!("Acquisition of {} – Day 1 PPA", bc.acquiree_name));
        header.reference = Some(bc.id.clone());

        let mut je = JournalEntry::new(header);
        let mut line_num: u32 = 1;

        // DR acquired assets
        for adj in &bc.purchase_price_allocation.identifiable_assets {
            if adj.fair_value > Decimal::ZERO {
                let account = asset_gl_account(&adj.asset_or_liability);
                let mut line = JournalEntryLine::debit(doc_id, line_num, account, adj.fair_value);
                line.line_text = Some(format!("Acquired asset: {}", adj.asset_or_liability));
                je.add_line(line);
                line_num += 1;
            }
        }

        // DR Goodwill (if any)
        if bc.goodwill > Decimal::ZERO {
            let mut line =
                JournalEntryLine::debit(doc_id, line_num, GOODWILL.to_string(), bc.goodwill);
            line.line_text = Some(format!("Goodwill – acquisition of {}", bc.acquiree_name));
            je.add_line(line);
            line_num += 1;
        }

        // CR acquired liabilities
        for adj in &bc.purchase_price_allocation.identifiable_liabilities {
            if adj.fair_value > Decimal::ZERO {
                let account = liability_gl_account(&adj.asset_or_liability);
                let mut line = JournalEntryLine::credit(doc_id, line_num, account, adj.fair_value);
                line.line_text = Some(format!("Assumed liability: {}", adj.asset_or_liability));
                je.add_line(line);
                line_num += 1;
            }
        }

        // CR Cash for cash portion of consideration
        if bc.consideration.cash > Decimal::ZERO {
            let mut line = JournalEntryLine::credit(
                doc_id,
                line_num,
                OPERATING_CASH.to_string(),
                bc.consideration.cash,
            );
            line.line_text = Some("Cash paid – business combination".to_string());
            je.add_line(line);
            line_num += 1;
        }

        // CR Shares issued (if any) – APIC placeholder account "3100"
        if let Some(shares_val) = bc.consideration.shares_issued_value {
            if shares_val > Decimal::ZERO {
                let mut line =
                    JournalEntryLine::credit(doc_id, line_num, "3100".to_string(), shares_val);
                line.line_text = Some("Shares issued – business combination".to_string());
                je.add_line(line);
                line_num += 1;
            }
        }

        // CR Contingent consideration liability (if any) – account "2800"
        if let Some(contingent) = bc.consideration.contingent_consideration {
            if contingent > Decimal::ZERO {
                let mut line =
                    JournalEntryLine::credit(doc_id, line_num, "2800".to_string(), contingent);
                line.line_text = Some("Contingent consideration liability".to_string());
                je.add_line(line);
                line_num += 1;
            }
        }

        // If bargain purchase (consideration < net identifiable assets): CR Gain
        let raw_goodwill =
            bc.consideration.total - bc.purchase_price_allocation.net_identifiable_assets_fv;
        if raw_goodwill < Decimal::ZERO {
            let gain = (-raw_goodwill).round_dp(2);
            let mut line =
                JournalEntryLine::credit(doc_id, line_num, BARGAIN_PURCHASE_GAIN.to_string(), gain);
            line.line_text = Some("Bargain purchase gain".to_string());
            je.add_line(line);
        }

        vec![je]
    }

    /// Generate amortization JEs for finite-lived acquired intangibles, one
    /// JE per fiscal period (month) within the generation window where
    /// the combination date falls before the period end.
    fn generate_amortization_journal_entries(
        &mut self,
        company_code: &str,
        currency: &str,
        bc: &BusinessCombination,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Vec<JournalEntry> {
        let mut jes = Vec::new();

        // Collect finite-lived intangibles from PPA
        let intangibles: Vec<(&AcquisitionFvAdjustment, u32)> = bc
            .purchase_price_allocation
            .identifiable_assets
            .iter()
            .filter_map(|adj| adj.useful_life_years.map(|life| (adj, life)))
            .filter(|(adj, _)| adj.fair_value > Decimal::ZERO)
            .collect();

        if intangibles.is_empty() {
            return jes;
        }

        // Build list of month-end posting dates within the window
        let mut period_dates: Vec<NaiveDate> = Vec::new();
        let acq_date = bc.acquisition_date;
        let mut current =
            NaiveDate::from_ymd_opt(start_date.year(), start_date.month(), 1).unwrap_or(start_date);

        loop {
            // Last day of the current month
            let month_end = last_day_of_month(current.year(), current.month());
            if month_end > end_date {
                break;
            }
            // Only post amortization after acquisition
            if month_end > acq_date {
                period_dates.push(month_end);
            }
            // Advance to next month
            let next_month = current.month() % 12 + 1;
            let next_year = if current.month() == 12 {
                current.year() + 1
            } else {
                current.year()
            };
            match NaiveDate::from_ymd_opt(next_year, next_month, 1) {
                Some(d) => current = d,
                None => break,
            }
        }

        for period_end in period_dates {
            let doc_id = self.uuid_factory.next();
            let mut header = JournalEntryHeader::with_deterministic_id(
                company_code.to_string(),
                period_end,
                doc_id,
            );
            header.document_type = "AM".to_string();
            header.currency = currency.to_string();
            header.source = TransactionSource::Automated;
            header.header_text = Some(format!(
                "Amortization – acquired intangibles ({})",
                bc.acquiree_name
            ));
            header.reference = Some(bc.id.clone());

            let mut je = JournalEntry::new(header);
            let mut line_num: u32 = 1;
            for (adj, life_years) in &intangibles {
                // Monthly amortization = fair_value / (useful_life_years * 12)
                let months = Decimal::from(*life_years) * Decimal::from(12u32);
                let monthly_amort = (adj.fair_value / months).round_dp(2);

                if monthly_amort == Decimal::ZERO {
                    continue;
                }

                let amort_account = intangible_amort_account(&adj.asset_or_liability);

                // DR Amortization Expense
                let mut dr_line = JournalEntryLine::debit(
                    doc_id,
                    line_num,
                    AMORTIZATION_EXPENSE.to_string(),
                    monthly_amort,
                );
                dr_line.line_text = Some(format!("Amortization – {}", adj.asset_or_liability));
                je.add_line(dr_line);
                line_num += 1;

                // CR Accumulated Amortization
                let mut cr_line =
                    JournalEntryLine::credit(doc_id, line_num, amort_account, monthly_amort);
                cr_line.line_text =
                    Some(format!("Accum. amortization – {}", adj.asset_or_liability));
                je.add_line(cr_line);
                line_num += 1;
            }

            // Only add JE if it has lines
            if !je.lines.is_empty() {
                jes.push(je);
            }
        }

        jes
    }

    // =========================================================================
    // Utility helpers
    // =========================================================================

    /// Return an amount that is `pct_min .. pct_max` percent of `base`, rounded to 2 dp.
    fn pct_of(&mut self, base: Decimal, pct_min: f64, pct_max: f64) -> Decimal {
        let pct = self.rng.random_range(pct_min..=pct_max);
        let pct_dec = Decimal::from_f64_retain(pct)
            .unwrap_or(Decimal::from_f64_retain(pct_min).unwrap_or(Decimal::ONE));
        (base * pct_dec).round_dp(2)
    }

    /// Apply a percentage step-up to a book value; returns fair value.
    fn apply_step_up(&mut self, book_value: Decimal, step_up_pct: f64) -> Decimal {
        let pct_dec = Decimal::from_f64_retain(step_up_pct).unwrap_or(Decimal::ZERO);
        (book_value * (Decimal::ONE + pct_dec)).round_dp(2)
    }

    /// Generate a random acquisition date within the period, biased toward
    /// the first three quarters to leave room for amortization.
    fn random_date_in_period(&mut self, start: NaiveDate, end: NaiveDate) -> NaiveDate {
        let total_days = (end - start).num_days();
        if total_days <= 0 {
            return start;
        }
        // Use first 75% of the window so amortization JEs can be generated
        let usable_days = (total_days * 3 / 4).max(1);
        let offset = self.rng.random_range(0i64..usable_days);
        start + chrono::Duration::days(offset)
    }
}

// ============================================================================
// GL account mapping helpers
// ============================================================================

/// Map an asset description to its GL account number.
fn asset_gl_account(description: &str) -> String {
    match description {
        "Property, Plant & Equipment" => FIXED_ASSETS.to_string(),
        "Customer Relationships" => CUSTOMER_RELATIONSHIPS.to_string(),
        "Trade Name" => TRADE_NAME.to_string(),
        "Developed Technology" => TECHNOLOGY.to_string(),
        "Inventory" => "1200".to_string(),
        "Accounts Receivable" => "1100".to_string(),
        _ => "1890".to_string(), // Other intangible assets
    }
}

/// Map a liability description to its GL account number.
fn liability_gl_account(description: &str) -> String {
    match description {
        "Accounts Payable" => "2000".to_string(),
        "Long-term Debt" => "2600".to_string(),
        "Deferred Revenue" => "2300".to_string(),
        _ => "2890".to_string(), // Other assumed liabilities
    }
}

/// Map an intangible asset to its accumulated amortization contra-account.
fn intangible_amort_account(description: &str) -> String {
    // All finite-lived intangibles use ACCUMULATED_AMORTIZATION.
    // In a real system these would be sub-accounts; for simplicity we use one.
    let _ = description;
    ACCUMULATED_AMORTIZATION.to_string()
}

/// Return the last calendar day of the given year/month.
fn last_day_of_month(year: i32, month: u32) -> NaiveDate {
    let next_month = month % 12 + 1;
    let next_year = if month == 12 { year + 1 } else { year };
    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|d| d.pred_opt())
        .unwrap_or_else(|| {
            // Fallback: use the 28th which is always valid.
            NaiveDate::from_ymd_opt(year, month, 28)
                .unwrap_or(NaiveDate::from_ymd_opt(year, 1, 28).unwrap_or(NaiveDate::MIN))
        })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_gen() -> BusinessCombinationGenerator {
        BusinessCombinationGenerator::new(42)
    }

    fn make_dates() -> (NaiveDate, NaiveDate) {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        (start, end)
    }

    #[test]
    fn test_basic_generation() {
        let mut gen = make_gen();
        let (start, end) = make_dates();
        let snap = gen.generate("C001", "USD", start, end, 2, "IFRS");

        assert_eq!(snap.combinations.len(), 2);
        assert!(!snap.journal_entries.is_empty());
    }

    #[test]
    fn test_goodwill_equals_consideration_minus_net_assets() {
        let mut gen = make_gen();
        let (start, end) = make_dates();
        let snap = gen.generate("C001", "USD", start, end, 3, "US_GAAP");

        for bc in &snap.combinations {
            let raw_goodwill =
                bc.consideration.total - bc.purchase_price_allocation.net_identifiable_assets_fv;
            if raw_goodwill >= Decimal::ZERO {
                assert_eq!(bc.goodwill, raw_goodwill, "Goodwill mismatch for {}", bc.id);
            } else {
                // Bargain purchase: goodwill must be zero
                assert_eq!(
                    bc.goodwill,
                    Decimal::ZERO,
                    "Bargain purchase goodwill should be zero for {}",
                    bc.id
                );
            }
        }
    }

    #[test]
    fn test_at_least_4_identifiable_assets() {
        let mut gen = make_gen();
        let (start, end) = make_dates();
        let snap = gen.generate("C001", "USD", start, end, 3, "IFRS");

        for bc in &snap.combinations {
            assert!(
                bc.purchase_price_allocation.identifiable_assets.len() >= 4,
                "PPA should have at least 4 assets, got {} for {}",
                bc.purchase_price_allocation.identifiable_assets.len(),
                bc.id
            );
        }
    }

    #[test]
    fn test_day1_jes_balanced() {
        let mut gen = make_gen();
        let (start, end) = make_dates();
        let snap = gen.generate("C001", "USD", start, end, 2, "IFRS");

        // Collect only Day 1 JEs (document_type "BC")
        let day1_jes: Vec<_> = snap
            .journal_entries
            .iter()
            .filter(|je| je.header.document_type == "BC")
            .collect();

        assert!(!day1_jes.is_empty(), "Should have Day 1 JEs");

        for je in &day1_jes {
            let total_debits: Decimal = je.lines.iter().map(|l| l.debit_amount).sum();
            let total_credits: Decimal = je.lines.iter().map(|l| l.credit_amount).sum();
            assert_eq!(
                total_debits, total_credits,
                "Day 1 JE {} is unbalanced: debits={}, credits={}",
                je.header.document_id, total_debits, total_credits
            );
        }
    }

    #[test]
    fn test_amortization_jes_balanced() {
        let mut gen = make_gen();
        let (start, end) = make_dates();
        let snap = gen.generate("C001", "USD", start, end, 2, "IFRS");

        let amort_jes: Vec<_> = snap
            .journal_entries
            .iter()
            .filter(|je| je.header.document_type == "AM")
            .collect();

        assert!(!amort_jes.is_empty(), "Should have amortization JEs");

        for je in &amort_jes {
            let total_debits: Decimal = je.lines.iter().map(|l| l.debit_amount).sum();
            let total_credits: Decimal = je.lines.iter().map(|l| l.credit_amount).sum();
            assert_eq!(
                total_debits, total_credits,
                "Amortization JE {} is unbalanced: debits={}, credits={}",
                je.header.document_id, total_debits, total_credits
            );
        }
    }

    #[test]
    fn test_ppa_fair_values_positive_for_assets() {
        let mut gen = make_gen();
        let (start, end) = make_dates();
        let snap = gen.generate("C001", "USD", start, end, 2, "US_GAAP");

        for bc in &snap.combinations {
            for adj in &bc.purchase_price_allocation.identifiable_assets {
                assert!(
                    adj.fair_value > Decimal::ZERO,
                    "Asset {} should have positive fair value for {}",
                    adj.asset_or_liability,
                    bc.id
                );
            }
        }
    }

    #[test]
    fn test_consideration_total_correct() {
        let mut gen = make_gen();
        let (start, end) = make_dates();
        let snap = gen.generate("C001", "USD", start, end, 3, "IFRS");

        for bc in &snap.combinations {
            let c = &bc.consideration;
            let computed_total = c.cash
                + c.shares_issued_value.unwrap_or(Decimal::ZERO)
                + c.contingent_consideration.unwrap_or(Decimal::ZERO);
            assert_eq!(
                computed_total, c.total,
                "Consideration components don't add up for {}",
                bc.id
            );
        }
    }

    #[test]
    fn test_deterministic_output() {
        let (start, end) = make_dates();
        let mut gen1 = BusinessCombinationGenerator::new(99);
        let mut gen2 = BusinessCombinationGenerator::new(99);

        let snap1 = gen1.generate("C001", "USD", start, end, 2, "IFRS");
        let snap2 = gen2.generate("C001", "USD", start, end, 2, "IFRS");

        assert_eq!(snap1.combinations.len(), snap2.combinations.len());
        for (a, b) in snap1.combinations.iter().zip(snap2.combinations.iter()) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.goodwill, b.goodwill);
            assert_eq!(a.consideration.total, b.consideration.total);
        }
        assert_eq!(snap1.journal_entries.len(), snap2.journal_entries.len());
    }

    #[test]
    fn test_zero_count_returns_empty() {
        let mut gen = make_gen();
        let (start, end) = make_dates();
        let snap = gen.generate("C001", "USD", start, end, 0, "IFRS");
        assert!(snap.combinations.is_empty());
        assert!(snap.journal_entries.is_empty());
    }
}
