//! Multi-tier vendor network models for supply chain simulation.
//!
//! Provides comprehensive vendor relationship modeling including:
//! - Supply chain tiers (Tier 1, 2, 3 suppliers)
//! - Strategic importance and spend classification
//! - Vendor clustering for realistic behavior patterns
//! - Lifecycle stages and dependency tracking
//! - Quality scoring and payment history

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of vendor relationship in the supply chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VendorRelationshipType {
    /// Direct supplier of goods or materials
    #[default]
    DirectSupplier,
    /// Provider of services
    ServiceProvider,
    /// Contract worker or firm
    Contractor,
    /// Product distributor
    Distributor,
    /// Manufacturer of finished goods
    Manufacturer,
    /// Supplier of raw materials
    RawMaterialSupplier,
    /// Original equipment manufacturer partner
    OemPartner,
    /// Affiliated company
    Affiliate,
    /// Joint venture partner
    JointVenturePartner,
    /// Subcontractor
    Subcontractor,
}

impl VendorRelationshipType {
    /// Get the relationship type code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::DirectSupplier => "DS",
            Self::ServiceProvider => "SP",
            Self::Contractor => "CT",
            Self::Distributor => "DI",
            Self::Manufacturer => "MF",
            Self::RawMaterialSupplier => "RM",
            Self::OemPartner => "OE",
            Self::Affiliate => "AF",
            Self::JointVenturePartner => "JV",
            Self::Subcontractor => "SC",
        }
    }

    /// Check if this is a strategic relationship type.
    pub fn is_strategic(&self) -> bool {
        matches!(
            self,
            Self::OemPartner | Self::JointVenturePartner | Self::Affiliate | Self::Manufacturer
        )
    }
}

/// Supply chain tier classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SupplyChainTier {
    /// Direct supplier to the company (full visibility)
    #[default]
    Tier1,
    /// Supplier to Tier 1 (partial visibility)
    Tier2,
    /// Supplier to Tier 2 (minimal visibility)
    Tier3,
}

impl SupplyChainTier {
    /// Get the tier number.
    pub fn tier_number(&self) -> u8 {
        match self {
            Self::Tier1 => 1,
            Self::Tier2 => 2,
            Self::Tier3 => 3,
        }
    }

    /// Get visibility level (0.0 to 1.0).
    pub fn visibility(&self) -> f64 {
        match self {
            Self::Tier1 => 1.0,
            Self::Tier2 => 0.5,
            Self::Tier3 => 0.2,
        }
    }

    /// Get the child tier (supplier to this tier).
    pub fn child_tier(&self) -> Option<Self> {
        match self {
            Self::Tier1 => Some(Self::Tier2),
            Self::Tier2 => Some(Self::Tier3),
            Self::Tier3 => None,
        }
    }

    /// Get the parent tier (customer of this tier).
    pub fn parent_tier(&self) -> Option<Self> {
        match self {
            Self::Tier1 => None,
            Self::Tier2 => Some(Self::Tier1),
            Self::Tier3 => Some(Self::Tier2),
        }
    }
}

/// Strategic importance level of a vendor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StrategicLevel {
    /// Critical to operations, single-source dependency
    Critical,
    /// Important strategic partner
    Important,
    /// Standard operational supplier
    #[default]
    Standard,
    /// Transactional, easily replaceable
    Transactional,
}

impl StrategicLevel {
    /// Get the importance score (0.0 to 1.0).
    pub fn importance_score(&self) -> f64 {
        match self {
            Self::Critical => 1.0,
            Self::Important => 0.75,
            Self::Standard => 0.5,
            Self::Transactional => 0.25,
        }
    }

    /// Get typical procurement oversight level.
    pub fn oversight_level(&self) -> &'static str {
        match self {
            Self::Critical => "executive",
            Self::Important => "senior_management",
            Self::Standard => "procurement_team",
            Self::Transactional => "automated",
        }
    }
}

/// Spend tier based on annual procurement volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SpendTier {
    /// Highest spend tier (top 5% by spend)
    Platinum,
    /// High spend tier (next 15% by spend)
    Gold,
    /// Medium spend tier (next 30% by spend)
    #[default]
    Silver,
    /// Lower spend tier (bottom 50% by spend)
    Bronze,
}

impl SpendTier {
    /// Get the minimum spend percentage for this tier.
    pub fn min_spend_percentile(&self) -> f64 {
        match self {
            Self::Platinum => 0.95,
            Self::Gold => 0.80,
            Self::Silver => 0.50,
            Self::Bronze => 0.0,
        }
    }

    /// Get the discount eligibility multiplier.
    pub fn discount_multiplier(&self) -> f64 {
        match self {
            Self::Platinum => 1.15,
            Self::Gold => 1.10,
            Self::Silver => 1.05,
            Self::Bronze => 1.0,
        }
    }

    /// Get the payment priority level.
    pub fn payment_priority(&self) -> u8 {
        match self {
            Self::Platinum => 1,
            Self::Gold => 2,
            Self::Silver => 3,
            Self::Bronze => 4,
        }
    }
}

/// Vendor cluster for behavioral grouping.
///
/// Based on research showing vendors typically cluster into 4 groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VendorCluster {
    /// Reliable strategic partners (~20% of vendors)
    ReliableStrategic,
    /// Standard operational vendors (~50% of vendors)
    #[default]
    StandardOperational,
    /// Transactional vendors (~25% of vendors)
    Transactional,
    /// Problematic vendors requiring monitoring (~5% of vendors)
    Problematic,
}

impl VendorCluster {
    /// Get the typical distribution percentage for this cluster.
    pub fn typical_distribution(&self) -> f64 {
        match self {
            Self::ReliableStrategic => 0.20,
            Self::StandardOperational => 0.50,
            Self::Transactional => 0.25,
            Self::Problematic => 0.05,
        }
    }

    /// Get the on-time delivery probability.
    pub fn on_time_delivery_probability(&self) -> f64 {
        match self {
            Self::ReliableStrategic => 0.98,
            Self::StandardOperational => 0.92,
            Self::Transactional => 0.85,
            Self::Problematic => 0.70,
        }
    }

    /// Get the quality issue probability.
    pub fn quality_issue_probability(&self) -> f64 {
        match self {
            Self::ReliableStrategic => 0.01,
            Self::StandardOperational => 0.03,
            Self::Transactional => 0.07,
            Self::Problematic => 0.15,
        }
    }

    /// Get the invoice accuracy probability.
    pub fn invoice_accuracy_probability(&self) -> f64 {
        match self {
            Self::ReliableStrategic => 0.99,
            Self::StandardOperational => 0.95,
            Self::Transactional => 0.90,
            Self::Problematic => 0.80,
        }
    }
}

/// Reason for vendor decline.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeclineReason {
    /// Quality degradation
    QualityIssues,
    /// Price increases
    PriceIssues,
    /// Delivery problems
    DeliveryIssues,
    /// Financial instability
    FinancialConcerns,
    /// Strategic shift to alternatives
    StrategicShift,
    /// Regulatory or compliance issues
    ComplianceIssues,
    /// Other reasons
    Other(String),
}

/// Reason for vendor termination.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminationReason {
    /// Contract expiration not renewed
    ContractExpired,
    /// Vendor breach of contract
    Breach,
    /// Vendor bankruptcy
    Bankruptcy,
    /// Mutual agreement
    MutualAgreement,
    /// Compliance violation
    ComplianceViolation,
    /// Performance issues
    PerformanceIssues,
    /// Strategic consolidation
    Consolidation,
    /// Vendor acquisition by another company
    Acquisition,
    /// Other reasons
    Other(String),
}

/// Vendor lifecycle stage tracking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VendorLifecycleStage {
    /// Initial onboarding phase
    Onboarding {
        started: NaiveDate,
        expected_completion: NaiveDate,
    },
    /// Ramp-up period (increasing volume)
    RampUp {
        started: NaiveDate,
        target_volume_percent: u8,
    },
    /// Steady state operations
    SteadyState { since: NaiveDate },
    /// Declining relationship
    Decline {
        started: NaiveDate,
        reason: DeclineReason,
    },
    /// Terminated relationship
    Terminated {
        date: NaiveDate,
        reason: TerminationReason,
    },
}

impl VendorLifecycleStage {
    /// Check if the vendor is active.
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Terminated { .. })
    }

    /// Check if the vendor is in good standing.
    pub fn is_good_standing(&self) -> bool {
        matches!(
            self,
            Self::Onboarding { .. } | Self::RampUp { .. } | Self::SteadyState { .. }
        )
    }

    /// Get the stage name.
    pub fn stage_name(&self) -> &'static str {
        match self {
            Self::Onboarding { .. } => "onboarding",
            Self::RampUp { .. } => "ramp_up",
            Self::SteadyState { .. } => "steady_state",
            Self::Decline { .. } => "decline",
            Self::Terminated { .. } => "terminated",
        }
    }
}

impl Default for VendorLifecycleStage {
    fn default() -> Self {
        Self::SteadyState {
            since: NaiveDate::from_ymd_opt(2020, 1, 1).expect("valid default date"),
        }
    }
}

/// Payment history summary for a vendor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentHistory {
    /// Total number of invoices paid
    pub total_invoices: u32,
    /// Number of invoices paid on time
    pub on_time_payments: u32,
    /// Number of early payments (discount taken)
    pub early_payments: u32,
    /// Number of late payments
    pub late_payments: u32,
    /// Total payment amount
    #[serde(with = "rust_decimal::serde::str")]
    pub total_amount: Decimal,
    /// Average days to payment
    pub average_days_to_pay: f64,
    /// Last payment date
    pub last_payment_date: Option<NaiveDate>,
    /// Total discounts captured
    #[serde(with = "rust_decimal::serde::str")]
    pub total_discounts: Decimal,
}

impl Default for PaymentHistory {
    fn default() -> Self {
        Self {
            total_invoices: 0,
            on_time_payments: 0,
            early_payments: 0,
            late_payments: 0,
            total_amount: Decimal::ZERO,
            average_days_to_pay: 30.0,
            last_payment_date: None,
            total_discounts: Decimal::ZERO,
        }
    }
}

impl PaymentHistory {
    /// Calculate on-time payment rate.
    pub fn on_time_rate(&self) -> f64 {
        if self.total_invoices == 0 {
            1.0
        } else {
            self.on_time_payments as f64 / self.total_invoices as f64
        }
    }

    /// Calculate early payment rate.
    pub fn early_payment_rate(&self) -> f64 {
        if self.total_invoices == 0 {
            0.0
        } else {
            self.early_payments as f64 / self.total_invoices as f64
        }
    }

    /// Record a payment.
    pub fn record_payment(
        &mut self,
        amount: Decimal,
        payment_date: NaiveDate,
        due_date: NaiveDate,
        discount_taken: Decimal,
    ) {
        self.total_invoices += 1;
        self.total_amount += amount;
        self.last_payment_date = Some(payment_date);

        if discount_taken > Decimal::ZERO {
            self.early_payments += 1;
            self.total_discounts += discount_taken;
        } else if payment_date <= due_date {
            self.on_time_payments += 1;
        } else {
            self.late_payments += 1;
        }

        // Update running average days to pay
        let days = (payment_date - due_date).num_days() as f64;
        let n = self.total_invoices as f64;
        self.average_days_to_pay = ((self.average_days_to_pay * (n - 1.0)) + days) / n;
    }
}

/// Vendor quality score based on multiple dimensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorQualityScore {
    /// Delivery performance (0.0 - 1.0)
    pub delivery_score: f64,
    /// Quality of goods/services (0.0 - 1.0)
    pub quality_score: f64,
    /// Invoice accuracy (0.0 - 1.0)
    pub invoice_accuracy_score: f64,
    /// Responsiveness (0.0 - 1.0)
    pub responsiveness_score: f64,
    /// Last evaluation date
    pub last_evaluation: NaiveDate,
    /// Number of evaluations
    pub evaluation_count: u32,
}

impl Default for VendorQualityScore {
    fn default() -> Self {
        Self {
            delivery_score: 0.9,
            quality_score: 0.9,
            invoice_accuracy_score: 0.95,
            responsiveness_score: 0.85,
            last_evaluation: NaiveDate::from_ymd_opt(2020, 1, 1).expect("valid default date"),
            evaluation_count: 0,
        }
    }
}

impl VendorQualityScore {
    /// Calculate overall quality score (weighted average).
    pub fn overall_score(&self) -> f64 {
        const DELIVERY_WEIGHT: f64 = 0.30;
        const QUALITY_WEIGHT: f64 = 0.35;
        const INVOICE_WEIGHT: f64 = 0.20;
        const RESPONSIVENESS_WEIGHT: f64 = 0.15;

        self.delivery_score * DELIVERY_WEIGHT
            + self.quality_score * QUALITY_WEIGHT
            + self.invoice_accuracy_score * INVOICE_WEIGHT
            + self.responsiveness_score * RESPONSIVENESS_WEIGHT
    }

    /// Get the quality rating grade.
    pub fn grade(&self) -> &'static str {
        let score = self.overall_score();
        if score >= 0.95 {
            "A+"
        } else if score >= 0.90 {
            "A"
        } else if score >= 0.85 {
            "B+"
        } else if score >= 0.80 {
            "B"
        } else if score >= 0.70 {
            "C"
        } else if score >= 0.60 {
            "D"
        } else {
            "F"
        }
    }

    /// Update scores from an evaluation.
    pub fn update(
        &mut self,
        delivery: f64,
        quality: f64,
        invoice_accuracy: f64,
        responsiveness: f64,
        eval_date: NaiveDate,
    ) {
        // Exponential moving average with alpha = 0.3
        const ALPHA: f64 = 0.3;

        self.delivery_score = ALPHA * delivery + (1.0 - ALPHA) * self.delivery_score;
        self.quality_score = ALPHA * quality + (1.0 - ALPHA) * self.quality_score;
        self.invoice_accuracy_score =
            ALPHA * invoice_accuracy + (1.0 - ALPHA) * self.invoice_accuracy_score;
        self.responsiveness_score =
            ALPHA * responsiveness + (1.0 - ALPHA) * self.responsiveness_score;
        self.last_evaluation = eval_date;
        self.evaluation_count += 1;
    }
}

/// Substitutability classification for single-source analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Substitutability {
    /// Easily replaceable (~60% of vendors)
    #[default]
    Easy,
    /// Moderate effort to replace (~30% of vendors)
    Moderate,
    /// Difficult to replace (~10% of vendors)
    Difficult,
}

impl Substitutability {
    /// Get the typical distribution percentage.
    pub fn typical_distribution(&self) -> f64 {
        match self {
            Self::Easy => 0.60,
            Self::Moderate => 0.30,
            Self::Difficult => 0.10,
        }
    }

    /// Get the estimated replacement time in months.
    pub fn replacement_time_months(&self) -> u8 {
        match self {
            Self::Easy => 1,
            Self::Moderate => 3,
            Self::Difficult => 6,
        }
    }

    /// Get the risk factor for concentration analysis.
    pub fn risk_factor(&self) -> f64 {
        match self {
            Self::Easy => 1.0,
            Self::Moderate => 1.5,
            Self::Difficult => 2.5,
        }
    }
}

/// Vendor dependency tracking for concentration analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorDependency {
    /// Vendor ID
    pub vendor_id: String,
    /// Is this a single-source vendor?
    pub is_single_source: bool,
    /// How easily can this vendor be replaced?
    pub substitutability: Substitutability,
    /// Concentration percentage (spend with this vendor / total category spend)
    pub concentration_percent: f64,
    /// Category of spend
    pub spend_category: String,
    /// Alternative vendors if available
    pub alternatives: Vec<String>,
    /// Last review date
    pub last_review_date: Option<NaiveDate>,
}

impl VendorDependency {
    /// Create a new vendor dependency record.
    pub fn new(vendor_id: impl Into<String>, spend_category: impl Into<String>) -> Self {
        Self {
            vendor_id: vendor_id.into(),
            is_single_source: false,
            substitutability: Substitutability::default(),
            concentration_percent: 0.0,
            spend_category: spend_category.into(),
            alternatives: Vec::new(),
            last_review_date: None,
        }
    }

    /// Calculate dependency risk score.
    pub fn risk_score(&self) -> f64 {
        let single_source_factor = if self.is_single_source { 2.0 } else { 1.0 };
        let concentration_factor = self.concentration_percent;
        let substitutability_factor = self.substitutability.risk_factor();

        // Composite risk score (0.0 to ~5.0)
        single_source_factor * concentration_factor * substitutability_factor
    }

    /// Check if this represents high risk.
    pub fn is_high_risk(&self) -> bool {
        self.is_single_source
            && matches!(self.substitutability, Substitutability::Difficult)
            && self.concentration_percent > 0.15
    }
}

/// Vendor relationship in the supply chain network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorRelationship {
    /// Vendor ID
    pub vendor_id: String,
    /// Type of relationship
    pub relationship_type: VendorRelationshipType,
    /// Supply chain tier
    pub tier: SupplyChainTier,
    /// Strategic importance level
    pub strategic_importance: StrategicLevel,
    /// Spend tier classification
    pub spend_tier: SpendTier,
    /// Behavioral cluster
    pub cluster: VendorCluster,
    /// Relationship start date
    pub start_date: NaiveDate,
    /// Relationship end date (if terminated)
    pub end_date: Option<NaiveDate>,
    /// Current lifecycle stage
    pub lifecycle_stage: VendorLifecycleStage,
    /// Payment history summary
    pub payment_history: PaymentHistory,
    /// Quality score
    pub quality_score: VendorQualityScore,
    /// Parent vendor ID (for Tier 2/3)
    pub parent_vendor: Option<String>,
    /// Child vendor IDs (suppliers to this vendor)
    pub child_vendors: Vec<String>,
    /// Dependency analysis
    pub dependency: Option<VendorDependency>,
    /// Annual spend amount
    #[serde(with = "rust_decimal::serde::str")]
    pub annual_spend: Decimal,
    /// Contract reference
    pub contract_id: Option<String>,
    /// Primary contact
    pub primary_contact: Option<String>,
    /// Notes
    pub notes: Option<String>,
}

impl VendorRelationship {
    /// Create a new vendor relationship.
    pub fn new(
        vendor_id: impl Into<String>,
        relationship_type: VendorRelationshipType,
        tier: SupplyChainTier,
        start_date: NaiveDate,
    ) -> Self {
        Self {
            vendor_id: vendor_id.into(),
            relationship_type,
            tier,
            strategic_importance: StrategicLevel::default(),
            spend_tier: SpendTier::default(),
            cluster: VendorCluster::default(),
            start_date,
            end_date: None,
            lifecycle_stage: VendorLifecycleStage::Onboarding {
                started: start_date,
                expected_completion: start_date + chrono::Duration::days(90),
            },
            payment_history: PaymentHistory::default(),
            quality_score: VendorQualityScore::default(),
            parent_vendor: None,
            child_vendors: Vec::new(),
            dependency: None,
            annual_spend: Decimal::ZERO,
            contract_id: None,
            primary_contact: None,
            notes: None,
        }
    }

    /// Set strategic importance.
    pub fn with_strategic_importance(mut self, level: StrategicLevel) -> Self {
        self.strategic_importance = level;
        self
    }

    /// Set spend tier.
    pub fn with_spend_tier(mut self, tier: SpendTier) -> Self {
        self.spend_tier = tier;
        self
    }

    /// Set cluster.
    pub fn with_cluster(mut self, cluster: VendorCluster) -> Self {
        self.cluster = cluster;
        self
    }

    /// Set parent vendor (for Tier 2/3).
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_vendor = Some(parent_id.into());
        self
    }

    /// Add a child vendor.
    pub fn add_child(&mut self, child_id: impl Into<String>) {
        self.child_vendors.push(child_id.into());
    }

    /// Set annual spend.
    pub fn with_annual_spend(mut self, spend: Decimal) -> Self {
        self.annual_spend = spend;
        self
    }

    /// Check if relationship is active.
    pub fn is_active(&self) -> bool {
        self.end_date.is_none() && self.lifecycle_stage.is_active()
    }

    /// Calculate relationship age in days.
    pub fn relationship_age_days(&self, as_of: NaiveDate) -> i64 {
        (as_of - self.start_date).num_days()
    }

    /// Get composite relationship score.
    pub fn relationship_score(&self) -> f64 {
        let quality = self.quality_score.overall_score();
        let payment = self.payment_history.on_time_rate();
        let strategic = self.strategic_importance.importance_score();
        let cluster_bonus = match self.cluster {
            VendorCluster::ReliableStrategic => 0.1,
            VendorCluster::StandardOperational => 0.0,
            VendorCluster::Transactional => -0.05,
            VendorCluster::Problematic => -0.15,
        };

        // Weighted composite score
        (quality * 0.4 + payment * 0.3 + strategic * 0.3 + cluster_bonus).clamp(0.0, 1.0)
    }
}

/// Multi-tier vendor network for a company.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VendorNetwork {
    /// Company code owning this network
    pub company_code: String,
    /// All vendor relationships
    pub relationships: HashMap<String, VendorRelationship>,
    /// Tier 1 vendor IDs
    pub tier1_vendors: Vec<String>,
    /// Tier 2 vendor IDs
    pub tier2_vendors: Vec<String>,
    /// Tier 3 vendor IDs
    pub tier3_vendors: Vec<String>,
    /// Network creation date
    pub created_date: Option<NaiveDate>,
    /// Network statistics
    pub statistics: NetworkStatistics,
}

/// Statistics for the vendor network.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkStatistics {
    /// Total vendor count
    pub total_vendors: usize,
    /// Active vendor count
    pub active_vendors: usize,
    /// Total annual spend
    #[serde(with = "rust_decimal::serde::str")]
    pub total_annual_spend: Decimal,
    /// Average relationship age in days
    pub avg_relationship_age_days: f64,
    /// Concentration in top 5 vendors
    pub top5_concentration: f64,
    /// Single-source vendor count
    pub single_source_count: usize,
    /// Cluster distribution
    pub cluster_distribution: HashMap<String, f64>,
}

impl VendorNetwork {
    /// Create a new vendor network.
    pub fn new(company_code: impl Into<String>) -> Self {
        Self {
            company_code: company_code.into(),
            relationships: HashMap::new(),
            tier1_vendors: Vec::new(),
            tier2_vendors: Vec::new(),
            tier3_vendors: Vec::new(),
            created_date: None,
            statistics: NetworkStatistics::default(),
        }
    }

    /// Add a vendor relationship.
    pub fn add_relationship(&mut self, relationship: VendorRelationship) {
        let vendor_id = relationship.vendor_id.clone();
        match relationship.tier {
            SupplyChainTier::Tier1 => self.tier1_vendors.push(vendor_id.clone()),
            SupplyChainTier::Tier2 => self.tier2_vendors.push(vendor_id.clone()),
            SupplyChainTier::Tier3 => self.tier3_vendors.push(vendor_id.clone()),
        }
        self.relationships.insert(vendor_id, relationship);
    }

    /// Get a relationship by vendor ID.
    pub fn get_relationship(&self, vendor_id: &str) -> Option<&VendorRelationship> {
        self.relationships.get(vendor_id)
    }

    /// Get a mutable relationship by vendor ID.
    pub fn get_relationship_mut(&mut self, vendor_id: &str) -> Option<&mut VendorRelationship> {
        self.relationships.get_mut(vendor_id)
    }

    /// Get all vendors in a tier.
    pub fn vendors_in_tier(&self, tier: SupplyChainTier) -> Vec<&VendorRelationship> {
        let ids = match tier {
            SupplyChainTier::Tier1 => &self.tier1_vendors,
            SupplyChainTier::Tier2 => &self.tier2_vendors,
            SupplyChainTier::Tier3 => &self.tier3_vendors,
        };
        ids.iter()
            .filter_map(|id| self.relationships.get(id))
            .collect()
    }

    /// Get child vendors (Tier N+1) of a given vendor.
    pub fn get_children(&self, vendor_id: &str) -> Vec<&VendorRelationship> {
        self.relationships
            .get(vendor_id)
            .map(|rel| {
                rel.child_vendors
                    .iter()
                    .filter_map(|id| self.relationships.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get parent vendor (Tier N-1) of a given vendor.
    pub fn get_parent(&self, vendor_id: &str) -> Option<&VendorRelationship> {
        self.relationships
            .get(vendor_id)
            .and_then(|rel| rel.parent_vendor.as_ref())
            .and_then(|parent_id| self.relationships.get(parent_id))
    }

    /// Calculate network statistics.
    pub fn calculate_statistics(&mut self, as_of: NaiveDate) {
        let active_count = self
            .relationships
            .values()
            .filter(|r| r.is_active())
            .count();

        let total_spend: Decimal = self.relationships.values().map(|r| r.annual_spend).sum();

        let avg_age = if self.relationships.is_empty() {
            0.0
        } else {
            self.relationships
                .values()
                .map(|r| r.relationship_age_days(as_of) as f64)
                .sum::<f64>()
                / self.relationships.len() as f64
        };

        // Calculate top 5 concentration
        let mut spends: Vec<Decimal> = self
            .relationships
            .values()
            .map(|r| r.annual_spend)
            .collect();
        spends.sort_by(|a, b| b.cmp(a));
        let top5_spend: Decimal = spends.iter().take(5).copied().sum();
        let top5_conc = if total_spend > Decimal::ZERO {
            (top5_spend / total_spend)
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        // Count single-source vendors
        let single_source = self
            .relationships
            .values()
            .filter(|r| {
                r.dependency
                    .as_ref()
                    .map(|d| d.is_single_source)
                    .unwrap_or(false)
            })
            .count();

        // Calculate cluster distribution
        let mut cluster_counts: HashMap<String, usize> = HashMap::new();
        for rel in self.relationships.values() {
            *cluster_counts
                .entry(format!("{:?}", rel.cluster))
                .or_insert(0) += 1;
        }
        let cluster_distribution: HashMap<String, f64> = cluster_counts
            .into_iter()
            .map(|(k, v)| (k, v as f64 / self.relationships.len().max(1) as f64))
            .collect();

        self.statistics = NetworkStatistics {
            total_vendors: self.relationships.len(),
            active_vendors: active_count,
            total_annual_spend: total_spend,
            avg_relationship_age_days: avg_age,
            top5_concentration: top5_conc,
            single_source_count: single_source,
            cluster_distribution,
        };
    }

    /// Check concentration limits.
    pub fn check_concentration_limits(&self, max_single_vendor: f64, max_top5: f64) -> Vec<String> {
        let mut violations = Vec::new();

        // Check individual vendor concentration
        let total_spend: Decimal = self.relationships.values().map(|r| r.annual_spend).sum();
        if total_spend > Decimal::ZERO {
            for rel in self.relationships.values() {
                let conc = (rel.annual_spend / total_spend)
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0);
                if conc > max_single_vendor {
                    violations.push(format!(
                        "Vendor {} concentration {:.1}% exceeds limit {:.1}%",
                        rel.vendor_id,
                        conc * 100.0,
                        max_single_vendor * 100.0
                    ));
                }
            }
        }

        // Check top 5 concentration
        if self.statistics.top5_concentration > max_top5 {
            violations.push(format!(
                "Top 5 vendor concentration {:.1}% exceeds limit {:.1}%",
                self.statistics.top5_concentration * 100.0,
                max_top5 * 100.0
            ));
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supply_chain_tier() {
        assert_eq!(SupplyChainTier::Tier1.tier_number(), 1);
        assert_eq!(SupplyChainTier::Tier2.visibility(), 0.5);
        assert_eq!(
            SupplyChainTier::Tier1.child_tier(),
            Some(SupplyChainTier::Tier2)
        );
        assert_eq!(SupplyChainTier::Tier3.child_tier(), None);
    }

    #[test]
    fn test_vendor_cluster_distribution() {
        let total: f64 = [
            VendorCluster::ReliableStrategic,
            VendorCluster::StandardOperational,
            VendorCluster::Transactional,
            VendorCluster::Problematic,
        ]
        .iter()
        .map(|c| c.typical_distribution())
        .sum();

        assert!((total - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_vendor_quality_score() {
        let mut score = VendorQualityScore::default();
        assert!(score.overall_score() > 0.8);

        score.update(
            0.95,
            0.90,
            0.98,
            0.85,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );
        assert_eq!(score.evaluation_count, 1);
        assert_eq!(score.grade(), "A");
    }

    #[test]
    fn test_payment_history() {
        let mut history = PaymentHistory::default();
        history.record_payment(
            Decimal::from(1000),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
            Decimal::ZERO,
        );

        assert_eq!(history.total_invoices, 1);
        assert_eq!(history.on_time_payments, 1);
        assert!((history.on_time_rate() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_vendor_relationship() {
        let rel = VendorRelationship::new(
            "V-001",
            VendorRelationshipType::DirectSupplier,
            SupplyChainTier::Tier1,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )
        .with_strategic_importance(StrategicLevel::Critical)
        .with_spend_tier(SpendTier::Platinum)
        .with_cluster(VendorCluster::ReliableStrategic)
        .with_annual_spend(Decimal::from(1000000));

        assert!(rel.is_active());
        assert_eq!(rel.strategic_importance, StrategicLevel::Critical);
        assert!(rel.relationship_score() > 0.5);
    }

    #[test]
    fn test_vendor_network() {
        let mut network = VendorNetwork::new("1000");

        let rel1 = VendorRelationship::new(
            "V-001",
            VendorRelationshipType::DirectSupplier,
            SupplyChainTier::Tier1,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )
        .with_annual_spend(Decimal::from(500000));

        let rel2 = VendorRelationship::new(
            "V-002",
            VendorRelationshipType::RawMaterialSupplier,
            SupplyChainTier::Tier2,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )
        .with_parent("V-001")
        .with_annual_spend(Decimal::from(200000));

        network.add_relationship(rel1);
        network.add_relationship(rel2);

        assert_eq!(network.tier1_vendors.len(), 1);
        assert_eq!(network.tier2_vendors.len(), 1);
        assert!(network.get_relationship("V-001").is_some());

        network.calculate_statistics(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap());
        assert_eq!(network.statistics.total_vendors, 2);
        assert_eq!(network.statistics.active_vendors, 2);
    }

    #[test]
    fn test_vendor_dependency() {
        let mut dep = VendorDependency::new("V-001", "Raw Materials");
        dep.is_single_source = true;
        dep.substitutability = Substitutability::Difficult;
        dep.concentration_percent = 0.25; // 25% concentration

        assert!(dep.is_high_risk());
        // Risk score = 2.0 (single_source) * 0.25 (concentration) * 2.5 (difficult) = 1.25
        assert!(dep.risk_score() > 1.0);
    }

    #[test]
    fn test_lifecycle_stage() {
        let stage = VendorLifecycleStage::SteadyState {
            since: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        };
        assert!(stage.is_active());
        assert!(stage.is_good_standing());

        let terminated = VendorLifecycleStage::Terminated {
            date: NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            reason: TerminationReason::ContractExpired,
        };
        assert!(!terminated.is_active());
    }
}
