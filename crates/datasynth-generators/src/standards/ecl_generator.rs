//! Expected Credit Loss generator — IFRS 9 / ASC 326.
//!
//! Implements the **simplified approach** for trade receivables using a
//! provision matrix based on AR aging data.
//!
//! # Loss rates by aging bucket (historical baseline)
//!
//! | Bucket      | Rate  |
//! |-------------|-------|
//! | Current     | 0.5%  |
//! | 1–30 days   | 2.0%  |
//! | 31–60 days  | 5.0%  |
//! | 61–90 days  | 10.0% |
//! | Over 90 days| 25.0% |
//!
//! Forward-looking adjustments are applied via scenario-weighted multipliers
//! (base / optimistic / pessimistic) configured in [`EclConfig`].
//!
//! # Outputs
//! - [`EclModel`] — complete ECL model with provision matrix
//! - [`EclProvisionMovement`] — provision roll-forward for the period
//! - [`JournalEntry`] — Bad Debt Expense DR / Allowance for Doubtful Accounts CR

use chrono::NaiveDate;
use datasynth_config::schema::EclConfig;
use datasynth_core::accounts::{control_accounts::AR_CONTROL, expense_accounts::BAD_DEBT};
use datasynth_core::models::expected_credit_loss::{
    EclApproach, EclModel, EclPortfolioSegment, EclProvisionMovement, EclStage, EclStageAllocation,
    ProvisionMatrix, ProvisionMatrixRow, ScenarioWeights,
};
use datasynth_core::models::journal_entry::{
    JournalEntry, JournalEntryHeader, JournalEntryLine, TransactionSource,
};
use datasynth_core::models::subledger::ar::AgingBucket;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ============================================================================
// GL account for "Allowance for Doubtful Accounts" (contra-asset)
// We use a sub-account of AR control: 1105
// ============================================================================
const ALLOWANCE_FOR_DOUBTFUL_ACCOUNTS: &str = "1105";

// ============================================================================
// Historical loss rates (as decimal fractions, NOT percentages)
// ============================================================================

/// Historical loss rate for the Current bucket (0.5%).
const RATE_CURRENT: Decimal = dec!(0.005);
/// Historical loss rate for the 1-30 days bucket (2.0%).
const RATE_1_30: Decimal = dec!(0.02);
/// Historical loss rate for the 31-60 days bucket (5.0%).
const RATE_31_60: Decimal = dec!(0.05);
/// Historical loss rate for the 61-90 days bucket (10.0%).
const RATE_61_90: Decimal = dec!(0.10);
/// Historical loss rate for the Over 90 days bucket (25.0%).
const RATE_OVER_90: Decimal = dec!(0.25);

// ============================================================================
// Snapshot
// ============================================================================

/// All outputs from one ECL generation run.
#[derive(Debug, Default)]
pub struct EclSnapshot {
    /// ECL models (one per company processed).
    pub ecl_models: Vec<EclModel>,
    /// Provision movement roll-forwards.
    pub provision_movements: Vec<EclProvisionMovement>,
    /// Journal entries (Bad Debt Expense / Allowance).
    pub journal_entries: Vec<JournalEntry>,
}

// ============================================================================
// Generator
// ============================================================================

/// Generates ECL models using the simplified approach for trade receivables.
pub struct EclGenerator {
    uuid_factory: DeterministicUuidFactory,
}

impl EclGenerator {
    /// Create a new ECL generator with a deterministic seed.
    pub fn new(seed: u64) -> Self {
        Self {
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::ExpectedCreditLoss),
        }
    }

    /// Generate ECL model from AR aging bucket totals.
    ///
    /// # Parameters
    /// - `entity_code`: company / entity identifier
    /// - `measurement_date`: balance-sheet date
    /// - `bucket_exposures`: gross AR balance per aging bucket (ordered: Current, 1-30, 31-60, 61-90, Over90)
    /// - `config`: ECL configuration (scenario weights / multipliers)
    /// - `period_label`: label for the provision movement (e.g. "2024-Q4")
    /// - `framework`: "IFRS_9" or "ASC_326"
    pub fn generate(
        &mut self,
        entity_code: &str,
        measurement_date: NaiveDate,
        bucket_exposures: &[(AgingBucket, Decimal)],
        config: &EclConfig,
        period_label: &str,
        framework: &str,
    ) -> EclSnapshot {
        // ---- Step 1: compute scenario-weighted forward-looking multiplier ------
        let base_w = Decimal::try_from(config.base_scenario_weight).unwrap_or(dec!(0.50));
        let base_m = Decimal::try_from(config.base_scenario_multiplier).unwrap_or(dec!(1.0));
        let opt_w = Decimal::try_from(config.optimistic_scenario_weight).unwrap_or(dec!(0.30));
        let opt_m = Decimal::try_from(config.optimistic_scenario_multiplier).unwrap_or(dec!(0.8));
        let pes_w = Decimal::try_from(config.pessimistic_scenario_weight).unwrap_or(dec!(0.20));
        let pes_m = Decimal::try_from(config.pessimistic_scenario_multiplier).unwrap_or(dec!(1.4));

        let blended_multiplier = (base_w * base_m + opt_w * opt_m + pes_w * pes_m).round_dp(6);

        let scenario_weights = ScenarioWeights {
            base: base_w,
            base_multiplier: base_m,
            optimistic: opt_w,
            optimistic_multiplier: opt_m,
            pessimistic: pes_w,
            pessimistic_multiplier: pes_m,
            blended_multiplier,
        };

        // ---- Step 2: build provision matrix rows --------------------------------
        let mut matrix_rows: Vec<ProvisionMatrixRow> = Vec::with_capacity(5);
        let mut total_provision = Decimal::ZERO;
        let mut total_exposure = Decimal::ZERO;

        for bucket in AgingBucket::all() {
            let exposure = bucket_exposures
                .iter()
                .find(|(b, _)| *b == bucket)
                .map(|(_, e)| *e)
                .unwrap_or(Decimal::ZERO);

            let historical_rate = historical_rate_for_bucket(bucket);
            let applied_rate = (historical_rate * blended_multiplier).round_dp(6);
            let provision = (exposure * applied_rate).round_dp(2);

            total_exposure += exposure;
            total_provision += provision;

            matrix_rows.push(ProvisionMatrixRow {
                bucket,
                historical_loss_rate: historical_rate,
                forward_looking_adjustment: blended_multiplier,
                applied_loss_rate: applied_rate,
                exposure,
                provision,
            });
        }

        let blended_loss_rate = if total_exposure.is_zero() {
            Decimal::ZERO
        } else {
            (total_provision / total_exposure).round_dp(6)
        };

        let provision_matrix = ProvisionMatrix {
            entity_code: entity_code.to_string(),
            measurement_date,
            scenario_weights,
            aging_buckets: matrix_rows,
            total_provision,
            total_exposure,
            blended_loss_rate,
        };

        // ---- Step 3: build portfolio segment (simplified: single segment) ------
        // Under simplified approach we map the entire AR portfolio to Stage 2
        // (lifetime ECL is always recognised, no stage 1/12-month ECL).
        // We distribute the provision matrix total into stage allocations for
        // reporting completeness (Stage 1 = Current bucket, Stage 2 = 1-90 days,
        // Stage 3 = Over 90 days).
        let stage_allocations =
            build_stage_allocations(&provision_matrix.aging_buckets, blended_multiplier);

        let segment = EclPortfolioSegment {
            segment_name: "Trade Receivables".to_string(),
            exposure_at_default: total_exposure,
            total_ecl: total_provision,
            staging: stage_allocations,
        };

        // ---- Step 4: top-level ECL model ----------------------------------------
        let model_id = self.uuid_factory.next().to_string();
        let ecl_model = EclModel {
            id: model_id,
            entity_code: entity_code.to_string(),
            approach: EclApproach::Simplified,
            measurement_date,
            framework: framework.to_string(),
            portfolio_segments: vec![segment],
            provision_matrix: Some(provision_matrix),
            total_ecl: total_provision,
            total_exposure,
        };

        // ---- Step 5: provision movement ------------------------------------------
        // For a first-period run opening = 0; closing = total provision.
        // Write-offs are estimated at 20% of the Over 90 bucket provision.
        let over90_provision = ecl_model
            .provision_matrix
            .as_ref()
            .and_then(|m| {
                m.aging_buckets
                    .iter()
                    .find(|r| r.bucket == AgingBucket::Over90Days)
                    .map(|r| r.provision)
            })
            .unwrap_or(Decimal::ZERO);

        let estimated_write_offs = (over90_provision * dec!(0.20)).round_dp(2);
        let recoveries = Decimal::ZERO;
        // TODO: multi-period continuity — opening balance always starts at zero because the
        // current single-period generation model has no prior-period state.  Proper ECL
        // rollforward continuity requires a persistent state store shared across generation
        // runs, which is a larger architectural change (see Fix 12 documentation).
        let opening = Decimal::ZERO; // first period only
        let new_originations = total_provision;
        let stage_transfers = Decimal::ZERO;
        let closing = (opening + new_originations + stage_transfers - estimated_write_offs
            + recoveries)
            .round_dp(2);
        let pl_charge =
            (new_originations + stage_transfers + recoveries - estimated_write_offs).round_dp(2);

        let movement_id = self.uuid_factory.next().to_string();
        let movement = EclProvisionMovement {
            id: movement_id,
            entity_code: entity_code.to_string(),
            period: period_label.to_string(),
            opening,
            new_originations,
            stage_transfers,
            write_offs: estimated_write_offs,
            recoveries,
            closing,
            pl_charge,
        };

        // ---- Step 6: journal entry ----------------------------------------------
        let je = build_ecl_journal_entry(
            &mut self.uuid_factory,
            entity_code,
            measurement_date,
            pl_charge,
        );

        EclSnapshot {
            ecl_models: vec![ecl_model],
            provision_movements: vec![movement],
            journal_entries: vec![je],
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Returns the historical loss rate for each aging bucket.
fn historical_rate_for_bucket(bucket: AgingBucket) -> Decimal {
    match bucket {
        AgingBucket::Current => RATE_CURRENT,
        AgingBucket::Days1To30 => RATE_1_30,
        AgingBucket::Days31To60 => RATE_31_60,
        AgingBucket::Days61To90 => RATE_61_90,
        AgingBucket::Over90Days => RATE_OVER_90,
    }
}

/// Build stage allocations from provision matrix rows.
///
/// Mapping:
/// - Stage 1 (12-month ECL): Current bucket
/// - Stage 2 (Lifetime): 1-30, 31-60, 61-90 days
/// - Stage 3 (Credit-impaired): Over 90 days
fn build_stage_allocations(
    rows: &[ProvisionMatrixRow],
    forward_looking_adjustment: Decimal,
) -> Vec<EclStageAllocation> {
    let mut stage1_exposure = Decimal::ZERO;
    let mut stage1_ecl = Decimal::ZERO;
    let mut stage1_hist_rate = Decimal::ZERO;

    let mut stage2_exposure = Decimal::ZERO;
    let mut stage2_ecl = Decimal::ZERO;
    let mut stage2_hist_rate = Decimal::ZERO;

    let mut stage3_exposure = Decimal::ZERO;
    let mut stage3_ecl = Decimal::ZERO;
    let mut stage3_hist_rate = Decimal::ZERO;

    for row in rows {
        match row.bucket {
            AgingBucket::Current => {
                stage1_exposure += row.exposure;
                stage1_ecl += row.provision;
                stage1_hist_rate = row.historical_loss_rate;
            }
            AgingBucket::Days1To30 | AgingBucket::Days31To60 | AgingBucket::Days61To90 => {
                stage2_exposure += row.exposure;
                stage2_ecl += row.provision;
                // Use the highest historical rate in the group for the summary
                if row.historical_loss_rate > stage2_hist_rate {
                    stage2_hist_rate = row.historical_loss_rate;
                }
            }
            AgingBucket::Over90Days => {
                stage3_exposure += row.exposure;
                stage3_ecl += row.provision;
                stage3_hist_rate = row.historical_loss_rate;
            }
        }
    }

    // PD / LGD: simplified approach doesn't separate PD and LGD, but we
    // model as PD × LGD = applied_loss_rate. For Stage 1/2 assume LGD = 1.0;
    // for Stage 3 assume LGD = 0.60 (40% recovery).
    let lgd_stage1 = dec!(1.0);
    let lgd_stage2 = dec!(1.0);
    let lgd_stage3 = dec!(0.60);

    let pd_stage1 = (stage1_hist_rate * forward_looking_adjustment).round_dp(6);
    let pd_stage2 = (stage2_hist_rate * forward_looking_adjustment).round_dp(6);
    let pd_stage3 = if lgd_stage3.is_zero() {
        Decimal::ZERO
    } else {
        (stage3_hist_rate * forward_looking_adjustment / lgd_stage3).round_dp(6)
    };

    vec![
        EclStageAllocation {
            stage: EclStage::Stage1Month12,
            exposure: stage1_exposure,
            probability_of_default: pd_stage1,
            loss_given_default: lgd_stage1,
            ecl_amount: stage1_ecl,
            forward_looking_adjustment,
        },
        EclStageAllocation {
            stage: EclStage::Stage2Lifetime,
            exposure: stage2_exposure,
            probability_of_default: pd_stage2,
            loss_given_default: lgd_stage2,
            ecl_amount: stage2_ecl,
            forward_looking_adjustment,
        },
        EclStageAllocation {
            stage: EclStage::Stage3CreditImpaired,
            exposure: stage3_exposure,
            probability_of_default: pd_stage3,
            loss_given_default: lgd_stage3,
            ecl_amount: stage3_ecl,
            forward_looking_adjustment,
        },
    ]
}

/// Build the ECL journal entry:
///
/// ```text
/// DR  Bad Debt Expense (6900)                    pl_charge
///   CR  Allowance for Doubtful Accounts (1105)   pl_charge
/// ```
fn build_ecl_journal_entry(
    _uuid_factory: &mut DeterministicUuidFactory,
    entity_code: &str,
    posting_date: NaiveDate,
    pl_charge: Decimal,
) -> JournalEntry {
    // Ignore zero or negative charges (no JE needed)
    let amount = pl_charge.max(Decimal::ZERO);

    let mut header = JournalEntryHeader::new(entity_code.to_string(), posting_date);
    header.header_text = Some(format!(
        "ECL provision — Bad Debt Expense / Allowance for Doubtful Accounts ({posting_date})"
    ));
    header.source = TransactionSource::Adjustment;
    header.reference = Some("IFRS9/ASC326-ECL".to_string());
    // Suppress unused-import warning: AR_CONTROL is documented but not in
    // this JE because the allowance is a contra-asset sub-account.
    let _ = AR_CONTROL;

    let doc_id = header.document_id;
    let mut je = JournalEntry::new(header);

    if amount > Decimal::ZERO {
        // DR Bad Debt Expense
        je.add_line(JournalEntryLine::debit(
            doc_id,
            1,
            BAD_DEBT.to_string(),
            amount,
        ));

        // CR Allowance for Doubtful Accounts
        je.add_line(JournalEntryLine::credit(
            doc_id,
            2,
            ALLOWANCE_FOR_DOUBTFUL_ACCOUNTS.to_string(),
            amount,
        ));
    }

    je
}
