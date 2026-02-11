//! Custom assertion macros for testing accounting invariants.

use datasynth_core::models::JournalEntry;
use rust_decimal::Decimal;

/// Assert that a journal entry is balanced (debits equal credits).
#[macro_export]
macro_rules! assert_balanced {
    ($entry:expr) => {{
        let entry = &$entry;
        let total_debits: rust_decimal::Decimal = entry.lines.iter().map(|l| l.debit_amount).sum();
        let total_credits: rust_decimal::Decimal =
            entry.lines.iter().map(|l| l.credit_amount).sum();
        assert_eq!(
            total_debits, total_credits,
            "Journal entry is not balanced: debits={}, credits={}",
            total_debits, total_credits
        );
    }};
}

/// Assert that all journal entries in a collection are balanced.
#[macro_export]
macro_rules! assert_all_balanced {
    ($entries:expr) => {{
        for (i, entry) in $entries.iter().enumerate() {
            let total_debits: rust_decimal::Decimal =
                entry.lines.iter().map(|l| l.debit_amount).sum();
            let total_credits: rust_decimal::Decimal =
                entry.lines.iter().map(|l| l.credit_amount).sum();
            assert_eq!(
                total_debits, total_credits,
                "Journal entry {} is not balanced: debits={}, credits={}",
                i, total_debits, total_credits
            );
        }
    }};
}

/// Assert that an amount follows Benford's Law distribution within tolerance.
/// This checks if the first digit distribution matches expected frequencies.
#[macro_export]
macro_rules! assert_benford_compliant {
    ($amounts:expr, $tolerance:expr) => {{
        let amounts = &$amounts;
        let expected = [0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046];
        let mut counts = [0u64; 9];
        let mut total = 0u64;

        for amount in amounts.iter() {
            if *amount > rust_decimal::Decimal::ZERO {
                let first_digit = amount
                    .to_string()
                    .chars()
                    .find(|c| c.is_ascii_digit() && *c != '0')
                    .map(|c| c.to_digit(10).unwrap() as usize);

                if let Some(d) = first_digit {
                    if d >= 1 && d <= 9 {
                        counts[d - 1] += 1;
                        total += 1;
                    }
                }
            }
        }

        if total > 0 {
            for (i, (count, exp)) in counts.iter().zip(expected.iter()).enumerate() {
                let observed = *count as f64 / total as f64;
                let diff = (observed - exp).abs();
                assert!(
                    diff < $tolerance,
                    "Benford's Law violation for digit {}: observed={:.4}, expected={:.4}, diff={:.4}",
                    i + 1,
                    observed,
                    exp,
                    diff
                );
            }
        }
    }};
}

/// Check if a journal entry is balanced.
pub fn is_balanced(entry: &JournalEntry) -> bool {
    let total_debits: Decimal = entry.lines.iter().map(|l| l.debit_amount).sum();
    let total_credits: Decimal = entry.lines.iter().map(|l| l.credit_amount).sum();
    total_debits == total_credits
}

/// Calculate the imbalance of a journal entry.
pub fn calculate_imbalance(entry: &JournalEntry) -> Decimal {
    let total_debits: Decimal = entry.lines.iter().map(|l| l.debit_amount).sum();
    let total_credits: Decimal = entry.lines.iter().map(|l| l.credit_amount).sum();
    total_debits - total_credits
}

/// Check if amounts follow Benford's Law distribution.
/// Returns the chi-squared statistic and whether it passes the test at p < 0.05.
pub fn check_benford_distribution(amounts: &[Decimal]) -> (f64, bool) {
    let expected = [
        0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046,
    ];
    let mut counts = [0u64; 9];
    let mut total = 0u64;

    for amount in amounts.iter() {
        if *amount > Decimal::ZERO {
            let first_digit = amount
                .to_string()
                .chars()
                .find(|c| c.is_ascii_digit() && *c != '0')
                .map(|c| c.to_digit(10).unwrap() as usize);

            if let Some(d) = first_digit {
                if (1..=9).contains(&d) {
                    counts[d - 1] += 1;
                    total += 1;
                }
            }
        }
    }

    if total == 0 {
        return (0.0, true);
    }

    // Calculate chi-squared statistic
    let mut chi_squared = 0.0;
    for (count, exp) in counts.iter().zip(expected.iter()) {
        let expected_count = exp * total as f64;
        if expected_count > 0.0 {
            let diff = *count as f64 - expected_count;
            chi_squared += diff * diff / expected_count;
        }
    }

    // Critical value for chi-squared with 8 degrees of freedom at p < 0.05 is 15.507
    // At p < 0.01 is 20.090
    let passes = chi_squared < 20.090;

    (chi_squared, passes)
}

/// Check that the accounting equation holds: Assets = Liabilities + Equity
pub fn check_accounting_equation(
    total_assets: Decimal,
    total_liabilities: Decimal,
    total_equity: Decimal,
) -> bool {
    total_assets == total_liabilities + total_equity
}

/// Verify trial balance is balanced (total debits = total credits).
pub fn check_trial_balance(debit_balances: &[Decimal], credit_balances: &[Decimal]) -> bool {
    let total_debits: Decimal = debit_balances.iter().copied().sum();
    let total_credits: Decimal = credit_balances.iter().copied().sum();
    total_debits == total_credits
}

// =============================================================================
// Enhanced Test Assertions
// =============================================================================

/// Assert that amounts pass Benford's Law chi-squared test.
/// Uses the chi-squared statistic with configurable threshold.
#[macro_export]
macro_rules! assert_benford_passes {
    ($amounts:expr, $threshold:expr) => {{
        let (chi_squared, passes) = $crate::assertions::check_benford_distribution(&$amounts);
        assert!(
            passes || chi_squared < $threshold,
            "Benford's Law test failed: chi-squared={:.4}, threshold={}",
            chi_squared,
            $threshold
        );
    }};
    ($amounts:expr) => {{
        let (chi_squared, passes) = $crate::assertions::check_benford_distribution(&$amounts);
        assert!(
            passes,
            "Benford's Law test failed: chi-squared={:.4}, p < 0.01 threshold=20.090",
            chi_squared
        );
    }};
}

/// Balance snapshot for coherence testing.
#[derive(Debug, Clone)]
pub struct BalanceSnapshot {
    /// Total assets
    pub assets: Decimal,
    /// Total liabilities
    pub liabilities: Decimal,
    /// Total equity
    pub equity: Decimal,
    /// Period identifier
    pub period: String,
}

impl BalanceSnapshot {
    /// Create a new balance snapshot.
    pub fn new(assets: Decimal, liabilities: Decimal, equity: Decimal, period: &str) -> Self {
        Self {
            assets,
            liabilities,
            equity,
            period: period.into(),
        }
    }

    /// Check if the accounting equation holds within tolerance.
    pub fn is_coherent(&self, tolerance: Decimal) -> bool {
        let diff = self.assets - (self.liabilities + self.equity);
        diff.abs() <= tolerance
    }
}

/// Assert that balance snapshots maintain accounting equation coherence.
/// Checks that Assets = Liabilities + Equity within tolerance.
#[macro_export]
macro_rules! assert_balance_coherent {
    ($snapshots:expr, $tolerance:expr) => {{
        let tolerance =
            rust_decimal::Decimal::try_from($tolerance).unwrap_or(rust_decimal::Decimal::ZERO);
        for snapshot in $snapshots.iter() {
            assert!(
                snapshot.is_coherent(tolerance),
                "Balance not coherent for period {}: assets={}, liabilities={}, equity={}, diff={}",
                snapshot.period,
                snapshot.assets,
                snapshot.liabilities,
                snapshot.equity,
                snapshot.assets - (snapshot.liabilities + snapshot.equity)
            );
        }
    }};
}

/// Subledger reconciliation data.
#[derive(Debug, Clone)]
pub struct SubledgerReconciliation {
    /// Subledger name (AR, AP, FA, Inventory)
    pub subledger: String,
    /// Total from subledger
    pub subledger_total: Decimal,
    /// GL control account balance
    pub gl_balance: Decimal,
    /// Period
    pub period: String,
}

impl SubledgerReconciliation {
    /// Create new reconciliation data.
    pub fn new(
        subledger: &str,
        subledger_total: Decimal,
        gl_balance: Decimal,
        period: &str,
    ) -> Self {
        Self {
            subledger: subledger.into(),
            subledger_total,
            gl_balance,
            period: period.into(),
        }
    }

    /// Check if subledger reconciles to GL within tolerance.
    pub fn is_reconciled(&self, tolerance: Decimal) -> bool {
        let diff = (self.subledger_total - self.gl_balance).abs();
        diff <= tolerance
    }

    /// Get the reconciliation difference.
    pub fn difference(&self) -> Decimal {
        self.subledger_total - self.gl_balance
    }
}

/// Assert that subledgers reconcile to GL control accounts.
#[macro_export]
macro_rules! assert_subledger_reconciled {
    ($reconciliations:expr, $tolerance:expr) => {{
        let tolerance =
            rust_decimal::Decimal::try_from($tolerance).unwrap_or(rust_decimal::Decimal::ZERO);
        for recon in $reconciliations.iter() {
            assert!(
                recon.is_reconciled(tolerance),
                "Subledger {} not reconciled for period {}: subledger={}, gl={}, diff={}",
                recon.subledger,
                recon.period,
                recon.subledger_total,
                recon.gl_balance,
                recon.difference()
            );
        }
    }};
}

/// Document chain validation result.
#[derive(Debug, Clone)]
pub struct DocumentChainResult {
    /// Chain identifier
    pub chain_id: String,
    /// Whether chain is complete
    pub is_complete: bool,
    /// Missing steps (if any)
    pub missing_steps: Vec<String>,
    /// Total steps expected
    pub expected_steps: usize,
    /// Actual steps found
    pub actual_steps: usize,
}

impl DocumentChainResult {
    /// Create a new chain result.
    pub fn new(chain_id: &str, expected_steps: usize, actual_steps: usize) -> Self {
        Self {
            chain_id: chain_id.into(),
            is_complete: actual_steps >= expected_steps,
            missing_steps: Vec::new(),
            expected_steps,
            actual_steps,
        }
    }

    /// Create a complete chain result.
    pub fn complete(chain_id: &str, steps: usize) -> Self {
        Self::new(chain_id, steps, steps)
    }

    /// Create an incomplete chain result.
    pub fn incomplete(
        chain_id: &str,
        expected: usize,
        actual: usize,
        missing: Vec<String>,
    ) -> Self {
        Self {
            chain_id: chain_id.into(),
            is_complete: false,
            missing_steps: missing,
            expected_steps: expected,
            actual_steps: actual,
        }
    }

    /// Get completion rate.
    pub fn completion_rate(&self) -> f64 {
        if self.expected_steps == 0 {
            1.0
        } else {
            self.actual_steps as f64 / self.expected_steps as f64
        }
    }
}

/// Check document chain completeness rate.
pub fn check_document_chain_completeness(chains: &[DocumentChainResult]) -> (f64, usize, usize) {
    if chains.is_empty() {
        return (1.0, 0, 0);
    }

    let complete_count = chains.iter().filter(|c| c.is_complete).count();
    let total_count = chains.len();
    let rate = complete_count as f64 / total_count as f64;

    (rate, complete_count, total_count)
}

/// Assert that document chains meet completeness threshold.
#[macro_export]
macro_rules! assert_document_chain_complete {
    ($chains:expr, $threshold:expr) => {{
        let (rate, complete, total) =
            $crate::assertions::check_document_chain_completeness(&$chains);
        assert!(
            rate >= $threshold,
            "Document chain completeness {:.2}% below threshold {:.2}%: {}/{} complete",
            rate * 100.0,
            $threshold * 100.0,
            complete,
            total
        );

        // Also report incomplete chains for debugging
        for chain in $chains.iter().filter(|c| !c.is_complete) {
            eprintln!(
                "Incomplete chain {}: {}/{} steps, missing: {:?}",
                chain.chain_id, chain.actual_steps, chain.expected_steps, chain.missing_steps
            );
        }
    }};
}

/// Fidelity comparison result.
#[derive(Debug, Clone)]
pub struct FidelityResult {
    /// Overall fidelity score (0.0 - 1.0)
    pub overall_score: f64,
    /// Statistical fidelity (distribution similarity)
    pub statistical_score: f64,
    /// Schema fidelity (structure match)
    pub schema_score: f64,
    /// Correlation fidelity (relationship preservation)
    pub correlation_score: f64,
    /// Whether fidelity passes threshold
    pub passes: bool,
    /// Threshold used
    pub threshold: f64,
}

impl FidelityResult {
    /// Create a new fidelity result.
    pub fn new(statistical: f64, schema: f64, correlation: f64, threshold: f64) -> Self {
        // Weighted average: statistical 50%, schema 25%, correlation 25%
        let overall = statistical * 0.50 + schema * 0.25 + correlation * 0.25;

        Self {
            overall_score: overall,
            statistical_score: statistical,
            schema_score: schema,
            correlation_score: correlation,
            passes: overall >= threshold,
            threshold,
        }
    }

    /// Create a perfect fidelity result (for self-comparison).
    pub fn perfect(threshold: f64) -> Self {
        Self::new(1.0, 1.0, 1.0, threshold)
    }
}

/// Check fidelity between synthetic data and fingerprint.
pub fn check_fidelity(
    statistical_score: f64,
    schema_score: f64,
    correlation_score: f64,
    threshold: f64,
) -> FidelityResult {
    FidelityResult::new(
        statistical_score,
        schema_score,
        correlation_score,
        threshold,
    )
}

/// Assert that fidelity passes the threshold.
#[macro_export]
macro_rules! assert_fidelity_passes {
    ($result:expr) => {{
        assert!(
            $result.passes,
            "Fidelity check failed: overall={:.4} < threshold={:.4}\n  \
             statistical={:.4}, schema={:.4}, correlation={:.4}",
            $result.overall_score,
            $result.threshold,
            $result.statistical_score,
            $result.schema_score,
            $result.correlation_score
        );
    }};
    ($statistical:expr, $schema:expr, $correlation:expr, $threshold:expr) => {{
        let result =
            $crate::assertions::check_fidelity($statistical, $schema, $correlation, $threshold);
        assert!(
            result.passes,
            "Fidelity check failed: overall={:.4} < threshold={:.4}\n  \
             statistical={:.4}, schema={:.4}, correlation={:.4}",
            result.overall_score,
            result.threshold,
            result.statistical_score,
            result.schema_score,
            result.correlation_score
        );
    }};
}

/// Convenience function to compute Mean Absolute Deviation for Benford analysis.
pub fn benford_mad(amounts: &[Decimal]) -> f64 {
    let expected = [
        0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046,
    ];
    let mut counts = [0u64; 9];
    let mut total = 0u64;

    for amount in amounts.iter() {
        if *amount > Decimal::ZERO {
            let first_digit = amount
                .to_string()
                .chars()
                .find(|c| c.is_ascii_digit() && *c != '0')
                .and_then(|c| c.to_digit(10))
                .map(|d| d as usize);

            if let Some(d) = first_digit {
                if (1..=9).contains(&d) {
                    counts[d - 1] += 1;
                    total += 1;
                }
            }
        }
    }

    if total == 0 {
        return 0.0;
    }

    // Calculate Mean Absolute Deviation
    let mut mad = 0.0;
    for (count, exp) in counts.iter().zip(expected.iter()) {
        let observed = *count as f64 / total as f64;
        mad += (observed - exp).abs();
    }

    mad / 9.0
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::fixtures::*;

    #[test]
    fn test_is_balanced() {
        let entry = balanced_journal_entry(Decimal::new(10000, 2));
        assert!(is_balanced(&entry));
    }

    #[test]
    fn test_is_not_balanced() {
        let entry = unbalanced_journal_entry();
        assert!(!is_balanced(&entry));
    }

    #[test]
    fn test_calculate_imbalance_balanced() {
        let entry = balanced_journal_entry(Decimal::new(10000, 2));
        assert_eq!(calculate_imbalance(&entry), Decimal::ZERO);
    }

    #[test]
    fn test_calculate_imbalance_unbalanced() {
        let entry = unbalanced_journal_entry();
        let imbalance = calculate_imbalance(&entry);
        assert_ne!(imbalance, Decimal::ZERO);
    }

    #[test]
    fn test_check_accounting_equation() {
        // Assets = 1000, Liabilities = 600, Equity = 400
        assert!(check_accounting_equation(
            Decimal::new(1000, 0),
            Decimal::new(600, 0),
            Decimal::new(400, 0)
        ));

        // Unbalanced: Assets = 1000, Liabilities = 600, Equity = 300
        assert!(!check_accounting_equation(
            Decimal::new(1000, 0),
            Decimal::new(600, 0),
            Decimal::new(300, 0)
        ));
    }

    #[test]
    fn test_check_trial_balance() {
        let debits = vec![Decimal::new(1000, 0), Decimal::new(500, 0)];
        let credits = vec![Decimal::new(1500, 0)];
        assert!(check_trial_balance(&debits, &credits));

        let unbalanced_credits = vec![Decimal::new(1000, 0)];
        assert!(!check_trial_balance(&debits, &unbalanced_credits));
    }

    #[test]
    fn test_benford_distribution_perfect() {
        // Create a distribution that follows Benford's Law
        let mut amounts = Vec::new();
        let expected_counts = [301, 176, 125, 97, 79, 67, 58, 51, 46]; // Per 1000

        for (digit, count) in expected_counts.iter().enumerate() {
            let base = Decimal::new((digit + 1) as i64, 0);
            for _ in 0..*count {
                amounts.push(base);
            }
        }

        let (chi_squared, passes) = check_benford_distribution(&amounts);
        assert!(passes, "Chi-squared: {}", chi_squared);
    }

    #[test]
    fn test_assert_balanced_macro() {
        let entry = balanced_journal_entry(Decimal::new(10000, 2));
        assert_balanced!(entry); // Should not panic
    }

    #[test]
    fn test_assert_all_balanced_macro() {
        let entries = [
            balanced_journal_entry(Decimal::new(10000, 2)),
            balanced_journal_entry(Decimal::new(20000, 2)),
            balanced_journal_entry(Decimal::new(30000, 2)),
        ];
        assert_all_balanced!(entries); // Should not panic
    }

    // =============================================================================
    // Tests for new enhanced assertions
    // =============================================================================

    #[test]
    fn test_balance_snapshot_coherent() {
        let snapshot = BalanceSnapshot::new(
            Decimal::new(1000, 0),
            Decimal::new(600, 0),
            Decimal::new(400, 0),
            "2025-01",
        );
        assert!(snapshot.is_coherent(Decimal::ZERO));
    }

    #[test]
    fn test_balance_snapshot_incoherent() {
        let snapshot = BalanceSnapshot::new(
            Decimal::new(1000, 0),
            Decimal::new(600, 0),
            Decimal::new(300, 0), // Assets != L + E
            "2025-01",
        );
        assert!(!snapshot.is_coherent(Decimal::ZERO));
    }

    #[test]
    fn test_balance_snapshot_with_tolerance() {
        let snapshot = BalanceSnapshot::new(
            Decimal::new(1001, 0), // Off by 1
            Decimal::new(600, 0),
            Decimal::new(400, 0),
            "2025-01",
        );
        assert!(!snapshot.is_coherent(Decimal::ZERO));
        assert!(snapshot.is_coherent(Decimal::new(1, 0)));
        assert!(snapshot.is_coherent(Decimal::new(5, 0)));
    }

    #[test]
    fn test_assert_balance_coherent_macro() {
        let snapshots = [
            BalanceSnapshot::new(
                Decimal::new(1000, 0),
                Decimal::new(600, 0),
                Decimal::new(400, 0),
                "2025-01",
            ),
            BalanceSnapshot::new(
                Decimal::new(1200, 0),
                Decimal::new(700, 0),
                Decimal::new(500, 0),
                "2025-02",
            ),
        ];
        assert_balance_coherent!(snapshots, 0.0);
    }

    #[test]
    fn test_subledger_reconciliation() {
        let recon = SubledgerReconciliation::new(
            "AR",
            Decimal::new(50000, 0),
            Decimal::new(50000, 0),
            "2025-01",
        );
        assert!(recon.is_reconciled(Decimal::ZERO));
        assert_eq!(recon.difference(), Decimal::ZERO);
    }

    #[test]
    fn test_subledger_reconciliation_with_tolerance() {
        let recon = SubledgerReconciliation::new(
            "AP",
            Decimal::new(50010, 0), // Off by 10
            Decimal::new(50000, 0),
            "2025-01",
        );
        assert!(!recon.is_reconciled(Decimal::new(5, 0)));
        assert!(recon.is_reconciled(Decimal::new(10, 0)));
        assert!(recon.is_reconciled(Decimal::new(100, 0)));
    }

    #[test]
    fn test_assert_subledger_reconciled_macro() {
        let reconciliations = [
            SubledgerReconciliation::new(
                "AR",
                Decimal::new(50000, 0),
                Decimal::new(50000, 0),
                "2025-01",
            ),
            SubledgerReconciliation::new(
                "AP",
                Decimal::new(30000, 0),
                Decimal::new(30000, 0),
                "2025-01",
            ),
        ];
        assert_subledger_reconciled!(reconciliations, 0.0);
    }

    #[test]
    fn test_document_chain_complete() {
        let chain = DocumentChainResult::complete("PO-001", 5);
        assert!(chain.is_complete);
        assert_eq!(chain.completion_rate(), 1.0);
    }

    #[test]
    fn test_document_chain_incomplete() {
        let chain =
            DocumentChainResult::incomplete("PO-002", 5, 3, vec!["Payment".into(), "Close".into()]);
        assert!(!chain.is_complete);
        assert_eq!(chain.completion_rate(), 0.6);
    }

    #[test]
    fn test_check_document_chain_completeness() {
        let chains = vec![
            DocumentChainResult::complete("PO-001", 5),
            DocumentChainResult::complete("PO-002", 5),
            DocumentChainResult::incomplete("PO-003", 5, 3, vec!["Payment".into()]),
        ];

        let (rate, complete, total) = check_document_chain_completeness(&chains);
        assert_eq!(complete, 2);
        assert_eq!(total, 3);
        assert!((rate - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_assert_document_chain_complete_macro() {
        let chains = vec![
            DocumentChainResult::complete("PO-001", 5),
            DocumentChainResult::complete("PO-002", 5),
            DocumentChainResult::complete("PO-003", 5),
        ];
        assert_document_chain_complete!(chains, 0.9);
    }

    #[test]
    fn test_fidelity_result() {
        let result = FidelityResult::new(0.95, 1.0, 0.90, 0.80);

        // Weighted: 0.95 * 0.5 + 1.0 * 0.25 + 0.90 * 0.25 = 0.475 + 0.25 + 0.225 = 0.95
        assert!((result.overall_score - 0.95).abs() < 0.001);
        assert!(result.passes);
    }

    #[test]
    fn test_fidelity_result_fails() {
        let result = FidelityResult::new(0.50, 0.50, 0.50, 0.80);

        // Weighted: 0.50 * 0.5 + 0.50 * 0.25 + 0.50 * 0.25 = 0.25 + 0.125 + 0.125 = 0.50
        assert!((result.overall_score - 0.50).abs() < 0.001);
        assert!(!result.passes);
    }

    #[test]
    fn test_fidelity_perfect() {
        let result = FidelityResult::perfect(0.90);
        assert_eq!(result.overall_score, 1.0);
        assert!(result.passes);
    }

    #[test]
    fn test_assert_fidelity_passes_macro() {
        let result = FidelityResult::new(0.95, 1.0, 0.90, 0.80);
        assert_fidelity_passes!(result);
    }

    #[test]
    fn test_assert_fidelity_passes_inline() {
        assert_fidelity_passes!(0.95, 1.0, 0.90, 0.80);
    }

    #[test]
    fn test_benford_mad() {
        // Create a perfect Benford distribution
        let mut amounts = Vec::new();
        let expected_counts = [301, 176, 125, 97, 79, 67, 58, 51, 46];

        for (digit, count) in expected_counts.iter().enumerate() {
            let base = Decimal::new((digit + 1) as i64, 0);
            for _ in 0..*count {
                amounts.push(base);
            }
        }

        let mad = benford_mad(&amounts);
        assert!(
            mad < 0.01,
            "Perfect Benford distribution should have very low MAD: {}",
            mad
        );
    }

    #[test]
    fn test_benford_mad_uniform() {
        // Create a uniform distribution (bad for Benford)
        let mut amounts = Vec::new();
        for digit in 1..=9 {
            for _ in 0..100 {
                amounts.push(Decimal::new(digit, 0));
            }
        }

        let mad = benford_mad(&amounts);
        assert!(
            mad > 0.02,
            "Uniform distribution should have high MAD: {}",
            mad
        );
    }
}
