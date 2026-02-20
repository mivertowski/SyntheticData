//! Loader for the comprehensive Plan Comptable Général (PCG) 2024 structure.
//!
//! Uses the official PCG tree from [arrhes/PCG](https://github.com/arrhes/PCG) to build
//! a Chart of Accounts with correct French account numbers and labels.

use serde::Deserialize;

use crate::models::{
    AccountSubType, AccountType, ChartOfAccounts, CoAComplexity, GLAccount, IndustrySector,
};

/// Root of the PCG JSON: array of top-level classes (1–8).
pub type PcgRoot = Vec<PcgNode>;

/// One node in the PCG tree (class, subclass, or account).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PcgNode {
    pub number: u32,
    pub label: String,
    #[serde(default)]
    pub system: String,
    #[serde(default)]
    pub accounts: Vec<PcgNode>,
}

/// Embedded PCG 2024 JSON (from https://github.com/arrhes/PCG).
const PCG_2024_JSON: &str = include_str!("../resources/pcg_2024.json");

/// Load the PCG 2024 tree from the embedded JSON.
pub fn load_pcg_2024() -> Result<PcgRoot, serde_json::Error> {
    serde_json::from_str(PCG_2024_JSON)
}

/// Flatten the PCG tree into (account_code, label, class) for postable accounts.
/// Includes nodes that are "base" or "developed", or leaves (empty accounts).
/// Stops when we have at least `max_accounts` (for complexity capping).
fn flatten_pcg(
    nodes: &[PcgNode],
    class_from_prefix: u8,
    out: &mut Vec<(u32, String, u8)>,
    max_accounts: usize,
) {
    if out.len() >= max_accounts {
        return;
    }
    for node in nodes {
        let class = if node.number < 10 {
            node.number as u8
        } else {
            class_from_prefix
        };
        let is_leaf = node.accounts.is_empty();
        let is_postable = is_leaf
            || node.system == "base"
            || node.system == "developed"
            || (node.system == "condensed" && node.accounts.is_empty());
        if is_postable {
            out.push((node.number, node.label.clone(), class));
        }
        if !node.accounts.is_empty() && out.len() < max_accounts {
            flatten_pcg(&node.accounts, class, out, max_accounts);
        }
    }
}

/// Normalize a PCG account number into our 6-digit GL format.
///
/// The PCG tree includes intermediate nodes like `41` (tiers) or `411` (clients).
/// For our generators and exports we use a 6-digit "base" account format; we
/// therefore right-pad shorter prefixes with zeros:
/// - 41 → 410000
/// - 411 → 411000
/// - 6011 → 601100
fn normalize_pcg_account_number(number: u32) -> String {
    let s = number.to_string();
    if s.len() >= 6 {
        return s;
    }
    let pow = (6 - s.len()) as u32;
    let factor = 10u32.pow(pow);
    format!("{:06}", number * factor)
}

/// Map PCG class and account number to our AccountType and AccountSubType.
fn pcg_to_account_type(class: u8, number: u32) -> (AccountType, AccountSubType) {
    use AccountSubType::{
        AccountsPayable, AccountsReceivable, AccruedLiabilities, AccumulatedDepreciation, Cash,
        CommonStock, FixedAssets, Inventory, LongTermDebt, OperatingExpenses, OtherAssets,
        OtherLiabilities, ProductRevenue, RetainedEarnings, SuspenseClearing,
    };
    use AccountType::{Asset, Equity, Expense, Liability, Revenue};
    match class {
        1 => {
            let is_equity = (10..13).contains(&number) || (100..130).contains(&number);
            if is_equity {
                if (101..=109).contains(&number) {
                    (Equity, CommonStock)
                } else {
                    (Equity, RetainedEarnings)
                }
            } else if (16..=169).contains(&number) {
                (Liability, LongTermDebt)
            } else {
                (Liability, AccruedLiabilities)
            }
        }
        2 => {
            if (28..=282).contains(&number) || (29..=297).contains(&number) {
                (Asset, AccumulatedDepreciation)
            } else {
                (Asset, FixedAssets)
            }
        }
        3 => (Asset, Inventory),
        4 => {
            if (41..=419).contains(&number) {
                (Asset, AccountsReceivable)
            } else if (40..=409).contains(&number) {
                (Liability, AccountsPayable)
            } else if (42..=428).contains(&number) {
                (Liability, AccruedLiabilities)
            } else {
                (Liability, OtherLiabilities)
            }
        }
        5 => (Asset, Cash),
        6 => (Expense, OperatingExpenses),
        7 => (Revenue, ProductRevenue),
        8 => (Asset, SuspenseClearing),
        _ => (Asset, OtherAssets),
    }
}

/// Build a Chart of Accounts from the comprehensive PCG 2024 structure.
/// Respects `complexity` by limiting the number of accounts (Small ~100, Medium ~400, Large up to full tree).
pub fn build_chart_of_accounts_from_pcg_2024(
    complexity: CoAComplexity,
    industry: IndustrySector,
) -> Result<ChartOfAccounts, serde_json::Error> {
    let root = load_pcg_2024()?;
    let max_accounts = complexity.target_count();
    let mut flat = Vec::with_capacity(max_accounts.min(5000));
    for class_node in &root {
        let class = class_node.number as u8;
        flatten_pcg(&class_node.accounts, class, &mut flat, max_accounts);
    }

    let coa_id = format!("COA_PCG_2024_{:?}_{}", industry, max_accounts);
    let name = format!("Plan Comptable Général 2024 – {:?}", industry);
    let mut coa = ChartOfAccounts::new(coa_id, name, "FR".to_string(), industry, complexity);
    coa.account_format = "######".to_string();

    for (number, label, class) in flat {
        let code = normalize_pcg_account_number(number);
        let (acc_type, sub_type) = pcg_to_account_type(class, number);
        let mut account = GLAccount::new(code, label, acc_type, sub_type);
        account.requires_cost_center = acc_type == AccountType::Expense;
        if class == 8 {
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
    fn test_load_pcg_2024() {
        let root = load_pcg_2024().unwrap();
        assert_eq!(root.len(), 8); // Classes 1-8
        assert_eq!(root[0].number, 1);
        assert_eq!(root[0].label, "Comptes de capitaux");
    }

    #[test]
    fn test_build_coa_from_pcg() {
        let coa = build_chart_of_accounts_from_pcg_2024(
            CoAComplexity::Small,
            IndustrySector::Manufacturing,
        )
        .unwrap();
        assert_eq!(coa.country, "FR");
        assert!(coa.account_count() >= 50);
        assert!(coa.account_count() <= 150);
    }
}
