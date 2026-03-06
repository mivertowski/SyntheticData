//! Loader for the SKR04 (Standardkontenrahmen 04) chart of accounts.
//!
//! Mirrors `pcg_loader.rs` for German GAAP. Loads the embedded `skr04_2024.json`
//! tree and builds a `ChartOfAccounts` with correct 4-digit account numbers.

use serde::Deserialize;

use crate::models::{
    AccountSubType, AccountType, ChartOfAccounts, CoAComplexity, GLAccount, IndustrySector,
};

/// Root of the SKR04 JSON: array of top-level classes (0–9).
pub type Skr04Root = Vec<Skr04Node>;

/// One node in the SKR04 tree (class, group, or account).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Skr04Node {
    pub number: u32,
    pub label: String,
    #[serde(default)]
    pub system: String,
    #[serde(default)]
    pub accounts: Vec<Skr04Node>,
}

/// Embedded SKR04 2024 JSON.
const SKR04_2024_JSON: &str = include_str!("../resources/skr04_2024.json");

/// Load the SKR04 2024 tree from the embedded JSON.
pub fn load_skr04_2024() -> Result<Skr04Root, serde_json::Error> {
    serde_json::from_str(SKR04_2024_JSON)
}

/// Flatten the SKR04 tree into (account_number, label, class) for postable accounts.
fn flatten_skr04(
    nodes: &[Skr04Node],
    class_from_parent: u8,
    out: &mut Vec<(u32, String, u8)>,
    max_accounts: usize,
) {
    if out.len() >= max_accounts {
        return;
    }
    for node in nodes {
        // Top-level class nodes have number 0-9
        let class = if node.number < 10 {
            node.number as u8
        } else {
            class_from_parent
        };

        let is_leaf = node.accounts.is_empty();
        let is_postable = is_leaf
            || node.system == "base"
            || node.system == "developed"
            || (node.system == "condensed" && node.accounts.is_empty());

        // Only include 4-digit accounts (10+)
        if is_postable && node.number >= 10 {
            out.push((node.number, node.label.clone(), class));
        }

        if !node.accounts.is_empty() && out.len() < max_accounts {
            flatten_skr04(&node.accounts, class, out, max_accounts);
        }
    }
}

/// Normalize an SKR04 account number to 4-digit format.
///
/// - 2-digit (60) → "0060"
/// - 3-digit (200) → "0200"
/// - 4-digit (1200) → "1200"
fn normalize_skr04_account_number(number: u32) -> String {
    format!("{number:04}")
}

/// Map SKR04 class and account number to AccountType and AccountSubType.
fn skr04_to_account_type(class: u8, number: u32) -> (AccountType, AccountSubType) {
    use AccountSubType::*;
    use AccountType::*;

    match class {
        0 => {
            // Fixed assets
            if (700..800).contains(&number) {
                (Asset, AccumulatedDepreciation)
            } else if (550..650).contains(&number) {
                (Asset, OtherAssets) // Financial assets
            } else {
                (Asset, FixedAssets)
            }
        }
        1 => {
            // Current assets
            if (1000..1200).contains(&number) {
                (Asset, Inventory)
            } else if (1200..1300).contains(&number) {
                (Asset, AccountsReceivable)
            } else if (1300..1600).contains(&number) {
                (Asset, OtherReceivables) // includes VAT receivable (1570-1599)
            } else if (1600..1700).contains(&number) || (1800..1900).contains(&number) {
                (Asset, Cash) // Kasse (16xx) / Bank (18xx)
            } else if (1900..2000).contains(&number) {
                (Asset, PrepaidExpenses)
            } else {
                (Asset, OtherAssets)
            }
        }
        2 => {
            // Equity
            if (2000..2050).contains(&number) {
                (Equity, CommonStock)
            } else if (2050..2400).contains(&number) {
                (Equity, RetainedEarnings) // Reserves
            } else if (2900..3000).contains(&number) {
                (Equity, OtherComprehensiveIncome) // CTA, results
            } else {
                (Equity, RetainedEarnings)
            }
        }
        3 => {
            // Liabilities
            if (3000..3100).contains(&number) {
                (Liability, AccruedLiabilities) // Provisions
            } else if (3100..3200).contains(&number) {
                if number < 3150 {
                    (Liability, ShortTermDebt)
                } else {
                    (Liability, LongTermDebt)
                }
            } else if (3250..3270).contains(&number) {
                (Liability, DeferredRevenue) // Received advances
            } else if (3300..3400).contains(&number) {
                (Liability, AccountsPayable) // Trade payables
            } else if (3500..3600).contains(&number) {
                (Liability, OtherLiabilities) // IC payables
            } else if (3700..3800).contains(&number) {
                (Liability, AccruedLiabilities) // Personnel payables
            } else if (3800..3900).contains(&number) {
                (Liability, TaxLiabilities) // VAT
            } else {
                (Liability, OtherLiabilities) // Passive accruals (39xx) and others
            }
        }
        4 => {
            // Revenue
            if (4000..4200).contains(&number) {
                (Revenue, ProductRevenue)
            } else if (4200..4500).contains(&number) {
                (Revenue, ServiceRevenue)
            } else if (4500..4600).contains(&number) {
                (Revenue, OtherIncome) // IC revenue
            } else if (4700..4800).contains(&number) {
                (Revenue, ProductRevenue) // Sales deductions
            } else {
                (Revenue, OtherIncome) // Other operating income (49xx) and others
            }
        }
        5 => {
            // Material / COGS
            (Expense, CostOfGoodsSold)
        }
        6 => {
            // Personnel & other operating expenses
            if (6000..6200).contains(&number) {
                (Expense, OperatingExpenses) // Personnel
            } else if (6200..6300).contains(&number) {
                (Expense, DepreciationExpense) // Depreciation
            } else if (6300..6600).contains(&number) {
                (Expense, OperatingExpenses) // Premises, insurance, vehicles (63xx-65xx)
            } else if (6600..6700).contains(&number) {
                (Expense, SellingExpenses) // Advertising, travel
            } else if (6700..6800).contains(&number) {
                (Expense, OperatingExpenses) // Transport
            } else {
                (Expense, OtherExpenses) // FX, irregular items
            }
        }
        7 => {
            // Financial income/expense
            if (7000..7200).contains(&number) {
                (Revenue, InterestIncome) // Financial income
            } else if (7200..7500).contains(&number) {
                (Expense, InterestExpense) // Financial expense
            } else {
                (Expense, OtherExpenses)
            }
        }
        8 => {
            // Tax / extraordinary (note: in our JSON, tax accounts have numbers 76xx)
            (Expense, TaxExpense)
        }
        9 => {
            // Statistical
            (Asset, SuspenseClearing)
        }
        _ => (Asset, OtherAssets),
    }
}

/// Build a Chart of Accounts from the SKR04 2024 structure.
///
/// Respects `complexity` by limiting the number of accounts.
pub fn build_chart_of_accounts_from_skr04(
    complexity: CoAComplexity,
    industry: IndustrySector,
) -> Result<ChartOfAccounts, serde_json::Error> {
    let root = load_skr04_2024()?;
    let max_accounts = complexity.target_count();
    let mut flat = Vec::with_capacity(max_accounts.min(2000));

    for class_node in &root {
        let class = class_node.number as u8;
        flatten_skr04(&class_node.accounts, class, &mut flat, max_accounts);
    }

    let coa_id = format!("COA_SKR04_2024_{industry:?}_{max_accounts}");
    let name = format!("Standardkontenrahmen 04 – {industry:?}");
    let mut coa = ChartOfAccounts::new(coa_id, name, "DE".to_string(), industry, complexity);
    coa.account_format = "####".to_string();

    for (number, label, class) in flat {
        let code = normalize_skr04_account_number(number);
        let (acc_type, sub_type) = skr04_to_account_type(class, number);
        let mut account = GLAccount::new(code, label, acc_type, sub_type);
        account.requires_cost_center = matches!(acc_type, AccountType::Expense);
        if class == 9 {
            account.is_suspense_account = true;
        }
        coa.add_account(account);
    }

    Ok(coa)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_load_skr04_2024() {
        let root = load_skr04_2024().unwrap();
        assert_eq!(root.len(), 10); // Classes 0-9
        assert_eq!(root[0].number, 0);
        assert_eq!(root[0].label, "Anlagevermögen");
        assert_eq!(root[9].number, 9);
    }

    #[test]
    fn test_normalize_skr04_account_number() {
        assert_eq!(normalize_skr04_account_number(60), "0060");
        assert_eq!(normalize_skr04_account_number(200), "0200");
        assert_eq!(normalize_skr04_account_number(1200), "1200");
        assert_eq!(normalize_skr04_account_number(6220), "6220");
    }

    #[test]
    fn test_skr04_loader_basic() {
        let coa =
            build_chart_of_accounts_from_skr04(CoAComplexity::Small, IndustrySector::Manufacturing)
                .unwrap();
        assert_eq!(coa.country, "DE");
        assert_eq!(coa.account_format, "####");
        assert!(
            coa.account_count() >= 30,
            "SKR04 small CoA should have at least 30 accounts, got {}",
            coa.account_count()
        );
    }

    #[test]
    fn test_skr04_class_coverage() {
        let coa =
            build_chart_of_accounts_from_skr04(CoAComplexity::Large, IndustrySector::Manufacturing)
                .unwrap();

        let first_digits: std::collections::HashSet<char> = coa
            .accounts
            .iter()
            .filter_map(|a| a.account_number.chars().next())
            .collect();

        // Classes 0-7 and 9 have distinct first-digit ranges.
        // Class 8 (Steuern/außerordentliches Ergebnis) uses 76xx-77xx numbers
        // which overlap with class 7 — this is correct per SKR04 structure.
        for digit in ['0', '1', '2', '3', '4', '5', '6', '7', '9'] {
            assert!(
                first_digits.contains(&digit),
                "SKR04 large CoA should have accounts starting with {}",
                digit
            );
        }

        // Verify we have tax accounts (class 8 structural) with 76xx numbers
        let has_tax = coa
            .accounts
            .iter()
            .any(|a| a.account_number.starts_with("76"));
        assert!(has_tax, "SKR04 should have tax accounts (76xx)");
    }

    #[test]
    fn test_skr04_4_digit_format() {
        let coa =
            build_chart_of_accounts_from_skr04(CoAComplexity::Small, IndustrySector::Manufacturing)
                .unwrap();

        for account in &coa.accounts {
            assert_eq!(
                account.account_number.len(),
                4,
                "SKR04 account {} should be 4 digits",
                account.account_number
            );
            assert!(
                account.account_number.chars().all(|c| c.is_ascii_digit()),
                "SKR04 account {} should be numeric",
                account.account_number
            );
        }
    }
}
