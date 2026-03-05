//! Collusion ring network modeling.
//!
//! Models fraud networks with multiple conspirators, coordinated schemes,
//! trust dynamics, and realistic behavioral patterns.

use chrono::NaiveDate;
use rand::Rng;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};

use datasynth_core::{AcfeFraudCategory, AnomalyDetectionDifficulty, ConcealmentTechnique};

/// Type of collusion ring based on participant composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CollusionRingType {
    // ========== Internal Collusion ==========
    /// Two employees colluding (e.g., approver + processor).
    EmployeePair,
    /// Small departmental ring (3-5 employees).
    DepartmentRing,
    /// Manager with one or more subordinates.
    ManagementSubordinate,
    /// Multiple employees across departments.
    CrossDepartment,

    // ========== Internal-External Collusion ==========
    /// Purchasing employee with vendor contact.
    EmployeeVendor,
    /// Sales rep with customer contact.
    EmployeeCustomer,
    /// Project manager with external contractor.
    EmployeeContractor,

    // ========== External Rings ==========
    /// Multiple vendors colluding for bid rigging.
    VendorRing,
    /// Multiple customers for return fraud schemes.
    CustomerRing,
}

impl CollusionRingType {
    /// Returns the typical size range for this ring type.
    pub fn typical_size_range(&self) -> (usize, usize) {
        match self {
            CollusionRingType::EmployeePair => (2, 2),
            CollusionRingType::DepartmentRing => (3, 5),
            CollusionRingType::ManagementSubordinate => (2, 4),
            CollusionRingType::CrossDepartment => (3, 6),
            CollusionRingType::EmployeeVendor => (2, 3),
            CollusionRingType::EmployeeCustomer => (2, 3),
            CollusionRingType::EmployeeContractor => (2, 4),
            CollusionRingType::VendorRing => (2, 4),
            CollusionRingType::CustomerRing => (2, 3),
        }
    }

    /// Returns whether this ring type involves external parties.
    pub fn involves_external(&self) -> bool {
        matches!(
            self,
            CollusionRingType::EmployeeVendor
                | CollusionRingType::EmployeeCustomer
                | CollusionRingType::EmployeeContractor
                | CollusionRingType::VendorRing
                | CollusionRingType::CustomerRing
        )
    }

    /// Returns the detection difficulty multiplier for this ring type.
    pub fn detection_difficulty_multiplier(&self) -> f64 {
        match self {
            // Internal collusion easier to detect through behavioral analysis
            CollusionRingType::EmployeePair => 1.2,
            CollusionRingType::DepartmentRing => 1.3,
            CollusionRingType::ManagementSubordinate => 1.5,
            CollusionRingType::CrossDepartment => 1.4,
            // External collusion harder due to limited visibility
            CollusionRingType::EmployeeVendor => 1.6,
            CollusionRingType::EmployeeCustomer => 1.5,
            CollusionRingType::EmployeeContractor => 1.7,
            CollusionRingType::VendorRing => 1.8,
            CollusionRingType::CustomerRing => 1.4,
        }
    }

    /// Returns all variants for iteration.
    pub fn all_variants() -> &'static [CollusionRingType] {
        &[
            CollusionRingType::EmployeePair,
            CollusionRingType::DepartmentRing,
            CollusionRingType::ManagementSubordinate,
            CollusionRingType::CrossDepartment,
            CollusionRingType::EmployeeVendor,
            CollusionRingType::EmployeeCustomer,
            CollusionRingType::EmployeeContractor,
            CollusionRingType::VendorRing,
            CollusionRingType::CustomerRing,
        ]
    }
}

/// Role of a conspirator within the fraud ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConspiratorRole {
    /// Conceives the scheme and recruits others.
    Initiator,
    /// Performs the actual transactions.
    Executor,
    /// Provides approvals or authorization overrides.
    Approver,
    /// Hides evidence and manipulates records.
    Concealer,
    /// Monitors for detection and warns others.
    Lookout,
    /// External recipient of proceeds (vendor, customer).
    Beneficiary,
    /// Provides inside information without direct participation.
    Informant,
}

impl ConspiratorRole {
    /// Returns the risk weight (likelihood of detection through this role).
    pub fn detection_risk_weight(&self) -> f64 {
        match self {
            ConspiratorRole::Initiator => 0.25,
            ConspiratorRole::Executor => 0.35,
            ConspiratorRole::Approver => 0.30,
            ConspiratorRole::Concealer => 0.20,
            ConspiratorRole::Lookout => 0.10,
            ConspiratorRole::Beneficiary => 0.40,
            ConspiratorRole::Informant => 0.15,
        }
    }

    /// Returns whether this role is typically internal to the organization.
    pub fn is_typically_internal(&self) -> bool {
        !matches!(self, ConspiratorRole::Beneficiary)
    }
}

/// Type of entity participating in the ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    /// Employee of the organization.
    Employee,
    /// Manager or executive.
    Manager,
    /// External vendor.
    Vendor,
    /// External customer.
    Customer,
    /// External contractor.
    Contractor,
}

/// A conspirator within a collusion ring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conspirator {
    /// Unique identifier for this conspirator.
    pub conspirator_id: Uuid,
    /// Reference to the entity (employee ID, vendor ID, etc.).
    pub entity_id: String,
    /// Type of entity.
    pub entity_type: EntityType,
    /// Role within the ring.
    pub role: ConspiratorRole,
    /// Date joined the ring.
    pub join_date: NaiveDate,
    /// Loyalty score (0.0-1.0): probability of not defecting under pressure.
    pub loyalty: f64,
    /// Risk tolerance (0.0-1.0): willingness to escalate scheme.
    pub risk_tolerance: f64,
    /// Share of proceeds (0.0-1.0).
    pub proceeds_share: f64,
    /// Department (for employees).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub department: Option<String>,
    /// Position level (for employees).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_level: Option<String>,
    /// Times the conspirator has been involved in successful transactions.
    pub successful_actions: u32,
    /// Times the conspirator has had close calls.
    pub near_misses: u32,
}

impl Conspirator {
    /// Creates a new conspirator.
    pub fn new(
        entity_id: impl Into<String>,
        entity_type: EntityType,
        role: ConspiratorRole,
        join_date: NaiveDate,
    ) -> Self {
        let uuid_factory = DeterministicUuidFactory::new(0, GeneratorType::Anomaly);
        Self {
            conspirator_id: uuid_factory.next(),
            entity_id: entity_id.into(),
            entity_type,
            role,
            join_date,
            loyalty: 0.7,
            risk_tolerance: 0.5,
            proceeds_share: 0.0,
            department: None,
            position_level: None,
            successful_actions: 0,
            near_misses: 0,
        }
    }

    /// Sets the loyalty score.
    pub fn with_loyalty(mut self, loyalty: f64) -> Self {
        self.loyalty = loyalty.clamp(0.0, 1.0);
        self
    }

    /// Sets the risk tolerance.
    pub fn with_risk_tolerance(mut self, tolerance: f64) -> Self {
        self.risk_tolerance = tolerance.clamp(0.0, 1.0);
        self
    }

    /// Sets the proceeds share.
    pub fn with_proceeds_share(mut self, share: f64) -> Self {
        self.proceeds_share = share.clamp(0.0, 1.0);
        self
    }

    /// Sets the department.
    pub fn with_department(mut self, department: impl Into<String>) -> Self {
        self.department = Some(department.into());
        self
    }

    /// Calculates defection probability based on current conditions.
    pub fn defection_probability(
        &self,
        detection_pressure: f64,
        months_in_scheme: u32,
        external_pressure: f64,
    ) -> f64 {
        // Base defection rate inversely proportional to loyalty
        let base_rate = 1.0 - self.loyalty;

        // Modify by detection pressure
        let pressure_factor = 1.0 + (detection_pressure * 0.5);

        // Modify by time in scheme (fatigue)
        let fatigue_factor = 1.0 + (months_in_scheme as f64 * 0.02);

        // External pressure (personal issues, fear)
        let external_factor = 1.0 + (external_pressure * 0.3);

        // Near misses increase defection likelihood
        let near_miss_factor = 1.0 + (self.near_misses as f64 * 0.15);

        let probability =
            base_rate * pressure_factor * fatigue_factor * external_factor * near_miss_factor;

        probability.clamp(0.0, 1.0)
    }

    /// Records a successful action.
    pub fn record_success(&mut self) {
        self.successful_actions += 1;
        // Success can increase risk tolerance
        self.risk_tolerance = (self.risk_tolerance + 0.02).min(1.0);
    }

    /// Records a near miss.
    pub fn record_near_miss(&mut self) {
        self.near_misses += 1;
        // Near misses decrease risk tolerance
        self.risk_tolerance = (self.risk_tolerance - 0.05).max(0.0);
    }
}

/// Status of a collusion ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RingStatus {
    /// Trust-building phase, small test transactions.
    Forming,
    /// Actively executing scheme.
    Active,
    /// Increasing amounts and frequency.
    Escalating,
    /// Temporarily paused due to fear or external events.
    Dormant,
    /// Breaking apart (member leaving or distrust).
    Dissolving,
    /// Caught by detection mechanisms.
    Detected,
    /// Successfully completed without detection.
    Completed,
}

impl RingStatus {
    /// Returns whether the ring is currently operational.
    pub fn is_operational(&self) -> bool {
        matches!(
            self,
            RingStatus::Forming | RingStatus::Active | RingStatus::Escalating
        )
    }

    /// Returns whether the ring has ended.
    pub fn is_terminated(&self) -> bool {
        matches!(
            self,
            RingStatus::Dissolving | RingStatus::Detected | RingStatus::Completed
        )
    }
}

/// Behavioral parameters for ring simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingBehavior {
    /// Average days between transactions.
    pub transaction_interval_days: u32,
    /// Variance in transaction timing (0.0-1.0).
    pub timing_variance: f64,
    /// Average transaction amount.
    pub avg_transaction_amount: Decimal,
    /// Escalation factor per successful month.
    pub escalation_factor: f64,
    /// Maximum amount per transaction.
    pub max_transaction_amount: Decimal,
    /// Preferred days of week (0=Mon, 6=Sun).
    pub preferred_days: Vec<u32>,
    /// Whether to avoid month-end periods.
    pub avoid_month_end: bool,
    /// Concealment techniques used.
    pub concealment_techniques: Vec<ConcealmentTechnique>,
}

impl Default for RingBehavior {
    fn default() -> Self {
        Self {
            transaction_interval_days: 14,
            timing_variance: 0.3,
            avg_transaction_amount: Decimal::new(5_000, 0),
            escalation_factor: 1.05,
            max_transaction_amount: Decimal::new(50_000, 0),
            preferred_days: vec![1, 2, 3], // Tue-Thu
            avoid_month_end: true,
            concealment_techniques: vec![
                ConcealmentTechnique::TransactionSplitting,
                ConcealmentTechnique::TimingExploitation,
            ],
        }
    }
}

/// Configuration for collusion ring generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollusionRingConfig {
    /// Probability of collusion in fraud schemes.
    pub collusion_rate: f64,
    /// Distribution of ring types.
    pub ring_type_weights: HashMap<String, f64>,
    /// Minimum ring duration in months.
    pub min_duration_months: u32,
    /// Maximum ring duration in months.
    pub max_duration_months: u32,
    /// Average loyalty score for new conspirators.
    pub avg_loyalty: f64,
    /// Average risk tolerance for new conspirators.
    pub avg_risk_tolerance: f64,
}

impl Default for CollusionRingConfig {
    fn default() -> Self {
        let mut ring_type_weights = HashMap::new();
        ring_type_weights.insert("employee_pair".to_string(), 0.25);
        ring_type_weights.insert("department_ring".to_string(), 0.15);
        ring_type_weights.insert("management_subordinate".to_string(), 0.15);
        ring_type_weights.insert("employee_vendor".to_string(), 0.20);
        ring_type_weights.insert("employee_customer".to_string(), 0.10);
        ring_type_weights.insert("vendor_ring".to_string(), 0.10);
        ring_type_weights.insert("customer_ring".to_string(), 0.05);

        Self {
            collusion_rate: 0.50, // ACFE reports ~50% of fraud involves collusion
            ring_type_weights,
            min_duration_months: 3,
            max_duration_months: 36,
            avg_loyalty: 0.70,
            avg_risk_tolerance: 0.50,
        }
    }
}

/// A collusion ring modeling multiple conspirators in a coordinated scheme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollusionRing {
    /// Unique ring identifier.
    pub ring_id: Uuid,
    /// Type of collusion ring.
    pub ring_type: CollusionRingType,
    /// ACFE fraud category.
    pub fraud_category: AcfeFraudCategory,
    /// Members of the ring.
    pub members: Vec<Conspirator>,
    /// Date the ring was formed.
    pub formation_date: NaiveDate,
    /// Current status.
    pub status: RingStatus,
    /// Total amount stolen by the ring.
    pub total_stolen: Decimal,
    /// Number of successful transactions.
    pub transaction_count: u32,
    /// Current detection risk (0.0-1.0).
    pub detection_risk: f64,
    /// Behavioral parameters.
    pub behavior: RingBehavior,
    /// Months the ring has been active.
    pub active_months: u32,
    /// Transaction IDs associated with this ring.
    pub transaction_ids: Vec<String>,
    /// Metadata for tracking.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl CollusionRing {
    /// Creates a new collusion ring.
    pub fn new(
        ring_type: CollusionRingType,
        fraud_category: AcfeFraudCategory,
        formation_date: NaiveDate,
    ) -> Self {
        let uuid_factory = DeterministicUuidFactory::new(0, GeneratorType::Anomaly);
        Self {
            ring_id: uuid_factory.next(),
            ring_type,
            fraud_category,
            members: Vec::new(),
            formation_date,
            status: RingStatus::Forming,
            total_stolen: Decimal::ZERO,
            transaction_count: 0,
            detection_risk: 0.05,
            behavior: RingBehavior::default(),
            active_months: 0,
            transaction_ids: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Adds a conspirator to the ring.
    pub fn add_member(&mut self, conspirator: Conspirator) {
        self.members.push(conspirator);
        self.update_detection_risk();
    }

    /// Returns the size of the ring.
    pub fn size(&self) -> usize {
        self.members.len()
    }

    /// Returns the initiator(s) of the ring.
    pub fn initiators(&self) -> Vec<&Conspirator> {
        self.members
            .iter()
            .filter(|m| m.role == ConspiratorRole::Initiator)
            .collect()
    }

    /// Returns the executor(s) of the ring.
    pub fn executors(&self) -> Vec<&Conspirator> {
        self.members
            .iter()
            .filter(|m| m.role == ConspiratorRole::Executor)
            .collect()
    }

    /// Returns the approver(s) of the ring.
    pub fn approvers(&self) -> Vec<&Conspirator> {
        self.members
            .iter()
            .filter(|m| m.role == ConspiratorRole::Approver)
            .collect()
    }

    /// Updates detection risk based on current state.
    fn update_detection_risk(&mut self) {
        // Base risk from ring size
        let size_risk = (self.members.len() as f64 * 0.05).min(0.3);

        // Risk from external members
        let external_count = self
            .members
            .iter()
            .filter(|m| !m.role.is_typically_internal())
            .count();
        let external_risk = external_count as f64 * 0.03;

        // Risk from transaction count
        let tx_risk = (self.transaction_count as f64 * 0.005).min(0.2);

        // Risk from total amount
        let amount_f64: f64 = self.total_stolen.try_into().unwrap_or(0.0);
        let amount_risk = ((amount_f64 / 100_000.0).ln().max(0.0) * 0.02).min(0.15);

        // Risk from time active
        let time_risk = (self.active_months as f64 * 0.01).min(0.2);

        // Ring type multiplier
        let type_multiplier = self.ring_type.detection_difficulty_multiplier();

        // Calculate total risk (capped at 0.95)
        self.detection_risk = ((size_risk + external_risk + tx_risk + amount_risk + time_risk)
            / type_multiplier)
            .min(0.95);
    }

    /// Records a successful transaction.
    pub fn record_transaction(&mut self, amount: Decimal, transaction_id: impl Into<String>) {
        self.total_stolen += amount;
        self.transaction_count += 1;
        self.transaction_ids.push(transaction_id.into());

        // Update member success counts
        for member in &mut self.members {
            if matches!(
                member.role,
                ConspiratorRole::Executor | ConspiratorRole::Approver | ConspiratorRole::Initiator
            ) {
                member.record_success();
            }
        }

        self.update_detection_risk();
    }

    /// Records a near miss event.
    pub fn record_near_miss(&mut self) {
        // All members become more cautious
        for member in &mut self.members {
            member.record_near_miss();
        }

        // Increase detection risk
        self.detection_risk = (self.detection_risk + 0.1).min(0.95);

        // Consider going dormant
        if self.detection_risk > 0.5 {
            self.status = RingStatus::Dormant;
        }
    }

    /// Advances the ring by one month.
    pub fn advance_month<R: Rng>(&mut self, rng: &mut R) {
        if !self.status.is_operational() {
            return;
        }

        self.active_months += 1;

        // Check for defection
        if self.check_defection(rng) {
            self.status = RingStatus::Dissolving;
            return;
        }

        // Check for detection
        if rng.random::<f64>() < self.detection_risk * 0.1 {
            self.status = RingStatus::Detected;
            return;
        }

        // Status progression
        match self.status {
            RingStatus::Forming if self.active_months >= 2 && self.transaction_count >= 3 => {
                self.status = RingStatus::Active;
            }
            RingStatus::Active if self.active_months >= 6 && self.detection_risk < 0.3 => {
                // Consider escalation if successful
                if rng.random::<f64>() < 0.3 {
                    self.status = RingStatus::Escalating;
                    self.behavior.avg_transaction_amount = self
                        .behavior
                        .avg_transaction_amount
                        .saturating_mul(Decimal::from_str_exact("1.5").unwrap_or(Decimal::ONE));
                }
            }
            RingStatus::Dormant if self.active_months.is_multiple_of(3) => {
                // Chance to reactivate
                if rng.random::<f64>() < 0.4 && self.detection_risk < 0.4 {
                    self.status = RingStatus::Active;
                    self.detection_risk *= 0.8; // Risk reduced during dormancy
                }
            }
            _ => {}
        }

        // Simulate transactions for operational rings
        // (Forming needs transactions too, since Forming→Active requires transaction_count >= 3)
        if matches!(
            self.status,
            RingStatus::Forming | RingStatus::Active | RingStatus::Escalating
        ) {
            let txns_per_month = (30 / self.behavior.transaction_interval_days.max(1)).max(1);
            for _ in 0..txns_per_month {
                // Variance: ±30% around avg_transaction_amount
                let variance = 0.7 + rng.random::<f64>() * 0.6; // 0.7 to 1.3
                let amount = self
                    .behavior
                    .avg_transaction_amount
                    .saturating_mul(Decimal::try_from(variance).unwrap_or(Decimal::ONE));
                let amount = amount.min(self.behavior.max_transaction_amount);
                let tx_id = format!("TX-{}-{:04}", self.ring_id, self.transaction_count + 1);
                self.record_transaction(amount, tx_id);
            }
            // Apply escalation factor for escalating rings
            if self.status == RingStatus::Escalating {
                self.behavior.avg_transaction_amount =
                    self.behavior.avg_transaction_amount.saturating_mul(
                        Decimal::try_from(self.behavior.escalation_factor).unwrap_or(Decimal::ONE),
                    );
            }
        }
    }

    /// Checks if any member defects.
    fn check_defection<R: Rng>(&self, rng: &mut R) -> bool {
        for member in &self.members {
            let defection_prob = member.defection_probability(
                self.detection_risk,
                self.active_months,
                rng.random::<f64>() * 0.3, // Random external pressure
            );

            if rng.random::<f64>() < defection_prob {
                return true;
            }
        }
        false
    }

    /// Returns the detection difficulty for this ring.
    pub fn detection_difficulty(&self) -> AnomalyDetectionDifficulty {
        // Base difficulty from concealment techniques
        let concealment_bonus: f64 = self
            .behavior
            .concealment_techniques
            .iter()
            .map(|c| c.difficulty_bonus())
            .sum();

        // Ring type multiplier
        let type_multiplier = self.ring_type.detection_difficulty_multiplier();

        // Combined score
        let score = (0.5 + concealment_bonus) * type_multiplier;

        AnomalyDetectionDifficulty::from_score(score.min(1.0))
    }

    /// Calculates the average share per member.
    pub fn avg_share_per_member(&self) -> Decimal {
        if self.members.is_empty() {
            return Decimal::ZERO;
        }
        self.total_stolen / Decimal::from(self.members.len())
    }

    /// Returns descriptive summary of the ring.
    pub fn description(&self) -> String {
        format!(
            "{:?} ring with {} members, {} transactions totaling {}, active {} months",
            self.ring_type,
            self.members.len(),
            self.transaction_count,
            self.total_stolen,
            self.active_months
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_collusion_ring_type() {
        let emp_pair = CollusionRingType::EmployeePair;
        assert_eq!(emp_pair.typical_size_range(), (2, 2));
        assert!(!emp_pair.involves_external());

        let emp_vendor = CollusionRingType::EmployeeVendor;
        assert!(emp_vendor.involves_external());
        assert!(emp_vendor.detection_difficulty_multiplier() > 1.0);
    }

    #[test]
    fn test_conspirator() {
        let conspirator = Conspirator::new(
            "EMP001",
            EntityType::Employee,
            ConspiratorRole::Executor,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )
        .with_loyalty(0.8)
        .with_risk_tolerance(0.6)
        .with_proceeds_share(0.4)
        .with_department("Accounting");

        assert_eq!(conspirator.loyalty, 0.8);
        assert_eq!(conspirator.risk_tolerance, 0.6);
        assert_eq!(conspirator.department, Some("Accounting".to_string()));
    }

    #[test]
    fn test_defection_probability() {
        let conspirator = Conspirator::new(
            "EMP001",
            EntityType::Employee,
            ConspiratorRole::Executor,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )
        .with_loyalty(0.9);

        // Low pressure = low defection
        let low_pressure = conspirator.defection_probability(0.1, 1, 0.0);
        assert!(low_pressure < 0.3);

        // High pressure = higher defection
        let high_pressure = conspirator.defection_probability(0.8, 12, 0.5);
        assert!(high_pressure > low_pressure);
    }

    #[test]
    fn test_collusion_ring() {
        let mut ring = CollusionRing::new(
            CollusionRingType::EmployeePair,
            AcfeFraudCategory::AssetMisappropriation,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        ring.add_member(Conspirator::new(
            "EMP001",
            EntityType::Employee,
            ConspiratorRole::Initiator,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        ));
        ring.add_member(Conspirator::new(
            "EMP002",
            EntityType::Employee,
            ConspiratorRole::Approver,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        ));

        assert_eq!(ring.size(), 2);
        assert_eq!(ring.initiators().len(), 1);
        assert_eq!(ring.approvers().len(), 1);
        assert_eq!(ring.status, RingStatus::Forming);
    }

    #[test]
    fn test_ring_transaction() {
        let mut ring = CollusionRing::new(
            CollusionRingType::EmployeeVendor,
            AcfeFraudCategory::Corruption,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        ring.add_member(Conspirator::new(
            "EMP001",
            EntityType::Employee,
            ConspiratorRole::Executor,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        ));

        ring.record_transaction(Decimal::new(10_000, 0), "TX001");
        assert_eq!(ring.total_stolen, Decimal::new(10_000, 0));
        assert_eq!(ring.transaction_count, 1);
        // Detection risk should have been recalculated after transaction
        assert!(ring.detection_risk >= 0.0);
    }

    #[test]
    fn test_ring_advance_month() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let mut ring = CollusionRing::new(
            CollusionRingType::DepartmentRing,
            AcfeFraudCategory::AssetMisappropriation,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        // Add members with high loyalty to prevent defection
        for i in 0..3 {
            ring.add_member(
                Conspirator::new(
                    format!("EMP{:03}", i),
                    EntityType::Employee,
                    if i == 0 {
                        ConspiratorRole::Initiator
                    } else {
                        ConspiratorRole::Executor
                    },
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                )
                .with_loyalty(0.99), // Very loyal to avoid random defection
            );
        }

        // Record some transactions first
        for i in 0..3 {
            ring.record_transaction(Decimal::new(1_000, 0), format!("TX{:03}", i));
        }

        // Advance to activate
        for _ in 0..3 {
            ring.advance_month(&mut rng);
        }

        assert!(ring.active_months >= 3);
        // Status might have changed
        assert!(ring.status.is_operational() || ring.status.is_terminated());
    }

    #[test]
    fn test_ring_near_miss() {
        let mut ring = CollusionRing::new(
            CollusionRingType::EmployeePair,
            AcfeFraudCategory::AssetMisappropriation,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        ring.add_member(Conspirator::new(
            "EMP001",
            EntityType::Employee,
            ConspiratorRole::Executor,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        ));

        let initial_risk = ring.detection_risk;
        ring.record_near_miss();

        assert!(ring.detection_risk > initial_risk);
        assert_eq!(ring.members[0].near_misses, 1);
    }

    #[test]
    fn test_ring_detection_difficulty() {
        let mut ring = CollusionRing::new(
            CollusionRingType::EmployeeVendor,
            AcfeFraudCategory::Corruption,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        ring.behavior.concealment_techniques = vec![
            ConcealmentTechnique::Collusion,
            ConcealmentTechnique::DocumentManipulation,
            ConcealmentTechnique::FalseDocumentation,
        ];

        let difficulty = ring.detection_difficulty();
        // With multiple concealment techniques and external involvement, should be hard+
        assert!(matches!(
            difficulty,
            AnomalyDetectionDifficulty::Hard | AnomalyDetectionDifficulty::Expert
        ));
    }
}
