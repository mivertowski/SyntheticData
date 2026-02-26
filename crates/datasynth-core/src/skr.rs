//! Standardkontenrahmen 04 (SKR04) – German GAAP (HGB) chart of accounts constants.
//!
//! SKR04 follows the Abschlussgliederungsprinzip (financial statement structure):
//! - Class 0: Fixed assets (Anlagevermögen)
//! - Class 1: Current assets (Umlaufvermögen)
//! - Class 2: Equity (Eigenkapital)
//! - Class 3: Liabilities (Fremdkapital)
//! - Class 4: Revenue (Betriebliche Erträge)
//! - Class 5: Material / COGS (Materialaufwand)
//! - Class 6: Personnel & operating expenses (Personalaufwand, sonstige betriebliche Aufwendungen)
//! - Class 7: Financial income and expenses (Finanzerträge und -aufwendungen)
//! - Class 8: Extraordinary / taxes (Außerordentliches Ergebnis, Steuern)
//! - Class 9: Statistical / internal (Statistische Konten)

/// Control accounts for subledger integration.
pub mod control_accounts {
    /// Forderungen aus Lieferungen und Leistungen (Accounts Receivable)
    pub const AR_CONTROL: &str = "1200";

    /// Verbindlichkeiten aus Lieferungen und Leistungen (Accounts Payable)
    pub const AP_CONTROL: &str = "3300";

    /// Vorräte — Roh-, Hilfs- und Betriebsstoffe (Inventory)
    pub const INVENTORY: &str = "1100";

    /// Sachanlagen (Fixed Assets — Property, Plant & Equipment)
    pub const FIXED_ASSETS: &str = "0200";

    /// Kumulierte Abschreibungen auf Sachanlagen (Accumulated Depreciation)
    pub const ACCUMULATED_DEPRECIATION: &str = "0700";

    /// Wareneingangs-/Rechnungseingangs-Verrechnungskonto (GR/IR Clearing)
    pub const GR_IR_CLEARING: &str = "3350";

    /// Forderungen gegen verbundene Unternehmen (IC AR Clearing)
    pub const IC_AR_CLEARING: &str = "1400";

    /// Verbindlichkeiten gegenüber verbundenen Unternehmen (IC AP Clearing)
    pub const IC_AP_CLEARING: &str = "3500";
}

/// Cash and bank accounts — Class 1.
pub mod cash_accounts {
    /// Bank (Primary bank account)
    pub const OPERATING_CASH: &str = "1800";

    /// Bankkonten (Secondary bank accounts)
    pub const BANK_ACCOUNT: &str = "1810";

    /// Kasse (Petty cash)
    pub const PETTY_CASH: &str = "1600";
}

/// Revenue accounts — Class 4.
pub mod revenue_accounts {
    /// Umsatzerlöse für eigene Erzeugnisse (Product Revenue)
    pub const PRODUCT_REVENUE: &str = "4000";

    /// Erlöse aus Leistungen (Service Revenue)
    pub const SERVICE_REVENUE: &str = "4400";

    /// Erlöse aus Lieferungen an verbundene Unternehmen (IC Revenue)
    pub const IC_REVENUE: &str = "4500";

    /// Skontoerträge (Purchase Discount Income)
    pub const PURCHASE_DISCOUNT_INCOME: &str = "4730";

    /// Sonstige betriebliche Erträge (Other Revenue)
    pub const OTHER_REVENUE: &str = "4900";

    /// Erlösschmälerungen / Skonti (Sales Discounts)
    pub const SALES_DISCOUNTS: &str = "4720";

    /// Erlösminderungen / Retouren (Sales Returns)
    pub const SALES_RETURNS: &str = "4710";
}

/// Expense accounts — Classes 5-6.
pub mod expense_accounts {
    /// Materialaufwand (Cost of Goods Sold / Material)
    pub const COGS: &str = "5000";

    /// Aufwendungen für Roh-, Hilfs- und Betriebsstoffe (Raw Materials)
    pub const RAW_MATERIALS: &str = "5100";

    /// Abschreibungen auf Sachanlagen (Depreciation Expense)
    pub const DEPRECIATION: &str = "6220";

    /// Löhne und Gehälter (Salaries and Wages)
    pub const SALARIES_WAGES: &str = "6000";

    /// Miete und Nebenkosten (Rent)
    pub const RENT: &str = "6310";

    /// Zinsaufwendungen (Interest Expense) — Class 7
    pub const INTEREST_EXPENSE: &str = "7300";

    /// Skonti (Purchase Discounts)
    pub const PURCHASE_DISCOUNTS: &str = "5730";

    /// Kursverluste / Kursgewinne (FX Gain/Loss) — Class 7
    pub const FX_GAIN_LOSS: &str = "7400";

    /// Abschreibungen auf Forderungen (Bad Debt Expense)
    pub const BAD_DEBT: &str = "6340";
}

/// Tax accounts.
pub mod tax_accounts {
    /// Umsatzsteuer (Output VAT) — Class 3
    pub const OUTPUT_VAT: &str = "3800";

    /// Vorsteuer (Input VAT) — Class 1
    pub const INPUT_VAT: &str = "1570";

    /// Steuerrückstellungen (Tax Receivable) — Class 1
    pub const TAX_RECEIVABLE: &str = "1550";

    /// Steuern vom Einkommen und Ertrag (Tax Expense) — Class 8
    pub const TAX_EXPENSE: &str = "7600";

    /// Passive latente Steuern (Deferred Tax Liability)
    pub const DEFERRED_TAX_LIABILITY: &str = "3060";

    /// Aktive latente Steuern (Deferred Tax Asset)
    pub const DEFERRED_TAX_ASSET: &str = "1550";
}

/// Liability accounts — Class 3.
pub mod liability_accounts {
    /// Sonstige Rückstellungen (Accrued Expenses / Other Provisions)
    pub const ACCRUED_EXPENSES: &str = "3070";

    /// Verbindlichkeiten aus Lohn und Gehalt (Accrued Salaries)
    pub const ACCRUED_SALARIES: &str = "3720";

    /// Erhaltene Anzahlungen auf Bestellungen (Unearned Revenue)
    pub const UNEARNED_REVENUE: &str = "3250";

    /// Kurzfristige Verbindlichkeiten gg. Kreditinstituten (Short-Term Debt)
    pub const SHORT_TERM_DEBT: &str = "3100";

    /// Langfristige Verbindlichkeiten gg. Kreditinstituten (Long-Term Debt)
    pub const LONG_TERM_DEBT: &str = "3150";

    /// Verbindlichkeiten gegenüber verbundenen Unternehmen (IC Payable)
    pub const IC_PAYABLE: &str = "3510";
}

/// Equity accounts — Class 2.
pub mod equity_accounts {
    /// Gezeichnetes Kapital (Common Stock / Subscribed Capital)
    pub const COMMON_STOCK: &str = "2000";

    /// Gewinnvortrag (Retained Earnings)
    pub const RETAINED_EARNINGS: &str = "2970";

    /// Jahresüberschuss / -fehlbetrag (Current Year Earnings)
    pub const CURRENT_YEAR_EARNINGS: &str = "2960";

    /// Währungsumrechnungsdifferenzen (Currency Translation Adjustment)
    pub const CTA: &str = "2909";

    /// Schlussbilanzkonto / Ergebniskonto (Income Summary)
    pub const INCOME_SUMMARY: &str = "2990";

    /// Ausschüttungen / Gewinnverwendung (Dividends Paid)
    pub const DIVIDENDS_PAID: &str = "2980";

    /// Rückstellungen (Provisions — HGB-specific) — nominally Class 3
    pub const PROVISIONS: &str = "3000";
}

/// Suspense and clearing accounts — Class 9.
pub mod suspense_accounts {
    /// Allgemeines Verrechnungskonto (General Suspense)
    pub const GENERAL_SUSPENSE: &str = "9000";

    /// Verrechnungskonto Lohn/Gehalt (Payroll Clearing)
    pub const PAYROLL_CLEARING: &str = "9100";

    /// Bankabstimmungskonto (Bank Reconciliation Suspense)
    pub const BANK_RECONCILIATION_SUSPENSE: &str = "9200";
}

/// Personnel accounts — Class 6.
pub mod personnel_accounts {
    /// Verbindlichkeiten aus Lohn und Gehalt (Wages Payable)
    pub const WAGES_PAYABLE: &str = "3720";

    /// Soziale Abgaben und Aufwendungen (Social Security)
    pub const SOCIAL_SECURITY: &str = "6100";
}

/// Fixed asset sub-accounts — Class 0.
pub mod fixed_asset_accounts {
    /// Grundstücke (Land)
    pub const LAND: &str = "0060";

    /// Gebäude (Buildings)
    pub const BUILDINGS: &str = "0090";

    /// Technische Anlagen und Maschinen (Machinery)
    pub const MACHINERY: &str = "0200";

    /// Fuhrpark (Vehicles)
    pub const VEHICLES: &str = "0320";

    /// Betriebs- und Geschäftsausstattung (Office Equipment)
    pub const OFFICE_EQUIPMENT: &str = "0400";

    /// EDV-Anlagen (IT Equipment)
    pub const IT_EQUIPMENT: &str = "0420";

    /// Geringwertige Wirtschaftsgüter (GWG — Low-Value Assets)
    pub const GWG: &str = "0480";
}

/// Return the SKR04 class (0–9) from a 4-digit account number.
#[inline]
pub fn skr_class(account: &str) -> Option<u8> {
    let first = account.chars().next()?;
    first.to_digit(10).map(|d| d as u8)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_skr_class_extraction() {
        assert_eq!(skr_class("1200"), Some(1));
        assert_eq!(skr_class("3300"), Some(3));
        assert_eq!(skr_class("0200"), Some(0));
        assert_eq!(skr_class("4000"), Some(4));
        assert_eq!(skr_class("9000"), Some(9));
        assert_eq!(skr_class(""), None);
    }

    #[test]
    fn test_control_accounts() {
        assert_eq!(control_accounts::AR_CONTROL, "1200");
        assert_eq!(control_accounts::AP_CONTROL, "3300");
        assert_eq!(control_accounts::INVENTORY, "1100");
    }

    #[test]
    fn test_all_accounts_are_4_digit() {
        let accounts = [
            control_accounts::AR_CONTROL,
            control_accounts::AP_CONTROL,
            control_accounts::INVENTORY,
            control_accounts::FIXED_ASSETS,
            cash_accounts::OPERATING_CASH,
            revenue_accounts::PRODUCT_REVENUE,
            expense_accounts::COGS,
            expense_accounts::SALARIES_WAGES,
            equity_accounts::COMMON_STOCK,
            equity_accounts::RETAINED_EARNINGS,
        ];
        for acct in accounts {
            assert_eq!(acct.len(), 4, "SKR04 account {} should be 4 digits", acct);
        }
    }
}
