//! Intercompany matching engine.
//!
//! Matches IC balances between entities and identifies discrepancies
//! for reconciliation and elimination.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

use datasynth_core::models::intercompany::{
    ICAggregatedBalance, ICMatchedPair, ICNettingArrangement, ICNettingPosition,
};
use datasynth_core::models::JournalEntry;

/// Result of IC matching process.
#[derive(Debug, Clone)]
pub struct ICMatchingResult {
    /// Matched balances (zero difference).
    pub matched_balances: Vec<ICAggregatedBalance>,
    /// Unmatched balances (non-zero difference).
    pub unmatched_balances: Vec<ICAggregatedBalance>,
    /// Total matched amount.
    pub total_matched: Decimal,
    /// Total unmatched amount.
    pub total_unmatched: Decimal,
    /// Match rate (0.0 to 1.0).
    pub match_rate: f64,
    /// As-of date.
    pub as_of_date: NaiveDate,
    /// Tolerance used for matching.
    pub tolerance: Decimal,
}

/// Configuration for IC matching.
#[derive(Debug, Clone)]
pub struct ICMatchingConfig {
    /// Tolerance for matching (amounts within this are considered matched).
    pub tolerance: Decimal,
    /// Match by IC reference (exact match).
    pub match_by_reference: bool,
    /// Match by amount (fuzzy match).
    pub match_by_amount: bool,
    /// Match by date range.
    pub date_range_days: i64,
    /// Auto-create adjustment entries for small differences.
    pub auto_adjust_threshold: Decimal,
    /// Currency for matching.
    pub base_currency: String,
}

impl Default for ICMatchingConfig {
    fn default() -> Self {
        Self {
            tolerance: dec!(0.01),
            match_by_reference: true,
            match_by_amount: true,
            date_range_days: 5,
            auto_adjust_threshold: dec!(100),
            base_currency: "USD".to_string(),
        }
    }
}

/// IC Matching Engine for reconciliation.
pub struct ICMatchingEngine {
    /// Configuration.
    config: ICMatchingConfig,
    /// IC balances by company pair.
    balances: HashMap<(String, String), ICAggregatedBalance>,
    /// Unmatched items by company.
    unmatched_items: HashMap<String, Vec<UnmatchedItem>>,
    /// Matching results history.
    matching_history: Vec<ICMatchingResult>,
}

impl ICMatchingEngine {
    /// Create a new matching engine.
    pub fn new(config: ICMatchingConfig) -> Self {
        Self {
            config,
            balances: HashMap::new(),
            unmatched_items: HashMap::new(),
            matching_history: Vec::new(),
        }
    }

    /// Add a receivable entry to the engine.
    pub fn add_receivable(
        &mut self,
        creditor: &str,
        debtor: &str,
        amount: Decimal,
        ic_reference: Option<&str>,
        date: NaiveDate,
    ) {
        let key = (creditor.to_string(), debtor.to_string());
        let balance = self.balances.entry(key.clone()).or_insert_with(|| {
            ICAggregatedBalance::new(
                creditor.to_string(),
                debtor.to_string(),
                format!("1310{}", &debtor[..debtor.len().min(2)]),
                format!("2110{}", &creditor[..creditor.len().min(2)]),
                self.config.base_currency.clone(),
                date,
            )
        });

        balance.receivable_balance += amount;
        balance.set_balances(balance.receivable_balance, balance.payable_balance);

        // Track for detailed matching
        self.unmatched_items
            .entry(creditor.to_string())
            .or_default()
            .push(UnmatchedItem {
                company: creditor.to_string(),
                counterparty: debtor.to_string(),
                amount,
                is_receivable: true,
                ic_reference: ic_reference.map(|s| s.to_string()),
                date,
                matched: false,
            });
    }

    /// Add a payable entry to the engine.
    pub fn add_payable(
        &mut self,
        debtor: &str,
        creditor: &str,
        amount: Decimal,
        ic_reference: Option<&str>,
        date: NaiveDate,
    ) {
        let key = (creditor.to_string(), debtor.to_string());
        let balance = self.balances.entry(key.clone()).or_insert_with(|| {
            ICAggregatedBalance::new(
                creditor.to_string(),
                debtor.to_string(),
                format!("1310{}", &debtor[..debtor.len().min(2)]),
                format!("2110{}", &creditor[..creditor.len().min(2)]),
                self.config.base_currency.clone(),
                date,
            )
        });

        balance.payable_balance += amount;
        balance.set_balances(balance.receivable_balance, balance.payable_balance);

        // Track for detailed matching
        self.unmatched_items
            .entry(debtor.to_string())
            .or_default()
            .push(UnmatchedItem {
                company: debtor.to_string(),
                counterparty: creditor.to_string(),
                amount,
                is_receivable: false,
                ic_reference: ic_reference.map(|s| s.to_string()),
                date,
                matched: false,
            });
    }

    /// Load matched pairs into the engine.
    pub fn load_matched_pairs(&mut self, pairs: &[ICMatchedPair]) {
        for pair in pairs {
            self.add_receivable(
                &pair.seller_company,
                &pair.buyer_company,
                pair.amount,
                Some(&pair.ic_reference),
                pair.transaction_date,
            );
            self.add_payable(
                &pair.buyer_company,
                &pair.seller_company,
                pair.amount,
                Some(&pair.ic_reference),
                pair.transaction_date,
            );
        }
    }

    /// Load journal entries and extract IC items.
    pub fn load_journal_entries(&mut self, entries: &[JournalEntry]) {
        for entry in entries {
            // Look for IC accounts in journal lines
            for line in &entry.lines {
                // Check for IC receivable accounts (1310xx pattern)
                if line.account_code.starts_with("1310") && line.debit_amount > Decimal::ZERO {
                    // Extract counterparty from account code
                    let counterparty = line.account_code[4..].to_string();
                    self.add_receivable(
                        entry.company_code(),
                        &counterparty,
                        line.debit_amount,
                        entry.header.reference.as_deref(),
                        entry.posting_date(),
                    );
                }

                // Check for IC payable accounts (2110xx pattern)
                if line.account_code.starts_with("2110") && line.credit_amount > Decimal::ZERO {
                    let counterparty = line.account_code[4..].to_string();
                    self.add_payable(
                        entry.company_code(),
                        &counterparty,
                        line.credit_amount,
                        entry.header.reference.as_deref(),
                        entry.posting_date(),
                    );
                }
            }
        }
    }

    /// Perform matching process.
    pub fn run_matching(&mut self, as_of_date: NaiveDate) -> ICMatchingResult {
        let mut matched_balances = Vec::new();
        let mut unmatched_balances = Vec::new();
        let mut total_matched = Decimal::ZERO;
        let mut total_unmatched = Decimal::ZERO;

        // First pass: match by IC reference
        if self.config.match_by_reference {
            self.match_by_reference();
        }

        // Second pass: match by amount
        if self.config.match_by_amount {
            self.match_by_amount();
        }

        // Evaluate results
        for balance in self.balances.values() {
            if balance.difference.abs() <= self.config.tolerance {
                matched_balances.push(balance.clone());
                total_matched += balance.elimination_amount();
            } else {
                unmatched_balances.push(balance.clone());
                total_unmatched += balance.difference.abs();
            }
        }

        let total_items = matched_balances.len() + unmatched_balances.len();
        let match_rate = if total_items > 0 {
            matched_balances.len() as f64 / total_items as f64
        } else {
            1.0
        };

        let result = ICMatchingResult {
            matched_balances,
            unmatched_balances,
            total_matched,
            total_unmatched,
            match_rate,
            as_of_date,
            tolerance: self.config.tolerance,
        };

        self.matching_history.push(result.clone());
        result
    }

    /// Match items by IC reference.
    fn match_by_reference(&mut self) {
        // Collect match candidates: (company, item_idx, counterparty, cp_item_idx)
        let mut matches_to_apply: Vec<(String, usize, String, usize)> = Vec::new();

        let companies: Vec<String> = self.unmatched_items.keys().cloned().collect();
        let tolerance = self.config.tolerance;

        for company in &companies {
            if let Some(items) = self.unmatched_items.get(company) {
                for (item_idx, item) in items.iter().enumerate() {
                    if item.matched || item.ic_reference.is_none() {
                        continue;
                    }

                    let ic_ref = item.ic_reference.as_ref().expect("checked is_none above");

                    // Look for matching item in counterparty
                    if let Some(counterparty_items) = self.unmatched_items.get(&item.counterparty) {
                        for (cp_idx, cp_item) in counterparty_items.iter().enumerate() {
                            if cp_item.matched {
                                continue;
                            }

                            if cp_item.ic_reference.as_ref() == Some(ic_ref)
                                && cp_item.counterparty == *company
                                && cp_item.is_receivable != item.is_receivable
                                && (cp_item.amount - item.amount).abs() <= tolerance
                            {
                                matches_to_apply.push((
                                    company.clone(),
                                    item_idx,
                                    item.counterparty.clone(),
                                    cp_idx,
                                ));
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Apply matches
        for (company, item_idx, counterparty, cp_idx) in matches_to_apply {
            if let Some(items) = self.unmatched_items.get_mut(&company) {
                if let Some(item) = items.get_mut(item_idx) {
                    item.matched = true;
                }
            }
            if let Some(cp_items) = self.unmatched_items.get_mut(&counterparty) {
                if let Some(cp_item) = cp_items.get_mut(cp_idx) {
                    cp_item.matched = true;
                }
            }
        }
    }

    /// Match items by amount.
    fn match_by_amount(&mut self) {
        // Collect match candidates: (company, item_idx, counterparty, cp_item_idx)
        let mut matches_to_apply: Vec<(String, usize, String, usize)> = Vec::new();

        let companies: Vec<String> = self.unmatched_items.keys().cloned().collect();
        let tolerance = self.config.tolerance;
        let date_range_days = self.config.date_range_days;

        for company in &companies {
            if let Some(items) = self.unmatched_items.get(company) {
                for (item_idx, item) in items.iter().enumerate() {
                    if item.matched {
                        continue;
                    }

                    // Look for matching amount in counterparty
                    if let Some(counterparty_items) = self.unmatched_items.get(&item.counterparty) {
                        for (cp_idx, cp_item) in counterparty_items.iter().enumerate() {
                            if cp_item.matched {
                                continue;
                            }

                            if cp_item.counterparty == *company
                                && cp_item.is_receivable != item.is_receivable
                                && (cp_item.amount - item.amount).abs() <= tolerance
                            {
                                // Check date range
                                let date_diff = (cp_item.date - item.date).num_days().abs();
                                if date_diff <= date_range_days {
                                    matches_to_apply.push((
                                        company.clone(),
                                        item_idx,
                                        item.counterparty.clone(),
                                        cp_idx,
                                    ));
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Apply matches
        for (company, item_idx, counterparty, cp_idx) in matches_to_apply {
            if let Some(items) = self.unmatched_items.get_mut(&company) {
                if let Some(item) = items.get_mut(item_idx) {
                    item.matched = true;
                }
            }
            if let Some(cp_items) = self.unmatched_items.get_mut(&counterparty) {
                if let Some(cp_item) = cp_items.get_mut(cp_idx) {
                    cp_item.matched = true;
                }
            }
        }
    }

    /// Get aggregated balances.
    pub fn get_balances(&self) -> Vec<&ICAggregatedBalance> {
        self.balances.values().collect()
    }

    /// Get unmatched balances.
    pub fn get_unmatched_balances(&self) -> Vec<&ICAggregatedBalance> {
        self.balances.values().filter(|b| !b.is_matched).collect()
    }

    /// Get balance for a specific company pair.
    pub fn get_balance(&self, creditor: &str, debtor: &str) -> Option<&ICAggregatedBalance> {
        self.balances
            .get(&(creditor.to_string(), debtor.to_string()))
    }

    /// Generate netting arrangement.
    pub fn generate_netting(
        &self,
        companies: Vec<String>,
        period_start: NaiveDate,
        period_end: NaiveDate,
        settlement_date: NaiveDate,
    ) -> ICNettingArrangement {
        let netting_ref = format!("NET{}", settlement_date.format("%Y%m%d"));

        let mut arrangement = ICNettingArrangement::new(
            netting_ref,
            companies.clone(),
            period_start,
            period_end,
            settlement_date,
            self.config.base_currency.clone(),
        );

        // Calculate gross positions for each company
        for company in &companies {
            let mut position =
                ICNettingPosition::new(company.clone(), self.config.base_currency.clone());

            // Sum receivables (company is creditor)
            for ((creditor, _), balance) in &self.balances {
                if creditor == company {
                    position.add_receivable(balance.receivable_balance);
                }
            }

            // Sum payables (company is debtor)
            for ((_, debtor), balance) in &self.balances {
                if debtor == company {
                    position.add_payable(balance.payable_balance);
                }
            }

            arrangement.total_gross_receivables += position.gross_receivables;
            arrangement.total_gross_payables += position.gross_payables;
            arrangement.gross_positions.push(position.clone());

            // Net position
            let mut net_position = position.clone();
            net_position.net_position = position.gross_receivables - position.gross_payables;
            arrangement.net_positions.push(net_position);
        }

        // Calculate net settlement
        let mut total_positive = Decimal::ZERO;
        for pos in &arrangement.net_positions {
            if pos.net_position > Decimal::ZERO {
                total_positive += pos.net_position;
            }
        }
        arrangement.net_settlement_amount = total_positive;
        arrangement.calculate_efficiency();

        arrangement
    }

    /// Get matching statistics.
    pub fn get_statistics(&self) -> MatchingStatistics {
        let total_receivables: Decimal = self.balances.values().map(|b| b.receivable_balance).sum();
        let total_payables: Decimal = self.balances.values().map(|b| b.payable_balance).sum();
        let total_difference: Decimal = self.balances.values().map(|b| b.difference.abs()).sum();

        let matched_count = self.balances.values().filter(|b| b.is_matched).count();
        let total_count = self.balances.len();

        MatchingStatistics {
            total_company_pairs: total_count,
            matched_pairs: matched_count,
            unmatched_pairs: total_count - matched_count,
            total_receivables,
            total_payables,
            total_difference,
            match_rate: if total_count > 0 {
                matched_count as f64 / total_count as f64
            } else {
                1.0
            },
        }
    }

    /// Clear all data.
    pub fn clear(&mut self) {
        self.balances.clear();
        self.unmatched_items.clear();
    }
}

/// An unmatched IC item for detailed matching.
#[derive(Debug, Clone)]
struct UnmatchedItem {
    /// Company code.
    company: String,
    /// Counterparty company code.
    counterparty: String,
    /// Amount.
    amount: Decimal,
    /// Is this a receivable (true) or payable (false)?
    is_receivable: bool,
    /// IC reference number.
    ic_reference: Option<String>,
    /// Transaction date.
    date: NaiveDate,
    /// Has been matched?
    matched: bool,
}

/// Statistics from matching process.
#[derive(Debug, Clone)]
pub struct MatchingStatistics {
    /// Total number of company pairs.
    pub total_company_pairs: usize,
    /// Number of matched pairs.
    pub matched_pairs: usize,
    /// Number of unmatched pairs.
    pub unmatched_pairs: usize,
    /// Total receivables amount.
    pub total_receivables: Decimal,
    /// Total payables amount.
    pub total_payables: Decimal,
    /// Total difference amount.
    pub total_difference: Decimal,
    /// Match rate (0.0 to 1.0).
    pub match_rate: f64,
}

/// IC Discrepancy for reconciliation.
#[derive(Debug, Clone)]
pub struct ICDiscrepancy {
    /// Creditor company.
    pub creditor: String,
    /// Debtor company.
    pub debtor: String,
    /// Receivable amount per creditor.
    pub receivable_amount: Decimal,
    /// Payable amount per debtor.
    pub payable_amount: Decimal,
    /// Difference.
    pub difference: Decimal,
    /// Suggested action.
    pub suggested_action: DiscrepancyAction,
    /// Currency.
    pub currency: String,
}

/// Suggested action for discrepancy resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscrepancyAction {
    /// Investigate - difference is significant.
    Investigate,
    /// Auto-adjust - difference is within threshold.
    AutoAdjust,
    /// Write-off - aged discrepancy.
    WriteOff,
    /// Currency adjustment needed.
    CurrencyAdjust,
}

impl ICMatchingEngine {
    /// Identify discrepancies requiring action.
    pub fn identify_discrepancies(&self) -> Vec<ICDiscrepancy> {
        let mut discrepancies = Vec::new();

        for balance in self.balances.values() {
            if !balance.is_matched {
                let action = if balance.difference.abs() <= self.config.auto_adjust_threshold {
                    DiscrepancyAction::AutoAdjust
                } else {
                    DiscrepancyAction::Investigate
                };

                discrepancies.push(ICDiscrepancy {
                    creditor: balance.creditor_company.clone(),
                    debtor: balance.debtor_company.clone(),
                    receivable_amount: balance.receivable_balance,
                    payable_amount: balance.payable_balance,
                    difference: balance.difference,
                    suggested_action: action,
                    currency: balance.currency.clone(),
                });
            }
        }

        discrepancies
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_matching_engine_basic() {
        let config = ICMatchingConfig::default();
        let mut engine = ICMatchingEngine::new(config);

        let date = NaiveDate::from_ymd_opt(2022, 6, 15).unwrap();

        // Add matching entries
        engine.add_receivable("1000", "1100", dec!(50000), Some("IC001"), date);
        engine.add_payable("1100", "1000", dec!(50000), Some("IC001"), date);

        let result = engine.run_matching(date);

        assert_eq!(result.matched_balances.len(), 1);
        assert_eq!(result.unmatched_balances.len(), 0);
        assert_eq!(result.match_rate, 1.0);
    }

    #[test]
    fn test_matching_engine_discrepancy() {
        let config = ICMatchingConfig::default();
        let mut engine = ICMatchingEngine::new(config);

        let date = NaiveDate::from_ymd_opt(2022, 6, 15).unwrap();

        // Add mismatched entries
        engine.add_receivable("1000", "1100", dec!(50000), Some("IC001"), date);
        engine.add_payable("1100", "1000", dec!(48000), Some("IC001"), date);

        let result = engine.run_matching(date);

        assert_eq!(result.unmatched_balances.len(), 1);
        assert_eq!(result.unmatched_balances[0].difference, dec!(2000));
    }

    #[test]
    fn test_matching_by_amount() {
        let config = ICMatchingConfig {
            tolerance: dec!(1),
            date_range_days: 3,
            ..Default::default()
        };

        let mut engine = ICMatchingEngine::new(config);

        let date1 = NaiveDate::from_ymd_opt(2022, 6, 15).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2022, 6, 16).unwrap();

        // Add entries without IC reference but matching amounts
        engine.add_receivable("1000", "1100", dec!(50000), None, date1);
        engine.add_payable("1100", "1000", dec!(50000), None, date2);

        let result = engine.run_matching(date2);

        assert_eq!(result.matched_balances.len(), 1);
    }

    #[test]
    fn test_generate_netting() {
        let config = ICMatchingConfig::default();
        let mut engine = ICMatchingEngine::new(config);

        let date = NaiveDate::from_ymd_opt(2022, 6, 30).unwrap();

        // Add multiple IC balances
        engine.add_receivable("1000", "1100", dec!(100000), Some("IC001"), date);
        engine.add_payable("1100", "1000", dec!(100000), Some("IC001"), date);
        engine.add_receivable("1100", "1200", dec!(50000), Some("IC002"), date);
        engine.add_payable("1200", "1100", dec!(50000), Some("IC002"), date);
        engine.add_receivable("1200", "1000", dec!(30000), Some("IC003"), date);
        engine.add_payable("1000", "1200", dec!(30000), Some("IC003"), date);

        let netting = engine.generate_netting(
            vec!["1000".to_string(), "1100".to_string(), "1200".to_string()],
            NaiveDate::from_ymd_opt(2022, 6, 1).unwrap(),
            date,
            NaiveDate::from_ymd_opt(2022, 7, 5).unwrap(),
        );

        assert_eq!(netting.participating_companies.len(), 3);
        assert!(netting.netting_efficiency > Decimal::ZERO);
    }

    #[test]
    fn test_identify_discrepancies() {
        let config = ICMatchingConfig {
            auto_adjust_threshold: dec!(100),
            ..Default::default()
        };

        let mut engine = ICMatchingEngine::new(config);
        let date = NaiveDate::from_ymd_opt(2022, 6, 15).unwrap();

        // Small discrepancy (auto-adjust)
        engine.add_receivable("1000", "1100", dec!(50000), Some("IC001"), date);
        engine.add_payable("1100", "1000", dec!(49950), Some("IC001"), date);

        // Large discrepancy (investigate)
        engine.add_receivable("1000", "1200", dec!(100000), Some("IC002"), date);
        engine.add_payable("1200", "1000", dec!(95000), Some("IC002"), date);

        engine.run_matching(date);
        let discrepancies = engine.identify_discrepancies();

        assert_eq!(discrepancies.len(), 2);

        let small = discrepancies.iter().find(|d| d.debtor == "1100").unwrap();
        assert_eq!(small.suggested_action, DiscrepancyAction::AutoAdjust);

        let large = discrepancies.iter().find(|d| d.debtor == "1200").unwrap();
        assert_eq!(large.suggested_action, DiscrepancyAction::Investigate);
    }
}
