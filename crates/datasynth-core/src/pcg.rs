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

    /// Immobilisations (Fixed assets) – Class 2
    pub const FIXED_ASSETS: &str = "215000";

    /// Amortissements (Accumulated depreciation) – Class 2
    pub const ACCUMULATED_DEPRECIATION: &str = "281000";

    /// GR/IR clearing – Class 4
    pub const GR_IR_CLEARING: &str = "408000";

    /// Intercompany AR – Class 4
    pub const IC_AR_CLEARING: &str = "411800";

    /// Intercompany AP – Class 4
    pub const IC_AP_CLEARING: &str = "401800";
}

/// Fixed asset subclass accounts (PCG 2024) for granular US→PCG mapping.
pub mod fixed_asset_accounts {
    /// Terrains (211)
    pub const TERRAINS: &str = "211000";
    /// Constructions (213)
    pub const CONSTRUCTIONS: &str = "213000";
    /// Installations techniques, matériel et outillage industriels (215)
    pub const INDUSTRIAL: &str = "215000";
    /// Matériel de transport (2182)
    pub const TRANSPORT: &str = "218200";
    /// Matériel de bureau et matériel informatique (2183)
    pub const OFFICE_IT: &str = "218300";
    /// Mobilier (2184)
    pub const FURNITURE: &str = "218400";
    /// Installations générales, agencements (2181)
    pub const LEASEHOLD: &str = "218100";
    /// Immobilisations corporelles en cours (231)
    pub const CIP: &str = "231000";
    /// Fournisseurs d'immobilisations (404) – liability/clearing for FA acquisition
    pub const SUPPLIERS_IMMO: &str = "404000";
}

/// Tax accounts (PCG 2024).
pub mod tax_accounts {
    /// Prélèvements à la source / Withholding tax (4421) – not VAT (445)
    pub const WHT: &str = "442100";
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
    /// Produits des cessions d'éléments d'actif (775) – gain on sale of assets, for capital gains
    pub const ASSET_DISPOSAL_PROCEEDS: &str = "775000";
}

/// Expenses – Class 6 (charges)
pub mod expense_accounts {
    pub const COGS: &str = "603000";
    /// Achats stockés / raw materials (Class 6)
    pub const RAW_MATERIALS: &str = "601000";
    /// Dotations aux amortissements (681) – generic
    pub const DEPRECIATION: &str = "681000";
    /// Dotations aux amortissements sur immobilisations (6811) – operating
    pub const DEPRECIATION_OPERATING: &str = "681100";
    /// Valeurs comptables des éléments d'actif cédés (675) – disposal, not recurring depreciation
    pub const DISPOSAL_VALUE: &str = "675000";
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

/// PCG has no class 9 suspense; use personnel (421) for payroll credit.
pub mod suspense_accounts {
    /// Payroll: credit wages payable (421000) instead of a clearing account
    pub const PAYROLL_CLEARING: &str = "421000";
}

/// Return the PCG class (1–9) from a 6-digit account number.
#[inline]
pub fn pcg_class(account: &str) -> Option<u8> {
    let first = account.chars().next()?;
    first.to_digit(10).map(|d| d as u8)
}
