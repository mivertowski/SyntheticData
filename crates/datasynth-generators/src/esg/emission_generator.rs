//! Emission generator — derives GHG Protocol Scope 1/2/3 emission records
//! from operational data (energy consumption, vendor spend, headcount).
//!
//! Uses EPA/DEFRA-style emission factors to convert activity data to CO2e tonnes.
use chrono::NaiveDate;
use datasynth_config::schema::EnvironmentalConfig;
use datasynth_core::models::{EmissionRecord, EmissionScope, EstimationMethod, Scope3Category};
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ---------------------------------------------------------------------------
// Input types — lightweight structs that upstream generators feed in
// ---------------------------------------------------------------------------

/// Energy consumption input for Scope 1 emission derivation.
#[derive(Debug, Clone)]
pub struct EnergyInput {
    pub facility_id: String,
    pub energy_type: EnergyInputType,
    /// Consumption in kWh.
    pub consumption_kwh: Decimal,
    /// Period start date (first of month).
    pub period: NaiveDate,
}

/// Energy input type for emission factor lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergyInputType {
    NaturalGas,
    Diesel,
    Coal,
    Electricity,
}

/// Vendor spend input for Scope 3 emission derivation.
#[derive(Debug, Clone)]
pub struct VendorSpendInput {
    pub vendor_id: String,
    pub category: String,
    pub spend: Decimal,
    pub country: String,
}

// ---------------------------------------------------------------------------
// Emission factors (kg CO2e per kWh, per USD, etc.)
// ---------------------------------------------------------------------------

/// Look up an activity-based emission factor (kg CO2e / kWh).
fn emission_factor_kg_per_kwh(energy_type: EnergyInputType) -> Decimal {
    match energy_type {
        // EPA GHG factors (approximate)
        EnergyInputType::NaturalGas => dec!(0.181), // kg CO2e / kWh
        EnergyInputType::Diesel => dec!(0.253),
        EnergyInputType::Coal => dec!(0.341),
        EnergyInputType::Electricity => dec!(0.417), // US grid average
    }
}

/// Look up a spend-based emission factor (kg CO2e / USD).
fn spend_emission_factor(category: &str, country: &str) -> Decimal {
    let base = match category {
        "manufacturing" => dec!(0.80),
        "construction" => dec!(0.65),
        "transportation" => dec!(0.55),
        "chemicals" => dec!(0.70),
        "agriculture" => dec!(0.60),
        "mining" => dec!(0.90),
        "office_supplies" => dec!(0.20),
        "professional_services" => dec!(0.15),
        "technology" => dec!(0.25),
        _ => dec!(0.40), // generic EEIO factor
    };

    // Country adjustment multiplier
    let country_mult = match country {
        "CN" => dec!(1.30),
        "IN" => dec!(1.25),
        "US" => dec!(1.00),
        "DE" | "FR" | "GB" => dec!(0.85),
        "JP" => dec!(0.90),
        _ => dec!(1.00),
    };

    base * country_mult
}

// ---------------------------------------------------------------------------
// EmissionGenerator
// ---------------------------------------------------------------------------

/// Generates [`EmissionRecord`] values from operational data.
///
/// Scope 1: fuel combustion (natural gas, diesel, coal) → activity-based
/// Scope 2: purchased electricity → activity-based
/// Scope 3: vendor spend → spend-based, business travel → average-data
pub struct EmissionGenerator {
    rng: ChaCha8Rng,
    config: EnvironmentalConfig,
    counter: u64,
}

impl EmissionGenerator {
    /// Create a new emission generator.
    pub fn new(config: EnvironmentalConfig, seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
            counter: 0,
        }
    }

    // ----- Scope 1: Direct emissions from fuel combustion -----

    /// Generate Scope 1 emission records from energy consumption data.
    ///
    /// Applies activity-based emission factors to fuel inputs
    /// (natural gas, diesel, coal). Electricity is excluded (Scope 2).
    pub fn generate_scope1(
        &mut self,
        entity_id: &str,
        energy_data: &[EnergyInput],
    ) -> Vec<EmissionRecord> {
        if !self.config.scope1.enabled {
            return Vec::new();
        }

        energy_data
            .iter()
            .filter(|e| e.energy_type != EnergyInputType::Electricity)
            .map(|e| {
                self.counter += 1;
                let factor = emission_factor_kg_per_kwh(e.energy_type);
                let co2e_kg = e.consumption_kwh * factor;
                // Convert kg to tonnes (÷ 1000)
                let co2e_tonnes = (co2e_kg / dec!(1000)).round_dp(4);

                // Small random variance (±5%) to simulate measurement uncertainty
                let variance = dec!(1) + self.random_variance();
                let co2e_tonnes = (co2e_tonnes * variance).round_dp(4);

                EmissionRecord {
                    id: format!("EM-{:06}", self.counter),
                    entity_id: entity_id.to_string(),
                    scope: EmissionScope::Scope1,
                    scope3_category: None,
                    facility_id: Some(e.facility_id.clone()),
                    period: e.period,
                    activity_data: Some(format!("{} kWh", e.consumption_kwh)),
                    activity_unit: Some("kWh".to_string()),
                    emission_factor: Some(factor),
                    co2e_tonnes,
                    estimation_method: EstimationMethod::ActivityBased,
                    source: Some(format!(
                        "EPA GHG factors ({})",
                        self.config.scope1.factor_region
                    )),
                }
            })
            .collect()
    }

    // ----- Scope 2: Indirect emissions from purchased electricity -----

    /// Generate Scope 2 emission records from purchased electricity data.
    pub fn generate_scope2(
        &mut self,
        entity_id: &str,
        energy_data: &[EnergyInput],
    ) -> Vec<EmissionRecord> {
        if !self.config.scope2.enabled {
            return Vec::new();
        }

        energy_data
            .iter()
            .filter(|e| e.energy_type == EnergyInputType::Electricity)
            .map(|e| {
                self.counter += 1;
                let factor = emission_factor_kg_per_kwh(EnergyInputType::Electricity);
                let co2e_kg = e.consumption_kwh * factor;
                let co2e_tonnes = (co2e_kg / dec!(1000)).round_dp(4);

                let variance = dec!(1) + self.random_variance();
                let co2e_tonnes = (co2e_tonnes * variance).round_dp(4);

                EmissionRecord {
                    id: format!("EM-{:06}", self.counter),
                    entity_id: entity_id.to_string(),
                    scope: EmissionScope::Scope2,
                    scope3_category: None,
                    facility_id: Some(e.facility_id.clone()),
                    period: e.period,
                    activity_data: Some(format!("{} kWh", e.consumption_kwh)),
                    activity_unit: Some("kWh".to_string()),
                    emission_factor: Some(factor),
                    co2e_tonnes,
                    estimation_method: EstimationMethod::ActivityBased,
                    source: Some(format!(
                        "Grid average ({})",
                        self.config.scope2.factor_region
                    )),
                }
            })
            .collect()
    }

    // ----- Scope 3: Value chain emissions -----

    /// Generate Scope 3 (Category 1: Purchased Goods) emission records from vendor spend.
    pub fn generate_scope3_purchased_goods(
        &mut self,
        entity_id: &str,
        vendor_spend: &[VendorSpendInput],
        start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> Vec<EmissionRecord> {
        if !self.config.scope3.enabled {
            return Vec::new();
        }

        vendor_spend
            .iter()
            .map(|vs| {
                self.counter += 1;
                let factor = spend_emission_factor(&vs.category, &vs.country);
                let co2e_kg = vs.spend * factor;
                let co2e_tonnes = (co2e_kg / dec!(1000)).round_dp(4);

                EmissionRecord {
                    id: format!("EM-{:06}", self.counter),
                    entity_id: entity_id.to_string(),
                    scope: EmissionScope::Scope3,
                    scope3_category: Some(Scope3Category::PurchasedGoods),
                    facility_id: None,
                    period: start_date,
                    activity_data: Some(format!("{} USD spend ({})", vs.spend, vs.category)),
                    activity_unit: Some("USD".to_string()),
                    emission_factor: Some(factor),
                    co2e_tonnes,
                    estimation_method: EstimationMethod::SpendBased,
                    source: Some(format!("EEIO factors ({})", vs.country)),
                }
            })
            .collect()
    }

    /// Generate Scope 3 (Category 6: Business Travel) from travel spend.
    pub fn generate_scope3_business_travel(
        &mut self,
        entity_id: &str,
        travel_spend: Decimal,
        period: NaiveDate,
    ) -> Vec<EmissionRecord> {
        if !self.config.scope3.enabled || travel_spend <= Decimal::ZERO {
            return Vec::new();
        }

        self.counter += 1;
        // Average emission factor for business travel: ~0.25 kg CO2e / USD
        let factor = dec!(0.25);
        let co2e_kg = travel_spend * factor;
        let co2e_tonnes = (co2e_kg / dec!(1000)).round_dp(4);

        vec![EmissionRecord {
            id: format!("EM-{:06}", self.counter),
            entity_id: entity_id.to_string(),
            scope: EmissionScope::Scope3,
            scope3_category: Some(Scope3Category::BusinessTravel),
            facility_id: None,
            period,
            activity_data: Some(format!("{} USD travel spend", travel_spend)),
            activity_unit: Some("USD".to_string()),
            emission_factor: Some(factor),
            co2e_tonnes,
            estimation_method: EstimationMethod::AverageData,
            source: Some("DEFRA business travel factors".to_string()),
        }]
    }

    /// Generate Scope 3 (Category 7: Employee Commuting) from headcount.
    pub fn generate_scope3_commuting(
        &mut self,
        entity_id: &str,
        headcount: u32,
        period: NaiveDate,
    ) -> Vec<EmissionRecord> {
        if !self.config.scope3.enabled || headcount == 0 {
            return Vec::new();
        }

        self.counter += 1;
        // Average commuting: ~2.5 tonnes CO2e / employee / year → per month
        let annual_per_employee = dec!(2.5);
        let monthly_per_employee = (annual_per_employee / dec!(12)).round_dp(4);
        let co2e_tonnes = (monthly_per_employee * Decimal::from(headcount)).round_dp(4);

        vec![EmissionRecord {
            id: format!("EM-{:06}", self.counter),
            entity_id: entity_id.to_string(),
            scope: EmissionScope::Scope3,
            scope3_category: Some(Scope3Category::EmployeeCommuting),
            facility_id: None,
            period,
            activity_data: Some(format!("{} employees", headcount)),
            activity_unit: Some("headcount".to_string()),
            emission_factor: None,
            co2e_tonnes,
            estimation_method: EstimationMethod::AverageData,
            source: Some("EPA commuting average factors".to_string()),
        }]
    }

    /// Small random variance ±5% for measurement uncertainty.
    fn random_variance(&mut self) -> Decimal {
        let v: f64 = self.rng.random_range(-0.05..0.05);
        Decimal::from_f64_retain(v).unwrap_or(Decimal::ZERO)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn test_scope1_emissions_from_energy() {
        let energy_data = vec![EnergyInput {
            facility_id: "F-001".into(),
            energy_type: EnergyInputType::NaturalGas,
            consumption_kwh: dec!(100000),
            period: d("2025-01-01"),
        }];

        let config = EnvironmentalConfig::default();
        let mut gen = EmissionGenerator::new(config, 42);
        let records = gen.generate_scope1("C001", &energy_data);

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].scope, EmissionScope::Scope1);
        assert!(records[0].co2e_tonnes > Decimal::ZERO);
        assert_eq!(
            records[0].estimation_method,
            EstimationMethod::ActivityBased
        );
        assert!(records[0].facility_id.is_some());
    }

    #[test]
    fn test_scope1_excludes_electricity() {
        let energy_data = vec![
            EnergyInput {
                facility_id: "F-001".into(),
                energy_type: EnergyInputType::Electricity,
                consumption_kwh: dec!(500000),
                period: d("2025-01-01"),
            },
            EnergyInput {
                facility_id: "F-001".into(),
                energy_type: EnergyInputType::NaturalGas,
                consumption_kwh: dec!(100000),
                period: d("2025-01-01"),
            },
        ];

        let config = EnvironmentalConfig::default();
        let mut gen = EmissionGenerator::new(config, 42);
        let records = gen.generate_scope1("C001", &energy_data);

        assert_eq!(
            records.len(),
            1,
            "Electricity should be excluded from Scope 1"
        );
        assert_eq!(records[0].scope, EmissionScope::Scope1);
    }

    #[test]
    fn test_scope2_from_electricity() {
        let energy_data = vec![EnergyInput {
            facility_id: "F-001".into(),
            energy_type: EnergyInputType::Electricity,
            consumption_kwh: dec!(200000),
            period: d("2025-01-01"),
        }];

        let config = EnvironmentalConfig::default();
        let mut gen = EmissionGenerator::new(config, 42);
        let records = gen.generate_scope2("C001", &energy_data);

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].scope, EmissionScope::Scope2);
        assert!(records[0].co2e_tonnes > Decimal::ZERO);
    }

    #[test]
    fn test_scope3_from_vendor_spend() {
        let vendor_spend = vec![
            VendorSpendInput {
                vendor_id: "V-001".into(),
                category: "office_supplies".into(),
                spend: dec!(50000),
                country: "US".into(),
            },
            VendorSpendInput {
                vendor_id: "V-002".into(),
                category: "manufacturing".into(),
                spend: dec!(200000),
                country: "CN".into(),
            },
        ];

        let config = EnvironmentalConfig::default();
        let mut gen = EmissionGenerator::new(config, 42);
        let records = gen.generate_scope3_purchased_goods(
            "C001",
            &vendor_spend,
            d("2025-01-01"),
            d("2025-12-31"),
        );

        assert_eq!(records.len(), 2);
        assert!(records.iter().all(|r| r.scope == EmissionScope::Scope3));
        assert!(records
            .iter()
            .all(|r| r.scope3_category == Some(Scope3Category::PurchasedGoods)));
        // Higher spend + manufacturing + China multiplier → higher emissions
        assert!(records[1].co2e_tonnes > records[0].co2e_tonnes);
    }

    #[test]
    fn test_scope3_business_travel() {
        let config = EnvironmentalConfig::default();
        let mut gen = EmissionGenerator::new(config, 42);
        let records = gen.generate_scope3_business_travel("C001", dec!(100000), d("2025-01-01"));

        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].scope3_category,
            Some(Scope3Category::BusinessTravel)
        );
        assert!(records[0].co2e_tonnes > Decimal::ZERO);
    }

    #[test]
    fn test_scope3_commuting() {
        let config = EnvironmentalConfig::default();
        let mut gen = EmissionGenerator::new(config, 42);
        let records = gen.generate_scope3_commuting("C001", 500, d("2025-06-01"));

        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].scope3_category,
            Some(Scope3Category::EmployeeCommuting)
        );
        // 500 employees × 2.5 t/yr / 12 ≈ 104 tonnes
        assert!(records[0].co2e_tonnes > dec!(100));
        assert!(records[0].co2e_tonnes < dec!(110));
    }

    #[test]
    fn test_disabled_scope_produces_nothing() {
        let mut config = EnvironmentalConfig::default();
        config.scope1.enabled = false;

        let energy_data = vec![EnergyInput {
            facility_id: "F-001".into(),
            energy_type: EnergyInputType::NaturalGas,
            consumption_kwh: dec!(100000),
            period: d("2025-01-01"),
        }];

        let mut gen = EmissionGenerator::new(config, 42);
        let records = gen.generate_scope1("C001", &energy_data);
        assert!(records.is_empty());
    }

    #[test]
    fn test_deterministic_emissions() {
        let energy_data = vec![EnergyInput {
            facility_id: "F-001".into(),
            energy_type: EnergyInputType::Diesel,
            consumption_kwh: dec!(50000),
            period: d("2025-01-01"),
        }];

        let config = EnvironmentalConfig::default();

        let mut gen1 = EmissionGenerator::new(config.clone(), 42);
        let r1 = gen1.generate_scope1("C001", &energy_data);

        let mut gen2 = EmissionGenerator::new(config, 42);
        let r2 = gen2.generate_scope1("C001", &energy_data);

        assert_eq!(r1.len(), r2.len());
        assert_eq!(r1[0].co2e_tonnes, r2[0].co2e_tonnes);
    }

    #[test]
    fn test_zero_spend_scope3() {
        let config = EnvironmentalConfig::default();
        let mut gen = EmissionGenerator::new(config, 42);
        let records = gen.generate_scope3_business_travel("C001", Decimal::ZERO, d("2025-01-01"));
        assert!(records.is_empty());
    }

    #[test]
    fn test_zero_headcount_commuting() {
        let config = EnvironmentalConfig::default();
        let mut gen = EmissionGenerator::new(config, 42);
        let records = gen.generate_scope3_commuting("C001", 0, d("2025-01-01"));
        assert!(records.is_empty());
    }
}
