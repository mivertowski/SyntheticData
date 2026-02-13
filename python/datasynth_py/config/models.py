"""Typed configuration models for the DataSynth Python wrapper."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional

import importlib.util

from datasynth_py.config.validation import ConfigValidationError, validate_config


@dataclass(frozen=True)
class GlobalSettings:
    """Global configuration settings matching the CLI schema."""

    industry: Optional[str] = None
    start_date: Optional[str] = None
    period_months: Optional[int] = None
    seed: Optional[int] = None
    group_currency: Optional[str] = None
    parallel: Optional[bool] = None
    worker_threads: Optional[int] = None
    memory_limit_mb: Optional[int] = None


@dataclass(frozen=True)
class CompanyConfig:
    """Single company configuration matching the CLI schema."""

    code: str
    name: str
    currency: str = "USD"
    country: str = "US"
    annual_transaction_volume: str = "ten_k"
    volume_weight: float = 1.0
    fiscal_year_variant: str = "K4"


@dataclass(frozen=True)
class ChartOfAccountsSettings:
    """Chart of Accounts configuration matching the CLI schema."""

    complexity: Optional[str] = None
    industry_specific: Optional[bool] = None


@dataclass(frozen=True)
class TransactionSettings:
    """Transaction generation settings."""

    # These are higher-level settings that map to the CLI schema
    count: Optional[int] = None
    currency: Optional[str] = None
    anomaly_rate: Optional[float] = None


@dataclass(frozen=True)
class OutputSettings:
    """Output configuration matching the CLI schema."""

    output_directory: Optional[str] = None
    formats: Optional[List[str]] = None
    compression_enabled: Optional[bool] = None
    compression_level: Optional[int] = None


@dataclass(frozen=True)
class FraudSettings:
    """Fraud simulation settings."""

    enabled: Optional[bool] = None
    rate: Optional[float] = None


@dataclass(frozen=True)
class BankingSettings:
    """Banking KYC/AML generation settings."""

    enabled: bool = False
    retail_customers: Optional[int] = None
    business_customers: Optional[int] = None
    trusts: Optional[int] = None
    typologies_enabled: Optional[bool] = None
    spoofing_enabled: Optional[bool] = None


@dataclass(frozen=True)
class ScenarioSettings:
    """Scenario configuration for metadata and tagging."""

    tags: Optional[List[str]] = None
    profile: Optional[str] = None
    ml_training: bool = False
    target_anomaly_ratio: Optional[float] = None
    description: Optional[str] = None
    metadata: Optional[Dict[str, str]] = None


@dataclass(frozen=True)
class TemporalDriftSettings:
    """Temporal drift configuration for distribution changes over time."""

    enabled: bool = False
    amount_mean_drift: float = 0.02
    amount_variance_drift: float = 0.01
    anomaly_rate_drift: float = 0.0
    concept_drift_rate: float = 0.0
    drift_type: str = "gradual"
    seasonal_drift: bool = True
    drift_start_period: Optional[int] = None


@dataclass(frozen=True)
class DataQualitySettings:
    """Data quality injection settings."""

    enabled: bool = False
    missing_rate: float = 0.05
    typo_rate: float = 0.02
    format_variation_rate: float = 0.03
    duplicate_rate: float = 0.01
    encoding_issue_rate: float = 0.005


@dataclass(frozen=True)
class ProcessLayerSettings:
    """Process layer settings for hypergraph construction.

    Controls which enterprise process families are included as Layer 2
    nodes in the hypergraph and whether OCPM events become hyperedges.
    """

    include_p2p: bool = True
    include_o2c: bool = True
    include_s2c: bool = True
    include_h2r: bool = True
    include_mfg: bool = True
    include_bank: bool = True
    include_audit: bool = True
    include_r2r: bool = True
    events_as_hyperedges: bool = True
    docs_per_counterparty_threshold: int = 20


@dataclass(frozen=True)
class HypergraphSettings:
    """Hypergraph export configuration.

    Multi-layer hypergraph with entity type codes 100-400 covering
    master data, process events, and OCPM hyperedges.
    """

    enabled: bool = False
    process_layer: Optional[ProcessLayerSettings] = None


@dataclass(frozen=True)
class GraphExportSettings:
    """Graph export configuration for accounting network ML training."""

    enabled: bool = False
    formats: Optional[List[str]] = None
    graph_types: Optional[List[str]] = None
    train_ratio: float = 0.7
    validation_ratio: float = 0.15
    output_subdirectory: str = "graphs"
    hypergraph: Optional[HypergraphSettings] = None


@dataclass(frozen=True)
class OcpmProcessSettings:
    """Process-specific OCPM configuration."""

    rework_probability: float = 0.05
    skip_step_probability: float = 0.02
    out_of_order_probability: float = 0.03


@dataclass(frozen=True)
class OcpmOutputSettings:
    """OCPM output format configuration."""

    ocel_json: bool = True
    ocel_xml: bool = False
    xes: bool = False
    xes_include_lifecycle: bool = True
    xes_include_resources: bool = True


@dataclass(frozen=True)
class OcpmSettings:
    """OCPM (Object-Centric Process Mining) configuration.

    Controls generation of OCEL 2.0 compatible event logs with 88 activity
    types across 8 process families (P2P, O2C, S2C, H2R, MFG, BANK, AUDIT,
    Bank Recon) and 52 object types with lifecycle states.
    """

    enabled: bool = False
    generate_lifecycle_events: bool = True
    include_object_relationships: bool = True
    compute_variants: bool = True
    max_variants: int = 0
    p2p_process: Optional[OcpmProcessSettings] = None
    o2c_process: Optional[OcpmProcessSettings] = None
    output: Optional[OcpmOutputSettings] = None


@dataclass(frozen=True)
class AuditSettings:
    """Audit data generation settings."""

    enabled: bool = False
    engagements: int = 5
    workpapers_per_engagement: int = 20
    evidence_per_workpaper: int = 5
    risks_per_engagement: int = 15
    findings_per_engagement: int = 8


@dataclass(frozen=True)
class StreamingSettings:
    """Streaming output API configuration."""

    enabled: bool = False
    buffer_size: int = 1000
    enable_progress: bool = True
    progress_interval: int = 100
    backpressure: str = "block"  # block, drop_oldest, drop_newest, buffer


@dataclass(frozen=True)
class RateLimitSettings:
    """Rate limiting configuration for controlled generation throughput."""

    enabled: bool = False
    entities_per_second: float = 10000.0
    burst_size: int = 100
    backpressure: str = "block"  # block, drop, buffer


# ============================================================================
# Advanced Distribution Configuration
# ============================================================================


@dataclass(frozen=True)
class MixtureComponentConfig:
    """Single component of a mixture distribution."""

    weight: float = 1.0
    mu: float = 6.0  # Log-normal mean (log scale)
    sigma: float = 1.5  # Log-normal std dev
    label: Optional[str] = None  # Optional label (e.g., "routine", "major")


@dataclass(frozen=True)
class MixtureDistributionConfig:
    """Mixture distribution configuration for amount generation."""

    enabled: bool = False
    distribution_type: str = "lognormal"  # lognormal, gaussian
    components: Optional[List[MixtureComponentConfig]] = None
    benford_compliance: bool = True


@dataclass(frozen=True)
class CorrelationFieldConfig:
    """Configuration for a single correlated field."""

    name: str
    distribution_type: str = "normal"  # normal, lognormal, uniform
    min_value: Optional[float] = None
    max_value: Optional[float] = None


@dataclass(frozen=True)
class CorrelationConfig:
    """Cross-field correlation configuration."""

    enabled: bool = False
    copula_type: str = "gaussian"  # gaussian, clayton, gumbel, frank, student_t
    fields: Optional[List[CorrelationFieldConfig]] = None
    matrix: Optional[List[List[float]]] = None  # Correlation matrix


@dataclass(frozen=True)
class ConditionalBreakpoint:
    """Breakpoint for conditional distributions."""

    threshold: float
    distribution_type: str = "lognormal"
    mu: float = 6.0
    sigma: float = 1.5


@dataclass(frozen=True)
class ConditionalDistributionConfig:
    """Conditional distribution configuration."""

    dependent_field: str
    condition_field: str
    breakpoints: Optional[List[ConditionalBreakpoint]] = None


@dataclass(frozen=True)
class RegimeChangeEventConfig:
    """Single regime change event configuration."""

    date: str  # ISO date string
    change_type: str  # acquisition, divestiture, policy_change, price_increase, restructuring
    description: Optional[str] = None
    volume_multiplier: float = 1.0
    amount_mean_shift: float = 0.0
    amount_variance_shift: float = 0.0


@dataclass(frozen=True)
class EconomicCycleConfig:
    """Economic cycle configuration for regime changes."""

    enabled: bool = False
    cycle_period_months: int = 48
    amplitude: float = 0.15
    recession_probability: float = 0.1
    recession_depth: float = 0.25


@dataclass(frozen=True)
class RegimeChangeConfig:
    """Regime change configuration for temporal distribution shifts."""

    enabled: bool = False
    changes: Optional[List[RegimeChangeEventConfig]] = None
    economic_cycle: Optional[EconomicCycleConfig] = None


@dataclass(frozen=True)
class StatisticalTestConfig:
    """Configuration for a single statistical validation test."""

    test_type: str  # benford_first_digit, distribution_fit, correlation_check, chi_squared, anderson_darling
    significance: float = 0.05
    threshold_mad: Optional[float] = None  # For Benford tests
    target_distribution: Optional[str] = None  # For distribution fit tests


@dataclass(frozen=True)
class StatisticalValidationConfig:
    """Statistical validation configuration."""

    enabled: bool = False
    tests: Optional[List[StatisticalTestConfig]] = None
    report_path: Optional[str] = None
    fail_on_violation: bool = False


@dataclass(frozen=True)
class AdvancedDistributionSettings:
    """Advanced statistical distribution configuration.

    Supports mixture models, cross-field correlations, conditional distributions,
    regime changes, and statistical validation.
    """

    enabled: bool = False
    amounts: Optional[MixtureDistributionConfig] = None
    correlations: Optional[CorrelationConfig] = None
    conditional: Optional[List[ConditionalDistributionConfig]] = None
    regime_changes: Optional[RegimeChangeConfig] = None
    industry_profile: Optional[str] = None  # retail, manufacturing, financial_services
    validation: Optional[StatisticalValidationConfig] = None


# ============================================================================
# Template/Realism Configuration
# ============================================================================


@dataclass(frozen=True)
class CultureDistributionConfig:
    """Distribution of name cultures for realistic generation."""

    western_us: float = 0.40
    hispanic: float = 0.20
    german: float = 0.10
    french: float = 0.05
    chinese: float = 0.10
    japanese: float = 0.05
    indian: float = 0.10


@dataclass(frozen=True)
class NameTemplateConfig:
    """Name generation template configuration."""

    culture_distribution: Optional[CultureDistributionConfig] = None
    email_domain: str = "company.com"
    generate_realistic_names: bool = True


@dataclass(frozen=True)
class DescriptionTemplateConfig:
    """Description generation template configuration."""

    generate_header_text: bool = True
    generate_line_text: bool = True


@dataclass(frozen=True)
class ReferenceTemplateConfig:
    """Reference number template configuration."""

    generate_references: bool = True
    invoice_prefix: str = "INV"
    po_prefix: str = "PO"
    so_prefix: str = "SO"


@dataclass(frozen=True)
class TemplateSettings:
    """Template configuration for realistic data generation.

    Controls name generation, description text, and reference number formats
    with support for cultural diversity and industry-specific patterns.
    """

    names: Optional[NameTemplateConfig] = None
    descriptions: Optional[DescriptionTemplateConfig] = None
    references: Optional[ReferenceTemplateConfig] = None


@dataclass(frozen=True)
class ValidTimeSettings:
    """Valid time configuration for temporal attributes."""

    closed_probability: float = 0.1
    avg_validity_days: int = 365
    validity_stddev_days: int = 90


@dataclass(frozen=True)
class TransactionTimeSettings:
    """Transaction time configuration for temporal attributes."""

    avg_recording_delay_seconds: int = 0
    allow_backdating: bool = False
    backdating_probability: float = 0.01


@dataclass(frozen=True)
class TemporalAttributeSettings:
    """Temporal attribute generation configuration for bi-temporal data."""

    enabled: bool = False
    valid_time: Optional[ValidTimeSettings] = None
    transaction_time: Optional[TransactionTimeSettings] = None
    generate_version_chains: bool = False
    avg_versions_per_entity: float = 1.5


@dataclass(frozen=True)
class CardinalityRule:
    """Cardinality rule for relationship generation."""

    rule_type: str  # one_to_one, one_to_many, many_to_one, many_to_many
    min_count: Optional[int] = None
    max_count: Optional[int] = None


@dataclass(frozen=True)
class RelationshipTypeConfig:
    """Configuration for a single relationship type."""

    name: str
    source_type: str
    target_type: str
    cardinality: Optional[CardinalityRule] = None
    weight: float = 1.0


@dataclass(frozen=True)
class RelationshipSettings:
    """Relationship generation configuration."""

    enabled: bool = False
    relationship_types: Optional[List[RelationshipTypeConfig]] = None
    allow_orphans: bool = True
    orphan_probability: float = 0.01
    allow_circular: bool = False
    max_circular_depth: int = 3


# ============================================================================
# Temporal Patterns Configuration
# ============================================================================


@dataclass(frozen=True)
class SettlementRulesConfig:
    """Settlement rules configuration for different instrument types."""

    equity_days: int = 2
    government_bonds_days: int = 1
    fx_spot_days: int = 2
    wire_cutoff_time: str = "14:00"


@dataclass(frozen=True)
class BusinessDaySchemaConfig:
    """Business day calculation configuration."""

    enabled: bool = False
    half_day_policy: str = "full_day"  # full_day, half_day, non_business_day
    settlement_rules: Optional[SettlementRulesConfig] = None


@dataclass(frozen=True)
class CalendarSchemaConfig:
    """Regional calendar configuration."""

    regions: Optional[List[str]] = None  # US, DE, GB, CN, JP, IN, BR, MX, AU, SG, KR
    custom_holidays: Optional[List[str]] = None  # ISO date strings


@dataclass(frozen=True)
class PeriodEndModelConfig:
    """Period-end decay model configuration."""

    start_day: int = -10
    base_multiplier: float = 1.0
    peak_multiplier: float = 3.5
    decay_rate: float = 0.3
    sustained_high_days: Optional[int] = None  # For extended_crunch model


@dataclass(frozen=True)
class PeriodEndSchemaConfig:
    """Period-end dynamics configuration."""

    enabled: bool = False
    model: str = "flat"  # flat, exponential, daily_profile, extended_crunch
    month_end: Optional[PeriodEndModelConfig] = None
    quarter_end: Optional[PeriodEndModelConfig] = None
    year_end: Optional[PeriodEndModelConfig] = None


@dataclass(frozen=True)
class LagDistributionConfig:
    """Lag distribution parameters (log-normal)."""

    mu: float = 0.5
    sigma: float = 0.8
    min_lag_hours: float = 0.0
    max_lag_hours: float = 48.0


@dataclass(frozen=True)
class CrossDayPostingConfig:
    """Cross-day posting configuration."""

    enabled: bool = False
    probability_by_hour: Optional[Dict[int, float]] = None  # hour -> probability


@dataclass(frozen=True)
class ProcessingLagSchemaConfig:
    """Processing lag modeling configuration."""

    enabled: bool = False
    sales_order_lag: Optional[LagDistributionConfig] = None
    goods_receipt_lag: Optional[LagDistributionConfig] = None
    invoice_receipt_lag: Optional[LagDistributionConfig] = None
    payment_lag: Optional[LagDistributionConfig] = None
    journal_entry_lag: Optional[LagDistributionConfig] = None
    cross_day_posting: Optional[CrossDayPostingConfig] = None


@dataclass(frozen=True)
class FourFourFiveSchemaConfig:
    """4-4-5 retail calendar configuration."""

    anchor: str = "last_saturday_of"  # first_sunday_of, last_saturday_of
    anchor_month: int = 1  # 1-12
    week_pattern: str = "four_four_five"  # four_four_five, four_five_four, five_four_four
    leap_week_placement: str = "q4_period3"  # q4_period3, q1_period1


@dataclass(frozen=True)
class FiscalCalendarSchemaConfig:
    """Fiscal calendar configuration."""

    enabled: bool = False
    calendar_type: str = "calendar_year"  # calendar_year, custom, four_four_five, thirteen_period
    year_start_month: Optional[int] = None  # 1-12 for custom
    year_start_day: Optional[int] = None  # 1-31 for custom
    four_four_five: Optional[FourFourFiveSchemaConfig] = None


@dataclass(frozen=True)
class IntraDaySegmentConfig:
    """Intra-day time segment configuration."""

    name: str
    start: str  # HH:MM format
    end: str  # HH:MM format
    multiplier: float = 1.0
    posting_type: str = "both"  # human, system, both


@dataclass(frozen=True)
class IntraDaySchemaConfig:
    """Intra-day patterns configuration."""

    enabled: bool = False
    segments: Optional[List[IntraDaySegmentConfig]] = None


@dataclass(frozen=True)
class EntityTimezoneMappingConfig:
    """Entity-to-timezone mapping configuration."""

    pattern: str  # e.g., "EU_*", "*_APAC", "1000"
    timezone: str  # IANA timezone name


@dataclass(frozen=True)
class TimezoneSchemaConfig:
    """Timezone handling configuration."""

    enabled: bool = False
    default_timezone: str = "America/New_York"
    consolidation_timezone: str = "UTC"
    entity_mappings: Optional[List[EntityTimezoneMappingConfig]] = None


@dataclass(frozen=True)
class TemporalPatternsConfig:
    """Comprehensive temporal patterns configuration.

    Controls business day calculations, regional calendars, period-end dynamics,
    processing lags, fiscal calendars, intra-day patterns, and timezone handling.
    """

    enabled: bool = False
    business_days: Optional[BusinessDaySchemaConfig] = None
    calendars: Optional[CalendarSchemaConfig] = None
    period_end: Optional[PeriodEndSchemaConfig] = None
    processing_lags: Optional[ProcessingLagSchemaConfig] = None
    fiscal_calendar: Optional[FiscalCalendarSchemaConfig] = None
    intraday: Optional[IntraDaySchemaConfig] = None
    timezones: Optional[TimezoneSchemaConfig] = None


# ============================================================================
# Accounting Standards Configuration (ASC 606, ASC 842, ASC 820, ASC 360)
# ============================================================================


@dataclass(frozen=True)
class RevenueRecognitionConfig:
    """Revenue recognition configuration (ASC 606/IFRS 15)."""

    enabled: bool = False
    generate_contracts: bool = True
    avg_obligations_per_contract: float = 2.0
    variable_consideration_rate: float = 0.15
    over_time_recognition_rate: float = 0.30
    contract_count: int = 100


@dataclass(frozen=True)
class LeaseAccountingConfig:
    """Lease accounting configuration (ASC 842/IFRS 16)."""

    enabled: bool = False
    lease_count: int = 50
    finance_lease_percent: float = 0.30
    avg_lease_term_months: int = 60
    generate_amortization: bool = True
    real_estate_percent: float = 0.40


@dataclass(frozen=True)
class FairValueConfig:
    """Fair value measurement configuration (ASC 820/IFRS 13)."""

    enabled: bool = False
    measurement_count: int = 30
    level1_percent: float = 0.60
    level2_percent: float = 0.30
    level3_percent: float = 0.10
    include_sensitivity_analysis: bool = True


@dataclass(frozen=True)
class ImpairmentConfig:
    """Impairment testing configuration (ASC 360/IAS 36)."""

    enabled: bool = False
    test_count: int = 15
    impairment_rate: float = 0.20
    generate_projections: bool = True
    include_goodwill: bool = True


@dataclass(frozen=True)
class AccountingStandardsConfig:
    """Accounting standards framework configuration.

    Supports US GAAP and IFRS with dual reporting mode:
    - ASC 606/IFRS 15: Revenue Recognition
    - ASC 842/IFRS 16: Lease Accounting
    - ASC 820/IFRS 13: Fair Value Measurement
    - ASC 360/IAS 36: Impairment Testing
    """

    enabled: bool = False
    framework: str = "us_gaap"  # us_gaap, ifrs, dual_reporting
    revenue_recognition: Optional[RevenueRecognitionConfig] = None
    leases: Optional[LeaseAccountingConfig] = None
    fair_value: Optional[FairValueConfig] = None
    impairment: Optional[ImpairmentConfig] = None
    generate_differences: bool = False


# ============================================================================
# Audit Standards Configuration (ISA, PCAOB, SOX)
# ============================================================================


@dataclass(frozen=True)
class IsaComplianceConfig:
    """ISA compliance tracking configuration."""

    enabled: bool = False
    compliance_level: str = "standard"  # basic, standard, comprehensive
    generate_isa_mappings: bool = True
    generate_coverage_summary: bool = True
    include_pcaob: bool = False
    framework: str = "isa"  # isa, pcaob, dual


@dataclass(frozen=True)
class AnalyticalProceduresConfig:
    """Analytical procedures configuration (ISA 520)."""

    enabled: bool = False
    procedures_per_account: int = 3
    variance_probability: float = 0.20
    generate_investigations: bool = True
    include_ratio_analysis: bool = True


@dataclass(frozen=True)
class ConfirmationsConfig:
    """External confirmations configuration (ISA 505)."""

    enabled: bool = False
    confirmation_count: int = 50
    positive_response_rate: float = 0.85
    exception_rate: float = 0.10
    non_response_rate: float = 0.05
    generate_alternative_procedures: bool = True


@dataclass(frozen=True)
class AuditOpinionConfig:
    """Audit opinion configuration (ISA 700/705/706/701)."""

    enabled: bool = False
    generate_kam: bool = True
    average_kam_count: int = 3
    modified_opinion_rate: float = 0.05
    include_emphasis_of_matter: bool = True
    include_going_concern: bool = True


@dataclass(frozen=True)
class SoxComplianceConfig:
    """SOX 302/404 compliance configuration."""

    enabled: bool = False
    generate_302_certifications: bool = True
    generate_404_assessments: bool = True
    materiality_threshold: float = 10000.0
    material_weakness_rate: float = 0.02
    significant_deficiency_rate: float = 0.05


@dataclass(frozen=True)
class PcaobConfig:
    """PCAOB-specific audit configuration."""

    enabled: bool = False
    is_pcaob_audit: bool = False
    generate_cam: bool = True
    include_icfr_opinion: bool = True
    generate_standard_mappings: bool = True


@dataclass(frozen=True)
class AuditStandardsConfig:
    """Audit standards framework configuration.

    Supports ISA and PCAOB standards:
    - ISA 200-720: International Standards on Auditing
    - ISA 520: Analytical Procedures
    - ISA 505: External Confirmations
    - ISA 700/705/706/701: Audit Reports
    - PCAOB AS: US Auditing Standards
    - SOX 302/404: Sarbanes-Oxley Compliance
    """

    enabled: bool = False
    isa_compliance: Optional[IsaComplianceConfig] = None
    analytical_procedures: Optional[AnalyticalProceduresConfig] = None
    confirmations: Optional[ConfirmationsConfig] = None
    opinion: Optional[AuditOpinionConfig] = None
    generate_audit_trail: bool = False
    sox: Optional[SoxComplianceConfig] = None
    pcaob: Optional[PcaobConfig] = None


# ============================================================================
# Enterprise Process Chain Configurations (v0.6.0)
# ============================================================================


@dataclass(frozen=True)
class SourceToPayConfig:
    """Source-to-pay process chain configuration."""
    enabled: bool = False
    spend_analysis: Optional[Dict[str, Any]] = None
    sourcing: Optional[Dict[str, Any]] = None
    qualification: Optional[Dict[str, Any]] = None
    rfx: Optional[Dict[str, Any]] = None
    contracts: Optional[Dict[str, Any]] = None
    catalog: Optional[Dict[str, Any]] = None
    scorecards: Optional[Dict[str, Any]] = None
    p2p_integration: Optional[Dict[str, Any]] = None


@dataclass(frozen=True)
class ManagementKpisSchemaConfig:
    """Management KPI generation configuration."""
    enabled: bool = False
    frequency: str = "monthly"


@dataclass(frozen=True)
class BudgetSchemaConfig:
    """Budget generation configuration."""
    enabled: bool = False
    revenue_growth_rate: float = 0.05
    expense_inflation_rate: float = 0.03
    variance_noise: float = 0.10


@dataclass(frozen=True)
class FinancialReportingConfig:
    """Financial reporting generation configuration."""
    enabled: bool = False
    generate_balance_sheet: bool = True
    generate_income_statement: bool = True
    generate_cash_flow: bool = True
    generate_changes_in_equity: bool = True
    comparative_periods: int = 1
    management_kpis: Optional[ManagementKpisSchemaConfig] = None
    budgets: Optional[BudgetSchemaConfig] = None


@dataclass(frozen=True)
class PayrollSchemaConfig:
    """Payroll generation configuration."""
    enabled: bool = True
    pay_frequency: str = "monthly"
    benefits_enrollment_rate: float = 0.60
    retirement_participation_rate: float = 0.45


@dataclass(frozen=True)
class TimeAttendanceSchemaConfig:
    """Time and attendance configuration."""
    enabled: bool = True
    overtime_rate: float = 0.10


@dataclass(frozen=True)
class ExpenseSchemaConfig:
    """Expense management configuration."""
    enabled: bool = True
    submission_rate: float = 0.30
    policy_violation_rate: float = 0.08


@dataclass(frozen=True)
class HrConfig:
    """HR (Hire-to-Retire) process configuration."""
    enabled: bool = False
    payroll: Optional[PayrollSchemaConfig] = None
    time_attendance: Optional[TimeAttendanceSchemaConfig] = None
    expenses: Optional[ExpenseSchemaConfig] = None


@dataclass(frozen=True)
class ProductionOrderSchemaConfig:
    """Production order configuration."""
    orders_per_month: int = 50
    avg_batch_size: int = 100
    yield_rate: float = 0.97
    make_to_order_rate: float = 0.20
    rework_rate: float = 0.03


@dataclass(frozen=True)
class ManufacturingCostingSchemaConfig:
    """Manufacturing costing configuration."""
    labor_rate_per_hour: float = 35.0
    overhead_rate: float = 1.50
    standard_cost_update_frequency: str = "quarterly"


@dataclass(frozen=True)
class RoutingSchemaConfig:
    """Routing configuration for production operations."""
    avg_operations: int = 4
    setup_time_hours: float = 1.5
    run_time_variation: float = 0.15


@dataclass(frozen=True)
class ManufacturingProcessConfig:
    """Manufacturing process configuration."""
    enabled: bool = False
    production_orders: Optional[ProductionOrderSchemaConfig] = None
    costing: Optional[ManufacturingCostingSchemaConfig] = None
    routing: Optional[RoutingSchemaConfig] = None


@dataclass(frozen=True)
class SalesQuoteSchemaConfig:
    """Sales quote pipeline configuration."""
    enabled: bool = False
    quotes_per_month: int = 30
    win_rate: float = 0.35
    validity_days: int = 30


@dataclass(frozen=True)
class VendorNetworkConfig:
    """Vendor network generation configuration."""
    enabled: bool = False
    depth: int = 3
    tiers: Optional[Dict[str, Any]] = None
    clusters: Optional[Dict[str, Any]] = None
    dependencies: Optional[Dict[str, Any]] = None


@dataclass(frozen=True)
class CustomerSegmentationConfig:
    """Customer segmentation generation configuration."""
    enabled: bool = False
    value_segments: Optional[Dict[str, Any]] = None
    lifecycle: Optional[Dict[str, Any]] = None
    networks: Optional[Dict[str, Any]] = None


@dataclass(frozen=True)
class RelationshipStrengthConfig:
    """Relationship strength calculation configuration."""
    enabled: bool = False
    calculation: Optional[Dict[str, Any]] = None
    thresholds: Optional[Dict[str, Any]] = None


@dataclass(frozen=True)
class CrossProcessLinksConfig:
    """Cross-process linkage configuration."""
    enabled: bool = False
    inventory_p2p_o2c: bool = True
    payment_bank_reconciliation: bool = True
    intercompany_bilateral: bool = True
    inventory_link_rate: float = 0.30


@dataclass(frozen=True)
class Config:
    """Root configuration container.

    This model maps to the datasynth-cli GeneratorConfig schema.
    """

    global_settings: Optional[GlobalSettings] = None
    companies: Optional[List[CompanyConfig]] = None
    chart_of_accounts: Optional[ChartOfAccountsSettings] = None
    transactions: Optional[TransactionSettings] = None
    output: Optional[OutputSettings] = None
    fraud: Optional[FraudSettings] = None
    banking: Optional[BankingSettings] = None
    scenario: Optional[ScenarioSettings] = None
    temporal: Optional[TemporalDriftSettings] = None
    data_quality: Optional[DataQualitySettings] = None
    graph_export: Optional[GraphExportSettings] = None
    ocpm: Optional[OcpmSettings] = None
    audit: Optional[AuditSettings] = None
    streaming: Optional[StreamingSettings] = None
    rate_limit: Optional[RateLimitSettings] = None
    temporal_attributes: Optional[TemporalAttributeSettings] = None
    relationships: Optional[RelationshipSettings] = None
    accounting_standards: Optional[AccountingStandardsConfig] = None
    audit_standards: Optional[AuditStandardsConfig] = None
    distributions: Optional[AdvancedDistributionSettings] = None
    templates: Optional[TemplateSettings] = None
    temporal_patterns: Optional[TemporalPatternsConfig] = None
    llm: Optional[Dict[str, Any]] = None
    diffusion: Optional[Dict[str, Any]] = None
    causal: Optional[Dict[str, Any]] = None
    source_to_pay: Optional[SourceToPayConfig] = None
    financial_reporting: Optional[FinancialReportingConfig] = None
    hr: Optional[HrConfig] = None
    manufacturing: Optional[ManufacturingProcessConfig] = None
    sales_quotes: Optional[SalesQuoteSchemaConfig] = None
    vendor_network: Optional[VendorNetworkConfig] = None
    customer_segmentation: Optional[CustomerSegmentationConfig] = None
    relationship_strength: Optional[RelationshipStrengthConfig] = None
    cross_process_links: Optional[CrossProcessLinksConfig] = None
    extra: Dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary matching CLI schema."""
        payload: Dict[str, Any] = {}

        if self.global_settings is not None:
            payload["global"] = _strip_none(self.global_settings.__dict__)

        if self.companies is not None:
            payload["companies"] = [
                _strip_none(c.__dict__) for c in self.companies
            ]

        if self.chart_of_accounts is not None:
            payload["chart_of_accounts"] = _strip_none(self.chart_of_accounts.__dict__)

        if self.transactions is not None:
            tx_dict = _strip_none(self.transactions.__dict__)
            # Map higher-level settings to CLI schema structure
            cli_transactions: Dict[str, Any] = {}
            if "count" in tx_dict:
                # The CLI doesn't have a direct 'count' field in transactions;
                # transaction count is derived from company volume settings
                pass
            if "currency" in tx_dict:
                # Currency is per-company in CLI schema
                pass
            if cli_transactions:
                payload["transactions"] = cli_transactions

        if self.output is not None:
            out_dict = _strip_none(self.output.__dict__)
            cli_output: Dict[str, Any] = {}
            if "output_directory" in out_dict:
                cli_output["output_directory"] = out_dict["output_directory"]
            if "formats" in out_dict:
                cli_output["formats"] = out_dict["formats"]
            if "compression_enabled" in out_dict or "compression_level" in out_dict:
                compression: Dict[str, Any] = {}
                if "compression_enabled" in out_dict:
                    compression["enabled"] = out_dict["compression_enabled"]
                if "compression_level" in out_dict:
                    compression["level"] = out_dict["compression_level"]
                cli_output["compression"] = compression
            if cli_output:
                payload["output"] = cli_output

        if self.fraud is not None:
            fraud_dict = _strip_none(self.fraud.__dict__)
            if fraud_dict:
                payload["fraud"] = fraud_dict

        if self.banking is not None:
            banking_dict = _strip_none(self.banking.__dict__)
            if banking_dict:
                payload["banking"] = banking_dict

        if self.scenario is not None:
            scenario_dict = _strip_none(self.scenario.__dict__)
            if scenario_dict:
                payload["scenario"] = scenario_dict

        if self.temporal is not None:
            temporal_dict = _strip_none(self.temporal.__dict__)
            if temporal_dict:
                payload["temporal"] = temporal_dict

        if self.data_quality is not None:
            dq_dict = _strip_none(self.data_quality.__dict__)
            if dq_dict:
                payload["data_quality"] = dq_dict

        if self.graph_export is not None:
            graph_dict = _strip_none(self.graph_export.__dict__)
            if "hypergraph" in graph_dict and self.graph_export.hypergraph is not None:
                hg_dict: Dict[str, Any] = {
                    "enabled": self.graph_export.hypergraph.enabled,
                }
                if self.graph_export.hypergraph.process_layer is not None:
                    hg_dict["process_layer"] = _strip_none(
                        self.graph_export.hypergraph.process_layer.__dict__
                    )
                graph_dict["hypergraph"] = hg_dict
            if graph_dict:
                payload["graph_export"] = graph_dict

        if self.ocpm is not None:
            ocpm_dict: Dict[str, Any] = {
                "enabled": self.ocpm.enabled,
                "generate_lifecycle_events": self.ocpm.generate_lifecycle_events,
                "include_object_relationships": self.ocpm.include_object_relationships,
                "compute_variants": self.ocpm.compute_variants,
                "max_variants": self.ocpm.max_variants,
            }
            if self.ocpm.p2p_process is not None:
                ocpm_dict["p2p_process"] = _strip_none(self.ocpm.p2p_process.__dict__)
            if self.ocpm.o2c_process is not None:
                ocpm_dict["o2c_process"] = _strip_none(self.ocpm.o2c_process.__dict__)
            if self.ocpm.output is not None:
                ocpm_dict["output"] = _strip_none(self.ocpm.output.__dict__)
            payload["ocpm"] = ocpm_dict

        if self.audit is not None:
            audit_dict = _strip_none(self.audit.__dict__)
            if audit_dict:
                payload["audit"] = audit_dict

        if self.streaming is not None:
            streaming_dict = _strip_none(self.streaming.__dict__)
            if streaming_dict:
                payload["streaming"] = streaming_dict

        if self.rate_limit is not None:
            rate_limit_dict = _strip_none(self.rate_limit.__dict__)
            if rate_limit_dict:
                payload["rate_limit"] = rate_limit_dict

        if self.temporal_attributes is not None:
            ta_dict: Dict[str, Any] = {"enabled": self.temporal_attributes.enabled}
            if self.temporal_attributes.valid_time is not None:
                ta_dict["valid_time"] = _strip_none(self.temporal_attributes.valid_time.__dict__)
            if self.temporal_attributes.transaction_time is not None:
                ta_dict["transaction_time"] = _strip_none(
                    self.temporal_attributes.transaction_time.__dict__
                )
            ta_dict["generate_version_chains"] = self.temporal_attributes.generate_version_chains
            ta_dict["avg_versions_per_entity"] = self.temporal_attributes.avg_versions_per_entity
            payload["temporal_attributes"] = ta_dict

        if self.relationships is not None:
            rel_dict: Dict[str, Any] = {"enabled": self.relationships.enabled}
            if self.relationships.relationship_types is not None:
                rel_dict["relationship_types"] = [
                    {
                        "name": rt.name,
                        "source_type": rt.source_type,
                        "target_type": rt.target_type,
                        "weight": rt.weight,
                        **(
                            {
                                "cardinality": {
                                    "rule_type": rt.cardinality.rule_type,
                                    **({"min": rt.cardinality.min_count} if rt.cardinality.min_count else {}),
                                    **({"max": rt.cardinality.max_count} if rt.cardinality.max_count else {}),
                                }
                            }
                            if rt.cardinality
                            else {}
                        ),
                    }
                    for rt in self.relationships.relationship_types
                ]
            rel_dict["allow_orphans"] = self.relationships.allow_orphans
            rel_dict["orphan_probability"] = self.relationships.orphan_probability
            rel_dict["allow_circular"] = self.relationships.allow_circular
            rel_dict["max_circular_depth"] = self.relationships.max_circular_depth
            payload["relationships"] = rel_dict

        if self.accounting_standards is not None:
            acct_dict: Dict[str, Any] = {
                "enabled": self.accounting_standards.enabled,
                "framework": self.accounting_standards.framework,
                "generate_differences": self.accounting_standards.generate_differences,
            }
            if self.accounting_standards.revenue_recognition is not None:
                acct_dict["revenue_recognition"] = _strip_none(
                    self.accounting_standards.revenue_recognition.__dict__
                )
            if self.accounting_standards.leases is not None:
                acct_dict["leases"] = _strip_none(self.accounting_standards.leases.__dict__)
            if self.accounting_standards.fair_value is not None:
                acct_dict["fair_value"] = _strip_none(
                    self.accounting_standards.fair_value.__dict__
                )
            if self.accounting_standards.impairment is not None:
                acct_dict["impairment"] = _strip_none(
                    self.accounting_standards.impairment.__dict__
                )
            payload["accounting_standards"] = acct_dict

        if self.audit_standards is not None:
            audit_std_dict: Dict[str, Any] = {
                "enabled": self.audit_standards.enabled,
                "generate_audit_trail": self.audit_standards.generate_audit_trail,
            }
            if self.audit_standards.isa_compliance is not None:
                audit_std_dict["isa_compliance"] = _strip_none(
                    self.audit_standards.isa_compliance.__dict__
                )
            if self.audit_standards.analytical_procedures is not None:
                audit_std_dict["analytical_procedures"] = _strip_none(
                    self.audit_standards.analytical_procedures.__dict__
                )
            if self.audit_standards.confirmations is not None:
                audit_std_dict["confirmations"] = _strip_none(
                    self.audit_standards.confirmations.__dict__
                )
            if self.audit_standards.opinion is not None:
                audit_std_dict["opinion"] = _strip_none(
                    self.audit_standards.opinion.__dict__
                )
            if self.audit_standards.sox is not None:
                audit_std_dict["sox"] = _strip_none(self.audit_standards.sox.__dict__)
            if self.audit_standards.pcaob is not None:
                audit_std_dict["pcaob"] = _strip_none(self.audit_standards.pcaob.__dict__)
            payload["audit_standards"] = audit_std_dict

        if self.distributions is not None:
            dist_dict: Dict[str, Any] = {"enabled": self.distributions.enabled}
            if self.distributions.amounts is not None:
                amounts_dict: Dict[str, Any] = {
                    "enabled": self.distributions.amounts.enabled,
                    "distribution_type": self.distributions.amounts.distribution_type,
                    "benford_compliance": self.distributions.amounts.benford_compliance,
                }
                if self.distributions.amounts.components is not None:
                    amounts_dict["components"] = [
                        _strip_none(c.__dict__) for c in self.distributions.amounts.components
                    ]
                dist_dict["amounts"] = amounts_dict
            if self.distributions.correlations is not None:
                corr_dict: Dict[str, Any] = {
                    "enabled": self.distributions.correlations.enabled,
                    "copula_type": self.distributions.correlations.copula_type,
                }
                if self.distributions.correlations.fields is not None:
                    corr_dict["fields"] = [
                        _strip_none(f.__dict__) for f in self.distributions.correlations.fields
                    ]
                if self.distributions.correlations.matrix is not None:
                    corr_dict["matrix"] = self.distributions.correlations.matrix
                dist_dict["correlations"] = corr_dict
            if self.distributions.conditional is not None:
                dist_dict["conditional"] = [
                    {
                        "dependent_field": c.dependent_field,
                        "condition_field": c.condition_field,
                        **({"breakpoints": [_strip_none(b.__dict__) for b in c.breakpoints]} if c.breakpoints else {}),
                    }
                    for c in self.distributions.conditional
                ]
            if self.distributions.regime_changes is not None:
                regime_dict: Dict[str, Any] = {"enabled": self.distributions.regime_changes.enabled}
                if self.distributions.regime_changes.changes is not None:
                    regime_dict["changes"] = [
                        _strip_none(c.__dict__) for c in self.distributions.regime_changes.changes
                    ]
                if self.distributions.regime_changes.economic_cycle is not None:
                    regime_dict["economic_cycle"] = _strip_none(
                        self.distributions.regime_changes.economic_cycle.__dict__
                    )
                dist_dict["regime_changes"] = regime_dict
            if self.distributions.industry_profile is not None:
                dist_dict["industry_profile"] = self.distributions.industry_profile
            if self.distributions.validation is not None:
                val_dict: Dict[str, Any] = {
                    "enabled": self.distributions.validation.enabled,
                    "fail_on_violation": self.distributions.validation.fail_on_violation,
                }
                if self.distributions.validation.tests is not None:
                    val_dict["tests"] = [_strip_none(t.__dict__) for t in self.distributions.validation.tests]
                if self.distributions.validation.report_path is not None:
                    val_dict["report_path"] = self.distributions.validation.report_path
                dist_dict["validation"] = val_dict
            payload["distributions"] = dist_dict

        if self.templates is not None:
            tmpl_dict: Dict[str, Any] = {}
            if self.templates.names is not None:
                names_dict: Dict[str, Any] = {
                    "email_domain": self.templates.names.email_domain,
                    "generate_realistic_names": self.templates.names.generate_realistic_names,
                }
                if self.templates.names.culture_distribution is not None:
                    names_dict["culture_distribution"] = _strip_none(
                        self.templates.names.culture_distribution.__dict__
                    )
                tmpl_dict["names"] = names_dict
            if self.templates.descriptions is not None:
                tmpl_dict["descriptions"] = _strip_none(self.templates.descriptions.__dict__)
            if self.templates.references is not None:
                tmpl_dict["references"] = _strip_none(self.templates.references.__dict__)
            payload["templates"] = tmpl_dict

        if self.temporal_patterns is not None:
            tp_dict: Dict[str, Any] = {"enabled": self.temporal_patterns.enabled}
            if self.temporal_patterns.business_days is not None:
                bd_dict: Dict[str, Any] = {
                    "enabled": self.temporal_patterns.business_days.enabled,
                    "half_day_policy": self.temporal_patterns.business_days.half_day_policy,
                }
                if self.temporal_patterns.business_days.settlement_rules is not None:
                    bd_dict["settlement_rules"] = _strip_none(
                        self.temporal_patterns.business_days.settlement_rules.__dict__
                    )
                tp_dict["business_days"] = bd_dict
            if self.temporal_patterns.calendars is not None:
                tp_dict["calendars"] = _strip_none(self.temporal_patterns.calendars.__dict__)
            if self.temporal_patterns.period_end is not None:
                pe_dict: Dict[str, Any] = {
                    "enabled": self.temporal_patterns.period_end.enabled,
                    "model": self.temporal_patterns.period_end.model,
                }
                if self.temporal_patterns.period_end.month_end is not None:
                    pe_dict["month_end"] = _strip_none(
                        self.temporal_patterns.period_end.month_end.__dict__
                    )
                if self.temporal_patterns.period_end.quarter_end is not None:
                    pe_dict["quarter_end"] = _strip_none(
                        self.temporal_patterns.period_end.quarter_end.__dict__
                    )
                if self.temporal_patterns.period_end.year_end is not None:
                    pe_dict["year_end"] = _strip_none(
                        self.temporal_patterns.period_end.year_end.__dict__
                    )
                tp_dict["period_end"] = pe_dict
            if self.temporal_patterns.processing_lags is not None:
                pl_dict: Dict[str, Any] = {"enabled": self.temporal_patterns.processing_lags.enabled}
                if self.temporal_patterns.processing_lags.sales_order_lag is not None:
                    pl_dict["sales_order_lag"] = _strip_none(
                        self.temporal_patterns.processing_lags.sales_order_lag.__dict__
                    )
                if self.temporal_patterns.processing_lags.goods_receipt_lag is not None:
                    pl_dict["goods_receipt_lag"] = _strip_none(
                        self.temporal_patterns.processing_lags.goods_receipt_lag.__dict__
                    )
                if self.temporal_patterns.processing_lags.invoice_receipt_lag is not None:
                    pl_dict["invoice_receipt_lag"] = _strip_none(
                        self.temporal_patterns.processing_lags.invoice_receipt_lag.__dict__
                    )
                if self.temporal_patterns.processing_lags.payment_lag is not None:
                    pl_dict["payment_lag"] = _strip_none(
                        self.temporal_patterns.processing_lags.payment_lag.__dict__
                    )
                if self.temporal_patterns.processing_lags.journal_entry_lag is not None:
                    pl_dict["journal_entry_lag"] = _strip_none(
                        self.temporal_patterns.processing_lags.journal_entry_lag.__dict__
                    )
                if self.temporal_patterns.processing_lags.cross_day_posting is not None:
                    tp_dict["cross_day_posting"] = _strip_none(
                        self.temporal_patterns.processing_lags.cross_day_posting.__dict__
                    )
                tp_dict["processing_lags"] = pl_dict
            if self.temporal_patterns.fiscal_calendar is not None:
                fc_dict: Dict[str, Any] = {
                    "enabled": self.temporal_patterns.fiscal_calendar.enabled,
                    "calendar_type": self.temporal_patterns.fiscal_calendar.calendar_type,
                }
                if self.temporal_patterns.fiscal_calendar.year_start_month is not None:
                    fc_dict["year_start_month"] = self.temporal_patterns.fiscal_calendar.year_start_month
                if self.temporal_patterns.fiscal_calendar.year_start_day is not None:
                    fc_dict["year_start_day"] = self.temporal_patterns.fiscal_calendar.year_start_day
                if self.temporal_patterns.fiscal_calendar.four_four_five is not None:
                    fc_dict["four_four_five"] = _strip_none(
                        self.temporal_patterns.fiscal_calendar.four_four_five.__dict__
                    )
                tp_dict["fiscal_calendar"] = fc_dict
            if self.temporal_patterns.intraday is not None:
                intra_dict: Dict[str, Any] = {"enabled": self.temporal_patterns.intraday.enabled}
                if self.temporal_patterns.intraday.segments is not None:
                    intra_dict["segments"] = [
                        _strip_none(s.__dict__) for s in self.temporal_patterns.intraday.segments
                    ]
                tp_dict["intraday"] = intra_dict
            if self.temporal_patterns.timezones is not None:
                tz_dict: Dict[str, Any] = {
                    "enabled": self.temporal_patterns.timezones.enabled,
                    "default_timezone": self.temporal_patterns.timezones.default_timezone,
                    "consolidation_timezone": self.temporal_patterns.timezones.consolidation_timezone,
                }
                if self.temporal_patterns.timezones.entity_mappings is not None:
                    tz_dict["entity_mappings"] = [
                        {"pattern": m.pattern, "timezone": m.timezone}
                        for m in self.temporal_patterns.timezones.entity_mappings
                    ]
                tp_dict["timezones"] = tz_dict
            payload["temporal_patterns"] = tp_dict

        if self.llm is not None:
            payload["llm"] = self.llm

        if self.diffusion is not None:
            payload["diffusion"] = self.diffusion

        if self.causal is not None:
            payload["causal"] = self.causal

        if self.source_to_pay is not None:
            s2p_dict: Dict[str, Any] = {"enabled": self.source_to_pay.enabled}
            if self.source_to_pay.spend_analysis is not None:
                s2p_dict["spend_analysis"] = self.source_to_pay.spend_analysis
            if self.source_to_pay.sourcing is not None:
                s2p_dict["sourcing"] = self.source_to_pay.sourcing
            if self.source_to_pay.qualification is not None:
                s2p_dict["qualification"] = self.source_to_pay.qualification
            if self.source_to_pay.rfx is not None:
                s2p_dict["rfx"] = self.source_to_pay.rfx
            if self.source_to_pay.contracts is not None:
                s2p_dict["contracts"] = self.source_to_pay.contracts
            if self.source_to_pay.catalog is not None:
                s2p_dict["catalog"] = self.source_to_pay.catalog
            if self.source_to_pay.scorecards is not None:
                s2p_dict["scorecards"] = self.source_to_pay.scorecards
            if self.source_to_pay.p2p_integration is not None:
                s2p_dict["p2p_integration"] = self.source_to_pay.p2p_integration
            payload["source_to_pay"] = s2p_dict

        if self.financial_reporting is not None:
            fr_dict: Dict[str, Any] = {
                "enabled": self.financial_reporting.enabled,
                "generate_balance_sheet": self.financial_reporting.generate_balance_sheet,
                "generate_income_statement": self.financial_reporting.generate_income_statement,
                "generate_cash_flow": self.financial_reporting.generate_cash_flow,
                "generate_changes_in_equity": self.financial_reporting.generate_changes_in_equity,
                "comparative_periods": self.financial_reporting.comparative_periods,
            }
            if self.financial_reporting.management_kpis is not None:
                fr_dict["management_kpis"] = _strip_none(
                    self.financial_reporting.management_kpis.__dict__
                )
            if self.financial_reporting.budgets is not None:
                fr_dict["budgets"] = _strip_none(self.financial_reporting.budgets.__dict__)
            payload["financial_reporting"] = fr_dict

        if self.hr is not None:
            hr_dict: Dict[str, Any] = {"enabled": self.hr.enabled}
            if self.hr.payroll is not None:
                hr_dict["payroll"] = _strip_none(self.hr.payroll.__dict__)
            if self.hr.time_attendance is not None:
                hr_dict["time_attendance"] = _strip_none(self.hr.time_attendance.__dict__)
            if self.hr.expenses is not None:
                hr_dict["expenses"] = _strip_none(self.hr.expenses.__dict__)
            payload["hr"] = hr_dict

        if self.manufacturing is not None:
            mfg_dict: Dict[str, Any] = {"enabled": self.manufacturing.enabled}
            if self.manufacturing.production_orders is not None:
                mfg_dict["production_orders"] = _strip_none(
                    self.manufacturing.production_orders.__dict__
                )
            if self.manufacturing.costing is not None:
                mfg_dict["costing"] = _strip_none(self.manufacturing.costing.__dict__)
            if self.manufacturing.routing is not None:
                mfg_dict["routing"] = _strip_none(self.manufacturing.routing.__dict__)
            payload["manufacturing"] = mfg_dict

        if self.sales_quotes is not None:
            payload["sales_quotes"] = _strip_none(self.sales_quotes.__dict__)

        if self.vendor_network is not None:
            vn_dict: Dict[str, Any] = {"enabled": self.vendor_network.enabled, "depth": self.vendor_network.depth}
            if self.vendor_network.tiers is not None:
                vn_dict["tiers"] = self.vendor_network.tiers
            if self.vendor_network.clusters is not None:
                vn_dict["clusters"] = self.vendor_network.clusters
            if self.vendor_network.dependencies is not None:
                vn_dict["dependencies"] = self.vendor_network.dependencies
            payload["vendor_network"] = vn_dict

        if self.customer_segmentation is not None:
            cs_dict: Dict[str, Any] = {"enabled": self.customer_segmentation.enabled}
            if self.customer_segmentation.value_segments is not None:
                cs_dict["value_segments"] = self.customer_segmentation.value_segments
            if self.customer_segmentation.lifecycle is not None:
                cs_dict["lifecycle"] = self.customer_segmentation.lifecycle
            if self.customer_segmentation.networks is not None:
                cs_dict["networks"] = self.customer_segmentation.networks
            payload["customer_segmentation"] = cs_dict

        if self.relationship_strength is not None:
            rs_dict: Dict[str, Any] = {"enabled": self.relationship_strength.enabled}
            if self.relationship_strength.calculation is not None:
                rs_dict["calculation"] = self.relationship_strength.calculation
            if self.relationship_strength.thresholds is not None:
                rs_dict["thresholds"] = self.relationship_strength.thresholds
            payload["relationship_strength"] = rs_dict

        if self.cross_process_links is not None:
            payload["cross_process_links"] = _strip_none(self.cross_process_links.__dict__)

        # Merge extra fields
        payload.update(self.extra)
        return payload

    def to_json(self, **kwargs: Any) -> str:
        import json

        return json.dumps(self.to_dict(), **kwargs)

    def to_yaml(self) -> str:
        yaml_spec = importlib.util.find_spec("yaml")
        if yaml_spec is None:
            raise MissingDependencyError(
                "PyYAML is required for Config.to_yaml(). Install with `pip install PyYAML`."
            )
        import yaml  # type: ignore

        return yaml.safe_dump(self.to_dict(), sort_keys=False)

    def override(self, **overrides: Any) -> "Config":
        current = self.to_dict()
        merged = _deep_merge(current, overrides)
        return Config.from_dict(merged)

    def validate(self) -> None:
        errors = validate_config(self)
        if errors:
            raise ConfigValidationError(errors)

    @staticmethod
    def from_dict(data: Dict[str, Any]) -> "Config":
        global_settings = _build_dataclass(GlobalSettings, data.get("global"))
        companies_data = data.get("companies")
        companies = None
        if companies_data is not None:
            if isinstance(companies_data, list):
                companies = [CompanyConfig(**c) for c in companies_data]
            elif isinstance(companies_data, dict):
                # Handle legacy format where companies was a dict with count
                # Generate default companies based on count
                count = companies_data.get("count", 1)
                industry = companies_data.get("industry", "retail")
                companies = [
                    CompanyConfig(
                        code=f"C{i + 1:03d}",
                        name=f"Company {i + 1}",
                    )
                    for i in range(count)
                ]
                # Set industry in global if not already set
                if global_settings is None:
                    global_settings = GlobalSettings(industry=industry)
                elif global_settings.industry is None:
                    global_settings = GlobalSettings(
                        industry=industry,
                        start_date=global_settings.start_date,
                        period_months=global_settings.period_months,
                        seed=global_settings.seed,
                        group_currency=global_settings.group_currency,
                        parallel=global_settings.parallel,
                        worker_threads=global_settings.worker_threads,
                        memory_limit_mb=global_settings.memory_limit_mb,
                    )

        chart_of_accounts_data = data.get("chart_of_accounts")
        chart_of_accounts = _build_dataclass(ChartOfAccountsSettings, chart_of_accounts_data)
        # Handle legacy format where complexity was in companies
        if chart_of_accounts is None and isinstance(data.get("companies"), dict):
            complexity = data.get("companies", {}).get("complexity")
            if complexity:
                chart_of_accounts = ChartOfAccountsSettings(complexity=complexity)

        transactions = _build_dataclass(TransactionSettings, data.get("transactions"))
        output = _build_dataclass(OutputSettings, data.get("output"))
        fraud = _build_dataclass(FraudSettings, data.get("fraud"))
        banking = _build_dataclass(BankingSettings, data.get("banking"))
        scenario = _build_dataclass(ScenarioSettings, data.get("scenario"))
        temporal = _build_dataclass(TemporalDriftSettings, data.get("temporal"))
        data_quality = _build_dataclass(DataQualitySettings, data.get("data_quality"))
        graph_export = None
        ge_data = data.get("graph_export")
        if ge_data is not None:
            hypergraph = None
            if ge_data.get("hypergraph"):
                hg_data = ge_data["hypergraph"]
                process_layer = _build_dataclass(
                    ProcessLayerSettings, hg_data.get("process_layer")
                )
                hypergraph = HypergraphSettings(
                    enabled=hg_data.get("enabled", False),
                    process_layer=process_layer,
                )
            graph_export = GraphExportSettings(
                enabled=ge_data.get("enabled", False),
                formats=ge_data.get("formats"),
                graph_types=ge_data.get("graph_types"),
                train_ratio=ge_data.get("train_ratio", 0.7),
                validation_ratio=ge_data.get("validation_ratio", 0.15),
                output_subdirectory=ge_data.get("output_subdirectory", "graphs"),
                hypergraph=hypergraph,
            )

        ocpm = None
        ocpm_data = data.get("ocpm")
        if ocpm_data is not None:
            ocpm = OcpmSettings(
                enabled=ocpm_data.get("enabled", False),
                generate_lifecycle_events=ocpm_data.get("generate_lifecycle_events", True),
                include_object_relationships=ocpm_data.get("include_object_relationships", True),
                compute_variants=ocpm_data.get("compute_variants", True),
                max_variants=ocpm_data.get("max_variants", 0),
                p2p_process=_build_dataclass(
                    OcpmProcessSettings, ocpm_data.get("p2p_process")
                ),
                o2c_process=_build_dataclass(
                    OcpmProcessSettings, ocpm_data.get("o2c_process")
                ),
                output=_build_dataclass(OcpmOutputSettings, ocpm_data.get("output")),
            )

        audit = _build_dataclass(AuditSettings, data.get("audit"))
        streaming = _build_dataclass(StreamingSettings, data.get("streaming"))
        rate_limit = _build_dataclass(RateLimitSettings, data.get("rate_limit"))

        # Build temporal_attributes with nested structures
        temporal_attributes = None
        ta_data = data.get("temporal_attributes")
        if ta_data is not None:
            valid_time = _build_dataclass(ValidTimeSettings, ta_data.get("valid_time"))
            transaction_time = _build_dataclass(
                TransactionTimeSettings, ta_data.get("transaction_time")
            )
            temporal_attributes = TemporalAttributeSettings(
                enabled=ta_data.get("enabled", False),
                valid_time=valid_time,
                transaction_time=transaction_time,
                generate_version_chains=ta_data.get("generate_version_chains", False),
                avg_versions_per_entity=ta_data.get("avg_versions_per_entity", 1.5),
            )

        # Build relationships with nested structures
        relationships = None
        rel_data = data.get("relationships")
        if rel_data is not None:
            rel_types = None
            if rel_data.get("relationship_types"):
                rel_types = []
                for rt in rel_data["relationship_types"]:
                    cardinality = None
                    if rt.get("cardinality"):
                        cardinality = CardinalityRule(
                            rule_type=rt["cardinality"].get("rule_type", "one_to_many"),
                            min_count=rt["cardinality"].get("min"),
                            max_count=rt["cardinality"].get("max"),
                        )
                    rel_types.append(
                        RelationshipTypeConfig(
                            name=rt["name"],
                            source_type=rt["source_type"],
                            target_type=rt["target_type"],
                            cardinality=cardinality,
                            weight=rt.get("weight", 1.0),
                        )
                    )
            relationships = RelationshipSettings(
                enabled=rel_data.get("enabled", False),
                relationship_types=rel_types,
                allow_orphans=rel_data.get("allow_orphans", True),
                orphan_probability=rel_data.get("orphan_probability", 0.01),
                allow_circular=rel_data.get("allow_circular", False),
                max_circular_depth=rel_data.get("max_circular_depth", 3),
            )

        # Build accounting_standards with nested structures
        accounting_standards = None
        acct_data = data.get("accounting_standards")
        if acct_data is not None:
            accounting_standards = AccountingStandardsConfig(
                enabled=acct_data.get("enabled", False),
                framework=acct_data.get("framework", "us_gaap"),
                revenue_recognition=_build_dataclass(
                    RevenueRecognitionConfig, acct_data.get("revenue_recognition")
                ),
                leases=_build_dataclass(LeaseAccountingConfig, acct_data.get("leases")),
                fair_value=_build_dataclass(FairValueConfig, acct_data.get("fair_value")),
                impairment=_build_dataclass(ImpairmentConfig, acct_data.get("impairment")),
                generate_differences=acct_data.get("generate_differences", False),
            )

        # Build audit_standards with nested structures
        audit_standards = None
        audit_std_data = data.get("audit_standards")
        if audit_std_data is not None:
            audit_standards = AuditStandardsConfig(
                enabled=audit_std_data.get("enabled", False),
                isa_compliance=_build_dataclass(
                    IsaComplianceConfig, audit_std_data.get("isa_compliance")
                ),
                analytical_procedures=_build_dataclass(
                    AnalyticalProceduresConfig, audit_std_data.get("analytical_procedures")
                ),
                confirmations=_build_dataclass(
                    ConfirmationsConfig, audit_std_data.get("confirmations")
                ),
                opinion=_build_dataclass(AuditOpinionConfig, audit_std_data.get("opinion")),
                generate_audit_trail=audit_std_data.get("generate_audit_trail", False),
                sox=_build_dataclass(SoxComplianceConfig, audit_std_data.get("sox")),
                pcaob=_build_dataclass(PcaobConfig, audit_std_data.get("pcaob")),
            )

        # Build distributions with nested structures
        distributions = None
        dist_data = data.get("distributions")
        if dist_data is not None:
            amounts = None
            if dist_data.get("amounts"):
                amounts_data = dist_data["amounts"]
                components = None
                if amounts_data.get("components"):
                    components = [
                        MixtureComponentConfig(**c) for c in amounts_data["components"]
                    ]
                amounts = MixtureDistributionConfig(
                    enabled=amounts_data.get("enabled", False),
                    distribution_type=amounts_data.get("distribution_type", "lognormal"),
                    components=components,
                    benford_compliance=amounts_data.get("benford_compliance", True),
                )
            correlations = None
            if dist_data.get("correlations"):
                corr_data = dist_data["correlations"]
                fields = None
                if corr_data.get("fields"):
                    fields = [CorrelationFieldConfig(**f) for f in corr_data["fields"]]
                correlations = CorrelationConfig(
                    enabled=corr_data.get("enabled", False),
                    copula_type=corr_data.get("copula_type", "gaussian"),
                    fields=fields,
                    matrix=corr_data.get("matrix"),
                )
            conditional = None
            if dist_data.get("conditional"):
                conditional = []
                for c in dist_data["conditional"]:
                    breakpoints = None
                    if c.get("breakpoints"):
                        breakpoints = [ConditionalBreakpoint(**b) for b in c["breakpoints"]]
                    conditional.append(
                        ConditionalDistributionConfig(
                            dependent_field=c["dependent_field"],
                            condition_field=c["condition_field"],
                            breakpoints=breakpoints,
                        )
                    )
            regime_changes = None
            if dist_data.get("regime_changes"):
                regime_data = dist_data["regime_changes"]
                changes = None
                if regime_data.get("changes"):
                    changes = [RegimeChangeEventConfig(**c) for c in regime_data["changes"]]
                economic_cycle = _build_dataclass(EconomicCycleConfig, regime_data.get("economic_cycle"))
                regime_changes = RegimeChangeConfig(
                    enabled=regime_data.get("enabled", False),
                    changes=changes,
                    economic_cycle=economic_cycle,
                )
            validation = None
            if dist_data.get("validation"):
                val_data = dist_data["validation"]
                tests = None
                if val_data.get("tests"):
                    tests = [StatisticalTestConfig(**t) for t in val_data["tests"]]
                validation = StatisticalValidationConfig(
                    enabled=val_data.get("enabled", False),
                    tests=tests,
                    report_path=val_data.get("report_path"),
                    fail_on_violation=val_data.get("fail_on_violation", False),
                )
            distributions = AdvancedDistributionSettings(
                enabled=dist_data.get("enabled", False),
                amounts=amounts,
                correlations=correlations,
                conditional=conditional,
                regime_changes=regime_changes,
                industry_profile=dist_data.get("industry_profile"),
                validation=validation,
            )

        # Build templates with nested structures
        templates = None
        tmpl_data = data.get("templates")
        if tmpl_data is not None:
            names = None
            if tmpl_data.get("names"):
                names_data = tmpl_data["names"]
                culture_dist = _build_dataclass(
                    CultureDistributionConfig, names_data.get("culture_distribution")
                )
                names = NameTemplateConfig(
                    culture_distribution=culture_dist,
                    email_domain=names_data.get("email_domain", "company.com"),
                    generate_realistic_names=names_data.get("generate_realistic_names", True),
                )
            descriptions = _build_dataclass(DescriptionTemplateConfig, tmpl_data.get("descriptions"))
            references = _build_dataclass(ReferenceTemplateConfig, tmpl_data.get("references"))
            templates = TemplateSettings(
                names=names,
                descriptions=descriptions,
                references=references,
            )

        # Build temporal_patterns with nested structures
        temporal_patterns = None
        tp_data = data.get("temporal_patterns")
        if tp_data is not None:
            business_days = None
            if tp_data.get("business_days"):
                bd_data = tp_data["business_days"]
                settlement_rules = _build_dataclass(
                    SettlementRulesConfig, bd_data.get("settlement_rules")
                )
                business_days = BusinessDaySchemaConfig(
                    enabled=bd_data.get("enabled", False),
                    half_day_policy=bd_data.get("half_day_policy", "full_day"),
                    settlement_rules=settlement_rules,
                )
            calendars = _build_dataclass(CalendarSchemaConfig, tp_data.get("calendars"))
            period_end = None
            if tp_data.get("period_end"):
                pe_data = tp_data["period_end"]
                period_end = PeriodEndSchemaConfig(
                    enabled=pe_data.get("enabled", False),
                    model=pe_data.get("model", "flat"),
                    month_end=_build_dataclass(PeriodEndModelConfig, pe_data.get("month_end")),
                    quarter_end=_build_dataclass(PeriodEndModelConfig, pe_data.get("quarter_end")),
                    year_end=_build_dataclass(PeriodEndModelConfig, pe_data.get("year_end")),
                )
            processing_lags = None
            if tp_data.get("processing_lags"):
                pl_data = tp_data["processing_lags"]
                cross_day = None
                if pl_data.get("cross_day_posting"):
                    cd_data = pl_data["cross_day_posting"]
                    cross_day = CrossDayPostingConfig(
                        enabled=cd_data.get("enabled", False),
                        probability_by_hour=cd_data.get("probability_by_hour"),
                    )
                processing_lags = ProcessingLagSchemaConfig(
                    enabled=pl_data.get("enabled", False),
                    sales_order_lag=_build_dataclass(LagDistributionConfig, pl_data.get("sales_order_lag")),
                    goods_receipt_lag=_build_dataclass(LagDistributionConfig, pl_data.get("goods_receipt_lag")),
                    invoice_receipt_lag=_build_dataclass(LagDistributionConfig, pl_data.get("invoice_receipt_lag")),
                    payment_lag=_build_dataclass(LagDistributionConfig, pl_data.get("payment_lag")),
                    journal_entry_lag=_build_dataclass(LagDistributionConfig, pl_data.get("journal_entry_lag")),
                    cross_day_posting=cross_day,
                )
            fiscal_calendar = None
            if tp_data.get("fiscal_calendar"):
                fc_data = tp_data["fiscal_calendar"]
                four_four_five = _build_dataclass(
                    FourFourFiveSchemaConfig, fc_data.get("four_four_five")
                )
                fiscal_calendar = FiscalCalendarSchemaConfig(
                    enabled=fc_data.get("enabled", False),
                    calendar_type=fc_data.get("calendar_type", "calendar_year"),
                    year_start_month=fc_data.get("year_start_month"),
                    year_start_day=fc_data.get("year_start_day"),
                    four_four_five=four_four_five,
                )
            intraday = None
            if tp_data.get("intraday"):
                intra_data = tp_data["intraday"]
                segments = None
                if intra_data.get("segments"):
                    segments = [IntraDaySegmentConfig(**s) for s in intra_data["segments"]]
                intraday = IntraDaySchemaConfig(
                    enabled=intra_data.get("enabled", False),
                    segments=segments,
                )
            timezones = None
            if tp_data.get("timezones"):
                tz_data = tp_data["timezones"]
                entity_mappings = None
                if tz_data.get("entity_mappings"):
                    entity_mappings = [
                        EntityTimezoneMappingConfig(**m) for m in tz_data["entity_mappings"]
                    ]
                timezones = TimezoneSchemaConfig(
                    enabled=tz_data.get("enabled", False),
                    default_timezone=tz_data.get("default_timezone", "America/New_York"),
                    consolidation_timezone=tz_data.get("consolidation_timezone", "UTC"),
                    entity_mappings=entity_mappings,
                )
            temporal_patterns = TemporalPatternsConfig(
                enabled=tp_data.get("enabled", False),
                business_days=business_days,
                calendars=calendars,
                period_end=period_end,
                processing_lags=processing_lags,
                fiscal_calendar=fiscal_calendar,
                intraday=intraday,
                timezones=timezones,
            )

        # LLM, diffusion, and causal configs are passed through as dicts
        llm = data.get("llm")
        diffusion = data.get("diffusion")
        causal = data.get("causal")

        # Build source_to_pay with nested structures
        source_to_pay = None
        s2p_data = data.get("source_to_pay")
        if s2p_data is not None:
            source_to_pay = SourceToPayConfig(
                enabled=s2p_data.get("enabled", False),
                spend_analysis=s2p_data.get("spend_analysis"),
                sourcing=s2p_data.get("sourcing"),
                qualification=s2p_data.get("qualification"),
                rfx=s2p_data.get("rfx"),
                contracts=s2p_data.get("contracts"),
                catalog=s2p_data.get("catalog"),
                scorecards=s2p_data.get("scorecards"),
                p2p_integration=s2p_data.get("p2p_integration"),
            )

        # Build financial_reporting with nested structures
        financial_reporting = None
        fr_data = data.get("financial_reporting")
        if fr_data is not None:
            financial_reporting = FinancialReportingConfig(
                enabled=fr_data.get("enabled", False),
                generate_balance_sheet=fr_data.get("generate_balance_sheet", True),
                generate_income_statement=fr_data.get("generate_income_statement", True),
                generate_cash_flow=fr_data.get("generate_cash_flow", True),
                generate_changes_in_equity=fr_data.get("generate_changes_in_equity", True),
                comparative_periods=fr_data.get("comparative_periods", 1),
                management_kpis=_build_dataclass(
                    ManagementKpisSchemaConfig, fr_data.get("management_kpis")
                ),
                budgets=_build_dataclass(BudgetSchemaConfig, fr_data.get("budgets")),
            )

        # Build hr with nested structures
        hr = None
        hr_data = data.get("hr")
        if hr_data is not None:
            hr = HrConfig(
                enabled=hr_data.get("enabled", False),
                payroll=_build_dataclass(PayrollSchemaConfig, hr_data.get("payroll")),
                time_attendance=_build_dataclass(
                    TimeAttendanceSchemaConfig, hr_data.get("time_attendance")
                ),
                expenses=_build_dataclass(ExpenseSchemaConfig, hr_data.get("expenses")),
            )

        # Build manufacturing with nested structures
        manufacturing = None
        mfg_data = data.get("manufacturing")
        if mfg_data is not None:
            manufacturing = ManufacturingProcessConfig(
                enabled=mfg_data.get("enabled", False),
                production_orders=_build_dataclass(
                    ProductionOrderSchemaConfig, mfg_data.get("production_orders")
                ),
                costing=_build_dataclass(
                    ManufacturingCostingSchemaConfig, mfg_data.get("costing")
                ),
                routing=_build_dataclass(RoutingSchemaConfig, mfg_data.get("routing")),
            )

        sales_quotes = _build_dataclass(SalesQuoteSchemaConfig, data.get("sales_quotes"))
        vendor_network = None
        vn_data = data.get("vendor_network")
        if vn_data is not None:
            vendor_network = VendorNetworkConfig(
                enabled=vn_data.get("enabled", False),
                depth=vn_data.get("depth", 3),
                tiers=vn_data.get("tiers"),
                clusters=vn_data.get("clusters"),
                dependencies=vn_data.get("dependencies"),
            )
        customer_segmentation = None
        cs_data = data.get("customer_segmentation")
        if cs_data is not None:
            customer_segmentation = CustomerSegmentationConfig(
                enabled=cs_data.get("enabled", False),
                value_segments=cs_data.get("value_segments"),
                lifecycle=cs_data.get("lifecycle"),
                networks=cs_data.get("networks"),
            )
        relationship_strength = None
        rs_data = data.get("relationship_strength")
        if rs_data is not None:
            relationship_strength = RelationshipStrengthConfig(
                enabled=rs_data.get("enabled", False),
                calculation=rs_data.get("calculation"),
                thresholds=rs_data.get("thresholds"),
            )
        cross_process_links = _build_dataclass(
            CrossProcessLinksConfig, data.get("cross_process_links")
        )

        known_keys = {
            "global", "companies", "chart_of_accounts", "transactions", "output",
            "fraud", "banking", "scenario", "temporal", "data_quality", "graph_export",
            "ocpm", "audit", "streaming", "rate_limit", "temporal_attributes",
            "relationships", "accounting_standards", "audit_standards", "distributions",
            "templates", "temporal_patterns", "llm", "diffusion", "causal",
            "source_to_pay", "financial_reporting", "hr", "manufacturing",
            "sales_quotes", "vendor_network", "customer_segmentation",
            "relationship_strength", "cross_process_links",
        }
        extra = {key: value for key, value in data.items() if key not in known_keys}

        return Config(
            global_settings=global_settings,
            companies=companies,
            chart_of_accounts=chart_of_accounts,
            transactions=transactions,
            output=output,
            fraud=fraud,
            banking=banking,
            scenario=scenario,
            temporal=temporal,
            data_quality=data_quality,
            graph_export=graph_export,
            ocpm=ocpm,
            audit=audit,
            streaming=streaming,
            rate_limit=rate_limit,
            temporal_attributes=temporal_attributes,
            relationships=relationships,
            accounting_standards=accounting_standards,
            audit_standards=audit_standards,
            distributions=distributions,
            templates=templates,
            temporal_patterns=temporal_patterns,
            llm=llm,
            diffusion=diffusion,
            causal=causal,
            source_to_pay=source_to_pay,
            financial_reporting=financial_reporting,
            hr=hr,
            manufacturing=manufacturing,
            sales_quotes=sales_quotes,
            vendor_network=vendor_network,
            customer_segmentation=customer_segmentation,
            relationship_strength=relationship_strength,
            cross_process_links=cross_process_links,
            extra=extra,
        )


# Legacy aliases for backwards compatibility
CompanySettings = CompanyConfig


def _strip_none(values: Dict[str, Any]) -> Dict[str, Any]:
    return {key: value for key, value in values.items() if value is not None}


def _deep_merge(base: Dict[str, Any], overrides: Dict[str, Any]) -> Dict[str, Any]:
    merged = dict(base)
    for key, value in overrides.items():
        if isinstance(value, dict) and isinstance(merged.get(key), dict):
            merged[key] = _deep_merge(merged[key], value)
        elif _is_dataclass_instance(value):
            merged[key] = _strip_none(value.__dict__)
        else:
            merged[key] = value
    return merged


def _build_dataclass(cls: Any, payload: Optional[Dict[str, Any]]) -> Optional[Any]:
    if payload is None:
        return None
    # Filter payload to only include fields that exist in the dataclass
    import dataclasses
    valid_fields = {f.name for f in dataclasses.fields(cls)}
    filtered_payload = {k: v for k, v in payload.items() if k in valid_fields}
    return cls(**filtered_payload)


def _is_dataclass_instance(value: Any) -> bool:
    return hasattr(value, "__dataclass_fields__")


class MissingDependencyError(RuntimeError):
    """Raised when an optional dependency is required but unavailable."""
