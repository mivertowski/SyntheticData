//! Healthcare transaction types.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::super::common::{IndustryGlAccount, IndustryJournalLine, IndustryTransaction};

/// Payer type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PayerType {
    /// Medicare (CMS).
    Medicare,
    /// Medicaid (state).
    Medicaid,
    /// Commercial insurance.
    Commercial { carrier: String },
    /// Self-pay patient.
    SelfPay,
    /// Workers compensation.
    WorkersComp,
    /// Tricare (military).
    Tricare,
    /// Veterans Affairs.
    Va,
}

impl PayerType {
    /// Returns the payer type code.
    pub fn code(&self) -> &str {
        match self {
            PayerType::Medicare => "MCR",
            PayerType::Medicaid => "MCD",
            PayerType::Commercial { .. } => "COM",
            PayerType::SelfPay => "SP",
            PayerType::WorkersComp => "WC",
            PayerType::Tricare => "TRC",
            PayerType::Va => "VA",
        }
    }

    /// Returns the expected reimbursement rate compared to charges.
    pub fn expected_reimbursement_rate(&self) -> f64 {
        match self {
            PayerType::Medicare => 0.35,
            PayerType::Medicaid => 0.25,
            PayerType::Commercial { .. } => 0.55,
            PayerType::SelfPay => 0.15,
            PayerType::WorkersComp => 0.70,
            PayerType::Tricare => 0.40,
            PayerType::Va => 0.38,
        }
    }
}

/// Coding system types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CodingSystem {
    /// ICD-10-CM diagnosis codes.
    Icd10Cm,
    /// ICD-10-PCS procedure codes.
    Icd10Pcs,
    /// CPT procedure codes.
    Cpt,
    /// HCPCS Level II codes.
    Hcpcs,
    /// DRG codes.
    Drg,
    /// Revenue codes.
    RevCode,
}

impl CodingSystem {
    /// Returns the code format description.
    pub fn format_description(&self) -> &'static str {
        match self {
            CodingSystem::Icd10Cm => "A00-Z99.99",
            CodingSystem::Icd10Pcs => "0-F16ABCD",
            CodingSystem::Cpt => "00100-99499",
            CodingSystem::Hcpcs => "A0000-V5999",
            CodingSystem::Drg => "001-999",
            CodingSystem::RevCode => "0001-0999",
        }
    }
}

/// Revenue cycle transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RevenueCycleTransaction {
    /// Patient registration.
    PatientRegistration {
        patient_id: String,
        encounter_id: String,
        payer: PayerType,
        date: NaiveDate,
    },
    /// Charge capture.
    ChargeCapture {
        encounter_id: String,
        charges: Vec<Charge>,
        total_charges: Decimal,
        date: NaiveDate,
    },
    /// Claim submission.
    ClaimSubmission {
        claim_id: String,
        encounter_id: String,
        payer: PayerType,
        billed_amount: Decimal,
        diagnosis_codes: Vec<String>,
        procedure_codes: Vec<String>,
        date: NaiveDate,
    },
    /// Payment posting.
    PaymentPosting {
        claim_id: String,
        payer: PayerType,
        payment_amount: Decimal,
        adjustments: Vec<Adjustment>,
        date: NaiveDate,
    },
    /// Denial management.
    DenialPosting {
        claim_id: String,
        denial_reason: DenialReason,
        denial_code: String,
        denied_amount: Decimal,
        date: NaiveDate,
    },
    /// Contractual adjustment.
    ContractualAdjustment {
        claim_id: String,
        adjustment_amount: Decimal,
        reason_code: String,
        date: NaiveDate,
    },
    /// Patient responsibility posting.
    PatientResponsibility {
        claim_id: String,
        patient_id: String,
        responsibility_type: PatientResponsibilityType,
        amount: Decimal,
        date: NaiveDate,
    },
    /// Bad debt write-off.
    BadDebtWriteOff {
        patient_id: String,
        amount: Decimal,
        aging_days: u32,
        date: NaiveDate,
    },
}

/// A charge line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Charge {
    /// Charge ID.
    pub charge_id: String,
    /// CPT/HCPCS code.
    pub procedure_code: String,
    /// Revenue code.
    pub revenue_code: String,
    /// Service description.
    pub description: String,
    /// Quantity.
    pub quantity: u32,
    /// Unit charge amount.
    pub unit_amount: Decimal,
    /// Total charge.
    pub total_amount: Decimal,
    /// Service date.
    pub service_date: NaiveDate,
    /// Modifier codes.
    pub modifiers: Vec<String>,
}

/// An adjustment line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adjustment {
    /// Adjustment reason code.
    pub reason_code: String,
    /// Adjustment amount.
    pub amount: Decimal,
    /// Adjustment type.
    pub adjustment_type: AdjustmentType,
}

/// Types of adjustments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdjustmentType {
    /// Contractual allowance.
    Contractual,
    /// Denial.
    Denial,
    /// Write-off.
    WriteOff,
    /// Bad debt.
    BadDebt,
    /// Charity care.
    Charity,
    /// Administrative.
    Administrative,
}

/// Denial reason categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DenialReason {
    /// Medical necessity not established.
    MedicalNecessity,
    /// Prior authorization not obtained.
    PriorAuthorization,
    /// Coverage terminated.
    CoverageTerminated,
    /// Duplicate claim.
    DuplicateClaim,
    /// Invalid coding.
    InvalidCoding,
    /// Timely filing exceeded.
    TimelyFiling,
    /// Bundled service.
    BundledService,
    /// Coordination of benefits.
    CoordinationOfBenefits,
}

/// Patient responsibility types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatientResponsibilityType {
    /// Copayment.
    Copay,
    /// Coinsurance.
    Coinsurance,
    /// Deductible.
    Deductible,
    /// Non-covered service.
    NonCovered,
}

/// Clinical transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClinicalTransaction {
    /// Procedure coding.
    ProcedureCoding {
        encounter_id: String,
        cpt_codes: Vec<String>,
        icd10_pcs_codes: Vec<String>,
        date: NaiveDate,
    },
    /// Diagnosis coding.
    DiagnosisCoding {
        encounter_id: String,
        icd10_codes: Vec<String>,
        principal_diagnosis: String,
        date: NaiveDate,
    },
    /// DRG assignment.
    DrgAssignment {
        encounter_id: String,
        drg_code: String,
        drg_weight: Decimal,
        expected_reimbursement: Decimal,
        date: NaiveDate,
    },
    /// Supply consumption.
    SupplyConsumption {
        encounter_id: String,
        supplies: Vec<SupplyLine>,
        total_cost: Decimal,
        date: NaiveDate,
    },
    /// Pharmacy dispensing.
    PharmacyDispensing {
        encounter_id: String,
        medications: Vec<MedicationLine>,
        total_cost: Decimal,
        date: NaiveDate,
    },
}

/// Supply line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyLine {
    /// Supply item ID.
    pub item_id: String,
    /// Quantity used.
    pub quantity: u32,
    /// Unit cost.
    pub unit_cost: Decimal,
}

/// Medication line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicationLine {
    /// NDC code.
    pub ndc: String,
    /// Drug name.
    pub drug_name: String,
    /// Quantity.
    pub quantity: Decimal,
    /// Unit cost.
    pub unit_cost: Decimal,
}

/// Union type for healthcare transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthcareTransaction {
    /// Revenue cycle transaction.
    RevenueCycle(RevenueCycleTransaction),
    /// Clinical transaction.
    Clinical(ClinicalTransaction),
}

impl IndustryTransaction for HealthcareTransaction {
    fn transaction_type(&self) -> &str {
        match self {
            HealthcareTransaction::RevenueCycle(rc) => match rc {
                RevenueCycleTransaction::PatientRegistration { .. } => "patient_registration",
                RevenueCycleTransaction::ChargeCapture { .. } => "charge_capture",
                RevenueCycleTransaction::ClaimSubmission { .. } => "claim_submission",
                RevenueCycleTransaction::PaymentPosting { .. } => "payment_posting",
                RevenueCycleTransaction::DenialPosting { .. } => "denial_posting",
                RevenueCycleTransaction::ContractualAdjustment { .. } => "contractual_adjustment",
                RevenueCycleTransaction::PatientResponsibility { .. } => "patient_responsibility",
                RevenueCycleTransaction::BadDebtWriteOff { .. } => "bad_debt_writeoff",
            },
            HealthcareTransaction::Clinical(clinical) => match clinical {
                ClinicalTransaction::ProcedureCoding { .. } => "procedure_coding",
                ClinicalTransaction::DiagnosisCoding { .. } => "diagnosis_coding",
                ClinicalTransaction::DrgAssignment { .. } => "drg_assignment",
                ClinicalTransaction::SupplyConsumption { .. } => "supply_consumption",
                ClinicalTransaction::PharmacyDispensing { .. } => "pharmacy_dispensing",
            },
        }
    }

    fn date(&self) -> NaiveDate {
        match self {
            HealthcareTransaction::RevenueCycle(rc) => match rc {
                RevenueCycleTransaction::PatientRegistration { date, .. }
                | RevenueCycleTransaction::ChargeCapture { date, .. }
                | RevenueCycleTransaction::ClaimSubmission { date, .. }
                | RevenueCycleTransaction::PaymentPosting { date, .. }
                | RevenueCycleTransaction::DenialPosting { date, .. }
                | RevenueCycleTransaction::ContractualAdjustment { date, .. }
                | RevenueCycleTransaction::PatientResponsibility { date, .. }
                | RevenueCycleTransaction::BadDebtWriteOff { date, .. } => *date,
            },
            HealthcareTransaction::Clinical(clinical) => match clinical {
                ClinicalTransaction::ProcedureCoding { date, .. }
                | ClinicalTransaction::DiagnosisCoding { date, .. }
                | ClinicalTransaction::DrgAssignment { date, .. }
                | ClinicalTransaction::SupplyConsumption { date, .. }
                | ClinicalTransaction::PharmacyDispensing { date, .. } => *date,
            },
        }
    }

    fn amount(&self) -> Option<Decimal> {
        match self {
            HealthcareTransaction::RevenueCycle(rc) => match rc {
                RevenueCycleTransaction::ChargeCapture { total_charges, .. } => {
                    Some(*total_charges)
                }
                RevenueCycleTransaction::ClaimSubmission { billed_amount, .. } => {
                    Some(*billed_amount)
                }
                RevenueCycleTransaction::PaymentPosting { payment_amount, .. } => {
                    Some(*payment_amount)
                }
                RevenueCycleTransaction::DenialPosting { denied_amount, .. } => {
                    Some(*denied_amount)
                }
                RevenueCycleTransaction::ContractualAdjustment {
                    adjustment_amount, ..
                } => Some(*adjustment_amount),
                RevenueCycleTransaction::PatientResponsibility { amount, .. } => Some(*amount),
                RevenueCycleTransaction::BadDebtWriteOff { amount, .. } => Some(*amount),
                _ => None,
            },
            HealthcareTransaction::Clinical(clinical) => match clinical {
                ClinicalTransaction::DrgAssignment {
                    expected_reimbursement,
                    ..
                } => Some(*expected_reimbursement),
                ClinicalTransaction::SupplyConsumption { total_cost, .. } => Some(*total_cost),
                ClinicalTransaction::PharmacyDispensing { total_cost, .. } => Some(*total_cost),
                _ => None,
            },
        }
    }

    fn accounts(&self) -> Vec<String> {
        match self {
            HealthcareTransaction::RevenueCycle(rc) => match rc {
                RevenueCycleTransaction::ChargeCapture { .. } => {
                    vec!["1200".to_string(), "4100".to_string()]
                }
                RevenueCycleTransaction::PaymentPosting { .. } => {
                    vec!["1000".to_string(), "1200".to_string()]
                }
                RevenueCycleTransaction::ContractualAdjustment { .. } => {
                    vec!["4200".to_string(), "1200".to_string()]
                }
                RevenueCycleTransaction::BadDebtWriteOff { .. } => {
                    vec!["6100".to_string(), "1200".to_string()]
                }
                _ => Vec::new(),
            },
            _ => Vec::new(),
        }
    }

    fn to_journal_lines(&self) -> Vec<IndustryJournalLine> {
        match self {
            HealthcareTransaction::RevenueCycle(RevenueCycleTransaction::ChargeCapture {
                total_charges,
                ..
            }) => {
                vec![
                    IndustryJournalLine::debit("1200", *total_charges, "Accounts Receivable"),
                    IndustryJournalLine::credit("4100", *total_charges, "Patient Service Revenue"),
                ]
            }
            HealthcareTransaction::RevenueCycle(RevenueCycleTransaction::PaymentPosting {
                payment_amount,
                ..
            }) => {
                vec![
                    IndustryJournalLine::debit("1000", *payment_amount, "Cash"),
                    IndustryJournalLine::credit("1200", *payment_amount, "Accounts Receivable"),
                ]
            }
            HealthcareTransaction::RevenueCycle(
                RevenueCycleTransaction::ContractualAdjustment {
                    adjustment_amount, ..
                },
            ) => {
                vec![
                    IndustryJournalLine::debit("4200", *adjustment_amount, "Contractual Allowance"),
                    IndustryJournalLine::credit("1200", *adjustment_amount, "Accounts Receivable"),
                ]
            }
            HealthcareTransaction::RevenueCycle(RevenueCycleTransaction::BadDebtWriteOff {
                amount,
                ..
            }) => {
                vec![
                    IndustryJournalLine::debit("6100", *amount, "Bad Debt Expense"),
                    IndustryJournalLine::credit("1200", *amount, "Accounts Receivable"),
                ]
            }
            _ => Vec::new(),
        }
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("industry".to_string(), "healthcare".to_string());
        meta.insert(
            "transaction_type".to_string(),
            self.transaction_type().to_string(),
        );
        meta
    }
}

/// Generator for healthcare transactions.
#[derive(Debug, Clone)]
pub struct HealthcareTransactionGenerator {
    /// Average encounters per day.
    pub avg_daily_encounters: u32,
    /// Denial rate (0.0-1.0).
    pub denial_rate: f64,
    /// Average charges per encounter.
    pub avg_charges_per_encounter: u32,
    /// Bad debt rate (0.0-1.0).
    pub bad_debt_rate: f64,
}

impl Default for HealthcareTransactionGenerator {
    fn default() -> Self {
        Self {
            avg_daily_encounters: 150,
            denial_rate: 0.05,
            avg_charges_per_encounter: 8,
            bad_debt_rate: 0.03,
        }
    }
}

impl HealthcareTransactionGenerator {
    /// Returns healthcare-specific GL accounts.
    pub fn gl_accounts() -> Vec<IndustryGlAccount> {
        vec![
            IndustryGlAccount::new("1000", "Cash and Cash Equivalents", "Asset", "Cash")
                .into_control(),
            IndustryGlAccount::new(
                "1200",
                "Patient Accounts Receivable",
                "Asset",
                "Receivables",
            )
            .into_control(),
            IndustryGlAccount::new(
                "1210",
                "Allowance for Doubtful Accounts",
                "Asset",
                "Receivables",
            )
            .with_normal_balance("Credit"),
            IndustryGlAccount::new("4100", "Patient Service Revenue", "Revenue", "Revenue")
                .with_normal_balance("Credit"),
            IndustryGlAccount::new("4200", "Contractual Allowances", "Revenue", "Deductions"),
            IndustryGlAccount::new("4210", "Charity Care", "Revenue", "Deductions"),
            IndustryGlAccount::new("4220", "Bad Debt Provision", "Revenue", "Deductions"),
            IndustryGlAccount::new("5100", "Salaries and Benefits", "Expense", "Labor"),
            IndustryGlAccount::new("5200", "Medical Supplies", "Expense", "Supplies"),
            IndustryGlAccount::new("5300", "Pharmaceuticals", "Expense", "Drugs"),
            IndustryGlAccount::new("5400", "Professional Fees", "Expense", "Professional"),
            IndustryGlAccount::new("6100", "Bad Debt Expense", "Expense", "Bad Debt"),
        ]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_payer_type() {
        let medicare = PayerType::Medicare;
        assert_eq!(medicare.code(), "MCR");
        assert!(medicare.expected_reimbursement_rate() > 0.3);

        let commercial = PayerType::Commercial {
            carrier: "BlueCross".to_string(),
        };
        assert_eq!(commercial.code(), "COM");
    }

    #[test]
    fn test_charge_capture() {
        let tx = HealthcareTransaction::RevenueCycle(RevenueCycleTransaction::ChargeCapture {
            encounter_id: "E001".to_string(),
            charges: vec![Charge {
                charge_id: "CHG001".to_string(),
                procedure_code: "99213".to_string(),
                revenue_code: "0510".to_string(),
                description: "Office Visit".to_string(),
                quantity: 1,
                unit_amount: Decimal::new(150, 0),
                total_amount: Decimal::new(150, 0),
                service_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                modifiers: Vec::new(),
            }],
            total_charges: Decimal::new(150, 0),
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        });

        assert_eq!(tx.transaction_type(), "charge_capture");
        assert_eq!(tx.amount(), Some(Decimal::new(150, 0)));

        let lines = tx.to_journal_lines();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_payment_posting() {
        let tx = HealthcareTransaction::RevenueCycle(RevenueCycleTransaction::PaymentPosting {
            claim_id: "CLM001".to_string(),
            payer: PayerType::Medicare,
            payment_amount: Decimal::new(100, 0),
            adjustments: vec![Adjustment {
                reason_code: "CO-45".to_string(),
                amount: Decimal::new(50, 0),
                adjustment_type: AdjustmentType::Contractual,
            }],
            date: NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
        });

        assert_eq!(tx.transaction_type(), "payment_posting");
    }

    #[test]
    fn test_gl_accounts() {
        let accounts = HealthcareTransactionGenerator::gl_accounts();
        assert!(accounts.len() >= 10);

        let ar = accounts.iter().find(|a| a.account_number == "1200");
        assert!(ar.is_some());
    }
}
