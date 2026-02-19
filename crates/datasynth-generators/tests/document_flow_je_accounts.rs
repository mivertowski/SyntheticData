use datasynth_core::accounts::{
    cash_accounts, control_accounts, expense_accounts, revenue_accounts,
};

/// Verify that DocumentFlowJeConfig defaults use centralized account constants.
#[test]
fn test_document_flow_je_config_uses_central_accounts() {
    let config = datasynth_generators::document_flow::DocumentFlowJeConfig::default();

    assert_eq!(config.ar_account, control_accounts::AR_CONTROL);
    assert_eq!(config.ap_account, control_accounts::AP_CONTROL);
    assert_eq!(config.inventory_account, control_accounts::INVENTORY);
    assert_eq!(
        config.gr_ir_clearing_account,
        control_accounts::GR_IR_CLEARING
    );
    assert_eq!(config.cash_account, cash_accounts::OPERATING_CASH);
    assert_eq!(config.revenue_account, revenue_accounts::PRODUCT_REVENUE);
    assert_eq!(config.cogs_account, expense_accounts::COGS);
}
