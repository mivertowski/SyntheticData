"""Blueprint registry for common DataSynth configurations."""

from __future__ import annotations

from typing import Callable, Dict, List, Optional

from datasynth_py.config.models import (
    AdvancedDistributionSettings,
    AuditSettings,
    BankingSettings,
    ChartOfAccountsSettings,
    CompanyConfig,
    Config,
    CorrelationConfig,
    CorrelationFieldConfig,
    CultureDistributionConfig,
    DataQualitySettings,
    DescriptionTemplateConfig,
    EconomicCycleConfig,
    FraudSettings,
    GlobalSettings,
    GraphExportSettings,
    MixtureComponentConfig,
    MixtureDistributionConfig,
    NameTemplateConfig,
    ReferenceTemplateConfig,
    RegimeChangeConfig,
    ScenarioSettings,
    StatisticalTestConfig,
    StatisticalValidationConfig,
    TemplateSettings,
)

BlueprintFactory = Callable[..., Config]


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
}


def list() -> List[str]:
    """List available blueprint names."""
    return sorted(_REGISTRY.keys())


def get(name: str) -> BlueprintFactory:
    """Get a blueprint factory by name."""
    return _REGISTRY[name]
