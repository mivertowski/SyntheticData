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

/// Extract 2-digit PCG subclass: 1011→10, 164→16, 4111→41
fn pcg_subclass(number: u32) -> u32 {
    let mut n = number;
    while n >= 100 {
        n /= 10;
    }
    n
}

/// Extract 3-digit PCG account group: 1011→101, 164→164, 4111→411
fn pcg_account_group(number: u32) -> u32 {
    let mut n = number;
    while n >= 1000 {
        n /= 10;
    }
    n
}

/// Map PCG class and account number to our AccountType and AccountSubType.
fn pcg_to_account_type(class: u8, number: u32) -> (AccountType, AccountSubType) {
    use AccountSubType::{
        AccountsPayable, AccountsReceivable, AccruedLiabilities, AccumulatedDepreciation, Cash,
        CommonStock, FixedAssets, Inventory, LongTermDebt, OperatingExpenses, OtherAssets,
        OtherLiabilities, ProductRevenue, RetainedEarnings, SuspenseClearing,
    };
    use AccountType::{Asset, Equity, Expense, Liability, Revenue};
    let sub = pcg_subclass(number);
    match class {
        1 => {
            if (10..=14).contains(&sub) {
                let group = pcg_account_group(number);
                if (101..=109).contains(&group) {
                    (Equity, CommonStock)
                } else {
                    (Equity, RetainedEarnings)
                }
            } else if sub == 15 {
                (Liability, AccruedLiabilities)
            } else if (16..=17).contains(&sub) {
                (Liability, LongTermDebt)
            } else {
                (Liability, OtherLiabilities)
            }
        }
        2 => {
            if (28..=29).contains(&sub) {
                (Asset, AccumulatedDepreciation)
            } else {
                (Asset, FixedAssets)
            }
        }
        3 => (Asset, Inventory),
        4 => {
            if sub == 40 {
                (Liability, AccountsPayable)
            } else if sub == 41 {
                (Asset, AccountsReceivable)
            } else if sub == 42 {
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

    let coa_id = format!("COA_PCG_2024_{industry:?}_{max_accounts}");
    let name = format!("Plan Comptable Général 2024 – {industry:?}");
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
    fn test_pcg_subclass() {
        assert_eq!(super::pcg_subclass(10), 10);
        assert_eq!(super::pcg_subclass(101), 10);
        assert_eq!(super::pcg_subclass(1011), 10);
        assert_eq!(super::pcg_subclass(164), 16);
        assert_eq!(super::pcg_subclass(4111), 41);
        assert_eq!(super::pcg_subclass(28), 28);
        assert_eq!(super::pcg_subclass(281), 28);
    }

    #[test]
    fn test_pcg_account_group() {
        assert_eq!(super::pcg_account_group(101), 101);
        assert_eq!(super::pcg_account_group(1011), 101);
        assert_eq!(super::pcg_account_group(10131), 101);
        assert_eq!(super::pcg_account_group(164), 164);
        assert_eq!(super::pcg_account_group(4111), 411);
    }

    #[test]
    fn test_pcg_to_account_type_multidigit() {
        use crate::models::{AccountSubType, AccountType};
        // Class 1: 1011 (Capital souscrit) should be Equity/CommonStock, not AccruedLiabilities
        let (ty, sub) = super::pcg_to_account_type(1, 1011);
        assert_eq!(ty, AccountType::Equity);
        assert_eq!(sub, AccountSubType::CommonStock);

        // 129 (Résultat) should be Equity/RetainedEarnings
        let (ty, sub) = super::pcg_to_account_type(1, 129);
        assert_eq!(ty, AccountType::Equity);
        assert_eq!(sub, AccountSubType::RetainedEarnings);

        // 1641 (Emprunts) should be Liability/LongTermDebt
        let (ty, sub) = super::pcg_to_account_type(1, 1641);
        assert_eq!(ty, AccountType::Liability);
        assert_eq!(sub, AccountSubType::LongTermDebt);

        // 151 (Provisions) should be Liability/AccruedLiabilities
        let (ty, sub) = super::pcg_to_account_type(1, 151);
        assert_eq!(ty, AccountType::Liability);
        assert_eq!(sub, AccountSubType::AccruedLiabilities);

        // Class 2: 2815 (Amort. immob.) should be AccumulatedDepreciation
        let (ty, sub) = super::pcg_to_account_type(2, 2815);
        assert_eq!(ty, AccountType::Asset);
        assert_eq!(sub, AccountSubType::AccumulatedDepreciation);

        // Class 4: 4111 (Clients) should be AccountsReceivable
        let (ty, sub) = super::pcg_to_account_type(4, 4111);
        assert_eq!(ty, AccountType::Asset);
        assert_eq!(sub, AccountSubType::AccountsReceivable);

        // 4011 (Fournisseurs) should be AccountsPayable
        let (ty, sub) = super::pcg_to_account_type(4, 4011);
        assert_eq!(ty, AccountType::Liability);
        assert_eq!(sub, AccountSubType::AccountsPayable);

        // 421 (Personnel) should be AccruedLiabilities
        let (ty, sub) = super::pcg_to_account_type(4, 421);
        assert_eq!(ty, AccountType::Liability);
        assert_eq!(sub, AccountSubType::AccruedLiabilities);
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
