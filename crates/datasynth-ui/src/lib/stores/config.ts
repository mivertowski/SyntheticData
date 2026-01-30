/**
 * Configuration store with dirty tracking and validation.
 *
 * Manages the application-wide generator configuration state.
 */
import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// Check if we're running in Tauri context with working IPC
function isTauriContext(): boolean {
  if (typeof window === 'undefined') return false;
  // Check for Tauri's IPC mechanism
  return '__TAURI_INTERNALS__' in window && '__TAURI_IPC__' in window;
}

// Wrap invoke with timeout and Tauri context check
async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  // Quick check before attempting invoke
  if (!isTauriContext()) {
    throw new Error('Not running in Tauri context');
  }

  // Add aggressive timeout to prevent hanging
  const timeoutMs = 2000;
  return Promise.race([
    invoke<T>(cmd, args),
    new Promise<T>((_, reject) =>
      setTimeout(() => reject(new Error('Tauri invoke timeout')), timeoutMs)
    )
  ]);
}

// Types matching the backend schema
export interface CompanyConfig {
  code: string;
  name: string;
  currency: string;
  country: string;
  fiscal_year_variant: string;
  annual_transaction_volume: string;
  volume_weight: number;
}

export interface GlobalConfig {
  seed: number | null;
  industry: string;
  start_date: string;
  period_months: number;
  group_currency: string;
  parallel: boolean;
  worker_threads: number;
  memory_limit_mb: number;
}

export interface ChartOfAccountsConfig {
  complexity: string;
  industry_specific: boolean;
  min_hierarchy_depth: number;
  max_hierarchy_depth: number;
}

export interface TransactionConfig {
  line_item_distribution: Record<string, number>;
  amount_distribution: AmountDistribution;
  source_distribution: Record<string, number>;
  seasonality: SeasonalityConfig;
}

export interface AmountDistribution {
  min_amount: number;
  max_amount: number;
  lognormal_mu: number;
  lognormal_sigma: number;
  round_number_probability: number;
  nice_number_probability: number;
  benford_compliance: boolean;
}

export interface SeasonalityConfig {
  month_end_spike: boolean;
  month_end_multiplier: number;
  quarter_end_spike: boolean;
  quarter_end_multiplier: number;
  year_end_spike: boolean;
  year_end_multiplier: number;
  day_of_week_patterns: boolean;
}

export interface FraudTypeDistribution {
  suspense_account_abuse: number;
  fictitious_transaction: number;
  revenue_manipulation: number;
  expense_capitalization: number;
  split_transaction: number;
  timing_anomaly: number;
  unauthorized_access: number;
  duplicate_payment: number;
}

export interface FraudConfig {
  enabled: boolean;
  fraud_rate: number;
  fraud_type_distribution: FraudTypeDistribution;
  clustering_enabled: boolean;
  clustering_factor: number;
  approval_thresholds: number[];
}

export interface InternalControlsConfig {
  enabled: boolean;
  exception_rate: number;
  sod_violation_rate: number;
  export_control_master_data: boolean;
  sox_materiality_threshold: number;
}

export interface CompressionConfig {
  enabled: boolean;
  algorithm: string;
  level: number;
}

export interface OutputConfig {
  mode: string;
  output_directory: string;
  formats: string[];
  compression: CompressionConfig;
  batch_size: number;
  include_acdoca: boolean;
  include_bseg: boolean;
  partition_by_period: boolean;
  partition_by_company: boolean;
}

export interface MasterDataConfig {
  vendors: EntityDistribution;
  customers: EntityDistribution;
  materials: EntityDistribution;
  assets: EntityDistribution;
  employees: EntityDistribution;
}

export interface EntityDistribution {
  count: number;
  distribution: Record<string, number>;
}

export interface DocumentLineCountDistribution {
  min_lines: number;
  max_lines: number;
  mode_lines: number;
}

export interface LatePaymentDaysDistribution {
  slightly_late_1_to_7: number;
  late_8_to_14: number;
  very_late_15_to_30: number;
  severely_late_31_to_60: number;
  extremely_late_over_60: number;
}

export interface P2PPaymentBehaviorConfig {
  late_payment_rate: number;
  late_payment_days_distribution: LatePaymentDaysDistribution;
  partial_payment_rate: number;
  payment_correction_rate: number;
}

export interface P2PFlowConfig {
  enabled: boolean;
  three_way_match_rate: number;
  partial_delivery_rate: number;
  price_variance_rate: number;
  max_price_variance_percent: number;
  quantity_variance_rate: number;
  average_po_to_gr_days: number;
  average_gr_to_invoice_days: number;
  average_invoice_to_payment_days: number;
  line_count_distribution: DocumentLineCountDistribution;
  payment_behavior: P2PPaymentBehaviorConfig;
}

export interface CashDiscountConfig {
  eligible_rate: number;
  taken_rate: number;
  discount_percent: number;
  discount_days: number;
}

export interface DunningPaymentRates {
  after_level_1: number;
  after_level_2: number;
  after_level_3: number;
  during_collection: number;
  never_pay: number;
}

export interface DunningConfig {
  enabled: boolean;
  level_1_days_overdue: number;
  level_2_days_overdue: number;
  level_3_days_overdue: number;
  collection_days_overdue: number;
  payment_after_dunning_rates: DunningPaymentRates;
  dunning_block_rate: number;
  interest_rate_per_year: number;
  dunning_charge: number;
}

export interface PartialPaymentConfig {
  rate: number;
  avg_days_until_remainder: number;
}

export interface ShortPaymentConfig {
  rate: number;
  max_short_percent: number;
}

export interface OnAccountPaymentConfig {
  rate: number;
  avg_days_until_application: number;
}

export interface PaymentCorrectionConfig {
  rate: number;
  avg_resolution_days: number;
}

export interface O2CPaymentBehaviorConfig {
  dunning: DunningConfig;
  partial_payments: PartialPaymentConfig;
  short_payments: ShortPaymentConfig;
  on_account_payments: OnAccountPaymentConfig;
  payment_corrections: PaymentCorrectionConfig;
}

export interface O2CFlowConfig {
  enabled: boolean;
  credit_check_failure_rate: number;
  partial_shipment_rate: number;
  return_rate: number;
  bad_debt_rate: number;
  average_so_to_delivery_days: number;
  average_delivery_to_invoice_days: number;
  average_invoice_to_receipt_days: number;
  line_count_distribution: DocumentLineCountDistribution;
  cash_discount: CashDiscountConfig;
  payment_behavior: O2CPaymentBehaviorConfig;
}

export interface DocumentFlowConfig {
  p2p: P2PFlowConfig;
  o2c: O2CFlowConfig;
  generate_document_references: boolean;
  export_flow_graph: boolean;
}

export interface BalanceConfig {
  generate_opening_balances: boolean;
  generate_trial_balances: boolean;
  target_gross_margin: number;
  target_dso_days: number;
  target_dpo_days: number;
  target_current_ratio: number;
  target_debt_to_equity: number;
  validate_balance_equation: boolean;
  reconcile_subledgers: boolean;
}

export interface BusinessProcessConfig {
  [key: string]: number;
  o2c_weight: number;
  p2p_weight: number;
  r2r_weight: number;
  h2r_weight: number;
  a2r_weight: number;
}

export interface PersonaDistribution {
  [key: string]: number;
  junior_accountant: number;
  senior_accountant: number;
  controller: number;
  manager: number;
  automated_system: number;
}

export interface UsersPerPersona {
  junior_accountant: number;
  senior_accountant: number;
  controller: number;
  manager: number;
  automated_system: number;
}

export interface UserPersonaConfig {
  persona_distribution: PersonaDistribution;
  users_per_persona: UsersPerPersona;
}

export interface CultureDistribution {
  [key: string]: number;
  western_us: number;
  hispanic: number;
  german: number;
  french: number;
  chinese: number;
  japanese: number;
  indian: number;
}

export interface NameTemplateConfig {
  culture_distribution: CultureDistribution;
  email_domain: string;
  generate_realistic_names: boolean;
}

export interface DescriptionTemplateConfig {
  generate_header_text: boolean;
  generate_line_text: boolean;
}

export interface ReferenceTemplateConfig {
  generate_references: boolean;
  invoice_prefix: string;
  po_prefix: string;
  so_prefix: string;
}

export interface TemplateConfig {
  names: NameTemplateConfig;
  descriptions: DescriptionTemplateConfig;
  references: ReferenceTemplateConfig;
}

export interface ApprovalThresholdConfig {
  amount: number;
  level: number;
  roles: string[];
}

export interface ApprovalConfig {
  enabled: boolean;
  auto_approve_threshold: number;
  rejection_rate: number;
  revision_rate: number;
  average_approval_delay_hours: number;
  thresholds: ApprovalThresholdConfig[];
}

export interface CustomDepartmentConfig {
  code: string;
  name: string;
  cost_center: string | null;
  primary_processes: string[];
  parent_code: string | null;
}

export interface DepartmentConfig {
  enabled: boolean;
  headcount_multiplier: number;
  custom_departments: CustomDepartmentConfig[];
}

export interface ICTransactionTypeDistribution {
  [key: string]: number;
  goods_sale: number;
  service_provided: number;
  loan: number;
  dividend: number;
  management_fee: number;
  royalty: number;
  cost_sharing: number;
}

export interface IntercompanyConfig {
  enabled: boolean;
  ic_transaction_rate: number;
  transfer_pricing_method: string;
  markup_percent: number;
  generate_matched_pairs: boolean;
  transaction_type_distribution: ICTransactionTypeDistribution;
  generate_eliminations: boolean;
}

// =============================================================================
// Scenario Configuration
// =============================================================================

export interface ScenarioConfig {
  tags: string[];
  profile: string | null;
  description: string | null;
  ml_training: boolean;
  target_anomaly_ratio: number | null;
  metadata: Record<string, string>;
}

// =============================================================================
// Temporal Drift Configuration
// =============================================================================

export type DriftType = 'gradual' | 'sudden' | 'recurring' | 'mixed';

export interface TemporalDriftConfig {
  enabled: boolean;
  amount_mean_drift: number;
  amount_variance_drift: number;
  anomaly_rate_drift: number;
  concept_drift_rate: number;
  sudden_drift_probability: number;
  sudden_drift_magnitude: number;
  seasonal_drift: boolean;
  drift_start_period: number;
  drift_type: DriftType;
}

// =============================================================================
// OCPM (Object-Centric Process Mining) Configuration
// =============================================================================

export interface OcpmProcessConfig {
  rework_probability: number;
  skip_step_probability: number;
  out_of_order_probability: number;
}

export interface OcpmOutputConfig {
  ocel_json: boolean;
  ocel_xml: boolean;
  flattened_csv: boolean;
  event_object_csv: boolean;
  object_relationship_csv: boolean;
  variants_csv: boolean;
}

export interface OcpmConfig {
  enabled: boolean;
  generate_lifecycle_events: boolean;
  include_object_relationships: boolean;
  compute_variants: boolean;
  max_variants: number;
  p2p_process: OcpmProcessConfig;
  o2c_process: OcpmProcessConfig;
  output: OcpmOutputConfig;
}

// =============================================================================
// Audit Generation Configuration
// =============================================================================

export interface AuditEngagementTypesConfig {
  financial_statement: number;
  sox_icfr: number;
  integrated: number;
  review: number;
  agreed_upon_procedures: number;
}

export interface SamplingConfig {
  statistical_rate: number;
  judgmental_rate: number;
  haphazard_rate: number;
  complete_examination_rate: number;
}

export interface WorkpaperConfig {
  average_per_phase: number;
  include_isa_references: boolean;
  include_sample_details: boolean;
  include_cross_references: boolean;
  sampling: SamplingConfig;
}

export interface AuditTeamConfig {
  min_team_size: number;
  max_team_size: number;
  specialist_probability: number;
}

export interface ReviewWorkflowConfig {
  average_review_delay_days: number;
  rework_probability: number;
  require_partner_signoff: boolean;
}

export interface AuditGenerationConfig {
  enabled: boolean;
  generate_workpapers: boolean;
  engagement_types: AuditEngagementTypesConfig;
  workpapers: WorkpaperConfig;
  team: AuditTeamConfig;
  review: ReviewWorkflowConfig;
}

// =============================================================================
// Banking/KYC/AML Configuration
// =============================================================================

export type RiskAppetite = 'low' | 'medium' | 'high';

export interface BankingPopulationConfig {
  retail_customers: number;
  retail_persona_weights: Record<string, number>;
  business_customers: number;
  business_persona_weights: Record<string, number>;
  trusts: number;
  household_rate: number;
  avg_household_size: number;
  period_months: number;
  start_date: string;
}

export interface BankingProductConfig {
  cash_intensity: number;
  cross_border_rate: number;
  card_vs_transfer: number;
  avg_accounts_retail: number;
  avg_accounts_business: number;
  debit_card_rate: number;
  international_rate: number;
}

export interface BankingComplianceConfig {
  risk_appetite: RiskAppetite;
  kyc_completeness: number;
  high_risk_tolerance: number;
  pep_rate: number;
  edd_threshold: number;
}

export interface SophisticationDistribution {
  basic: number;
  standard: number;
  professional: number;
  advanced: number;
}

export interface BankingTypologyConfig {
  suspicious_rate: number;
  structuring_rate: number;
  funnel_rate: number;
  layering_rate: number;
  mule_rate: number;
  fraud_rate: number;
  sophistication: SophisticationDistribution;
  detectability: number;
  round_tripping_rate: number;
  trade_based_rate: number;
}

export interface BankingSpoofingConfig {
  enabled: boolean;
  intensity: number;
  spoof_timing: boolean;
  spoof_amounts: boolean;
  spoof_merchants: boolean;
  spoof_geography: boolean;
  add_delays: boolean;
}

export interface BankingOutputConfigSection {
  directory: string;
  include_customers: boolean;
  include_accounts: boolean;
  include_transactions: boolean;
  include_counterparties: boolean;
  include_beneficial_ownership: boolean;
  include_transaction_labels: boolean;
  include_entity_labels: boolean;
  include_relationship_labels: boolean;
  include_case_narratives: boolean;
  include_graph: boolean;
}

export interface BankingConfig {
  enabled: boolean;
  population: BankingPopulationConfig;
  products: BankingProductConfig;
  compliance: BankingComplianceConfig;
  typologies: BankingTypologyConfig;
  spoofing: BankingSpoofingConfig;
  output: BankingOutputConfigSection;
}

// =============================================================================
// Fingerprint Configuration (UI-specific)
// =============================================================================

export type PrivacyLevel = 'minimal' | 'standard' | 'high' | 'maximum';

export interface FingerprintConfig {
  enabled: boolean;
  privacy_level: PrivacyLevel;
  streaming: boolean;
  scale: number;
  preserve_correlations: boolean;
  input_path: string;
  output_path: string;
}

// =============================================================================
// Advanced Distributions Configuration
// =============================================================================

export interface MixtureComponentConfig {
  weight: number;
  mu: number;
  sigma: number;
  label: string | null;
}

export interface MixtureDistributionConfig {
  enabled: boolean;
  distribution_type: string;
  components: MixtureComponentConfig[];
  benford_compliance: boolean;
}

export interface CorrelationFieldConfig {
  name: string;
  distribution_type: string;
  min_value: number | null;
  max_value: number | null;
}

export type CopulaType = 'gaussian' | 'clayton' | 'gumbel' | 'frank' | 'student_t';

export interface CorrelationConfig {
  enabled: boolean;
  copula_type: CopulaType;
  fields: CorrelationFieldConfig[];
  matrix: number[][];
}

export interface RegimeChangeEventConfig {
  date: string;
  change_type: string;
  description: string | null;
  volume_multiplier: number;
  amount_mean_shift: number;
  amount_variance_shift: number;
}

export interface EconomicCycleConfig {
  enabled: boolean;
  cycle_period_months: number;
  amplitude: number;
  recession_probability: number;
  recession_depth: number;
}

export interface RegimeChangeConfig {
  enabled: boolean;
  changes: RegimeChangeEventConfig[];
  economic_cycle: EconomicCycleConfig;
}

export interface StatisticalTestConfig {
  test_type: string;
  significance: number;
  threshold_mad: number | null;
  target_distribution: string | null;
}

export interface StatisticalValidationConfig {
  enabled: boolean;
  tests: StatisticalTestConfig[];
  fail_on_violation: boolean;
}

export type IndustryProfileType = 'retail' | 'manufacturing' | 'financial_services' | 'healthcare' | 'technology';

export interface AdvancedDistributionConfig {
  enabled: boolean;
  amounts: MixtureDistributionConfig;
  correlations: CorrelationConfig;
  regime_changes: RegimeChangeConfig;
  industry_profile: IndustryProfileType | null;
  validation: StatisticalValidationConfig;
}

// =============================================================================
// Temporal Patterns Configuration (Business Days, Period-End, Processing Lags)
// =============================================================================

export interface SettlementRulesConfig {
  equity_days: number;
  government_bonds_days: number;
  fx_spot_days: number;
  corporate_bonds_days: number;
  wire_cutoff_time: string;
  wire_international_days: number;
  ach_days: number;
}

export interface BusinessDayConfig {
  enabled: boolean;
  half_day_policy: string;
  settlement_rules: SettlementRulesConfig;
  month_end_convention: string;
  weekend_days: string[] | null;
}

export interface CustomHolidayConfig {
  name: string;
  month: number;
  day: number;
  activity_multiplier: number;
}

export interface CalendarConfig {
  regions: string[];
  custom_holidays: CustomHolidayConfig[];
}

export interface PeriodEndModelConfig {
  inherit_from: string | null;
  additional_multiplier: number | null;
  start_day: number | null;
  base_multiplier: number | null;
  peak_multiplier: number | null;
  decay_rate: number | null;
  sustained_high_days: number | null;
}

export interface PeriodEndConfig {
  model: string | null;
  month_end: PeriodEndModelConfig | null;
  quarter_end: PeriodEndModelConfig | null;
  year_end: PeriodEndModelConfig | null;
}

export interface LagDistributionConfig {
  mu: number;
  sigma: number;
  min_hours: number | null;
  max_hours: number | null;
}

export interface CrossDayPostingConfig {
  enabled: boolean;
  probability_by_hour: Record<number, number>;
}

export interface ProcessingLagConfig {
  enabled: boolean;
  sales_order_lag: LagDistributionConfig | null;
  purchase_order_lag: LagDistributionConfig | null;
  goods_receipt_lag: LagDistributionConfig | null;
  invoice_receipt_lag: LagDistributionConfig | null;
  invoice_issue_lag: LagDistributionConfig | null;
  payment_lag: LagDistributionConfig | null;
  journal_entry_lag: LagDistributionConfig | null;
  cross_day_posting: CrossDayPostingConfig | null;
}

export interface FourFourFiveConfig {
  pattern: string;
  anchor_type: string;
  anchor_month: number;
  leap_week_placement: string;
}

export interface FiscalCalendarConfig {
  enabled: boolean;
  calendar_type: string;
  year_start_month: number | null;
  year_start_day: number | null;
  four_four_five: FourFourFiveConfig | null;
}

export interface IntraDaySegmentConfig {
  name: string;
  start: string;
  end: string;
  multiplier: number;
  posting_type: string;
}

export interface IntraDayConfig {
  enabled: boolean;
  segments: IntraDaySegmentConfig[];
}

export interface EntityTimezoneMapping {
  pattern: string;
  timezone: string;
}

export interface TimezoneConfig {
  enabled: boolean;
  default_timezone: string;
  consolidation_timezone: string;
  entity_mappings: EntityTimezoneMapping[];
}

export interface TemporalPatternsConfig {
  enabled: boolean;
  business_days: BusinessDayConfig;
  calendars: CalendarConfig;
  period_end: PeriodEndConfig;
  processing_lags: ProcessingLagConfig;
  fiscal_calendar: FiscalCalendarConfig;
  intraday: IntraDayConfig;
  timezones: TimezoneConfig;
}

// Full generator config
export interface GeneratorConfig {
  global: GlobalConfig;
  companies: CompanyConfig[];
  chart_of_accounts: ChartOfAccountsConfig;
  transactions: TransactionConfig;
  output: OutputConfig;
  fraud: FraudConfig;
  internal_controls: InternalControlsConfig;
  business_processes: BusinessProcessConfig;
  user_personas: UserPersonaConfig;
  templates: TemplateConfig;
  approval: ApprovalConfig;
  departments: DepartmentConfig;
  master_data: MasterDataConfig;
  document_flows: DocumentFlowConfig;
  intercompany: IntercompanyConfig;
  balance: BalanceConfig;
  // New feature areas
  scenario: ScenarioConfig;
  temporal: TemporalDriftConfig;
  ocpm: OcpmConfig;
  audit: AuditGenerationConfig;
  banking: BankingConfig;
  fingerprint: FingerprintConfig;
  distributions: AdvancedDistributionConfig;
  temporal_patterns: TemporalPatternsConfig;
}

// Default configuration
export function createDefaultConfig(): GeneratorConfig {
  return {
    global: {
      seed: null,
      industry: 'manufacturing',
      start_date: '2024-01-01',
      period_months: 12,
      group_currency: 'USD',
      parallel: true,
      worker_threads: 0,
      memory_limit_mb: 0,
    },
    companies: [{
      code: '1000',
      name: 'US Manufacturing',
      currency: 'USD',
      country: 'US',
      fiscal_year_variant: 'K4',
      annual_transaction_volume: 'hundred_k',
      volume_weight: 1.0,
    }],
    chart_of_accounts: {
      complexity: 'medium',
      industry_specific: true,
      min_hierarchy_depth: 2,
      max_hierarchy_depth: 5,
    },
    transactions: {
      line_item_distribution: {
        '2': 0.61,
        '3': 0.06,
        '4': 0.17,
        '5': 0.03,
        '6': 0.03,
        '7-9': 0.04,
        '10-99': 0.06,
      },
      amount_distribution: {
        min_amount: 0.01,
        max_amount: 100000000,
        lognormal_mu: 7.0,
        lognormal_sigma: 2.5,
        round_number_probability: 0.25,
        nice_number_probability: 0.15,
        benford_compliance: true,
      },
      source_distribution: {
        manual: 0.1,
        interface: 0.3,
        batch: 0.4,
        recurring: 0.2,
      },
      seasonality: {
        month_end_spike: true,
        month_end_multiplier: 2.5,
        quarter_end_spike: true,
        quarter_end_multiplier: 4.0,
        year_end_spike: true,
        year_end_multiplier: 6.0,
        day_of_week_patterns: true,
      },
    },
    output: {
      mode: 'flat_file',
      output_directory: './output',
      formats: ['csv'],
      compression: {
        enabled: true,
        algorithm: 'gzip',
        level: 6,
      },
      batch_size: 100000,
      include_acdoca: true,
      include_bseg: false,
      partition_by_period: true,
      partition_by_company: false,
    },
    fraud: {
      enabled: false,
      fraud_rate: 0.005,
      fraud_type_distribution: {
        suspense_account_abuse: 0.25,
        fictitious_transaction: 0.15,
        revenue_manipulation: 0.10,
        expense_capitalization: 0.10,
        split_transaction: 0.15,
        timing_anomaly: 0.10,
        unauthorized_access: 0.10,
        duplicate_payment: 0.05,
      },
      clustering_enabled: false,
      clustering_factor: 3.0,
      approval_thresholds: [1000, 5000, 10000, 25000, 50000, 100000],
    },
    internal_controls: {
      enabled: false,
      exception_rate: 0.02,
      sod_violation_rate: 0.01,
      export_control_master_data: true,
      sox_materiality_threshold: 10000,
    },
    master_data: {
      vendors: { count: 100, distribution: {} },
      customers: { count: 100, distribution: {} },
      materials: { count: 200, distribution: {} },
      assets: { count: 50, distribution: {} },
      employees: { count: 20, distribution: {} },
    },
    document_flows: {
      p2p: {
        enabled: true,
        three_way_match_rate: 0.95,
        partial_delivery_rate: 0.15,
        price_variance_rate: 0.08,
        max_price_variance_percent: 0.05,
        quantity_variance_rate: 0.05,
        average_po_to_gr_days: 14,
        average_gr_to_invoice_days: 5,
        average_invoice_to_payment_days: 30,
        line_count_distribution: {
          min_lines: 1,
          max_lines: 20,
          mode_lines: 3,
        },
        payment_behavior: {
          late_payment_rate: 0.15,
          late_payment_days_distribution: {
            slightly_late_1_to_7: 0.50,
            late_8_to_14: 0.25,
            very_late_15_to_30: 0.15,
            severely_late_31_to_60: 0.07,
            extremely_late_over_60: 0.03,
          },
          partial_payment_rate: 0.05,
          payment_correction_rate: 0.02,
        },
      },
      o2c: {
        enabled: true,
        credit_check_failure_rate: 0.02,
        partial_shipment_rate: 0.10,
        return_rate: 0.03,
        bad_debt_rate: 0.01,
        average_so_to_delivery_days: 7,
        average_delivery_to_invoice_days: 1,
        average_invoice_to_receipt_days: 45,
        line_count_distribution: {
          min_lines: 1,
          max_lines: 20,
          mode_lines: 3,
        },
        cash_discount: {
          eligible_rate: 0.30,
          taken_rate: 0.60,
          discount_percent: 0.02,
          discount_days: 10,
        },
        payment_behavior: {
          dunning: {
            enabled: true,
            level_1_days_overdue: 14,
            level_2_days_overdue: 28,
            level_3_days_overdue: 42,
            collection_days_overdue: 60,
            payment_after_dunning_rates: {
              after_level_1: 0.40,
              after_level_2: 0.30,
              after_level_3: 0.15,
              during_collection: 0.05,
              never_pay: 0.10,
            },
            dunning_block_rate: 0.05,
            interest_rate_per_year: 0.08,
            dunning_charge: 15.0,
          },
          partial_payments: {
            rate: 0.08,
            avg_days_until_remainder: 30,
          },
          short_payments: {
            rate: 0.03,
            max_short_percent: 0.10,
          },
          on_account_payments: {
            rate: 0.02,
            avg_days_until_application: 14,
          },
          payment_corrections: {
            rate: 0.02,
            avg_resolution_days: 7,
          },
        },
      },
      generate_document_references: true,
      export_flow_graph: false,
    },
    business_processes: {
      o2c_weight: 0.35,
      p2p_weight: 0.30,
      r2r_weight: 0.20,
      h2r_weight: 0.10,
      a2r_weight: 0.05,
    },
    user_personas: {
      persona_distribution: {
        junior_accountant: 0.15,
        senior_accountant: 0.15,
        controller: 0.05,
        manager: 0.05,
        automated_system: 0.60,
      },
      users_per_persona: {
        junior_accountant: 10,
        senior_accountant: 5,
        controller: 2,
        manager: 3,
        automated_system: 20,
      },
    },
    templates: {
      names: {
        culture_distribution: {
          western_us: 0.40,
          hispanic: 0.20,
          german: 0.10,
          french: 0.05,
          chinese: 0.10,
          japanese: 0.05,
          indian: 0.10,
        },
        email_domain: 'company.com',
        generate_realistic_names: true,
      },
      descriptions: {
        generate_header_text: true,
        generate_line_text: true,
      },
      references: {
        generate_references: true,
        invoice_prefix: 'INV',
        po_prefix: 'PO',
        so_prefix: 'SO',
      },
    },
    approval: {
      enabled: false,
      auto_approve_threshold: 1000,
      rejection_rate: 0.02,
      revision_rate: 0.05,
      average_approval_delay_hours: 4.0,
      thresholds: [
        { amount: 1000, level: 1, roles: ['senior_accountant'] },
        { amount: 10000, level: 2, roles: ['senior_accountant', 'controller'] },
        { amount: 100000, level: 3, roles: ['senior_accountant', 'controller', 'manager'] },
        { amount: 500000, level: 4, roles: ['senior_accountant', 'controller', 'manager', 'executive'] },
      ],
    },
    departments: {
      enabled: false,
      headcount_multiplier: 1.0,
      custom_departments: [],
    },
    intercompany: {
      enabled: false,
      ic_transaction_rate: 0.15,
      transfer_pricing_method: 'cost_plus',
      markup_percent: 0.05,
      generate_matched_pairs: true,
      transaction_type_distribution: {
        goods_sale: 0.35,
        service_provided: 0.20,
        loan: 0.10,
        dividend: 0.05,
        management_fee: 0.15,
        royalty: 0.10,
        cost_sharing: 0.05,
      },
      generate_eliminations: false,
    },
    balance: {
      generate_opening_balances: false,
      generate_trial_balances: true,
      target_gross_margin: 0.35,
      target_dso_days: 45,
      target_dpo_days: 30,
      target_current_ratio: 1.5,
      target_debt_to_equity: 0.5,
      validate_balance_equation: true,
      reconcile_subledgers: true,
    },
    // New feature areas
    scenario: {
      tags: [],
      profile: null,
      description: null,
      ml_training: false,
      target_anomaly_ratio: null,
      metadata: {},
    },
    temporal: {
      enabled: false,
      amount_mean_drift: 0.02,
      amount_variance_drift: 0.0,
      anomaly_rate_drift: 0.0,
      concept_drift_rate: 0.01,
      sudden_drift_probability: 0.0,
      sudden_drift_magnitude: 2.0,
      seasonal_drift: false,
      drift_start_period: 0,
      drift_type: 'gradual',
    },
    ocpm: {
      enabled: false,
      generate_lifecycle_events: true,
      include_object_relationships: true,
      compute_variants: true,
      max_variants: 0,
      p2p_process: {
        rework_probability: 0.05,
        skip_step_probability: 0.02,
        out_of_order_probability: 0.03,
      },
      o2c_process: {
        rework_probability: 0.05,
        skip_step_probability: 0.02,
        out_of_order_probability: 0.03,
      },
      output: {
        ocel_json: true,
        ocel_xml: false,
        flattened_csv: true,
        event_object_csv: true,
        object_relationship_csv: true,
        variants_csv: true,
      },
    },
    audit: {
      enabled: false,
      generate_workpapers: true,
      engagement_types: {
        financial_statement: 0.40,
        sox_icfr: 0.20,
        integrated: 0.25,
        review: 0.10,
        agreed_upon_procedures: 0.05,
      },
      workpapers: {
        average_per_phase: 5,
        include_isa_references: true,
        include_sample_details: true,
        include_cross_references: true,
        sampling: {
          statistical_rate: 0.40,
          judgmental_rate: 0.30,
          haphazard_rate: 0.20,
          complete_examination_rate: 0.10,
        },
      },
      team: {
        min_team_size: 3,
        max_team_size: 8,
        specialist_probability: 0.30,
      },
      review: {
        average_review_delay_days: 2,
        rework_probability: 0.15,
        require_partner_signoff: true,
      },
    },
    banking: {
      enabled: false,
      population: {
        retail_customers: 10000,
        retail_persona_weights: {
          student: 0.15,
          early_career: 0.25,
          mid_career: 0.30,
          retiree: 0.15,
          high_net_worth: 0.05,
          gig_worker: 0.10,
        },
        business_customers: 1000,
        business_persona_weights: {
          small_business: 0.50,
          mid_market: 0.25,
          enterprise: 0.05,
          cash_intensive: 0.10,
          import_export: 0.05,
          professional_services: 0.05,
        },
        trusts: 100,
        household_rate: 0.4,
        avg_household_size: 2.3,
        period_months: 12,
        start_date: '2024-01-01',
      },
      products: {
        cash_intensity: 0.15,
        cross_border_rate: 0.05,
        card_vs_transfer: 0.6,
        avg_accounts_retail: 1.5,
        avg_accounts_business: 2.5,
        debit_card_rate: 0.85,
        international_rate: 0.10,
      },
      compliance: {
        risk_appetite: 'medium',
        kyc_completeness: 0.95,
        high_risk_tolerance: 0.05,
        pep_rate: 0.01,
        edd_threshold: 50000,
      },
      typologies: {
        suspicious_rate: 0.02,
        structuring_rate: 0.004,
        funnel_rate: 0.003,
        layering_rate: 0.003,
        mule_rate: 0.005,
        fraud_rate: 0.005,
        sophistication: {
          basic: 0.4,
          standard: 0.35,
          professional: 0.2,
          advanced: 0.05,
        },
        detectability: 0.5,
        round_tripping_rate: 0.001,
        trade_based_rate: 0.001,
      },
      spoofing: {
        enabled: true,
        intensity: 0.3,
        spoof_timing: true,
        spoof_amounts: true,
        spoof_merchants: true,
        spoof_geography: false,
        add_delays: true,
      },
      output: {
        directory: 'banking',
        include_customers: true,
        include_accounts: true,
        include_transactions: true,
        include_counterparties: true,
        include_beneficial_ownership: true,
        include_transaction_labels: true,
        include_entity_labels: true,
        include_relationship_labels: true,
        include_case_narratives: true,
        include_graph: true,
      },
    },
    fingerprint: {
      enabled: false,
      privacy_level: 'standard',
      streaming: false,
      scale: 1.0,
      preserve_correlations: true,
      input_path: '',
      output_path: '',
    },
    distributions: {
      enabled: false,
      amounts: {
        enabled: false,
        distribution_type: 'lognormal',
        components: [
          { weight: 0.60, mu: 6.0, sigma: 1.5, label: 'routine' },
          { weight: 0.30, mu: 8.5, sigma: 1.0, label: 'significant' },
          { weight: 0.10, mu: 11.0, sigma: 0.8, label: 'major' },
        ],
        benford_compliance: true,
      },
      correlations: {
        enabled: false,
        copula_type: 'gaussian',
        fields: [
          { name: 'amount', distribution_type: 'lognormal', min_value: null, max_value: null },
          { name: 'line_items', distribution_type: 'normal', min_value: 1, max_value: 20 },
        ],
        matrix: [
          [1.0, 0.65],
          [0.65, 1.0],
        ],
      },
      regime_changes: {
        enabled: false,
        changes: [],
        economic_cycle: {
          enabled: false,
          cycle_period_months: 48,
          amplitude: 0.15,
          recession_probability: 0.1,
          recession_depth: 0.25,
        },
      },
      industry_profile: null,
      validation: {
        enabled: false,
        tests: [
          { test_type: 'benford_first_digit', significance: 0.05, threshold_mad: 0.015, target_distribution: null },
          { test_type: 'distribution_fit', significance: 0.05, threshold_mad: null, target_distribution: 'lognormal' },
        ],
        fail_on_violation: false,
      },
    },
    temporal_patterns: {
      enabled: false,
      business_days: {
        enabled: true,
        half_day_policy: 'half_day',
        settlement_rules: {
          equity_days: 2,
          government_bonds_days: 1,
          fx_spot_days: 2,
          corporate_bonds_days: 2,
          wire_cutoff_time: '14:00',
          wire_international_days: 1,
          ach_days: 1,
        },
        month_end_convention: 'modified_following',
        weekend_days: null,
      },
      calendars: {
        regions: ['US'],
        custom_holidays: [],
      },
      period_end: {
        model: null,
        month_end: null,
        quarter_end: null,
        year_end: null,
      },
      processing_lags: {
        enabled: true,
        sales_order_lag: null,
        purchase_order_lag: null,
        goods_receipt_lag: null,
        invoice_receipt_lag: null,
        invoice_issue_lag: null,
        payment_lag: null,
        journal_entry_lag: null,
        cross_day_posting: null,
      },
      fiscal_calendar: {
        enabled: false,
        calendar_type: 'calendar_year',
        year_start_month: null,
        year_start_day: null,
        four_four_five: null,
      },
      intraday: {
        enabled: false,
        segments: [],
      },
      timezones: {
        enabled: false,
        default_timezone: 'America/New_York',
        consolidation_timezone: 'UTC',
        entity_mappings: [],
      },
    },
  };
}

// Validation errors
export interface ValidationError {
  field: string;
  message: string;
}

// Store state
function createConfigStore() {
  // Initialize with default config immediately for browser-only mode
  const defaultCfg = createDefaultConfig();

  // The current configuration being edited
  const config = writable<GeneratorConfig | null>(defaultCfg);

  // The original (saved) configuration for dirty tracking
  const originalConfig = writable<GeneratorConfig | null>(JSON.parse(JSON.stringify(defaultCfg)));

  // Loading and saving states
  const loading = writable(false);
  const saving = writable(false);
  const error = writable<string | null>(null);

  // Derived: is the config dirty (has unsaved changes)?
  const isDirty = derived(
    [config, originalConfig],
    ([$config, $originalConfig]) => {
      if (!$config || !$originalConfig) return false;
      return JSON.stringify($config) !== JSON.stringify($originalConfig);
    }
  );

  // Derived: validation errors
  const validationErrors = derived(config, ($config) => {
    if (!$config) return [];
    return validateConfig($config);
  });

  // Derived: is the config valid?
  const isValid = derived(validationErrors, ($errors) => $errors.length === 0);

  // Load configuration from backend (or keep default in browser mode)
  async function load(): Promise<void> {
    loading.set(true);
    error.set(null);

    try {
      const response = await safeInvoke<{ success: boolean; config: GeneratorConfig | null; message: string }>('get_config');
      if (response.success && response.config) {
        config.set(response.config);
        originalConfig.set(JSON.parse(JSON.stringify(response.config)));
      }
      // If backend returns no config, keep the default that was already set
    } catch (e) {
      // In browser mode, keep the default config that was already initialized
      // Only set error if it's not a context/timeout issue (those are expected in browser)
      const errorMsg = String(e);
      if (!errorMsg.includes('Tauri context') && !errorMsg.includes('timeout')) {
        error.set(errorMsg);
      }
    } finally {
      loading.set(false);
    }
  }

  // Save configuration to backend
  async function save(): Promise<boolean> {
    const currentConfig = get(config);
    if (!currentConfig) return false;

    // Validate first
    const errors = validateConfig(currentConfig);
    if (errors.length > 0) {
      error.set(errors.map(e => e.message).join('; '));
      return false;
    }

    saving.set(true);
    error.set(null);

    try {
      const response = await safeInvoke<{ success: boolean; message: string }>('set_config', { config: currentConfig });
      if (response.success) {
        originalConfig.set(JSON.parse(JSON.stringify(currentConfig)));
        return true;
      } else {
        error.set(response.message);
        return false;
      }
    } catch (e) {
      const errorMsg = String(e);
      // In browser mode, simulate successful save (config is stored in memory)
      if (errorMsg.includes('Tauri context') || errorMsg.includes('timeout')) {
        originalConfig.set(JSON.parse(JSON.stringify(currentConfig)));
        return true;
      }
      error.set(errorMsg);
      return false;
    } finally {
      saving.set(false);
    }
  }

  // Reset to original (discard changes)
  function reset(): void {
    const original = get(originalConfig);
    if (original) {
      config.set(JSON.parse(JSON.stringify(original)));
    }
    error.set(null);
  }

  // Apply a preset configuration
  function applyPreset(preset: GeneratorConfig): void {
    config.set(JSON.parse(JSON.stringify(preset)));
  }

  // Update a specific field
  function updateField<K extends keyof GeneratorConfig>(section: K, value: GeneratorConfig[K]): void {
    config.update(cfg => {
      if (!cfg) return cfg;
      return { ...cfg, [section]: value };
    });
  }

  return {
    // Readable stores
    config: { subscribe: config.subscribe },
    loading: { subscribe: loading.subscribe },
    saving: { subscribe: saving.subscribe },
    error: { subscribe: error.subscribe },
    isDirty: { subscribe: isDirty.subscribe },
    validationErrors: { subscribe: validationErrors.subscribe },
    isValid: { subscribe: isValid.subscribe },

    // Actions
    load,
    save,
    reset,
    applyPreset,
    updateField,

    // Direct config update (for form bindings)
    set: config.set,
    update: config.update,
  };
}

// Validate configuration
function validateConfig(config: GeneratorConfig): ValidationError[] {
  const errors: ValidationError[] = [];

  // Global settings validation
  if (!config.global.start_date.match(/^\d{4}-\d{2}-\d{2}$/)) {
    errors.push({ field: 'global.start_date', message: 'Start date must be in YYYY-MM-DD format' });
  }

  if (config.global.period_months < 1 || config.global.period_months > 120) {
    errors.push({ field: 'global.period_months', message: 'Period must be between 1 and 120 months' });
  }

  if (config.global.memory_limit_mb < 0) {
    errors.push({ field: 'global.memory_limit_mb', message: 'Memory limit cannot be negative' });
  }

  // Company validation
  if (config.companies.length === 0) {
    errors.push({ field: 'companies', message: 'At least one company is required' });
  }

  config.companies.forEach((company, i) => {
    if (!company.code) {
      errors.push({ field: `companies[${i}].code`, message: `Company ${i + 1}: Code is required` });
    }
    if (!company.name) {
      errors.push({ field: `companies[${i}].name`, message: `Company ${i + 1}: Name is required` });
    }
    if (company.volume_weight <= 0) {
      errors.push({ field: `companies[${i}].volume_weight`, message: `Company ${i + 1}: Volume weight must be positive` });
    }
  });

  // Chart of accounts validation
  if (config.chart_of_accounts.min_hierarchy_depth < 1) {
    errors.push({ field: 'chart_of_accounts.min_hierarchy_depth', message: 'Minimum hierarchy depth must be at least 1' });
  }

  if (config.chart_of_accounts.max_hierarchy_depth < config.chart_of_accounts.min_hierarchy_depth) {
    errors.push({ field: 'chart_of_accounts.max_hierarchy_depth', message: 'Maximum hierarchy depth must be >= minimum' });
  }

  // Transaction settings validation
  if (config.transactions.amount_distribution.min_amount < 0) {
    errors.push({ field: 'transactions.amount_distribution.min_amount', message: 'Minimum amount cannot be negative' });
  }

  if (config.transactions.amount_distribution.max_amount <= config.transactions.amount_distribution.min_amount) {
    errors.push({ field: 'transactions.amount_distribution.max_amount', message: 'Maximum amount must be greater than minimum' });
  }

  // Fraud validation
  if (config.fraud.enabled && (config.fraud.fraud_rate < 0 || config.fraud.fraud_rate > 0.1)) {
    errors.push({ field: 'fraud.fraud_rate', message: 'Fraud rate must be between 0 and 10%' });
  }

  // Internal controls validation
  if (config.internal_controls.enabled) {
    if (config.internal_controls.exception_rate < 0 || config.internal_controls.exception_rate > 0.1) {
      errors.push({ field: 'internal_controls.exception_rate', message: 'Exception rate must be between 0 and 10%' });
    }
    if (config.internal_controls.sod_violation_rate < 0 || config.internal_controls.sod_violation_rate > 0.1) {
      errors.push({ field: 'internal_controls.sod_violation_rate', message: 'SoD violation rate must be between 0 and 10%' });
    }
  }

  // Temporal drift validation
  if (config.temporal?.enabled) {
    if (config.temporal.amount_mean_drift < -1 || config.temporal.amount_mean_drift > 1) {
      errors.push({ field: 'temporal.amount_mean_drift', message: 'Amount mean drift must be between -100% and 100%' });
    }
    if (config.temporal.concept_drift_rate < 0 || config.temporal.concept_drift_rate > 1) {
      errors.push({ field: 'temporal.concept_drift_rate', message: 'Concept drift rate must be between 0 and 1' });
    }
    if (config.temporal.sudden_drift_probability < 0 || config.temporal.sudden_drift_probability > 1) {
      errors.push({ field: 'temporal.sudden_drift_probability', message: 'Sudden drift probability must be between 0 and 1' });
    }
  }

  // OCPM validation
  if (config.ocpm?.enabled) {
    if (config.ocpm.p2p_process.rework_probability < 0 || config.ocpm.p2p_process.rework_probability > 1) {
      errors.push({ field: 'ocpm.p2p_process.rework_probability', message: 'Rework probability must be between 0 and 1' });
    }
    if (config.ocpm.o2c_process.rework_probability < 0 || config.ocpm.o2c_process.rework_probability > 1) {
      errors.push({ field: 'ocpm.o2c_process.rework_probability', message: 'Rework probability must be between 0 and 1' });
    }
  }

  // Audit validation
  if (config.audit?.enabled) {
    const engagementSum =
      config.audit.engagement_types.financial_statement +
      config.audit.engagement_types.sox_icfr +
      config.audit.engagement_types.integrated +
      config.audit.engagement_types.review +
      config.audit.engagement_types.agreed_upon_procedures;
    if (Math.abs(engagementSum - 1.0) > 0.01) {
      errors.push({ field: 'audit.engagement_types', message: 'Engagement type weights must sum to 100%' });
    }
    if (config.audit.team.min_team_size > config.audit.team.max_team_size) {
      errors.push({ field: 'audit.team.max_team_size', message: 'Max team size must be >= min team size' });
    }
  }

  // Banking validation
  if (config.banking?.enabled) {
    if (config.banking.population.retail_customers === 0 &&
        config.banking.population.business_customers === 0 &&
        config.banking.population.trusts === 0) {
      errors.push({ field: 'banking.population', message: 'At least one customer type must have non-zero count' });
    }
    if (config.banking.spoofing.intensity < 0 || config.banking.spoofing.intensity > 1) {
      errors.push({ field: 'banking.spoofing.intensity', message: 'Spoofing intensity must be between 0 and 1' });
    }
    // Validate typology rates don't exceed suspicious rate
    const typologySum = config.banking.typologies.structuring_rate +
      config.banking.typologies.funnel_rate +
      config.banking.typologies.layering_rate +
      config.banking.typologies.mule_rate +
      config.banking.typologies.fraud_rate;
    if (typologySum > config.banking.typologies.suspicious_rate + 0.001) {
      errors.push({ field: 'banking.typologies', message: 'Sum of typology rates exceeds suspicious rate' });
    }
  }

  // Fingerprint validation
  if (config.fingerprint?.enabled) {
    if (config.fingerprint.scale < 0.1 || config.fingerprint.scale > 10) {
      errors.push({ field: 'fingerprint.scale', message: 'Scale must be between 0.1 and 10' });
    }
  }

  // Distributions validation
  if (config.distributions?.enabled) {
    // Validate mixture component weights sum to 1.0
    if (config.distributions.amounts?.enabled && config.distributions.amounts.components.length > 0) {
      const weightSum = config.distributions.amounts.components.reduce((sum, c) => sum + c.weight, 0);
      if (Math.abs(weightSum - 1.0) > 0.01) {
        errors.push({ field: 'distributions.amounts.components', message: 'Mixture component weights must sum to 1.0' });
      }
      // Validate sigma values are positive
      for (const comp of config.distributions.amounts.components) {
        if (comp.sigma <= 0) {
          errors.push({ field: 'distributions.amounts.components', message: 'Sigma values must be positive' });
          break;
        }
      }
    }
    // Validate correlation matrix
    if (config.distributions.correlations?.enabled && config.distributions.correlations.matrix.length > 0) {
      const n = config.distributions.correlations.fields.length;
      if (config.distributions.correlations.matrix.length !== n) {
        errors.push({ field: 'distributions.correlations.matrix', message: 'Correlation matrix dimensions must match number of fields' });
      }
      // Check diagonal is 1.0 and values are in [-1, 1]
      for (let i = 0; i < config.distributions.correlations.matrix.length; i++) {
        const row = config.distributions.correlations.matrix[i];
        if (row.length !== n) {
          errors.push({ field: 'distributions.correlations.matrix', message: 'Correlation matrix must be square' });
          break;
        }
        if (Math.abs(row[i] - 1.0) > 0.001) {
          errors.push({ field: 'distributions.correlations.matrix', message: 'Diagonal elements must be 1.0' });
          break;
        }
        for (const val of row) {
          if (val < -1 || val > 1) {
            errors.push({ field: 'distributions.correlations.matrix', message: 'Correlation values must be between -1 and 1' });
            break;
          }
        }
      }
    }
    // Validate economic cycle parameters
    if (config.distributions.regime_changes?.economic_cycle?.enabled) {
      if (config.distributions.regime_changes.economic_cycle.amplitude < 0 || config.distributions.regime_changes.economic_cycle.amplitude > 1) {
        errors.push({ field: 'distributions.regime_changes.economic_cycle.amplitude', message: 'Amplitude must be between 0 and 1' });
      }
      if (config.distributions.regime_changes.economic_cycle.recession_probability < 0 || config.distributions.regime_changes.economic_cycle.recession_probability > 1) {
        errors.push({ field: 'distributions.regime_changes.economic_cycle.recession_probability', message: 'Recession probability must be between 0 and 1' });
      }
    }
  }

  return errors;
}

// Export the singleton store
export const configStore = createConfigStore();

// Industry options
export const INDUSTRIES = [
  { value: 'manufacturing', label: 'Manufacturing' },
  { value: 'retail', label: 'Retail' },
  { value: 'financial_services', label: 'Financial Services' },
  { value: 'healthcare', label: 'Healthcare' },
  { value: 'technology', label: 'Technology' },
  { value: 'professional_services', label: 'Professional Services' },
  { value: 'energy', label: 'Energy' },
  { value: 'transportation', label: 'Transportation' },
  { value: 'real_estate', label: 'Real Estate' },
  { value: 'telecommunications', label: 'Telecommunications' },
];

// CoA complexity options
export const COA_COMPLEXITIES = [
  { value: 'small', label: 'Small (~100 accounts)' },
  { value: 'medium', label: 'Medium (~400 accounts)' },
  { value: 'large', label: 'Large (~2500 accounts)' },
];

// Transaction volume options
export const TRANSACTION_VOLUMES = [
  { value: 'ten_k', label: '10K (Small)' },
  { value: 'hundred_k', label: '100K (Medium)' },
  { value: 'one_m', label: '1M (Large)' },
  { value: 'ten_m', label: '10M (Enterprise)' },
  { value: 'hundred_m', label: '100M (Massive)' },
];

// Output format options
export const OUTPUT_FORMATS = [
  { value: 'csv', label: 'CSV', available: true },
  { value: 'json', label: 'JSON', available: true },
  { value: 'parquet', label: 'Parquet (not implemented)', available: false },
];

// Compression options
export const COMPRESSION_OPTIONS = [
  { value: 'none', label: 'None' },
  { value: 'gzip', label: 'GZip' },
  { value: 'zstd', label: 'Zstandard' },
  { value: 'lz4', label: 'LZ4' },
];

// Privacy level options (for fingerprinting)
export const PRIVACY_LEVELS = [
  { value: 'minimal', label: 'Minimal', epsilon: 5.0, k: 3, description: 'Low privacy, high utility' },
  { value: 'standard', label: 'Standard', epsilon: 1.0, k: 5, description: 'Balanced (recommended)' },
  { value: 'high', label: 'High', epsilon: 0.5, k: 10, description: 'Higher privacy' },
  { value: 'maximum', label: 'Maximum', epsilon: 0.1, k: 20, description: 'Maximum privacy' },
];

// Drift type options
export const DRIFT_TYPES = [
  { value: 'gradual', label: 'Gradual', description: 'Continuous drift over time (like inflation)' },
  { value: 'sudden', label: 'Sudden', description: 'Point-in-time shifts (like policy changes)' },
  { value: 'recurring', label: 'Recurring', description: 'Cyclic patterns (like seasonal variations)' },
  { value: 'mixed', label: 'Mixed', description: 'Gradual background with occasional sudden shifts' },
];

// Scenario profile presets
export const SCENARIO_PROFILES = [
  { value: 'clean', label: 'Clean', description: 'Minimal data quality issues' },
  { value: 'noisy', label: 'Noisy', description: 'Moderate issues (5% missing, 2% typos)' },
  { value: 'legacy', label: 'Legacy', description: 'Heavy issues simulating legacy systems' },
];

// Risk appetite options (for banking)
export const RISK_APPETITES = [
  { value: 'low', label: 'Low', description: 'Conservative risk tolerance' },
  { value: 'medium', label: 'Medium', description: 'Balanced risk tolerance' },
  { value: 'high', label: 'High', description: 'Aggressive risk tolerance' },
];

// Retail persona options (for banking)
export const RETAIL_PERSONAS = [
  { value: 'student', label: 'Student' },
  { value: 'early_career', label: 'Early Career' },
  { value: 'mid_career', label: 'Mid Career' },
  { value: 'retiree', label: 'Retiree' },
  { value: 'high_net_worth', label: 'High Net Worth' },
  { value: 'gig_worker', label: 'Gig Worker' },
];

// Business persona options (for banking)
export const BUSINESS_PERSONAS = [
  { value: 'small_business', label: 'Small Business' },
  { value: 'mid_market', label: 'Mid Market' },
  { value: 'enterprise', label: 'Enterprise' },
  { value: 'cash_intensive', label: 'Cash Intensive' },
  { value: 'import_export', label: 'Import/Export' },
  { value: 'professional_services', label: 'Professional Services' },
];

// Distribution types for mixture models
export const DISTRIBUTION_TYPES = [
  { value: 'lognormal', label: 'Log-Normal', description: 'Positive values with right skew (amounts, prices)' },
  { value: 'gaussian', label: 'Gaussian', description: 'Symmetric bell curve (errors, variations)' },
];

// Copula types for correlation modeling
export const COPULA_TYPES = [
  { value: 'gaussian', label: 'Gaussian', description: 'Symmetric, no tail dependence (general use)' },
  { value: 'clayton', label: 'Clayton', description: 'Lower tail dependence (risk modeling)' },
  { value: 'gumbel', label: 'Gumbel', description: 'Upper tail dependence (extreme events)' },
  { value: 'frank', label: 'Frank', description: 'Symmetric, no tail dependence (alternative)' },
  { value: 'student_t', label: 'Student-t', description: 'Both tail dependencies (heavy tails)' },
];

// Industry profiles for distribution settings
export const INDUSTRY_PROFILES = [
  { value: 'retail', label: 'Retail', description: 'High volume, lower amounts, seasonal patterns' },
  { value: 'manufacturing', label: 'Manufacturing', description: 'Moderate volume, varied amounts, equipment purchases' },
  { value: 'financial_services', label: 'Financial Services', description: 'Mixed volumes, wide amount range, regulatory patterns' },
  { value: 'healthcare', label: 'Healthcare', description: 'Billing cycles, insurance patterns' },
  { value: 'technology', label: 'Technology', description: 'Subscription revenue, capital expenses' },
];

// Regime change types
export const REGIME_CHANGE_TYPES = [
  { value: 'acquisition', label: 'Acquisition', description: 'Volume and amount increase from M&A' },
  { value: 'divestiture', label: 'Divestiture', description: 'Volume decrease from asset sale' },
  { value: 'policy_change', label: 'Policy Change', description: 'Threshold or process changes' },
  { value: 'price_increase', label: 'Price Increase', description: 'Amount mean shift' },
  { value: 'restructuring', label: 'Restructuring', description: 'Pattern changes from reorganization' },
];

// Statistical test types
export const STATISTICAL_TEST_TYPES = [
  { value: 'benford_first_digit', label: "Benford's Law (1st Digit)", description: 'Test first digit distribution' },
  { value: 'distribution_fit', label: 'Distribution Fit', description: 'K-S test against target distribution' },
  { value: 'correlation_check', label: 'Correlation Check', description: 'Verify expected field correlations' },
  { value: 'chi_squared', label: 'Chi-Squared', description: 'Categorical/binned distribution test' },
  { value: 'anderson_darling', label: 'Anderson-Darling', description: 'Goodness-of-fit with tail sensitivity' },
];

// =============================================================================
// Temporal Patterns Constants
// =============================================================================

// Holiday calendar regions
export const CALENDAR_REGIONS = [
  { value: 'US', label: 'United States', description: 'US federal holidays' },
  { value: 'DE', label: 'Germany', description: 'German national holidays' },
  { value: 'GB', label: 'United Kingdom', description: 'UK bank holidays' },
  { value: 'CN', label: 'China', description: 'Chinese public holidays (incl. lunar)' },
  { value: 'JP', label: 'Japan', description: 'Japanese national holidays' },
  { value: 'IN', label: 'India', description: 'Indian national holidays' },
  { value: 'BR', label: 'Brazil', description: 'Brazilian national holidays' },
  { value: 'MX', label: 'Mexico', description: 'Mexican national holidays' },
  { value: 'AU', label: 'Australia', description: 'Australian national holidays' },
  { value: 'SG', label: 'Singapore', description: 'Singapore public holidays' },
  { value: 'KR', label: 'South Korea', description: 'Korean national holidays' },
];

// Half-day policies
export const HALF_DAY_POLICIES = [
  { value: 'full_day', label: 'Full Day', description: 'Half-days treated as full business days' },
  { value: 'half_day', label: 'Half Day', description: 'Half-days have 50% activity (default)' },
  { value: 'non_business_day', label: 'Non-Business Day', description: 'Half-days treated as non-business days' },
];

// Month-end conventions
export const MONTH_END_CONVENTIONS = [
  { value: 'modified_following', label: 'Modified Following', description: 'Move to next business day unless it changes month, then previous (default)' },
  { value: 'following', label: 'Following', description: 'Always move to next business day' },
  { value: 'preceding', label: 'Preceding', description: 'Always move to previous business day' },
  { value: 'end_of_month', label: 'End of Month', description: 'Always use last business day of month' },
];

// Period-end models
export const PERIOD_END_MODELS = [
  { value: 'flat', label: 'Flat Multiplier', description: 'Constant spike on period-end days' },
  { value: 'exponential', label: 'Exponential', description: 'Gradually increasing activity (realistic)' },
  { value: 'extended_crunch', label: 'Extended Crunch', description: 'Sustained high activity for several days' },
  { value: 'daily_profile', label: 'Daily Profile', description: 'Custom multiplier per day-to-close' },
];

// Fiscal calendar types
export const FISCAL_CALENDAR_TYPES = [
  { value: 'calendar_year', label: 'Calendar Year', description: 'Standard January-December fiscal year' },
  { value: 'custom', label: 'Custom Year Start', description: 'Choose fiscal year start month/day' },
  { value: 'four_four_five', label: '4-4-5 Calendar', description: 'Retail calendar with 4-4-5 week pattern' },
  { value: 'thirteen_period', label: '13-Period', description: '13 four-week periods per year' },
];

// 4-4-5 week patterns
export const FOUR_FOUR_FIVE_PATTERNS = [
  { value: 'four_four_five', label: '4-4-5', description: 'Standard pattern (Walmart, Target)' },
  { value: 'four_five_four', label: '4-5-4', description: 'Alternative pattern' },
  { value: 'five_four_four', label: '5-4-4', description: 'Alternative pattern' },
];

// 4-4-5 anchor types
export const FOUR_FOUR_FIVE_ANCHORS = [
  { value: 'last_saturday', label: 'Last Saturday', description: 'Year ends on last Saturday of anchor month' },
  { value: 'first_sunday', label: 'First Sunday', description: 'Year starts on first Sunday of anchor month' },
  { value: 'nearest_saturday', label: 'Nearest Saturday', description: 'Year ends on Saturday nearest month end' },
];

// Common timezones
export const COMMON_TIMEZONES = [
  { value: 'America/New_York', label: 'Eastern (US)', offset: 'UTC-5/-4' },
  { value: 'America/Chicago', label: 'Central (US)', offset: 'UTC-6/-5' },
  { value: 'America/Denver', label: 'Mountain (US)', offset: 'UTC-7/-6' },
  { value: 'America/Los_Angeles', label: 'Pacific (US)', offset: 'UTC-8/-7' },
  { value: 'Europe/London', label: 'London', offset: 'UTC+0/+1' },
  { value: 'Europe/Berlin', label: 'Berlin', offset: 'UTC+1/+2' },
  { value: 'Europe/Paris', label: 'Paris', offset: 'UTC+1/+2' },
  { value: 'Europe/Zurich', label: 'Zurich', offset: 'UTC+1/+2' },
  { value: 'Asia/Tokyo', label: 'Tokyo', offset: 'UTC+9' },
  { value: 'Asia/Shanghai', label: 'Shanghai', offset: 'UTC+8' },
  { value: 'Asia/Singapore', label: 'Singapore', offset: 'UTC+8' },
  { value: 'Asia/Kolkata', label: 'Kolkata', offset: 'UTC+5:30' },
  { value: 'Australia/Sydney', label: 'Sydney', offset: 'UTC+10/+11' },
  { value: 'UTC', label: 'UTC', offset: 'UTC+0' },
];

// Intra-day posting types
export const POSTING_TYPES = [
  { value: 'both', label: 'Both', description: 'Both human and system postings' },
  { value: 'human', label: 'Human Only', description: 'Only human-initiated postings' },
  { value: 'system', label: 'System Only', description: 'Only system-initiated postings' },
];

// Default intra-day segments
export const DEFAULT_INTRADAY_SEGMENTS = [
  { name: 'morning_spike', start: '08:30', end: '10:00', multiplier: 1.8, posting_type: 'both' },
  { name: 'mid_morning', start: '10:00', end: '12:00', multiplier: 1.0, posting_type: 'both' },
  { name: 'lunch_dip', start: '12:00', end: '13:30', multiplier: 0.4, posting_type: 'human' },
  { name: 'afternoon', start: '13:30', end: '16:00', multiplier: 1.0, posting_type: 'both' },
  { name: 'eod_rush', start: '16:00', end: '17:30', multiplier: 1.5, posting_type: 'human' },
  { name: 'after_hours', start: '17:30', end: '23:59', multiplier: 0.3, posting_type: 'system' },
];
