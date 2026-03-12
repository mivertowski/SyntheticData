//! Tests for budget rebalancing API and enforced phase ordering.
//!
//! Verifies that:
//! - `NodeBudget::suggest()` correctly calculates suggested allocations
//! - `NodeBudget::rebalance()` redistributes surplus from low-demand layers
//! - `HypergraphBuilder::count_demand()` tallies entities per layer
//! - `HypergraphBuilder::add_all_ordered()` inserts audit before banking
//! - Audit nodes survive when banking is large and L2 budget is tight

use chrono::{NaiveDate, Utc};
use rust_decimal_macros::dec;
use uuid::Uuid;

use datasynth_banking::models::{BankAccount, BankTransaction, BankingCustomer, CounterpartyRef};
use datasynth_core::models::audit::{
    AuditEngagement, AuditEvidence, AuditFinding, EngagementType, EvidenceSource, EvidenceType,
    FindingType, JudgmentType, ProfessionalJudgment, RiskAssessment, RiskCategory, Workpaper,
    WorkpaperSection,
};
use datasynth_core::models::banking::{
    BankAccountType, Direction, TransactionCategory, TransactionChannel,
};
use datasynth_core::models::{ControlType, InternalControl, Vendor, VendorType};
use datasynth_graph::builders::hypergraph::{
    BuilderInput, HypergraphBuilder, HypergraphConfig, LayerDemand,
};
use datasynth_graph::models::hypergraph::NodeBudget;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

// ---------------------------------------------------------------------------
// NodeBudget::suggest
// ---------------------------------------------------------------------------

#[test]
fn suggest_returns_same_total() {
    let budget = NodeBudget::new(10_000);
    let suggestion = budget.suggest(100, 5000, 200);
    assert_eq!(suggestion.total, 10_000);
    assert_eq!(
        suggestion.l1_suggested + suggestion.l2_suggested + suggestion.l3_suggested,
        10_000,
        "Suggested allocations must sum to total"
    );
}

#[test]
fn suggest_clamps_to_demand_when_below_max() {
    let budget = NodeBudget::new(10_000);
    // L1 max=2000, L2 max=7000, L3 max=1000
    // demand: L1=100, L2=4000, L3=100
    let suggestion = budget.suggest(100, 4000, 100);
    assert_eq!(suggestion.l1_suggested, 100);
    assert_eq!(suggestion.l3_suggested, 100);
    // All surplus goes to L2 (no unsatisfied demand elsewhere)
    assert_eq!(suggestion.l2_suggested, 10_000 - 100 - 100);
}

#[test]
fn suggest_redistributes_surplus_proportionally() {
    let budget = NodeBudget::new(10_000);
    // L1 max=2000, L2 max=7000, L3 max=1000
    // demand: L1=100, L2=9000, L3=3000
    let suggestion = budget.suggest(100, 9000, 3000);
    // surplus from L1 = 2000-100 = 1900
    // L2 unsat = 9000-7000 = 2000, L3 unsat = 3000-1000 = 2000
    // Proportional: each gets floor(1900 * 2000/4000) = 950
    assert_eq!(suggestion.l1_suggested, 100);
    assert_eq!(suggestion.l2_suggested, 7000 + 950);
    assert_eq!(suggestion.l3_suggested, 1000 + 950);
    assert_eq!(suggestion.surplus_redistributed, 1900);
}

#[test]
fn rebalance_applies_suggestion() {
    let mut budget = NodeBudget::new(5000);
    let old_total = budget.total_max();
    budget.rebalance(100, 4000, 100);
    assert_eq!(budget.total_max(), old_total, "Total must not change");
    assert_eq!(budget.layer1_max, 100);
    assert_eq!(budget.layer3_max, 100);
    assert_eq!(budget.layer2_max, 5000 - 100 - 100);
}

// ---------------------------------------------------------------------------
// HypergraphBuilder::suggest_budget + rebalance_with_demand
// ---------------------------------------------------------------------------

#[test]
fn builder_suggest_budget_returns_suggestion() {
    let builder = HypergraphBuilder::new(HypergraphConfig {
        max_nodes: 5000,
        ..Default::default()
    });
    let demand = LayerDemand {
        l1: 50,
        l2: 3000,
        l3: 200,
    };
    let suggestion = builder.suggest_budget(&demand);
    assert_eq!(suggestion.total, 5000);
    assert!(
        suggestion.l1_suggested >= 50,
        "L1 should get at least its demand"
    );
    assert!(
        suggestion.l2_suggested >= 3000,
        "L2 should get at least its demand"
    );
    assert!(
        suggestion.l3_suggested >= 200,
        "L3 should get at least its demand"
    );
}

#[test]
fn builder_rebalance_with_demand_updates_budget() {
    let mut builder = HypergraphBuilder::new(HypergraphConfig {
        max_nodes: 5000,
        ..Default::default()
    });
    let demand = LayerDemand {
        l1: 50,
        l2: 4000,
        l3: 100,
    };
    builder.rebalance_with_demand(&demand);
    let b = builder.budget();
    assert_eq!(b.layer1_max, 50);
    assert!(b.layer2_max >= 4000);
    assert_eq!(b.total_max(), 5000);
}

// ---------------------------------------------------------------------------
// HypergraphBuilder::count_demand
// ---------------------------------------------------------------------------

#[test]
fn count_demand_tallies_entities_per_layer() {
    let controls = vec![InternalControl::new(
        "C001",
        "Test Control",
        ControlType::Preventive,
        "Prevent errors",
    )];
    let vendors = vec![
        Vendor::new("V001", "Vendor 1", VendorType::Supplier),
        Vendor::new("V002", "Vendor 2", VendorType::Supplier),
    ];

    let engagements = vec![AuditEngagement::new(
        "E1",
        "Test Co",
        EngagementType::default(),
        2024,
        date(2024, 12, 31),
    )];

    let input = BuilderInput {
        controls: &controls,
        vendors: &vendors,
        audit_engagements: &engagements,
        ..Default::default()
    };
    let demand = HypergraphBuilder::count_demand(&input);
    // L1 = 22 (COSO) + 1 (control) + 2 (vendors) = 25
    assert_eq!(demand.l1, 25);
    // L2 = 1 (engagement)
    assert_eq!(demand.l2, 1);
    // L3 = 0
    assert_eq!(demand.l3, 0);
}

// ---------------------------------------------------------------------------
// Phase ordering: audit before banking
// ---------------------------------------------------------------------------

fn make_audit_engagements(n: usize) -> Vec<AuditEngagement> {
    (0..n)
        .map(|_| {
            AuditEngagement::new(
                "E1",
                "Test Co",
                EngagementType::default(),
                2024,
                date(2024, 12, 31),
            )
        })
        .collect()
}

fn make_workpapers(engagements: &[AuditEngagement], per_engagement: usize) -> Vec<Workpaper> {
    engagements
        .iter()
        .flat_map(|eng| {
            (0..per_engagement).map(move |i| {
                Workpaper::new(
                    eng.engagement_id,
                    &format!("WP-{i}"),
                    &format!("Workpaper {i}"),
                    WorkpaperSection::default(),
                )
            })
        })
        .collect()
}

fn make_audit_findings(
    engagements: &[AuditEngagement],
    per_engagement: usize,
) -> Vec<AuditFinding> {
    engagements
        .iter()
        .flat_map(|eng| {
            (0..per_engagement)
                .map(move |_| AuditFinding::new(eng.engagement_id, FindingType::default(), "Test"))
        })
        .collect()
}

fn make_audit_evidence(
    engagements: &[AuditEngagement],
    per_engagement: usize,
) -> Vec<AuditEvidence> {
    engagements
        .iter()
        .flat_map(|eng| {
            (0..per_engagement).map(move |_| {
                AuditEvidence::new(
                    eng.engagement_id,
                    EvidenceType::default(),
                    EvidenceSource::default(),
                    "Test evidence",
                )
            })
        })
        .collect()
}

fn make_risk_assessments(
    engagements: &[AuditEngagement],
    per_engagement: usize,
) -> Vec<RiskAssessment> {
    engagements
        .iter()
        .flat_map(|eng| {
            (0..per_engagement).map(move |_| {
                RiskAssessment::new(
                    eng.engagement_id,
                    RiskCategory::default(),
                    "Test area",
                    "Test assertion",
                )
            })
        })
        .collect()
}

fn make_judgments(
    engagements: &[AuditEngagement],
    per_engagement: usize,
) -> Vec<ProfessionalJudgment> {
    engagements
        .iter()
        .flat_map(|eng| {
            (0..per_engagement).map(move |_| {
                ProfessionalJudgment::new(
                    eng.engagement_id,
                    JudgmentType::default(),
                    "Test subject",
                )
            })
        })
        .collect()
}

fn make_banking_customers(n: usize) -> Vec<BankingCustomer> {
    (0..n)
        .map(|i| {
            BankingCustomer::new_retail(
                Uuid::new_v4(),
                &format!("First{i}"),
                &format!("Last{i}"),
                "US",
                date(2024, 1, 1),
            )
        })
        .collect()
}

fn make_bank_accounts(customers: &[BankingCustomer]) -> Vec<BankAccount> {
    customers
        .iter()
        .map(|c| {
            BankAccount::new(
                Uuid::new_v4(),
                format!("ACCT-{}", c.customer_id),
                BankAccountType::Checking,
                c.customer_id,
                "USD",
                date(2024, 1, 1),
            )
        })
        .collect()
}

fn make_bank_transactions(accounts: &[BankAccount], per_account: usize) -> Vec<BankTransaction> {
    accounts
        .iter()
        .flat_map(|acct| {
            (0..per_account).map(move |i| {
                BankTransaction::new(
                    Uuid::new_v4(),
                    acct.account_id,
                    dec!(100),
                    "USD",
                    Direction::Outbound,
                    TransactionChannel::default(),
                    TransactionCategory::Salary,
                    CounterpartyRef::merchant(Uuid::new_v4(), &format!("Merchant-{i}")),
                    &format!("TXN-{i}"),
                    Utc::now(),
                )
            })
        })
        .collect()
}

/// Minimal config that only enables audit + banking + cross-layer edges off.
fn audit_bank_config(max_nodes: usize) -> HypergraphConfig {
    HypergraphConfig {
        max_nodes,
        include_coso: false,
        include_controls: false,
        include_sox: false,
        include_vendors: false,
        include_customers: false,
        include_employees: false,
        include_p2p: false,
        include_o2c: false,
        include_s2c: false,
        include_h2r: false,
        include_mfg: false,
        include_bank: true,
        include_audit: true,
        include_compliance: false,
        include_r2r: false,
        include_tax: false,
        include_treasury: false,
        include_esg: false,
        include_project: false,
        include_intercompany: false,
        include_temporal_events: false,
        include_accounts: false,
        je_as_hyperedges: false,
        include_cross_layer_edges: false,
        ..Default::default()
    }
}

#[test]
fn audit_nodes_not_dropped_when_banking_is_large() {
    let mut builder = HypergraphBuilder::new(audit_bank_config(500));

    // ~35 audit entities
    let engagements = make_audit_engagements(5);
    let workpapers = make_workpapers(&engagements, 2);
    let findings = make_audit_findings(&engagements, 1);
    let evidence = make_audit_evidence(&engagements, 1);
    let risks = make_risk_assessments(&engagements, 1);
    let judgments = make_judgments(&engagements, 1);
    let total_audit = engagements.len()
        + workpapers.len()
        + findings.len()
        + evidence.len()
        + risks.len()
        + judgments.len();

    // 500 bank transactions across 10 customers
    let bank_customers = make_banking_customers(10);
    let bank_accounts = make_bank_accounts(&bank_customers);
    let bank_transactions = make_bank_transactions(&bank_accounts, 50);

    let input = BuilderInput {
        audit_engagements: &engagements,
        workpapers: &workpapers,
        audit_findings: &findings,
        audit_evidence: &evidence,
        risk_assessments: &risks,
        professional_judgments: &judgments,
        banking_customers: &bank_customers,
        bank_accounts: &bank_accounts,
        bank_transactions: &bank_transactions,
        ..Default::default()
    };

    let demand = HypergraphBuilder::count_demand(&input);
    builder.rebalance_with_demand(&demand);
    builder.add_all_ordered(&input);
    let hg = builder.build();

    let audit_types = [
        "audit_engagement",
        "workpaper",
        "audit_finding",
        "audit_evidence",
        "risk_assessment",
        "professional_judgment",
    ];
    let audit_node_count: usize = hg
        .nodes
        .iter()
        .filter(|n| audit_types.contains(&n.entity_type.as_str()))
        .count();

    assert_eq!(
        audit_node_count, total_audit,
        "All {} audit nodes must survive (got {}); banking must not drop them.",
        total_audit, audit_node_count
    );

    let bank_node_count: usize = hg
        .nodes
        .iter()
        .filter(|n| {
            matches!(
                n.entity_type.as_str(),
                "banking_customer" | "bank_account" | "bank_transaction"
            )
        })
        .count();
    assert!(bank_node_count > 0, "Some banking nodes should be present");
}

#[test]
fn add_all_ordered_inserts_audit_before_banking() {
    let mut builder = HypergraphBuilder::new(audit_bank_config(5000));

    let engagements = make_audit_engagements(2);
    let workpapers = make_workpapers(&engagements, 1);
    let findings = make_audit_findings(&engagements, 1);
    let evidence = make_audit_evidence(&engagements, 1);
    let risks = make_risk_assessments(&engagements, 1);
    let judgments = make_judgments(&engagements, 1);

    let bank_customers = make_banking_customers(5);
    let bank_accounts = make_bank_accounts(&bank_customers);
    let bank_transactions = make_bank_transactions(&bank_accounts, 10);

    let input = BuilderInput {
        audit_engagements: &engagements,
        workpapers: &workpapers,
        audit_findings: &findings,
        audit_evidence: &evidence,
        risk_assessments: &risks,
        professional_judgments: &judgments,
        banking_customers: &bank_customers,
        bank_accounts: &bank_accounts,
        bank_transactions: &bank_transactions,
        ..Default::default()
    };

    builder.add_all_ordered(&input);
    let hg = builder.build();

    let first_audit_idx = hg
        .nodes
        .iter()
        .position(|n| n.entity_type == "audit_engagement")
        .expect("Should have audit nodes");

    let first_bank_txn_idx = hg
        .nodes
        .iter()
        .position(|n| n.entity_type == "bank_transaction")
        .expect("Should have bank transaction nodes");

    assert!(
        first_audit_idx < first_bank_txn_idx,
        "Audit nodes (idx={}) must appear before bank transactions (idx={}) \
         due to phase ordering",
        first_audit_idx, first_bank_txn_idx
    );
}

// ---------------------------------------------------------------------------
// Budget rebalancing redistributes unused capacity
// ---------------------------------------------------------------------------

#[test]
fn budget_rebalancing_redistributes_unused_capacity() {
    let mut budget = NodeBudget::new(5000);
    budget.rebalance(100, 4000, 100);
    assert_eq!(budget.layer1_max, 100);
    assert_eq!(budget.layer3_max, 100);
    assert!(budget.layer2_max >= 4000, "L2 got {}", budget.layer2_max);
    assert_eq!(budget.total_max(), 5000);
}

#[test]
fn budget_rebalancing_with_l3_surplus_goes_to_l2() {
    let mut budget = NodeBudget::new(10_000);
    budget.rebalance(200, 8000, 50);
    assert_eq!(budget.layer1_max, 200);
    assert_eq!(budget.layer3_max, 50);
    assert_eq!(budget.layer2_max, 10_000 - 200 - 50);
    assert_eq!(budget.total_max(), 10_000);
}

#[test]
fn rebalance_with_zero_demand_gives_all_to_l2() {
    let mut budget = NodeBudget::new(1000);
    budget.rebalance(0, 0, 0);
    assert_eq!(budget.layer1_max, 0);
    assert_eq!(budget.layer2_max, 1000);
    assert_eq!(budget.layer3_max, 0);
}

#[test]
fn rebalance_when_all_layers_over_budget() {
    let mut budget = NodeBudget::new(1000);
    // L1 max=200, L2 max=700, L3 max=100
    budget.rebalance(300, 900, 200);
    // No surplus, nothing moves
    assert_eq!(budget.layer1_max, 200);
    assert_eq!(budget.layer2_max, 700);
    assert_eq!(budget.layer3_max, 100);
    assert_eq!(budget.total_max(), 1000);
}

// ---------------------------------------------------------------------------
// NodeBudgetSuggestion edge cases
// ---------------------------------------------------------------------------

#[test]
fn suggest_with_exact_demand_matches_default() {
    let budget = NodeBudget::new(1000);
    let suggestion = budget.suggest(200, 700, 100);
    assert_eq!(suggestion.l1_suggested, 200);
    assert_eq!(suggestion.l2_suggested, 700);
    assert_eq!(suggestion.l3_suggested, 100);
    assert_eq!(suggestion.surplus_redistributed, 0);
}

#[test]
fn suggest_with_massive_l2_demand() {
    let budget = NodeBudget::new(1000);
    let suggestion = budget.suggest(10, 100_000, 10);
    assert_eq!(suggestion.l1_suggested, 10);
    assert_eq!(suggestion.l3_suggested, 10);
    assert_eq!(suggestion.l2_suggested, 980);
    assert_eq!(suggestion.total, 1000);
}

// ---------------------------------------------------------------------------
// Integration: add_all_ordered + rebalancing produces valid hypergraph
// ---------------------------------------------------------------------------

#[test]
fn add_all_ordered_produces_valid_metadata() {
    let config = HypergraphConfig {
        max_nodes: 2000,
        include_coso: true,
        include_controls: true,
        include_sox: false,
        include_vendors: false,
        include_customers: false,
        include_employees: false,
        include_p2p: false,
        include_o2c: false,
        include_s2c: false,
        include_h2r: false,
        include_mfg: false,
        include_bank: false,
        include_audit: true,
        include_compliance: false,
        include_r2r: false,
        include_tax: false,
        include_treasury: false,
        include_esg: false,
        include_project: false,
        include_intercompany: false,
        include_temporal_events: false,
        include_accounts: false,
        je_as_hyperedges: false,
        include_cross_layer_edges: true,
        ..Default::default()
    };
    let mut builder = HypergraphBuilder::new(config);

    let controls = vec![InternalControl::new(
        "C001",
        "Test Control",
        ControlType::Preventive,
        "Prevent errors",
    )];
    let engagements = make_audit_engagements(3);
    let workpapers = make_workpapers(&engagements, 2);

    let input = BuilderInput {
        controls: &controls,
        audit_engagements: &engagements,
        workpapers: &workpapers,
        ..Default::default()
    };

    let demand = HypergraphBuilder::count_demand(&input);
    builder.rebalance_with_demand(&demand);
    builder.add_all_ordered(&input);
    let hg = builder.build();

    assert_eq!(hg.metadata.num_nodes, hg.nodes.len());
    assert_eq!(hg.metadata.num_edges, hg.edges.len());
    assert_eq!(hg.metadata.num_hyperedges, hg.hyperedges.len());

    assert!(hg.budget_report.total_used <= hg.budget_report.total_budget);
    assert!(hg.budget_report.layer1_used <= hg.budget_report.layer1_budget);
    assert!(hg.budget_report.layer2_used <= hg.budget_report.layer2_budget);

    for node in &hg.nodes {
        assert!(
            node.properties.contains_key("process_family"),
            "Node {} ({}) missing process_family tag",
            node.id, node.entity_type
        );
    }
}
