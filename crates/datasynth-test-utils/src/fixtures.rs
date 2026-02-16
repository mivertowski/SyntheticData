//! Pre-built test fixtures and configurations.

use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use datasynth_config::schema::{
    AccountingStandardsConfig, AuditGenerationConfig, AuditStandardsConfig,
    BehavioralDriftSchemaConfig, ChartOfAccountsConfig, CompanyConfig, ComplianceSchemaConfig,
    CrossProcessLinksSchemaConfig, CustomerSegmentationSchemaConfig, DataQualitySchemaConfig,
    DriftLabelingSchemaConfig, FingerprintPrivacyConfig, FraudConfig, GeneratorConfig,
    GlobalConfig, GraphExportConfig, IndustrySpecificConfig, MarketDriftSchemaConfig, OcpmConfig,
    OrganizationalEventsSchemaConfig, OutputConfig, QualityGatesSchemaConfig,
    RateLimitSchemaConfig, RelationshipSchemaConfig, RelationshipStrengthSchemaConfig,
    ScenarioConfig, StreamingSchemaConfig, TemporalAttributeSchemaConfig, TransactionVolume,
    VendorNetworkSchemaConfig,
};
use datasynth_core::models::{
    AccountSubType, AccountType, BusinessProcess, CoAComplexity, GLAccount, IndustrySector,
    JournalEntry, JournalEntryHeader, JournalEntryLine, TransactionSource,
};

/// Create a minimal test configuration.
pub fn minimal_config() -> GeneratorConfig {
    GeneratorConfig {
        global: GlobalConfig {
            seed: Some(42),
            industry: IndustrySector::Manufacturing,
            start_date: "2024-01-01".to_string(),
            period_months: 1,
            group_currency: "USD".to_string(),
            parallel: false,
            worker_threads: 0,
            memory_limit_mb: 0,
        },
        companies: vec![CompanyConfig {
            code: "TEST".to_string(),
            name: "Test Company".to_string(),
            currency: "USD".to_string(),
            country: "US".to_string(),
            annual_transaction_volume: TransactionVolume::TenK,
            volume_weight: 1.0,
            fiscal_year_variant: "K4".to_string(),
        }],
        chart_of_accounts: ChartOfAccountsConfig {
            complexity: CoAComplexity::Small,
            industry_specific: false,
            custom_accounts: None,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 3,
        },
        transactions: Default::default(),
        output: OutputConfig::default(),
        fraud: FraudConfig {
            enabled: false,
            ..Default::default()
        },
        internal_controls: Default::default(),
        business_processes: Default::default(),
        user_personas: Default::default(),
        templates: Default::default(),
        approval: Default::default(),
        departments: Default::default(),
        master_data: Default::default(),
        document_flows: Default::default(),
        intercompany: Default::default(),
        balance: Default::default(),
        ocpm: OcpmConfig::default(),
        audit: AuditGenerationConfig::default(),
        banking: datasynth_banking::BankingConfig::default(),
        data_quality: DataQualitySchemaConfig::default(),
        scenario: ScenarioConfig::default(),
        temporal: Default::default(),
        graph_export: GraphExportConfig::default(),
        streaming: StreamingSchemaConfig::default(),
        rate_limit: RateLimitSchemaConfig::default(),
        temporal_attributes: TemporalAttributeSchemaConfig::default(),
        relationships: RelationshipSchemaConfig::default(),
        accounting_standards: AccountingStandardsConfig::default(),
        audit_standards: AuditStandardsConfig::default(),
        distributions: Default::default(),
        temporal_patterns: Default::default(),
        vendor_network: VendorNetworkSchemaConfig::default(),
        customer_segmentation: CustomerSegmentationSchemaConfig::default(),
        relationship_strength: RelationshipStrengthSchemaConfig::default(),
        cross_process_links: CrossProcessLinksSchemaConfig::default(),
        organizational_events: OrganizationalEventsSchemaConfig::default(),
        behavioral_drift: BehavioralDriftSchemaConfig::default(),
        market_drift: MarketDriftSchemaConfig::default(),
        drift_labeling: DriftLabelingSchemaConfig::default(),
        anomaly_injection: Default::default(),
        industry_specific: IndustrySpecificConfig::default(),
        fingerprint_privacy: FingerprintPrivacyConfig::default(),
        quality_gates: QualityGatesSchemaConfig::default(),
        compliance: ComplianceSchemaConfig::default(),
        webhooks: Default::default(),
        llm: Default::default(),
        diffusion: Default::default(),
        causal: Default::default(),
        source_to_pay: Default::default(),
        financial_reporting: Default::default(),
        hr: Default::default(),
        manufacturing: Default::default(),
        sales_quotes: Default::default(),
        tax: Default::default(),
        treasury: Default::default(),
        project_accounting: Default::default(),
        esg: Default::default(),
    }
}

/// Create a test configuration with fraud enabled.
pub fn fraud_enabled_config() -> GeneratorConfig {
    let mut config = minimal_config();
    config.fraud.enabled = true;
    config.fraud.fraud_rate = 0.1;
    config
}

/// Create a test configuration for multi-company scenarios.
pub fn multi_company_config() -> GeneratorConfig {
    let mut config = minimal_config();
    config.companies = vec![
        CompanyConfig {
            code: "1000".to_string(),
            name: "Parent Company".to_string(),
            currency: "USD".to_string(),
            country: "US".to_string(),
            annual_transaction_volume: TransactionVolume::TenK,
            volume_weight: 0.6,
            fiscal_year_variant: "K4".to_string(),
        },
        CompanyConfig {
            code: "2000".to_string(),
            name: "Subsidiary EU".to_string(),
            currency: "EUR".to_string(),
            country: "DE".to_string(),
            annual_transaction_volume: TransactionVolume::TenK,
            volume_weight: 0.3,
            fiscal_year_variant: "K4".to_string(),
        },
        CompanyConfig {
            code: "3000".to_string(),
            name: "Subsidiary Asia".to_string(),
            currency: "JPY".to_string(),
            country: "JP".to_string(),
            annual_transaction_volume: TransactionVolume::TenK,
            volume_weight: 0.1,
            fiscal_year_variant: "K4".to_string(),
        },
    ];
    config.global.period_months = 12;
    config
}

/// Create a balanced test journal entry.
pub fn balanced_journal_entry(amount: Decimal) -> JournalEntry {
    let doc_id = Uuid::new_v4();
    let posting_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

    JournalEntry {
        header: JournalEntryHeader {
            document_id: doc_id,
            company_code: "TEST".to_string(),
            fiscal_year: 2024,
            fiscal_period: 1,
            posting_date,
            document_date: posting_date,
            created_at: Utc::now(),
            document_type: "SA".to_string(),
            currency: "USD".to_string(),
            exchange_rate: Decimal::ONE,
            reference: None,
            header_text: Some("Test entry".to_string()),
            created_by: "TESTUSER".to_string(),
            user_persona: "test_user".to_string(),
            source: TransactionSource::Manual,
            business_process: Some(BusinessProcess::R2R),
            ledger: "0L".to_string(),
            is_fraud: false,
            fraud_type: None,
            batch_id: None,
            control_ids: vec![],
            sox_relevant: false,
            control_status: Default::default(),
            sod_violation: false,
            sod_conflict_type: None,
            approval_workflow: None,
            ocpm_event_ids: vec![],
            ocpm_object_ids: vec![],
            ocpm_case_id: None,
            is_anomaly: false,
            anomaly_id: None,
            anomaly_type: None,
        },
        lines: vec![
            JournalEntryLine::debit(doc_id, 1, "100000".to_string(), amount),
            JournalEntryLine::credit(doc_id, 2, "200000".to_string(), amount),
        ],
    }
}

/// Create an unbalanced journal entry (for testing error cases).
pub fn unbalanced_journal_entry() -> JournalEntry {
    let mut entry = balanced_journal_entry(Decimal::new(1000, 2));
    // Make it unbalanced by changing the credit amount
    entry.lines[1].credit_amount = Decimal::new(500, 2);
    entry.lines[1].local_amount = Decimal::new(-500, 2);
    entry
}

/// Create a test GL account.
pub fn test_gl_account(
    number: &str,
    account_type: AccountType,
    sub_type: AccountSubType,
) -> GLAccount {
    GLAccount::new(
        number.to_string(),
        format!("Test Account {}", number),
        account_type,
        sub_type,
    )
}

/// Create test GL accounts for common account types.
pub fn standard_test_accounts() -> Vec<GLAccount> {
    vec![
        test_gl_account("100000", AccountType::Asset, AccountSubType::Cash),
        test_gl_account(
            "110000",
            AccountType::Asset,
            AccountSubType::AccountsReceivable,
        ),
        test_gl_account("120000", AccountType::Asset, AccountSubType::Inventory),
        test_gl_account("150000", AccountType::Asset, AccountSubType::FixedAssets),
        test_gl_account(
            "200000",
            AccountType::Liability,
            AccountSubType::AccountsPayable,
        ),
        test_gl_account(
            "210000",
            AccountType::Liability,
            AccountSubType::AccruedLiabilities,
        ),
        test_gl_account(
            "300000",
            AccountType::Equity,
            AccountSubType::RetainedEarnings,
        ),
        test_gl_account(
            "400000",
            AccountType::Revenue,
            AccountSubType::ProductRevenue,
        ),
        test_gl_account(
            "500000",
            AccountType::Expense,
            AccountSubType::CostOfGoodsSold,
        ),
        test_gl_account(
            "600000",
            AccountType::Expense,
            AccountSubType::OperatingExpenses,
        ),
    ]
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_config_is_valid() {
        let config = minimal_config();
        assert_eq!(config.global.period_months, 1);
        assert_eq!(config.companies.len(), 1);
        assert_eq!(config.companies[0].code, "TEST");
    }

    #[test]
    fn test_fraud_enabled_config() {
        let config = fraud_enabled_config();
        assert!(config.fraud.enabled);
        assert!((config.fraud.fraud_rate - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_multi_company_config() {
        let config = multi_company_config();
        assert_eq!(config.companies.len(), 3);
        assert_eq!(config.global.period_months, 12);
    }

    #[test]
    fn test_balanced_entry_is_balanced() {
        let entry = balanced_journal_entry(Decimal::new(10000, 2));
        let total_debits: Decimal = entry.lines.iter().map(|l| l.debit_amount).sum();
        let total_credits: Decimal = entry.lines.iter().map(|l| l.credit_amount).sum();
        assert_eq!(total_debits, total_credits);
    }

    #[test]
    fn test_unbalanced_entry_is_unbalanced() {
        let entry = unbalanced_journal_entry();
        let total_debits: Decimal = entry.lines.iter().map(|l| l.debit_amount).sum();
        let total_credits: Decimal = entry.lines.iter().map(|l| l.credit_amount).sum();
        assert_ne!(total_debits, total_credits);
    }

    #[test]
    fn test_standard_accounts_cover_all_types() {
        let accounts = standard_test_accounts();
        assert!(accounts
            .iter()
            .any(|a| a.account_type == AccountType::Asset));
        assert!(accounts
            .iter()
            .any(|a| a.account_type == AccountType::Liability));
        assert!(accounts
            .iter()
            .any(|a| a.account_type == AccountType::Equity));
        assert!(accounts
            .iter()
            .any(|a| a.account_type == AccountType::Revenue));
        assert!(accounts
            .iter()
            .any(|a| a.account_type == AccountType::Expense));
    }
}
