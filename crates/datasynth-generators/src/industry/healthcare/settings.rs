//! Healthcare industry settings and configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Healthcare facility type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FacilityType {
    /// Acute care hospital.
    Hospital,
    /// Ambulatory surgery center.
    AmbulatorySurgery,
    /// Physician practice.
    PhysicianPractice,
    /// Skilled nursing facility.
    SkilledNursing,
    /// Home health agency.
    HomeHealth,
    /// Durable medical equipment supplier.
    Dme,
    /// Clinical laboratory.
    Laboratory,
    /// Imaging center.
    ImagingCenter,
}

impl FacilityType {
    /// Returns the CMS provider type code.
    pub fn provider_type_code(&self) -> &'static str {
        match self {
            FacilityType::Hospital => "01",
            FacilityType::AmbulatorySurgery => "21",
            FacilityType::PhysicianPractice => "11",
            FacilityType::SkilledNursing => "31",
            FacilityType::HomeHealth => "34",
            FacilityType::Dme => "09",
            FacilityType::Laboratory => "81",
            FacilityType::ImagingCenter => "22",
        }
    }
}

/// Healthcare industry settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthcareSettings {
    /// Facility type.
    pub facility_type: FacilityType,
    /// Payer mix distribution.
    pub payer_mix: HashMap<String, f64>,
    /// Coding systems enabled.
    pub coding_systems: CodingSystemSettings,
    /// Compliance settings.
    pub compliance: HealthcareCompliance,
    /// Average daily encounters.
    pub avg_daily_encounters: u32,
    /// Average charges per encounter.
    pub avg_charges_per_encounter: u32,
    /// Denial rate (0.0-1.0).
    pub denial_rate: f64,
    /// Bad debt rate (0.0-1.0).
    pub bad_debt_rate: f64,
    /// Charity care rate (0.0-1.0).
    pub charity_care_rate: f64,
    /// Anomaly injection rates.
    #[serde(default)]
    pub anomaly_rates: HealthcareAnomalyRates,
}

/// Coding system settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingSystemSettings {
    /// ICD-10 diagnosis coding.
    pub icd10: bool,
    /// CPT procedure coding.
    pub cpt: bool,
    /// DRG grouping.
    pub drg: bool,
    /// HCPCS Level II coding.
    pub hcpcs: bool,
    /// Revenue codes.
    pub revenue_codes: bool,
}

impl Default for CodingSystemSettings {
    fn default() -> Self {
        Self {
            icd10: true,
            cpt: true,
            drg: true,
            hcpcs: true,
            revenue_codes: true,
        }
    }
}

/// Healthcare compliance settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthcareCompliance {
    /// HIPAA compliance.
    pub hipaa: bool,
    /// Stark Law compliance.
    pub stark_law: bool,
    /// Anti-Kickback Statute compliance.
    pub anti_kickback: bool,
    /// False Claims Act compliance.
    pub false_claims_act: bool,
    /// EMTALA compliance (for hospitals).
    pub emtala: bool,
}

impl Default for HealthcareCompliance {
    fn default() -> Self {
        Self {
            hipaa: true,
            stark_law: true,
            anti_kickback: true,
            false_claims_act: true,
            emtala: true,
        }
    }
}

/// Healthcare anomaly injection rates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthcareAnomalyRates {
    /// Upcoding rate.
    pub upcoding: f64,
    /// Unbundling rate.
    pub unbundling: f64,
    /// Phantom billing rate.
    pub phantom_billing: f64,
    /// Kickback rate.
    pub kickbacks: f64,
    /// Duplicate billing rate.
    pub duplicate_billing: f64,
    /// Medical necessity abuse rate.
    pub medical_necessity_abuse: f64,
}

impl Default for HealthcareAnomalyRates {
    fn default() -> Self {
        Self {
            upcoding: 0.02,
            unbundling: 0.015,
            phantom_billing: 0.005,
            kickbacks: 0.003,
            duplicate_billing: 0.008,
            medical_necessity_abuse: 0.01,
        }
    }
}

impl Default for HealthcareSettings {
    fn default() -> Self {
        let mut payer_mix = HashMap::new();
        payer_mix.insert("medicare".to_string(), 0.40);
        payer_mix.insert("medicaid".to_string(), 0.20);
        payer_mix.insert("commercial".to_string(), 0.30);
        payer_mix.insert("self_pay".to_string(), 0.10);

        Self {
            facility_type: FacilityType::Hospital,
            payer_mix,
            coding_systems: CodingSystemSettings::default(),
            compliance: HealthcareCompliance::default(),
            avg_daily_encounters: 150,
            avg_charges_per_encounter: 8,
            denial_rate: 0.05,
            bad_debt_rate: 0.03,
            charity_care_rate: 0.02,
            anomaly_rates: HealthcareAnomalyRates::default(),
        }
    }
}

impl HealthcareSettings {
    /// Creates settings for a specific facility type.
    #[allow(clippy::field_reassign_with_default)]
    pub fn for_facility(facility_type: FacilityType) -> Self {
        let mut settings = Self::default();
        settings.facility_type = facility_type;

        // Adjust based on facility type
        match facility_type {
            FacilityType::PhysicianPractice => {
                settings.avg_daily_encounters = 30;
                settings.avg_charges_per_encounter = 3;
            }
            FacilityType::AmbulatorySurgery => {
                settings.avg_daily_encounters = 20;
                settings.avg_charges_per_encounter = 15;
            }
            FacilityType::SkilledNursing => {
                settings.avg_daily_encounters = 100;
                settings.avg_charges_per_encounter = 5;
            }
            _ => {}
        }

        settings
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_facility_type() {
        let hospital = FacilityType::Hospital;
        assert_eq!(hospital.provider_type_code(), "01");

        let physician = FacilityType::PhysicianPractice;
        assert_eq!(physician.provider_type_code(), "11");
    }

    #[test]
    fn test_healthcare_settings() {
        let settings = HealthcareSettings::default();

        assert_eq!(settings.facility_type, FacilityType::Hospital);
        assert!(settings.payer_mix.len() >= 4);
        assert!(settings.compliance.hipaa);
    }

    #[test]
    fn test_facility_specific_settings() {
        let settings = HealthcareSettings::for_facility(FacilityType::PhysicianPractice);

        assert_eq!(settings.avg_daily_encounters, 30);
        assert_eq!(settings.avg_charges_per_encounter, 3);
    }
}
