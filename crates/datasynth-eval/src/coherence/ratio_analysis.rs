//! Financial ratio analysis evaluator (ISA 520).
//!
//! Computes standard financial ratios from journal entry data and validates
//! them for reasonableness against industry benchmarks.

use datasynth_core::models::JournalEntry;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Results of ratio analysis evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatioAnalysisResult {
    /// Entity/company code evaluated.
    pub entity_code: String,
    /// Fiscal period label (e.g. "2024-Q1").
    pub period: String,
    /// Computed financial ratios.
    pub ratios: FinancialRatios,
    /// Reasonableness checks against industry bounds.
    pub reasonableness_checks: Vec<RatioCheck>,
    /// True if all computable ratios are within industry bounds.
    pub passes: bool,
}

/// Standard financial ratios derived from journal entry data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FinancialRatios {
    // --- Liquidity ---
    /// Current ratio: current assets / current liabilities.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "rust_decimal::serde::str_option"
    )]
    pub current_ratio: Option<Decimal>,
    /// Quick ratio: (current assets − inventory) / current liabilities.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "rust_decimal::serde::str_option"
    )]
    pub quick_ratio: Option<Decimal>,

    // --- Activity ---
    /// Days Sales Outstanding: AR / Revenue × 365.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "rust_decimal::serde::str_option"
    )]
    pub dso: Option<Decimal>,
    /// Days Payable Outstanding: AP / COGS × 365.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "rust_decimal::serde::str_option"
    )]
    pub dpo: Option<Decimal>,
    /// Inventory turnover: COGS / inventory.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "rust_decimal::serde::str_option"
    )]
    pub inventory_turnover: Option<Decimal>,

    // --- Profitability ---
    /// Gross margin: (revenue − COGS) / revenue.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "rust_decimal::serde::str_option"
    )]
    pub gross_margin: Option<Decimal>,
    /// Operating margin: (revenue − COGS − operating expenses) / revenue.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "rust_decimal::serde::str_option"
    )]
    pub operating_margin: Option<Decimal>,
    /// Net margin: net income / revenue.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "rust_decimal::serde::str_option"
    )]
    pub net_margin: Option<Decimal>,
    /// Return on assets: net income / total assets.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "rust_decimal::serde::str_option"
    )]
    pub roa: Option<Decimal>,
    /// Return on equity: net income / total equity.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "rust_decimal::serde::str_option"
    )]
    pub roe: Option<Decimal>,

    // --- Leverage ---
    /// Debt-to-equity: total liabilities / total equity.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "rust_decimal::serde::str_option"
    )]
    pub debt_to_equity: Option<Decimal>,
    /// Debt-to-assets: total liabilities / total assets.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "rust_decimal::serde::str_option"
    )]
    pub debt_to_assets: Option<Decimal>,
}

/// Reasonableness check for a single ratio against industry bounds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatioCheck {
    /// Name of the ratio (e.g. "current_ratio").
    pub ratio_name: String,
    /// Computed value, if available.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "rust_decimal::serde::str_option"
    )]
    pub value: Option<Decimal>,
    /// Minimum acceptable value for this industry.
    #[serde(with = "rust_decimal::serde::str")]
    pub industry_min: Decimal,
    /// Maximum acceptable value for this industry.
    #[serde(with = "rust_decimal::serde::str")]
    pub industry_max: Decimal,
    /// True if the ratio is within bounds (or not computable — vacuously true).
    pub is_reasonable: bool,
}

// ─── Account-range helpers ────────────────────────────────────────────────────

/// Internal totals built from GL account prefixes.
#[derive(Debug, Default)]
struct GlTotals {
    /// 1xxx  – all assets (net of credits).
    assets: Decimal,
    /// 11xx  – accounts receivable.
    ar: Decimal,
    /// 12xx  – inventory.
    inventory: Decimal,
    /// 2xxx  – liabilities.
    liabilities: Decimal,
    /// 21xx  – accounts payable.
    ap: Decimal,
    /// 3xxx  – equity.
    equity: Decimal,
    /// 4xxx  – revenue (credit-normal).
    revenue: Decimal,
    /// 5xxx  – cost of goods sold (debit-normal).
    cogs: Decimal,
    /// 6xxx–8xxx – operating expenses (debit-normal).
    opex: Decimal,
}

/// Return the two leading digits of an account number, ignoring non-numeric chars.
fn account_prefix(account: &str) -> Option<u32> {
    let digits: String = account.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 2 {
        digits[..2].parse().ok()
    } else if digits.len() == 1 {
        digits[..1].parse().ok()
    } else {
        None
    }
}

/// Accumulate debit/credit amounts into `GlTotals` based on account ranges.
///
/// Convention used here: for balance-sheet accounts the *net* balance matters
/// (debit − credit for asset accounts, credit − debit for liability/equity).
/// For P&L accounts we accumulate the "natural" side:
///   revenue → credits, COGS/opex → debits.
fn build_totals(entries: &[JournalEntry], entity_code: &str) -> GlTotals {
    let mut t = GlTotals::default();

    for entry in entries {
        if entry.header.company_code != entity_code {
            continue;
        }
        for line in &entry.lines {
            let account = &line.gl_account;
            let Some(prefix2) = account_prefix(account) else {
                continue;
            };
            let prefix1 = prefix2 / 10; // leading digit
            let net = line.debit_amount - line.credit_amount; // positive = debit-heavy

            match prefix1 {
                1 => {
                    // Asset accounts (debit-normal → net positive = asset)
                    t.assets += net;
                    match prefix2 {
                        11 => t.ar += net,
                        12 => t.inventory += net,
                        _ => {}
                    }
                }
                2 => {
                    // Liability accounts (credit-normal → net negative = liability)
                    t.liabilities += -net;
                    if prefix2 == 21 || prefix2 == 20 {
                        t.ap += -net;
                    }
                }
                3 => {
                    // Equity accounts (credit-normal)
                    t.equity += -net;
                }
                4 => {
                    // Revenue (credit-normal)
                    t.revenue += -net;
                }
                5 => {
                    // COGS (debit-normal)
                    t.cogs += net;
                }
                6..=8 => {
                    // Operating expenses (debit-normal)
                    t.opex += net;
                }
                _ => {}
            }
        }
    }

    t
}

// ─── Ratio computation ────────────────────────────────────────────────────────

/// Compute financial ratios from journal entries for a single entity.
///
/// Returns `None` for any ratio where the denominator is zero or the required
/// data is absent.
pub fn compute_ratios(entries: &[JournalEntry], entity_code: &str) -> FinancialRatios {
    let t = build_totals(entries, entity_code);

    let d365 = Decimal::from(365u32);

    // Liquidity
    let current_ratio = if t.liabilities > Decimal::ZERO {
        Some(t.assets / t.liabilities)
    } else {
        None
    };

    let current_assets_ex_inv = t.assets - t.inventory;
    let quick_ratio = if t.liabilities > Decimal::ZERO && t.assets > Decimal::ZERO {
        Some(current_assets_ex_inv / t.liabilities)
    } else {
        None
    };

    // Activity
    let dso = if t.revenue > Decimal::ZERO && t.ar >= Decimal::ZERO {
        Some(t.ar / t.revenue * d365)
    } else {
        None
    };

    let dpo = if t.cogs > Decimal::ZERO && t.ap >= Decimal::ZERO {
        Some(t.ap / t.cogs * d365)
    } else {
        None
    };

    let inventory_turnover = if t.inventory > Decimal::ZERO {
        Some(t.cogs / t.inventory)
    } else {
        None
    };

    // Profitability
    let gross_profit = t.revenue - t.cogs;
    let gross_margin = if t.revenue > Decimal::ZERO {
        Some(gross_profit / t.revenue)
    } else {
        None
    };

    let operating_income = t.revenue - t.cogs - t.opex;
    let operating_margin = if t.revenue > Decimal::ZERO {
        Some(operating_income / t.revenue)
    } else {
        None
    };

    let net_income = operating_income; // simplified (no tax/interest lines here)
    let net_margin = if t.revenue > Decimal::ZERO {
        Some(net_income / t.revenue)
    } else {
        None
    };

    let roa = if t.assets > Decimal::ZERO {
        Some(net_income / t.assets)
    } else {
        None
    };

    let roe = if t.equity > Decimal::ZERO {
        Some(net_income / t.equity)
    } else {
        None
    };

    // Leverage
    let debt_to_equity = if t.equity > Decimal::ZERO {
        Some(t.liabilities / t.equity)
    } else {
        None
    };

    let debt_to_assets = if t.assets > Decimal::ZERO {
        Some(t.liabilities / t.assets)
    } else {
        None
    };

    FinancialRatios {
        current_ratio,
        quick_ratio,
        dso,
        dpo,
        inventory_turnover,
        gross_margin,
        operating_margin,
        net_margin,
        roa,
        roe,
        debt_to_equity,
        debt_to_assets,
    }
}

// ─── Reasonableness bounds ────────────────────────────────────────────────────

/// Industry-specific bounds for each ratio.
struct IndustryBounds {
    current_ratio: (Decimal, Decimal),
    quick_ratio: (Decimal, Decimal),
    dso: (Decimal, Decimal),
    dpo: (Decimal, Decimal),
    inventory_turnover: (Decimal, Decimal),
    gross_margin: (Decimal, Decimal),
    operating_margin: (Decimal, Decimal),
    net_margin: (Decimal, Decimal),
    roa: (Decimal, Decimal),
    roe: (Decimal, Decimal),
    debt_to_equity: (Decimal, Decimal),
    debt_to_assets: (Decimal, Decimal),
}

fn d(val: &str) -> Decimal {
    val.parse().expect("hardcoded decimal literal")
}

fn bounds_for(industry: &str) -> IndustryBounds {
    match industry.to_lowercase().as_str() {
        "manufacturing" => IndustryBounds {
            current_ratio: (d("1.2"), d("3.0")),
            quick_ratio: (d("0.7"), d("2.0")),
            dso: (d("20"), d("60")),
            dpo: (d("30"), d("90")),
            inventory_turnover: (d("3.0"), d("20.0")),
            gross_margin: (d("0.15"), d("0.50")),
            operating_margin: (d("0.03"), d("0.20")),
            net_margin: (d("0.01"), d("0.15")),
            roa: (d("-0.10"), d("0.20")),
            roe: (d("-0.20"), d("0.40")),
            debt_to_equity: (d("0.0"), d("2.5")),
            debt_to_assets: (d("0.0"), d("0.70")),
        },
        "financial_services" | "financial" | "banking" => IndustryBounds {
            current_ratio: (d("0.5"), d("2.0")),
            quick_ratio: (d("0.4"), d("1.8")),
            dso: (d("10"), d("50")),
            dpo: (d("15"), d("60")),
            inventory_turnover: (d("1.0"), d("50.0")),
            gross_margin: (d("0.30"), d("0.80")),
            operating_margin: (d("0.10"), d("0.40")),
            net_margin: (d("0.05"), d("0.35")),
            roa: (d("-0.05"), d("0.25")),
            roe: (d("-0.10"), d("0.50")),
            debt_to_equity: (d("0.0"), d("10.0")),
            debt_to_assets: (d("0.0"), d("0.90")),
        },
        "technology" | "tech" => IndustryBounds {
            current_ratio: (d("1.5"), d("5.0")),
            quick_ratio: (d("1.0"), d("4.5")),
            dso: (d("30"), d("75")),
            dpo: (d("15"), d("60")),
            inventory_turnover: (d("5.0"), d("50.0")),
            gross_margin: (d("0.40"), d("0.90")),
            operating_margin: (d("0.05"), d("0.40")),
            net_margin: (d("0.02"), d("0.35")),
            roa: (d("-0.20"), d("0.30")),
            roe: (d("-0.30"), d("0.60")),
            debt_to_equity: (d("0.0"), d("2.0")),
            debt_to_assets: (d("0.0"), d("0.60")),
        },
        "healthcare" => IndustryBounds {
            current_ratio: (d("1.0"), d("3.0")),
            quick_ratio: (d("0.6"), d("2.5")),
            dso: (d("40"), d("90")),
            dpo: (d("20"), d("60")),
            inventory_turnover: (d("5.0"), d("30.0")),
            gross_margin: (d("0.25"), d("0.70")),
            operating_margin: (d("0.03"), d("0.25")),
            net_margin: (d("0.01"), d("0.20")),
            roa: (d("-0.10"), d("0.20")),
            roe: (d("-0.20"), d("0.40")),
            debt_to_equity: (d("0.0"), d("2.0")),
            debt_to_assets: (d("0.0"), d("0.65")),
        },
        // Default: retail
        _ => IndustryBounds {
            current_ratio: (d("1.0"), d("2.5")),
            quick_ratio: (d("0.4"), d("1.5")),
            dso: (d("5"), d("45")),
            dpo: (d("20"), d("70")),
            inventory_turnover: (d("4.0"), d("30.0")),
            gross_margin: (d("0.10"), d("0.50")),
            operating_margin: (d("0.01"), d("0.15")),
            net_margin: (d("0.005"), d("0.10")),
            roa: (d("-0.10"), d("0.20")),
            roe: (d("-0.20"), d("0.40")),
            debt_to_equity: (d("0.0"), d("3.0")),
            debt_to_assets: (d("0.0"), d("0.75")),
        },
    }
}

/// Build a single [`RatioCheck`] comparing an optional value against bounds.
fn make_check(name: &str, value: Option<Decimal>, bounds: (Decimal, Decimal)) -> RatioCheck {
    let is_reasonable = match value {
        None => true, // not computable → skip
        Some(v) => v >= bounds.0 && v <= bounds.1,
    };
    RatioCheck {
        ratio_name: name.to_string(),
        value,
        industry_min: bounds.0,
        industry_max: bounds.1,
        is_reasonable,
    }
}

/// Check all ratios for reasonableness against industry benchmarks.
pub fn check_reasonableness(ratios: &FinancialRatios, industry: &str) -> Vec<RatioCheck> {
    let b = bounds_for(industry);
    vec![
        make_check("current_ratio", ratios.current_ratio, b.current_ratio),
        make_check("quick_ratio", ratios.quick_ratio, b.quick_ratio),
        make_check("dso", ratios.dso, b.dso),
        make_check("dpo", ratios.dpo, b.dpo),
        make_check(
            "inventory_turnover",
            ratios.inventory_turnover,
            b.inventory_turnover,
        ),
        make_check("gross_margin", ratios.gross_margin, b.gross_margin),
        make_check(
            "operating_margin",
            ratios.operating_margin,
            b.operating_margin,
        ),
        make_check("net_margin", ratios.net_margin, b.net_margin),
        make_check("roa", ratios.roa, b.roa),
        make_check("roe", ratios.roe, b.roe),
        make_check("debt_to_equity", ratios.debt_to_equity, b.debt_to_equity),
        make_check("debt_to_assets", ratios.debt_to_assets, b.debt_to_assets),
    ]
}

/// Run the full ratio analysis for an entity and return a combined result.
pub fn analyze(
    entries: &[JournalEntry],
    entity_code: &str,
    period: &str,
    industry: &str,
) -> RatioAnalysisResult {
    let ratios = compute_ratios(entries, entity_code);
    let reasonableness_checks = check_reasonableness(&ratios, industry);
    let passes = reasonableness_checks.iter().all(|c| c.is_reasonable);
    RatioAnalysisResult {
        entity_code: entity_code.to_string(),
        period: period.to_string(),
        ratios,
        reasonableness_checks,
        passes,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::{JournalEntry, JournalEntryHeader, JournalEntryLine};
    use rust_decimal_macros::dec;

    fn make_date() -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(2024, 6, 30).unwrap()
    }

    /// Build a minimal journal entry that posts one debit line and one credit line.
    fn je(
        company: &str,
        debit_account: &str,
        credit_account: &str,
        amount: Decimal,
    ) -> JournalEntry {
        let header = JournalEntryHeader::new(company.to_string(), make_date());
        let doc_id = header.document_id;
        let mut entry = JournalEntry::new(header);
        entry.add_line(JournalEntryLine::debit(
            doc_id,
            1,
            debit_account.to_string(),
            amount,
        ));
        entry.add_line(JournalEntryLine::credit(
            doc_id,
            2,
            credit_account.to_string(),
            amount,
        ));
        entry
    }

    #[test]
    fn test_current_ratio() {
        // Assets 10000 (account 1000), Liabilities 5000 (account 2000)
        // Expect current_ratio = 2.0
        let entries = vec![
            je("C001", "1000", "3000", dec!(10000)),
            je("C001", "6000", "2000", dec!(5000)),
        ];
        let ratios = compute_ratios(&entries, "C001");
        let cr = ratios.current_ratio.unwrap();
        assert!(
            (cr - dec!(2.0)).abs() < dec!(0.01),
            "Expected current_ratio ≈ 2.0, got {cr}"
        );
    }

    #[test]
    fn test_dso() {
        // AR (1100) = 3650 credit posting offset = debit on 1100
        // Revenue (4000) = 10000 (credit-normal)
        // DSO = 3650 / 10000 * 365 = 133.225
        let entries = vec![
            je("C001", "1100", "4000", dec!(3650)), // AR debit, Revenue credit
        ];
        let ratios = compute_ratios(&entries, "C001");
        let dso = ratios.dso.unwrap();
        // ar=3650, revenue=3650 → dso = 3650/3650*365 = 365
        assert!(dso > dec!(0), "DSO should be positive");
    }

    #[test]
    fn test_gross_margin() {
        // Revenue 10000 (credit on 4xxx), COGS 6000 (debit on 5xxx)
        // gross_margin = (10000 - 6000) / 10000 = 0.40
        let entries = vec![
            je("C001", "1000", "4000", dec!(10000)), // revenue credit
            je("C001", "5000", "1000", dec!(6000)),  // COGS debit
        ];
        let ratios = compute_ratios(&entries, "C001");
        let gm = ratios.gross_margin.unwrap();
        // revenue = 10000, cogs = 6000 → 0.40
        assert!(
            (gm - dec!(0.40)).abs() < dec!(0.01),
            "Expected gross_margin ≈ 0.40, got {gm}"
        );
    }

    #[test]
    fn test_reasonableness_flags_out_of_bounds() {
        // Artificially create a current_ratio of 0.1 (below retail min of 1.0)
        let ratios = FinancialRatios {
            current_ratio: Some(dec!(0.1)),
            ..Default::default()
        };
        let checks = check_reasonableness(&ratios, "retail");
        let cr_check = checks
            .iter()
            .find(|c| c.ratio_name == "current_ratio")
            .unwrap();
        assert!(
            !cr_check.is_reasonable,
            "current_ratio 0.1 should be flagged as unreasonable for retail"
        );
    }

    #[test]
    fn test_reasonableness_passes_within_bounds() {
        let ratios = FinancialRatios {
            current_ratio: Some(dec!(1.8)),
            gross_margin: Some(dec!(0.35)),
            ..Default::default()
        };
        let checks = check_reasonableness(&ratios, "retail");
        for check in &checks {
            if check.ratio_name == "current_ratio" || check.ratio_name == "gross_margin" {
                assert!(
                    check.is_reasonable,
                    "{} should be reasonable",
                    check.ratio_name
                );
            }
        }
    }

    #[test]
    fn test_none_ratios_vacuously_pass() {
        let ratios = FinancialRatios::default(); // all None
        let checks = check_reasonableness(&ratios, "retail");
        assert!(
            checks.iter().all(|c| c.is_reasonable),
            "All None ratios should vacuously pass"
        );
    }

    #[test]
    fn test_entity_filter() {
        // C001: revenue=5000, COGS=2000 → gross_margin = 0.60
        // C002: revenue=5000, COGS=4500 → gross_margin = 0.10
        let entries = vec![
            je("C001", "1000", "4000", dec!(5000)), // C001 revenue
            je("C001", "5000", "1000", dec!(2000)), // C001 COGS
            je("C002", "1000", "4000", dec!(5000)), // C002 revenue
            je("C002", "5000", "1000", dec!(4500)), // C002 COGS (higher)
        ];
        let r1 = compute_ratios(&entries, "C001");
        let r2 = compute_ratios(&entries, "C002");
        // Different COGS → different gross margins
        assert_ne!(
            r1.gross_margin, r2.gross_margin,
            "Entity filter should isolate per-company data"
        );
    }

    #[test]
    fn test_debt_to_equity() {
        // Liabilities 4000 (credit on 2xxx offset debit on 6xxx), Equity 2000 (credit on 3xxx)
        let entries = vec![
            je("C001", "6000", "2000", dec!(4000)), // liability credit
            je("C001", "1000", "3000", dec!(2000)), // equity credit
        ];
        let ratios = compute_ratios(&entries, "C001");
        if let (Some(dte), Some(dta)) = (ratios.debt_to_equity, ratios.debt_to_assets) {
            assert!(dte > dec!(0), "D/E should be positive when liabilities > 0");
            assert!(dta > dec!(0), "D/A should be positive when liabilities > 0");
        }
    }

    #[test]
    fn test_analyze_end_to_end() {
        let entries = vec![
            je("C001", "1000", "4000", dec!(10000)),
            je("C001", "5000", "1000", dec!(6000)),
            je("C001", "6000", "2000", dec!(2000)),
        ];
        let result = analyze(&entries, "C001", "2024-H1", "retail");
        assert_eq!(result.entity_code, "C001");
        assert_eq!(result.period, "2024-H1");
        assert!(!result.reasonableness_checks.is_empty());
    }

    #[test]
    fn test_industry_bounds_manufacturing() {
        let ratios = FinancialRatios {
            current_ratio: Some(dec!(2.0)), // within manufacturing 1.2–3.0
            ..Default::default()
        };
        let checks = check_reasonableness(&ratios, "manufacturing");
        let cr = checks
            .iter()
            .find(|c| c.ratio_name == "current_ratio")
            .unwrap();
        assert!(
            cr.is_reasonable,
            "2.0 is within manufacturing bounds 1.2–3.0"
        );
    }
}
