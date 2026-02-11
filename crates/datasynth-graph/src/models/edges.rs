//! Edge models for graph representation.

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::nodes::NodeId;

/// Unique identifier for an edge.
pub type EdgeId = u64;

/// Type of edge in the graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    /// Transaction flow (debit/credit).
    Transaction,
    /// Approval relationship.
    Approval,
    /// Reports-to relationship.
    ReportsTo,
    /// Ownership relationship.
    Ownership,
    /// Intercompany relationship.
    Intercompany,
    /// Document reference.
    DocumentReference,
    /// Cost allocation.
    CostAllocation,
    /// Custom edge type.
    Custom(String),
}

impl EdgeType {
    /// Returns the type name as a string.
    pub fn as_str(&self) -> &str {
        match self {
            EdgeType::Transaction => "Transaction",
            EdgeType::Approval => "Approval",
            EdgeType::ReportsTo => "ReportsTo",
            EdgeType::Ownership => "Ownership",
            EdgeType::Intercompany => "Intercompany",
            EdgeType::DocumentReference => "DocumentReference",
            EdgeType::CostAllocation => "CostAllocation",
            EdgeType::Custom(s) => s,
        }
    }
}

/// Direction of an edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeDirection {
    /// Directed edge (source -> target).
    Directed,
    /// Undirected edge (bidirectional).
    Undirected,
}

/// An edge in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Unique edge ID.
    pub id: EdgeId,
    /// Source node ID.
    pub source: NodeId,
    /// Target node ID.
    pub target: NodeId,
    /// Edge type.
    pub edge_type: EdgeType,
    /// Edge direction.
    pub direction: EdgeDirection,
    /// Edge weight (e.g., transaction amount).
    pub weight: f64,
    /// Edge features for ML.
    pub features: Vec<f64>,
    /// Edge properties.
    pub properties: HashMap<String, EdgeProperty>,
    /// Labels for supervised learning.
    pub labels: Vec<String>,
    /// Is this edge anomalous?
    pub is_anomaly: bool,
    /// Anomaly type if anomalous.
    pub anomaly_type: Option<String>,
    /// Timestamp (for temporal graphs).
    pub timestamp: Option<NaiveDate>,
}

impl GraphEdge {
    /// Creates a new graph edge.
    pub fn new(id: EdgeId, source: NodeId, target: NodeId, edge_type: EdgeType) -> Self {
        Self {
            id,
            source,
            target,
            edge_type,
            direction: EdgeDirection::Directed,
            weight: 1.0,
            features: Vec::new(),
            properties: HashMap::new(),
            labels: Vec::new(),
            is_anomaly: false,
            anomaly_type: None,
            timestamp: None,
        }
    }

    /// Sets the edge weight.
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    /// Adds a numeric feature.
    pub fn with_feature(mut self, value: f64) -> Self {
        self.features.push(value);
        self
    }

    /// Adds multiple numeric features.
    pub fn with_features(mut self, values: Vec<f64>) -> Self {
        self.features.extend(values);
        self
    }

    /// Adds a property.
    pub fn with_property(mut self, name: &str, value: EdgeProperty) -> Self {
        self.properties.insert(name.to_string(), value);
        self
    }

    /// Sets the timestamp.
    pub fn with_timestamp(mut self, timestamp: NaiveDate) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Makes the edge undirected.
    pub fn undirected(mut self) -> Self {
        self.direction = EdgeDirection::Undirected;
        self
    }

    /// Marks the edge as anomalous.
    pub fn as_anomaly(mut self, anomaly_type: &str) -> Self {
        self.is_anomaly = true;
        self.anomaly_type = Some(anomaly_type.to_string());
        self
    }

    /// Adds a label.
    pub fn with_label(mut self, label: &str) -> Self {
        self.labels.push(label.to_string());
        self
    }

    /// Returns the feature vector dimension.
    pub fn feature_dim(&self) -> usize {
        self.features.len()
    }
}

/// Property value for an edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeProperty {
    /// String value.
    String(String),
    /// Integer value.
    Int(i64),
    /// Float value.
    Float(f64),
    /// Decimal value.
    Decimal(Decimal),
    /// Boolean value.
    Bool(bool),
    /// Date value.
    Date(NaiveDate),
}

impl EdgeProperty {
    /// Converts to string representation.
    pub fn to_string_value(&self) -> String {
        match self {
            EdgeProperty::String(s) => s.clone(),
            EdgeProperty::Int(i) => i.to_string(),
            EdgeProperty::Float(f) => f.to_string(),
            EdgeProperty::Decimal(d) => d.to_string(),
            EdgeProperty::Bool(b) => b.to_string(),
            EdgeProperty::Date(d) => d.to_string(),
        }
    }

    /// Converts to numeric value.
    pub fn to_numeric(&self) -> Option<f64> {
        match self {
            EdgeProperty::Int(i) => Some(*i as f64),
            EdgeProperty::Float(f) => Some(*f),
            EdgeProperty::Decimal(d) => (*d).try_into().ok(),
            EdgeProperty::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }
}

/// Transaction edge with accounting-specific features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionEdge {
    /// Base edge.
    pub edge: GraphEdge,
    /// Document number.
    pub document_number: String,
    /// Company code.
    pub company_code: String,
    /// Posting date.
    pub posting_date: NaiveDate,
    /// Debit amount.
    pub debit_amount: Decimal,
    /// Credit amount.
    pub credit_amount: Decimal,
    /// Is this a debit (true) or credit (false)?
    pub is_debit: bool,
    /// Cost center.
    pub cost_center: Option<String>,
    /// Business process.
    pub business_process: Option<String>,
}

impl TransactionEdge {
    /// Creates a new transaction edge.
    pub fn new(
        id: EdgeId,
        source: NodeId,
        target: NodeId,
        document_number: String,
        posting_date: NaiveDate,
        amount: Decimal,
        is_debit: bool,
    ) -> Self {
        let amount_f64: f64 = amount.try_into().unwrap_or(0.0);
        let mut edge = GraphEdge::new(id, source, target, EdgeType::Transaction)
            .with_weight(amount_f64.abs())
            .with_timestamp(posting_date);

        edge.properties.insert(
            "document_number".to_string(),
            EdgeProperty::String(document_number.clone()),
        );
        edge.properties
            .insert("posting_date".to_string(), EdgeProperty::Date(posting_date));
        edge.properties
            .insert("is_debit".to_string(), EdgeProperty::Bool(is_debit));

        Self {
            edge,
            document_number,
            company_code: String::new(),
            posting_date,
            debit_amount: if is_debit { amount } else { Decimal::ZERO },
            credit_amount: if !is_debit { amount } else { Decimal::ZERO },
            is_debit,
            cost_center: None,
            business_process: None,
        }
    }

    /// Computes features for the transaction edge.
    pub fn compute_features(&mut self) {
        // Amount features
        let amount: f64 = if self.is_debit {
            self.debit_amount.try_into().unwrap_or(0.0)
        } else {
            self.credit_amount.try_into().unwrap_or(0.0)
        };

        // Log amount
        self.edge.features.push((amount.abs() + 1.0).ln());

        // Debit/Credit indicator
        self.edge
            .features
            .push(if self.is_debit { 1.0 } else { 0.0 });

        // Temporal features
        let weekday = self.posting_date.weekday().num_days_from_monday() as f64;
        self.edge.features.push(weekday / 6.0); // Normalized to [0, 1]

        let day = self.posting_date.day() as f64;
        self.edge.features.push(day / 31.0); // Normalized

        let month = self.posting_date.month() as f64;
        self.edge.features.push(month / 12.0); // Normalized

        // Is month end (last 3 days)
        let is_month_end = day >= 28.0;
        self.edge
            .features
            .push(if is_month_end { 1.0 } else { 0.0 });

        // Is year end (December)
        let is_year_end = month == 12.0;
        self.edge.features.push(if is_year_end { 1.0 } else { 0.0 });

        // Benford's law probability (first digit)
        let first_digit = Self::extract_first_digit(amount);
        let benford_prob = Self::benford_probability(first_digit);
        self.edge.features.push(benford_prob);
    }

    /// Extracts the first significant digit of a number.
    fn extract_first_digit(value: f64) -> u32 {
        if value == 0.0 {
            return 0;
        }
        let abs_val = value.abs();
        let log10 = abs_val.log10().floor();
        let normalized = abs_val / 10_f64.powf(log10);
        normalized.floor() as u32
    }

    /// Returns the expected Benford's law probability for a digit.
    fn benford_probability(digit: u32) -> f64 {
        if digit == 0 || digit > 9 {
            return 0.0;
        }
        (1.0 + 1.0 / digit as f64).log10()
    }
}

/// Approval edge for approval networks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalEdge {
    /// Base edge.
    pub edge: GraphEdge,
    /// Document that was approved.
    pub document_number: String,
    /// Approval date.
    pub approval_date: NaiveDate,
    /// Approval amount.
    pub amount: Decimal,
    /// Approval action (Approve, Reject, Forward).
    pub action: String,
    /// Is this within the user's approval limit?
    pub within_limit: bool,
}

impl ApprovalEdge {
    /// Creates a new approval edge.
    pub fn new(
        id: EdgeId,
        approver_node: NodeId,
        requester_node: NodeId,
        document_number: String,
        approval_date: NaiveDate,
        amount: Decimal,
        action: &str,
    ) -> Self {
        let amount_f64: f64 = amount.try_into().unwrap_or(0.0);
        let edge = GraphEdge::new(id, approver_node, requester_node, EdgeType::Approval)
            .with_weight(amount_f64)
            .with_timestamp(approval_date)
            .with_property("action", EdgeProperty::String(action.to_string()));

        Self {
            edge,
            document_number,
            approval_date,
            amount,
            action: action.to_string(),
            within_limit: true,
        }
    }

    /// Computes features for the approval edge.
    pub fn compute_features(&mut self) {
        // Amount (log-scaled)
        let amount_f64: f64 = self.amount.try_into().unwrap_or(0.0);
        self.edge.features.push((amount_f64.abs() + 1.0).ln());

        // Action encoding
        let action_code = match self.action.as_str() {
            "Approve" => 1.0,
            "Reject" => 0.0,
            "Forward" => 0.5,
            _ => 0.5,
        };
        self.edge.features.push(action_code);

        // Within limit
        self.edge
            .features
            .push(if self.within_limit { 1.0 } else { 0.0 });
    }
}

/// Ownership edge for entity relationship graphs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipEdge {
    /// Base edge.
    pub edge: GraphEdge,
    /// Parent company code.
    pub parent_code: String,
    /// Subsidiary company code.
    pub subsidiary_code: String,
    /// Ownership percentage.
    pub ownership_percent: Decimal,
    /// Consolidation method.
    pub consolidation_method: String,
    /// Effective date.
    pub effective_date: NaiveDate,
}

impl OwnershipEdge {
    /// Creates a new ownership edge.
    pub fn new(
        id: EdgeId,
        parent_node: NodeId,
        subsidiary_node: NodeId,
        ownership_percent: Decimal,
        effective_date: NaiveDate,
    ) -> Self {
        let pct_f64: f64 = ownership_percent.try_into().unwrap_or(0.0);
        let edge = GraphEdge::new(id, parent_node, subsidiary_node, EdgeType::Ownership)
            .with_weight(pct_f64)
            .with_timestamp(effective_date);

        Self {
            edge,
            parent_code: String::new(),
            subsidiary_code: String::new(),
            ownership_percent,
            consolidation_method: "Full".to_string(),
            effective_date,
        }
    }

    /// Computes features for the ownership edge.
    pub fn compute_features(&mut self) {
        // Ownership percentage (normalized)
        let pct: f64 = self.ownership_percent.try_into().unwrap_or(0.0);
        self.edge.features.push(pct / 100.0);

        // Consolidation method encoding
        let method_code = match self.consolidation_method.as_str() {
            "Full" => 1.0,
            "Proportional" => 0.5,
            "Equity" => 0.25,
            _ => 0.0,
        };
        self.edge.features.push(method_code);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_edge_creation() {
        let edge = GraphEdge::new(1, 10, 20, EdgeType::Transaction)
            .with_weight(1000.0)
            .with_feature(0.5);

        assert_eq!(edge.id, 1);
        assert_eq!(edge.source, 10);
        assert_eq!(edge.target, 20);
        assert_eq!(edge.weight, 1000.0);
    }

    #[test]
    fn test_transaction_edge() {
        let mut tx = TransactionEdge::new(
            1,
            10,
            20,
            "DOC001".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            Decimal::new(10000, 2),
            true,
        );
        tx.compute_features();

        assert!(!tx.edge.features.is_empty());
    }

    #[test]
    fn test_benford_probability() {
        // Digit 1 should have ~30.1% probability
        let prob1 = TransactionEdge::benford_probability(1);
        assert!((prob1 - 0.301).abs() < 0.001);

        // Digit 9 should have ~4.6% probability
        let prob9 = TransactionEdge::benford_probability(9);
        assert!((prob9 - 0.046).abs() < 0.001);
    }
}
