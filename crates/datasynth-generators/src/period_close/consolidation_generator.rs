//! Consolidation generator.
//!
//! Aggregates per-entity trial balances into a consolidated group view,
//! applies intercompany elimination journal entries, and produces both:
//! - `Vec<FinancialStatementLineItem>` for the consolidated financial statement
//! - `ConsolidationSchedule` showing the pre-elimination → elimination → post-elimination flow

use datasynth_core::models::{
    ConsolidationLineItem, ConsolidationSchedule, FinancialStatementLineItem, JournalEntry,
};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Handles consolidation of multi-entity financial data.
pub struct ConsolidationGenerator;

impl ConsolidationGenerator {
    /// Generate consolidated financial statement line items and a consolidation schedule.
    ///
    /// # Arguments
    /// * `entity_trial_balances` – map of entity_code → (account_category → net balance).
    ///   The net balance is `debit_balance - credit_balance` for each category.
    /// * `elimination_entries` – journal entries where `header.is_elimination == true`.
    /// * `period_label` – string label for the period (e.g. "2024-03").
    ///
    /// # Returns
    /// A tuple of:
    /// - consolidated `FinancialStatementLineItem` list (one per account category, post-elimination)
    /// - `ConsolidationSchedule` with full breakdown
    pub fn consolidate(
        entity_trial_balances: &HashMap<String, HashMap<String, Decimal>>,
        elimination_entries: &[JournalEntry],
        period_label: &str,
    ) -> (Vec<FinancialStatementLineItem>, ConsolidationSchedule) {
        // Step 1: Collect all account categories across all entities
        let mut all_categories: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        for balances in entity_trial_balances.values() {
            for cat in balances.keys() {
                all_categories.insert(cat.clone());
            }
        }

        // Step 2: Compute elimination adjustments per account category.
        // For each elimination JE, map its GL account to a category, then
        // accumulate net debit-credit effect.
        let mut elim_by_category: HashMap<String, Decimal> = HashMap::new();
        for je in elimination_entries {
            if !je.header.is_elimination {
                continue;
            }
            for line in &je.lines {
                let category = category_from_account_code(&line.gl_account);
                let net = line.debit_amount - line.credit_amount;
                *elim_by_category
                    .entry(category)
                    .or_insert(Decimal::ZERO) += net;
            }
        }

        // Step 3: Build schedule line items
        let mut schedule_lines: Vec<ConsolidationLineItem> = Vec::new();
        let mut consolidated_items: Vec<FinancialStatementLineItem> = Vec::new();

        for (sort_idx, category) in all_categories.iter().enumerate() {
            // Per-entity amounts for this category
            let mut entity_amounts: HashMap<String, Decimal> = HashMap::new();
            let mut pre_total = Decimal::ZERO;

            for (entity_code, balances) in entity_trial_balances {
                let amount = balances.get(category).copied().unwrap_or(Decimal::ZERO);
                entity_amounts.insert(entity_code.clone(), amount);
                pre_total += amount;
            }

            let elimination_adj = elim_by_category
                .get(category)
                .copied()
                .unwrap_or(Decimal::ZERO);
            let post_total = pre_total + elimination_adj;

            schedule_lines.push(ConsolidationLineItem {
                account_category: category.clone(),
                entity_amounts,
                pre_elimination_total: pre_total,
                elimination_adjustments: elimination_adj,
                post_elimination_total: post_total,
            });

            // Build a FinancialStatementLineItem for the consolidated statement
            consolidated_items.push(FinancialStatementLineItem {
                line_code: format!("CONS-{}", category.to_uppercase()),
                label: category.clone(),
                section: section_for_category(category),
                sort_order: (sort_idx + 1) as u32,
                amount: post_total,
                amount_prior: None,
                indent_level: 0,
                is_total: false,
                gl_accounts: Vec::new(),
            });
        }

        let schedule = ConsolidationSchedule {
            period: period_label.to_string(),
            line_items: schedule_lines,
        };

        (consolidated_items, schedule)
    }

    /// Build a per-entity trial balance map from raw journal entries.
    ///
    /// Returns `HashMap<entity_code, HashMap<account_category, net_balance>>`.
    /// Non-elimination entries only (pass `include_eliminations = false`) or all.
    pub fn build_entity_trial_balances(
        journal_entries: &[JournalEntry],
        include_eliminations: bool,
    ) -> HashMap<String, HashMap<String, Decimal>> {
        let mut result: HashMap<String, HashMap<String, Decimal>> = HashMap::new();

        for je in journal_entries {
            if !include_eliminations && je.header.is_elimination {
                continue;
            }
            let entity = je.header.company_code.clone();
            let entity_map = result.entry(entity).or_default();

            for line in &je.lines {
                let category = category_from_account_code(&line.gl_account);
                let net = line.debit_amount - line.credit_amount;
                *entity_map.entry(category).or_insert(Decimal::ZERO) += net;
            }
        }

        result
    }
}

/// Map an account code prefix to one of the category strings used by
/// `FinancialStatementGenerator` / `build_cumulative_trial_balance`.
pub(crate) fn category_from_account_code(account: &str) -> String {
    let prefix = account.get(..1).unwrap_or("");
    let two = account.get(..2).unwrap_or("");
    match prefix {
        "1" => match two {
            "10" | "11" => {
                if account.starts_with("11") {
                    "Receivables".to_string()
                } else {
                    "Cash".to_string()
                }
            }
            "13" => "Inventory".to_string(),
            "15" | "16" | "17" | "18" | "19" => "FixedAssets".to_string(),
            _ => "Cash".to_string(),
        },
        "2" => match two {
            "20" => "Payables".to_string(),
            "21" => "AccruedLiabilities".to_string(),
            "25" | "26" | "27" | "28" | "29" => "LongTermDebt".to_string(),
            _ => "AccruedLiabilities".to_string(),
        },
        "3" => "Equity".to_string(),
        "4" => "Revenue".to_string(),
        "5" => "CostOfSales".to_string(),
        "6" | "7" => "OperatingExpenses".to_string(),
        _ => "Other".to_string(),
    }
}

fn section_for_category(category: &str) -> String {
    match category {
        "Cash" => "Current Assets",
        "Receivables" => "Current Assets",
        "Inventory" => "Current Assets",
        "FixedAssets" => "Non-Current Assets",
        "Payables" => "Current Liabilities",
        "AccruedLiabilities" => "Current Liabilities",
        "LongTermDebt" => "Non-Current Liabilities",
        "Equity" => "Equity",
        "Revenue" => "Revenue",
        "CostOfSales" => "Cost of Sales",
        "OperatingExpenses" => "Operating Expenses",
        _ => "Other",
    }
    .to_string()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_entity_tbs() -> HashMap<String, HashMap<String, Decimal>> {
        let mut tbs = HashMap::new();

        let mut c001 = HashMap::new();
        c001.insert("Cash".to_string(), Decimal::from(100_000));
        c001.insert("Revenue".to_string(), Decimal::from(-500_000));
        c001.insert("Receivables".to_string(), Decimal::from(200_000));
        tbs.insert("C001".to_string(), c001);

        let mut c002 = HashMap::new();
        c002.insert("Cash".to_string(), Decimal::from(50_000));
        c002.insert("Revenue".to_string(), Decimal::from(-300_000));
        c002.insert("Payables".to_string(), Decimal::from(-80_000));
        tbs.insert("C002".to_string(), c002);

        tbs
    }

    #[test]
    fn test_consolidate_no_eliminations() {
        let tbs = make_entity_tbs();
        let (items, schedule) =
            ConsolidationGenerator::consolidate(&tbs, &[], "2024-03");

        // Schedule period label
        assert_eq!(schedule.period, "2024-03");

        // All categories present
        let cats: Vec<&str> = schedule
            .line_items
            .iter()
            .map(|li| li.account_category.as_str())
            .collect();
        assert!(cats.contains(&"Cash"));
        assert!(cats.contains(&"Revenue"));
        assert!(cats.contains(&"Receivables"));
        assert!(cats.contains(&"Payables"));

        // Pre-elimination = sum of entity amounts; eliminations = 0
        for li in &schedule.line_items {
            let entity_sum: Decimal = li.entity_amounts.values().copied().sum();
            assert_eq!(li.pre_elimination_total, entity_sum,
                "pre_elimination_total should equal sum of entity amounts for {}",
                li.account_category);
            assert_eq!(li.elimination_adjustments, Decimal::ZERO,
                "no eliminations expected for {}", li.account_category);
            assert_eq!(li.post_elimination_total, li.pre_elimination_total,
                "post should equal pre when no eliminations for {}", li.account_category);
        }

        // Consolidated line items should match schedule count
        assert_eq!(items.len(), schedule.line_items.len());
    }

    #[test]
    fn test_pre_elimination_equals_entity_sum() {
        let tbs = make_entity_tbs();
        let (_, schedule) = ConsolidationGenerator::consolidate(&tbs, &[], "2024-01");

        let cash_line = schedule
            .line_items
            .iter()
            .find(|li| li.account_category == "Cash")
            .unwrap();

        // Cash: C001=100_000, C002=50_000 → pre=150_000
        assert_eq!(cash_line.pre_elimination_total, Decimal::from(150_000));
        assert_eq!(
            cash_line.entity_amounts.get("C001").copied().unwrap(),
            Decimal::from(100_000)
        );
        assert_eq!(
            cash_line.entity_amounts.get("C002").copied().unwrap(),
            Decimal::from(50_000)
        );
    }

    #[test]
    fn test_post_equals_pre_plus_adjustment() {
        let tbs = make_entity_tbs();
        let (_, schedule) = ConsolidationGenerator::consolidate(&tbs, &[], "2024-01");

        for li in &schedule.line_items {
            assert_eq!(
                li.post_elimination_total,
                li.pre_elimination_total + li.elimination_adjustments,
                "post = pre + adj invariant failed for {}",
                li.account_category
            );
        }
    }

    #[test]
    fn test_single_entity_consolidated_equals_standalone() {
        let mut tbs = HashMap::new();
        let mut c001 = HashMap::new();
        c001.insert("Cash".to_string(), Decimal::from(100_000));
        c001.insert("Revenue".to_string(), Decimal::from(-500_000));
        tbs.insert("C001".to_string(), c001);

        let (items, schedule) = ConsolidationGenerator::consolidate(&tbs, &[], "2024-01");

        // With single entity, consolidated = standalone (no elimination effect)
        for li in &schedule.line_items {
            let standalone = *li.entity_amounts.get("C001").unwrap();
            assert_eq!(
                li.post_elimination_total, standalone,
                "single-entity consolidated should equal standalone for {}",
                li.account_category
            );
        }
        assert!(!items.is_empty());
    }
}
