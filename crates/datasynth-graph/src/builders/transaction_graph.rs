//! Transaction graph builder.
//!
//! Builds a graph where:
//! - Nodes are GL accounts (or entities like vendors, customers)
//! - Edges are transactions (journal entry lines)

use std::collections::HashMap;

use rust_decimal::Decimal;

use datasynth_core::models::JournalEntry;

use crate::models::{
    AccountNode, EdgeType, Graph, GraphEdge, GraphNode, GraphType, NodeId, NodeType,
    TransactionEdge,
};

/// Configuration for transaction graph building.
#[derive(Debug, Clone)]
pub struct TransactionGraphConfig {
    /// Whether to include vendor nodes.
    pub include_vendors: bool,
    /// Whether to include customer nodes.
    pub include_customers: bool,
    /// Whether to create edges between debit and credit accounts.
    pub create_debit_credit_edges: bool,
    /// Whether to create edges from/to document nodes.
    pub include_document_nodes: bool,
    /// Minimum edge weight to include.
    pub min_edge_weight: f64,
    /// Whether to aggregate parallel edges.
    pub aggregate_parallel_edges: bool,
}

impl Default for TransactionGraphConfig {
    fn default() -> Self {
        Self {
            include_vendors: false,
            include_customers: false,
            create_debit_credit_edges: true,
            include_document_nodes: false,
            min_edge_weight: 0.0,
            aggregate_parallel_edges: false,
        }
    }
}

/// Builder for transaction graphs.
pub struct TransactionGraphBuilder {
    config: TransactionGraphConfig,
    graph: Graph,
    /// Map from account code to node ID.
    account_nodes: HashMap<String, NodeId>,
    /// Map from document number to node ID (if document nodes enabled).
    document_nodes: HashMap<String, NodeId>,
    /// For edge aggregation: (source, target) -> aggregated amount.
    edge_aggregation: HashMap<(NodeId, NodeId), AggregatedEdge>,
}

impl TransactionGraphBuilder {
    /// Creates a new transaction graph builder.
    pub fn new(config: TransactionGraphConfig) -> Self {
        Self {
            config,
            graph: Graph::new("transaction_network", GraphType::Transaction),
            account_nodes: HashMap::new(),
            document_nodes: HashMap::new(),
            edge_aggregation: HashMap::new(),
        }
    }

    /// Adds a journal entry to the graph.
    pub fn add_journal_entry(&mut self, entry: &JournalEntry) {
        if self.config.include_document_nodes {
            self.add_journal_entry_with_document(entry);
        } else if self.config.create_debit_credit_edges {
            self.add_journal_entry_debit_credit(entry);
        }
    }

    /// Adds journal entry creating edges between debit and credit accounts.
    fn add_journal_entry_debit_credit(&mut self, entry: &JournalEntry) {
        // Collect debit and credit lines
        let debits: Vec<_> = entry
            .lines
            .iter()
            .filter(|l| l.debit_amount > Decimal::ZERO)
            .collect();

        let credits: Vec<_> = entry
            .lines
            .iter()
            .filter(|l| l.credit_amount > Decimal::ZERO)
            .collect();

        // Create edges from debit accounts to credit accounts
        for debit in &debits {
            let source_id = self.get_or_create_account_node(
                debit.account_code(),
                debit.account_description(),
                entry.company_code(),
            );

            for credit in &credits {
                let target_id = self.get_or_create_account_node(
                    credit.account_code(),
                    credit.account_description(),
                    entry.company_code(),
                );

                // Calculate edge weight (proportional allocation)
                let total_debit: Decimal = debits.iter().map(|d| d.debit_amount).sum();
                let total_credit: Decimal = credits.iter().map(|c| c.credit_amount).sum();

                let proportion =
                    (debit.debit_amount / total_debit) * (credit.credit_amount / total_credit);
                let edge_amount = debit.debit_amount * proportion;
                let edge_weight: f64 = edge_amount.try_into().unwrap_or(0.0);

                if edge_weight < self.config.min_edge_weight {
                    continue;
                }

                if self.config.aggregate_parallel_edges {
                    self.aggregate_edge(source_id, target_id, edge_weight, entry);
                } else {
                    let mut tx_edge = TransactionEdge::new(
                        0,
                        source_id,
                        target_id,
                        entry.document_number(),
                        entry.posting_date(),
                        edge_amount,
                        true,
                    );
                    tx_edge.company_code = entry.company_code().to_string();
                    tx_edge.cost_center = debit.cost_center.clone();
                    tx_edge.business_process = entry
                        .header
                        .business_process
                        .as_ref()
                        .map(|bp| format!("{:?}", bp));
                    tx_edge.compute_features();

                    // Propagate anomaly flag from journal entry to graph edge
                    if entry.header.is_anomaly {
                        tx_edge.edge.is_anomaly = true;
                        if let Some(ref anomaly_type) = entry.header.anomaly_type {
                            tx_edge.edge.anomaly_type = Some(format!("{:?}", anomaly_type));
                        }
                    }

                    self.graph.add_edge(tx_edge.edge);
                }
            }
        }
    }

    /// Adds journal entry with document nodes.
    fn add_journal_entry_with_document(&mut self, entry: &JournalEntry) {
        // Create or get document node
        let doc_id =
            self.get_or_create_document_node(&entry.document_number(), entry.company_code());

        // Create edges from document to each account
        for line in &entry.lines {
            let account_id = self.get_or_create_account_node(
                line.account_code(),
                line.account_description(),
                entry.company_code(),
            );

            let is_debit = line.debit_amount > Decimal::ZERO;
            let amount = if is_debit {
                line.debit_amount
            } else {
                line.credit_amount
            };

            let mut tx_edge = TransactionEdge::new(
                0,
                doc_id,
                account_id,
                entry.document_number(),
                entry.posting_date(),
                amount,
                is_debit,
            );
            tx_edge.company_code = entry.company_code().to_string();
            tx_edge.cost_center = line.cost_center.clone();
            tx_edge.business_process = entry
                .header
                .business_process
                .as_ref()
                .map(|bp| format!("{:?}", bp));
            tx_edge.compute_features();

            // Propagate anomaly flag from journal entry to graph edge
            if entry.header.is_anomaly {
                tx_edge.edge.is_anomaly = true;
                if let Some(ref anomaly_type) = entry.header.anomaly_type {
                    tx_edge.edge.anomaly_type = Some(format!("{:?}", anomaly_type));
                }
            }

            self.graph.add_edge(tx_edge.edge);
        }
    }

    /// Gets or creates an account node.
    fn get_or_create_account_node(
        &mut self,
        account_code: &str,
        account_name: &str,
        company_code: &str,
    ) -> NodeId {
        let key = format!("{}_{}", company_code, account_code);

        if let Some(&id) = self.account_nodes.get(&key) {
            return id;
        }

        let mut account = AccountNode::new(
            0,
            account_code.to_string(),
            account_name.to_string(),
            Self::infer_account_type(account_code),
            company_code.to_string(),
        );
        account.is_balance_sheet = Self::is_balance_sheet_account(account_code);
        account.normal_balance = Self::infer_normal_balance(account_code);
        account.compute_features();

        let id = self.graph.add_node(account.node);
        self.account_nodes.insert(key, id);
        id
    }

    /// Gets or creates a document node.
    fn get_or_create_document_node(&mut self, document_number: &str, company_code: &str) -> NodeId {
        let key = format!("{}_{}", company_code, document_number);

        if let Some(&id) = self.document_nodes.get(&key) {
            return id;
        }

        let node = GraphNode::new(
            0,
            NodeType::JournalEntry,
            document_number.to_string(),
            document_number.to_string(),
        );

        let id = self.graph.add_node(node);
        self.document_nodes.insert(key, id);
        id
    }

    /// Aggregates edges between the same source and target.
    fn aggregate_edge(
        &mut self,
        source: NodeId,
        target: NodeId,
        weight: f64,
        entry: &JournalEntry,
    ) {
        let key = (source, target);
        let agg = self.edge_aggregation.entry(key).or_insert(AggregatedEdge {
            source,
            target,
            total_weight: 0.0,
            count: 0,
            first_date: entry.posting_date(),
            last_date: entry.posting_date(),
        });

        agg.total_weight += weight;
        agg.count += 1;
        if entry.posting_date() < agg.first_date {
            agg.first_date = entry.posting_date();
        }
        if entry.posting_date() > agg.last_date {
            agg.last_date = entry.posting_date();
        }
    }

    /// Infers account type from account code.
    fn infer_account_type(account_code: &str) -> String {
        if account_code.is_empty() {
            return "Unknown".to_string();
        }

        match account_code
            .chars()
            .next()
            .expect("non-empty checked above")
        {
            '1' => "Asset".to_string(),
            '2' => "Liability".to_string(),
            '3' => "Equity".to_string(),
            '4' => "Revenue".to_string(),
            '5' | '6' | '7' => "Expense".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    /// Checks if account is balance sheet.
    fn is_balance_sheet_account(account_code: &str) -> bool {
        if account_code.is_empty() {
            return false;
        }

        matches!(
            account_code
                .chars()
                .next()
                .expect("non-empty checked above"),
            '1' | '2' | '3'
        )
    }

    /// Infers normal balance from account code.
    fn infer_normal_balance(account_code: &str) -> String {
        if account_code.is_empty() {
            return "Debit".to_string();
        }

        match account_code
            .chars()
            .next()
            .expect("non-empty checked above")
        {
            '1' | '5' | '6' | '7' => "Debit".to_string(),
            '2' | '3' | '4' => "Credit".to_string(),
            _ => "Debit".to_string(),
        }
    }

    /// Builds the final graph.
    pub fn build(mut self) -> Graph {
        // If aggregating, create the aggregated edges now
        if self.config.aggregate_parallel_edges {
            for ((source, target), agg) in self.edge_aggregation {
                let mut edge = GraphEdge::new(0, source, target, EdgeType::Transaction)
                    .with_weight(agg.total_weight)
                    .with_timestamp(agg.last_date);

                // Add aggregation features
                edge.features.push((agg.total_weight + 1.0).ln());
                edge.features.push(agg.count as f64);

                let duration = (agg.last_date - agg.first_date).num_days() as f64;
                edge.features.push(duration);

                self.graph.add_edge(edge);
            }
        }

        self.graph.compute_statistics();
        self.graph
    }

    /// Adds multiple journal entries.
    pub fn add_journal_entries(&mut self, entries: &[JournalEntry]) {
        for entry in entries {
            self.add_journal_entry(entry);
        }
    }
}

/// Aggregated edge data.
#[allow(dead_code)]
struct AggregatedEdge {
    source: NodeId,
    target: NodeId,
    total_weight: f64,
    count: usize,
    first_date: chrono::NaiveDate,
    last_date: chrono::NaiveDate,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::{BusinessProcess, JournalEntryLine};
    use rust_decimal_macros::dec;

    fn create_test_entry() -> JournalEntry {
        let mut entry = JournalEntry::new_simple(
            "JE001".to_string(),
            "1000".to_string(),
            chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "Test Entry".to_string(),
        );

        let doc_id = entry.header.document_id;

        entry.add_line(JournalEntryLine::debit(
            doc_id,
            1,
            "1000".to_string(),
            dec!(1000),
        ));

        entry.add_line(JournalEntryLine::credit(
            doc_id,
            2,
            "4000".to_string(),
            dec!(1000),
        ));

        entry
    }

    fn create_test_entry_with_business_process(bp: BusinessProcess) -> JournalEntry {
        let mut entry = create_test_entry();
        entry.header.business_process = Some(bp);
        entry
    }

    #[test]
    fn test_build_transaction_graph() {
        let mut builder = TransactionGraphBuilder::new(TransactionGraphConfig::default());
        builder.add_journal_entry(&create_test_entry());

        let graph = builder.build();

        assert_eq!(graph.node_count(), 2); // Cash and Revenue
        assert_eq!(graph.edge_count(), 1); // One transaction edge
    }

    #[test]
    fn test_with_document_nodes() {
        let config = TransactionGraphConfig {
            include_document_nodes: true,
            create_debit_credit_edges: false,
            ..Default::default()
        };

        let mut builder = TransactionGraphBuilder::new(config);
        builder.add_journal_entry(&create_test_entry());

        let graph = builder.build();

        assert_eq!(graph.node_count(), 3); // Document + Cash + Revenue
        assert_eq!(graph.edge_count(), 2); // Document to each account
    }

    #[test]
    fn test_business_process_edge_metadata() {
        let mut builder = TransactionGraphBuilder::new(TransactionGraphConfig::default());
        let entry = create_test_entry_with_business_process(BusinessProcess::P2P);
        builder.add_journal_entry(&entry);

        let graph = builder.build();

        // All edges should have the document_number property set
        for edge in graph.edges.values() {
            assert!(edge.properties.contains_key("document_number"));
        }
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_business_process_with_document_nodes() {
        let config = TransactionGraphConfig {
            include_document_nodes: true,
            create_debit_credit_edges: false,
            ..Default::default()
        };

        let mut builder = TransactionGraphBuilder::new(config);
        let entry = create_test_entry_with_business_process(BusinessProcess::O2C);
        builder.add_journal_entry(&entry);

        let graph = builder.build();

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);
    }
}
