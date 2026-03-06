//! Customer segmentation and lifecycle models.
//!
//! Provides comprehensive customer relationship modeling including:
//! - Value-based segmentation (Enterprise, Mid-Market, SMB, Consumer)
//! - Customer lifecycle stages with transition tracking
//! - Network position for referrals and corporate hierarchies
//! - Revenue attribution and churn prediction

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Customer value segment classification.
///
/// Based on research showing typical B2B/B2C customer distributions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CustomerValueSegment {
    /// Enterprise customers (~5% of customers, ~40% of revenue)
    Enterprise,
    /// Mid-market customers (~20% of customers, ~35% of revenue)
    #[default]
    MidMarket,
    /// Small/medium business (~50% of customers, ~20% of revenue)
    Smb,
    /// Consumer/individual (~25% of customers, ~5% of revenue)
    Consumer,
}

impl CustomerValueSegment {
    /// Get the typical customer share percentage.
    pub fn customer_share(&self) -> f64 {
        match self {
            Self::Enterprise => 0.05,
            Self::MidMarket => 0.20,
            Self::Smb => 0.50,
            Self::Consumer => 0.25,
        }
    }

    /// Get the typical revenue share percentage.
    pub fn revenue_share(&self) -> f64 {
        match self {
            Self::Enterprise => 0.40,
            Self::MidMarket => 0.35,
            Self::Smb => 0.20,
            Self::Consumer => 0.05,
        }
    }

    /// Get the typical order value range.
    pub fn order_value_range(&self) -> (Decimal, Decimal) {
        match self {
            Self::Enterprise => (Decimal::from(50000), Decimal::from(5000000)),
            Self::MidMarket => (Decimal::from(5000), Decimal::from(50000)),
            Self::Smb => (Decimal::from(500), Decimal::from(5000)),
            Self::Consumer => (Decimal::from(50), Decimal::from(500)),
        }
    }

    /// Get the segment code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Enterprise => "ENT",
            Self::MidMarket => "MID",
            Self::Smb => "SMB",
            Self::Consumer => "CON",
        }
    }

    /// Get the typical service level.
    pub fn service_level(&self) -> &'static str {
        match self {
            Self::Enterprise => "dedicated_team",
            Self::MidMarket => "named_account_manager",
            Self::Smb => "shared_support",
            Self::Consumer => "self_service",
        }
    }

    /// Get the typical payment terms in days.
    pub fn typical_payment_terms_days(&self) -> u16 {
        match self {
            Self::Enterprise => 60,
            Self::MidMarket => 45,
            Self::Smb => 30,
            Self::Consumer => 0, // Immediate
        }
    }

    /// Get the strategic importance score.
    pub fn importance_score(&self) -> f64 {
        match self {
            Self::Enterprise => 1.0,
            Self::MidMarket => 0.7,
            Self::Smb => 0.4,
            Self::Consumer => 0.2,
        }
    }
}

/// Risk triggers for at-risk customers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskTrigger {
    /// Declining order frequency
    DecliningOrderFrequency,
    /// Declining order value
    DecliningOrderValue,
    /// Payment issues
    PaymentIssues,
    /// Complaints or support tickets
    Complaints,
    /// Reduced engagement
    ReducedEngagement,
    /// Competitor mention
    CompetitorMention,
    /// Contract expiring soon
    ContractExpiring,
    /// Key contact departure
    ContactDeparture,
    /// Budget cuts announced
    BudgetCuts,
    /// Organizational restructuring
    Restructuring,
    /// Custom trigger
    Other(String),
}

impl RiskTrigger {
    /// Get the severity score (0.0 to 1.0).
    pub fn severity(&self) -> f64 {
        match self {
            Self::DecliningOrderFrequency => 0.6,
            Self::DecliningOrderValue => 0.5,
            Self::PaymentIssues => 0.8,
            Self::Complaints => 0.7,
            Self::ReducedEngagement => 0.4,
            Self::CompetitorMention => 0.9,
            Self::ContractExpiring => 0.5,
            Self::ContactDeparture => 0.6,
            Self::BudgetCuts => 0.7,
            Self::Restructuring => 0.5,
            Self::Other(_) => 0.5,
        }
    }
}

/// Churn reason for lost customers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChurnReason {
    /// Price was too high
    Price,
    /// Switched to competitor
    Competitor,
    /// Poor service quality
    ServiceQuality,
    /// Product didn't meet needs
    ProductFit,
    /// Business closure
    BusinessClosed,
    /// Budget constraints
    BudgetConstraints,
    /// Internal consolidation
    Consolidation,
    /// Acquisition by another company
    Acquisition,
    /// Natural end of need
    ProjectCompleted,
    /// Unknown reason
    Unknown,
    /// Other reason
    Other(String),
}

/// Customer lifecycle stage with metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomerLifecycleStage {
    /// Potential customer, not yet converted
    Prospect {
        /// Probability of conversion (0.0 to 1.0)
        conversion_probability: f64,
        /// Lead source
        source: Option<String>,
        /// First contact date
        first_contact_date: NaiveDate,
    },
    /// Newly acquired customer
    New {
        /// Date of first order
        first_order_date: NaiveDate,
        /// Onboarding completed
        onboarding_complete: bool,
    },
    /// Growing customer (increasing spend)
    Growth {
        /// Start of growth phase
        since: NaiveDate,
        /// Year-over-year growth rate
        growth_rate: f64,
    },
    /// Mature customer (stable relationship)
    Mature {
        /// Date when stable state achieved
        stable_since: NaiveDate,
        /// Average annual spend
        #[serde(with = "rust_decimal::serde::str")]
        avg_annual_spend: Decimal,
    },
    /// Customer showing churn signals
    AtRisk {
        /// Risk triggers detected
        triggers: Vec<RiskTrigger>,
        /// Date when flagged at-risk
        flagged_date: NaiveDate,
        /// Estimated churn probability
        churn_probability: f64,
    },
    /// Customer has churned
    Churned {
        /// Date of last activity
        last_activity: NaiveDate,
        /// Probability of win-back
        win_back_probability: f64,
        /// Reason for churn
        reason: Option<ChurnReason>,
    },
    /// Won-back customer
    WonBack {
        /// Original churn date
        churned_date: NaiveDate,
        /// Win-back date
        won_back_date: NaiveDate,
    },
}

impl CustomerLifecycleStage {
    /// Check if the customer is active.
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Prospect { .. } | Self::Churned { .. })
    }

    /// Check if the customer is in good standing.
    pub fn is_good_standing(&self) -> bool {
        matches!(
            self,
            Self::New { .. } | Self::Growth { .. } | Self::Mature { .. } | Self::WonBack { .. }
        )
    }

    /// Get the stage name.
    pub fn stage_name(&self) -> &'static str {
        match self {
            Self::Prospect { .. } => "prospect",
            Self::New { .. } => "new",
            Self::Growth { .. } => "growth",
            Self::Mature { .. } => "mature",
            Self::AtRisk { .. } => "at_risk",
            Self::Churned { .. } => "churned",
            Self::WonBack { .. } => "won_back",
        }
    }

    /// Get the retention priority (1 = highest).
    pub fn retention_priority(&self) -> u8 {
        match self {
            Self::AtRisk { .. } => 1,
            Self::Growth { .. } => 2,
            Self::Mature { .. } => 3,
            Self::New { .. } => 4,
            Self::WonBack { .. } => 5,
            Self::Churned { .. } => 6,
            Self::Prospect { .. } => 7,
        }
    }
}

impl Default for CustomerLifecycleStage {
    fn default() -> Self {
        Self::Mature {
            stable_since: NaiveDate::from_ymd_opt(2020, 1, 1).expect("valid default date"),
            avg_annual_spend: Decimal::from(50000),
        }
    }
}

/// Customer network position for referrals and hierarchies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerNetworkPosition {
    /// Customer ID
    pub customer_id: String,
    /// Customer who referred this customer
    pub referred_by: Option<String>,
    /// Customers this customer referred
    pub referrals_made: Vec<String>,
    /// Parent customer in corporate hierarchy
    pub parent_customer: Option<String>,
    /// Child customers in corporate hierarchy
    pub child_customers: Vec<String>,
    /// Whether billing is consolidated to parent
    pub billing_consolidation: bool,
    /// Industry cluster for similar customer analysis
    pub industry_cluster_id: Option<String>,
    /// Geographic region
    pub region: Option<String>,
    /// Date joined the network
    pub network_join_date: Option<NaiveDate>,
}

impl CustomerNetworkPosition {
    /// Create a new network position.
    pub fn new(customer_id: impl Into<String>) -> Self {
        Self {
            customer_id: customer_id.into(),
            referred_by: None,
            referrals_made: Vec::new(),
            parent_customer: None,
            child_customers: Vec::new(),
            billing_consolidation: false,
            industry_cluster_id: None,
            region: None,
            network_join_date: None,
        }
    }

    /// Set referral source.
    pub fn with_referral(mut self, referrer_id: impl Into<String>) -> Self {
        self.referred_by = Some(referrer_id.into());
        self
    }

    /// Set parent in corporate hierarchy.
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_customer = Some(parent_id.into());
        self
    }

    /// Add a referral made.
    pub fn add_referral(&mut self, referred_id: impl Into<String>) {
        self.referrals_made.push(referred_id.into());
    }

    /// Add a child customer.
    pub fn add_child(&mut self, child_id: impl Into<String>) {
        self.child_customers.push(child_id.into());
    }

    /// Get total network influence (referrals + children).
    pub fn network_influence(&self) -> usize {
        self.referrals_made.len() + self.child_customers.len()
    }

    /// Check if this is a root customer (no parent).
    pub fn is_root(&self) -> bool {
        self.parent_customer.is_none()
    }

    /// Check if this customer was referred.
    pub fn was_referred(&self) -> bool {
        self.referred_by.is_some()
    }
}

/// Customer engagement metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerEngagement {
    /// Total orders placed
    pub total_orders: u32,
    /// Orders in the last 12 months
    pub orders_last_12_months: u32,
    /// Total revenue (lifetime)
    #[serde(with = "rust_decimal::serde::str")]
    pub lifetime_revenue: Decimal,
    /// Revenue in the last 12 months
    #[serde(with = "rust_decimal::serde::str")]
    pub revenue_last_12_months: Decimal,
    /// Average order value
    #[serde(with = "rust_decimal::serde::str")]
    pub average_order_value: Decimal,
    /// Days since last order
    pub days_since_last_order: u32,
    /// Last order date
    pub last_order_date: Option<NaiveDate>,
    /// First order date
    pub first_order_date: Option<NaiveDate>,
    /// Number of products purchased
    pub products_purchased: u32,
    /// Support tickets created
    pub support_tickets: u32,
    /// Net promoter score (if available)
    pub nps_score: Option<i8>,
}

impl Default for CustomerEngagement {
    fn default() -> Self {
        Self {
            total_orders: 0,
            orders_last_12_months: 0,
            lifetime_revenue: Decimal::ZERO,
            revenue_last_12_months: Decimal::ZERO,
            average_order_value: Decimal::ZERO,
            days_since_last_order: 0,
            last_order_date: None,
            first_order_date: None,
            products_purchased: 0,
            support_tickets: 0,
            nps_score: None,
        }
    }
}

impl CustomerEngagement {
    /// Record an order.
    pub fn record_order(&mut self, amount: Decimal, order_date: NaiveDate, product_count: u32) {
        self.total_orders += 1;
        self.lifetime_revenue += amount;
        self.products_purchased += product_count;

        // Update average order value
        if self.total_orders > 0 {
            self.average_order_value = self.lifetime_revenue / Decimal::from(self.total_orders);
        }

        // Update first/last order dates
        if self.first_order_date.is_none() {
            self.first_order_date = Some(order_date);
        }
        self.last_order_date = Some(order_date);
        self.days_since_last_order = 0;
    }

    /// Update days since last order (call periodically).
    pub fn update_days_since_last_order(&mut self, current_date: NaiveDate) {
        if let Some(last_order) = self.last_order_date {
            self.days_since_last_order = (current_date - last_order).num_days().max(0) as u32;
        }
    }

    /// Calculate customer health score (0.0 to 1.0).
    pub fn health_score(&self) -> f64 {
        let mut score = 0.0;

        // Order frequency component (30%)
        let order_freq_score = if self.orders_last_12_months > 0 {
            (self.orders_last_12_months as f64 / 12.0).min(1.0)
        } else {
            0.0
        };
        score += 0.30 * order_freq_score;

        // Recency component (30%)
        let recency_score = if self.days_since_last_order == 0 {
            1.0
        } else {
            (1.0 - (self.days_since_last_order as f64 / 365.0)).max(0.0)
        };
        score += 0.30 * recency_score;

        // Value component (25%)
        let value_score = if self.average_order_value > Decimal::ZERO {
            let aov_f64 = self
                .average_order_value
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0);
            (aov_f64 / 10000.0).min(1.0) // Normalize to $10k
        } else {
            0.0
        };
        score += 0.25 * value_score;

        // NPS component (15%)
        if let Some(nps) = self.nps_score {
            // Cast to i32 first to avoid overflow (NPS is -100 to +100, i8 range is -128 to +127)
            let nps_normalized = ((nps as i32 + 100) as f64 / 200.0).clamp(0.0, 1.0);
            score += 0.15 * nps_normalized;
        } else {
            score += 0.15 * 0.5; // Neutral if no NPS
        }

        score.clamp(0.0, 1.0)
    }
}

/// Segmented customer record with full metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentedCustomer {
    /// Customer ID
    pub customer_id: String,
    /// Customer name
    pub name: String,
    /// Value segment
    pub segment: CustomerValueSegment,
    /// Current lifecycle stage
    pub lifecycle_stage: CustomerLifecycleStage,
    /// Network position
    pub network_position: CustomerNetworkPosition,
    /// Engagement metrics
    pub engagement: CustomerEngagement,
    /// Segment assignment date
    pub segment_assigned_date: NaiveDate,
    /// Previous segment (if changed)
    pub previous_segment: Option<CustomerValueSegment>,
    /// Segment change date
    pub segment_change_date: Option<NaiveDate>,
    /// Industry
    pub industry: Option<String>,
    /// Annual contract value
    #[serde(with = "rust_decimal::serde::str")]
    pub annual_contract_value: Decimal,
    /// Churn risk score (0.0 to 1.0)
    pub churn_risk_score: f64,
    /// Upsell potential score (0.0 to 1.0)
    pub upsell_potential: f64,
    /// Account manager
    pub account_manager: Option<String>,
}

impl SegmentedCustomer {
    /// Create a new segmented customer.
    pub fn new(
        customer_id: impl Into<String>,
        name: impl Into<String>,
        segment: CustomerValueSegment,
        assignment_date: NaiveDate,
    ) -> Self {
        let customer_id = customer_id.into();
        Self {
            customer_id: customer_id.clone(),
            name: name.into(),
            segment,
            lifecycle_stage: CustomerLifecycleStage::default(),
            network_position: CustomerNetworkPosition::new(customer_id),
            engagement: CustomerEngagement::default(),
            segment_assigned_date: assignment_date,
            previous_segment: None,
            segment_change_date: None,
            industry: None,
            annual_contract_value: Decimal::ZERO,
            churn_risk_score: 0.0,
            upsell_potential: 0.5,
            account_manager: None,
        }
    }

    /// Set lifecycle stage.
    pub fn with_lifecycle_stage(mut self, stage: CustomerLifecycleStage) -> Self {
        self.lifecycle_stage = stage;
        self
    }

    /// Set industry.
    pub fn with_industry(mut self, industry: impl Into<String>) -> Self {
        self.industry = Some(industry.into());
        self
    }

    /// Set annual contract value.
    pub fn with_annual_contract_value(mut self, value: Decimal) -> Self {
        self.annual_contract_value = value;
        self
    }

    /// Change segment (with history tracking).
    pub fn change_segment(&mut self, new_segment: CustomerValueSegment, change_date: NaiveDate) {
        if self.segment != new_segment {
            self.previous_segment = Some(self.segment);
            self.segment = new_segment;
            self.segment_change_date = Some(change_date);
        }
    }

    /// Update churn risk score based on engagement and lifecycle.
    pub fn calculate_churn_risk(&mut self) {
        let mut risk = 0.0;

        // Lifecycle stage risk
        match &self.lifecycle_stage {
            CustomerLifecycleStage::AtRisk {
                churn_probability, ..
            } => {
                risk += 0.4 * churn_probability;
            }
            CustomerLifecycleStage::New { .. } => risk += 0.15,
            CustomerLifecycleStage::WonBack { .. } => risk += 0.25,
            CustomerLifecycleStage::Growth { .. } => risk += 0.05,
            CustomerLifecycleStage::Mature { .. } => risk += 0.10,
            _ => {}
        }

        // Engagement health
        let health = self.engagement.health_score();
        risk += 0.4 * (1.0 - health);

        // Days since last order
        let recency_risk = (self.engagement.days_since_last_order as f64 / 180.0).min(1.0);
        risk += 0.2 * recency_risk;

        self.churn_risk_score = risk.clamp(0.0, 1.0);
    }

    /// Get customer lifetime value estimate.
    pub fn estimated_lifetime_value(&self) -> Decimal {
        // Simple CLV: ACV * expected relationship duration
        let expected_years = match self.segment {
            CustomerValueSegment::Enterprise => Decimal::from(8),
            CustomerValueSegment::MidMarket => Decimal::from(5),
            CustomerValueSegment::Smb => Decimal::from(3),
            CustomerValueSegment::Consumer => Decimal::from(2),
        };
        let retention_factor =
            Decimal::from_f64_retain(1.0 - self.churn_risk_score).unwrap_or(Decimal::ONE);
        self.annual_contract_value * expected_years * retention_factor
    }

    /// Check if customer is high value.
    pub fn is_high_value(&self) -> bool {
        matches!(
            self.segment,
            CustomerValueSegment::Enterprise | CustomerValueSegment::MidMarket
        )
    }
}

/// Pool of segmented customers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SegmentedCustomerPool {
    /// All segmented customers
    pub customers: Vec<SegmentedCustomer>,
    /// Index by segment
    #[serde(skip)]
    segment_index: HashMap<CustomerValueSegment, Vec<usize>>,
    /// Index by lifecycle stage name
    #[serde(skip)]
    lifecycle_index: HashMap<String, Vec<usize>>,
    /// Pool statistics
    pub statistics: SegmentStatistics,
}

/// Statistics for the segmented customer pool.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SegmentStatistics {
    /// Total customers by segment
    pub customers_by_segment: HashMap<String, usize>,
    /// Revenue by segment
    pub revenue_by_segment: HashMap<String, Decimal>,
    /// Total revenue
    #[serde(with = "rust_decimal::serde::str")]
    pub total_revenue: Decimal,
    /// Average churn risk
    pub avg_churn_risk: f64,
    /// Referral rate
    pub referral_rate: f64,
    /// Customers at risk
    pub at_risk_count: usize,
}

impl SegmentedCustomerPool {
    /// Create a new empty pool.
    pub fn new() -> Self {
        Self {
            customers: Vec::new(),
            segment_index: HashMap::new(),
            lifecycle_index: HashMap::new(),
            statistics: SegmentStatistics::default(),
        }
    }

    /// Add a customer to the pool.
    pub fn add_customer(&mut self, customer: SegmentedCustomer) {
        let idx = self.customers.len();
        let segment = customer.segment;
        let stage_name = customer.lifecycle_stage.stage_name().to_string();

        self.customers.push(customer);

        self.segment_index.entry(segment).or_default().push(idx);
        self.lifecycle_index
            .entry(stage_name)
            .or_default()
            .push(idx);
    }

    /// Get customers by segment.
    pub fn by_segment(&self, segment: CustomerValueSegment) -> Vec<&SegmentedCustomer> {
        self.segment_index
            .get(&segment)
            .map(|indices| indices.iter().map(|&idx| &self.customers[idx]).collect())
            .unwrap_or_default()
    }

    /// Get customers by lifecycle stage.
    pub fn by_lifecycle_stage(&self, stage_name: &str) -> Vec<&SegmentedCustomer> {
        self.lifecycle_index
            .get(stage_name)
            .map(|indices| indices.iter().map(|&idx| &self.customers[idx]).collect())
            .unwrap_or_default()
    }

    /// Get at-risk customers.
    pub fn at_risk_customers(&self) -> Vec<&SegmentedCustomer> {
        self.customers
            .iter()
            .filter(|c| matches!(c.lifecycle_stage, CustomerLifecycleStage::AtRisk { .. }))
            .collect()
    }

    /// Get high-value customers.
    pub fn high_value_customers(&self) -> Vec<&SegmentedCustomer> {
        self.customers
            .iter()
            .filter(|c| c.is_high_value())
            .collect()
    }

    /// Rebuild indexes (call after deserialization).
    pub fn rebuild_indexes(&mut self) {
        self.segment_index.clear();
        self.lifecycle_index.clear();

        for (idx, customer) in self.customers.iter().enumerate() {
            self.segment_index
                .entry(customer.segment)
                .or_default()
                .push(idx);
            self.lifecycle_index
                .entry(customer.lifecycle_stage.stage_name().to_string())
                .or_default()
                .push(idx);
        }
    }

    /// Calculate pool statistics.
    pub fn calculate_statistics(&mut self) {
        let mut customers_by_segment: HashMap<String, usize> = HashMap::new();
        let mut revenue_by_segment: HashMap<String, Decimal> = HashMap::new();
        let mut total_revenue = Decimal::ZERO;
        let mut total_churn_risk = 0.0;
        let mut referral_count = 0usize;
        let mut at_risk_count = 0usize;

        for customer in &self.customers {
            let segment_name = format!("{:?}", customer.segment);

            *customers_by_segment
                .entry(segment_name.clone())
                .or_insert(0) += 1;
            *revenue_by_segment
                .entry(segment_name)
                .or_insert(Decimal::ZERO) += customer.annual_contract_value;

            total_revenue += customer.annual_contract_value;
            total_churn_risk += customer.churn_risk_score;

            if customer.network_position.was_referred() {
                referral_count += 1;
            }

            if matches!(
                customer.lifecycle_stage,
                CustomerLifecycleStage::AtRisk { .. }
            ) {
                at_risk_count += 1;
            }
        }

        let avg_churn_risk = if self.customers.is_empty() {
            0.0
        } else {
            total_churn_risk / self.customers.len() as f64
        };

        let referral_rate = if self.customers.is_empty() {
            0.0
        } else {
            referral_count as f64 / self.customers.len() as f64
        };

        self.statistics = SegmentStatistics {
            customers_by_segment,
            revenue_by_segment,
            total_revenue,
            avg_churn_risk,
            referral_rate,
            at_risk_count,
        };
    }

    /// Check segment distribution against targets.
    pub fn check_segment_distribution(&self) -> Vec<String> {
        let mut issues = Vec::new();
        let total = self.customers.len() as f64;

        if total == 0.0 {
            return issues;
        }

        for segment in [
            CustomerValueSegment::Enterprise,
            CustomerValueSegment::MidMarket,
            CustomerValueSegment::Smb,
            CustomerValueSegment::Consumer,
        ] {
            let expected = segment.customer_share();
            let actual = self
                .segment_index
                .get(&segment)
                .map(std::vec::Vec::len)
                .unwrap_or(0) as f64
                / total;

            // Allow 20% deviation
            if (actual - expected).abs() > expected * 0.2 {
                issues.push(format!(
                    "Segment {:?} distribution {:.1}% deviates from expected {:.1}%",
                    segment,
                    actual * 100.0,
                    expected * 100.0
                ));
            }
        }

        issues
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_value_segment() {
        // Verify shares sum to 1.0
        let total_customer_share: f64 = [
            CustomerValueSegment::Enterprise,
            CustomerValueSegment::MidMarket,
            CustomerValueSegment::Smb,
            CustomerValueSegment::Consumer,
        ]
        .iter()
        .map(|s| s.customer_share())
        .sum();

        assert!((total_customer_share - 1.0).abs() < 0.01);

        let total_revenue_share: f64 = [
            CustomerValueSegment::Enterprise,
            CustomerValueSegment::MidMarket,
            CustomerValueSegment::Smb,
            CustomerValueSegment::Consumer,
        ]
        .iter()
        .map(|s| s.revenue_share())
        .sum();

        assert!((total_revenue_share - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_lifecycle_stage() {
        let stage = CustomerLifecycleStage::Growth {
            since: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            growth_rate: 0.15,
        };

        assert!(stage.is_active());
        assert!(stage.is_good_standing());
        assert_eq!(stage.stage_name(), "growth");
    }

    #[test]
    fn test_at_risk_lifecycle() {
        let stage = CustomerLifecycleStage::AtRisk {
            triggers: vec![
                RiskTrigger::DecliningOrderFrequency,
                RiskTrigger::Complaints,
            ],
            flagged_date: NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            churn_probability: 0.6,
        };

        assert!(stage.is_active());
        assert!(!stage.is_good_standing());
        assert_eq!(stage.retention_priority(), 1);
    }

    #[test]
    fn test_customer_network_position() {
        let mut pos = CustomerNetworkPosition::new("C-001")
            .with_referral("C-000")
            .with_parent("C-PARENT");

        pos.add_referral("C-002");
        pos.add_referral("C-003");
        pos.add_child("C-SUB-001");

        assert!(pos.was_referred());
        assert!(!pos.is_root());
        assert_eq!(pos.network_influence(), 3);
    }

    #[test]
    fn test_customer_engagement() {
        let mut engagement = CustomerEngagement::default();
        engagement.record_order(
            Decimal::from(5000),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            3,
        );
        engagement.record_order(
            Decimal::from(7500),
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            5,
        );

        assert_eq!(engagement.total_orders, 2);
        assert_eq!(engagement.lifetime_revenue, Decimal::from(12500));
        assert_eq!(engagement.products_purchased, 8);
        assert!(engagement.average_order_value > Decimal::ZERO);
    }

    #[test]
    fn test_segmented_customer() {
        let customer = SegmentedCustomer::new(
            "C-001",
            "Acme Corp",
            CustomerValueSegment::Enterprise,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )
        .with_annual_contract_value(Decimal::from(500000))
        .with_industry("Technology");

        assert!(customer.is_high_value());
        assert_eq!(customer.segment.code(), "ENT");
        assert!(customer.estimated_lifetime_value() > Decimal::ZERO);
    }

    #[test]
    fn test_segment_change() {
        let mut customer = SegmentedCustomer::new(
            "C-001",
            "Growing Inc",
            CustomerValueSegment::Smb,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        );

        customer.change_segment(
            CustomerValueSegment::MidMarket,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert_eq!(customer.segment, CustomerValueSegment::MidMarket);
        assert_eq!(customer.previous_segment, Some(CustomerValueSegment::Smb));
        assert!(customer.segment_change_date.is_some());
    }

    #[test]
    fn test_segmented_customer_pool() {
        let mut pool = SegmentedCustomerPool::new();

        pool.add_customer(SegmentedCustomer::new(
            "C-001",
            "Enterprise Corp",
            CustomerValueSegment::Enterprise,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        ));

        pool.add_customer(SegmentedCustomer::new(
            "C-002",
            "SMB Inc",
            CustomerValueSegment::Smb,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        ));

        assert_eq!(pool.customers.len(), 2);
        assert_eq!(pool.by_segment(CustomerValueSegment::Enterprise).len(), 1);
        assert_eq!(pool.high_value_customers().len(), 1);
    }

    #[test]
    fn test_churn_risk_calculation() {
        let mut customer = SegmentedCustomer::new(
            "C-001",
            "At Risk Corp",
            CustomerValueSegment::MidMarket,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )
        .with_lifecycle_stage(CustomerLifecycleStage::AtRisk {
            triggers: vec![RiskTrigger::DecliningOrderFrequency],
            flagged_date: NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            churn_probability: 0.7,
        });

        customer.engagement.days_since_last_order = 90;
        customer.calculate_churn_risk();

        assert!(customer.churn_risk_score > 0.3);
    }
}
