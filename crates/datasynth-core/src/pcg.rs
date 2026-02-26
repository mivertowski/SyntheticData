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

/// Fixed asset sub-accounts – Class 2.
pub mod fixed_asset_accounts {
    /// Terrains (Land)
    pub const TERRAINS: &str = "211000";
    /// Constructions (Buildings)
    pub const CONSTRUCTIONS: &str = "213000";
    /// Installations techniques, matériel et outillage industriels
    pub const INDUSTRIAL: &str = "215000";
    /// Matériel de transport (Vehicles)
    pub const TRANSPORT: &str = "218200";
    /// Matériel de bureau (Office Equipment)
    pub const OFFICE_EQUIPMENT: &str = "218300";
    /// Matériel informatique (IT Equipment)
    pub const IT_EQUIPMENT: &str = "218400";
}

/// Tax accounts – Classes 4 & 6.
pub mod tax_accounts {
    /// TVA déductible sur biens et services (Input VAT)
    pub const INPUT_VAT: &str = "445660";
    /// TVA collectée (Output VAT)
    pub const OUTPUT_VAT: &str = "445710";
    /// Retenue à la source (Withholding Tax Payable)
    pub const WHT_PAYABLE: &str = "442100";
    /// Crédit d'impôt (Tax Receivable)
    pub const TAX_RECEIVABLE: &str = "443000";
    /// Impôt sur les bénéfices (Tax Expense)
    pub const TAX_EXPENSE: &str = "695000";
    /// Provisions pour impôts différés passifs (Deferred Tax Liability)
    pub const DEFERRED_TAX_LIABILITY: &str = "155000";
    /// Charges à répartir / actif impôt différé (Deferred Tax Asset)
    pub const DEFERRED_TAX_ASSET: &str = "481000";
}

/// Suspense and clearing accounts.
pub mod suspense_accounts {
    /// Personnel – Rémunérations dues (Payroll Clearing)
    pub const PAYROLL_CLEARING: &str = "421000";
    /// Comptes d'attente (General Suspense)
    pub const GENERAL_SUSPENSE: &str = "471000";
}

/// Additional revenue accounts – Class 7.
pub mod additional_revenue {
    /// Produits intercompany (IC Revenue)
    pub const IC_REVENUE: &str = "757000";
    /// Escomptes obtenus / rabais sur achats (Purchase Discount Income)
    pub const PURCHASE_DISCOUNT_INCOME: &str = "765000";
    /// Rabais, remises et ristournes accordés (Sales Returns)
    pub const SALES_RETURNS: &str = "709100";
}

/// Additional expense accounts – Class 6.
pub mod additional_expense {
    /// Achats de matières premières (Raw Materials)
    pub const RAW_MATERIALS: &str = "601000";
    /// Rabais, remises sur achats (Purchase Discounts)
    pub const PURCHASE_DISCOUNTS: &str = "609000";
    /// Pertes de change (FX Gain/Loss)
    pub const FX_GAIN_LOSS: &str = "666000";
    /// Pertes sur créances irrécouvrables (Bad Debt)
    pub const BAD_DEBT: &str = "654000";
}

/// Liability accounts – Class 4.
pub mod liability_accounts {
    /// Charges à payer (Accrued Expenses)
    pub const ACCRUED_EXPENSES: &str = "428000";
    /// Charges de personnel à payer (Accrued Salaries)
    pub const ACCRUED_SALARIES: &str = "428400";
    /// Produits constatés d'avance (Unearned Revenue)
    pub const UNEARNED_REVENUE: &str = "487000";
    /// Comptes courants des associés / IC Payable (Group Companies)
    pub const IC_PAYABLE: &str = "451000";
}

/// Additional equity accounts – Class 1.
pub mod equity_accounts {
    /// Prime d'émission (Additional Paid-In Capital)
    pub const APIC: &str = "104000";
    /// Résultat de l'exercice (Current Year Earnings)
    pub const CURRENT_YEAR_EARNINGS: &str = "120000";
    /// Écart de conversion (Currency Translation Adjustment)
    pub const CTA: &str = "107000";
    /// Solde intermédiaire de gestion (Income Summary)
    pub const INCOME_SUMMARY: &str = "129900";
    /// Associés – dividendes à payer (Dividends Paid)
    pub const DIVIDENDS_PAID: &str = "457000";
}

/// Return the PCG class (1–9) from a 6-digit account number.
#[inline]
pub fn pcg_class(account: &str) -> Option<u8> {
    let first = account.chars().next()?;
    first.to_digit(10).map(|d| d as u8)
}
