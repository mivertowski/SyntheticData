//! Country pack schema types.
//!
//! Defines the `CountryPack` struct and all sub-structs that map to the
//! country-pack JSON schema (spec §3.1–§3.16). Every field uses `#[serde(default)]`
//! so that partial packs (e.g. `_default.json`) deserialize cleanly.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Top-level
// ---------------------------------------------------------------------------

/// A complete country pack loaded from JSON.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CountryPack {
    /// Schema version (e.g. "1.0").
    #[serde(default)]
    pub schema_version: String,
    /// ISO 3166-1 alpha-2 country code, or "_DEFAULT".
    #[serde(default)]
    pub country_code: String,
    /// Human-readable country name.
    #[serde(default)]
    pub country_name: String,
    /// Region grouping: AMERICAS, EMEA, APAC.
    #[serde(default)]
    pub region: String,

    #[serde(default)]
    pub locale: LocaleConfig,
    #[serde(default)]
    pub names: NamesConfig,
    #[serde(default)]
    pub holidays: HolidaysConfig,
    #[serde(default)]
    pub tax: CountryTaxConfig,
    #[serde(default)]
    pub address: AddressConfig,
    #[serde(default)]
    pub phone: PhoneConfig,
    #[serde(default)]
    pub banking: BankingCountryConfig,
    #[serde(default)]
    pub business_rules: BusinessRulesConfig,
    #[serde(default)]
    pub legal_entities: LegalEntitiesConfig,
    #[serde(default)]
    pub accounting: AccountingCountryConfig,
    #[serde(default)]
    pub payroll: PayrollCountryConfig,
    #[serde(default)]
    pub vendor_templates: EntityTemplatesConfig,
    #[serde(default)]
    pub customer_templates: EntityTemplatesConfig,
    #[serde(default)]
    pub material_templates: MaterialTemplatesConfig,
    #[serde(default)]
    pub document_texts: DocumentTextsConfig,
}

// ---------------------------------------------------------------------------
// §3.2  Locale
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocaleConfig {
    #[serde(default)]
    pub language_code: String,
    #[serde(default)]
    pub language_name: String,
    #[serde(default)]
    pub default_currency: String,
    #[serde(default)]
    pub currency_symbol: String,
    #[serde(default = "default_currency_decimal_places")]
    pub currency_decimal_places: u8,
    #[serde(default)]
    pub number_format: NumberFormatConfig,
    #[serde(default)]
    pub date_format: DateFormatConfig,
    #[serde(default)]
    pub default_timezone: String,
    #[serde(default)]
    pub weekend_days: Vec<String>,
    #[serde(default)]
    pub fiscal_year: FiscalYearConfig,
}

fn default_currency_decimal_places() -> u8 {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NumberFormatConfig {
    #[serde(default = "default_decimal_separator")]
    pub decimal_separator: String,
    #[serde(default = "default_thousands_separator")]
    pub thousands_separator: String,
    #[serde(default)]
    pub grouping: Vec<u8>,
}

fn default_decimal_separator() -> String {
    ".".to_string()
}

fn default_thousands_separator() -> String {
    ",".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DateFormatConfig {
    #[serde(default)]
    pub short: String,
    #[serde(default)]
    pub long: String,
    #[serde(default)]
    pub iso: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiscalYearConfig {
    #[serde(default = "default_one")]
    pub start_month: u32,
    #[serde(default = "default_one")]
    pub start_day: u32,
    #[serde(default = "default_fiscal_variant_str")]
    pub variant: String,
}

impl Default for FiscalYearConfig {
    fn default() -> Self {
        Self {
            start_month: 1,
            start_day: 1,
            variant: "calendar".to_string(),
        }
    }
}

fn default_one() -> u32 {
    1
}

fn default_fiscal_variant_str() -> String {
    "calendar".to_string()
}

// ---------------------------------------------------------------------------
// §3.3  Names
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NamesConfig {
    #[serde(default)]
    pub cultures: Vec<CultureConfig>,
    #[serde(default)]
    pub email_domains: Vec<String>,
    #[serde(default)]
    pub username_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CultureConfig {
    #[serde(default)]
    pub culture_id: String,
    #[serde(default)]
    pub weight: f64,
    #[serde(default)]
    pub male_first_names: Vec<String>,
    #[serde(default)]
    pub female_first_names: Vec<String>,
    #[serde(default)]
    pub last_names: Vec<String>,
    #[serde(default = "default_western")]
    pub name_order: String,
    #[serde(default)]
    pub titles: TitleConfig,
    #[serde(default)]
    pub academic_titles: Vec<String>,
}

fn default_western() -> String {
    "western".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TitleConfig {
    #[serde(default)]
    pub male: Vec<String>,
    #[serde(default)]
    pub female: Vec<String>,
}

// ---------------------------------------------------------------------------
// §3.4  Holidays
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HolidaysConfig {
    #[serde(default = "default_gregorian")]
    pub calendar_type: String,
    #[serde(default)]
    pub fixed: Vec<FixedHoliday>,
    #[serde(default)]
    pub easter_relative: Vec<EasterRelativeHoliday>,
    #[serde(default)]
    pub nth_weekday: Vec<NthWeekdayHoliday>,
    #[serde(default)]
    pub last_weekday: Vec<LastWeekdayHoliday>,
    #[serde(default)]
    pub lunar: Vec<LunarHoliday>,
    #[serde(default)]
    pub regional_holidays: RegionalHolidaysConfig,
    #[serde(default)]
    pub holiday_seasons: Vec<HolidaySeasonConfig>,
}

fn default_gregorian() -> String {
    "gregorian".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FixedHoliday {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub name_en: String,
    #[serde(default)]
    pub month: u32,
    #[serde(default)]
    pub day: u32,
    #[serde(default = "default_holiday_activity")]
    pub activity_multiplier: f64,
    #[serde(default)]
    pub observe_weekend_rule: bool,
}

fn default_holiday_activity() -> f64 {
    0.05
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EasterRelativeHoliday {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub name_en: String,
    #[serde(default)]
    pub offset_days: i32,
    #[serde(default = "default_holiday_activity")]
    pub activity_multiplier: f64,
}

/// "Nth weekday of month" holiday (e.g. 3rd Monday of January = MLK Day).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NthWeekdayHoliday {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub name_en: String,
    /// 1-12
    #[serde(default)]
    pub month: u32,
    /// Day of week: "monday", "tuesday", ..., "sunday"
    #[serde(default)]
    pub weekday: String,
    /// 1-based occurrence (1=first, 2=second, ..., 4=fourth)
    #[serde(default)]
    pub occurrence: u32,
    /// Days to add after the computed date (e.g. 1 for "day after Thanksgiving").
    #[serde(default)]
    pub offset_days: i32,
    #[serde(default = "default_holiday_activity")]
    pub activity_multiplier: f64,
}

/// "Last weekday of month" holiday (e.g. last Monday of May = Memorial Day).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LastWeekdayHoliday {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub name_en: String,
    /// 1-12
    #[serde(default)]
    pub month: u32,
    /// Day of week: "monday", "tuesday", ..., "sunday"
    #[serde(default)]
    pub weekday: String,
    #[serde(default = "default_holiday_activity")]
    pub activity_multiplier: f64,
}

/// Lunar-calendar holiday resolved by a named algorithm at runtime.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LunarHoliday {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub name_en: String,
    /// Algorithm name dispatched by `lunar::resolve_lunar_holiday()`.
    /// Examples: "chinese_new_year", "diwali", "vesak", "hari_raya_puasa",
    /// "hari_raya_haji", "deepavali", "korean_new_year",
    /// "korean_buddha_birthday", "chuseok"
    #[serde(default)]
    pub algorithm: String,
    /// Number of consecutive holiday days (e.g. Chinese New Year = 7).
    #[serde(default = "default_duration")]
    pub duration_days: u32,
    #[serde(default = "default_holiday_activity")]
    pub activity_multiplier: f64,
}

fn default_duration() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegionalHolidaysConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub regions: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HolidaySeasonConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub name_en: String,
    #[serde(default)]
    pub start: MonthDay,
    #[serde(default)]
    pub end: MonthDay,
    #[serde(default = "default_holiday_activity")]
    pub activity_multiplier: f64,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonthDay {
    #[serde(default)]
    pub month: u32,
    #[serde(default)]
    pub day: u32,
}

// ---------------------------------------------------------------------------
// §3.5  Tax
// ---------------------------------------------------------------------------

/// Country-level tax configuration.
/// Named `CountryTaxConfig` to avoid collision with the generator-level `TaxConfig`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CountryTaxConfig {
    #[serde(default)]
    pub corporate_income_tax: CorporateIncomeTaxConfig,
    #[serde(default)]
    pub vat: VatConfig,
    #[serde(default)]
    pub withholding_tax: WithholdingTaxConfig,
    #[serde(default)]
    pub payroll_tax: PayrollTaxBracketsConfig,
    #[serde(default)]
    pub transfer_pricing: TransferPricingConfig,
    /// Sub-national tax jurisdictions (US states, German Bundesländer, etc.).
    #[serde(default)]
    pub subnational: Vec<SubnationalTaxConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CorporateIncomeTaxConfig {
    #[serde(default)]
    pub standard_rate: f64,
    #[serde(default)]
    pub trade_tax_rate: f64,
    #[serde(default)]
    pub solidarity_surcharge: f64,
    #[serde(default)]
    pub effective_combined_rate: f64,
    #[serde(default)]
    pub small_business_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VatConfig {
    #[serde(default)]
    pub standard_rate: f64,
    #[serde(default)]
    pub reduced_rates: Vec<ReducedRate>,
    #[serde(default)]
    pub zero_rated: Vec<String>,
    #[serde(default)]
    pub exempt: Vec<String>,
    #[serde(default)]
    pub registration_threshold: Option<f64>,
    #[serde(default)]
    pub filing_frequency: String,
    #[serde(default)]
    pub reverse_charge_applicable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReducedRate {
    #[serde(default)]
    pub rate: f64,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub applies_to: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WithholdingTaxConfig {
    #[serde(default)]
    pub dividends_domestic: f64,
    #[serde(default)]
    pub dividends_foreign_default: f64,
    #[serde(default)]
    pub interest: f64,
    #[serde(default)]
    pub royalties: f64,
    #[serde(default)]
    pub services: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PayrollTaxBracketsConfig {
    #[serde(default)]
    pub income_tax_brackets: Vec<TaxBracket>,
    #[serde(default)]
    pub social_security: serde_json::Value,
    #[serde(default)]
    pub church_tax_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaxBracket {
    #[serde(default)]
    pub up_to: Option<f64>,
    #[serde(default)]
    pub above: Option<f64>,
    #[serde(default)]
    pub rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransferPricingConfig {
    #[serde(default)]
    pub documentation_required: bool,
    #[serde(default)]
    pub methods: Vec<String>,
    #[serde(default)]
    pub safe_harbor_rules: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubnationalTaxConfig {
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub rate: f64,
    #[serde(default)]
    pub tax_type: String,
}

// ---------------------------------------------------------------------------
// §3.6  Address
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AddressConfig {
    #[serde(default)]
    pub format_template: String,
    #[serde(default)]
    pub components: AddressComponentsConfig,
    #[serde(default)]
    pub postal_code: PostalCodeConfig,
    #[serde(default)]
    pub building_number: BuildingNumberConfig,
    #[serde(default)]
    pub country_calling_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AddressComponentsConfig {
    #[serde(default)]
    pub street_names: Vec<String>,
    #[serde(default)]
    pub city_names: Vec<String>,
    #[serde(default)]
    pub state_names: Vec<String>,
    #[serde(default)]
    pub state_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostalCodeConfig {
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub regex: String,
    #[serde(default)]
    pub ranges: Vec<PostalCodeRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostalCodeRange {
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildingNumberConfig {
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub range: Vec<u32>,
}

// ---------------------------------------------------------------------------
// §3.7  Phone
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PhoneConfig {
    #[serde(default)]
    pub country_calling_code: String,
    #[serde(default)]
    pub formats: PhoneFormatsConfig,
    #[serde(default)]
    pub area_codes: Vec<String>,
    #[serde(default)]
    pub subscriber_length: SubscriberLengthConfig,
    #[serde(default)]
    pub display_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PhoneFormatsConfig {
    #[serde(default)]
    pub landline: String,
    #[serde(default)]
    pub mobile: String,
    #[serde(default)]
    pub freephone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubscriberLengthConfig {
    #[serde(default)]
    pub min: u32,
    #[serde(default)]
    pub max: u32,
}

// ---------------------------------------------------------------------------
// §3.8  Banking
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BankingCountryConfig {
    #[serde(default)]
    pub account_format: String,
    #[serde(default)]
    pub iban: IbanConfig,
    #[serde(default)]
    pub domestic_format: Option<DomesticBankFormatConfig>,
    #[serde(default)]
    pub bank_names: Vec<String>,
    #[serde(default)]
    pub swift_prefix: String,
    #[serde(default)]
    pub payment_systems: Vec<String>,
    #[serde(default)]
    pub settlement_rules: SettlementRulesConfig,
    #[serde(default)]
    pub kyc_requirements: KycRequirementsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IbanConfig {
    #[serde(default)]
    pub country_prefix: String,
    #[serde(default)]
    pub length: u32,
    #[serde(default)]
    pub bban_structure: String,
    #[serde(default)]
    pub check_digit_algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DomesticBankFormatConfig {
    #[serde(default)]
    pub routing_number_length: u32,
    #[serde(default)]
    pub account_number_length: u32,
    #[serde(default)]
    pub format_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettlementRulesConfig {
    #[serde(default)]
    pub domestic_transfer_days: u32,
    #[serde(default)]
    pub international_transfer_days: u32,
    #[serde(default)]
    pub wire_cutoff_time: String,
    #[serde(default)]
    pub direct_debit_lead_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KycRequirementsConfig {
    #[serde(default)]
    pub id_document_types: Vec<String>,
    #[serde(default)]
    pub pep_screening_required: bool,
    #[serde(default)]
    pub beneficial_ownership_threshold: f64,
    #[serde(default)]
    pub enhanced_due_diligence_triggers: Vec<String>,
}

// ---------------------------------------------------------------------------
// §3.9  Business Rules
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BusinessRulesConfig {
    #[serde(default)]
    pub invoice: InvoiceRulesConfig,
    #[serde(default)]
    pub payment_terms: PaymentTermsConfig,
    #[serde(default)]
    pub approval_thresholds: ApprovalThresholdsConfig,
    #[serde(default)]
    pub data_privacy: DataPrivacyConfig,
    /// Country-level carbon intensity multiplier for spend-based Scope 3 emissions.
    /// Defaults to 0.0 (unset); the generator treats 0.0 as 1.0.
    #[serde(default)]
    pub emission_country_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InvoiceRulesConfig {
    #[serde(default)]
    pub numbering_format: String,
    #[serde(default)]
    pub mandatory_fields: Vec<String>,
    #[serde(default)]
    pub retention_years: u32,
    #[serde(default)]
    pub electronic_invoice_mandatory: bool,
    #[serde(default)]
    pub e_invoice_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaymentTermsConfig {
    #[serde(default = "default_payment_days")]
    pub default_days: u32,
    #[serde(default)]
    pub common_terms: Vec<u32>,
    #[serde(default)]
    pub early_payment_discount: EarlyPaymentDiscountConfig,
    #[serde(default)]
    pub late_payment_interest: LatePaymentInterestConfig,
}

fn default_payment_days() -> u32 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EarlyPaymentDiscountConfig {
    #[serde(default)]
    pub common_rate: f64,
    #[serde(default)]
    pub common_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LatePaymentInterestConfig {
    #[serde(default)]
    pub statutory_rate: f64,
    #[serde(default)]
    pub base_rate_reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApprovalThresholdsConfig {
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub levels: Vec<ApprovalLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApprovalLevel {
    #[serde(default)]
    pub up_to: Option<f64>,
    #[serde(default)]
    pub above: Option<f64>,
    #[serde(default)]
    pub approver: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataPrivacyConfig {
    #[serde(default)]
    pub regulation: String,
    #[serde(default)]
    pub pseudonymization_required: bool,
    #[serde(default)]
    pub retention_limits: RetentionLimitsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RetentionLimitsConfig {
    #[serde(default)]
    pub employee_data_years: u32,
    #[serde(default)]
    pub financial_records_years: u32,
    #[serde(default)]
    pub tax_records_years: u32,
}

// ---------------------------------------------------------------------------
// §3.10  Legal Entities
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LegalEntitiesConfig {
    #[serde(default)]
    pub entity_types: Vec<EntityTypeConfig>,
    #[serde(default)]
    pub tax_id_format: IdFormatConfig,
    #[serde(default)]
    pub vat_id_format: VatIdFormatConfig,
    #[serde(default)]
    pub registration_authority: String,
    #[serde(default)]
    pub registration_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityTypeConfig {
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub name_en: String,
    #[serde(default)]
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdFormatConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub regex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VatIdFormatConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub regex: String,
}

// ---------------------------------------------------------------------------
// §3.11  Accounting
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountingCountryConfig {
    #[serde(default)]
    pub framework: String,
    #[serde(default)]
    pub secondary_framework: Option<String>,
    #[serde(default)]
    pub local_gaap_name: String,
    #[serde(default)]
    pub chart_of_accounts: ChartOfAccountsCountryConfig,
    #[serde(default)]
    pub audit_framework: String,
    #[serde(default)]
    pub regulatory: RegulatoryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChartOfAccountsCountryConfig {
    #[serde(default)]
    pub standard: String,
    #[serde(default)]
    pub numbering_length: u32,
    #[serde(default)]
    pub account_ranges: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegulatoryConfig {
    #[serde(default)]
    pub sox_applicable: bool,
    #[serde(default)]
    pub local_regulations: Vec<String>,
    #[serde(default)]
    pub filing_requirements: Vec<FilingRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilingRequirement {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub name_en: String,
    #[serde(default)]
    pub deadline_months_after_ye: u32,
}

// ---------------------------------------------------------------------------
// §3.12  Payroll
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PayrollCountryConfig {
    #[serde(default)]
    pub pay_frequency: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub statutory_deductions: Vec<PayrollDeduction>,
    #[serde(default)]
    pub employer_contributions: Vec<PayrollDeduction>,
    #[serde(default)]
    pub minimum_wage: MinimumWageConfig,
    #[serde(default)]
    pub working_hours: WorkingHoursConfig,
    #[serde(default)]
    pub thirteenth_month: bool,
    #[serde(default)]
    pub severance: SeveranceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PayrollDeduction {
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub name_en: String,
    /// "percentage" | "progressive" | "fixed"
    #[serde(default)]
    pub deduction_type: String,
    #[serde(rename = "type", default)]
    pub type_field: String,
    #[serde(default)]
    pub rate: f64,
    #[serde(default)]
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MinimumWageConfig {
    #[serde(default)]
    pub hourly: f64,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub effective_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkingHoursConfig {
    #[serde(default = "default_weekly_hours")]
    pub standard_weekly: f64,
    #[serde(default)]
    pub max_daily: f64,
    #[serde(default)]
    pub statutory_annual_leave_days: u32,
    #[serde(default)]
    pub common_annual_leave_days: u32,
}

fn default_weekly_hours() -> f64 {
    40.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SeveranceConfig {
    #[serde(default)]
    pub statutory: bool,
    #[serde(default)]
    pub formula: String,
}

// ---------------------------------------------------------------------------
// §3.13 / §3.14  Entity Templates (vendor + customer share the same shape)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityTemplatesConfig {
    #[serde(default)]
    pub name_patterns: Vec<String>,
    #[serde(default)]
    pub industry_words: serde_json::Value,
}

// ---------------------------------------------------------------------------
// §3.15  Material Templates
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MaterialTemplatesConfig {
    #[serde(default)]
    pub categories: serde_json::Value,
    #[serde(default)]
    pub unit_of_measure_labels: serde_json::Value,
}

// ---------------------------------------------------------------------------
// §3.16  Document Texts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentTextsConfig {
    #[serde(default)]
    pub purchase_order: DocumentTextGroup,
    #[serde(default)]
    pub invoice: DocumentTextGroup,
    #[serde(default)]
    pub journal_entry: DocumentTextGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentTextGroup {
    #[serde(default)]
    pub header_templates: Vec<String>,
    #[serde(default)]
    pub line_descriptions: Vec<String>,
    #[serde(default)]
    pub posting_texts: Vec<String>,
}

// ---------------------------------------------------------------------------
// Emission factor (extension beyond spec — for ESG generator)
// ---------------------------------------------------------------------------

/// Country-level carbon intensity multiplier for spend-based Scope 3 emissions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmissionFactorConfig {
    #[serde(default = "default_emission_multiplier")]
    pub country_multiplier: f64,
}

fn default_emission_multiplier() -> f64 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_country_pack() {
        let pack = CountryPack::default();
        assert!(pack.country_code.is_empty());
        assert!(pack.holidays.fixed.is_empty());
        assert!(pack.names.cultures.is_empty());
    }

    #[test]
    fn test_deserialize_minimal_json() {
        let json =
            r#"{"schema_version": "1.0", "country_code": "US", "country_name": "United States"}"#;
        let pack: CountryPack = serde_json::from_str(json).expect("should parse");
        assert_eq!(pack.country_code, "US");
        assert_eq!(pack.schema_version, "1.0");
        assert!(pack.holidays.fixed.is_empty());
    }

    #[test]
    fn test_deserialize_fixed_holiday() {
        let json = r#"{
            "name": "New Year's Day",
            "name_en": "New Year's Day",
            "month": 1,
            "day": 1,
            "activity_multiplier": 0.05,
            "observe_weekend_rule": true
        }"#;
        let h: FixedHoliday = serde_json::from_str(json).expect("should parse");
        assert_eq!(h.month, 1);
        assert_eq!(h.day, 1);
        assert!(h.observe_weekend_rule);
    }

    #[test]
    fn test_deserialize_nth_weekday_holiday() {
        let json = r#"{
            "name": "MLK Day",
            "name_en": "Martin Luther King Jr. Day",
            "month": 1,
            "weekday": "monday",
            "occurrence": 3,
            "activity_multiplier": 0.1
        }"#;
        let h: NthWeekdayHoliday = serde_json::from_str(json).expect("should parse");
        assert_eq!(h.month, 1);
        assert_eq!(h.weekday, "monday");
        assert_eq!(h.occurrence, 3);
    }
}
