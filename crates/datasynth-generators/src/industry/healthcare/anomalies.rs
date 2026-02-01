//! Healthcare-specific anomalies.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::super::common::IndustryAnomaly;

/// Healthcare-specific anomaly types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthcareAnomaly {
    /// Coding a higher-paying diagnosis/procedure than documented.
    Upcoding {
        original_code: String,
        upcoded_to: String,
        revenue_impact: Decimal,
        encounter_id: String,
    },
    /// Billing separately for services that should be bundled.
    Unbundling {
        bundled_code: String,
        unbundled_codes: Vec<String>,
        revenue_impact: Decimal,
        encounter_id: String,
    },
    /// Billing for services not rendered.
    PhantomBilling {
        patient_id: String,
        service_not_rendered: String,
        billed_amount: Decimal,
    },
    /// Billing multiple times for same service.
    DuplicateBilling {
        claim_id: String,
        duplicate_claim_id: String,
        amount: Decimal,
    },
    /// Kickback for patient referrals.
    PhysicianReferralKickback {
        referring_physician: String,
        recipient_entity: String,
        kickback_amount: Decimal,
        patient_count: u32,
    },
    /// Medical director paid for no services.
    MedicalDirectorFraud {
        physician_id: String,
        payment_amount: Decimal,
        services_rendered: bool,
    },
    /// Unauthorized access to patient records.
    HipaaViolation {
        patient_id: String,
        accessor_id: String,
        access_reason: String,
        unauthorized: bool,
    },
    /// Medically unnecessary services.
    MedicalNecessityAbuse {
        encounter_id: String,
        unnecessary_services: Vec<String>,
        total_charges: Decimal,
    },
    /// False certification for admission.
    FalseCertification {
        encounter_id: String,
        certification_type: CertificationType,
        certifying_physician: String,
    },
    /// Cost report manipulation.
    CostReportManipulation {
        manipulation_type: CostReportManipulationType,
        inflated_amount: Decimal,
        fiscal_year: u32,
    },
    /// Ambulance services not medically necessary.
    AmbulanceFraud {
        transport_id: String,
        patient_ambulatory: bool,
        billed_amount: Decimal,
    },
    /// Durable medical equipment fraud.
    DmeFraud {
        scheme_type: DmeFraudType,
        equipment_type: String,
        amount: Decimal,
    },
    /// Laboratory fraud (unnecessary tests, kickbacks).
    LabFraud {
        scheme_type: LabFraudType,
        amount: Decimal,
        tests_affected: u32,
    },
}

/// Types of false certifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CertificationType {
    /// Home health certification.
    HomeHealth,
    /// Skilled nursing certification.
    SkilledNursing,
    /// Hospice certification.
    Hospice,
    /// Inpatient admission certification.
    InpatientAdmission,
}

/// Types of cost report manipulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CostReportManipulationType {
    /// Inflated bad debt.
    InflatedBadDebt,
    /// Inflated charity care.
    InflatedCharityCare,
    /// Misallocated costs.
    MisallocatedCosts,
    /// Improper GME costs.
    ImproperGmeCosts,
}

/// Types of DME fraud.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DmeFraudType {
    /// Billing for equipment not provided.
    PhantomBilling,
    /// Billing for higher-cost equipment.
    Upcoding,
    /// Billing rental as purchase.
    RentalAsPurchase,
    /// Kickbacks for referrals.
    ReferralKickback,
}

/// Types of lab fraud.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LabFraudType {
    /// Medically unnecessary tests.
    UnnecessaryTests,
    /// Kickbacks for referrals.
    ReferralKickbacks,
    /// Unbundling lab panels.
    PanelUnbundling,
    /// Upcoding test complexity.
    ComplexityUpcoding,
}

impl IndustryAnomaly for HealthcareAnomaly {
    fn anomaly_type(&self) -> &str {
        match self {
            HealthcareAnomaly::Upcoding { .. } => "upcoding",
            HealthcareAnomaly::Unbundling { .. } => "unbundling",
            HealthcareAnomaly::PhantomBilling { .. } => "phantom_billing",
            HealthcareAnomaly::DuplicateBilling { .. } => "duplicate_billing",
            HealthcareAnomaly::PhysicianReferralKickback { .. } => "physician_referral_kickback",
            HealthcareAnomaly::MedicalDirectorFraud { .. } => "medical_director_fraud",
            HealthcareAnomaly::HipaaViolation { .. } => "hipaa_violation",
            HealthcareAnomaly::MedicalNecessityAbuse { .. } => "medical_necessity_abuse",
            HealthcareAnomaly::FalseCertification { .. } => "false_certification",
            HealthcareAnomaly::CostReportManipulation { .. } => "cost_report_manipulation",
            HealthcareAnomaly::AmbulanceFraud { .. } => "ambulance_fraud",
            HealthcareAnomaly::DmeFraud { .. } => "dme_fraud",
            HealthcareAnomaly::LabFraud { .. } => "lab_fraud",
        }
    }

    fn severity(&self) -> u8 {
        match self {
            HealthcareAnomaly::HipaaViolation { .. } => 3,
            HealthcareAnomaly::DuplicateBilling { .. } => 3,
            HealthcareAnomaly::Upcoding { .. } => 4,
            HealthcareAnomaly::Unbundling { .. } => 4,
            HealthcareAnomaly::MedicalNecessityAbuse { .. } => 4,
            HealthcareAnomaly::AmbulanceFraud { .. } => 4,
            HealthcareAnomaly::DmeFraud { .. } => 4,
            HealthcareAnomaly::LabFraud { .. } => 4,
            HealthcareAnomaly::PhantomBilling { .. } => 5,
            HealthcareAnomaly::PhysicianReferralKickback { .. } => 5,
            HealthcareAnomaly::MedicalDirectorFraud { .. } => 5,
            HealthcareAnomaly::FalseCertification { .. } => 5,
            HealthcareAnomaly::CostReportManipulation { .. } => 5,
        }
    }

    fn detection_difficulty(&self) -> &str {
        match self {
            HealthcareAnomaly::DuplicateBilling { .. } => "easy",
            HealthcareAnomaly::HipaaViolation { .. } => "moderate",
            HealthcareAnomaly::Upcoding { .. } => "moderate",
            HealthcareAnomaly::Unbundling { .. } => "moderate",
            HealthcareAnomaly::MedicalNecessityAbuse { .. } => "hard",
            HealthcareAnomaly::AmbulanceFraud { .. } => "hard",
            HealthcareAnomaly::DmeFraud { .. } => "hard",
            HealthcareAnomaly::LabFraud { .. } => "hard",
            HealthcareAnomaly::PhantomBilling { .. } => "expert",
            HealthcareAnomaly::PhysicianReferralKickback { .. } => "expert",
            HealthcareAnomaly::MedicalDirectorFraud { .. } => "expert",
            HealthcareAnomaly::FalseCertification { .. } => "expert",
            HealthcareAnomaly::CostReportManipulation { .. } => "expert",
        }
    }

    fn indicators(&self) -> Vec<String> {
        match self {
            HealthcareAnomaly::Upcoding { .. } => vec![
                "higher_acuity_than_documented".to_string(),
                "diagnosis_code_pattern_anomaly".to_string(),
                "increased_case_mix_index".to_string(),
            ],
            HealthcareAnomaly::Unbundling { .. } => vec![
                "separate_billing_for_related_services".to_string(),
                "modifier_usage_pattern".to_string(),
                "ncci_edit_bypass".to_string(),
            ],
            HealthcareAnomaly::PhantomBilling { .. } => vec![
                "service_without_clinical_documentation".to_string(),
                "patient_not_present".to_string(),
                "attending_not_available".to_string(),
            ],
            HealthcareAnomaly::PhysicianReferralKickback { .. } => vec![
                "referral_pattern_concentration".to_string(),
                "unusual_compensation_arrangements".to_string(),
                "fair_market_value_deviation".to_string(),
            ],
            HealthcareAnomaly::CostReportManipulation { .. } => vec![
                "cost_allocation_anomaly".to_string(),
                "bad_debt_trend_deviation".to_string(),
                "uncompensated_care_spike".to_string(),
            ],
            _ => vec!["general_healthcare_anomaly".to_string()],
        }
    }

    fn regulatory_concerns(&self) -> Vec<String> {
        match self {
            HealthcareAnomaly::Upcoding { .. }
            | HealthcareAnomaly::Unbundling { .. }
            | HealthcareAnomaly::PhantomBilling { .. } => vec![
                "false_claims_act".to_string(),
                "anti_kickback_statute".to_string(),
                "civil_monetary_penalties".to_string(),
            ],
            HealthcareAnomaly::PhysicianReferralKickback { .. }
            | HealthcareAnomaly::MedicalDirectorFraud { .. } => vec![
                "stark_law".to_string(),
                "anti_kickback_statute".to_string(),
                "false_claims_act".to_string(),
            ],
            HealthcareAnomaly::HipaaViolation { .. } => vec![
                "hipaa_privacy_rule".to_string(),
                "hipaa_security_rule".to_string(),
                "ocr_investigation".to_string(),
            ],
            HealthcareAnomaly::CostReportManipulation { .. } => vec![
                "false_claims_act".to_string(),
                "cms_cost_report_regulations".to_string(),
                "program_exclusion".to_string(),
            ],
            _ => vec![
                "false_claims_act".to_string(),
                "anti_kickback_statute".to_string(),
            ],
        }
    }
}

impl HealthcareAnomaly {
    /// Returns the financial impact of this anomaly.
    pub fn financial_impact(&self) -> Option<Decimal> {
        match self {
            HealthcareAnomaly::Upcoding { revenue_impact, .. } => Some(*revenue_impact),
            HealthcareAnomaly::Unbundling { revenue_impact, .. } => Some(*revenue_impact),
            HealthcareAnomaly::PhantomBilling { billed_amount, .. } => Some(*billed_amount),
            HealthcareAnomaly::DuplicateBilling { amount, .. } => Some(*amount),
            HealthcareAnomaly::PhysicianReferralKickback {
                kickback_amount, ..
            } => Some(*kickback_amount),
            HealthcareAnomaly::MedicalDirectorFraud { payment_amount, .. } => Some(*payment_amount),
            HealthcareAnomaly::MedicalNecessityAbuse { total_charges, .. } => Some(*total_charges),
            HealthcareAnomaly::CostReportManipulation {
                inflated_amount, ..
            } => Some(*inflated_amount),
            HealthcareAnomaly::AmbulanceFraud { billed_amount, .. } => Some(*billed_amount),
            HealthcareAnomaly::DmeFraud { amount, .. } => Some(*amount),
            HealthcareAnomaly::LabFraud { amount, .. } => Some(*amount),
            _ => None,
        }
    }

    /// Returns potential False Claims Act treble damages.
    pub fn potential_fca_liability(&self) -> Option<Decimal> {
        self.financial_impact()
            .map(|impact| impact * Decimal::new(3, 0))
    }

    /// Returns whether this involves a licensed professional.
    pub fn involves_licensed_professional(&self) -> bool {
        matches!(
            self,
            HealthcareAnomaly::PhysicianReferralKickback { .. }
                | HealthcareAnomaly::MedicalDirectorFraud { .. }
                | HealthcareAnomaly::FalseCertification { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upcoding() {
        let anomaly = HealthcareAnomaly::Upcoding {
            original_code: "99213".to_string(),
            upcoded_to: "99215".to_string(),
            revenue_impact: Decimal::new(75, 0),
            encounter_id: "E001".to_string(),
        };

        assert_eq!(anomaly.anomaly_type(), "upcoding");
        assert_eq!(anomaly.severity(), 4);
        assert_eq!(anomaly.detection_difficulty(), "moderate");
        assert_eq!(anomaly.financial_impact(), Some(Decimal::new(75, 0)));
    }

    #[test]
    fn test_phantom_billing() {
        let anomaly = HealthcareAnomaly::PhantomBilling {
            patient_id: "P001".to_string(),
            service_not_rendered: "99215".to_string(),
            billed_amount: Decimal::new(250, 0),
        };

        assert_eq!(anomaly.severity(), 5);
        assert_eq!(anomaly.detection_difficulty(), "expert");
        assert!(anomaly
            .regulatory_concerns()
            .contains(&"false_claims_act".to_string()));
    }

    #[test]
    fn test_kickback() {
        let anomaly = HealthcareAnomaly::PhysicianReferralKickback {
            referring_physician: "DR001".to_string(),
            recipient_entity: "LAB001".to_string(),
            kickback_amount: Decimal::new(10_000, 0),
            patient_count: 50,
        };

        assert!(anomaly.involves_licensed_professional());
        assert!(anomaly
            .regulatory_concerns()
            .contains(&"stark_law".to_string()));
        assert_eq!(
            anomaly.potential_fca_liability(),
            Some(Decimal::new(30_000, 0))
        );
    }

    #[test]
    fn test_hipaa_violation() {
        let anomaly = HealthcareAnomaly::HipaaViolation {
            patient_id: "P001".to_string(),
            accessor_id: "EMP001".to_string(),
            access_reason: "curiosity".to_string(),
            unauthorized: true,
        };

        assert!(anomaly
            .regulatory_concerns()
            .contains(&"hipaa_privacy_rule".to_string()));
        assert_eq!(anomaly.severity(), 3);
    }
}
