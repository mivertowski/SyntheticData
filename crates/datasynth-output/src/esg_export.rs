//! Export ESG / sustainability data to CSV files.
//!
//! Exports emission records, energy consumption, water usage, waste records,
//! workforce diversity, pay equity, safety incidents/metrics, governance,
//! supplier ESG assessments, disclosures, materiality, climate scenarios,
//! and ESG anomaly labels.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use datasynth_core::error::SynthResult;
use datasynth_core::models::{
    ClimateScenario, EmissionRecord, EnergyConsumption, EsgDisclosure, GovernanceMetric,
    MaterialityAssessment, PayEquityMetric, SafetyIncident, SafetyMetric, SupplierEsgAssessment,
    WasteRecord, WaterUsage, WorkforceDiversityMetric,
};

// Re-export the anomaly label type from the generators crate
// (used via the esg_anomaly_labels export method which takes a generic vec)

// ---------------------------------------------------------------------------
// Export summary
// ---------------------------------------------------------------------------

/// Summary of exported ESG data.
#[derive(Debug, Default)]
pub struct EsgExportSummary {
    pub emission_records: usize,
    pub energy_consumption: usize,
    pub water_usage: usize,
    pub waste_records: usize,
    pub workforce_diversity: usize,
    pub pay_equity: usize,
    pub safety_incidents: usize,
    pub safety_metrics: usize,
    pub governance_metrics: usize,
    pub supplier_assessments: usize,
    pub disclosures: usize,
    pub materiality_assessments: usize,
    pub climate_scenarios: usize,
}

impl EsgExportSummary {
    /// Total number of rows exported across all files.
    pub fn total(&self) -> usize {
        self.emission_records
            + self.energy_consumption
            + self.water_usage
            + self.waste_records
            + self.workforce_diversity
            + self.pay_equity
            + self.safety_incidents
            + self.safety_metrics
            + self.governance_metrics
            + self.supplier_assessments
            + self.disclosures
            + self.materiality_assessments
            + self.climate_scenarios
    }
}

// ---------------------------------------------------------------------------
// Exporter
// ---------------------------------------------------------------------------

/// Exporter for ESG / sustainability data.
pub struct EsgExporter {
    output_dir: PathBuf,
}

impl EsgExporter {
    /// Create a new ESG exporter writing to the given directory.
    pub fn new(output_dir: impl AsRef<Path>) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    /// Export emission records to `emission_records.csv`.
    pub fn export_emissions(&self, data: &[EmissionRecord]) -> SynthResult<usize> {
        let path = self.output_dir.join("emission_records.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            w,
            "id,entity_id,scope,scope3_category,facility_id,period,activity_data,activity_unit,emission_factor,co2e_tonnes,estimation_method,source"
        )?;

        for r in data {
            writeln!(
                w,
                "{},{},{:?},{},{},{},{},{},{},{},{:?},{}",
                r.id,
                r.entity_id,
                r.scope,
                opt_debug(&r.scope3_category),
                r.facility_id.as_deref().unwrap_or(""),
                r.period,
                esc(r.activity_data.as_deref().unwrap_or("")),
                r.activity_unit.as_deref().unwrap_or(""),
                opt_dec(&r.emission_factor),
                r.co2e_tonnes,
                r.estimation_method,
                esc(r.source.as_deref().unwrap_or("")),
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export energy consumption to `energy_consumption.csv`.
    pub fn export_energy(&self, data: &[EnergyConsumption]) -> SynthResult<usize> {
        let path = self.output_dir.join("energy_consumption.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            w,
            "id,entity_id,facility_id,period,energy_source,consumption_kwh,cost,currency,is_renewable"
        )?;

        for r in data {
            writeln!(
                w,
                "{},{},{},{},{:?},{},{},{},{}",
                r.id,
                r.entity_id,
                r.facility_id,
                r.period,
                r.energy_source,
                r.consumption_kwh,
                r.cost,
                r.currency,
                r.is_renewable,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export water usage to `water_usage.csv`.
    pub fn export_water(&self, data: &[WaterUsage]) -> SynthResult<usize> {
        let path = self.output_dir.join("water_usage.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            w,
            "id,entity_id,facility_id,period,source,withdrawal_m3,discharge_m3,consumption_m3,is_water_stressed_area"
        )?;

        for r in data {
            writeln!(
                w,
                "{},{},{},{},{:?},{},{},{},{}",
                r.id,
                r.entity_id,
                r.facility_id,
                r.period,
                r.source,
                r.withdrawal_m3,
                r.discharge_m3,
                r.consumption_m3,
                r.is_water_stressed_area,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export waste records to `waste_records.csv`.
    pub fn export_waste(&self, data: &[WasteRecord]) -> SynthResult<usize> {
        let path = self.output_dir.join("waste_records.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            w,
            "id,entity_id,facility_id,period,waste_type,disposal_method,quantity_tonnes,is_diverted_from_landfill"
        )?;

        for r in data {
            writeln!(
                w,
                "{},{},{},{},{:?},{:?},{},{}",
                r.id,
                r.entity_id,
                r.facility_id,
                r.period,
                r.waste_type,
                r.disposal_method,
                r.quantity_tonnes,
                r.is_diverted_from_landfill,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export workforce diversity to `workforce_diversity.csv`.
    pub fn export_diversity(&self, data: &[WorkforceDiversityMetric]) -> SynthResult<usize> {
        let path = self.output_dir.join("workforce_diversity.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            w,
            "id,entity_id,period,dimension,level,category,headcount,total_headcount,percentage"
        )?;

        for r in data {
            writeln!(
                w,
                "{},{},{},{:?},{:?},{},{},{},{}",
                r.id,
                r.entity_id,
                r.period,
                r.dimension,
                r.level,
                esc(&r.category),
                r.headcount,
                r.total_headcount,
                r.percentage,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export pay equity metrics to `pay_equity_metrics.csv`.
    pub fn export_pay_equity(&self, data: &[PayEquityMetric]) -> SynthResult<usize> {
        let path = self.output_dir.join("pay_equity_metrics.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            w,
            "id,entity_id,period,dimension,reference_group,comparison_group,reference_median_salary,comparison_median_salary,pay_gap_ratio,sample_size"
        )?;

        for r in data {
            writeln!(
                w,
                "{},{},{},{:?},{},{},{},{},{},{}",
                r.id,
                r.entity_id,
                r.period,
                r.dimension,
                r.reference_group,
                r.comparison_group,
                r.reference_median_salary,
                r.comparison_median_salary,
                r.pay_gap_ratio,
                r.sample_size,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export safety incidents to `safety_incidents.csv`.
    pub fn export_safety_incidents(&self, data: &[SafetyIncident]) -> SynthResult<usize> {
        let path = self.output_dir.join("safety_incidents.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            w,
            "id,entity_id,facility_id,date,incident_type,days_away,is_recordable,description"
        )?;

        for r in data {
            writeln!(
                w,
                "{},{},{},{},{:?},{},{},{}",
                r.id,
                r.entity_id,
                r.facility_id,
                r.date,
                r.incident_type,
                r.days_away,
                r.is_recordable,
                esc(&r.description),
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export safety metrics to `safety_metrics.csv`.
    pub fn export_safety_metrics(&self, data: &[SafetyMetric]) -> SynthResult<usize> {
        let path = self.output_dir.join("safety_metrics.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            w,
            "id,entity_id,period,total_hours_worked,recordable_incidents,lost_time_incidents,days_away,near_misses,fatalities,trir,ltir,dart_rate"
        )?;

        for r in data {
            writeln!(
                w,
                "{},{},{},{},{},{},{},{},{},{},{},{}",
                r.id,
                r.entity_id,
                r.period,
                r.total_hours_worked,
                r.recordable_incidents,
                r.lost_time_incidents,
                r.days_away,
                r.near_misses,
                r.fatalities,
                r.trir,
                r.ltir,
                r.dart_rate,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export governance metrics to `governance_metrics.csv`.
    pub fn export_governance(&self, data: &[GovernanceMetric]) -> SynthResult<usize> {
        let path = self.output_dir.join("governance_metrics.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            w,
            "id,entity_id,period,board_size,independent_directors,female_directors,board_independence_ratio,board_gender_diversity_ratio,ethics_training_completion_pct,whistleblower_reports,anti_corruption_violations"
        )?;

        for r in data {
            writeln!(
                w,
                "{},{},{},{},{},{},{},{},{},{},{}",
                r.id,
                r.entity_id,
                r.period,
                r.board_size,
                r.independent_directors,
                r.female_directors,
                r.board_independence_ratio,
                r.board_gender_diversity_ratio,
                r.ethics_training_completion_pct,
                r.whistleblower_reports,
                r.anti_corruption_violations,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export supplier ESG assessments to `supplier_esg_assessments.csv`.
    pub fn export_supplier_assessments(
        &self,
        data: &[SupplierEsgAssessment],
    ) -> SynthResult<usize> {
        let path = self.output_dir.join("supplier_esg_assessments.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            w,
            "id,entity_id,vendor_id,assessment_date,method,environmental_score,social_score,governance_score,overall_score,risk_flag,corrective_actions_required"
        )?;

        for r in data {
            writeln!(
                w,
                "{},{},{},{},{:?},{},{},{},{},{:?},{}",
                r.id,
                r.entity_id,
                r.vendor_id,
                r.assessment_date,
                r.method,
                r.environmental_score,
                r.social_score,
                r.governance_score,
                r.overall_score,
                r.risk_flag,
                r.corrective_actions_required,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export ESG disclosures to `esg_disclosures.csv`.
    pub fn export_disclosures(&self, data: &[EsgDisclosure]) -> SynthResult<usize> {
        let path = self.output_dir.join("esg_disclosures.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            w,
            "id,entity_id,reporting_period_start,reporting_period_end,framework,assurance_level,disclosure_topic,metric_value,metric_unit,is_assured"
        )?;

        for r in data {
            writeln!(
                w,
                "{},{},{},{},{:?},{:?},{},{},{},{}",
                r.id,
                r.entity_id,
                r.reporting_period_start,
                r.reporting_period_end,
                r.framework,
                r.assurance_level,
                esc(&r.disclosure_topic),
                esc(&r.metric_value),
                esc(&r.metric_unit),
                r.is_assured,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export materiality assessments to `materiality_assessments.csv`.
    pub fn export_materiality(&self, data: &[MaterialityAssessment]) -> SynthResult<usize> {
        let path = self.output_dir.join("materiality_assessments.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            w,
            "id,entity_id,period,topic,impact_score,financial_score,combined_score,is_material"
        )?;

        for r in data {
            writeln!(
                w,
                "{},{},{},{},{},{},{},{}",
                r.id,
                r.entity_id,
                r.period,
                esc(&r.topic),
                r.impact_score,
                r.financial_score,
                r.combined_score,
                r.is_material,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export climate scenarios to `climate_scenarios.csv`.
    pub fn export_climate_scenarios(&self, data: &[ClimateScenario]) -> SynthResult<usize> {
        let path = self.output_dir.join("climate_scenarios.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::with_capacity(256 * 1024, file);

        writeln!(
            w,
            "id,entity_id,scenario_type,time_horizon,description,temperature_rise_c,transition_risk_impact,physical_risk_impact,financial_impact"
        )?;

        for r in data {
            writeln!(
                w,
                "{},{},{:?},{:?},{},{},{},{},{}",
                r.id,
                r.entity_id,
                r.scenario_type,
                r.time_horizon,
                esc(&r.description),
                r.temperature_rise_c,
                r.transition_risk_impact,
                r.physical_risk_impact,
                r.financial_impact,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export all ESG data at once.
    #[allow(clippy::too_many_arguments)]
    pub fn export_all(
        &self,
        emissions: &[EmissionRecord],
        energy: &[EnergyConsumption],
        water: &[WaterUsage],
        waste: &[WasteRecord],
        diversity: &[WorkforceDiversityMetric],
        pay_equity: &[PayEquityMetric],
        incidents: &[SafetyIncident],
        safety_metrics: &[SafetyMetric],
        governance: &[GovernanceMetric],
        supplier_assessments: &[SupplierEsgAssessment],
        disclosures: &[EsgDisclosure],
        materiality: &[MaterialityAssessment],
        climate_scenarios: &[ClimateScenario],
    ) -> SynthResult<EsgExportSummary> {
        std::fs::create_dir_all(&self.output_dir)?;

        Ok(EsgExportSummary {
            emission_records: self.export_emissions(emissions)?,
            energy_consumption: self.export_energy(energy)?,
            water_usage: self.export_water(water)?,
            waste_records: self.export_waste(waste)?,
            workforce_diversity: self.export_diversity(diversity)?,
            pay_equity: self.export_pay_equity(pay_equity)?,
            safety_incidents: self.export_safety_incidents(incidents)?,
            safety_metrics: self.export_safety_metrics(safety_metrics)?,
            governance_metrics: self.export_governance(governance)?,
            supplier_assessments: self.export_supplier_assessments(supplier_assessments)?,
            disclosures: self.export_disclosures(disclosures)?,
            materiality_assessments: self.export_materiality(materiality)?,
            climate_scenarios: self.export_climate_scenarios(climate_scenarios)?,
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn esc(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn opt_debug<T: std::fmt::Debug>(opt: &Option<T>) -> String {
    match opt {
        Some(v) => format!("{v:?}"),
        None => String::new(),
    }
}

fn opt_dec(opt: &Option<rust_decimal::Decimal>) -> String {
    match opt {
        Some(v) => v.to_string(),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::*;
    use rust_decimal_macros::dec;
    use tempfile::TempDir;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn test_export_emissions_csv() {
        let dir = TempDir::new().unwrap();
        let exporter = EsgExporter::new(dir.path());

        let emissions = vec![EmissionRecord {
            id: "EM-001".into(),
            entity_id: "C001".into(),
            scope: EmissionScope::Scope1,
            scope3_category: None,
            facility_id: Some("FAC-001".into()),
            period: d("2025-01-01"),
            activity_data: Some("100000 kWh".into()),
            activity_unit: Some("kWh".into()),
            emission_factor: Some(dec!(0.181)),
            co2e_tonnes: dec!(18.1),
            estimation_method: EstimationMethod::ActivityBased,
            source: Some("EPA".into()),
        }];

        let count = exporter.export_emissions(&emissions).unwrap();
        assert_eq!(count, 1);

        let content = std::fs::read_to_string(dir.path().join("emission_records.csv")).unwrap();
        assert!(content.contains("EM-001"));
        assert!(content.contains("Scope1"));
    }

    #[test]
    fn test_export_energy_csv() {
        let dir = TempDir::new().unwrap();
        let exporter = EsgExporter::new(dir.path());

        let energy = vec![EnergyConsumption {
            id: "EN-001".into(),
            entity_id: "C001".into(),
            facility_id: "FAC-001".into(),
            period: d("2025-01-01"),
            energy_source: EnergySourceType::Electricity,
            consumption_kwh: dec!(150000),
            cost: dec!(18000),
            currency: "USD".into(),
            is_renewable: false,
        }];

        let count = exporter.export_energy(&energy).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_export_all_creates_files() {
        let dir = TempDir::new().unwrap();
        let exporter = EsgExporter::new(dir.path());

        let summary = exporter
            .export_all(
                &[],
                &[],
                &[],
                &[],
                &[],
                &[],
                &[],
                &[],
                &[],
                &[],
                &[],
                &[],
                &[],
            )
            .unwrap();

        assert_eq!(summary.total(), 0);

        // All 13 files should exist (with headers only)
        let expected_files = [
            "emission_records.csv",
            "energy_consumption.csv",
            "water_usage.csv",
            "waste_records.csv",
            "workforce_diversity.csv",
            "pay_equity_metrics.csv",
            "safety_incidents.csv",
            "safety_metrics.csv",
            "governance_metrics.csv",
            "supplier_esg_assessments.csv",
            "esg_disclosures.csv",
            "materiality_assessments.csv",
            "climate_scenarios.csv",
        ];

        for f in &expected_files {
            assert!(dir.path().join(f).exists(), "Expected file {} to exist", f);
        }
    }

    #[test]
    fn test_export_summary_total() {
        let summary = EsgExportSummary {
            emission_records: 10,
            energy_consumption: 5,
            water_usage: 3,
            waste_records: 8,
            workforce_diversity: 20,
            pay_equity: 4,
            safety_incidents: 15,
            safety_metrics: 1,
            governance_metrics: 1,
            supplier_assessments: 12,
            disclosures: 18,
            materiality_assessments: 12,
            climate_scenarios: 12,
        };
        assert_eq!(summary.total(), 121);
    }
}
