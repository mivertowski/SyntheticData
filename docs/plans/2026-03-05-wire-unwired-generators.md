# Wire Unwired Generators Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Connect all existing but unwired generators into the orchestrator for v1.0.0 feature completeness.

**Architecture:** Each generator gets: (1) a snapshot struct added to `EnhancedGenerationResult`, (2) a phase method in the orchestrator, (3) a call site in `generate()`, (4) stats propagation, (5) JSON output in `output_writer.rs`, and (6) a `PhaseConfig` toggle. Generators that need extension (DisruptionManager, CollusionNetwork) get a new `generate_*()` bulk method before wiring. The IndustryTransaction factory dispatches to concrete generators based on `config.global.industry`.

**Tech Stack:** Rust, ChaCha8 RNG, serde, chrono. Test with `cargo test -j2`.

---

## Execution Groups

- **Group A (Tasks 1-2):** Process & Org events — standalone, no data deps
- **Group B (Tasks 3-4):** Counterfactual & RedFlag — need JEs / anomaly labels
- **Group C (Task 5):** Temporal attributes — generic, wraps existing entities
- **Group D (Task 6):** Entity graph — needs master data + JE summaries
- **Group E (Tasks 7-8):** Disruption & Collusion — need extension methods first
- **Group F (Task 9):** Industry transactions — factory + dispatch
- **Group G (Task 10):** Integration tests + commit

---

### Task 1: Wire ProcessEvolutionGenerator

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs`
- Modify: `crates/datasynth-cli/src/output_writer.rs`

**What it does:** Generates process evolution events (approval workflow changes, automation events, policy changes, control enhancements) over the configured date range.

**Step 1: Add snapshot and stats fields to orchestrator**

In `enhanced_orchestrator.rs`, add to `EnhancedGenerationResult` (after `project_accounting` field ~line 783):

```rust
/// Process evolution events (workflow changes, automation, policy shifts)
pub process_evolution: Vec<datasynth_generators::process_evolution_generator::ProcessEvolutionEvent>,
/// Organizational events (M&A, restructuring, leadership changes)
pub organizational_events: Vec<datasynth_generators::organizational_event_generator::OrganizationalEvent>,
```

In `EnhancedGenerationStatistics`, add:

```rust
#[serde(default)]
pub process_evolution_event_count: usize,
#[serde(default)]
pub organizational_event_count: usize,
```

In `PhaseConfig`, add:

```rust
/// Generate process evolution and organizational events
pub generate_evolution_events: bool,
```

Default it to `true` in the `Default` impl.

**Step 2: Add phase method**

After `phase_project_accounting()`, add a new phase method:

```rust
/// Phase 24: Generate Process Evolution and Organizational Events.
fn phase_evolution_events(
    &self,
    stats: &mut EnhancedGenerationStatistics,
) -> SynthResult<(
    Vec<datasynth_generators::process_evolution_generator::ProcessEvolutionEvent>,
    Vec<datasynth_generators::organizational_event_generator::OrganizationalEvent>,
)> {
    if !self.phase_config.generate_evolution_events {
        debug!("Phase 24: Skipped (evolution events disabled)");
        return Ok((Vec::new(), Vec::new()));
    }
    info!("Phase 24: Generating Process Evolution & Organizational Events");

    let seed = self.seed;
    let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
        .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
    let end_date = start_date + chrono::Months::new(self.config.global.period_months);

    // Process evolution events
    let mut proc_gen = datasynth_generators::process_evolution_generator::ProcessEvolutionGenerator::new(seed + 100);
    let proc_events = proc_gen.generate_events(start_date, end_date);
    stats.process_evolution_event_count = proc_events.len();

    // Organizational events
    let company_codes: Vec<String> = self.config.companies.iter().map(|c| c.code.clone()).collect();
    let mut org_gen = datasynth_generators::organizational_event_generator::OrganizationalEventGenerator::new(seed + 101);
    let org_events = org_gen.generate_events(start_date, end_date, &company_codes);
    stats.organizational_event_count = org_events.len();

    info!(
        "Evolution events generated: {} process evolution, {} organizational",
        proc_events.len(), org_events.len()
    );
    self.check_resources_with_log("post-evolution-events")?;

    Ok((proc_events, org_events))
}
```

**Step 3: Call phase in generate() and wire into result**

In `generate()`, after the Phase 23 call (~line 1690), add:

```rust
// Phase 24: Process Evolution & Organizational Events
let (process_evolution, organizational_events) = self.phase_evolution_events(&mut stats)?;
```

Wire into `EnhancedGenerationResult` construction:

```rust
process_evolution,
organizational_events,
```

**Step 4: Add output export**

In `output_writer.rs`, before the statistics section (~line 1083), add:

```rust
// ========================================================================
// Process Evolution & Organizational Events
// ========================================================================
if !result.process_evolution.is_empty() || !result.organizational_events.is_empty() {
    let events_dir = output_dir.join("events");
    std::fs::create_dir_all(&events_dir)?;
    info!("Writing process evolution and organizational events...");

    write_json_safe(
        &result.process_evolution,
        &events_dir.join("process_evolution_events.json"),
        "Process evolution events",
    );
    write_json_safe(
        &result.organizational_events,
        &events_dir.join("organizational_events.json"),
        "Organizational events",
    );
}
```

**Step 5: Verify**

```bash
cargo check -j2
cargo test -j2 -p datasynth-runtime --lib
```

**Step 6: Commit**

```bash
git add -A && git commit -m "feat(runtime): wire ProcessEvolution + OrganizationalEvent generators (Phase 24)"
```

---

### Task 2: Wire CounterfactualGenerator

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs`
- Modify: `crates/datasynth-cli/src/output_writer.rs`

**What it does:** Takes generated JEs and produces counterfactual pairs — (original, mutated) entries with anomaly labels — for ML training data.

**Step 1: Add fields**

In `EnhancedGenerationResult`:

```rust
/// Counterfactual JE pairs for ML training (original + anomalous mutation)
pub counterfactual_pairs: Vec<datasynth_generators::counterfactual::CounterfactualPair>,
```

In `EnhancedGenerationStatistics`:

```rust
#[serde(default)]
pub counterfactual_pair_count: usize,
```

In `PhaseConfig`:

```rust
/// Generate counterfactual JE pairs for ML training
pub generate_counterfactuals: bool,
```

Default to `false` (opt-in, since it's ML-specific and doubles JE output size).

**Step 2: Add phase method**

```rust
/// Phase 25: Generate Counterfactual JE Pairs for ML training.
fn phase_counterfactuals(
    &self,
    journal_entries: &[JournalEntry],
    stats: &mut EnhancedGenerationStatistics,
) -> SynthResult<Vec<datasynth_generators::counterfactual::CounterfactualPair>> {
    if !self.phase_config.generate_counterfactuals || journal_entries.is_empty() {
        debug!("Phase 25: Skipped (counterfactuals disabled or no JEs)");
        return Ok(Vec::new());
    }
    info!("Phase 25: Generating Counterfactual JE Pairs");

    use datasynth_generators::counterfactual::{CounterfactualGenerator, CounterfactualSpec};

    let mut gen = CounterfactualGenerator::new(self.seed + 110);

    // Generate one counterfactual per JE using varied specs
    let specs = [
        CounterfactualSpec::ScaleAmount { factor: 10.0 },
        CounterfactualSpec::ShiftDate { days: -45 },
        CounterfactualSpec::SelfApprove,
        CounterfactualSpec::SplitTransaction { split_count: 3 },
    ];

    let pairs: Vec<_> = journal_entries
        .iter()
        .enumerate()
        .map(|(i, je)| gen.generate(je, &specs[i % specs.len()]))
        .collect();

    stats.counterfactual_pair_count = pairs.len();
    info!("Counterfactual pairs generated: {}", pairs.len());
    self.check_resources_with_log("post-counterfactuals")?;

    Ok(pairs)
}
```

**Step 3: Call in generate() after Phase 8 (anomaly injection), since we want clean JEs**

Actually — call it after JE generation but before anomaly injection, so counterfactuals use clean originals. Place after Phase 4 block (~line 1494):

```rust
// Phase 25: Counterfactual pairs (uses clean JEs before anomaly injection)
let counterfactual_pairs = self.phase_counterfactuals(&entries, &mut stats)?;
```

Wire into result:

```rust
counterfactual_pairs,
```

**Step 4: Add output export**

```rust
// ========================================================================
// Counterfactual Pairs (ML Training)
// ========================================================================
if !result.counterfactual_pairs.is_empty() {
    let ml_dir = output_dir.join("ml_training");
    std::fs::create_dir_all(&ml_dir)?;
    info!("Writing counterfactual pairs...");

    write_json_safe(
        &result.counterfactual_pairs,
        &ml_dir.join("counterfactual_pairs.json"),
        "Counterfactual pairs",
    );
}
```

**Step 5: Verify and commit**

```bash
cargo check -j2
cargo test -j2 -p datasynth-runtime --lib
git add -A && git commit -m "feat(runtime): wire CounterfactualGenerator for ML training pairs (Phase 25)"
```

---

### Task 3: Wire RedFlagGenerator

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs`
- Modify: `crates/datasynth-cli/src/output_writer.rs`

**What it does:** Injects fraud red-flag indicators onto document IDs (from P2P/O2C chains). Uses the anomaly labels to know which documents are fraudulent.

**Step 1: Add fields**

In `EnhancedGenerationResult`:

```rust
/// Fraud red flags on documents
pub red_flags: Vec<datasynth_generators::fraud::red_flags::RedFlag>,
```

In `EnhancedGenerationStatistics`:

```rust
#[serde(default)]
pub red_flag_count: usize,
```

No new `PhaseConfig` toggle — gate on `config.fraud.enabled`.

**Step 2: Add phase method**

```rust
/// Phase 26: Generate Fraud Red Flags on documents.
fn phase_red_flags(
    &self,
    anomaly_labels: &AnomalyLabels,
    document_flows: &DocumentFlowSnapshot,
    stats: &mut EnhancedGenerationStatistics,
) -> SynthResult<Vec<datasynth_generators::fraud::red_flags::RedFlag>> {
    if !self.config.fraud.enabled {
        debug!("Phase 26: Skipped (fraud disabled)");
        return Ok(Vec::new());
    }
    info!("Phase 26: Generating Fraud Red Flags");

    use datasynth_generators::fraud::red_flags::RedFlagGenerator;

    let generator = RedFlagGenerator::new();
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(self.seed + 120);

    // Collect document IDs from P2P and O2C chains
    let fraud_doc_ids: std::collections::HashSet<String> = anomaly_labels
        .labels
        .iter()
        .filter(|l| l.is_fraud)
        .map(|l| l.source_id.clone())
        .collect();

    let mut flags = Vec::new();

    // Apply to P2P purchase orders
    for chain in &document_flows.p2p_chains {
        let doc_id = &chain.purchase_order.header.document_id;
        let is_fraud = fraud_doc_ids.contains(doc_id);
        flags.extend(generator.inject_flags(doc_id, is_fraud, &mut rng));
    }

    // Apply to O2C sales orders
    for chain in &document_flows.o2c_chains {
        let doc_id = &chain.sales_order.header.document_id;
        let is_fraud = fraud_doc_ids.contains(doc_id);
        flags.extend(generator.inject_flags(doc_id, is_fraud, &mut rng));
    }

    stats.red_flag_count = flags.len();
    info!("Red flags generated: {}", flags.len());
    self.check_resources_with_log("post-red-flags")?;

    Ok(flags)
}
```

**Step 3: Call after anomaly injection (Phase 8) so we have fraud labels**

```rust
// Phase 26: Fraud red flags
let red_flags = self.phase_red_flags(&anomaly_labels, &document_flows, &mut stats)?;
```

Wire into result: `red_flags,`

**Step 4: Add output export**

```rust
if !result.red_flags.is_empty() {
    let labels_dir = output_dir.join("labels");
    std::fs::create_dir_all(&labels_dir)?;
    write_json_safe(
        &result.red_flags,
        &labels_dir.join("fraud_red_flags.json"),
        "Fraud red flags",
    );
}
```

**Step 5: Verify and commit**

```bash
cargo check -j2
cargo test -j2 -p datasynth-runtime --lib
git add -A && git commit -m "feat(runtime): wire RedFlagGenerator for fraud indicators (Phase 26)"
```

---

### Task 4: Wire TemporalAttributeGenerator

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs`
- Modify: `crates/datasynth-cli/src/output_writer.rs`

**What it does:** Adds bi-temporal versioning (valid_time + transaction_time) to vendors, producing version chains that model how vendor data changes over time.

**Step 1: Add fields**

In `EnhancedGenerationResult`:

```rust
/// Bi-temporal vendor version chains
pub temporal_vendor_chains: Vec<datasynth_generators::temporal::TemporalVersionChain<datasynth_core::models::Vendor>>,
```

In `EnhancedGenerationStatistics`:

```rust
#[serde(default)]
pub temporal_version_chain_count: usize,
```

Gate on existing config: `config.temporal_attributes.enabled` (the `TemporalAttributeSchemaConfig` already exists in schema.rs).

**Step 2: Add phase method**

```rust
/// Phase 27: Generate Temporal Attribute Version Chains.
fn phase_temporal_attributes(
    &self,
    stats: &mut EnhancedGenerationStatistics,
) -> SynthResult<Vec<datasynth_generators::temporal::TemporalVersionChain<datasynth_core::models::Vendor>>> {
    if !self.config.temporal_attributes.enabled {
        debug!("Phase 27: Skipped (temporal attributes disabled)");
        return Ok(Vec::new());
    }
    info!("Phase 27: Generating Temporal Attribute Version Chains");

    use datasynth_generators::temporal::TemporalAttributeGenerator;

    let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
        .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;

    let mut gen = TemporalAttributeGenerator::with_defaults(self.seed + 130, start_date);

    let chains: Vec<_> = self
        .master_data
        .vendors
        .iter()
        .map(|v| {
            let id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, v.vendor_id.as_bytes());
            gen.generate_version_chain(v.clone(), id)
        })
        .collect();

    stats.temporal_version_chain_count = chains.len();
    info!("Temporal version chains generated: {}", chains.len());
    self.check_resources_with_log("post-temporal-attributes")?;

    Ok(chains)
}
```

**Important:** This phase must run BEFORE `std::mem::take(&mut self.master_data)` in the result construction. It currently does since all phases run before the result is built.

**Step 3: Call in generate() and wire**

Place after Phase 23:

```rust
// Phase 27: Temporal attribute version chains
let temporal_vendor_chains = self.phase_temporal_attributes(&mut stats)?;
```

Wire into result: `temporal_vendor_chains,`

**Step 4: Add output export**

```rust
if !result.temporal_vendor_chains.is_empty() {
    let temporal_dir = output_dir.join("temporal");
    std::fs::create_dir_all(&temporal_dir)?;
    write_json_safe(
        &result.temporal_vendor_chains,
        &temporal_dir.join("vendor_version_chains.json"),
        "Vendor version chains",
    );
}
```

**Step 5: Verify and commit**

```bash
cargo check -j2
cargo test -j2 -p datasynth-runtime --lib
git add -A && git commit -m "feat(runtime): wire TemporalAttributeGenerator for bi-temporal versioning (Phase 27)"
```

---

### Task 5: Wire EntityGraphGenerator

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs`
- Modify: `crates/datasynth-cli/src/output_writer.rs`

**What it does:** Builds a relationship graph from master data entities and transaction summaries, with relationship strength scores.

**Step 1: Add fields**

In `EnhancedGenerationResult`:

```rust
/// Entity relationship graph with strength scores
pub entity_relationship_graph: Option<datasynth_core::models::EntityGraph>,
/// Cross-process links (P2P ↔ O2C via inventory)
pub cross_process_links: Vec<datasynth_core::models::CrossProcessLink>,
```

In `EnhancedGenerationStatistics`:

```rust
#[serde(default)]
pub entity_relationship_node_count: usize,
#[serde(default)]
pub entity_relationship_edge_count: usize,
#[serde(default)]
pub cross_process_link_count: usize,
```

Gate on `config.relationship_strength.enabled` and `config.cross_process_links.enabled`.

**Step 2: Add phase method**

```rust
/// Phase 28: Generate Entity Relationship Graph and Cross-Process Links.
fn phase_entity_relationships(
    &self,
    journal_entries: &[JournalEntry],
    document_flows: &DocumentFlowSnapshot,
    stats: &mut EnhancedGenerationStatistics,
) -> SynthResult<(Option<datasynth_core::models::EntityGraph>, Vec<datasynth_core::models::CrossProcessLink>)> {
    use datasynth_generators::relationships::entity_graph_generator::{
        EntityGraphGenerator, EntitySummary, TransactionSummary,
    };
    use datasynth_core::models::GraphEntityType;

    let mut entity_graph = None;
    let mut cross_links = Vec::new();

    // Relationship graph
    if self.config.relationship_strength.enabled && !self.master_data.vendors.is_empty() {
        info!("Phase 28: Generating Entity Relationship Graph");

        let start_date = NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
        let end_date = start_date + chrono::Months::new(self.config.global.period_months);
        let company_code = self.config.companies.first()
            .map(|c| c.code.as_str()).unwrap_or("1000");

        let vendor_summaries: Vec<EntitySummary> = self.master_data.vendors.iter().map(|v| {
            EntitySummary {
                entity_id: v.vendor_id.clone(),
                name: v.name.clone(),
                first_activity_date: start_date,
                entity_type: GraphEntityType::Vendor,
            }
        }).collect();

        let customer_summaries: Vec<EntitySummary> = self.master_data.customers.iter().map(|c| {
            EntitySummary {
                entity_id: c.customer_id.clone(),
                name: c.name.clone(),
                first_activity_date: start_date,
                entity_type: GraphEntityType::Customer,
            }
        }).collect();

        // Build transaction summaries from JEs
        let mut txn_summaries = std::collections::HashMap::new();
        for je in journal_entries {
            if je.lines.len() >= 2 {
                let key = (
                    je.header.company_code.clone(),
                    je.lines[0].gl_account.clone(),
                );
                let entry = txn_summaries.entry(key).or_insert_with(|| TransactionSummary {
                    total_volume: rust_decimal::Decimal::ZERO,
                    transaction_count: 0,
                    first_transaction_date: start_date,
                    last_transaction_date: end_date,
                });
                entry.transaction_count += 1;
                entry.total_volume += je.lines[0].debit_amount;
            }
        }

        let mut gen = EntityGraphGenerator::new(self.seed + 140);
        let graph = gen.generate_entity_graph(
            company_code,
            end_date,
            &vendor_summaries,
            &customer_summaries,
            &txn_summaries,
        );

        stats.entity_relationship_node_count = graph.nodes.len();
        stats.entity_relationship_edge_count = graph.edges.len();
        info!(
            "Entity graph generated: {} nodes, {} edges",
            graph.nodes.len(), graph.edges.len()
        );

        entity_graph = Some(graph);
    }

    // Cross-process links
    if self.config.cross_process_links.enabled {
        use datasynth_generators::relationships::entity_graph_generator::{
            GoodsReceiptRef, DeliveryRef,
        };

        let gr_refs: Vec<GoodsReceiptRef> = document_flows.p2p_chains.iter().map(|chain| {
            GoodsReceiptRef {
                document_id: chain.goods_receipt.as_ref()
                    .map(|gr| gr.document_id.clone())
                    .unwrap_or_default(),
                material_id: chain.purchase_order.items.first()
                    .map(|i| i.material_id.clone())
                    .unwrap_or_default(),
                quantity: chain.goods_receipt.as_ref()
                    .map(|gr| gr.total_value)
                    .unwrap_or_default(),
                date: chain.goods_receipt.as_ref()
                    .map(|gr| gr.posting_date)
                    .unwrap_or_default(),
            }
        }).collect();

        let del_refs: Vec<DeliveryRef> = document_flows.o2c_chains.iter().map(|chain| {
            DeliveryRef {
                document_id: chain.delivery.as_ref()
                    .map(|d| d.delivery_id.clone())
                    .unwrap_or_default(),
                material_id: chain.sales_order.items.first()
                    .map(|i| i.material_id.clone())
                    .unwrap_or_default(),
                quantity: chain.delivery.as_ref()
                    .map(|d| d.total_weight.unwrap_or_default())
                    .unwrap_or_default(),
                date: chain.delivery.as_ref()
                    .map(|d| d.actual_delivery_date.unwrap_or(d.planned_delivery_date))
                    .unwrap_or_default(),
            }
        }).collect();

        let mut gen = EntityGraphGenerator::new(self.seed + 141);
        cross_links = gen.generate_cross_process_links(&gr_refs, &del_refs);
        stats.cross_process_link_count = cross_links.len();
        info!("Cross-process links generated: {}", cross_links.len());
    }

    self.check_resources_with_log("post-entity-relationships")?;
    Ok((entity_graph, cross_links))
}
```

**Note:** The exact field names on `GoodsReceiptRef`, `DeliveryRef`, P2P chain, and O2C chain structs will need adjustment during implementation based on actual struct definitions. The implementer should read those structs first.

**Step 3: Call and wire**

```rust
// Phase 28: Entity relationship graph + cross-process links
let (entity_relationship_graph, cross_process_links) =
    self.phase_entity_relationships(&entries, &document_flows, &mut stats)?;
```

**Step 4: Output export**

```rust
if let Some(ref graph) = result.entity_relationship_graph {
    let rel_dir = output_dir.join("relationships");
    std::fs::create_dir_all(&rel_dir)?;
    match serde_json::to_string_pretty(graph) {
        Ok(json) => {
            if let Err(e) = std::fs::write(rel_dir.join("entity_relationship_graph.json"), json) {
                warn!("Failed to write entity graph: {}", e);
            } else {
                info!("  Entity relationship graph written");
            }
        }
        Err(e) => warn!("Failed to serialize entity graph: {}", e),
    }
}
if !result.cross_process_links.is_empty() {
    let rel_dir = output_dir.join("relationships");
    std::fs::create_dir_all(&rel_dir)?;
    write_json_safe(
        &result.cross_process_links,
        &rel_dir.join("cross_process_links.json"),
        "Cross-process links",
    );
}
```

**Step 5: Verify and commit**

```bash
cargo check -j2
cargo test -j2 -p datasynth-runtime --lib
git add -A && git commit -m "feat(runtime): wire EntityGraphGenerator for relationship graphs (Phase 28)"
```

---

### Task 6: Extend and Wire DisruptionManager

**Files:**
- Modify: `crates/datasynth-generators/src/disruption/mod.rs`
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs`
- Modify: `crates/datasynth-cli/src/output_writer.rs`

**What it does:** The DisruptionManager currently has an event-management API but no bulk generation method. Add `generate_disruptions()` that creates realistic disruption events over a date range, then wire it.

**Step 1: Add bulk generate method to DisruptionManager**

In `crates/datasynth-generators/src/disruption/mod.rs`, add after the existing impl block:

```rust
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Generates a set of realistic disruption events over a date range.
pub struct DisruptionGenerator {
    rng: ChaCha8Rng,
}

impl DisruptionGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Generate 1-3 disruption events per year across the date range.
    pub fn generate(
        &mut self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        company_codes: &[String],
    ) -> Vec<DisruptionEvent> {
        use rand::Rng;

        let months = ((end_date - start_date).num_days() as f64 / 30.0).ceil() as u32;
        let events_target = (months as f64 / 12.0 * 2.0).ceil() as usize; // ~2 per year

        let mut events = Vec::new();
        let disruption_types = [
            "SystemOutage", "SystemMigration", "ProcessChange",
            "DataRecovery", "RegulatoryChange",
        ];

        for i in 0..events_target {
            let offset_days = self.rng.random_range(0..=(end_date - start_date).num_days() as u32);
            let event_date = start_date + chrono::Duration::days(offset_days as i64);
            let severity: u8 = self.rng.random_range(1..=5);
            let dtype_idx = i % disruption_types.len();

            let affected = if company_codes.len() > 1 {
                let count = self.rng.random_range(1..=company_codes.len().min(3));
                company_codes[..count].to_vec()
            } else {
                company_codes.to_vec()
            };

            let mut labels = HashMap::new();
            labels.insert("disruption_type".to_string(), disruption_types[dtype_idx].to_string());
            labels.insert("severity".to_string(), severity.to_string());

            events.push(DisruptionEvent {
                event_id: format!("DISRUPT-{:04}", i + 1),
                disruption_type: match dtype_idx {
                    0 => DisruptionType::SystemOutage(OutageConfig {
                        start_date: event_date,
                        end_date: event_date + chrono::Duration::days(self.rng.random_range(1..=5)),
                        affected_systems: vec!["ERP".to_string()],
                        data_loss: severity >= 4,
                        recovery_mode: Some(RecoveryMode::Backfill),
                        cause: OutageCause::UnplannedFailure,
                    }),
                    1 => DisruptionType::SystemMigration(MigrationConfig {
                        cutover_date: event_date,
                        source_system: "Legacy ERP".to_string(),
                        target_system: "SAP S/4".to_string(),
                        dual_run_start: event_date - chrono::Duration::days(30),
                        dual_run_end: event_date + chrono::Duration::days(30),
                        format_changes: vec![],
                    }),
                    // ... other types follow similar pattern
                    _ => DisruptionType::ProcessChange(ProcessChangeConfig {
                        effective_date: event_date,
                        change_type: ProcessChangeType::WorkflowChange,
                        description: format!("Process change event {}", i + 1),
                        affected_accounts: vec![],
                        before_rules: HashMap::new(),
                        after_rules: HashMap::new(),
                    }),
                },
                description: format!("{} event on {}", disruption_types[dtype_idx], event_date),
                severity,
                affected_companies: affected,
                labels,
            });
        }

        events.sort_by_key(|e| match &e.disruption_type {
            DisruptionType::SystemOutage(c) => c.start_date,
            DisruptionType::SystemMigration(c) => c.cutover_date,
            DisruptionType::ProcessChange(c) => c.effective_date,
            DisruptionType::DataRecovery(c) => c.recovery_start,
            DisruptionType::RegulatoryChange(c) => c.effective_date,
        });

        events
    }
}
```

**Note:** The implementer must check the exact fields of `MigrationConfig`, `ProcessChangeConfig`, `RecoveryConfig`, `RegulatoryConfig` in the disruption module and adjust the construction accordingly.

**Step 2: Wire into orchestrator**

Add to `EnhancedGenerationResult`:

```rust
pub disruption_events: Vec<datasynth_generators::disruption::DisruptionEvent>,
```

Stats:

```rust
#[serde(default)]
pub disruption_event_count: usize,
```

Phase method (simple — just calls the new generator), call in generate(), output export.

**Step 3: Verify and commit**

```bash
cargo check -j2
cargo test -j2 -p datasynth-generators -p datasynth-runtime
git add -A && git commit -m "feat(generators+runtime): add DisruptionGenerator and wire into orchestrator"
```

---

### Task 7: Extend and Wire CollusionNetwork

**Files:**
- Modify: `crates/datasynth-generators/src/fraud/collusion/network.rs` (or add `generator.rs`)
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs`
- Modify: `crates/datasynth-cli/src/output_writer.rs`

**What it does:** The CollusionRing struct models fraud rings. Add a `CollusionRingGenerator` that creates realistic collusion scenarios from employee/vendor master data.

**Step 1: Add CollusionRingGenerator**

Create a new `generate_rings()` function (or struct) that:
- Takes employee IDs, vendor IDs, seed, date range
- Generates 1-3 rings (configurable)
- Assigns members from the employee/vendor pool
- Simulates a few months of activity
- Returns `Vec<CollusionRing>`

The implementer should read the existing `CollusionRing::new()`, `add_member()`, `advance_month()` API and compose them.

**Step 2: Wire into orchestrator**

Add to `EnhancedGenerationResult`:

```rust
pub collusion_rings: Vec<datasynth_generators::fraud::collusion::CollusionRing>,
```

Stats:

```rust
#[serde(default)]
pub collusion_ring_count: usize,
```

Gate on `config.fraud.enabled && config.fraud.clustering_enabled`.

Phase method creates rings from master data employees and vendors, advances them through months.

**Step 3: Output export**

```rust
write_json_safe(&result.collusion_rings, &labels_dir.join("collusion_rings.json"), "Collusion rings");
```

**Step 4: Verify and commit**

```bash
cargo check -j2
cargo test -j2 -p datasynth-generators -p datasynth-runtime
git add -A && git commit -m "feat(generators+runtime): add CollusionRingGenerator and wire into orchestrator"
```

---

### Task 8: Wire IndustryTransactionGenerator Factory

**Files:**
- Create: `crates/datasynth-generators/src/industry/factory.rs`
- Modify: `crates/datasynth-generators/src/industry/mod.rs`
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs`
- Modify: `crates/datasynth-cli/src/output_writer.rs`

**What it does:** The industry module has concrete generator structs (Retail, Manufacturing, Healthcare) with `gl_accounts()` but no `generate_transactions()`. Build a factory that dispatches based on `config.global.industry` and returns industry-specific GL accounts as the initial deliverable. Transaction generation can be added incrementally later.

**Step 1: Create factory**

In `crates/datasynth-generators/src/industry/factory.rs`:

```rust
//! Industry-specific generator factory.
//!
//! Dispatches to the correct concrete generator based on industry preset.

use super::common::IndustryGlAccount;

/// Industry generation output.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct IndustryOutput {
    /// Industry-specific GL accounts
    pub gl_accounts: Vec<IndustryGlAccount>,
    /// Industry identifier
    pub industry: String,
}

/// Generate industry-specific data based on the configured industry preset.
pub fn generate_industry_output(industry: &str) -> IndustryOutput {
    let gl_accounts = match industry.to_lowercase().as_str() {
        "retail" => super::retail::transactions::RetailTransactionGenerator::gl_accounts(),
        "manufacturing" => super::manufacturing::transactions::ManufacturingTransactionGenerator::gl_accounts(),
        "healthcare" => super::healthcare::transactions::HealthcareTransactionGenerator::gl_accounts(),
        _ => Vec::new(), // Unknown industry — no extra accounts
    };

    IndustryOutput {
        gl_accounts,
        industry: industry.to_string(),
    }
}
```

**Step 2: Export from mod.rs**

In `crates/datasynth-generators/src/industry/mod.rs`, add:

```rust
pub mod factory;
```

**Step 3: Wire into orchestrator**

Add to `EnhancedGenerationResult`:

```rust
pub industry_output: Option<datasynth_generators::industry::factory::IndustryOutput>,
```

Stats:

```rust
#[serde(default)]
pub industry_gl_account_count: usize,
```

Gate on `config.industry_specific.enabled`.

Simple phase method:

```rust
/// Phase 29: Industry-Specific Data.
fn phase_industry_data(
    &self,
    stats: &mut EnhancedGenerationStatistics,
) -> Option<datasynth_generators::industry::factory::IndustryOutput> {
    if !self.config.industry_specific.enabled {
        return None;
    }
    info!("Phase 29: Generating Industry-Specific Data");

    let output = datasynth_generators::industry::factory::generate_industry_output(
        &self.config.global.industry,
    );
    stats.industry_gl_account_count = output.gl_accounts.len();
    info!("Industry GL accounts: {} ({})", output.gl_accounts.len(), output.industry);
    Some(output)
}
```

**Step 4: Output export**

```rust
if let Some(ref industry) = result.industry_output {
    let ind_dir = output_dir.join("industry");
    std::fs::create_dir_all(&ind_dir)?;
    match serde_json::to_string_pretty(industry) {
        Ok(json) => {
            if let Err(e) = std::fs::write(ind_dir.join("industry_data.json"), json) {
                warn!("Failed to write industry data: {}", e);
            } else {
                info!("  Industry data written");
            }
        }
        Err(e) => warn!("Failed to serialize industry data: {}", e),
    }
}
```

**Step 5: Verify and commit**

```bash
cargo check -j2
cargo test -j2 -p datasynth-generators -p datasynth-runtime
git add -A && git commit -m "feat(generators+runtime): add industry transaction factory and wire into orchestrator (Phase 29)"
```

---

### Task 9: Integration Tests

**Files:**
- Create: `crates/datasynth-runtime/tests/unwired_generators_integration.rs`

**What it does:** Validates that all newly wired generators produce non-empty output when enabled and empty output when disabled.

**Step 1: Write integration test**

```rust
//! Integration tests for newly wired generators.

use datasynth_config::schema::SynthConfig;
use datasynth_runtime::enhanced_orchestrator::{EnhancedOrchestrator, PhaseConfig};

fn test_config() -> SynthConfig {
    let yaml = include_str!("../../../config/examples/small.yaml");
    serde_yaml::from_str(yaml).unwrap_or_else(|_| SynthConfig::default())
}

#[test]
fn test_evolution_events_generated() {
    let config = test_config();
    let phase = PhaseConfig {
        generate_evolution_events: true,
        ..Default::default()
    };
    let mut orch = EnhancedOrchestrator::new(config, phase).unwrap();
    let result = orch.generate().unwrap();
    assert!(!result.process_evolution.is_empty(), "Should generate process evolution events");
    assert!(!result.organizational_events.is_empty(), "Should generate organizational events");
}

#[test]
fn test_counterfactuals_generated_when_enabled() {
    let config = test_config();
    let phase = PhaseConfig {
        generate_counterfactuals: true,
        generate_journal_entries: true,
        ..Default::default()
    };
    let mut orch = EnhancedOrchestrator::new(config, phase).unwrap();
    let result = orch.generate().unwrap();
    // Counterfactuals only generated if JEs exist
    if !result.journal_entries.is_empty() {
        assert!(!result.counterfactual_pairs.is_empty(), "Should generate counterfactual pairs");
    }
}

#[test]
fn test_disabled_phases_produce_empty_output() {
    let config = test_config();
    let phase = PhaseConfig {
        generate_evolution_events: false,
        generate_counterfactuals: false,
        ..Default::default()
    };
    let mut orch = EnhancedOrchestrator::new(config, phase).unwrap();
    let result = orch.generate().unwrap();
    assert!(result.process_evolution.is_empty());
    assert!(result.organizational_events.is_empty());
    assert!(result.counterfactual_pairs.is_empty());
}
```

The implementer should add tests for each wired generator, checking both enabled and disabled paths.

**Step 2: Run tests**

```bash
cargo test -j2 -p datasynth-runtime --test unwired_generators_integration
```

**Step 3: Commit**

```bash
git add -A && git commit -m "test: add integration tests for newly wired generators"
```

---

### Task 10: Final Verification and Cleanup

**Step 1: Run full test suite**

```bash
cargo test -j2 --workspace
```

**Step 2: Run clippy and fmt**

```bash
cargo fmt
cargo clippy -j2 --workspace -- -D warnings
```

**Step 3: Update CHANGELOG.md**

Add entries for all newly wired generators under `## [1.0.0]` Added section.

**Step 4: Final commit**

```bash
git add -A && git commit -m "chore: fmt, clippy, changelog for wired generators"
```
