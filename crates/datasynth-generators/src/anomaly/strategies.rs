//! Injection strategies for anomaly generation.
//!
//! Strategies determine how anomalies are applied to existing data.

use chrono::Datelike;
use rand::Rng;
use rust_decimal::Decimal;

use datasynth_core::models::{
    AnomalyType, ControlStatus, ErrorType, FraudType, JournalEntry, ProcessIssueType,
    RelationalAnomalyType, StatisticalAnomalyType,
};
use datasynth_core::uuid_factory::DeterministicUuidFactory;

/// Base trait for injection strategies.
pub trait InjectionStrategy {
    /// Name of the strategy.
    fn name(&self) -> &'static str;

    /// Whether this strategy can be applied to the given entry.
    fn can_apply(&self, entry: &JournalEntry) -> bool;

    /// Applies the strategy to modify an entry.
    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult;
}

/// Result of an injection attempt.
#[derive(Debug, Clone)]
pub struct InjectionResult {
    /// Whether the injection was successful.
    pub success: bool,
    /// Description of what was modified.
    pub description: String,
    /// Monetary impact of the anomaly.
    pub monetary_impact: Option<Decimal>,
    /// Related entity IDs.
    pub related_entities: Vec<String>,
    /// Additional metadata.
    pub metadata: Vec<(String, String)>,
}

impl InjectionResult {
    /// Creates a successful result.
    pub fn success(description: &str) -> Self {
        Self {
            success: true,
            description: description.to_string(),
            monetary_impact: None,
            related_entities: Vec::new(),
            metadata: Vec::new(),
        }
    }

    /// Creates a failed result.
    pub fn failure(reason: &str) -> Self {
        Self {
            success: false,
            description: reason.to_string(),
            monetary_impact: None,
            related_entities: Vec::new(),
            metadata: Vec::new(),
        }
    }

    /// Adds monetary impact.
    pub fn with_impact(mut self, impact: Decimal) -> Self {
        self.monetary_impact = Some(impact);
        self
    }

    /// Adds a related entity.
    pub fn with_entity(mut self, entity: &str) -> Self {
        self.related_entities.push(entity.to_string());
        self
    }

    /// Adds metadata.
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.push((key.to_string(), value.to_string()));
        self
    }
}

/// Strategy for modifying amounts.
pub struct AmountModificationStrategy {
    /// Minimum multiplier for amount changes.
    pub min_multiplier: f64,
    /// Maximum multiplier for amount changes.
    pub max_multiplier: f64,
    /// Whether to use round numbers.
    pub prefer_round_numbers: bool,
    /// Whether to rebalance the entry after modification.
    /// If true, a corresponding line will be adjusted to maintain balance.
    /// If false, the entry will become unbalanced (for intentional fraud detection).
    pub rebalance_entry: bool,
}

impl Default for AmountModificationStrategy {
    fn default() -> Self {
        Self {
            min_multiplier: 2.0,
            max_multiplier: 10.0,
            prefer_round_numbers: false,
            rebalance_entry: true, // Default to maintaining balance
        }
    }
}

impl InjectionStrategy for AmountModificationStrategy {
    fn name(&self) -> &'static str {
        "AmountModification"
    }

    fn can_apply(&self, entry: &JournalEntry) -> bool {
        !entry.lines.is_empty()
    }

    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult {
        if entry.lines.is_empty() {
            return InjectionResult::failure("No lines to modify");
        }

        let line_idx = rng.gen_range(0..entry.lines.len());
        let is_debit = entry.lines[line_idx].debit_amount > Decimal::ZERO;
        let original_amount = if is_debit {
            entry.lines[line_idx].debit_amount
        } else {
            entry.lines[line_idx].credit_amount
        };

        let multiplier = rng.gen_range(self.min_multiplier..self.max_multiplier);
        let mut new_amount =
            original_amount * Decimal::from_f64_retain(multiplier).unwrap_or(Decimal::ONE);

        // Round to nice number if preferred
        if self.prefer_round_numbers {
            let magnitude = new_amount.to_string().len() as i32 - 2;
            let round_factor = Decimal::new(10_i64.pow(magnitude.max(0) as u32), 0);
            new_amount = (new_amount / round_factor).round() * round_factor;
        }

        let impact = new_amount - original_amount;
        let account_code = entry.lines[line_idx].account_code.clone();

        // Apply the modification
        if is_debit {
            entry.lines[line_idx].debit_amount = new_amount;
        } else {
            entry.lines[line_idx].credit_amount = new_amount;
        }

        // Rebalance the entry if configured to do so
        if self.rebalance_entry {
            // Find a line on the opposite side to adjust
            let balancing_idx = entry.lines.iter().position(|l| {
                if is_debit {
                    l.credit_amount > Decimal::ZERO
                } else {
                    l.debit_amount > Decimal::ZERO
                }
            });

            if let Some(bal_idx) = balancing_idx {
                // Adjust the balancing line by the same impact
                if is_debit {
                    entry.lines[bal_idx].credit_amount += impact;
                } else {
                    entry.lines[bal_idx].debit_amount += impact;
                }
            }
        }

        match anomaly_type {
            AnomalyType::Fraud(FraudType::RoundDollarManipulation) => {
                InjectionResult::success(&format!(
                    "Modified amount from {} to {} (round dollar){}",
                    original_amount,
                    new_amount,
                    if self.rebalance_entry {
                        " [rebalanced]"
                    } else {
                        " [UNBALANCED]"
                    }
                ))
                .with_impact(impact)
                .with_entity(&account_code)
            }
            AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount) => {
                InjectionResult::success(&format!(
                    "Inflated amount by {:.1}x to {}{}",
                    multiplier,
                    new_amount,
                    if self.rebalance_entry {
                        " [rebalanced]"
                    } else {
                        " [UNBALANCED]"
                    }
                ))
                .with_impact(impact)
                .with_metadata("multiplier", &format!("{:.2}", multiplier))
            }
            _ => InjectionResult::success(&format!(
                "Modified amount to {}{}",
                new_amount,
                if self.rebalance_entry {
                    " [rebalanced]"
                } else {
                    " [UNBALANCED]"
                }
            ))
            .with_impact(impact),
        }
    }
}

/// Strategy for modifying dates.
pub struct DateModificationStrategy {
    /// Maximum days to backdate.
    pub max_backdate_days: i64,
    /// Maximum days to future-date.
    pub max_future_days: i64,
    /// Whether to cross period boundaries.
    pub cross_period_boundary: bool,
}

impl Default for DateModificationStrategy {
    fn default() -> Self {
        Self {
            max_backdate_days: 30,
            max_future_days: 7,
            cross_period_boundary: true,
        }
    }
}

impl InjectionStrategy for DateModificationStrategy {
    fn name(&self) -> &'static str {
        "DateModification"
    }

    fn can_apply(&self, _entry: &JournalEntry) -> bool {
        true
    }

    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult {
        let original_date = entry.header.posting_date;

        let (days_offset, description) = match anomaly_type {
            AnomalyType::Error(ErrorType::BackdatedEntry) => {
                let days = rng.gen_range(1..=self.max_backdate_days);
                (-days, format!("Backdated by {} days", days))
            }
            AnomalyType::Error(ErrorType::FutureDatedEntry) => {
                let days = rng.gen_range(1..=self.max_future_days);
                (days, format!("Future-dated by {} days", days))
            }
            AnomalyType::Error(ErrorType::WrongPeriod) => {
                // Move to previous or next month
                let direction: i64 = if rng.gen_bool(0.5) { -1 } else { 1 };
                let days = direction * 32; // Ensure crossing month boundary
                (days, "Posted to wrong period".to_string())
            }
            AnomalyType::ProcessIssue(ProcessIssueType::LatePosting) => {
                let days = rng.gen_range(5..=15);
                entry.header.document_date = entry.header.posting_date; // Document date stays same
                entry.header.posting_date = original_date + chrono::Duration::days(days);
                return InjectionResult::success(&format!(
                    "Late posting: {} days after transaction",
                    days
                ))
                .with_metadata("delay_days", &days.to_string());
            }
            _ => (0, "Date unchanged".to_string()),
        };

        if days_offset != 0 {
            entry.header.posting_date = original_date + chrono::Duration::days(days_offset);
        }

        InjectionResult::success(&description)
            .with_metadata("original_date", &original_date.to_string())
            .with_metadata("new_date", &entry.header.posting_date.to_string())
    }
}

/// Strategy for document duplication.
pub struct DuplicationStrategy {
    /// Whether to modify amounts slightly.
    pub vary_amounts: bool,
    /// Amount variance factor.
    pub amount_variance: f64,
    /// Whether to change document numbers.
    pub change_doc_number: bool,
}

impl Default for DuplicationStrategy {
    fn default() -> Self {
        Self {
            vary_amounts: false,
            amount_variance: 0.01,
            change_doc_number: true,
        }
    }
}

impl DuplicationStrategy {
    /// Creates a duplicate of the entry.
    pub fn duplicate<R: Rng>(
        &self,
        entry: &JournalEntry,
        rng: &mut R,
        uuid_factory: &DeterministicUuidFactory,
    ) -> JournalEntry {
        let mut duplicate = entry.clone();

        if self.change_doc_number {
            // Generate a new UUID for the duplicate
            duplicate.header.document_id = uuid_factory.next();
            // Update line items to reference the new document ID
            for line in &mut duplicate.lines {
                line.document_id = duplicate.header.document_id;
            }
        }

        if self.vary_amounts {
            for line in &mut duplicate.lines {
                let variance = 1.0 + rng.gen_range(-self.amount_variance..self.amount_variance);
                let variance_dec = Decimal::from_f64_retain(variance).unwrap_or(Decimal::ONE);

                if line.debit_amount > Decimal::ZERO {
                    line.debit_amount = (line.debit_amount * variance_dec).round_dp(2);
                }
                if line.credit_amount > Decimal::ZERO {
                    line.credit_amount = (line.credit_amount * variance_dec).round_dp(2);
                }
            }
        }

        duplicate
    }
}

/// Strategy for approval-related anomalies.
pub struct ApprovalAnomalyStrategy {
    /// Approval threshold to target.
    pub approval_threshold: Decimal,
    /// Buffer below threshold.
    pub threshold_buffer: Decimal,
}

impl Default for ApprovalAnomalyStrategy {
    fn default() -> Self {
        Self {
            approval_threshold: Decimal::new(10000, 0),
            threshold_buffer: Decimal::new(100, 0),
        }
    }
}

impl InjectionStrategy for ApprovalAnomalyStrategy {
    fn name(&self) -> &'static str {
        "ApprovalAnomaly"
    }

    fn can_apply(&self, entry: &JournalEntry) -> bool {
        entry.total_debit() > Decimal::ZERO
    }

    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult {
        match anomaly_type {
            AnomalyType::Fraud(FraudType::JustBelowThreshold) => {
                // Set total to just below threshold
                let target = self.approval_threshold
                    - self.threshold_buffer
                    - Decimal::new(rng.gen_range(1..50), 0);

                let current_total = entry.total_debit();
                if current_total == Decimal::ZERO {
                    return InjectionResult::failure("Cannot scale zero amount");
                }

                let scale = target / current_total;
                for line in &mut entry.lines {
                    line.debit_amount = (line.debit_amount * scale).round_dp(2);
                    line.credit_amount = (line.credit_amount * scale).round_dp(2);
                }

                InjectionResult::success(&format!(
                    "Adjusted total to {} (just below threshold {})",
                    entry.total_debit(),
                    self.approval_threshold
                ))
                .with_metadata("threshold", &self.approval_threshold.to_string())
            }
            AnomalyType::Fraud(FraudType::ExceededApprovalLimit) => {
                // Set total to exceed threshold
                let target = self.approval_threshold * Decimal::new(15, 1); // 1.5x threshold

                let current_total = entry.total_debit();
                if current_total == Decimal::ZERO {
                    return InjectionResult::failure("Cannot scale zero amount");
                }

                let scale = target / current_total;
                for line in &mut entry.lines {
                    line.debit_amount = (line.debit_amount * scale).round_dp(2);
                    line.credit_amount = (line.credit_amount * scale).round_dp(2);
                }

                InjectionResult::success(&format!(
                    "Exceeded approval limit: {} vs limit {}",
                    entry.total_debit(),
                    self.approval_threshold
                ))
                .with_impact(entry.total_debit() - self.approval_threshold)
            }
            _ => InjectionResult::failure("Unsupported anomaly type for this strategy"),
        }
    }
}

/// Strategy for description/text anomalies.
pub struct DescriptionAnomalyStrategy {
    /// Vague descriptions to use.
    pub vague_descriptions: Vec<String>,
}

impl Default for DescriptionAnomalyStrategy {
    fn default() -> Self {
        Self {
            vague_descriptions: vec![
                "Misc".to_string(),
                "Adjustment".to_string(),
                "Correction".to_string(),
                "Various".to_string(),
                "Other".to_string(),
                "TBD".to_string(),
                "See attachment".to_string(),
                "As discussed".to_string(),
                "Per management".to_string(),
                ".".to_string(),
                "xxx".to_string(),
                "test".to_string(),
            ],
        }
    }
}

impl InjectionStrategy for DescriptionAnomalyStrategy {
    fn name(&self) -> &'static str {
        "DescriptionAnomaly"
    }

    fn can_apply(&self, _entry: &JournalEntry) -> bool {
        true
    }

    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        _anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult {
        let original = entry.description().unwrap_or("").to_string();
        let vague = &self.vague_descriptions[rng.gen_range(0..self.vague_descriptions.len())];
        entry.set_description(vague.clone());

        InjectionResult::success(&format!(
            "Changed description from '{}' to '{}'",
            original, vague
        ))
        .with_metadata("original_description", &original)
    }
}

/// Strategy for Benford's Law violations.
pub struct BenfordViolationStrategy {
    /// Target first digits (rarely occurring).
    pub target_digits: Vec<u32>,
    /// Whether to rebalance the entry after modification.
    pub rebalance_entry: bool,
}

impl Default for BenfordViolationStrategy {
    fn default() -> Self {
        Self {
            target_digits: vec![5, 6, 7, 8, 9], // Less common first digits
            rebalance_entry: true,              // Default to maintaining balance
        }
    }
}

impl InjectionStrategy for BenfordViolationStrategy {
    fn name(&self) -> &'static str {
        "BenfordViolation"
    }

    fn can_apply(&self, entry: &JournalEntry) -> bool {
        !entry.lines.is_empty()
    }

    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        _anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult {
        if entry.lines.is_empty() {
            return InjectionResult::failure("No lines to modify");
        }

        let line_idx = rng.gen_range(0..entry.lines.len());
        let is_debit = entry.lines[line_idx].debit_amount > Decimal::ZERO;
        let original_amount = if is_debit {
            entry.lines[line_idx].debit_amount
        } else {
            entry.lines[line_idx].credit_amount
        };

        // Get target first digit
        let target_digit = self.target_digits[rng.gen_range(0..self.target_digits.len())];

        // Calculate new amount with target first digit
        let original_str = original_amount.to_string();
        let magnitude = original_str.replace('.', "").trim_start_matches('0').len() as i32 - 1;
        // Limit magnitude to prevent overflow (10^18 is max safe for i64)
        let safe_magnitude = magnitude.clamp(0, 18) as u32;

        let base = Decimal::new(10_i64.pow(safe_magnitude), 0);
        let new_amount = base * Decimal::new(target_digit as i64, 0)
            + Decimal::new(rng.gen_range(0..10_i64.pow(safe_magnitude)), 0);

        let impact = new_amount - original_amount;

        // Apply the modification
        if is_debit {
            entry.lines[line_idx].debit_amount = new_amount;
        } else {
            entry.lines[line_idx].credit_amount = new_amount;
        }

        // Rebalance the entry if configured to do so
        if self.rebalance_entry {
            // Find a line on the opposite side to adjust
            let balancing_idx = entry.lines.iter().position(|l| {
                if is_debit {
                    l.credit_amount > Decimal::ZERO
                } else {
                    l.debit_amount > Decimal::ZERO
                }
            });

            if let Some(bal_idx) = balancing_idx {
                // Adjust the balancing line by the same impact
                if is_debit {
                    entry.lines[bal_idx].credit_amount += impact;
                } else {
                    entry.lines[bal_idx].debit_amount += impact;
                }
            }
        }

        let first_digit = target_digit;
        let benford_prob = (1.0 + 1.0 / first_digit as f64).log10();

        InjectionResult::success(&format!(
            "Created Benford violation: first digit {} (expected probability {:.1}%){}",
            first_digit,
            benford_prob * 100.0,
            if self.rebalance_entry {
                " [rebalanced]"
            } else {
                " [UNBALANCED]"
            }
        ))
        .with_impact(impact)
        .with_metadata("first_digit", &first_digit.to_string())
        .with_metadata("benford_probability", &format!("{:.4}", benford_prob))
    }
}

/// Strategy for split transactions (structuring to avoid thresholds).
pub struct SplitTransactionStrategy {
    /// Threshold above which transactions are split.
    pub split_threshold: Decimal,
    /// Number of splits to create.
    pub min_splits: usize,
    pub max_splits: usize,
    /// Buffer below threshold.
    pub threshold_buffer: Decimal,
}

impl Default for SplitTransactionStrategy {
    fn default() -> Self {
        Self {
            split_threshold: Decimal::new(10000, 0),
            min_splits: 2,
            max_splits: 5,
            threshold_buffer: Decimal::new(500, 0),
        }
    }
}

impl InjectionStrategy for SplitTransactionStrategy {
    fn name(&self) -> &'static str {
        "SplitTransaction"
    }

    fn can_apply(&self, entry: &JournalEntry) -> bool {
        // Can only split entries above threshold
        entry.total_debit() > self.split_threshold
    }

    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        _anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult {
        let total = entry.total_debit();
        if total <= self.split_threshold {
            return InjectionResult::failure("Amount below split threshold");
        }

        let num_splits = rng.gen_range(self.min_splits..=self.max_splits);
        let target_per_split =
            self.split_threshold - self.threshold_buffer - Decimal::new(rng.gen_range(1..100), 0);

        // Scale down all lines to fit first split
        let scale = target_per_split / total;
        for line in &mut entry.lines {
            line.debit_amount = (line.debit_amount * scale).round_dp(2);
            line.credit_amount = (line.credit_amount * scale).round_dp(2);
        }

        InjectionResult::success(&format!(
            "Split ${} transaction into {} parts of ~${} each (below ${} threshold)",
            total, num_splits, target_per_split, self.split_threshold
        ))
        .with_impact(total)
        .with_metadata("original_amount", &total.to_string())
        .with_metadata("num_splits", &num_splits.to_string())
        .with_metadata("threshold", &self.split_threshold.to_string())
    }
}

/// Strategy for skipped approval anomalies.
pub struct SkippedApprovalStrategy {
    /// Threshold above which approval is required.
    pub approval_threshold: Decimal,
}

impl Default for SkippedApprovalStrategy {
    fn default() -> Self {
        Self {
            approval_threshold: Decimal::new(5000, 0),
        }
    }
}

impl InjectionStrategy for SkippedApprovalStrategy {
    fn name(&self) -> &'static str {
        "SkippedApproval"
    }

    fn can_apply(&self, entry: &JournalEntry) -> bool {
        // Can only skip approval on entries above threshold
        entry.total_debit() > self.approval_threshold
    }

    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        _anomaly_type: &AnomalyType,
        _rng: &mut R,
    ) -> InjectionResult {
        let amount = entry.total_debit();
        if amount <= self.approval_threshold {
            return InjectionResult::failure("Amount below approval threshold");
        }

        // Mark control status as exception (simulates skipped approval)
        entry.header.control_status = ControlStatus::Exception;
        entry.header.sod_violation = true;

        InjectionResult::success(&format!(
            "Skipped required approval for ${} entry (threshold: ${})",
            amount, self.approval_threshold
        ))
        .with_impact(amount)
        .with_metadata("threshold", &self.approval_threshold.to_string())
    }
}

/// Strategy for weekend/holiday posting anomalies.
pub struct WeekendPostingStrategy;

impl Default for WeekendPostingStrategy {
    fn default() -> Self {
        Self
    }
}

impl InjectionStrategy for WeekendPostingStrategy {
    fn name(&self) -> &'static str {
        "WeekendPosting"
    }

    fn can_apply(&self, _entry: &JournalEntry) -> bool {
        true
    }

    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        _anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult {
        use chrono::Weekday;

        let original_date = entry.header.posting_date;
        let weekday = original_date.weekday();

        // Find days until next weekend
        let days_to_weekend = match weekday {
            Weekday::Mon => 5,
            Weekday::Tue => 4,
            Weekday::Wed => 3,
            Weekday::Thu => 2,
            Weekday::Fri => 1,
            Weekday::Sat => 0,
            Weekday::Sun => 0,
        };

        // Move to Saturday or Sunday
        let weekend_day = if rng.gen_bool(0.6) {
            days_to_weekend
        } else {
            days_to_weekend + 1
        };
        let new_date = original_date + chrono::Duration::days(weekend_day as i64);

        entry.header.posting_date = new_date;

        InjectionResult::success(&format!(
            "Moved posting from {} ({:?}) to {} ({:?})",
            original_date,
            weekday,
            new_date,
            new_date.weekday()
        ))
        .with_metadata("original_date", &original_date.to_string())
        .with_metadata("new_date", &new_date.to_string())
    }
}

/// Strategy for reversed amount errors.
pub struct ReversedAmountStrategy;

impl Default for ReversedAmountStrategy {
    fn default() -> Self {
        Self
    }
}

impl InjectionStrategy for ReversedAmountStrategy {
    fn name(&self) -> &'static str {
        "ReversedAmount"
    }

    fn can_apply(&self, entry: &JournalEntry) -> bool {
        entry.lines.len() >= 2
    }

    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        _anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult {
        if entry.lines.len() < 2 {
            return InjectionResult::failure("Need at least 2 lines to reverse");
        }

        // Pick a random line and swap its debit/credit
        let line_idx = rng.gen_range(0..entry.lines.len());
        let line = &mut entry.lines[line_idx];

        let original_debit = line.debit_amount;
        let original_credit = line.credit_amount;

        line.debit_amount = original_credit;
        line.credit_amount = original_debit;

        let impact = original_debit.max(original_credit);

        InjectionResult::success(&format!(
            "Reversed amounts on line {}: DR {} → CR {}, CR {} → DR {}",
            line_idx + 1,
            original_debit,
            line.credit_amount,
            original_credit,
            line.debit_amount
        ))
        .with_impact(impact * Decimal::new(2, 0)) // Double impact due to both sides being wrong
        .with_metadata("line_number", &(line_idx + 1).to_string())
    }
}

/// Strategy for transposed digits errors.
pub struct TransposedDigitsStrategy;

impl Default for TransposedDigitsStrategy {
    fn default() -> Self {
        Self
    }
}

impl InjectionStrategy for TransposedDigitsStrategy {
    fn name(&self) -> &'static str {
        "TransposedDigits"
    }

    fn can_apply(&self, entry: &JournalEntry) -> bool {
        // Need at least one line with amount >= 10 (two digits to transpose)
        entry.lines.iter().any(|l| {
            let amount = if l.debit_amount > Decimal::ZERO {
                l.debit_amount
            } else {
                l.credit_amount
            };
            amount >= Decimal::new(10, 0)
        })
    }

    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        _anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult {
        // Find lines with transposable amounts
        let valid_lines: Vec<usize> = entry
            .lines
            .iter()
            .enumerate()
            .filter(|(_, l)| {
                let amount = if l.debit_amount > Decimal::ZERO {
                    l.debit_amount
                } else {
                    l.credit_amount
                };
                amount >= Decimal::new(10, 0)
            })
            .map(|(i, _)| i)
            .collect();

        if valid_lines.is_empty() {
            return InjectionResult::failure("No lines with transposable amounts");
        }

        let line_idx = valid_lines[rng.gen_range(0..valid_lines.len())];
        let line = &mut entry.lines[line_idx];

        let is_debit = line.debit_amount > Decimal::ZERO;
        let original_amount = if is_debit {
            line.debit_amount
        } else {
            line.credit_amount
        };

        // Transpose two adjacent digits
        let amount_str = original_amount.to_string().replace('.', "");
        let chars: Vec<char> = amount_str.chars().collect();

        if chars.len() < 2 {
            return InjectionResult::failure("Amount too small to transpose");
        }

        // Pick a position to transpose (not the decimal point)
        let pos = rng.gen_range(0..chars.len() - 1);
        let mut new_chars = chars.clone();
        new_chars.swap(pos, pos + 1);

        let new_str: String = new_chars.into_iter().collect();
        let new_amount = new_str.parse::<i64>().unwrap_or(0);
        let scale = original_amount.scale();
        let new_decimal = Decimal::new(new_amount, scale);

        let impact = (new_decimal - original_amount).abs();

        if is_debit {
            line.debit_amount = new_decimal;
        } else {
            line.credit_amount = new_decimal;
        }

        InjectionResult::success(&format!(
            "Transposed digits: {} → {} (positions {} and {})",
            original_amount,
            new_decimal,
            pos + 1,
            pos + 2
        ))
        .with_impact(impact)
        .with_metadata("original_amount", &original_amount.to_string())
        .with_metadata("new_amount", &new_decimal.to_string())
    }
}

/// Strategy for dormant account activity.
pub struct DormantAccountStrategy {
    /// List of dormant account codes to use.
    pub dormant_accounts: Vec<String>,
}

impl Default for DormantAccountStrategy {
    fn default() -> Self {
        Self {
            dormant_accounts: vec![
                "199999".to_string(), // Suspense
                "299999".to_string(), // Legacy clearing
                "399999".to_string(), // Obsolete account
                "999999".to_string(), // Test account
            ],
        }
    }
}

impl InjectionStrategy for DormantAccountStrategy {
    fn name(&self) -> &'static str {
        "DormantAccountActivity"
    }

    fn can_apply(&self, entry: &JournalEntry) -> bool {
        !entry.lines.is_empty() && !self.dormant_accounts.is_empty()
    }

    fn apply<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        _anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult {
        if entry.lines.is_empty() || self.dormant_accounts.is_empty() {
            return InjectionResult::failure("No lines or dormant accounts");
        }

        let line_idx = rng.gen_range(0..entry.lines.len());
        let line = &mut entry.lines[line_idx];

        let original_account = line.gl_account.clone();
        let dormant_account = &self.dormant_accounts[rng.gen_range(0..self.dormant_accounts.len())];

        line.gl_account = dormant_account.clone();
        line.account_code = dormant_account.clone();

        let amount = if line.debit_amount > Decimal::ZERO {
            line.debit_amount
        } else {
            line.credit_amount
        };

        InjectionResult::success(&format!(
            "Changed account from {} to dormant account {}",
            original_account, dormant_account
        ))
        .with_impact(amount)
        .with_entity(dormant_account)
        .with_metadata("original_account", &original_account)
    }
}

/// Collection of all available strategies.
#[derive(Default)]
pub struct StrategyCollection {
    pub amount_modification: AmountModificationStrategy,
    pub date_modification: DateModificationStrategy,
    pub duplication: DuplicationStrategy,
    pub approval_anomaly: ApprovalAnomalyStrategy,
    pub description_anomaly: DescriptionAnomalyStrategy,
    pub benford_violation: BenfordViolationStrategy,
    pub split_transaction: SplitTransactionStrategy,
    pub skipped_approval: SkippedApprovalStrategy,
    pub weekend_posting: WeekendPostingStrategy,
    pub reversed_amount: ReversedAmountStrategy,
    pub transposed_digits: TransposedDigitsStrategy,
    pub dormant_account: DormantAccountStrategy,
}

impl StrategyCollection {
    /// Checks if the strategy can be applied to an entry.
    pub fn can_apply(&self, entry: &JournalEntry, anomaly_type: &AnomalyType) -> bool {
        match anomaly_type {
            // Amount-based strategies
            AnomalyType::Fraud(FraudType::RoundDollarManipulation)
            | AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount)
            | AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyLowAmount) => {
                self.amount_modification.can_apply(entry)
            }
            // Date-based strategies
            AnomalyType::Error(ErrorType::BackdatedEntry)
            | AnomalyType::Error(ErrorType::FutureDatedEntry)
            | AnomalyType::Error(ErrorType::WrongPeriod)
            | AnomalyType::ProcessIssue(ProcessIssueType::LatePosting) => {
                self.date_modification.can_apply(entry)
            }
            // Approval threshold strategies
            AnomalyType::Fraud(FraudType::JustBelowThreshold)
            | AnomalyType::Fraud(FraudType::ExceededApprovalLimit) => {
                self.approval_anomaly.can_apply(entry)
            }
            // Description strategies
            AnomalyType::ProcessIssue(ProcessIssueType::VagueDescription) => {
                self.description_anomaly.can_apply(entry)
            }
            // Benford's Law strategies
            AnomalyType::Statistical(StatisticalAnomalyType::BenfordViolation) => {
                self.benford_violation.can_apply(entry)
            }
            // Split transaction (structuring)
            AnomalyType::Fraud(FraudType::SplitTransaction) => {
                self.split_transaction.can_apply(entry)
            }
            // Skipped approval
            AnomalyType::ProcessIssue(ProcessIssueType::SkippedApproval) => {
                self.skipped_approval.can_apply(entry)
            }
            // Weekend posting
            AnomalyType::ProcessIssue(ProcessIssueType::WeekendPosting)
            | AnomalyType::ProcessIssue(ProcessIssueType::AfterHoursPosting) => {
                self.weekend_posting.can_apply(entry)
            }
            // Reversed amount
            AnomalyType::Error(ErrorType::ReversedAmount) => self.reversed_amount.can_apply(entry),
            // Transposed digits
            AnomalyType::Error(ErrorType::TransposedDigits) => {
                self.transposed_digits.can_apply(entry)
            }
            // Dormant account
            AnomalyType::Relational(RelationalAnomalyType::DormantAccountActivity) => {
                self.dormant_account.can_apply(entry)
            }
            // Default fallback
            _ => self.amount_modification.can_apply(entry),
        }
    }

    /// Applies the appropriate strategy for an anomaly type.
    pub fn apply_strategy<R: Rng>(
        &self,
        entry: &mut JournalEntry,
        anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> InjectionResult {
        match anomaly_type {
            // Amount-based strategies
            AnomalyType::Fraud(FraudType::RoundDollarManipulation)
            | AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount)
            | AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyLowAmount) => {
                self.amount_modification.apply(entry, anomaly_type, rng)
            }
            // Date-based strategies
            AnomalyType::Error(ErrorType::BackdatedEntry)
            | AnomalyType::Error(ErrorType::FutureDatedEntry)
            | AnomalyType::Error(ErrorType::WrongPeriod)
            | AnomalyType::ProcessIssue(ProcessIssueType::LatePosting) => {
                self.date_modification.apply(entry, anomaly_type, rng)
            }
            // Approval threshold strategies
            AnomalyType::Fraud(FraudType::JustBelowThreshold)
            | AnomalyType::Fraud(FraudType::ExceededApprovalLimit) => {
                self.approval_anomaly.apply(entry, anomaly_type, rng)
            }
            // Description strategies
            AnomalyType::ProcessIssue(ProcessIssueType::VagueDescription) => {
                self.description_anomaly.apply(entry, anomaly_type, rng)
            }
            // Benford's Law strategies
            AnomalyType::Statistical(StatisticalAnomalyType::BenfordViolation) => {
                self.benford_violation.apply(entry, anomaly_type, rng)
            }
            // Split transaction (structuring)
            AnomalyType::Fraud(FraudType::SplitTransaction) => {
                self.split_transaction.apply(entry, anomaly_type, rng)
            }
            // Skipped approval
            AnomalyType::ProcessIssue(ProcessIssueType::SkippedApproval) => {
                self.skipped_approval.apply(entry, anomaly_type, rng)
            }
            // Weekend posting
            AnomalyType::ProcessIssue(ProcessIssueType::WeekendPosting)
            | AnomalyType::ProcessIssue(ProcessIssueType::AfterHoursPosting) => {
                self.weekend_posting.apply(entry, anomaly_type, rng)
            }
            // Reversed amount
            AnomalyType::Error(ErrorType::ReversedAmount) => {
                self.reversed_amount.apply(entry, anomaly_type, rng)
            }
            // Transposed digits
            AnomalyType::Error(ErrorType::TransposedDigits) => {
                self.transposed_digits.apply(entry, anomaly_type, rng)
            }
            // Dormant account
            AnomalyType::Relational(RelationalAnomalyType::DormantAccountActivity) => {
                self.dormant_account.apply(entry, anomaly_type, rng)
            }
            // Default fallback
            _ => self.amount_modification.apply(entry, anomaly_type, rng),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::JournalEntryLine;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use rust_decimal_macros::dec;

    fn create_test_entry() -> JournalEntry {
        let mut entry = JournalEntry::new_simple(
            "JE001".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            "Test Entry".to_string(),
        );

        entry.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: "5000".to_string(),
            debit_amount: dec!(1000),
            ..Default::default()
        });

        entry.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: "1000".to_string(),
            credit_amount: dec!(1000),
            ..Default::default()
        });

        entry
    }

    #[test]
    fn test_amount_modification() {
        let strategy = AmountModificationStrategy::default();
        let mut entry = create_test_entry();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let result = strategy.apply(
            &mut entry,
            &AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount),
            &mut rng,
        );

        assert!(result.success);
        assert!(result.monetary_impact.is_some());
    }

    #[test]
    fn test_amount_modification_rebalanced() {
        let strategy = AmountModificationStrategy {
            rebalance_entry: true,
            ..Default::default()
        };
        let mut entry = create_test_entry();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // Entry should start balanced
        assert!(entry.is_balanced());

        let result = strategy.apply(
            &mut entry,
            &AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount),
            &mut rng,
        );

        assert!(result.success);
        // Entry should remain balanced after rebalancing
        assert!(
            entry.is_balanced(),
            "Entry should remain balanced after amount modification with rebalancing"
        );
    }

    #[test]
    fn test_amount_modification_unbalanced_fraud() {
        let strategy = AmountModificationStrategy {
            rebalance_entry: false, // Intentionally create unbalanced entry for fraud detection
            ..Default::default()
        };
        let mut entry = create_test_entry();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // Entry should start balanced
        assert!(entry.is_balanced());

        let result = strategy.apply(
            &mut entry,
            &AnomalyType::Fraud(FraudType::RoundDollarManipulation),
            &mut rng,
        );

        assert!(result.success);
        // Entry should be unbalanced when rebalance is disabled
        assert!(
            !entry.is_balanced(),
            "Entry should be unbalanced when rebalance_entry is false"
        );
    }

    #[test]
    fn test_benford_violation_rebalanced() {
        let strategy = BenfordViolationStrategy {
            rebalance_entry: true,
            ..Default::default()
        };
        let mut entry = create_test_entry();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // Entry should start balanced
        assert!(entry.is_balanced());

        let result = strategy.apply(
            &mut entry,
            &AnomalyType::Statistical(StatisticalAnomalyType::BenfordViolation),
            &mut rng,
        );

        assert!(result.success);
        // Entry should remain balanced after rebalancing
        assert!(
            entry.is_balanced(),
            "Entry should remain balanced after Benford violation with rebalancing"
        );
    }

    #[test]
    fn test_date_modification() {
        let strategy = DateModificationStrategy::default();
        let mut entry = create_test_entry();
        let original_date = entry.header.posting_date;
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let result = strategy.apply(
            &mut entry,
            &AnomalyType::Error(ErrorType::BackdatedEntry),
            &mut rng,
        );

        assert!(result.success);
        assert!(entry.header.posting_date < original_date);
    }

    #[test]
    fn test_description_anomaly() {
        let strategy = DescriptionAnomalyStrategy::default();
        let mut entry = create_test_entry();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let result = strategy.apply(
            &mut entry,
            &AnomalyType::ProcessIssue(ProcessIssueType::VagueDescription),
            &mut rng,
        );

        assert!(result.success);
        let desc = entry.description().unwrap_or("").to_string();
        assert!(strategy.vague_descriptions.contains(&desc));
    }
}
