//! Banking network graph builder.
//!
//! Builds graphs for AML/KYC analysis where:
//! - Nodes are customers, accounts, and counterparties
//! - Edges are transactions and ownership relationships
//! - Ground truth labels identify suspicious activity

use std::collections::HashMap;

use chrono::{Datelike, Timelike};

use datasynth_banking::models::{BankAccount, BankTransaction, BankingCustomer, CounterpartyPool};
use datasynth_core::models::banking::{AmlTypology, Direction, RiskTier};

use crate::models::{EdgeType, Graph, GraphEdge, GraphNode, GraphType, NodeId, NodeType};

/// Configuration for banking graph building.
#[derive(Debug, Clone)]
pub struct BankingGraphConfig {
    /// Include customer nodes.
    pub include_customers: bool,
    /// Include account nodes.
    pub include_accounts: bool,
    /// Include counterparty nodes.
    pub include_counterparties: bool,
    /// Include beneficial ownership edges.
    pub include_beneficial_ownership: bool,
    /// Create transaction edges.
    pub create_transaction_edges: bool,
    /// Minimum transaction amount to include as edge.
    pub min_transaction_amount: f64,
    /// Aggregate parallel edges between same nodes.
    pub aggregate_parallel_edges: bool,
    /// Include temporal features.
    pub include_temporal_features: bool,
    /// Include risk features.
    pub include_risk_features: bool,
}

impl Default for BankingGraphConfig {
    fn default() -> Self {
        Self {
            include_customers: true,
            include_accounts: true,
            include_counterparties: true,
            include_beneficial_ownership: true,
            create_transaction_edges: true,
            min_transaction_amount: 0.0,
            aggregate_parallel_edges: false,
            include_temporal_features: true,
            include_risk_features: true,
        }
    }
}

/// Builder for banking network graphs.
pub struct BankingGraphBuilder {
    config: BankingGraphConfig,
    graph: Graph,
    /// Map from customer ID to node ID.
    customer_nodes: HashMap<String, NodeId>,
    /// Map from account ID to node ID.
    account_nodes: HashMap<String, NodeId>,
    /// Map from counterparty name to node ID.
    counterparty_nodes: HashMap<String, NodeId>,
    /// For edge aggregation: (source, target) -> aggregated data.
    edge_aggregation: HashMap<(NodeId, NodeId), AggregatedBankingEdge>,
}

impl BankingGraphBuilder {
    /// Creates a new banking graph builder.
    pub fn new(config: BankingGraphConfig) -> Self {
        Self {
            config,
            graph: Graph::new("banking_network", GraphType::Custom("banking".to_string())),
            customer_nodes: HashMap::new(),
            account_nodes: HashMap::new(),
            counterparty_nodes: HashMap::new(),
            edge_aggregation: HashMap::new(),
        }
    }

    /// Adds customers to the graph.
    pub fn add_customers(&mut self, customers: &[BankingCustomer]) {
        if !self.config.include_customers {
            return;
        }

        for customer in customers {
            self.add_customer(customer);
        }
    }

    /// Adds a single customer to the graph.
    fn add_customer(&mut self, customer: &BankingCustomer) -> NodeId {
        let key = customer.customer_id.to_string();

        if let Some(&id) = self.customer_nodes.get(&key) {
            return id;
        }

        let mut node = GraphNode::new(
            0,
            NodeType::Customer,
            key.clone(),
            customer.name.display_name().to_string(),
        );

        // Add categorical features
        node.categorical_features.insert(
            "customer_type".to_string(),
            format!("{:?}", customer.customer_type),
        );
        node.categorical_features.insert(
            "residence_country".to_string(),
            customer.residence_country.clone(),
        );
        node.categorical_features
            .insert("risk_tier".to_string(), format!("{:?}", customer.risk_tier));

        // Add risk features
        if self.config.include_risk_features {
            // Risk tier encoding (0=Low to 1=Highest)
            let risk_score = match customer.risk_tier {
                RiskTier::Low => 0.0,
                RiskTier::Medium => 0.33,
                RiskTier::High => 0.67,
                RiskTier::VeryHigh | RiskTier::Prohibited => 1.0,
            };
            node.features.push(risk_score);

            // PEP indicator
            node.features.push(if customer.is_pep { 1.0 } else { 0.0 });

            // Number of accounts
            node.features.push(customer.account_ids.len() as f64);

            // KYC profile features
            let kyc = &customer.kyc_profile;

            // Expected monthly turnover encoding
            let turnover_band = match format!("{:?}", kyc.expected_monthly_turnover).as_str() {
                "VeryLow" => 1.0,
                "Low" => 2.0,
                "Medium" => 3.0,
                "High" => 4.0,
                "VeryHigh" => 5.0,
                _ => 3.0,
            };
            node.features.push(turnover_band);

            // Cash intensity
            let cash_intensity: f64 = match format!("{:?}", kyc.cash_intensity).as_str() {
                "VeryLow" => 0.0,
                "Low" => 0.25,
                "Moderate" => 0.5,
                "High" => 0.75,
                "VeryHigh" => 1.0,
                _ => 0.5,
            };
            node.features.push(cash_intensity);
        }

        // Mark anomaly if customer is suspicious
        if customer.is_mule {
            node = node.as_anomaly("money_mule");
            node.labels.push("mule".to_string());
        }

        let id = self.graph.add_node(node);
        self.customer_nodes.insert(key, id);
        id
    }

    /// Adds accounts to the graph.
    pub fn add_accounts(&mut self, accounts: &[BankAccount], customers: &[BankingCustomer]) {
        if !self.config.include_accounts {
            return;
        }

        // Build customer lookup
        let customer_map: HashMap<_, _> = customers.iter().map(|c| (c.customer_id, c)).collect();

        for account in accounts {
            let account_id = self.add_account(account);

            // Create ownership edge from customer to account
            if let Some(customer) = customer_map.get(&account.primary_owner_id) {
                let customer_id = self.add_customer(customer);

                let edge = GraphEdge::new(0, customer_id, account_id, EdgeType::Ownership)
                    .with_weight(1.0)
                    .with_property(
                        "relationship",
                        crate::models::EdgeProperty::String("account_owner".to_string()),
                    );

                self.graph.add_edge(edge);
            }
        }
    }

    /// Adds a single account to the graph.
    fn add_account(&mut self, account: &BankAccount) -> NodeId {
        let key = account.account_id.to_string();

        if let Some(&id) = self.account_nodes.get(&key) {
            return id;
        }

        let mut node = GraphNode::new(
            0,
            NodeType::Account,
            key.clone(),
            format!("{:?} - {}", account.account_type, account.account_number),
        );

        // Add categorical features
        node.categorical_features.insert(
            "account_type".to_string(),
            format!("{:?}", account.account_type),
        );
        node.categorical_features
            .insert("currency".to_string(), account.currency.clone());
        node.categorical_features
            .insert("status".to_string(), format!("{:?}", account.status));

        // Add numeric features
        if self.config.include_risk_features {
            // Balance (log-scaled)
            let balance: f64 = account.current_balance.try_into().unwrap_or(0.0);
            node.features.push((balance.abs() + 1.0).ln());

            // Overdraft limit (log-scaled)
            let limit: f64 = account.overdraft_limit.try_into().unwrap_or(0.0);
            node.features.push((limit + 1.0).ln());

            // Has debit card
            node.features.push(if account.features.debit_card {
                1.0
            } else {
                0.0
            });

            // Has international capability
            node.features
                .push(if account.features.international_transfers {
                    1.0
                } else {
                    0.0
                });
        }

        let id = self.graph.add_node(node);
        self.account_nodes.insert(key, id);
        id
    }

    /// Adds counterparties to the graph.
    pub fn add_counterparties(&mut self, pool: &CounterpartyPool) {
        if !self.config.include_counterparties {
            return;
        }

        for merchant in &pool.merchants {
            self.add_counterparty_node(
                &merchant.name,
                "merchant",
                Some(&format!("{:?}", merchant.mcc)),
            );
        }

        for employer in &pool.employers {
            let industry = employer.industry_code.as_deref().unwrap_or("Unknown");
            self.add_counterparty_node(&employer.name, "employer", Some(industry));
        }

        for utility in &pool.utilities {
            self.add_counterparty_node(
                &utility.name,
                "utility",
                Some(&format!("{:?}", utility.utility_type)),
            );
        }
    }

    /// Adds a counterparty node.
    fn add_counterparty_node(
        &mut self,
        name: &str,
        cp_type: &str,
        category: Option<&str>,
    ) -> NodeId {
        let key = format!("{}_{}", cp_type, name);

        if let Some(&id) = self.counterparty_nodes.get(&key) {
            return id;
        }

        let mut node = GraphNode::new(
            0,
            NodeType::Custom("Counterparty".to_string()),
            key.clone(),
            name.to_string(),
        );

        node.categorical_features
            .insert("counterparty_type".to_string(), cp_type.to_string());

        if let Some(cat) = category {
            node.categorical_features
                .insert("category".to_string(), cat.to_string());
        }

        let id = self.graph.add_node(node);
        self.counterparty_nodes.insert(key, id);
        id
    }

    /// Adds transactions to the graph.
    pub fn add_transactions(&mut self, transactions: &[BankTransaction]) {
        if !self.config.create_transaction_edges {
            return;
        }

        for txn in transactions {
            self.add_transaction(txn);
        }
    }

    /// Adds a single transaction to the graph.
    fn add_transaction(&mut self, txn: &BankTransaction) {
        let amount: f64 = txn.amount.try_into().unwrap_or(0.0);
        if amount < self.config.min_transaction_amount {
            return;
        }

        // Get or create account node
        let account_key = txn.account_id.to_string();
        let account_node = *self.account_nodes.get(&account_key).unwrap_or(&0);
        if account_node == 0 {
            return; // Account not in graph
        }

        // Get or create counterparty node
        let cp_key = format!("counterparty_{}", txn.counterparty.name);
        let counterparty_node = if let Some(&id) = self.counterparty_nodes.get(&cp_key) {
            id
        } else {
            self.add_counterparty_node(
                &txn.counterparty.name,
                &format!("{:?}", txn.counterparty.counterparty_type),
                None,
            )
        };

        // Determine edge direction based on transaction direction
        let (source, target) = match txn.direction {
            Direction::Inbound => (counterparty_node, account_node),
            Direction::Outbound => (account_node, counterparty_node),
        };

        if self.config.aggregate_parallel_edges {
            self.aggregate_transaction_edge(source, target, txn);
        } else {
            let edge = self.create_transaction_edge(source, target, txn);
            self.graph.add_edge(edge);
        }
    }

    /// Creates a transaction edge with features.
    fn create_transaction_edge(
        &self,
        source: NodeId,
        target: NodeId,
        txn: &BankTransaction,
    ) -> GraphEdge {
        let amount: f64 = txn.amount.try_into().unwrap_or(0.0);

        let mut edge = GraphEdge::new(0, source, target, EdgeType::Transaction)
            .with_weight(amount)
            .with_timestamp(txn.timestamp_initiated.date_naive());

        // Add transaction properties
        edge.properties.insert(
            "transaction_id".to_string(),
            crate::models::EdgeProperty::String(txn.transaction_id.to_string()),
        );
        edge.properties.insert(
            "channel".to_string(),
            crate::models::EdgeProperty::String(format!("{:?}", txn.channel)),
        );
        edge.properties.insert(
            "category".to_string(),
            crate::models::EdgeProperty::String(format!("{:?}", txn.category)),
        );

        // Add numeric features
        // Log amount
        edge.features.push((amount + 1.0).ln());

        // Direction encoding
        edge.features.push(match txn.direction {
            Direction::Inbound => 1.0,
            Direction::Outbound => 0.0,
        });

        // Channel encoding
        let channel_code = match format!("{:?}", txn.channel).as_str() {
            "CardPresent" => 0.0,
            "CardNotPresent" => 1.0,
            "Ach" => 2.0,
            "Wire" => 3.0,
            "Cash" => 4.0,
            "Atm" => 5.0,
            "Branch" => 6.0,
            "Mobile" => 7.0,
            "Online" => 8.0,
            "Swift" => 9.0,
            _ => 10.0,
        };
        edge.features.push(channel_code / 10.0); // Normalized

        // Temporal features
        if self.config.include_temporal_features {
            let weekday = txn.timestamp_initiated.weekday().num_days_from_monday() as f64;
            edge.features.push(weekday / 6.0);

            let hour = txn.timestamp_initiated.hour() as f64;
            edge.features.push(hour / 23.0);

            let day = txn.timestamp_initiated.day() as f64;
            edge.features.push(day / 31.0);

            let month = txn.timestamp_initiated.month() as f64;
            edge.features.push(month / 12.0);

            // Is weekend
            edge.features.push(if weekday >= 5.0 { 1.0 } else { 0.0 });

            // Is off-hours (before 7am or after 10pm)
            let is_off_hours = !(7.0..=22.0).contains(&hour);
            edge.features.push(if is_off_hours { 1.0 } else { 0.0 });
        }

        // Risk features
        if self.config.include_risk_features {
            // Is cash transaction
            edge.features.push(if txn.is_cash() { 1.0 } else { 0.0 });

            // Is cross-border
            edge.features
                .push(if txn.is_cross_border() { 1.0 } else { 0.0 });

            // Risk score from transaction
            edge.features
                .push(txn.calculate_risk_score() as f64 / 100.0);
        }

        // Ground truth labels
        if txn.is_suspicious {
            edge = edge.as_anomaly(&format!(
                "{:?}",
                txn.suspicion_reason.unwrap_or(AmlTypology::Structuring)
            ));

            if let Some(typology) = txn.suspicion_reason {
                edge.labels.push(format!("{:?}", typology));
            }

            if let Some(stage) = txn.laundering_stage {
                edge.labels.push(format!("{:?}", stage));
            }

            if txn.is_spoofed {
                edge.labels.push("spoofed".to_string());
            }
        }

        edge
    }

    /// Aggregates transaction edges between same source and target.
    fn aggregate_transaction_edge(
        &mut self,
        source: NodeId,
        target: NodeId,
        txn: &BankTransaction,
    ) {
        let key = (source, target);
        let amount: f64 = txn.amount.try_into().unwrap_or(0.0);
        let date = txn.timestamp_initiated.date_naive();

        let agg = self
            .edge_aggregation
            .entry(key)
            .or_insert(AggregatedBankingEdge {
                source,
                target,
                total_amount: 0.0,
                count: 0,
                suspicious_count: 0,
                first_date: date,
                last_date: date,
                channels: HashMap::new(),
            });

        agg.total_amount += amount;
        agg.count += 1;

        if txn.is_suspicious {
            agg.suspicious_count += 1;
        }

        if date < agg.first_date {
            agg.first_date = date;
        }
        if date > agg.last_date {
            agg.last_date = date;
        }

        let channel = format!("{:?}", txn.channel);
        *agg.channels.entry(channel).or_insert(0) += 1;
    }

    /// Builds the final graph.
    pub fn build(mut self) -> Graph {
        // If aggregating, create the aggregated edges now
        if self.config.aggregate_parallel_edges {
            for ((source, target), agg) in self.edge_aggregation {
                let mut edge = GraphEdge::new(0, source, target, EdgeType::Transaction)
                    .with_weight(agg.total_amount)
                    .with_timestamp(agg.last_date);

                // Aggregation features
                edge.features.push((agg.total_amount + 1.0).ln());
                edge.features.push(agg.count as f64);
                edge.features.push(agg.suspicious_count as f64);
                edge.features
                    .push(agg.suspicious_count as f64 / agg.count.max(1) as f64);

                let duration = (agg.last_date - agg.first_date).num_days() as f64;
                edge.features.push(duration);

                // Average amount per transaction
                edge.features
                    .push(agg.total_amount / agg.count.max(1) as f64);

                // Transaction frequency (per day)
                edge.features.push(agg.count as f64 / duration.max(1.0));

                // Number of unique channels
                edge.features.push(agg.channels.len() as f64);

                // Mark as anomaly if any suspicious transactions
                if agg.suspicious_count > 0 {
                    edge = edge.as_anomaly("suspicious_link");
                }

                self.graph.add_edge(edge);
            }
        }

        self.graph.compute_statistics();
        self.graph
    }
}

/// Aggregated banking edge data (for future edge aggregation support).
#[allow(dead_code)]
struct AggregatedBankingEdge {
    source: NodeId,
    target: NodeId,
    total_amount: f64,
    count: usize,
    suspicious_count: usize,
    first_date: chrono::NaiveDate,
    last_date: chrono::NaiveDate,
    channels: HashMap<String, usize>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_banking::models::CounterpartyRef;
    use datasynth_core::models::banking::{
        BankAccountType, TransactionCategory, TransactionChannel,
    };
    use rust_decimal::Decimal;
    use uuid::Uuid;

    fn create_test_customer() -> BankingCustomer {
        BankingCustomer::new_retail(
            Uuid::new_v4(),
            "John",
            "Doe",
            "US",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )
    }

    fn create_test_account(customer: &BankingCustomer) -> BankAccount {
        BankAccount::new(
            Uuid::new_v4(),
            "****1234".to_string(),
            BankAccountType::Checking,
            customer.customer_id,
            "USD",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )
    }

    fn create_test_transaction(account: &BankAccount) -> BankTransaction {
        BankTransaction::new(
            Uuid::new_v4(),
            account.account_id,
            Decimal::from(1000),
            "USD",
            Direction::Outbound,
            TransactionChannel::CardPresent,
            TransactionCategory::Shopping,
            CounterpartyRef::merchant(Uuid::new_v4(), "Test Store"),
            "Test purchase",
            chrono::Utc::now(),
        )
    }

    #[test]
    fn test_build_banking_graph() {
        let customer = create_test_customer();
        let account = create_test_account(&customer);
        let txn = create_test_transaction(&account);

        let mut builder = BankingGraphBuilder::new(BankingGraphConfig::default());
        builder.add_customers(std::slice::from_ref(&customer));
        builder.add_accounts(
            std::slice::from_ref(&account),
            std::slice::from_ref(&customer),
        );
        builder.add_transactions(std::slice::from_ref(&txn));

        let graph = builder.build();

        // Should have customer, account, and counterparty nodes
        assert!(graph.node_count() >= 2);
        // Should have ownership and transaction edges
        assert!(graph.edge_count() >= 1);
    }

    #[test]
    fn test_suspicious_transaction_labels() {
        let customer = create_test_customer();
        let account = create_test_account(&customer);
        let mut txn = create_test_transaction(&account);

        // Mark as suspicious
        txn = txn.mark_suspicious(AmlTypology::Structuring, "CASE-001");

        let mut builder = BankingGraphBuilder::new(BankingGraphConfig::default());
        builder.add_customers(std::slice::from_ref(&customer));
        builder.add_accounts(
            std::slice::from_ref(&account),
            std::slice::from_ref(&customer),
        );
        builder.add_transactions(std::slice::from_ref(&txn));

        let graph = builder.build();

        // Check that suspicious edge exists
        let suspicious_edges = graph.anomalous_edges();
        assert!(!suspicious_edges.is_empty());
    }
}
