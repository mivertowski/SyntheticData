//! Industry presets for quick configuration.

use crate::schema::*;
use datasynth_core::models::{CoAComplexity, IndustrySector};

/// Create a preset configuration for a specific industry.
pub fn create_preset(
    industry: IndustrySector,
    company_count: usize,
    period_months: u32,
    complexity: CoAComplexity,
    volume: TransactionVolume,
) -> GeneratorConfig {
    let companies = generate_companies(industry, company_count, volume);

    GeneratorConfig {
        global: GlobalConfig {
            seed: None,
            industry,
            start_date: "2024-01-01".to_string(),
            period_months,
            group_currency: "USD".to_string(),
            parallel: true,
            worker_threads: 0,
            memory_limit_mb: 0,
        },
        companies,
        chart_of_accounts: ChartOfAccountsConfig {
            complexity,
            industry_specific: true,
            custom_accounts: None,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 5,
        },
        transactions: TransactionConfig::default(),
        output: OutputConfig::default(),
        fraud: FraudConfig::default(),
        data_quality: DataQualitySchemaConfig::default(),
        internal_controls: InternalControlsConfig::default(),
        business_processes: get_business_process_config(industry),
        user_personas: UserPersonaConfig::default(),
        templates: TemplateConfig::default(),
        approval: ApprovalConfig::default(),
        departments: DepartmentConfig::default(),
        master_data: MasterDataConfig::default(),
        document_flows: DocumentFlowConfig::default(),
        intercompany: IntercompanyConfig::default(),
        balance: BalanceConfig::default(),
        ocpm: OcpmConfig::default(),
        audit: AuditGenerationConfig::default(),
        banking: datasynth_banking::BankingConfig::default(),
        scenario: ScenarioConfig::default(),
        temporal: TemporalDriftConfig::default(),
        graph_export: GraphExportConfig::default(),
        streaming: StreamingSchemaConfig::default(),
        rate_limit: RateLimitSchemaConfig::default(),
        temporal_attributes: TemporalAttributeSchemaConfig::default(),
        relationships: RelationshipSchemaConfig::default(),
        accounting_standards: AccountingStandardsConfig::default(),
        audit_standards: AuditStandardsConfig::default(),
        distributions: AdvancedDistributionConfig::default(),
        temporal_patterns: TemporalPatternsConfig::default(),
        vendor_network: VendorNetworkSchemaConfig::default(),
        customer_segmentation: CustomerSegmentationSchemaConfig::default(),
        relationship_strength: RelationshipStrengthSchemaConfig::default(),
        cross_process_links: CrossProcessLinksSchemaConfig::default(),
        organizational_events: OrganizationalEventsSchemaConfig::default(),
        behavioral_drift: BehavioralDriftSchemaConfig::default(),
        market_drift: MarketDriftSchemaConfig::default(),
        drift_labeling: DriftLabelingSchemaConfig::default(),
    }
}

/// Generate company configurations based on industry.
fn generate_companies(
    industry: IndustrySector,
    count: usize,
    volume: TransactionVolume,
) -> Vec<CompanyConfig> {
    let regions = match industry {
        IndustrySector::Manufacturing => vec![
            ("1000", "US Manufacturing", "USD", "US"),
            ("2000", "EU Manufacturing", "EUR", "DE"),
            ("3000", "APAC Manufacturing", "CNY", "CN"),
        ],
        IndustrySector::Retail => vec![
            ("1000", "US Retail", "USD", "US"),
            ("2000", "UK Retail", "GBP", "GB"),
            ("3000", "EU Retail", "EUR", "FR"),
        ],
        IndustrySector::FinancialServices => vec![
            ("1000", "US Banking", "USD", "US"),
            ("2000", "Swiss Banking", "CHF", "CH"),
            ("3000", "UK Banking", "GBP", "GB"),
        ],
        IndustrySector::Healthcare => vec![
            ("1000", "US Healthcare", "USD", "US"),
            ("2000", "EU Healthcare", "EUR", "DE"),
        ],
        IndustrySector::Technology => vec![
            ("1000", "US Tech", "USD", "US"),
            ("2000", "EU Tech", "EUR", "IE"),
            ("3000", "APAC Tech", "JPY", "JP"),
        ],
        _ => vec![
            ("1000", "HQ", "USD", "US"),
            ("2000", "Subsidiary", "EUR", "DE"),
        ],
    };

    regions
        .iter()
        .take(count)
        .enumerate()
        .map(|(i, (code, name, currency, country))| CompanyConfig {
            code: code.to_string(),
            name: name.to_string(),
            currency: currency.to_string(),
            country: country.to_string(),
            fiscal_year_variant: "K4".to_string(),
            annual_transaction_volume: volume,
            volume_weight: if i == 0 { 1.0 } else { 0.5 },
        })
        .collect()
}

/// Get industry-specific business process weights.
fn get_business_process_config(industry: IndustrySector) -> BusinessProcessConfig {
    match industry {
        IndustrySector::Manufacturing => BusinessProcessConfig {
            o2c_weight: 0.25,
            p2p_weight: 0.40, // Heavy procurement
            r2r_weight: 0.15,
            h2r_weight: 0.10,
            a2r_weight: 0.10, // More assets
        },
        IndustrySector::Retail => BusinessProcessConfig {
            o2c_weight: 0.50, // Heavy sales
            p2p_weight: 0.30,
            r2r_weight: 0.10,
            h2r_weight: 0.07,
            a2r_weight: 0.03,
        },
        IndustrySector::FinancialServices => BusinessProcessConfig {
            o2c_weight: 0.30,
            p2p_weight: 0.15,
            r2r_weight: 0.40, // Heavy reporting
            h2r_weight: 0.10,
            a2r_weight: 0.05,
        },
        IndustrySector::Healthcare => BusinessProcessConfig {
            o2c_weight: 0.35,
            p2p_weight: 0.30,
            r2r_weight: 0.15,
            h2r_weight: 0.15, // Labor intensive
            a2r_weight: 0.05,
        },
        IndustrySector::Technology => BusinessProcessConfig {
            o2c_weight: 0.40,
            p2p_weight: 0.20,
            r2r_weight: 0.20,
            h2r_weight: 0.15, // Knowledge workers
            a2r_weight: 0.05,
        },
        _ => BusinessProcessConfig::default(),
    }
}

/// Quick preset for demo/testing purposes.
pub fn demo_preset() -> GeneratorConfig {
    create_preset(
        IndustrySector::Manufacturing,
        1,
        3,
        CoAComplexity::Small,
        TransactionVolume::TenK,
    )
}

/// Preset for stress testing with high volume.
pub fn stress_test_preset() -> GeneratorConfig {
    create_preset(
        IndustrySector::Manufacturing,
        3,
        12,
        CoAComplexity::Large,
        TransactionVolume::TenM,
    )
}
