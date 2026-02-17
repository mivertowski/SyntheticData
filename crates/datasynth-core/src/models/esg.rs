//! ESG (Environmental, Social, Governance) and Sustainability Reporting Models.
//!
//! Covers the GHG Protocol Scope 1/2/3 emissions, energy/water/waste tracking,
//! workforce diversity, pay equity, safety, governance metrics, supply chain ESG
//! assessments, and disclosure/assurance records.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

// ===========================================================================
// Environmental — Emissions
// ===========================================================================

/// GHG Protocol emission scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EmissionScope {
    /// Direct emissions from owned/controlled sources
    #[default]
    Scope1,
    /// Indirect emissions from purchased energy
    Scope2,
    /// All other indirect emissions in the value chain
    Scope3,
}

/// GHG Protocol Scope 3 categories (15 categories).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Scope3Category {
    #[default]
    PurchasedGoods,
    CapitalGoods,
    FuelAndEnergy,
    UpstreamTransport,
    WasteGenerated,
    BusinessTravel,
    EmployeeCommuting,
    UpstreamLeased,
    DownstreamTransport,
    ProcessingOfSoldProducts,
    UseOfSoldProducts,
    EndOfLifeTreatment,
    DownstreamLeased,
    Franchises,
    Investments,
}

/// Method used to estimate emissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EstimationMethod {
    /// Activity-based (consumption × emission factor)
    #[default]
    ActivityBased,
    /// Spend-based (procurement spend × EEIO factor)
    SpendBased,
    /// Supplier-specific (primary data from supply chain)
    SupplierSpecific,
    /// Average-data approach
    AverageData,
}

/// A single GHG emission record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmissionRecord {
    pub id: String,
    pub entity_id: String,
    pub scope: EmissionScope,
    pub scope3_category: Option<Scope3Category>,
    pub facility_id: Option<String>,
    pub period: NaiveDate,
    pub activity_data: Option<String>,
    pub activity_unit: Option<String>,
    #[serde(with = "rust_decimal::serde::str_option")]
    pub emission_factor: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::str")]
    pub co2e_tonnes: Decimal,
    pub estimation_method: EstimationMethod,
    pub source: Option<String>,
}

// ===========================================================================
// Environmental — Energy
// ===========================================================================

/// Type of energy source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EnergySourceType {
    #[default]
    Electricity,
    NaturalGas,
    Diesel,
    Coal,
    SolarPv,
    WindOnshore,
    Biomass,
    Geothermal,
}

impl EnergySourceType {
    /// Whether this energy source is renewable.
    pub fn is_renewable(&self) -> bool {
        matches!(
            self,
            Self::SolarPv | Self::WindOnshore | Self::Biomass | Self::Geothermal
        )
    }
}

/// Energy consumption record for a facility in a period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyConsumption {
    pub id: String,
    pub entity_id: String,
    pub facility_id: String,
    pub period: NaiveDate,
    pub energy_source: EnergySourceType,
    #[serde(with = "rust_decimal::serde::str")]
    pub consumption_kwh: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub cost: Decimal,
    pub currency: String,
    pub is_renewable: bool,
}

// ===========================================================================
// Environmental — Water
// ===========================================================================

/// Water source type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WaterSource {
    #[default]
    Municipal,
    Groundwater,
    SurfaceWater,
    Rainwater,
    Recycled,
}

/// Water usage record for a facility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterUsage {
    pub id: String,
    pub entity_id: String,
    pub facility_id: String,
    pub period: NaiveDate,
    pub source: WaterSource,
    #[serde(with = "rust_decimal::serde::str")]
    pub withdrawal_m3: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub discharge_m3: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub consumption_m3: Decimal,
    pub is_water_stressed_area: bool,
}

impl WaterUsage {
    /// Consumption = withdrawal − discharge.
    pub fn computed_consumption(&self) -> Decimal {
        (self.withdrawal_m3 - self.discharge_m3).max(Decimal::ZERO)
    }
}

// ===========================================================================
// Environmental — Waste
// ===========================================================================

/// Waste type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WasteType {
    #[default]
    General,
    Hazardous,
    Electronic,
    Organic,
    Construction,
}

/// Waste disposal method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DisposalMethod {
    #[default]
    Landfill,
    Recycled,
    Composted,
    Incinerated,
    Reused,
}

/// Waste generation record for a facility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasteRecord {
    pub id: String,
    pub entity_id: String,
    pub facility_id: String,
    pub period: NaiveDate,
    pub waste_type: WasteType,
    pub disposal_method: DisposalMethod,
    #[serde(with = "rust_decimal::serde::str")]
    pub quantity_tonnes: Decimal,
    pub is_diverted_from_landfill: bool,
}

impl WasteRecord {
    /// Whether the waste was diverted from landfill.
    pub fn computed_diversion(&self) -> bool {
        !matches!(
            self.disposal_method,
            DisposalMethod::Landfill | DisposalMethod::Incinerated
        )
    }
}

// ===========================================================================
// Social — Diversity
// ===========================================================================

/// Dimension of workforce diversity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DiversityDimension {
    #[default]
    Gender,
    Ethnicity,
    Age,
    Disability,
    VeteranStatus,
}

/// Organization level for metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationLevel {
    #[default]
    Corporate,
    Department,
    Team,
    Executive,
    Board,
}

/// Workforce diversity metric for a reporting period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkforceDiversityMetric {
    pub id: String,
    pub entity_id: String,
    pub period: NaiveDate,
    pub dimension: DiversityDimension,
    pub level: OrganizationLevel,
    pub category: String,
    pub headcount: u32,
    pub total_headcount: u32,
    #[serde(with = "rust_decimal::serde::str")]
    pub percentage: Decimal,
}

impl WorkforceDiversityMetric {
    /// Computed percentage = headcount / total_headcount.
    pub fn computed_percentage(&self) -> Decimal {
        if self.total_headcount == 0 {
            return Decimal::ZERO;
        }
        (Decimal::from(self.headcount) / Decimal::from(self.total_headcount)).round_dp(4)
    }
}

// ===========================================================================
// Social — Pay Equity
// ===========================================================================

/// Pay equity metric comparing compensation across groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayEquityMetric {
    pub id: String,
    pub entity_id: String,
    pub period: NaiveDate,
    pub dimension: DiversityDimension,
    pub reference_group: String,
    pub comparison_group: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub reference_median_salary: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub comparison_median_salary: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub pay_gap_ratio: Decimal,
    pub sample_size: u32,
}

impl PayEquityMetric {
    /// Computed pay gap ratio = comparison / reference.
    pub fn computed_pay_gap_ratio(&self) -> Decimal {
        if self.reference_median_salary.is_zero() {
            return dec!(1.00);
        }
        (self.comparison_median_salary / self.reference_median_salary).round_dp(4)
    }
}

// ===========================================================================
// Social — Safety
// ===========================================================================

/// Type of safety incident.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IncidentType {
    #[default]
    Injury,
    Illness,
    NearMiss,
    Fatality,
    PropertyDamage,
}

/// Individual safety incident record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyIncident {
    pub id: String,
    pub entity_id: String,
    pub facility_id: String,
    pub date: NaiveDate,
    pub incident_type: IncidentType,
    pub days_away: u32,
    pub is_recordable: bool,
    pub description: String,
}

/// Aggregate safety metrics for a period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyMetric {
    pub id: String,
    pub entity_id: String,
    pub period: NaiveDate,
    pub total_hours_worked: u64,
    pub recordable_incidents: u32,
    pub lost_time_incidents: u32,
    pub days_away: u32,
    pub near_misses: u32,
    pub fatalities: u32,
    #[serde(with = "rust_decimal::serde::str")]
    pub trir: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub ltir: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub dart_rate: Decimal,
}

impl SafetyMetric {
    /// TRIR (Total Recordable Incident Rate) = recordable × 200,000 / hours.
    pub fn computed_trir(&self) -> Decimal {
        if self.total_hours_worked == 0 {
            return Decimal::ZERO;
        }
        (Decimal::from(self.recordable_incidents) * dec!(200000)
            / Decimal::from(self.total_hours_worked))
        .round_dp(4)
    }

    /// LTIR (Lost Time Incident Rate) = lost_time × 200,000 / hours.
    pub fn computed_ltir(&self) -> Decimal {
        if self.total_hours_worked == 0 {
            return Decimal::ZERO;
        }
        (Decimal::from(self.lost_time_incidents) * dec!(200000)
            / Decimal::from(self.total_hours_worked))
        .round_dp(4)
    }

    /// DART (Days Away, Restricted, or Transferred) rate.
    pub fn computed_dart_rate(&self) -> Decimal {
        if self.total_hours_worked == 0 {
            return Decimal::ZERO;
        }
        (Decimal::from(self.days_away) * dec!(200000) / Decimal::from(self.total_hours_worked))
            .round_dp(4)
    }
}

// ===========================================================================
// Governance
// ===========================================================================

/// Governance metric for a reporting period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceMetric {
    pub id: String,
    pub entity_id: String,
    pub period: NaiveDate,
    pub board_size: u32,
    pub independent_directors: u32,
    pub female_directors: u32,
    #[serde(with = "rust_decimal::serde::str")]
    pub board_independence_ratio: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub board_gender_diversity_ratio: Decimal,
    pub ethics_training_completion_pct: f64,
    pub whistleblower_reports: u32,
    pub anti_corruption_violations: u32,
}

impl GovernanceMetric {
    /// Computed board independence = independent / total.
    pub fn computed_independence_ratio(&self) -> Decimal {
        if self.board_size == 0 {
            return Decimal::ZERO;
        }
        (Decimal::from(self.independent_directors) / Decimal::from(self.board_size)).round_dp(4)
    }
}

// ===========================================================================
// Supply Chain ESG
// ===========================================================================

/// ESG risk flag for a supplier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EsgRiskFlag {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

/// Method used for supplier ESG assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentMethod {
    #[default]
    SelfAssessment,
    ThirdPartyAudit,
    OnSiteAssessment,
    DocumentReview,
}

/// Supplier ESG assessment record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierEsgAssessment {
    pub id: String,
    pub entity_id: String,
    pub vendor_id: String,
    pub assessment_date: NaiveDate,
    pub method: AssessmentMethod,
    #[serde(with = "rust_decimal::serde::str")]
    pub environmental_score: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub social_score: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub governance_score: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub overall_score: Decimal,
    pub risk_flag: EsgRiskFlag,
    pub corrective_actions_required: u32,
}

impl SupplierEsgAssessment {
    /// Computed overall score = average of E, S, G.
    pub fn computed_overall_score(&self) -> Decimal {
        ((self.environmental_score + self.social_score + self.governance_score) / dec!(3))
            .round_dp(2)
    }
}

// ===========================================================================
// Reporting & Disclosure
// ===========================================================================

/// ESG reporting framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EsgFramework {
    /// Global Reporting Initiative
    #[default]
    Gri,
    /// European Sustainability Reporting Standards
    Esrs,
    /// Sustainability Accounting Standards Board
    Sasb,
    /// Task Force on Climate-related Financial Disclosures
    Tcfd,
    /// International Sustainability Standards Board
    Issb,
}

/// Level of assurance for ESG data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AssuranceLevel {
    /// No external assurance
    #[default]
    None,
    /// Limited assurance
    Limited,
    /// Reasonable assurance
    Reasonable,
}

/// An ESG disclosure record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsgDisclosure {
    pub id: String,
    pub entity_id: String,
    pub reporting_period_start: NaiveDate,
    pub reporting_period_end: NaiveDate,
    pub framework: EsgFramework,
    pub assurance_level: AssuranceLevel,
    pub disclosure_topic: String,
    pub metric_value: String,
    pub metric_unit: String,
    pub is_assured: bool,
}

/// Materiality assessment with double materiality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialityAssessment {
    pub id: String,
    pub entity_id: String,
    pub period: NaiveDate,
    pub topic: String,
    /// Impact materiality (outward impact on environment/society)
    #[serde(with = "rust_decimal::serde::str")]
    pub impact_score: Decimal,
    /// Financial materiality (inward impact on the enterprise)
    #[serde(with = "rust_decimal::serde::str")]
    pub financial_score: Decimal,
    /// Combined score
    #[serde(with = "rust_decimal::serde::str")]
    pub combined_score: Decimal,
    pub is_material: bool,
}

impl MaterialityAssessment {
    /// Double materiality: material if either dimension exceeds threshold.
    pub fn is_material_at_threshold(&self, threshold: Decimal) -> bool {
        self.impact_score >= threshold || self.financial_score >= threshold
    }
}

/// TCFD climate scenario type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioType {
    /// Well-below 2°C (Paris-aligned)
    #[default]
    WellBelow2C,
    /// Orderly transition
    Orderly,
    /// Disorderly transition
    Disorderly,
    /// Hot house world
    HotHouse,
}

/// Time horizon for scenario analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TimeHorizon {
    /// Short-term (1–3 years)
    Short,
    /// Medium-term (3–10 years)
    #[default]
    Medium,
    /// Long-term (10–30 years)
    Long,
}

/// A TCFD climate scenario analysis record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClimateScenario {
    pub id: String,
    pub entity_id: String,
    pub scenario_type: ScenarioType,
    pub time_horizon: TimeHorizon,
    pub description: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub temperature_rise_c: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub transition_risk_impact: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub physical_risk_impact: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub financial_impact: Decimal,
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    // -- Emissions --

    #[test]
    fn test_emission_record_serde_roundtrip() {
        let record = EmissionRecord {
            id: "EM-001".to_string(),
            entity_id: "C001".to_string(),
            scope: EmissionScope::Scope1,
            scope3_category: None,
            facility_id: Some("F-001".to_string()),
            period: d("2025-01-01"),
            activity_data: Some("100000 kWh".to_string()),
            activity_unit: Some("kWh".to_string()),
            emission_factor: Some(dec!(0.18)),
            co2e_tonnes: dec!(18),
            estimation_method: EstimationMethod::ActivityBased,
            source: Some("Natural gas combustion".to_string()),
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: EmissionRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.co2e_tonnes, dec!(18));
        assert_eq!(deserialized.scope, EmissionScope::Scope1);
    }

    #[test]
    fn test_emission_factor_calculation() {
        // activity_data * emission_factor = co2e
        let consumption_kwh = dec!(100000);
        let factor = dec!(0.18); // natural gas kg CO2e per kWh
        let co2e_kg = consumption_kwh * factor;
        let co2e_tonnes = co2e_kg / dec!(1000);
        assert_eq!(co2e_tonnes, dec!(18));
    }

    // -- Energy --

    #[test]
    fn test_energy_source_renewable() {
        assert!(EnergySourceType::SolarPv.is_renewable());
        assert!(EnergySourceType::WindOnshore.is_renewable());
        assert!(!EnergySourceType::NaturalGas.is_renewable());
        assert!(!EnergySourceType::Electricity.is_renewable());
    }

    // -- Water --

    #[test]
    fn test_water_consumption_formula() {
        let usage = WaterUsage {
            id: "W-001".to_string(),
            entity_id: "C001".to_string(),
            facility_id: "F-001".to_string(),
            period: d("2025-01-01"),
            source: WaterSource::Municipal,
            withdrawal_m3: dec!(5000),
            discharge_m3: dec!(3500),
            consumption_m3: dec!(1500),
            is_water_stressed_area: false,
        };

        assert_eq!(usage.computed_consumption(), dec!(1500));
    }

    // -- Waste --

    #[test]
    fn test_waste_diversion() {
        let recycled = WasteRecord {
            id: "WS-001".to_string(),
            entity_id: "C001".to_string(),
            facility_id: "F-001".to_string(),
            period: d("2025-01-01"),
            waste_type: WasteType::General,
            disposal_method: DisposalMethod::Recycled,
            quantity_tonnes: dec!(100),
            is_diverted_from_landfill: true,
        };
        assert!(recycled.computed_diversion());

        let landfill = WasteRecord {
            disposal_method: DisposalMethod::Landfill,
            ..recycled.clone()
        };
        assert!(!landfill.computed_diversion());
    }

    // -- Safety --

    #[test]
    fn test_trir_formula() {
        let metric = SafetyMetric {
            id: "SM-001".to_string(),
            entity_id: "C001".to_string(),
            period: d("2025-01-01"),
            total_hours_worked: 1_000_000,
            recordable_incidents: 5,
            lost_time_incidents: 2,
            days_away: 30,
            near_misses: 15,
            fatalities: 0,
            trir: dec!(1.0000),
            ltir: dec!(0.4000),
            dart_rate: dec!(6.0000),
        };

        // TRIR = 5 * 200,000 / 1,000,000 = 1.0
        assert_eq!(metric.computed_trir(), dec!(1.0000));
        // LTIR = 2 * 200,000 / 1,000,000 = 0.4
        assert_eq!(metric.computed_ltir(), dec!(0.4000));
        // DART = 30 * 200,000 / 1,000,000 = 6.0
        assert_eq!(metric.computed_dart_rate(), dec!(6.0000));
    }

    #[test]
    fn test_trir_zero_hours() {
        let metric = SafetyMetric {
            id: "SM-002".to_string(),
            entity_id: "C001".to_string(),
            period: d("2025-01-01"),
            total_hours_worked: 0,
            recordable_incidents: 0,
            lost_time_incidents: 0,
            days_away: 0,
            near_misses: 0,
            fatalities: 0,
            trir: Decimal::ZERO,
            ltir: Decimal::ZERO,
            dart_rate: Decimal::ZERO,
        };
        assert_eq!(metric.computed_trir(), Decimal::ZERO);
    }

    // -- Diversity --

    #[test]
    fn test_diversity_percentage() {
        let metric = WorkforceDiversityMetric {
            id: "WD-001".to_string(),
            entity_id: "C001".to_string(),
            period: d("2025-01-01"),
            dimension: DiversityDimension::Gender,
            level: OrganizationLevel::Corporate,
            category: "Female".to_string(),
            headcount: 450,
            total_headcount: 1000,
            percentage: dec!(0.4500),
        };

        assert_eq!(metric.computed_percentage(), dec!(0.4500));
    }

    // -- Pay Equity --

    #[test]
    fn test_pay_gap_ratio() {
        let metric = PayEquityMetric {
            id: "PE-001".to_string(),
            entity_id: "C001".to_string(),
            period: d("2025-01-01"),
            dimension: DiversityDimension::Gender,
            reference_group: "Male".to_string(),
            comparison_group: "Female".to_string(),
            reference_median_salary: dec!(85000),
            comparison_median_salary: dec!(78000),
            pay_gap_ratio: dec!(0.9176),
            sample_size: 500,
        };

        // 78000 / 85000 ≈ 0.9176
        assert_eq!(metric.computed_pay_gap_ratio(), dec!(0.9176));
    }

    // -- Governance --

    #[test]
    fn test_board_independence() {
        let metric = GovernanceMetric {
            id: "GOV-001".to_string(),
            entity_id: "C001".to_string(),
            period: d("2025-01-01"),
            board_size: 12,
            independent_directors: 8,
            female_directors: 4,
            board_independence_ratio: dec!(0.6667),
            board_gender_diversity_ratio: dec!(0.3333),
            ethics_training_completion_pct: 0.95,
            whistleblower_reports: 3,
            anti_corruption_violations: 0,
        };

        assert_eq!(metric.computed_independence_ratio(), dec!(0.6667));
    }

    // -- Supplier ESG --

    #[test]
    fn test_supplier_esg_overall_score() {
        let assessment = SupplierEsgAssessment {
            id: "SEA-001".to_string(),
            entity_id: "C001".to_string(),
            vendor_id: "V-001".to_string(),
            assessment_date: d("2025-06-15"),
            method: AssessmentMethod::ThirdPartyAudit,
            environmental_score: dec!(75),
            social_score: dec!(80),
            governance_score: dec!(85),
            overall_score: dec!(80),
            risk_flag: EsgRiskFlag::Low,
            corrective_actions_required: 0,
        };

        assert_eq!(assessment.computed_overall_score(), dec!(80.00));
    }

    // -- Materiality --

    #[test]
    fn test_materiality_double_threshold() {
        let assessment = MaterialityAssessment {
            id: "MA-001".to_string(),
            entity_id: "C001".to_string(),
            period: d("2025-01-01"),
            topic: "Climate Change".to_string(),
            impact_score: dec!(8.5),
            financial_score: dec!(6.0),
            combined_score: dec!(7.25),
            is_material: true,
        };

        // Material if either dimension ≥ 7.0
        assert!(assessment.is_material_at_threshold(dec!(7.0)));
        // Not material if both need to be ≥ 9.0
        assert!(!assessment.is_material_at_threshold(dec!(9.0)));
    }

    // -- Serde --

    #[test]
    fn test_safety_metric_serde_roundtrip() {
        let metric = SafetyMetric {
            id: "SM-100".to_string(),
            entity_id: "C001".to_string(),
            period: d("2025-01-01"),
            total_hours_worked: 500_000,
            recordable_incidents: 3,
            lost_time_incidents: 1,
            days_away: 10,
            near_misses: 8,
            fatalities: 0,
            trir: dec!(1.2000),
            ltir: dec!(0.4000),
            dart_rate: dec!(4.0000),
        };

        let json = serde_json::to_string(&metric).unwrap();
        let deserialized: SafetyMetric = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.trir, dec!(1.2000));
        assert_eq!(deserialized.recordable_incidents, 3);
    }

    #[test]
    fn test_climate_scenario_serde() {
        let scenario = ClimateScenario {
            id: "CS-001".to_string(),
            entity_id: "C001".to_string(),
            scenario_type: ScenarioType::WellBelow2C,
            time_horizon: TimeHorizon::Long,
            description: "Paris-aligned scenario".to_string(),
            temperature_rise_c: dec!(1.5),
            transition_risk_impact: dec!(-50000),
            physical_risk_impact: dec!(-10000),
            financial_impact: dec!(-60000),
        };

        let json = serde_json::to_string(&scenario).unwrap();
        let deserialized: ClimateScenario = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.scenario_type, ScenarioType::WellBelow2C);
        assert_eq!(deserialized.temperature_rise_c, dec!(1.5));
    }
}
