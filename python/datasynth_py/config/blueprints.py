"""Blueprint registry for common DataSynth configurations."""

from __future__ import annotations

from typing import Any, Callable, Dict, List, Optional

from datasynth_py.config.models import (
    AdvancedDistributionSettings,
    AuditSettings,
    BankingSettings,
    BudgetSchemaConfig,
    BusinessDaySchemaConfig,
    CalendarSchemaConfig,
    ChartOfAccountsSettings,
    CompanyConfig,
    Config,
    CorrelationConfig,
    CorrelationFieldConfig,
    CrossProcessLinksConfig,
    CultureDistributionConfig,
    CustomerSegmentationConfig,
    DataQualitySettings,
    DescriptionTemplateConfig,
    EconomicCycleConfig,
    EntityTimezoneMappingConfig,
    FinancialReportingConfig,
    FraudSettings,
    GlobalSettings,
    GraphExportSettings,
    HrConfig,
    HypergraphSettings,
    IntraDaySchemaConfig,
    IntraDaySegmentConfig,
    ManagementKpisSchemaConfig,
    ManufacturingProcessConfig,
    MixtureComponentConfig,
    MixtureDistributionConfig,
    NameTemplateConfig,
    OcpmSettings,
    PayrollSchemaConfig,
    PeriodEndModelConfig,
    PeriodEndSchemaConfig,
    ProcessLayerSettings,
    ProcessingLagSchemaConfig,
    LagDistributionConfig,
    ProductionOrderSchemaConfig,
    ReferenceTemplateConfig,
    RegimeChangeConfig,
    SalesQuoteSchemaConfig,
    ScenarioSettings,
    SettlementRulesConfig,
    SourceToPayConfig,
    StatisticalTestConfig,
    StatisticalValidationConfig,
    TemplateSettings,
    TemporalPatternsConfig,
    TimezoneSchemaConfig,
    VendorNetworkConfig,
)

BlueprintFactory = Callable[..., Config]

FRAUD_PACKS = ["revenue_fraud", "payroll_ghost", "vendor_kickback", "management_override", "comprehensive"]


def retail_small(
    companies: int = 3,
    transactions: int = 5000,
    realistic_names: bool = True,
) -> Config:
    """Create a small retail configuration.

    Args:
        companies: Number of companies to generate.
        transactions: Transaction volume hint (maps to volume preset).
        realistic_names: Enable realistic name generation with cultural diversity.
    """
    volume = _transactions_to_volume(transactions)
    templates = None
    if realistic_names:
        templates = TemplateSettings(
            names=NameTemplateConfig(
                culture_distribution=CultureDistributionConfig(),
                generate_realistic_names=True,
            ),
            descriptions=DescriptionTemplateConfig(),
            references=ReferenceTemplateConfig(),
        )
    return Config(
        global_settings=GlobalSettings(
            industry="retail",
            start_date="2024-01-01",
            period_months=12,
        ),
        companies=[
            CompanyConfig(
                code=f"R{i + 1:03d}",
                name=f"Retail Company {i + 1}",
                currency="USD",
                country="US",
                annual_transaction_volume=volume,
            )
            for i in range(companies)
        ],
        chart_of_accounts=ChartOfAccountsSettings(complexity="small"),
        templates=templates,
    )


def banking_medium(companies: int = 5, transactions: int = 20000) -> Config:
    """Create a medium financial services configuration.

    Args:
        companies: Number of companies to generate.
        transactions: Transaction volume hint (maps to volume preset).
    """
    volume = _transactions_to_volume(transactions)
    return Config(
        global_settings=GlobalSettings(
            industry="financial_services",
            start_date="2024-01-01",
            period_months=12,
        ),
        companies=[
            CompanyConfig(
                code=f"F{i + 1:03d}",
                name=f"Financial Services Company {i + 1}",
                currency="USD",
                country="US",
                annual_transaction_volume=volume,
            )
            for i in range(companies)
        ],
        chart_of_accounts=ChartOfAccountsSettings(complexity="medium"),
        fraud=FraudSettings(enabled=True, rate=0.01),
    )


def manufacturing_large(companies: int = 10, transactions: int = 100000) -> Config:
    """Create a large manufacturing configuration.

    Args:
        companies: Number of companies to generate.
        transactions: Transaction volume hint (maps to volume preset).
    """
    volume = _transactions_to_volume(transactions)
    return Config(
        global_settings=GlobalSettings(
            industry="manufacturing",
            start_date="2024-01-01",
            period_months=12,
        ),
        companies=[
            CompanyConfig(
                code=f"M{i + 1:03d}",
                name=f"Manufacturing Company {i + 1}",
                currency="USD",
                country="US",
                annual_transaction_volume=volume,
            )
            for i in range(companies)
        ],
        chart_of_accounts=ChartOfAccountsSettings(complexity="large"),
        manufacturing=ManufacturingProcessConfig(enabled=True),
    )


def banking_aml(customers: int = 1000, typologies: bool = True) -> Config:
    """Create a banking KYC/AML focused configuration.

    Enables banking transaction generation with AML typology injection
    for training fraud detection and compliance models.

    Args:
        customers: Number of banking customers to generate.
        typologies: Whether to inject AML typologies (structuring, layering, etc.).
    """
    return Config(
        global_settings=GlobalSettings(
            industry="financial_services",
            start_date="2024-01-01",
            period_months=12,
        ),
        companies=[
            CompanyConfig(
                code="BANK001",
                name="DataSynth Bank",
                currency="USD",
                country="US",
                annual_transaction_volume="hundred_k",
            ),
        ],
        chart_of_accounts=ChartOfAccountsSettings(complexity="medium"),
        banking=BankingSettings(
            enabled=True,
            retail_customers=int(customers * 0.7),
            business_customers=int(customers * 0.25),
            trusts=int(customers * 0.05),
            typologies_enabled=typologies,
        ),
        scenario=ScenarioSettings(
            tags=["banking", "aml", "compliance"],
            ml_training=True,
        ),
    )


def ml_training(
    industry: str = "manufacturing",
    anomaly_ratio: float = 0.05,
    with_data_quality: bool = True,
    with_distributions: bool = True,
) -> Config:
    """Create a configuration optimized for ML training datasets.

    Generates balanced datasets with labeled anomalies and optional
    data quality issues for robust model training.

    Args:
        industry: Industry sector for the data.
        anomaly_ratio: Target ratio of anomalous transactions (0.0-1.0).
        with_data_quality: Whether to inject data quality variations.
        with_distributions: Enable advanced mixture distributions for realistic amounts.
    """
    distributions = None
    if with_distributions:
        # Multi-modal amount distribution for realistic training data
        distributions = AdvancedDistributionSettings(
            enabled=True,
            amounts=MixtureDistributionConfig(
                enabled=True,
                distribution_type="lognormal",
                components=[
                    MixtureComponentConfig(weight=0.60, mu=6.0, sigma=1.5, label="routine"),
                    MixtureComponentConfig(weight=0.30, mu=8.5, sigma=1.0, label="significant"),
                    MixtureComponentConfig(weight=0.10, mu=11.0, sigma=0.8, label="major"),
                ],
                benford_compliance=True,
            ),
            industry_profile=industry,
            validation=StatisticalValidationConfig(
                enabled=True,
                tests=[
                    StatisticalTestConfig(test_type="benford_first_digit", threshold_mad=0.015),
                    StatisticalTestConfig(test_type="distribution_fit", target_distribution="lognormal"),
                ],
            ),
        )
    return Config(
        global_settings=GlobalSettings(
            industry=industry,
            start_date="2024-01-01",
            period_months=12,
        ),
        companies=[
            CompanyConfig(
                code="ML001",
                name="ML Training Corp",
                currency="USD",
                country="US",
                annual_transaction_volume="hundred_k",
            ),
        ],
        chart_of_accounts=ChartOfAccountsSettings(complexity="medium"),
        fraud=FraudSettings(
            enabled=True,
            rate=anomaly_ratio,
        ),
        data_quality=DataQualitySettings(
            enabled=with_data_quality,
            missing_rate=0.03,
            typo_rate=0.02,
        ) if with_data_quality else None,
        distributions=distributions,
        scenario=ScenarioSettings(
            tags=["ml_training", "labeled_data"],
            ml_training=True,
            target_anomaly_ratio=anomaly_ratio,
        ),
        graph_export=GraphExportSettings(
            enabled=True,
            formats=["pytorch_geometric"],
        ),
        templates=TemplateSettings(
            names=NameTemplateConfig(generate_realistic_names=True),
        ),
    )


def with_graph_export(base_config: Config, formats: Optional[List[str]] = None) -> Config:
    """Add graph export to an existing configuration.

    Args:
        base_config: Base configuration to extend.
        formats: Export formats (pytorch_geometric, neo4j, dgl). Defaults to pytorch_geometric.

    Returns:
        New Config with graph export enabled.
    """
    if formats is None:
        formats = ["pytorch_geometric"]

    graph_settings = GraphExportSettings(
        enabled=True,
        formats=formats,
    )

    # Use override to merge in graph export
    return base_config.override(graph_export=graph_settings.__dict__)


def audit_engagement(
    engagements: int = 5,
    with_evidence: bool = True,
) -> Config:
    """Create a configuration for audit data generation.

    Generates audit engagements, workpapers, evidence, and findings
    following ISA standards.

    Args:
        engagements: Number of audit engagements to generate.
        with_evidence: Whether to generate evidence items.
    """
    return Config(
        global_settings=GlobalSettings(
            industry="financial_services",
            start_date="2024-01-01",
            period_months=12,
        ),
        companies=[
            CompanyConfig(
                code="AUDITEE001",
                name="Auditee Corporation",
                currency="USD",
                country="US",
                annual_transaction_volume="hundred_k",
            ),
        ],
        chart_of_accounts=ChartOfAccountsSettings(complexity="medium"),
        audit=AuditSettings(
            enabled=True,
            engagements=engagements,
            workpapers_per_engagement=20,
            evidence_per_workpaper=5 if with_evidence else 0,
            risks_per_engagement=15,
            findings_per_engagement=8,
        ),
        scenario=ScenarioSettings(
            tags=["audit", "isa"],
        ),
    )


def with_distributions(
    base_config: Config,
    industry_profile: Optional[str] = None,
    with_correlations: bool = False,
) -> Config:
    """Add advanced distribution settings to an existing configuration.

    Args:
        base_config: Base configuration to extend.
        industry_profile: Industry profile for distributions (retail, manufacturing, financial_services).
        with_correlations: Enable cross-field correlation modeling.

    Returns:
        New Config with advanced distributions enabled.
    """
    correlations = None
    if with_correlations:
        correlations = CorrelationConfig(
            enabled=True,
            copula_type="gaussian",
            fields=[
                CorrelationFieldConfig(name="amount", distribution_type="lognormal"),
                CorrelationFieldConfig(name="line_items", distribution_type="normal", min_value=1, max_value=20),
            ],
            matrix=[[1.0, 0.65], [0.65, 1.0]],
        )

    dist_settings = AdvancedDistributionSettings(
        enabled=True,
        amounts=MixtureDistributionConfig(
            enabled=True,
            distribution_type="lognormal",
            components=[
                MixtureComponentConfig(weight=0.60, mu=6.0, sigma=1.5, label="routine"),
                MixtureComponentConfig(weight=0.30, mu=8.5, sigma=1.0, label="significant"),
                MixtureComponentConfig(weight=0.10, mu=11.0, sigma=0.8, label="major"),
            ],
            benford_compliance=True,
        ),
        correlations=correlations,
        industry_profile=industry_profile,
    )

    return base_config.override(distributions=dist_settings.__dict__)


def with_regime_changes(
    base_config: Config,
    with_economic_cycle: bool = True,
) -> Config:
    """Add regime change configuration for temporal distribution shifts.

    Simulates realistic business events like acquisitions, restructuring,
    and economic cycles that affect transaction patterns over time.

    Args:
        base_config: Base configuration to extend.
        with_economic_cycle: Enable economic cycle modeling.

    Returns:
        New Config with regime changes enabled.
    """
    economic_cycle = None
    if with_economic_cycle:
        economic_cycle = EconomicCycleConfig(
            enabled=True,
            cycle_period_months=48,
            amplitude=0.15,
            recession_probability=0.1,
            recession_depth=0.25,
        )

    # Ensure distributions are enabled
    existing_dist = base_config.distributions
    if existing_dist is None:
        existing_dist = AdvancedDistributionSettings(enabled=True)

    regime_config = RegimeChangeConfig(
        enabled=True,
        economic_cycle=economic_cycle,
    )

    return base_config.override(
        distributions={
            **existing_dist.__dict__,
            "regime_changes": regime_config.__dict__,
        }
    )


def with_templates(
    base_config: Config,
    email_domain: str = "company.com",
    invoice_prefix: str = "INV",
    po_prefix: str = "PO",
) -> Config:
    """Add realistic template settings for names and references.

    Args:
        base_config: Base configuration to extend.
        email_domain: Email domain for generated users.
        invoice_prefix: Prefix for invoice reference numbers.
        po_prefix: Prefix for purchase order reference numbers.

    Returns:
        New Config with template settings enabled.
    """
    template_settings = TemplateSettings(
        names=NameTemplateConfig(
            culture_distribution=CultureDistributionConfig(),
            email_domain=email_domain,
            generate_realistic_names=True,
        ),
        descriptions=DescriptionTemplateConfig(
            generate_header_text=True,
            generate_line_text=True,
        ),
        references=ReferenceTemplateConfig(
            generate_references=True,
            invoice_prefix=invoice_prefix,
            po_prefix=po_prefix,
        ),
    )

    return base_config.override(templates=template_settings.__dict__)


def with_temporal_patterns(
    base_config: Config,
    regions: Optional[List[str]] = None,
    with_business_days: bool = True,
    with_period_end_curves: bool = True,
    with_processing_lags: bool = False,
    with_intraday_patterns: bool = False,
    with_timezones: bool = False,
    default_timezone: str = "America/New_York",
) -> Config:
    """Add temporal pattern configuration for realistic time-based behavior.

    Enables business day calculations, regional holiday calendars, period-end
    volume spikes, processing lag modeling, and timezone handling.

    Args:
        base_config: Base configuration to extend.
        regions: Holiday calendar regions (US, DE, GB, CN, JP, IN, BR, MX, AU, SG, KR).
        with_business_days: Enable business day calculations and settlement rules.
        with_period_end_curves: Enable exponential period-end volume curves.
        with_processing_lags: Enable event-to-posting lag modeling.
        with_intraday_patterns: Enable intra-day time segment patterns.
        with_timezones: Enable multi-region timezone handling.
        default_timezone: Default IANA timezone name.

    Returns:
        New Config with temporal patterns enabled.
    """
    if regions is None:
        regions = ["US"]

    business_days = None
    if with_business_days:
        business_days = BusinessDaySchemaConfig(
            enabled=True,
            half_day_policy="full_day",
            settlement_rules=SettlementRulesConfig(
                equity_days=2,
                government_bonds_days=1,
                fx_spot_days=2,
                wire_cutoff_time="14:00",
            ),
        )

    calendars = CalendarSchemaConfig(regions=regions)

    period_end = None
    if with_period_end_curves:
        period_end = PeriodEndSchemaConfig(
            enabled=True,
            model="exponential",
            month_end=PeriodEndModelConfig(
                start_day=-10,
                base_multiplier=1.0,
                peak_multiplier=3.5,
                decay_rate=0.3,
            ),
            quarter_end=PeriodEndModelConfig(
                start_day=-10,
                base_multiplier=1.0,
                peak_multiplier=5.0,
                decay_rate=0.25,
            ),
            year_end=PeriodEndModelConfig(
                start_day=-15,
                base_multiplier=1.0,
                peak_multiplier=6.0,
                decay_rate=0.2,
            ),
        )

    processing_lags = None
    if with_processing_lags:
        processing_lags = ProcessingLagSchemaConfig(
            enabled=True,
            sales_order_lag=LagDistributionConfig(mu=0.5, sigma=0.8),
            goods_receipt_lag=LagDistributionConfig(mu=1.5, sigma=0.5),
            invoice_receipt_lag=LagDistributionConfig(mu=2.0, sigma=0.6),
        )

    intraday = None
    if with_intraday_patterns:
        intraday = IntraDaySchemaConfig(
            enabled=True,
            segments=[
                IntraDaySegmentConfig(
                    name="morning_spike",
                    start="08:30",
                    end="10:00",
                    multiplier=1.8,
                    posting_type="both",
                ),
                IntraDaySegmentConfig(
                    name="lunch_dip",
                    start="12:00",
                    end="13:30",
                    multiplier=0.4,
                    posting_type="human",
                ),
                IntraDaySegmentConfig(
                    name="eod_rush",
                    start="16:00",
                    end="17:30",
                    multiplier=1.5,
                    posting_type="both",
                ),
            ],
        )

    timezones = None
    if with_timezones:
        timezones = TimezoneSchemaConfig(
            enabled=True,
            default_timezone=default_timezone,
            consolidation_timezone="UTC",
            entity_mappings=[
                EntityTimezoneMappingConfig(pattern="EU_*", timezone="Europe/London"),
                EntityTimezoneMappingConfig(pattern="APAC_*", timezone="Asia/Singapore"),
            ],
        )

    # Build the temporal patterns dict manually to ensure proper nesting
    tp_dict: Dict[str, Any] = {"enabled": True}

    if business_days is not None:
        bd_dict: Dict[str, Any] = {
            "enabled": business_days.enabled,
            "half_day_policy": business_days.half_day_policy,
        }
        if business_days.settlement_rules is not None:
            bd_dict["settlement_rules"] = {
                "equity_days": business_days.settlement_rules.equity_days,
                "government_bonds_days": business_days.settlement_rules.government_bonds_days,
                "fx_spot_days": business_days.settlement_rules.fx_spot_days,
                "wire_cutoff_time": business_days.settlement_rules.wire_cutoff_time,
            }
        tp_dict["business_days"] = bd_dict

    if calendars is not None:
        tp_dict["calendars"] = {"regions": calendars.regions}

    if period_end is not None:
        pe_dict: Dict[str, Any] = {
            "enabled": period_end.enabled,
            "model": period_end.model,
        }
        if period_end.month_end is not None:
            pe_dict["month_end"] = {
                "start_day": period_end.month_end.start_day,
                "base_multiplier": period_end.month_end.base_multiplier,
                "peak_multiplier": period_end.month_end.peak_multiplier,
                "decay_rate": period_end.month_end.decay_rate,
            }
        if period_end.quarter_end is not None:
            pe_dict["quarter_end"] = {
                "start_day": period_end.quarter_end.start_day,
                "base_multiplier": period_end.quarter_end.base_multiplier,
                "peak_multiplier": period_end.quarter_end.peak_multiplier,
                "decay_rate": period_end.quarter_end.decay_rate,
            }
        if period_end.year_end is not None:
            pe_dict["year_end"] = {
                "start_day": period_end.year_end.start_day,
                "base_multiplier": period_end.year_end.base_multiplier,
                "peak_multiplier": period_end.year_end.peak_multiplier,
                "decay_rate": period_end.year_end.decay_rate,
            }
        tp_dict["period_end"] = pe_dict

    if processing_lags is not None:
        pl_dict: Dict[str, Any] = {"enabled": processing_lags.enabled}
        if processing_lags.sales_order_lag is not None:
            pl_dict["sales_order_lag"] = {
                "mu": processing_lags.sales_order_lag.mu,
                "sigma": processing_lags.sales_order_lag.sigma,
            }
        if processing_lags.goods_receipt_lag is not None:
            pl_dict["goods_receipt_lag"] = {
                "mu": processing_lags.goods_receipt_lag.mu,
                "sigma": processing_lags.goods_receipt_lag.sigma,
            }
        if processing_lags.invoice_receipt_lag is not None:
            pl_dict["invoice_receipt_lag"] = {
                "mu": processing_lags.invoice_receipt_lag.mu,
                "sigma": processing_lags.invoice_receipt_lag.sigma,
            }
        tp_dict["processing_lags"] = pl_dict

    if intraday is not None:
        intra_dict: Dict[str, Any] = {"enabled": intraday.enabled}
        if intraday.segments is not None:
            intra_dict["segments"] = [
                {
                    "name": s.name,
                    "start": s.start,
                    "end": s.end,
                    "multiplier": s.multiplier,
                    "posting_type": s.posting_type,
                }
                for s in intraday.segments
            ]
        tp_dict["intraday"] = intra_dict

    if timezones is not None:
        tz_dict: Dict[str, Any] = {
            "enabled": timezones.enabled,
            "default_timezone": timezones.default_timezone,
            "consolidation_timezone": timezones.consolidation_timezone,
        }
        if timezones.entity_mappings is not None:
            tz_dict["entity_mappings"] = [
                {"pattern": m.pattern, "timezone": m.timezone}
                for m in timezones.entity_mappings
            ]
        tp_dict["timezones"] = tz_dict

    return base_config.override(temporal_patterns=tp_dict)


def statistical_validation(
    industry: str = "manufacturing",
    transactions: int = 100000,
) -> Config:
    """Create a configuration focused on statistical validation.

    Generates data with comprehensive statistical tests enabled to verify
    distribution compliance (Benford's Law, mixture distributions, correlations).

    Args:
        industry: Industry sector for the data.
        transactions: Transaction volume hint.

    Returns:
        Config with statistical validation enabled.
    """
    volume = _transactions_to_volume(transactions)
    return Config(
        global_settings=GlobalSettings(
            industry=industry,
            start_date="2024-01-01",
            period_months=12,
        ),
        companies=[
            CompanyConfig(
                code="STAT001",
                name="Statistical Validation Corp",
                currency="USD",
                country="US",
                annual_transaction_volume=volume,
            ),
        ],
        chart_of_accounts=ChartOfAccountsSettings(complexity="medium"),
        distributions=AdvancedDistributionSettings(
            enabled=True,
            amounts=MixtureDistributionConfig(
                enabled=True,
                distribution_type="lognormal",
                components=[
                    MixtureComponentConfig(weight=0.60, mu=6.0, sigma=1.5, label="routine"),
                    MixtureComponentConfig(weight=0.30, mu=8.5, sigma=1.0, label="significant"),
                    MixtureComponentConfig(weight=0.10, mu=11.0, sigma=0.8, label="major"),
                ],
                benford_compliance=True,
            ),
            correlations=CorrelationConfig(
                enabled=True,
                copula_type="gaussian",
                fields=[
                    CorrelationFieldConfig(name="amount", distribution_type="lognormal"),
                    CorrelationFieldConfig(name="line_items", distribution_type="normal", min_value=1, max_value=20),
                    CorrelationFieldConfig(name="approval_level", distribution_type="normal", min_value=1, max_value=5),
                ],
                matrix=[
                    [1.00, 0.65, 0.72],
                    [0.65, 1.00, 0.55],
                    [0.72, 0.55, 1.00],
                ],
            ),
            industry_profile=industry,
            validation=StatisticalValidationConfig(
                enabled=True,
                tests=[
                    StatisticalTestConfig(test_type="benford_first_digit", threshold_mad=0.015),
                    StatisticalTestConfig(test_type="distribution_fit", target_distribution="lognormal", significance=0.05),
                    StatisticalTestConfig(test_type="chi_squared", significance=0.05),
                    StatisticalTestConfig(test_type="anderson_darling", significance=0.05),
                    StatisticalTestConfig(test_type="correlation_check", significance=0.05),
                ],
                fail_on_violation=False,
            ),
        ),
        templates=TemplateSettings(
            names=NameTemplateConfig(generate_realistic_names=True),
        ),
        scenario=ScenarioSettings(
            tags=["statistical_validation", "quality_assurance"],
        ),
    )


def with_llm_enrichment(
    base_config: Optional[Config] = None,
    provider: str = "mock",
    model: str = "default",
) -> Config:
    """Add LLM enrichment to a base config.

    Enables LLM-augmented generation for realistic vendor names, transaction
    descriptions, anomaly explanations, and memo fields.

    Args:
        base_config: Base configuration to extend. Defaults to retail_small().
        provider: LLM provider name (mock, openai, anthropic, http).
        model: Model identifier for the LLM provider.

    Returns:
        New Config with LLM enrichment enabled.
    """
    config = base_config if base_config is not None else retail_small()
    llm_config: Dict[str, Any] = {
        "enabled": True,
        "provider": provider,
        "model": model,
        "cache_enabled": True,
        "enrichment": {
            "vendor_names": True,
            "transaction_descriptions": True,
            "anomaly_explanations": True,
            "memo_fields": True,
        },
    }
    return config.override(llm=llm_config)


def with_diffusion(
    base_config: Optional[Config] = None,
    n_steps: int = 1000,
    schedule: str = "cosine",
    hybrid_weight: float = 0.3,
) -> Config:
    """Add diffusion model enhancement to a base config.

    Enables diffusion-based generation for learned distribution capture,
    optionally combined with rule-based generators in hybrid mode.

    Args:
        base_config: Base configuration to extend. Defaults to retail_small().
        n_steps: Number of diffusion steps.
        schedule: Noise schedule type (cosine, linear, sigmoid).
        hybrid_weight: Weight of diffusion output in hybrid mode (0.0-1.0).

    Returns:
        New Config with diffusion model enabled.
    """
    config = base_config if base_config is not None else retail_small()
    diffusion_config: Dict[str, Any] = {
        "enabled": True,
        "backend": "statistical",
        "n_steps": n_steps,
        "noise_schedule": schedule,
        "hybrid_mode": {
            "enabled": True,
            "diffusion_weight": hybrid_weight,
        },
        "training": {
            "enabled": False,
            "epochs": 100,
            "batch_size": 256,
        },
    }
    return config.override(diffusion=diffusion_config)


def with_causal(
    base_config: Optional[Config] = None,
    template: str = "fraud_detection",
) -> Config:
    """Add causal generation overlay to a base config.

    Enables causal graph specification and interventional/counterfactual
    generation for what-if scenario modeling.

    Args:
        base_config: Base configuration to extend. Defaults to retail_small().
        template: Causal graph template (fraud_detection, revenue_impact, supply_chain).

    Returns:
        New Config with causal generation enabled.
    """
    config = base_config if base_config is not None else retail_small()
    causal_config: Dict[str, Any] = {
        "enabled": True,
        "template": template,
        "interventions": {
            "enabled": True,
        },
        "counterfactuals": {
            "enabled": True,
            "samples_per_record": 5,
        },
        "validation": {
            "enabled": True,
            "check_causal_structure": True,
        },
    }
    return config.override(causal=causal_config)


def with_sourcing(
    base_config: Config,
    projects_per_year: int = 10,
) -> Config:
    """Add source-to-pay procurement pipeline to an existing configuration.

    Enables sourcing projects, supplier qualification, RFx events,
    bid evaluation, procurement contracts, and catalog management.

    Args:
        base_config: Base configuration to extend.
        projects_per_year: Number of sourcing projects per year.

    Returns:
        New Config with source-to-pay enabled.
    """
    s2p = SourceToPayConfig(
        enabled=True,
        sourcing={"projects_per_year": projects_per_year},
    )
    return base_config.override(source_to_pay=s2p.__dict__)


def with_financial_reporting(
    base_config: Config,
    with_kpis: bool = True,
    with_budgets: bool = True,
) -> Config:
    """Add financial reporting to an existing configuration.

    Generates balance sheet, income statement, cash flow statement,
    and changes in equity from trial balance data.

    Args:
        base_config: Base configuration to extend.
        with_kpis: Enable management KPI generation.
        with_budgets: Enable budget variance analysis.

    Returns:
        New Config with financial reporting enabled.
    """
    fr_dict: Dict[str, Any] = {
        "enabled": True,
        "generate_balance_sheet": True,
        "generate_income_statement": True,
        "generate_cash_flow": True,
        "generate_changes_in_equity": True,
    }
    if with_kpis:
        fr_dict["management_kpis"] = {"enabled": True, "frequency": "monthly"}
    if with_budgets:
        fr_dict["budgets"] = {
            "enabled": True,
            "revenue_growth_rate": 0.05,
            "expense_inflation_rate": 0.03,
        }
    return base_config.override(financial_reporting=fr_dict)


def with_hr(
    base_config: Config,
    with_payroll: bool = True,
    with_time_tracking: bool = True,
    with_expenses: bool = True,
) -> Config:
    """Add HR/payroll generation to an existing configuration.

    Generates payroll runs, time entries, and expense reports
    for the employee pool.

    Args:
        base_config: Base configuration to extend.
        with_payroll: Enable payroll generation.
        with_time_tracking: Enable time and attendance tracking.
        with_expenses: Enable expense report generation.

    Returns:
        New Config with HR generation enabled.
    """
    hr_dict: Dict[str, Any] = {"enabled": True}
    if with_payroll:
        hr_dict["payroll"] = {"enabled": True, "pay_frequency": "monthly"}
    if with_time_tracking:
        hr_dict["time_attendance"] = {"enabled": True, "overtime_rate": 0.10}
    if with_expenses:
        hr_dict["expenses"] = {"enabled": True, "submission_rate": 0.30}
    return base_config.override(hr=hr_dict)


def with_manufacturing(
    base_config: Config,
    orders_per_month: int = 50,
) -> Config:
    """Add manufacturing process chain to an existing configuration.

    Generates production orders, routing operations, component issues,
    and production variances.

    Args:
        base_config: Base configuration to extend.
        orders_per_month: Number of production orders per month.

    Returns:
        New Config with manufacturing enabled.
    """
    mfg_dict: Dict[str, Any] = {
        "enabled": True,
        "production_orders": {"orders_per_month": orders_per_month},
    }
    return base_config.override(manufacturing=mfg_dict)


def with_sales_quotes(
    base_config: Config,
    quotes_per_month: int = 30,
    win_rate: float = 0.35,
) -> Config:
    """Add sales quote pipeline to an existing configuration.

    Generates sales quotes that precede sales orders in the O2C flow.

    Args:
        base_config: Base configuration to extend.
        quotes_per_month: Number of quotes generated per month.
        win_rate: Fraction of quotes that convert to sales orders.

    Returns:
        New Config with sales quotes enabled.
    """
    sq_dict: Dict[str, Any] = {
        "enabled": True,
        "quotes_per_month": quotes_per_month,
        "win_rate": win_rate,
    }
    return base_config.override(sales_quotes=sq_dict)


def with_process_mining(
    base_config: Config,
    events_as_hyperedges: bool = True,
) -> Config:
    """Add OCEL 2.0 process mining and hypergraph integration to an existing configuration.

    Enables OCPM event log generation across all 8 process families (P2P, O2C,
    S2C, H2R, MFG, BANK, AUDIT, Bank Recon) with 88 activity types and 52
    object types. Optionally wires events into the hypergraph as hyperedges.

    Args:
        base_config: Base configuration to extend.
        events_as_hyperedges: Include OCPM events as hyperedges in the hypergraph.

    Returns:
        New Config with OCPM and hypergraph integration enabled.
    """
    ocpm_dict: Dict[str, Any] = {
        "enabled": True,
        "generate_lifecycle_events": True,
        "include_object_relationships": True,
        "compute_variants": True,
    }

    # Ensure graph export is enabled with hypergraph support
    ge_dict: Dict[str, Any] = {
        "enabled": True,
        "hypergraph": {
            "enabled": True,
            "process_layer": {
                "include_p2p": True,
                "include_o2c": True,
                "include_s2c": True,
                "include_h2r": True,
                "include_mfg": True,
                "include_bank": True,
                "include_audit": True,
                "include_r2r": True,
                "events_as_hyperedges": events_as_hyperedges,
            },
        },
    }

    # Preserve existing graph_export formats if present
    if base_config.graph_export is not None:
        if base_config.graph_export.formats is not None:
            ge_dict["formats"] = base_config.graph_export.formats

    return base_config.override(ocpm=ocpm_dict, graph_export=ge_dict)


def with_fraud_packs(
    base_config: Config,
    packs: Optional[List[str]] = None,
    fraud_rate: Optional[float] = None,
) -> Config:
    """Add fraud scenario packs to an existing configuration.

    Fraud packs are pre-configured bundles of fraud patterns that can be
    layered onto any configuration for ML training and audit testing.

    Args:
        base_config: Base configuration to extend.
        packs: List of fraud pack names to enable. Defaults to ["comprehensive"].
            Valid packs: revenue_fraud, payroll_ghost, vendor_kickback,
            management_override, comprehensive.
        fraud_rate: Optional fraud rate override (0.0-1.0).

    Returns:
        New Config with fraud packs enabled.

    Raises:
        ValueError: If an unknown pack name is provided.
    """
    if packs is None:
        packs = ["comprehensive"]
    for pack in packs:
        if pack not in FRAUD_PACKS:
            raise ValueError(
                f"Unknown fraud pack '{pack}'. Valid packs: {', '.join(FRAUD_PACKS)}"
            )
    fraud_dict: Dict[str, Any] = {
        "enabled": True,
        "fraud_packs": packs,
    }
    if fraud_rate is not None:
        fraud_dict["rate"] = fraud_rate
    return base_config.override(fraud=fraud_dict)


def with_scenarios(
    base_config: Config,
    template: str = "fraud_detection",
    with_interventions: bool = True,
) -> Config:
    """Add counterfactual scenario configuration to an existing configuration.

    Enables causal DAG-based scenario generation with interventions
    and counterfactual analysis for what-if modeling.

    Args:
        base_config: Base configuration to extend.
        template: Scenario template (fraud_detection, revenue_impact, supply_chain).
        with_interventions: Enable intervention support for what-if analysis.

    Returns:
        New Config with scenario generation enabled.
    """
    scenario_dict: Dict[str, Any] = {
        "enabled": True,
        "template": template,
        "interventions": {"enabled": with_interventions},
        "counterfactuals": {"enabled": True, "samples_per_record": 5},
    }
    return base_config.override(causal=scenario_dict)


def with_streaming(
    base_config: Config,
    buffer_size: int = 1000,
    backpressure: str = "block",
) -> Config:
    """Add streaming pipeline configuration to an existing configuration.

    Enables the streaming output pipeline for real-time data generation
    with configurable buffering and backpressure strategies.

    Args:
        base_config: Base configuration to extend.
        buffer_size: Size of the stream buffer (default: 1000).
        backpressure: Backpressure strategy (block, drop_oldest, drop_newest, buffer).

    Returns:
        New Config with streaming pipeline enabled.
    """
    valid_strategies = ["block", "drop_oldest", "drop_newest", "buffer"]
    if backpressure not in valid_strategies:
        raise ValueError(
            f"Unknown backpressure strategy '{backpressure}'. "
            f"Valid strategies: {', '.join(valid_strategies)}"
        )
    streaming_dict: Dict[str, Any] = {
        "enabled": True,
        "buffer_size": buffer_size,
        "backpressure": backpressure,
    }
    return base_config.override(streaming=streaming_dict)


def _transactions_to_volume(count: int) -> str:
    """Map transaction count to volume preset."""
    if count <= 10_000:
        return "ten_k"
    elif count <= 100_000:
        return "hundred_k"
    elif count <= 1_000_000:
        return "one_m"
    elif count <= 10_000_000:
        return "ten_m"
    else:
        return "hundred_m"


_REGISTRY: Dict[str, BlueprintFactory] = {
    "retail_small": retail_small,
    "banking_medium": banking_medium,
    "manufacturing_large": manufacturing_large,
    "banking_aml": banking_aml,
    "ml_training": ml_training,
    "audit_engagement": audit_engagement,
    "statistical_validation": statistical_validation,
    "with_llm_enrichment": with_llm_enrichment,
    "with_diffusion": with_diffusion,
    "with_causal": with_causal,
    "with_sourcing": with_sourcing,
    "with_financial_reporting": with_financial_reporting,
    "with_hr": with_hr,
    "with_manufacturing": with_manufacturing,
    "with_process_mining": with_process_mining,
    "with_sales_quotes": with_sales_quotes,
    "with_fraud_packs": with_fraud_packs,
    "with_scenarios": with_scenarios,
    "with_streaming": with_streaming,
}


def list() -> List[str]:
    """List available blueprint names."""
    return sorted(_REGISTRY.keys())


def get(name: str) -> BlueprintFactory:
    """Get a blueprint factory by name."""
    return _REGISTRY[name]
