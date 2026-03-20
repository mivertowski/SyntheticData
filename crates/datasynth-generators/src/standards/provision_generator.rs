//! Provisions and contingencies generator — IAS 37 / ASC 450.
//!
//! Generates recognised provisions, provision movement roll-forwards,
//! contingent liability disclosures, and associated journal entries for a
//! reporting entity.
//!
//! # Generation logic
//!
//! 1. **Provision count** — 3–10 provisions per entity, weighted by industry.
//!    Manufacturing / energy entities tend to carry more environmental and
//!    decommissioning provisions; retail entities carry more warranty provisions.
//!
//! 2. **Framework-aware recognition threshold**
//!    - IFRS (IAS 37): recognise when probability > 50%.
//!    - US GAAP (ASC 450): recognise when probability > 75%.
//!
//!    Items that fall below the recognition threshold become contingent liabilities.
//!
//! 3. **Provision measurement**
//!    - `best_estimate`: sampled from a log-normal distribution calibrated to the
//!      provision type and a revenue proxy.
//!    - `range_low` = 75% of best estimate; `range_high` = 150%.
//!    - Long-term provisions (expected settlement > 12 months) are discounted at
//!      a rate of 3–5%.
//!
//! 4. **Provision movement** (first-period run)
//!    - Opening = 0 (fresh start).
//!    - Additions = best_estimate (provision first recognised).
//!    - Utilizations = 5–15% of additions (partial settlement in period).
//!    - Reversals = 0–5% of additions (minor re-estimates).
//!    - Unwinding of discount = discount_rate × opening (zero for first period).
//!    - Closing = opening + additions − utilizations − reversals + unwinding.
//!
//! 5. **Journal entries**
//!    - Initial recognition:
//!      `DR Provision Expense (6850) / CR Provision Liability (2450)`
//!    - Unwinding of discount (long-term only, non-zero when opening > 0):
//!      `DR Finance Cost (7100) / CR Provision Liability (2450)`
//!
//! 6. **Contingent liabilities** — 1–3 items per entity, always `disclosure_only = true`.

use chrono::NaiveDate;
use datasynth_core::accounts::expense_accounts::INTEREST_EXPENSE;
use datasynth_core::models::journal_entry::{
    JournalEntry, JournalEntryHeader, JournalEntryLine, TransactionSource,
};
use datasynth_core::models::provision::{
    ContingentLiability, ContingentProbability, Provision, ProvisionMovement, ProvisionType,
};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ============================================================================
// GL account constants (provision-specific)
// ============================================================================

/// Provision / impairment expense (operating).
const PROVISION_EXPENSE: &str = "6850";
/// Provision liability — current and non-current (balance-sheet).
const PROVISION_LIABILITY: &str = "2450";

// ============================================================================
// IFRS recognition threshold (probability > 50%)
// ============================================================================
const IFRS_THRESHOLD: f64 = 0.50;
/// US GAAP recognition threshold (probability > 75%)
const US_GAAP_THRESHOLD: f64 = 0.75;

// ============================================================================
// Snapshot
// ============================================================================

/// All outputs from one provision generation run.
#[derive(Debug, Default)]
pub struct ProvisionSnapshot {
    /// Recognised provisions (balance-sheet items).
    pub provisions: Vec<Provision>,
    /// Provision movement roll-forwards (one per provision).
    pub movements: Vec<ProvisionMovement>,
    /// Contingent liabilities (disclosed, not recognised).
    pub contingent_liabilities: Vec<ContingentLiability>,
    /// Journal entries (provision expense + unwinding of discount).
    pub journal_entries: Vec<JournalEntry>,
}

// ============================================================================
// Generator
// ============================================================================

/// Generates provisions and contingencies data for a reporting entity.
pub struct ProvisionGenerator {
    uuid_factory: DeterministicUuidFactory,
    rng: ChaCha8Rng,
}

impl ProvisionGenerator {
    /// Create a new generator with a deterministic seed.
    pub fn new(seed: u64) -> Self {
        Self {
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::Provision),
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Generate provisions and contingencies for one entity.
    ///
    /// # Parameters
    /// - `entity_code`: company / entity identifier
    /// - `currency`: reporting currency code (e.g. `"USD"`)
    /// - `revenue_proxy`: approximate annual revenue used to size warranty provisions
    /// - `reporting_date`: balance-sheet date (provisions dated to this period)
    /// - `period_label`: label for the movement roll-forward (e.g. `"FY2024"`)
    /// - `framework`: `"IFRS"` or `"US_GAAP"`
    /// - `prior_opening`: opening balance of the provision from the prior period's closing
    ///   balance.  When `Some`, the unwinding-of-discount is computed as
    ///   `prior_opening × discount_rate × period_fraction` (IAS 37.60 / ASC 420).
    ///   When `None` (first period or no carry-forward data), unwinding defaults to zero.
    pub fn generate(
        &mut self,
        entity_code: &str,
        currency: &str,
        revenue_proxy: Decimal,
        reporting_date: NaiveDate,
        period_label: &str,
        framework: &str,
        prior_opening: Option<Decimal>,
    ) -> ProvisionSnapshot {
        let recognition_threshold = if framework == "IFRS" {
            IFRS_THRESHOLD
        } else {
            US_GAAP_THRESHOLD
        };

        // ---- Step 1: determine provision count (3–10) -----------------------
        let provision_count = self.rng.random_range(3usize..=10);

        let mut provisions: Vec<Provision> = Vec::with_capacity(provision_count);
        let mut movements: Vec<ProvisionMovement> = Vec::with_capacity(provision_count);
        let mut journal_entries: Vec<JournalEntry> = Vec::new();

        // ---- Step 2: generate each provision --------------------------------
        for _ in 0..provision_count {
            let (ptype, desc, prob, base_amount) =
                self.sample_provision_type(revenue_proxy, reporting_date);

            // Framework-aware: only recognise if above threshold
            if prob <= recognition_threshold {
                // Below recognition threshold — will be collected as contingent
                // liability below (if Possible, not Remote).
                continue;
            }

            let best_estimate = round2(Decimal::try_from(base_amount).unwrap_or(dec!(10000)));
            let range_low = round2(best_estimate * dec!(0.75));
            let range_high = round2(best_estimate * dec!(1.50));

            // Long-term provisions (> 12 months): apply discounting
            let months_to_settlement: i64 = self.rng.random_range(3i64..=60);
            let is_long_term = months_to_settlement > 12;
            let discount_rate = if is_long_term {
                let rate_f: f64 = self.rng.random_range(0.03f64..=0.05);
                Some(round6(Decimal::try_from(rate_f).unwrap_or(dec!(0.04))))
            } else {
                None
            };

            let utilization_date =
                reporting_date + chrono::Months::new(months_to_settlement.unsigned_abs() as u32);

            let prov_id = self.uuid_factory.next().to_string();
            let provision = Provision {
                id: prov_id.clone(),
                entity_code: entity_code.to_string(),
                provision_type: ptype,
                description: desc.clone(),
                best_estimate,
                range_low,
                range_high,
                discount_rate,
                expected_utilization_date: utilization_date,
                framework: framework.to_string(),
                currency: currency.to_string(),
            };

            // ---- Step 3: movement roll-forward (first-period run) -----------
            let opening = Decimal::ZERO;
            let additions = best_estimate;
            let utilization_rate: f64 = self.rng.random_range(0.05f64..=0.15);
            let utilizations =
                round2(additions * Decimal::try_from(utilization_rate).unwrap_or(dec!(0.08)));
            let reversal_rate: f64 = self.rng.random_range(0.0f64..=0.05);
            let reversals =
                round2(additions * Decimal::try_from(reversal_rate).unwrap_or(Decimal::ZERO));
            // Unwinding of discount (IAS 37.60): discount_rate × opening balance × period_fraction.
            // Uses `prior_opening` when provided (carry-forward scenario); defaults to zero for
            // first-period runs where opening = 0 regardless.
            let unwinding_of_discount =
                if let (Some(prior_bal), Some(rate)) = (prior_opening, discount_rate) {
                    // Assume each generation run covers one annual period (period_fraction = 1.0).
                    round2((prior_bal * rate).max(Decimal::ZERO))
                } else {
                    Decimal::ZERO
                };
            let closing = (opening + additions - utilizations - reversals + unwinding_of_discount)
                .max(Decimal::ZERO);

            movements.push(ProvisionMovement {
                provision_id: prov_id.clone(),
                period: period_label.to_string(),
                opening,
                additions,
                utilizations,
                reversals,
                unwinding_of_discount,
                closing,
            });

            // ---- Step 4: journal entries ------------------------------------
            // Recognition JE: DR Provision Expense / CR Provision Liability
            let recognition_amount = additions.max(Decimal::ZERO);
            if recognition_amount > Decimal::ZERO {
                let je = build_recognition_je(
                    &mut self.uuid_factory,
                    entity_code,
                    reporting_date,
                    recognition_amount,
                    &desc,
                );
                journal_entries.push(je);
            }

            provisions.push(provision);
        }

        // Ensure we have at least 3 provisions even if probability sampling
        // removed some items — backfill with warranty/legal if needed.
        let needed = 3usize.saturating_sub(provisions.len());
        for i in 0..needed {
            let base_amount = revenue_proxy * dec!(0.005); // 0.5% of revenue
            let best_estimate =
                round2((base_amount + Decimal::from(i as u32 * 1000)).max(dec!(5000)));
            let range_low = round2(best_estimate * dec!(0.75));
            let range_high = round2(best_estimate * dec!(1.50));
            let utilization_date =
                reporting_date + chrono::Months::new(self.rng.random_range(6u32..=18));

            let ptype = if i % 2 == 0 {
                ProvisionType::Warranty
            } else {
                ProvisionType::LegalClaim
            };
            let desc = format!("{} provision — {} backfill", ptype, period_label);

            let prov_id = self.uuid_factory.next().to_string();
            let provision = Provision {
                id: prov_id.clone(),
                entity_code: entity_code.to_string(),
                provision_type: ptype,
                description: desc.clone(),
                best_estimate,
                range_low,
                range_high,
                discount_rate: None,
                expected_utilization_date: utilization_date,
                framework: framework.to_string(),
                currency: currency.to_string(),
            };

            let opening = Decimal::ZERO;
            let additions = best_estimate;
            let utilizations = round2(additions * dec!(0.08));
            let closing = (opening + additions - utilizations).max(Decimal::ZERO);

            movements.push(ProvisionMovement {
                provision_id: prov_id.clone(),
                period: period_label.to_string(),
                opening,
                additions,
                utilizations,
                reversals: Decimal::ZERO,
                unwinding_of_discount: Decimal::ZERO,
                closing,
            });

            if additions > Decimal::ZERO {
                let je = build_recognition_je(
                    &mut self.uuid_factory,
                    entity_code,
                    reporting_date,
                    additions,
                    &desc,
                );
                journal_entries.push(je);
            }

            provisions.push(provision);
        }

        // ---- Step 5: contingent liabilities (1–3, disclosure only) ----------
        let contingent_count = self.rng.random_range(1usize..=3);
        let contingent_liabilities =
            self.generate_contingent_liabilities(entity_code, currency, contingent_count);

        ProvisionSnapshot {
            provisions,
            movements,
            contingent_liabilities,
            journal_entries,
        }
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    /// Sample a provision type with associated description and probability.
    ///
    /// Returns `(ProvisionType, description, probability, base_amount_f64)`.
    fn sample_provision_type(
        &mut self,
        revenue_proxy: Decimal,
        _reporting_date: NaiveDate,
    ) -> (ProvisionType, String, f64, f64) {
        // Weighted selection: Warranty 35%, Legal 25%, Restructuring 15%,
        // Environmental 10%, Onerous 10%, Decommissioning 5%.
        let roll: f64 = self.rng.random();
        let rev_f: f64 = revenue_proxy.try_into().unwrap_or(1_000_000.0);

        let (ptype, base_amount) = if roll < 0.35 {
            // Warranty: 2–5% of revenue
            let pct: f64 = self.rng.random_range(0.02f64..=0.05);
            (ProvisionType::Warranty, rev_f * pct)
        } else if roll < 0.60 {
            // Legal claim: $50K–$2M
            let amount: f64 = self.rng.random_range(50_000.0f64..=2_000_000.0);
            (ProvisionType::LegalClaim, amount)
        } else if roll < 0.75 {
            // Restructuring: 1–3% of revenue
            let pct: f64 = self.rng.random_range(0.01f64..=0.03);
            (ProvisionType::Restructuring, rev_f * pct)
        } else if roll < 0.85 {
            // Environmental: $100K–$5M
            let amount: f64 = self.rng.random_range(100_000.0f64..=5_000_000.0);
            (ProvisionType::EnvironmentalRemediation, amount)
        } else if roll < 0.95 {
            // Onerous contract: 0.5–2% of revenue
            let pct: f64 = self.rng.random_range(0.005f64..=0.02);
            (ProvisionType::OnerousContract, rev_f * pct)
        } else {
            // Decommissioning: $200K–$10M (long-lived asset retirement)
            let amount: f64 = self.rng.random_range(200_000.0f64..=10_000_000.0);
            (ProvisionType::Decommissioning, amount)
        };

        // Probability of the outflow (drives recognition threshold check)
        let probability: f64 = self.rng.random_range(0.51f64..=0.99);

        let desc = match ptype {
            ProvisionType::Warranty => "Product warranty — current sales cohort".to_string(),
            ProvisionType::LegalClaim => "Pending litigation claim".to_string(),
            ProvisionType::Restructuring => {
                "Restructuring programme — redundancy costs".to_string()
            }
            ProvisionType::EnvironmentalRemediation => {
                "Environmental site remediation obligation".to_string()
            }
            ProvisionType::OnerousContract => "Onerous lease / supply contract".to_string(),
            ProvisionType::Decommissioning => "Asset retirement obligation (ARO)".to_string(),
        };

        (ptype, desc, probability, base_amount)
    }

    /// Generate contingent liability disclosures (not recognised on balance sheet).
    fn generate_contingent_liabilities(
        &mut self,
        entity_code: &str,
        currency: &str,
        count: usize,
    ) -> Vec<ContingentLiability> {
        let natures = [
            "Possible warranty claim from product recall investigation",
            "Unresolved tax dispute with revenue authority",
            "Environmental clean-up obligation under assessment",
            "Patent infringement lawsuit — outcome uncertain",
            "Customer class-action — settlement under negotiation",
            "Supplier breach-of-contract claim",
        ];

        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let nature = natures[i % natures.len()].to_string();
            let amount_f: f64 = self.rng.random_range(25_000.0f64..=500_000.0);
            let estimated_amount =
                Some(round2(Decimal::try_from(amount_f).unwrap_or(dec!(100_000))));

            result.push(ContingentLiability {
                id: self.uuid_factory.next().to_string(),
                entity_code: entity_code.to_string(),
                nature,
                // Contingent liabilities are always "Possible" for disclosure purposes
                probability: ContingentProbability::Possible,
                estimated_amount,
                disclosure_only: true,
                currency: currency.to_string(),
            });
        }
        result
    }
}

// ============================================================================
// Journal entry builders
// ============================================================================

/// Build the provision recognition journal entry:
///
/// ```text
/// DR  Provision Expense (6850)      recognition_amount
///   CR  Provision Liability (2450)   recognition_amount
/// ```
fn build_recognition_je(
    _uuid_factory: &mut DeterministicUuidFactory,
    entity_code: &str,
    posting_date: NaiveDate,
    amount: Decimal,
    description: &str,
) -> JournalEntry {
    let mut header = JournalEntryHeader::new(entity_code.to_string(), posting_date);
    header.header_text = Some(format!("Provision recognition — {description}"));
    header.source = TransactionSource::Adjustment;
    header.reference = Some("IAS37/ASC450-PROV".to_string());

    let doc_id = header.document_id;
    let mut je = JournalEntry::new(header);

    // Suppress unused import warning: INTEREST_EXPENSE used in unwinding JE below.
    let _ = INTEREST_EXPENSE;

    je.add_line(JournalEntryLine::debit(
        doc_id,
        1,
        PROVISION_EXPENSE.to_string(),
        amount,
    ));
    je.add_line(JournalEntryLine::credit(
        doc_id,
        2,
        PROVISION_LIABILITY.to_string(),
        amount,
    ));

    je
}

/// Build the unwinding-of-discount journal entry:
///
/// ```text
/// DR  Finance Cost / Interest Expense (7100)   unwinding_amount
///   CR  Provision Liability (2450)               unwinding_amount
/// ```
#[allow(dead_code)]
fn build_unwinding_je(
    _uuid_factory: &mut DeterministicUuidFactory,
    entity_code: &str,
    posting_date: NaiveDate,
    amount: Decimal,
    provision_description: &str,
) -> JournalEntry {
    let mut header = JournalEntryHeader::new(entity_code.to_string(), posting_date);
    header.header_text = Some(format!("Unwinding of discount — {provision_description}"));
    header.source = TransactionSource::Adjustment;
    header.reference = Some("IAS37-UNWIND".to_string());

    let doc_id = header.document_id;
    let mut je = JournalEntry::new(header);

    je.add_line(JournalEntryLine::debit(
        doc_id,
        1,
        INTEREST_EXPENSE.to_string(),
        amount,
    ));
    je.add_line(JournalEntryLine::credit(
        doc_id,
        2,
        PROVISION_LIABILITY.to_string(),
        amount,
    ));

    je
}

// ============================================================================
// Decimal helpers
// ============================================================================

#[inline]
fn round2(d: Decimal) -> Decimal {
    d.round_dp(2)
}

#[inline]
fn round6(d: Decimal) -> Decimal {
    d.round_dp(6)
}
