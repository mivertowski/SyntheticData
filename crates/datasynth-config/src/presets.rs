//! Industry presets for quick configuration.
//!
//! Each preset creates companies with ISO 3166-1 alpha-2 country codes and
//! matching ISO 4217 currencies. The country pack system provides locale-aware
//! data (holidays, names, tax rules, address formats, etc.) for each company's
//! country. Built-in packs exist for US, DE, and GB; all other country codes
//! fall back to the `_default` pack automatically via `CountryPackRegistry`.
//!
//! The preset tax jurisdictions are kept in sync with preset company countries
//! so that generated tax data is consistent with company locations.

use crate::schema::*;
use datasynth_core::models::{CoAComplexity, IndustrySector};

/// Create a preset configuration for a specific industry.
///
/// Companies are assigned country codes that match the industry's typical
/// geographic footprint. The country pack system will automatically load
/// the appropriate pack for each company's country (US, DE, GB have dedicated
/// packs; others fall back to `_default`).
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
            presentation_currency: None,
            parallel: true,
            worker_threads: 0,
            memory_limit_mb: 0,
            fiscal_year_months: None,
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
        anomaly_injection: EnhancedAnomalyConfig::default(),
        industry_specific: IndustrySpecificConfig::default(),
        fingerprint_privacy: FingerprintPrivacyConfig::default(),
        quality_gates: QualityGatesSchemaConfig::default(),
        compliance: ComplianceSchemaConfig::default(),
        webhooks: WebhookSchemaConfig::default(),
        llm: LlmSchemaConfig::default(),
        diffusion: DiffusionSchemaConfig::default(),
        causal: CausalSchemaConfig::default(),
        source_to_pay: SourceToPayConfig::default(),
        financial_reporting: FinancialReportingConfig::default(),
        hr: HrConfig::default(),
        manufacturing: get_manufacturing_config(industry),
        sales_quotes: SalesQuoteConfig::default(),
        tax: get_tax_config(industry),
        treasury: get_treasury_config(industry),
        project_accounting: get_project_accounting_config(industry),
        esg: get_esg_config(industry),
        // Country packs use built-in defaults (US, DE, GB + _default fallback).
        // Set to None so the runtime loads only embedded packs without
        // requiring an external directory. Companies with countries that lack
        // a dedicated pack (e.g. CN, FR, CH, IE, JP) gracefully fall back to
        // the _default pack which provides sensible baseline data.
        country_packs: None,
        scenarios: ScenariosConfig::default(),
        session: SessionSchemaConfig::default(),
        compliance_regulations: ComplianceRegulationsConfig::default(),
    }
}

/// Generate company configurations based on industry.
///
/// Each industry preset defines a realistic geographic footprint with companies
/// in different regions. Country codes are ISO 3166-1 alpha-2 and currencies are
/// ISO 4217, paired appropriately (e.g. "DE" with "EUR", "JP" with "JPY").
///
/// Country pack coverage:
/// - **Built-in packs** (US, DE, GB): Full locale data including holidays,
///   names, tax rules, addresses, banking formats, payroll rules.
/// - **Fallback countries** (CN, FR, CH, IE, JP): Use the `_default` pack
///   which provides English-language baseline data. Users can supply external
///   country pack JSON files for full locale support.
fn generate_companies(
    industry: IndustrySector,
    count: usize,
    volume: TransactionVolume,
) -> Vec<CompanyConfig> {
    // Each tuple: (company_code, name, currency, country_code)
    // Countries with built-in packs: US, DE, GB
    // Countries using _default fallback: CN, FR, CH, IE, JP
    let regions = match industry {
        IndustrySector::Manufacturing => vec![
            ("1000", "US Manufacturing", "USD", "US"), // built-in pack
            ("2000", "EU Manufacturing", "EUR", "DE"), // built-in pack
            ("3000", "APAC Manufacturing", "CNY", "CN"), // _default fallback
        ],
        IndustrySector::Retail => vec![
            ("1000", "US Retail", "USD", "US"), // built-in pack
            ("2000", "UK Retail", "GBP", "GB"), // built-in pack
            ("3000", "EU Retail", "EUR", "FR"), // _default fallback
        ],
        IndustrySector::FinancialServices => vec![
            ("1000", "US Banking", "USD", "US"),    // built-in pack
            ("2000", "Swiss Banking", "CHF", "CH"), // _default fallback
            ("3000", "UK Banking", "GBP", "GB"),    // built-in pack
        ],
        IndustrySector::Healthcare => vec![
            ("1000", "US Healthcare", "USD", "US"), // built-in pack
            ("2000", "EU Healthcare", "EUR", "DE"), // built-in pack
        ],
        IndustrySector::Technology => vec![
            ("1000", "US Tech", "USD", "US"),   // built-in pack
            ("2000", "EU Tech", "EUR", "IE"),   // _default fallback
            ("3000", "APAC Tech", "JPY", "JP"), // _default fallback
        ],
        _ => vec![
            ("1000", "HQ", "USD", "US"),         // built-in pack
            ("2000", "Subsidiary", "EUR", "DE"), // built-in pack
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
            functional_currency: None,
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

/// Enhance any industry preset with full audit simulation capabilities.
///
/// Apply this overlay on top of any `create_preset(...)` result to enable all
/// audit-relevant v1.3.0 features:
/// - Audit standards (ISA / PCAOB / SOX)
/// - Internal controls with COSO 2013
/// - Accounting standards (revenue recognition, leases, fair value, impairment)
/// - Anomaly injection at a 2% rate (fraud + errors)
/// - Network features (vendor network, customer segmentation, cross-process links)
/// - Intercompany transactions
/// - Balance and trial balance generation
///
/// # Example
/// ```rust,ignore
/// let config = audit_group_overlay(create_preset(
///     IndustrySector::Manufacturing, 3, 12,
///     CoAComplexity::Large, TransactionVolume::HundredK,
/// ));
/// ```
pub fn audit_group_overlay(mut config: GeneratorConfig) -> GeneratorConfig {
    // ── Audit standards (ISA + PCAOB + SOX) ──────────────────────────────────
    config.audit_standards.enabled = true;
    config.audit_standards.isa_compliance.enabled = true;
    config.audit_standards.isa_compliance.framework = "dual".to_string(); // ISA + PCAOB
    config.audit_standards.generate_audit_trail = true;
    config.audit_standards.sox.enabled = true;
    config.audit_standards.pcaob.enabled = true;

    // ── Accounting standards (US GAAP by default) ─────────────────────────────
    config.accounting_standards.enabled = true;
    config.accounting_standards.revenue_recognition.enabled = true;
    config.accounting_standards.leases.enabled = true;
    config.accounting_standards.fair_value.enabled = true;
    config.accounting_standards.impairment.enabled = true;

    // ── Internal controls with COSO 2013 ──────────────────────────────────────
    config.internal_controls.enabled = true;
    config.internal_controls.coso_enabled = true;
    config.internal_controls.include_entity_level_controls = true;
    config.internal_controls.target_maturity_level = "managed".to_string();
    config.internal_controls.exception_rate = 0.02;
    config.internal_controls.sod_violation_rate = 0.01;

    // ── Anomaly injection at audit-relevant rate ───────────────────────────────
    config.anomaly_injection.enabled = true;
    config.anomaly_injection.rates.total_rate = 0.02;
    config.anomaly_injection.rates.fraud_rate = 0.01;
    config.anomaly_injection.rates.error_rate = 0.005;
    config.anomaly_injection.rates.process_rate = 0.005;
    config.anomaly_injection.difficulty_classification.enabled = true;
    config.anomaly_injection.context_aware.enabled = true;

    // ── Network / interconnectivity features ──────────────────────────────────
    config.vendor_network.enabled = true;
    config.customer_segmentation.enabled = true;
    config.cross_process_links.enabled = true;
    config.cross_process_links.payment_bank_reconciliation = true;
    config.cross_process_links.intercompany_bilateral = true;
    config.relationship_strength.enabled = true;

    // ── Intercompany transactions ─────────────────────────────────────────────
    config.intercompany.enabled = true;
    config.intercompany.generate_matched_pairs = true;
    config.intercompany.generate_eliminations = true;

    // ── Balance and trial balance generation ──────────────────────────────────
    config.balance.generate_opening_balances = true;
    config.balance.generate_trial_balances = true;
    config.balance.reconcile_subledgers = true;

    // ── Scenario tags for traceability ────────────────────────────────────────
    config.scenario.tags.push("audit_group".to_string());
    config.scenario.tags.push("audit_simulation".to_string());

    config
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

/// AssureTwin comprehensive preset for full-featured demos.
///
/// Generates data for all 5 process families (P2P, O2C, R2R, H2R, ATR) with:
/// - 2% fraud injection rate with specific patterns
/// - OCEL 2.0 event logs for process mining
/// - Internal controls with COSO 2013 framework
/// - US GAAP and SOX compliance
/// - Multi-stage fraud schemes and near-miss anomalies
/// - Vendor network and customer segmentation
/// - Graph export for ML training
///
/// Use this preset for AssureTwin platform demonstrations showcasing
/// fraud detection, process mining, and audit analytics capabilities.
///
/// Companies use Manufacturing preset countries (US, DE, CN). The US and DE
/// companies benefit from built-in country packs; the APAC (CN) company uses
/// the `_default` fallback pack.
pub fn assuretwin_comprehensive_preset() -> GeneratorConfig {
    let mut config = create_preset(
        IndustrySector::Manufacturing,
        3,  // 3 companies for intercompany transactions
        12, // 12 months for seasonal patterns
        CoAComplexity::Large,
        TransactionVolume::HundredK,
    );

    // Update company names for AssureTwin branding
    if config.companies.len() >= 3 {
        config.companies[0].name = "AssureTwin Corp HQ".to_string();
        config.companies[0].volume_weight = 0.5;
        config.companies[1].name = "AssureTwin Corp EU".to_string();
        config.companies[1].volume_weight = 0.3;
        config.companies[2].name = "AssureTwin Corp APAC".to_string();
        config.companies[2].volume_weight = 0.2;
    }

    // Set deterministic seed
    config.global.seed = Some(20240101);
    config.global.worker_threads = 8;
    config.global.memory_limit_mb = 8192;

    // Expand chart of accounts hierarchy
    config.chart_of_accounts.min_hierarchy_depth = 3;
    config.chart_of_accounts.max_hierarchy_depth = 6;

    // Enable fraud at 2% rate with specific patterns
    config.fraud.enabled = true;
    config.fraud.fraud_rate = 0.02;
    config.fraud.clustering_enabled = true;
    config.fraud.clustering_factor = 0.3;
    config.fraud.approval_thresholds = vec![1000.0, 5000.0, 25000.0, 100000.0];

    // Enable OCEL 2.0 process mining
    config.ocpm.enabled = true;
    config.ocpm.generate_lifecycle_events = true;
    config.ocpm.include_object_relationships = true;
    config.ocpm.compute_variants = true;

    // Enable internal controls with COSO
    config.internal_controls.enabled = true;
    config.internal_controls.coso_enabled = true;
    config.internal_controls.include_entity_level_controls = true;
    config.internal_controls.target_maturity_level = "managed".to_string();
    config.internal_controls.exception_rate = 0.02;
    config.internal_controls.sod_violation_rate = 0.01;

    // Enable accounting standards (US GAAP)
    config.accounting_standards.enabled = true;
    config.accounting_standards.revenue_recognition.enabled = true;
    config.accounting_standards.leases.enabled = true;
    config.accounting_standards.fair_value.enabled = true;
    config.accounting_standards.impairment.enabled = true;

    // Enable audit standards with SOX
    config.audit_standards.enabled = true;
    config.audit_standards.isa_compliance.enabled = true;
    config.audit_standards.isa_compliance.framework = "dual".to_string(); // ISA + PCAOB
    config.audit_standards.generate_audit_trail = true;
    config.audit_standards.sox.enabled = true;
    config.audit_standards.pcaob.enabled = true;

    // Enable enhanced anomaly injection with multi-stage schemes
    config.anomaly_injection.enabled = true;
    config.anomaly_injection.rates.total_rate = 0.02;
    config.anomaly_injection.rates.fraud_rate = 0.01;
    config.anomaly_injection.rates.error_rate = 0.005;
    config.anomaly_injection.rates.process_rate = 0.005;
    config.anomaly_injection.multi_stage_schemes.enabled = true;
    config.anomaly_injection.correlated_injection.enabled = true;
    config.anomaly_injection.near_miss.enabled = true;
    config.anomaly_injection.near_miss.proportion = 0.30;
    config.anomaly_injection.difficulty_classification.enabled = true;
    config.anomaly_injection.context_aware.enabled = true;

    // Enable network features
    config.vendor_network.enabled = true;
    config.vendor_network.depth = 3;
    config.customer_segmentation.enabled = true;
    config.cross_process_links.enabled = true;
    config.cross_process_links.inventory_p2p_o2c = true;
    config.cross_process_links.payment_bank_reconciliation = true;
    config.cross_process_links.intercompany_bilateral = true;

    // Enable relationship strength calculation
    config.relationship_strength.enabled = true;

    // Enable temporal drift for realistic patterns
    config.temporal.enabled = true;
    config.temporal.amount_mean_drift = 0.02;
    config.temporal.amount_variance_drift = 0.01;
    config.temporal.concept_drift_rate = 0.05;

    // Enable graph export for ML training
    config.graph_export.enabled = true;
    config.graph_export.train_ratio = 0.70;
    config.graph_export.validation_ratio = 0.15;

    // Enable intercompany transactions
    config.intercompany.enabled = true;
    config.intercompany.ic_transaction_rate = 0.15;
    config.intercompany.generate_matched_pairs = true;
    config.intercompany.generate_eliminations = true;

    // Enable balance and trial balance generation
    config.balance.generate_opening_balances = true;
    config.balance.generate_trial_balances = true;
    config.balance.reconcile_subledgers = true;

    // Set scenario metadata
    config.scenario.tags = vec![
        "assuretwin".to_string(),
        "comprehensive".to_string(),
        "fraud_detection".to_string(),
        "process_mining".to_string(),
        "audit".to_string(),
        "ocel_2_0".to_string(),
        "ml_training".to_string(),
    ];
    config.scenario.profile = Some("clean".to_string());
    config.scenario.description = Some(
        "AssureTwin comprehensive demo with all features enabled - fraud detection, process mining, audit analytics".to_string(),
    );
    config.scenario.ml_training = true;
    config.scenario.target_anomaly_ratio = Some(0.02);

    // Configure output
    config.output.output_directory = std::path::PathBuf::from("./output/assuretwin_comprehensive");
    config.output.include_acdoca = true;
    config.output.partition_by_period = true;
    config.output.partition_by_company = true;

    config
}

/// Get industry-specific tax configuration.
///
/// Tax jurisdictions are kept in sync with the company countries defined in
/// `generate_companies` for each industry sector.
fn get_tax_config(industry: IndustrySector) -> TaxConfig {
    match industry {
        IndustrySector::Manufacturing => TaxConfig {
            enabled: true,
            jurisdictions: TaxJurisdictionConfig {
                countries: vec!["US".into(), "DE".into(), "CN".into()],
                include_subnational: true,
            },
            vat_gst: VatGstConfig {
                enabled: true,
                ..Default::default()
            },
            withholding: WithholdingTaxSchemaConfig {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
        IndustrySector::Retail => TaxConfig {
            enabled: true,
            jurisdictions: TaxJurisdictionConfig {
                countries: vec!["US".into(), "GB".into(), "FR".into()],
                include_subnational: true,
            },
            sales_tax: SalesTaxConfig {
                enabled: true,
                ..Default::default()
            },
            vat_gst: VatGstConfig {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
        IndustrySector::FinancialServices => TaxConfig {
            enabled: true,
            jurisdictions: TaxJurisdictionConfig {
                countries: vec!["US".into(), "CH".into(), "GB".into()],
                include_subnational: false,
            },
            provisions: TaxProvisionSchemaConfig {
                enabled: true,
                ..Default::default()
            },
            withholding: WithholdingTaxSchemaConfig {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
        IndustrySector::Healthcare => TaxConfig {
            enabled: true,
            jurisdictions: TaxJurisdictionConfig {
                countries: vec!["US".into(), "DE".into()],
                include_subnational: true,
            },
            sales_tax: SalesTaxConfig {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
        IndustrySector::Technology => TaxConfig {
            enabled: true,
            jurisdictions: TaxJurisdictionConfig {
                countries: vec!["US".into(), "IE".into(), "JP".into()],
                include_subnational: false,
            },
            provisions: TaxProvisionSchemaConfig {
                enabled: true,
                ..Default::default()
            },
            withholding: WithholdingTaxSchemaConfig {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
        _ => TaxConfig::default(),
    }
}

/// Get industry-specific treasury configuration.
fn get_treasury_config(industry: IndustrySector) -> TreasuryConfig {
    match industry {
        IndustrySector::Manufacturing => TreasuryConfig {
            enabled: true,
            cash_positioning: CashPositioningConfig {
                enabled: true,
                ..Default::default()
            },
            cash_forecasting: CashForecastingConfig {
                enabled: true,
                ..Default::default()
            },
            hedging: HedgingSchemaConfig {
                enabled: true,
                hedge_ratio: 0.70,
                ..Default::default()
            },
            debt: DebtSchemaConfig {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
        IndustrySector::Retail => TreasuryConfig {
            enabled: true,
            cash_positioning: CashPositioningConfig {
                enabled: true,
                ..Default::default()
            },
            cash_forecasting: CashForecastingConfig {
                enabled: true,
                ..Default::default()
            },
            cash_pooling: CashPoolingConfig {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
        IndustrySector::FinancialServices => TreasuryConfig {
            enabled: true,
            cash_positioning: CashPositioningConfig {
                enabled: true,
                ..Default::default()
            },
            cash_forecasting: CashForecastingConfig {
                enabled: true,
                ..Default::default()
            },
            hedging: HedgingSchemaConfig {
                enabled: true,
                hedge_ratio: 0.90,
                ..Default::default()
            },
            debt: DebtSchemaConfig {
                enabled: true,
                ..Default::default()
            },
            netting: NettingSchemaConfig {
                enabled: true,
                ..Default::default()
            },
            bank_guarantees: BankGuaranteeSchemaConfig {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
        IndustrySector::Healthcare => TreasuryConfig {
            enabled: true,
            cash_positioning: CashPositioningConfig {
                enabled: true,
                ..Default::default()
            },
            cash_forecasting: CashForecastingConfig {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
        IndustrySector::Technology => TreasuryConfig {
            enabled: true,
            cash_positioning: CashPositioningConfig {
                enabled: true,
                ..Default::default()
            },
            cash_forecasting: CashForecastingConfig {
                enabled: true,
                ..Default::default()
            },
            hedging: HedgingSchemaConfig {
                enabled: true,
                hedge_ratio: 0.60,
                ..Default::default()
            },
            ..Default::default()
        },
        _ => TreasuryConfig::default(),
    }
}

/// Get industry-specific project accounting configuration.
fn get_project_accounting_config(industry: IndustrySector) -> ProjectAccountingConfig {
    match industry {
        IndustrySector::Manufacturing => ProjectAccountingConfig {
            enabled: true,
            project_count: 15,
            project_types: ProjectTypeDistribution {
                capital: 0.35,
                internal: 0.15,
                customer: 0.20,
                r_and_d: 0.10,
                maintenance: 0.15,
                technology: 0.05,
            },
            ..Default::default()
        },
        IndustrySector::Retail => ProjectAccountingConfig {
            enabled: true,
            project_count: 8,
            project_types: ProjectTypeDistribution {
                capital: 0.20,
                internal: 0.30,
                customer: 0.10,
                r_and_d: 0.05,
                maintenance: 0.10,
                technology: 0.25,
            },
            ..Default::default()
        },
        IndustrySector::FinancialServices => ProjectAccountingConfig {
            enabled: true,
            project_count: 12,
            project_types: ProjectTypeDistribution {
                capital: 0.10,
                internal: 0.25,
                customer: 0.15,
                r_and_d: 0.10,
                maintenance: 0.05,
                technology: 0.35,
            },
            ..Default::default()
        },
        IndustrySector::Healthcare => ProjectAccountingConfig {
            enabled: true,
            project_count: 10,
            project_types: ProjectTypeDistribution {
                capital: 0.30,
                internal: 0.10,
                customer: 0.10,
                r_and_d: 0.30,
                maintenance: 0.15,
                technology: 0.05,
            },
            ..Default::default()
        },
        IndustrySector::Technology => ProjectAccountingConfig {
            enabled: true,
            project_count: 20,
            project_types: ProjectTypeDistribution {
                capital: 0.05,
                internal: 0.15,
                customer: 0.30,
                r_and_d: 0.30,
                maintenance: 0.05,
                technology: 0.15,
            },
            ..Default::default()
        },
        _ => ProjectAccountingConfig::default(),
    }
}

/// Get industry-specific manufacturing configuration.
fn get_manufacturing_config(industry: IndustrySector) -> ManufacturingProcessConfig {
    match industry {
        IndustrySector::Manufacturing => ManufacturingProcessConfig {
            enabled: true,
            ..Default::default()
        },
        _ => ManufacturingProcessConfig::default(),
    }
}

/// Get industry-specific ESG configuration.
fn get_esg_config(industry: IndustrySector) -> EsgConfig {
    match industry {
        IndustrySector::Manufacturing => EsgConfig {
            enabled: true,
            environmental: EnvironmentalConfig {
                enabled: true,
                ..Default::default()
            },
            social: SocialConfig {
                ..Default::default()
            },
            supply_chain_esg: SupplyChainEsgConfig {
                enabled: true,
                assessment_coverage: 0.90,
                high_risk_countries: vec!["CN".into(), "BD".into(), "MM".into(), "VN".into()],
            },
            reporting: EsgReportingConfig {
                ..Default::default()
            },
            climate_scenarios: ClimateScenarioConfig {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
        IndustrySector::Retail => EsgConfig {
            enabled: true,
            environmental: EnvironmentalConfig {
                enabled: true,
                ..Default::default()
            },
            social: SocialConfig {
                ..Default::default()
            },
            supply_chain_esg: SupplyChainEsgConfig {
                enabled: true,
                assessment_coverage: 0.85,
                high_risk_countries: vec!["CN".into(), "BD".into(), "MM".into(), "KH".into()],
            },
            reporting: EsgReportingConfig {
                ..Default::default()
            },
            climate_scenarios: ClimateScenarioConfig {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
        IndustrySector::FinancialServices => EsgConfig {
            enabled: true,
            reporting: EsgReportingConfig {
                ..Default::default()
            },
            climate_scenarios: ClimateScenarioConfig {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
        IndustrySector::Healthcare => EsgConfig {
            enabled: true,
            social: SocialConfig {
                ..Default::default()
            },
            environmental: EnvironmentalConfig {
                enabled: true,
                ..Default::default()
            },
            reporting: EsgReportingConfig {
                ..Default::default()
            },
            ..Default::default()
        },
        IndustrySector::Technology => EsgConfig {
            enabled: true,
            environmental: EnvironmentalConfig {
                enabled: true,
                ..Default::default()
            },
            reporting: EsgReportingConfig {
                ..Default::default()
            },
            climate_scenarios: ClimateScenarioConfig {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
        _ => EsgConfig::default(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// All ISO 3166-1 alpha-2 codes used by any preset.
    const ALL_PRESET_COUNTRIES: &[&str] = &["US", "DE", "CN", "GB", "FR", "CH", "IE", "JP"];

    /// Countries with built-in country packs (no external files needed).
    const BUILTIN_PACK_COUNTRIES: &[&str] = &["US", "DE", "GB"];

    /// Helper: verify that every company has a valid 2-letter country code
    /// and that company countries are a subset of the tax jurisdictions.
    fn assert_country_tax_consistency(config: &GeneratorConfig) {
        let company_countries: Vec<&str> = config
            .companies
            .iter()
            .map(|c| c.country.as_str())
            .collect();
        let tax_countries = &config.tax.jurisdictions.countries;

        for cc in &company_countries {
            assert_eq!(cc.len(), 2, "country code '{cc}' must be 2 characters");
            assert!(
                cc.chars().all(|c| c.is_ascii_uppercase()),
                "country code '{cc}' must be uppercase ASCII"
            );
        }

        // Every company country should appear in the tax jurisdictions
        if config.tax.enabled {
            for cc in &company_countries {
                assert!(
                    tax_countries.iter().any(|tc| tc == cc),
                    "company country '{cc}' missing from tax jurisdictions {:?}",
                    tax_countries
                );
            }
        }
    }

    /// Helper: verify that country codes used in presets are recognized.
    fn assert_valid_preset_countries(config: &GeneratorConfig) {
        for company in &config.companies {
            assert!(
                ALL_PRESET_COUNTRIES.contains(&company.country.as_str()),
                "unexpected country '{}' in preset; update ALL_PRESET_COUNTRIES if intentional",
                company.country
            );
        }
    }

    #[test]
    fn test_demo_preset() {
        let config = demo_preset();
        assert_eq!(config.companies.len(), 1);
        assert_eq!(config.global.period_months, 3);
        assert_eq!(config.chart_of_accounts.complexity, CoAComplexity::Small);
        assert_valid_preset_countries(&config);
    }

    #[test]
    fn test_stress_test_preset() {
        let config = stress_test_preset();
        assert_eq!(config.companies.len(), 3);
        assert_eq!(config.global.period_months, 12);
        assert_eq!(config.chart_of_accounts.complexity, CoAComplexity::Large);
        assert_valid_preset_countries(&config);
    }

    #[test]
    fn test_all_industries_have_valid_countries() {
        let industries = [
            IndustrySector::Manufacturing,
            IndustrySector::Retail,
            IndustrySector::FinancialServices,
            IndustrySector::Healthcare,
            IndustrySector::Technology,
        ];
        for industry in industries {
            let config = create_preset(
                industry,
                10, // request more than available to get all
                12,
                CoAComplexity::Medium,
                TransactionVolume::HundredK,
            );
            assert_valid_preset_countries(&config);
            assert_country_tax_consistency(&config);
        }
    }

    #[test]
    fn test_preset_country_currency_pairs() {
        // Verify that country-currency pairs are correct across all presets
        let expected_pairs: &[(&str, &str)] = &[
            ("US", "USD"),
            ("DE", "EUR"),
            ("CN", "CNY"),
            ("GB", "GBP"),
            ("FR", "EUR"),
            ("CH", "CHF"),
            ("IE", "EUR"),
            ("JP", "JPY"),
        ];
        let industries = [
            IndustrySector::Manufacturing,
            IndustrySector::Retail,
            IndustrySector::FinancialServices,
            IndustrySector::Healthcare,
            IndustrySector::Technology,
        ];
        for industry in industries {
            let config = create_preset(
                industry,
                10,
                12,
                CoAComplexity::Medium,
                TransactionVolume::HundredK,
            );
            for company in &config.companies {
                let matching = expected_pairs.iter().find(|(cc, _)| *cc == company.country);
                assert!(
                    matching.is_some(),
                    "no expected currency pair for country '{}' in {:?}",
                    company.country,
                    industry
                );
                let (_, expected_currency) = matching.unwrap();
                assert_eq!(
                    company.currency, *expected_currency,
                    "country '{}' should use currency '{}' but got '{}' in {:?}",
                    company.country, expected_currency, company.currency, industry
                );
            }
        }
    }

    #[test]
    fn test_builtin_pack_coverage() {
        // Verify which preset companies have built-in country packs vs fallback
        let config = create_preset(
            IndustrySector::Manufacturing,
            3,
            12,
            CoAComplexity::Medium,
            TransactionVolume::HundredK,
        );
        let builtin: Vec<_> = config
            .companies
            .iter()
            .filter(|c| BUILTIN_PACK_COUNTRIES.contains(&c.country.as_str()))
            .collect();
        let fallback: Vec<_> = config
            .companies
            .iter()
            .filter(|c| !BUILTIN_PACK_COUNTRIES.contains(&c.country.as_str()))
            .collect();
        // Manufacturing: US (builtin), DE (builtin), CN (fallback)
        assert_eq!(builtin.len(), 2);
        assert_eq!(fallback.len(), 1);
        assert_eq!(fallback[0].country, "CN");
    }

    #[test]
    fn test_assuretwin_comprehensive_preset() {
        let config = assuretwin_comprehensive_preset();

        // Verify company structure
        assert_eq!(config.companies.len(), 3);
        assert_eq!(config.companies[0].name, "AssureTwin Corp HQ");
        assert_eq!(config.companies[1].name, "AssureTwin Corp EU");
        assert_eq!(config.companies[2].name, "AssureTwin Corp APAC");

        // Verify seed is set
        assert_eq!(config.global.seed, Some(20240101));

        // Verify fraud configuration
        assert!(config.fraud.enabled);
        assert!((config.fraud.fraud_rate - 0.02).abs() < 0.001);
        assert!(config.fraud.clustering_enabled);

        // Verify OCEL 2.0 is enabled
        assert!(config.ocpm.enabled);
        assert!(config.ocpm.generate_lifecycle_events);
        assert!(config.ocpm.include_object_relationships);
        assert!(config.ocpm.compute_variants);

        // Verify internal controls
        assert!(config.internal_controls.enabled);
        assert!(config.internal_controls.coso_enabled);
        assert!(config.internal_controls.include_entity_level_controls);
        assert_eq!(config.internal_controls.target_maturity_level, "managed");

        // Verify accounting standards
        assert!(config.accounting_standards.enabled);
        assert!(config.accounting_standards.revenue_recognition.enabled);
        assert!(config.accounting_standards.leases.enabled);
        assert!(config.accounting_standards.fair_value.enabled);

        // Verify audit standards
        assert!(config.audit_standards.enabled);
        assert!(config.audit_standards.sox.enabled);

        // Verify anomaly injection
        assert!(config.anomaly_injection.enabled);
        assert!(config.anomaly_injection.multi_stage_schemes.enabled);
        assert!(config.anomaly_injection.correlated_injection.enabled);
        assert!(config.anomaly_injection.near_miss.enabled);
        assert!((config.anomaly_injection.near_miss.proportion - 0.30).abs() < 0.001);

        // Verify network features
        assert!(config.vendor_network.enabled);
        assert_eq!(config.vendor_network.depth, 3);
        assert!(config.customer_segmentation.enabled);
        assert!(config.cross_process_links.enabled);

        // Verify temporal drift
        assert!(config.temporal.enabled);
        assert!((config.temporal.amount_mean_drift - 0.02).abs() < 0.001);

        // Verify graph export
        assert!(config.graph_export.enabled);
        assert!((config.graph_export.train_ratio - 0.70).abs() < 0.001);

        // Verify intercompany
        assert!(config.intercompany.enabled);
        assert!((config.intercompany.ic_transaction_rate - 0.15).abs() < 0.001);

        // Verify scenario tags
        assert!(config.scenario.tags.contains(&"assuretwin".to_string()));
        assert!(config
            .scenario
            .tags
            .contains(&"fraud_detection".to_string()));
        assert!(config.scenario.tags.contains(&"process_mining".to_string()));
        assert!(config.scenario.tags.contains(&"audit".to_string()));
        assert!(config.scenario.tags.contains(&"ocel_2_0".to_string()));
        assert!(config.scenario.tags.contains(&"ml_training".to_string()));

        // Verify ML training configuration
        assert!(config.scenario.ml_training);
        assert_eq!(config.scenario.target_anomaly_ratio, Some(0.02));

        // Verify company countries are set correctly
        assert_eq!(config.companies[0].country, "US");
        assert_eq!(config.companies[1].country, "DE");
        assert_eq!(config.companies[2].country, "CN");

        // Verify country-tax consistency
        assert_country_tax_consistency(&config);
    }

    #[test]
    fn test_assuretwin_preset_business_processes() {
        let config = assuretwin_comprehensive_preset();

        // Verify all 5 process families have weights
        let bp = &config.business_processes;
        assert!(bp.o2c_weight > 0.0);
        assert!(bp.p2p_weight > 0.0);
        assert!(bp.r2r_weight > 0.0);
        assert!(bp.h2r_weight > 0.0);
        assert!(bp.a2r_weight > 0.0);

        // Weights should sum to approximately 1.0
        let total = bp.o2c_weight + bp.p2p_weight + bp.r2r_weight + bp.h2r_weight + bp.a2r_weight;
        assert!((total - 1.0).abs() < 0.01);
    }
}
