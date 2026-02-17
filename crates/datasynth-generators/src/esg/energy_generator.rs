//! Energy consumption generator — creates facility-level energy records.
//!
//! Generates monthly energy consumption data per facility with a mix of
//! renewable and non-renewable sources. For manufacturing entities, consumption
//! can be correlated with production volume.

use chrono::{Datelike, NaiveDate};
use datasynth_config::schema::EnergySchemaConfig;
use datasynth_core::models::{EnergyConsumption, EnergySourceType, WasteRecord, WasteType, DisposalMethod, WaterUsage, WaterSource};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Generates [`EnergyConsumption`] records for facilities.
pub struct EnergyGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    config: EnergySchemaConfig,
    counter: u64,
}

impl EnergyGenerator {
    /// Create a new energy generator.
    pub fn new(seed: u64, config: EnergySchemaConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::Esg),
            config,
            counter: 0,
        }
    }

    /// Generate monthly energy consumption records for an entity's facilities.
    ///
    /// Produces one record per energy source per facility per month.
    pub fn generate(
        &mut self,
        entity_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Vec<EnergyConsumption> {
        if !self.config.enabled {
            return Vec::new();
        }

        let mut records = Vec::new();
        let facility_count = self.config.facility_count.max(1);

        // Generate facility IDs
        let facilities: Vec<String> = (1..=facility_count)
            .map(|i| format!("FAC-{:03}", i))
            .collect();

        // Determine which sources each facility uses
        for facility_id in &facilities {
            let mut month = start_date;
            while month <= end_date {
                // Each facility has 2-4 energy sources
                let sources = self.pick_sources();
                for (source, base_kwh) in &sources {
                    self.counter += 1;

                    // Seasonal variation: higher in winter/summer (HVAC)
                    let seasonal = self.seasonal_factor(month);
                    let variance: f64 = self.rng.gen_range(0.85..1.15);
                    let consumption_kwh = (*base_kwh
                        * Decimal::from_f64_retain(seasonal).unwrap_or(dec!(1))
                        * Decimal::from_f64_retain(variance).unwrap_or(dec!(1)))
                    .round_dp(2);

                    // Cost per kWh varies by source
                    let cost_per_kwh = self.cost_per_kwh(*source);
                    let cost = (consumption_kwh * cost_per_kwh).round_dp(2);

                    records.push(EnergyConsumption {
                        id: format!("EN-{:06}", self.counter),
                        entity_id: entity_id.to_string(),
                        facility_id: facility_id.clone(),
                        period: month,
                        energy_source: *source,
                        consumption_kwh,
                        cost,
                        currency: "USD".to_string(),
                        is_renewable: source.is_renewable(),
                    });
                }

                // Advance to next month
                month = next_month(month);
            }
        }

        records
    }

    /// Pick a mix of energy sources with base monthly consumption.
    fn pick_sources(&mut self) -> Vec<(EnergySourceType, Decimal)> {
        let mut sources = Vec::new();

        // Electricity is always present (base: 50k–200k kWh/month)
        let elec_base: f64 = self.rng.gen_range(50_000.0..200_000.0);
        sources.push((
            EnergySourceType::Electricity,
            Decimal::from_f64_retain(elec_base).unwrap_or(dec!(100000)),
        ));

        // Natural gas (80% of facilities)
        if self.rng.gen::<f64>() < 0.80 {
            let gas_base: f64 = self.rng.gen_range(20_000.0..100_000.0);
            sources.push((
                EnergySourceType::NaturalGas,
                Decimal::from_f64_retain(gas_base).unwrap_or(dec!(50000)),
            ));
        }

        // Renewable based on target
        if self.rng.gen::<f64>() < self.config.renewable_target {
            let renewable_type = if self.rng.gen::<f64>() < 0.6 {
                EnergySourceType::SolarPv
            } else {
                EnergySourceType::WindOnshore
            };
            let renewable_base: f64 = self.rng.gen_range(10_000.0..80_000.0);
            sources.push((
                renewable_type,
                Decimal::from_f64_retain(renewable_base).unwrap_or(dec!(30000)),
            ));
        }

        // Diesel (20% of facilities — backup generators, fleet)
        if self.rng.gen::<f64>() < 0.20 {
            let diesel_base: f64 = self.rng.gen_range(5_000.0..30_000.0);
            sources.push((
                EnergySourceType::Diesel,
                Decimal::from_f64_retain(diesel_base).unwrap_or(dec!(10000)),
            ));
        }

        sources
    }

    /// Seasonal multiplier: higher in winter (Jan/Feb/Dec) and summer (Jul/Aug).
    fn seasonal_factor(&self, date: NaiveDate) -> f64 {
        match date.month() {
            1 | 2 | 12 => 1.20,  // Winter heating
            7 | 8 => 1.15,       // Summer cooling
            6 | 9 => 1.05,
            _ => 1.00,
        }
    }

    /// Cost per kWh by energy source.
    fn cost_per_kwh(&self, source: EnergySourceType) -> Decimal {
        match source {
            EnergySourceType::Electricity => dec!(0.12),
            EnergySourceType::NaturalGas => dec!(0.04),
            EnergySourceType::Diesel => dec!(0.15),
            EnergySourceType::Coal => dec!(0.03),
            EnergySourceType::SolarPv => dec!(0.06),
            EnergySourceType::WindOnshore => dec!(0.05),
            EnergySourceType::Biomass => dec!(0.07),
            EnergySourceType::Geothermal => dec!(0.08),
        }
    }
}

/// Generates [`WaterUsage`] records for facilities.
pub struct WaterGenerator {
    rng: ChaCha8Rng,
    counter: u64,
    facility_count: u32,
}

impl WaterGenerator {
    /// Create a new water generator.
    pub fn new(seed: u64, facility_count: u32) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            counter: 0,
            facility_count: facility_count.max(1),
        }
    }

    /// Generate monthly water usage records.
    pub fn generate(
        &mut self,
        entity_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Vec<WaterUsage> {
        let mut records = Vec::new();

        for fac in 1..=self.facility_count {
            let facility_id = format!("FAC-{:03}", fac);
            let is_stressed = self.rng.gen::<f64>() < 0.15; // 15% in water-stressed areas
            let mut month = start_date;

            while month <= end_date {
                self.counter += 1;

                let source = self.pick_water_source();
                let withdrawal: f64 = self.rng.gen_range(500.0..5000.0);
                let discharge_pct: f64 = self.rng.gen_range(0.50..0.85);
                let withdrawal_m3 = Decimal::from_f64_retain(withdrawal).unwrap_or(dec!(2000));
                let discharge_m3 = (withdrawal_m3
                    * Decimal::from_f64_retain(discharge_pct).unwrap_or(dec!(0.7)))
                .round_dp(2);
                let consumption_m3 = (withdrawal_m3 - discharge_m3).round_dp(2);

                records.push(WaterUsage {
                    id: format!("WA-{:06}", self.counter),
                    entity_id: entity_id.to_string(),
                    facility_id: facility_id.clone(),
                    period: month,
                    source,
                    withdrawal_m3,
                    discharge_m3,
                    consumption_m3,
                    is_water_stressed_area: is_stressed,
                });

                month = next_month(month);
            }
        }

        records
    }

    fn pick_water_source(&mut self) -> WaterSource {
        let roll: f64 = self.rng.gen::<f64>();
        if roll < 0.50 {
            WaterSource::Municipal
        } else if roll < 0.70 {
            WaterSource::Groundwater
        } else if roll < 0.85 {
            WaterSource::SurfaceWater
        } else if roll < 0.95 {
            WaterSource::Recycled
        } else {
            WaterSource::Rainwater
        }
    }
}

/// Generates [`WasteRecord`] entries for facilities.
pub struct WasteGenerator {
    rng: ChaCha8Rng,
    counter: u64,
    diversion_target: f64,
    facility_count: u32,
}

impl WasteGenerator {
    /// Create a new waste generator.
    pub fn new(seed: u64, diversion_target: f64, facility_count: u32) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            counter: 0,
            diversion_target,
            facility_count: facility_count.max(1),
        }
    }

    /// Generate monthly waste records.
    pub fn generate(
        &mut self,
        entity_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Vec<WasteRecord> {
        let mut records = Vec::new();

        for fac in 1..=self.facility_count {
            let facility_id = format!("FAC-{:03}", fac);
            let mut month = start_date;

            while month <= end_date {
                // Each facility generates 2-4 waste streams
                let stream_count = self.rng.gen_range(2u32..=4);
                for _ in 0..stream_count {
                    self.counter += 1;
                    let waste_type = self.pick_waste_type();
                    let disposal = self.pick_disposal();
                    let quantity: f64 = self.rng.gen_range(5.0..200.0);

                    records.push(WasteRecord {
                        id: format!("WS-{:06}", self.counter),
                        entity_id: entity_id.to_string(),
                        facility_id: facility_id.clone(),
                        period: month,
                        waste_type,
                        disposal_method: disposal,
                        quantity_tonnes: Decimal::from_f64_retain(quantity)
                            .unwrap_or(dec!(50))
                            .round_dp(2),
                        is_diverted_from_landfill: !matches!(
                            disposal,
                            DisposalMethod::Landfill | DisposalMethod::Incinerated
                        ),
                    });
                }

                month = next_month(month);
            }
        }

        records
    }

    fn pick_waste_type(&mut self) -> WasteType {
        let roll: f64 = self.rng.gen::<f64>();
        if roll < 0.45 {
            WasteType::General
        } else if roll < 0.60 {
            WasteType::Organic
        } else if roll < 0.75 {
            WasteType::Construction
        } else if roll < 0.90 {
            WasteType::Electronic
        } else {
            WasteType::Hazardous
        }
    }

    fn pick_disposal(&mut self) -> DisposalMethod {
        // Bias toward diversion target
        if self.rng.gen::<f64>() < self.diversion_target {
            let roll: f64 = self.rng.gen::<f64>();
            if roll < 0.50 {
                DisposalMethod::Recycled
            } else if roll < 0.80 {
                DisposalMethod::Composted
            } else {
                DisposalMethod::Reused
            }
        } else if self.rng.gen::<f64>() < 0.70 {
            DisposalMethod::Landfill
        } else {
            DisposalMethod::Incinerated
        }
    }
}

/// Advance to the first of the next month.
fn next_month(date: NaiveDate) -> NaiveDate {
    if date.month() == 12 {
        NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap_or(date)
    } else {
        NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1).unwrap_or(date)
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
    fn test_energy_generation_basic() {
        let config = EnergySchemaConfig {
            enabled: true,
            facility_count: 2,
            renewable_target: 0.50,
        };
        let mut gen = EnergyGenerator::new(42, config);
        let records = gen.generate("C001", d("2025-01-01"), d("2025-03-01"));

        assert!(!records.is_empty());
        assert!(records.iter().all(|r| r.entity_id == "C001"));
        assert!(records.iter().all(|r| r.consumption_kwh > Decimal::ZERO));
        assert!(records.iter().all(|r| r.cost > Decimal::ZERO));
    }

    #[test]
    fn test_energy_renewable_vs_nonrenewable() {
        let config = EnergySchemaConfig {
            enabled: true,
            facility_count: 3,
            renewable_target: 1.0, // Force renewables
        };
        let mut gen = EnergyGenerator::new(42, config);
        let records = gen.generate("C001", d("2025-01-01"), d("2025-01-01"));

        let renewable: Decimal = records
            .iter()
            .filter(|r| r.is_renewable)
            .map(|r| r.consumption_kwh)
            .sum();
        let total: Decimal = records.iter().map(|r| r.consumption_kwh).sum();

        assert!(renewable > Decimal::ZERO, "Should have some renewable energy");
        assert!(total > renewable, "Total should include non-renewable too");
    }

    #[test]
    fn test_energy_disabled() {
        let config = EnergySchemaConfig {
            enabled: false,
            facility_count: 5,
            renewable_target: 0.50,
        };
        let mut gen = EnergyGenerator::new(42, config);
        let records = gen.generate("C001", d("2025-01-01"), d("2025-12-01"));
        assert!(records.is_empty());
    }

    #[test]
    fn test_water_generation() {
        let mut gen = WaterGenerator::new(42, 2);
        let records = gen.generate("C001", d("2025-01-01"), d("2025-03-01"));

        assert!(!records.is_empty());
        for r in &records {
            assert!(r.withdrawal_m3 >= r.discharge_m3);
            assert_eq!(r.consumption_m3, (r.withdrawal_m3 - r.discharge_m3).round_dp(2));
        }
    }

    #[test]
    fn test_waste_generation() {
        let mut gen = WasteGenerator::new(42, 0.50, 2);
        let records = gen.generate("C001", d("2025-01-01"), d("2025-03-01"));

        assert!(!records.is_empty());
        for r in &records {
            assert!(r.quantity_tonnes > Decimal::ZERO);
            // Verify diversion flag matches disposal method
            assert_eq!(r.is_diverted_from_landfill, r.computed_diversion());
        }
    }

    #[test]
    fn test_waste_diversion_target() {
        // High diversion target → most waste should be diverted
        let mut gen = WasteGenerator::new(42, 0.90, 3);
        let records = gen.generate("C001", d("2025-01-01"), d("2025-06-01"));

        let diverted = records.iter().filter(|r| r.is_diverted_from_landfill).count();
        let pct = diverted as f64 / records.len() as f64;
        assert!(
            pct > 0.50,
            "High diversion target should result in >50% diversion, got {:.0}%",
            pct * 100.0
        );
    }
}
