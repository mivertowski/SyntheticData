//! Node models for graph representation.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a node.
pub type NodeId = u64;

/// Type of node in the graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    /// GL Account node.
    Account,
    /// Journal Entry document node.
    JournalEntry,
    /// Vendor node.
    Vendor,
    /// Customer node.
    Customer,
    /// User/Employee node.
    User,
    /// Company/Legal Entity node.
    Company,
    /// Cost Center node.
    CostCenter,
    /// Profit Center node.
    ProfitCenter,
    /// Material node.
    Material,
    /// Fixed Asset node.
    FixedAsset,
    /// Custom node type.
    Custom(String),
}

impl NodeType {
    /// Returns the type name as a string.
    pub fn as_str(&self) -> &str {
        match self {
            NodeType::Account => "Account",
            NodeType::JournalEntry => "JournalEntry",
            NodeType::Vendor => "Vendor",
            NodeType::Customer => "Customer",
            NodeType::User => "User",
            NodeType::Company => "Company",
            NodeType::CostCenter => "CostCenter",
            NodeType::ProfitCenter => "ProfitCenter",
            NodeType::Material => "Material",
            NodeType::FixedAsset => "FixedAsset",
            NodeType::Custom(s) => s,
        }
    }
}

/// A node in the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique node ID.
    pub id: NodeId,
    /// Node type.
    pub node_type: NodeType,
    /// External ID (e.g., account code, vendor ID).
    pub external_id: String,
    /// Node label for display.
    pub label: String,
    /// Numeric features for ML.
    pub features: Vec<f64>,
    /// Categorical features (will be one-hot encoded).
    pub categorical_features: HashMap<String, String>,
    /// Node properties.
    pub properties: HashMap<String, NodeProperty>,
    /// Labels for supervised learning.
    pub labels: Vec<String>,
    /// Is this node an anomaly?
    pub is_anomaly: bool,
    /// Anomaly type if anomalous.
    pub anomaly_type: Option<String>,
}

impl GraphNode {
    /// Creates a new graph node.
    pub fn new(id: NodeId, node_type: NodeType, external_id: String, label: String) -> Self {
        Self {
            id,
            node_type,
            external_id,
            label,
            features: Vec::new(),
            categorical_features: HashMap::new(),
            properties: HashMap::new(),
            labels: Vec::new(),
            is_anomaly: false,
            anomaly_type: None,
        }
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

    /// Adds a categorical feature.
    pub fn with_categorical(mut self, name: &str, value: &str) -> Self {
        self.categorical_features
            .insert(name.to_string(), value.to_string());
        self
    }

    /// Adds a property.
    pub fn with_property(mut self, name: &str, value: NodeProperty) -> Self {
        self.properties.insert(name.to_string(), value);
        self
    }

    /// Marks the node as anomalous.
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

/// Property value for a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeProperty {
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
    /// List of strings.
    StringList(Vec<String>),
}

impl NodeProperty {
    /// Converts to string representation.
    pub fn to_string_value(&self) -> String {
        match self {
            NodeProperty::String(s) => s.clone(),
            NodeProperty::Int(i) => i.to_string(),
            NodeProperty::Float(f) => f.to_string(),
            NodeProperty::Decimal(d) => d.to_string(),
            NodeProperty::Bool(b) => b.to_string(),
            NodeProperty::Date(d) => d.to_string(),
            NodeProperty::StringList(v) => v.join(","),
        }
    }

    /// Converts to numeric value (for features).
    pub fn to_numeric(&self) -> Option<f64> {
        match self {
            NodeProperty::Int(i) => Some(*i as f64),
            NodeProperty::Float(f) => Some(*f),
            NodeProperty::Decimal(d) => (*d).try_into().ok(),
            NodeProperty::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }
}

/// Account node with accounting-specific features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountNode {
    /// Base node.
    pub node: GraphNode,
    /// Account code.
    pub account_code: String,
    /// Account name.
    pub account_name: String,
    /// Account type (Asset, Liability, etc.).
    pub account_type: String,
    /// Account category.
    pub account_category: Option<String>,
    /// Is balance sheet account.
    pub is_balance_sheet: bool,
    /// Normal balance (Debit/Credit).
    pub normal_balance: String,
    /// Company code.
    pub company_code: String,
    /// Country code (ISO 3166-1 alpha-2) of the owning company.
    pub country: Option<String>,
}

impl AccountNode {
    /// Creates a new account node.
    pub fn new(
        id: NodeId,
        account_code: String,
        account_name: String,
        account_type: String,
        company_code: String,
    ) -> Self {
        let node = GraphNode::new(
            id,
            NodeType::Account,
            account_code.clone(),
            format!("{} - {}", account_code, account_name),
        );

        Self {
            node,
            account_code,
            account_name,
            account_type,
            account_category: None,
            is_balance_sheet: false,
            normal_balance: "Debit".to_string(),
            company_code,
            country: None,
        }
    }

    /// Computes features for the account node.
    pub fn compute_features(&mut self) {
        // Account type encoding
        let type_feature = match self.account_type.as_str() {
            "Asset" => 0.0,
            "Liability" => 1.0,
            "Equity" => 2.0,
            "Revenue" => 3.0,
            "Expense" => 4.0,
            _ => 5.0,
        };
        self.node.features.push(type_feature);

        // Balance sheet indicator
        self.node
            .features
            .push(if self.is_balance_sheet { 1.0 } else { 0.0 });

        // Normal balance encoding
        self.node.features.push(if self.normal_balance == "Debit" {
            1.0
        } else {
            0.0
        });

        // Account code as normalized numeric feature [0, 1]
        // Parse up to 4 leading digits and divide by 10000.
        let code_prefix: String = self
            .account_code
            .chars()
            .take(4)
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(code_num) = code_prefix.parse::<f64>() {
            self.node.features.push(code_num / 10000.0);
        } else {
            self.node.features.push(0.0);
        }

        // Add categorical features
        self.node
            .categorical_features
            .insert("account_type".to_string(), self.account_type.clone());
        self.node
            .categorical_features
            .insert("company_code".to_string(), self.company_code.clone());
        if let Some(ref country) = self.country {
            self.node
                .categorical_features
                .insert("country".to_string(), country.clone());
        }
    }
}

/// User node for approval networks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserNode {
    /// Base node.
    pub node: GraphNode,
    /// User ID.
    pub user_id: String,
    /// User name.
    pub user_name: String,
    /// Department.
    pub department: Option<String>,
    /// Role.
    pub role: Option<String>,
    /// Manager ID.
    pub manager_id: Option<String>,
    /// Approval limit.
    pub approval_limit: Option<Decimal>,
    /// Is active.
    pub is_active: bool,
}

impl UserNode {
    /// Creates a new user node.
    pub fn new(id: NodeId, user_id: String, user_name: String) -> Self {
        let node = GraphNode::new(id, NodeType::User, user_id.clone(), user_name.clone());

        Self {
            node,
            user_id,
            user_name,
            department: None,
            role: None,
            manager_id: None,
            approval_limit: None,
            is_active: true,
        }
    }

    /// Computes features for the user node.
    pub fn compute_features(&mut self) {
        // Active status
        self.node
            .features
            .push(if self.is_active { 1.0 } else { 0.0 });

        // Approval limit (log-scaled)
        if let Some(limit) = self.approval_limit {
            let limit_f64: f64 = limit.try_into().unwrap_or(0.0);
            self.node.features.push((limit_f64 + 1.0).ln());
        } else {
            self.node.features.push(0.0);
        }

        // Add categorical features
        if let Some(ref dept) = self.department {
            self.node
                .categorical_features
                .insert("department".to_string(), dept.clone());
        }
        if let Some(ref role) = self.role {
            self.node
                .categorical_features
                .insert("role".to_string(), role.clone());
        }
    }
}

/// Company/Entity node for entity relationship graphs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyNode {
    /// Base node.
    pub node: GraphNode,
    /// Company code.
    pub company_code: String,
    /// Company name.
    pub company_name: String,
    /// Country.
    pub country: String,
    /// Currency.
    pub currency: String,
    /// Is parent company.
    pub is_parent: bool,
    /// Parent company code.
    pub parent_code: Option<String>,
    /// Ownership percentage (if subsidiary).
    pub ownership_percent: Option<Decimal>,
}

impl CompanyNode {
    /// Creates a new company node.
    pub fn new(id: NodeId, company_code: String, company_name: String) -> Self {
        let node = GraphNode::new(
            id,
            NodeType::Company,
            company_code.clone(),
            company_name.clone(),
        );

        Self {
            node,
            company_code,
            company_name,
            country: "US".to_string(),
            currency: "USD".to_string(),
            is_parent: false,
            parent_code: None,
            ownership_percent: None,
        }
    }

    /// Computes features for the company node.
    pub fn compute_features(&mut self) {
        // Is parent
        self.node
            .features
            .push(if self.is_parent { 1.0 } else { 0.0 });

        // Ownership percentage
        if let Some(pct) = self.ownership_percent {
            let pct_f64: f64 = pct.try_into().unwrap_or(0.0);
            self.node.features.push(pct_f64 / 100.0);
        } else {
            self.node.features.push(1.0); // 100% for parent
        }

        // Add categorical features
        self.node
            .categorical_features
            .insert("country".to_string(), self.country.clone());
        self.node
            .categorical_features
            .insert("currency".to_string(), self.currency.clone());
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_node_creation() {
        let node = GraphNode::new(1, NodeType::Account, "1000".to_string(), "Cash".to_string())
            .with_feature(100.0)
            .with_categorical("type", "Asset");

        assert_eq!(node.id, 1);
        assert_eq!(node.features.len(), 1);
        assert!(node.categorical_features.contains_key("type"));
    }

    #[test]
    fn test_account_node() {
        let mut account = AccountNode::new(
            1,
            "1000".to_string(),
            "Cash".to_string(),
            "Asset".to_string(),
            "1000".to_string(),
        );
        account.is_balance_sheet = true;
        account.compute_features();

        assert!(!account.node.features.is_empty());
    }
}
