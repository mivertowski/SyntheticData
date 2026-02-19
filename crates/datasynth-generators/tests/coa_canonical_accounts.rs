use datasynth_core::accounts::{
    cash_accounts, control_accounts, equity_accounts, expense_accounts, liability_accounts,
    revenue_accounts, suspense_accounts, tax_accounts,
};
use datasynth_core::models::{CoAComplexity, IndustrySector};

/// Verify the CoA contains all canonical accounts from accounts.rs.
#[test]
fn test_coa_contains_canonical_accounts() {
    let mut gen = datasynth_generators::ChartOfAccountsGenerator::new(
        CoAComplexity::Small,
        IndustrySector::Manufacturing,
        42,
    );
    let coa = gen.generate();

    let canonical = vec![
        // Cash accounts
        cash_accounts::OPERATING_CASH,
        cash_accounts::BANK_ACCOUNT,
        cash_accounts::PETTY_CASH,
        cash_accounts::WIRE_CLEARING,
        // Control accounts
        control_accounts::AR_CONTROL,
        control_accounts::AP_CONTROL,
        control_accounts::INVENTORY,
        control_accounts::FIXED_ASSETS,
        control_accounts::ACCUMULATED_DEPRECIATION,
        control_accounts::GR_IR_CLEARING,
        control_accounts::IC_AR_CLEARING,
        control_accounts::IC_AP_CLEARING,
        // Tax accounts
        tax_accounts::SALES_TAX_PAYABLE,
        tax_accounts::VAT_PAYABLE,
        tax_accounts::WITHHOLDING_TAX_PAYABLE,
        tax_accounts::INPUT_VAT,
        tax_accounts::TAX_EXPENSE,
        tax_accounts::DEFERRED_TAX_LIABILITY,
        tax_accounts::DEFERRED_TAX_ASSET,
        // Liability accounts
        liability_accounts::ACCRUED_EXPENSES,
        liability_accounts::ACCRUED_SALARIES,
        liability_accounts::ACCRUED_BENEFITS,
        liability_accounts::UNEARNED_REVENUE,
        liability_accounts::SHORT_TERM_DEBT,
        liability_accounts::LONG_TERM_DEBT,
        liability_accounts::IC_PAYABLE,
        // Revenue accounts
        revenue_accounts::PRODUCT_REVENUE,
        revenue_accounts::SERVICE_REVENUE,
        revenue_accounts::IC_REVENUE,
        revenue_accounts::OTHER_REVENUE,
        revenue_accounts::SALES_DISCOUNTS,
        revenue_accounts::SALES_RETURNS,
        // Expense accounts
        expense_accounts::COGS,
        expense_accounts::RAW_MATERIALS,
        expense_accounts::DIRECT_LABOR,
        expense_accounts::MANUFACTURING_OVERHEAD,
        expense_accounts::DEPRECIATION,
        expense_accounts::SALARIES_WAGES,
        expense_accounts::BENEFITS,
        expense_accounts::RENT,
        expense_accounts::UTILITIES,
        expense_accounts::OFFICE_SUPPLIES,
        expense_accounts::TRAVEL_ENTERTAINMENT,
        expense_accounts::PROFESSIONAL_FEES,
        expense_accounts::INSURANCE,
        expense_accounts::BAD_DEBT,
        expense_accounts::INTEREST_EXPENSE,
        expense_accounts::PURCHASE_DISCOUNTS,
        expense_accounts::FX_GAIN_LOSS,
        // Equity accounts
        equity_accounts::COMMON_STOCK,
        equity_accounts::APIC,
        equity_accounts::RETAINED_EARNINGS,
        equity_accounts::CURRENT_YEAR_EARNINGS,
        equity_accounts::TREASURY_STOCK,
        equity_accounts::CTA,
        // Suspense accounts
        suspense_accounts::GENERAL_SUSPENSE,
        suspense_accounts::PAYROLL_CLEARING,
        suspense_accounts::BANK_RECONCILIATION_SUSPENSE,
        suspense_accounts::IC_ELIMINATION_SUSPENSE,
    ];

    for account_num in &canonical {
        assert!(
            coa.accounts
                .iter()
                .any(|a| a.account_number == *account_num),
            "CoA missing canonical account: {}",
            account_num
        );
    }
}

/// Verify canonical accounts coexist with auto-generated accounts.
#[test]
fn test_canonical_accounts_do_not_replace_generated() {
    let mut gen = datasynth_generators::ChartOfAccountsGenerator::new(
        CoAComplexity::Small,
        IndustrySector::Manufacturing,
        42,
    );
    let coa = gen.generate();

    // Canonical accounts are in the 1000-9300 range (4-digit numbers).
    // Auto-generated accounts start at 100000+ (6-digit numbers).
    let has_canonical = coa.accounts.iter().any(|a| a.account_number.len() == 4);
    let has_generated = coa.accounts.iter().any(|a| a.account_number.len() == 6);

    assert!(
        has_canonical,
        "CoA should contain 4-digit canonical accounts"
    );
    assert!(
        has_generated,
        "CoA should still contain 6-digit auto-generated accounts"
    );
}

/// Verify canonical accounts can be retrieved by account number.
#[test]
fn test_canonical_accounts_are_indexed() {
    let mut gen = datasynth_generators::ChartOfAccountsGenerator::new(
        CoAComplexity::Small,
        IndustrySector::Manufacturing,
        42,
    );
    let coa = gen.generate();

    // Should be able to look up canonical accounts directly
    assert!(
        coa.get_account(control_accounts::AR_CONTROL).is_some(),
        "AR_CONTROL should be findable via get_account"
    );
    assert!(
        coa.get_account(cash_accounts::OPERATING_CASH).is_some(),
        "OPERATING_CASH should be findable via get_account"
    );
    assert!(
        coa.get_account(expense_accounts::COGS).is_some(),
        "COGS should be findable via get_account"
    );
}
