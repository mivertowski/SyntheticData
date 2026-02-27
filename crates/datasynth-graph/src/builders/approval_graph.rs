//! Approval graph builder.
//!
//! Builds a graph where:
//! - Nodes are users/employees
//! - Edges are approval relationships

use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

use datasynth_core::models::{ApprovalRecord, User};

use crate::models::{ApprovalEdge, EdgeType, Graph, GraphEdge, GraphType, NodeId, UserNode};

/// Configuration for approval graph building.
#[derive(Debug, Clone)]
pub struct ApprovalGraphConfig {
    /// Whether to include reports-to edges.
    pub include_hierarchy: bool,
    /// Whether to track potential SoD violations.
    pub track_sod_violations: bool,
    /// Minimum number of approvals to include edge.
    pub min_approval_count: usize,
    /// Whether to aggregate multiple approvals.
    pub aggregate_approvals: bool,
}

impl Default for ApprovalGraphConfig {
    fn default() -> Self {
        Self {
            include_hierarchy: true,
            track_sod_violations: true,
            min_approval_count: 1,
            aggregate_approvals: false,
        }
    }
}

/// Builder for approval graphs.
pub struct ApprovalGraphBuilder {
    config: ApprovalGraphConfig,
    graph: Graph,
    /// Map from user ID to node ID.
    user_nodes: HashMap<String, NodeId>,
    /// Approval aggregation.
    approval_aggregation: HashMap<(NodeId, NodeId), ApprovalAggregation>,
}

impl ApprovalGraphBuilder {
    /// Creates a new approval graph builder.
    pub fn new(config: ApprovalGraphConfig) -> Self {
        Self {
            config,
            graph: Graph::new("approval_network", GraphType::Approval),
            user_nodes: HashMap::new(),
            approval_aggregation: HashMap::new(),
        }
    }

    /// Adds users to the graph.
    pub fn add_users(&mut self, users: &[User]) {
        for user in users {
            self.get_or_create_user_node(user);
        }

        if self.config.include_hierarchy {
            tracing::warn!(
                "include_hierarchy requires manager_id field on User model — not yet supported"
            );
        }
    }

    /// Adds an approval record to the graph.
    pub fn add_approval(&mut self, approval: &ApprovalRecord) {
        let approver_id = self.ensure_user_node(&approval.approver_id, &approval.approver_name);
        let requester_id = self.ensure_user_node(
            &approval.requester_id,
            approval.requester_name.as_deref().unwrap_or("Unknown"),
        );

        if self.config.aggregate_approvals {
            self.aggregate_approval(approver_id, requester_id, approval);
        } else {
            let mut edge = ApprovalEdge::new(
                0,
                approver_id,
                requester_id,
                approval.document_number.clone(),
                approval.approval_date,
                approval.amount,
                &approval.action,
            );

            // Check if within limit
            if let Some(limit) = approval.approval_limit {
                edge.within_limit = approval.amount <= limit;
                if !edge.within_limit && self.config.track_sod_violations {
                    edge.edge = edge.edge.as_anomaly("ApprovalLimitExceeded");
                }
            }

            edge.compute_features();
            self.graph.add_edge(edge.edge);
        }
    }

    /// Adds multiple approval records.
    pub fn add_approvals(&mut self, approvals: &[ApprovalRecord]) {
        for approval in approvals {
            self.add_approval(approval);
        }
    }

    /// Marks a self-approval (user approves own request).
    pub fn mark_self_approval(&mut self, user_id: &str, _document_number: &str, date: NaiveDate) {
        if let Some(&node_id) = self.user_nodes.get(user_id) {
            // Self-loop edge
            let edge = GraphEdge::new(0, node_id, node_id, EdgeType::Approval)
                .with_timestamp(date)
                .as_anomaly("SelfApproval");
            self.graph.add_edge(edge);
        }
    }

    /// Gets or creates a user node from a User struct.
    fn get_or_create_user_node(&mut self, user: &User) -> NodeId {
        if let Some(&id) = self.user_nodes.get(&user.user_id) {
            return id;
        }

        let mut user_node = UserNode::new(0, user.user_id.clone(), user.display_name.clone());
        user_node.department = user.department.clone();
        user_node.is_active = user.is_active;
        user_node.compute_features();

        let id = self.graph.add_node(user_node.node);
        self.user_nodes.insert(user.user_id.clone(), id);
        id
    }

    /// Ensures a user node exists.
    fn ensure_user_node(&mut self, user_id: &str, user_name: &str) -> NodeId {
        if let Some(&id) = self.user_nodes.get(user_id) {
            return id;
        }

        let mut user_node = UserNode::new(0, user_id.to_string(), user_name.to_string());
        user_node.compute_features();

        let id = self.graph.add_node(user_node.node);
        self.user_nodes.insert(user_id.to_string(), id);
        id
    }

    /// Aggregates approval data.
    fn aggregate_approval(
        &mut self,
        approver: NodeId,
        requester: NodeId,
        approval: &ApprovalRecord,
    ) {
        let key = (approver, requester);
        let amount: f64 = approval.amount.try_into().unwrap_or(0.0);

        let agg = self
            .approval_aggregation
            .entry(key)
            .or_insert(ApprovalAggregation {
                approver,
                requester,
                total_amount: 0.0,
                count: 0,
                approve_count: 0,
                reject_count: 0,
                first_date: approval.approval_date,
                last_date: approval.approval_date,
            });

        agg.total_amount += amount;
        agg.count += 1;

        match approval.action.as_str() {
            "Approve" | "Approved" => agg.approve_count += 1,
            "Reject" | "Rejected" => agg.reject_count += 1,
            _ => {}
        }

        if approval.approval_date < agg.first_date {
            agg.first_date = approval.approval_date;
        }
        if approval.approval_date > agg.last_date {
            agg.last_date = approval.approval_date;
        }
    }

    /// Builds the final graph.
    pub fn build(mut self) -> Graph {
        // Create aggregated edges
        if self.config.aggregate_approvals {
            for ((approver, requester), agg) in self.approval_aggregation {
                if agg.count < self.config.min_approval_count {
                    continue;
                }

                let mut edge = GraphEdge::new(0, approver, requester, EdgeType::Approval)
                    .with_weight(agg.total_amount)
                    .with_timestamp(agg.last_date);

                // Features
                edge.features.push((agg.total_amount + 1.0).ln());
                edge.features.push(agg.count as f64);
                edge.features
                    .push(agg.approve_count as f64 / agg.count as f64);
                edge.features
                    .push((agg.last_date - agg.first_date).num_days() as f64);

                self.graph.add_edge(edge);
            }
        }

        self.graph.compute_statistics();
        self.graph
    }
}

/// Aggregated approval data.
#[allow(dead_code)]
struct ApprovalAggregation {
    approver: NodeId,
    requester: NodeId,
    total_amount: f64,
    count: usize,
    approve_count: usize,
    reject_count: usize,
    first_date: NaiveDate,
    last_date: NaiveDate,
}

/// Simplified approval record for building graphs without full ApprovalRecord.
#[derive(Debug, Clone)]
pub struct SimpleApproval {
    pub approver_id: String,
    pub approver_name: String,
    pub requester_id: String,
    pub requester_name: String,
    pub document_number: String,
    pub approval_date: NaiveDate,
    pub amount: Decimal,
    pub action: String,
}

impl SimpleApproval {
    /// Converts to ApprovalRecord.
    pub fn to_approval_record(&self) -> ApprovalRecord {
        ApprovalRecord {
            approval_id: format!("APR-{}", self.document_number),
            document_number: self.document_number.clone(),
            document_type: "JE".to_string(),
            company_code: "1000".to_string(),
            requester_id: self.requester_id.clone(),
            requester_name: Some(self.requester_name.clone()),
            approver_id: self.approver_id.clone(),
            approver_name: self.approver_name.clone(),
            approval_date: self.approval_date,
            action: self.action.clone(),
            amount: self.amount,
            approval_limit: None,
            comments: None,
            delegation_from: None,
            is_auto_approved: false,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_approval_graph() {
        let mut builder = ApprovalGraphBuilder::new(ApprovalGraphConfig::default());

        let approval = ApprovalRecord {
            approval_id: "APR001".to_string(),
            document_number: "JE001".to_string(),
            document_type: "JE".to_string(),
            company_code: "1000".to_string(),
            requester_id: "USER001".to_string(),
            requester_name: Some("John Doe".to_string()),
            approver_id: "USER002".to_string(),
            approver_name: "Jane Smith".to_string(),
            approval_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            action: "Approve".to_string(),
            amount: dec!(10000),
            approval_limit: Some(dec!(50000)),
            comments: None,
            delegation_from: None,
            is_auto_approved: false,
        };

        builder.add_approval(&approval);

        let graph = builder.build();

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }
}
