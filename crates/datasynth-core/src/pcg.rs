//! Plan Comptable Général (PCG) – French GAAP chart of accounts constants.
//!
//! PCG uses a decimal classification with classes 1–9:
//! - Class 1: Equity and liabilities (capitaux)
//! - Class 2: Fixed assets (immobilisations)
//! - Class 3: Inventory and work in progress (stocks)
//! - Class 4: Receivables and payables (tiers)
//! - Class 5: Financial / cash (financiers)
//! - Class 6: Expenses (charges)
//! - Class 7: Income (produits)
//! - Class 8: Special accounts (comptes spéciaux)
//! - Class 9: Analytical accounts (comptes analytiques)

/// PCG control and main account ranges (6-digit base).
/// First digit = class; next two = subclass; last three = account.
///
/// L’idée de base:
/// Un bail peut être compté de deux façons :
/// - Bail en « exploitation » (operating) —
///   On enregistre surtout les loyers en charges au fil du temps.
/// - Bail « finance » (finance) —
///   On considère qu’on « possède presque » l’actif.
///
/// La classification dépend du référentiel (US GAAP, IFRS, French GAAP).
pub mod control_accounts {
    /// Clients (Accounts Receivable) – Class 4
    pub const AR_CONTROL: &str = "411000";

    /// Fournisseurs (Accounts Payable) – Class 4
    pub const AP_CONTROL: &str = "401000";

    /// Inventories – Class 3
    pub const INVENTORY: &str = "310000";

    /// Immobilisations corporelles (Fixed assets) – Class 2
    pub const FIXED_ASSETS: &str = "210000";

    /// Amortissements (Accumulated depreciation) – Class 2
    pub const ACCUMULATED_DEPRECIATION: &str = "281000";

    /// GR/IR clearing – Class 4
    pub const GR_IR_CLEARING: &str = "408000";

    /// Intercompany AR – Class 4
    pub const IC_AR_CLEARING: &str = "411800";

    /// Intercompany AP – Class 4
    pub const IC_AP_CLEARING: &str = "401800";
}

/// Cash and bank – Class 5
pub mod cash_accounts {
    pub const OPERATING_CASH: &str = "530000";
    pub const BANK_ACCOUNT: &str = "512000";
    pub const PETTY_CASH: &str = "531000";
}

/// Revenue – Class 7 (produits)
pub mod revenue_accounts {
    pub const PRODUCT_REVENUE: &str = "701000";
    pub const SERVICE_REVENUE: &str = "706000";
    pub const OTHER_REVENUE: &str = "758000";
    pub const SALES_DISCOUNTS: &str = "709000";
}

/// Expenses – Class 6 (charges)
pub mod expense_accounts {
    pub const COGS: &str = "603000";
    pub const DEPRECIATION: &str = "681000";
    pub const SALARIES_WAGES: &str = "641100";
    pub const RENT: &str = "613000";
    pub const INTEREST_EXPENSE: &str = "661000";
}

/// Equity and liabilities – Class 1
pub mod equity_liability_accounts {
    pub const COMMON_STOCK: &str = "101000";
    pub const RETAINED_EARNINGS: &str = "129000";
    pub const PROVISIONS: &str = "151000";
    pub const SHORT_TERM_DEBT: &str = "164000";
    pub const LONG_TERM_DEBT: &str = "163000";
}

/// Personnel – Class 4 (sub-class 42)
pub mod personnel_accounts {
    pub const WAGES_PAYABLE: &str = "421000";
}

/// Return the PCG class (1–9) from a 6-digit account number.
#[inline]
pub fn pcg_class(account: &str) -> Option<u8> {
    let first = account.chars().next()?;
    first.to_digit(10).map(|d| d as u8)
}
