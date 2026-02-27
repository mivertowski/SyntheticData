# Codebase Quality Fixes v0.9.3 — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix all 29 remaining quality issues across 4 severity tiers to reach production-grade codebase for v0.9.3 release.

**Architecture:** Fix critical division-by-zero bugs and panics first (Tier 1), then production unwraps and determinism issues (Tier 2), then edge cases and dead code (Tier 3), and finally quality polish (Tier 4). Group by crate to minimize recompilation.

**Tech Stack:** Rust, `rust_decimal`, `uuid`, `tracing` for warnings

---

### Task 1: Tier 1 — Fix K-Anonymity division by zero

**Files:**
- Modify: `crates/datasynth-fingerprint/src/privacy/kanonymity.rs:20-48`

**Step 1: Add early return for total == 0**

```rust
pub fn filter_frequencies(
    &self,
    frequencies: Vec<(String, u64)>,
    total: u64,
) -> (Vec<(String, f64)>, usize) {
    if total == 0 {
        tracing::warn!("K-anonymity filter called with total=0, returning empty frequencies");
        return (Vec::new(), frequencies.len());
    }

    let threshold = self.k.max(self.min_occurrence) as u64;
    // ... rest unchanged
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-fingerprint -- kanonymity
```

---

### Task 2: Tier 1 — Fix Federated aggregation division by zero

**Files:**
- Modify: `crates/datasynth-fingerprint/src/federated/protocol.rs:271-285`

**Step 1: Add validation for total_record_count == 0 at the start of aggregate_weighted**

```rust
fn aggregate_weighted(
    &self,
    partials: &[PartialFingerprint],
    n_cols: usize,
    total_record_count: u64,
    total_epsilon: f64,
) -> Result<AggregatedFingerprint, String> {
    if total_record_count == 0 {
        return Err("Cannot aggregate fingerprints: total record count is zero".to_string());
    }
    let total_f = total_record_count as f64;
    // ... rest unchanged
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-fingerprint -- federated
```

---

### Task 3: Tier 1 — Fix graph edge remapping ghost edges

**Files:**
- Modify: `crates/datasynth-graph/src/exporters/pytorch_geometric.rs:165-172`
- Modify: `crates/datasynth-graph/src/exporters/dgl.rs:207-212`

**Step 1: Fix pytorch_geometric.rs — filter edges with missing nodes and log warning**

```rust
// Remap edge indices, skipping edges with missing node IDs
let mut sources_remapped: Vec<i64> = Vec::with_capacity(sources.len());
let mut targets_remapped: Vec<i64> = Vec::with_capacity(targets.len());
let mut skipped_edges = 0usize;

for (src, dst) in sources.iter().zip(targets.iter()) {
    match (id_to_idx.get(src), id_to_idx.get(dst)) {
        (Some(&s), Some(&d)) => {
            sources_remapped.push(s as i64);
            targets_remapped.push(d as i64);
        }
        _ => {
            skipped_edges += 1;
        }
    }
}
if skipped_edges > 0 {
    tracing::warn!(
        "PyTorch Geometric export: skipped {} edges with missing node IDs",
        skipped_edges
    );
}
```

**Step 2: Fix dgl.rs — same pattern for COO data**

```rust
let mut coo_data: Vec<Vec<i64>> = Vec::with_capacity(num_edges);
let mut skipped_edges = 0usize;

for i in 0..num_edges {
    match (id_to_idx.get(&sources[i]), id_to_idx.get(&targets[i])) {
        (Some(&s), Some(&d)) => {
            coo_data.push(vec![s as i64, d as i64]);
        }
        _ => {
            skipped_edges += 1;
        }
    }
}
if skipped_edges > 0 {
    tracing::warn!(
        "DGL export: skipped {} edges with missing node IDs",
        skipped_edges
    );
}
```

Also fix the type/edge type remapping at lines 300-303 and 329-332 — replace `.unwrap_or(&0)` with logging:

```rust
.map(|id| {
    let node = graph.nodes.get(id).expect("node ID from keys()");
    *type_to_idx.get(&node.node_type).unwrap_or_else(|| {
        tracing::warn!("Unknown node type '{}', defaulting to index 0", node.node_type);
        &0
    })
})
```

**Step 3: Run tests**

```bash
cargo test -p datasynth-graph -- pytorch && cargo test -p datasynth-graph -- dgl
```

---

### Task 4: Tier 1 — Fix GoBD document ID substring panic

**Files:**
- Modify: `crates/datasynth-output/src/formats/gobd.rs:54`

**Step 1: Replace unsafe substring with safe truncation**

Change line 54 from:
```rust
let beleg_nummer = escape_gobd_field(&je.header.document_id.to_string()[..8]);
```
To:
```rust
let doc_id_str = je.header.document_id.to_string();
let beleg_nummer = escape_gobd_field(
    &doc_id_str[..doc_id_str.len().min(8)]
);
```

Note: `document_id` is a UUID, so it's always long enough, but this defensive code prevents panics if the format changes.

**Step 2: Run tests**

```bash
cargo test -p datasynth-output -- gobd
```

---

### Task 5: Tier 2 — Fix Prometheus metrics unwraps

**Files:**
- Modify: `crates/datasynth-server/src/observability/otel.rs:65-72`

**Step 1: Replace unwraps with proper error handling**

```rust
#[cfg(feature = "otel")]
pub fn render_prometheus_metrics() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::default_registry().gather();
    let mut buf = Vec::new();
    if let Err(e) = encoder.encode(&metric_families, &mut buf) {
        tracing::error!("Failed to encode Prometheus metrics: {}", e);
        return String::from("# Error encoding metrics\n");
    }
    String::from_utf8(buf).unwrap_or_else(|e| {
        tracing::error!("Prometheus metrics buffer is not valid UTF-8: {}", e);
        String::from("# Error: invalid UTF-8 in metrics\n")
    })
}
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-server
```

---

### Task 6: Tier 2 — Fix rate limit header unwraps

**Files:**
- Modify: `crates/datasynth-server/src/rest/rate_limit_backend.rs:176,180`

**Step 1: Replace `.parse().unwrap()` with `HeaderValue::from()`**

```rust
headers.insert("X-RateLimit-Limit", HeaderValue::from(max_requests));
headers.insert("X-RateLimit-Remaining", HeaderValue::from(remaining));
```

`HeaderValue::from(u32)` is infallible — no unwrap needed.

**Step 2: Run tests**

```bash
cargo test -p datasynth-server -- rate_limit
```

---

### Task 7: Tier 2 — Fix non-deterministic household UUIDs

**Files:**
- Modify: `crates/datasynth-banking/src/generators/customer_generator.rs:1043`

**Step 1: Replace `Uuid::new_v4()` with deterministic UUID**

Find the context around line 1043 to determine which UUID factory is available. Use `self.uuid_factory.create()` or construct one from the deterministic RNG:

```rust
// Replace:
let household_id = Uuid::new_v4();

// With (use existing deterministic factory or rng):
let household_id = self.uuid_factory.create("household");
```

If `uuid_factory` isn't available at that scope, create from rng bytes:

```rust
let mut bytes = [0u8; 16];
self.rng.fill(&mut bytes);
let household_id = uuid::Builder::from_random_bytes(bytes).into_uuid();
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-banking -- customer
```

---

### Task 8: Tier 2 — Fix streaming orchestrator silent date fallbacks

**Files:**
- Modify: `crates/datasynth-runtime/src/streaming_orchestrator.rs:383,509,575`

**Step 1: Add `tracing::warn!` before fallback for all 3 instances**

Replace each `unwrap_or_else` with a version that logs:

```rust
let start_date = NaiveDate::parse_from_str(&config.global.start_date, "%Y-%m-%d")
    .unwrap_or_else(|e| {
        tracing::warn!(
            "Failed to parse start_date '{}': {}. Defaulting to 2024-01-01",
            config.global.start_date, e
        );
        NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date")
    });
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-runtime -- streaming
```

---

### Task 9: Tier 2 — Fix gate engine stubs (CorrelationPreservation + Custom metrics)

**Files:**
- Modify: `crates/datasynth-eval/src/gates/engine.rs:372-379,543-551`

**Step 1: Upgrade warnings to `error!` level and mark gate as failed-due-to-unavailable**

For CorrelationPreservation (line 372-379):
```rust
QualityMetric::CorrelationPreservation => {
    tracing::error!(
        "CorrelationPreservation gate '{}' cannot be evaluated — metric not implemented",
        gate.name
    );
    (
        None,
        "correlation preservation metric not implemented — gate cannot be evaluated".to_string(),
    )
}
```

For Custom metrics (line 543-551):
```rust
QualityMetric::Custom(name) => {
    tracing::error!(
        "Custom metric '{}' gate '{}' cannot be evaluated — custom metrics not implemented",
        name, gate.name
    );
    (
        None,
        format!(
            "custom metric '{}' not implemented — gate cannot be evaluated",
            name
        ),
    )
}
```

**Step 2: Update the None handler (line 287-298) to check if the gate should fail on missing metrics**

Add a `fail_on_unavailable` field to `QualityGate` (or use existing config) and respect it:

```rust
None => {
    let should_fail = gate.fail_on_unavailable.unwrap_or(false);
    if should_fail {
        tracing::warn!("Gate '{}' failed: metric not available ({})", gate.name, message);
    }
    GateCheckResult {
        gate_name: gate.name.clone(),
        metric: gate.metric.clone(),
        passed: !should_fail,
        actual_value: None,
        threshold: gate.threshold,
        comparison: gate.comparison.clone(),
        message: format!(
            "{}: metric not available ({}){}",
            gate.name, message,
            if should_fail { " — gate failed (fail_on_unavailable=true)" } else { "" }
        ),
    }
}
```

Check if `QualityGate` struct allows adding `fail_on_unavailable: Option<bool>`. If not possible without breaking changes, just upgrade the log levels.

**Step 3: Run tests**

```bash
cargo test -p datasynth-eval -- gate
```

---

### Task 10: Tier 2 — Fix fingerprint extraction silent error drops

**Files:**
- Modify: `crates/datasynth-fingerprint/src/extraction/mod.rs` (find the extractor caller code)

**Step 1: Add `tracing::warn!` before silent `Err(_) => None` patterns**

Find the code that calls the optional extractors and log the error before discarding:

```rust
Err(e) => {
    tracing::warn!("Optional {} extraction failed: {}", component_name, e);
    None
}
```

Apply to all optional component extraction calls (correlations, integrity, anomalies, rules).

**Step 2: Run tests**

```bash
cargo test -p datasynth-fingerprint -- extract
```

---

### Task 11: Tier 3 — Fix anomaly rounding magnitude calculation

**Files:**
- Modify: `crates/datasynth-generators/src/anomaly/strategies.rs:146-149`

**Step 1: Replace string-length-based magnitude with proper decimal math**

```rust
if self.prefer_round_numbers {
    let abs_amount = new_amount.abs();
    let magnitude = if abs_amount >= Decimal::ONE {
        // Count digits before decimal point using log10
        let digits = abs_amount.to_string()
            .split('.')
            .next()
            .map(|s| s.trim_start_matches('-').len())
            .unwrap_or(1);
        (digits as i32 - 1).max(0)
    } else {
        0
    };
    let round_factor = Decimal::new(10_i64.pow(magnitude as u32), 0);
    new_amount = (new_amount / round_factor).round() * round_factor;
}
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-generators -- anomaly
```

---

### Task 12: Tier 3 — Fix drift detection underflow on small datasets

**Files:**
- Modify: `crates/datasynth-eval/src/statistical/drift_detection.rs:403`

**Step 1: Add early return guard**

Before the loop, add:

```rust
if values.len() < self.window_size {
    tracing::debug!(
        "Drift detection: not enough values ({}) for window size ({}), returning empty",
        values.len(), self.window_size
    );
    return Vec::new();
}
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-eval -- drift
```

---

### Task 13: Tier 3 — Expose hardcoded P2P/O2C rates in config schema

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs:151,164,192`
- Modify: `crates/datasynth-config/src/schema.rs` (add fields to P2PFlowConfig and O2CFlowConfig)

**Step 1: Add optional config fields**

In the P2P/O2C flow config structs, add:

```rust
// In P2PFlowConfig:
#[serde(default)]
pub over_delivery_rate: Option<f64>,
#[serde(default)]
pub early_payment_discount_rate: Option<f64>,

// In O2CFlowConfig:
#[serde(default)]
pub late_payment_rate: Option<f64>,
```

**Step 2: Use config values in enhanced_orchestrator.rs**

```rust
over_delivery_rate: config.document_flows.p2p.over_delivery_rate.unwrap_or(0.02),
early_payment_discount_rate: config.document_flows.p2p.early_payment_discount_rate.unwrap_or(0.30),
late_payment_rate: config.document_flows.o2c.late_payment_rate.unwrap_or(0.15),
```

Remove the `// TODO` comments.

**Step 3: Run tests**

```bash
cargo test -p datasynth-config && cargo test -p datasynth-runtime -- orchestrator
```

---

### Task 14: Tier 3 — Fix rayon thread pool silent error

**Files:**
- Modify: `crates/datasynth-cli/src/main.rs:284`

**Step 1: Log the error instead of silently discarding**

```rust
if let Err(e) = rayon::ThreadPoolBuilder::new()
    .num_threads(effective_threads)
    .build_global()
{
    eprintln!("Warning: failed to configure thread pool with {} threads: {}", effective_threads, e);
}
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-cli
```

---

### Task 15: Tier 3 — Fix label export serialization errors

**Files:**
- Modify: `crates/datasynth-runtime/src/label_export.rs:80,93,120,122,127`

**Step 1: Replace `.unwrap_or_default()` with logging version**

Create a helper:

```rust
fn serialize_or_warn<T: serde::Serialize>(value: &T, field_name: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|e| {
        tracing::warn!("Failed to serialize {} for label export: {}", field_name, e);
        String::new()
    })
}
```

Then replace each `.unwrap_or_default()` call:
- Line 80: `related_entities: serialize_or_warn(&label.related_entities, "related_entities"),`
- Line 93: `structured_strategy_json: label.structured_strategy.as_ref().map(|s| serialize_or_warn(s, "structured_strategy")),`
- Line 120: `causal_reason_json: label.causal_reason.as_ref().map(|r| serialize_or_warn(r, "causal_reason")),`
- Line 122: `child_anomaly_ids: serialize_or_warn(&label.child_anomaly_ids, "child_anomaly_ids"),`
- Line 127: `metadata_json: serialize_or_warn(&label.metadata, "metadata"),`

**Step 2: Run tests**

```bash
cargo test -p datasynth-runtime -- label
```

---

### Task 16: Tier 3 — Fix distribution fitter NaN risk

**Files:**
- Modify: `crates/datasynth-fingerprint/src/synthesis/distribution_fitter.rs:40`

**Step 1: Add explicit guard for mean > 0**

```rust
if min > 0.0 && mean > 0.0 && std_dev >= 0.0 {
    let log_values_mean = mean.ln();
    let cv = std_dev / mean;
    let sigma = (1.0 + cv.powi(2)).ln().sqrt();
    // ... rest of log-normal fitting
} else if min > 0.0 {
    tracing::warn!("Distribution fitter: positive data but mean={}, std_dev={} — skipping log-normal", mean, std_dev);
}
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-fingerprint -- distribution
```

---

### Task 17: Tier 3 — Fix PCAOB series parse fallback

**Files:**
- Modify: `crates/datasynth-standards/src/audit/pcaob.rs:253`

**Step 1: Add warning on parse failure**

```rust
let num: u32 = self.number().parse().unwrap_or_else(|e| {
    tracing::warn!(
        "Failed to parse PCAOB standard number '{}': {}. Defaulting to 0",
        self.number(), e
    );
    0
});
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-standards -- pcaob
```

---

### Task 18: Tier 3 — Remove dead code structs and fields

**Files:**
- Modify: `crates/datasynth-graph/src/builders/transaction_graph.rs:351` (remove `AggregatedEdge`)
- Modify: `crates/datasynth-graph/src/builders/approval_graph.rs:227` (remove `ApprovalAggregation`)
- Modify: `crates/datasynth-graph/src/builders/banking_graph.rs:574` (remove `AggregatedBankingEdge`)
- Modify: `crates/datasynth-core/src/models/balance/balance_relationship.rs:304-305` (remove `account_groups`)
- Modify: `crates/datasynth-core/src/distributions/line_item.rs:155-156` (remove `line_config`)

**Step 1: Delete the dead structs and fields**

Remove each `#[allow(dead_code)]` annotated struct/field. For fields, also remove them from constructors.

**Step 2: Run tests**

```bash
cargo test -p datasynth-graph && cargo test -p datasynth-core
```

---

### Task 19: Tier 3 — Fix streaming orchestrator company fallback

**Files:**
- Modify: `crates/datasynth-runtime/src/streaming_orchestrator.rs:389,585`

**Step 1: Add warning when defaulting to "1000"**

```rust
let company_code = config
    .companies
    .first()
    .map(|c| c.code.as_str())
    .unwrap_or_else(|| {
        tracing::warn!("No companies configured, defaulting to company code '1000'");
        "1000"
    });
```

Apply to both instances at lines 389 and 585.

**Step 2: Extract default seed to constant**

At the top of the file, add:

```rust
/// Default RNG seed when not specified in config.
const DEFAULT_SEED: u64 = 42;
```

Then replace all 4 instances of `.unwrap_or(42)` at lines 354, 380, 485, 569 with `.unwrap_or(DEFAULT_SEED)`.

**Step 3: Run tests**

```bash
cargo test -p datasynth-runtime -- streaming
```

---

### Task 20: Tier 4 — Fix CLI env var for API key

**Files:**
- Modify: `crates/datasynth-cli/src/main.rs:398`

**Step 1: Replace unsafe `set_var` with deprecation warning**

Since the env var pattern is already in use, we can't simply remove it. Instead, add a warning:

```rust
if let Some(ref key) = stream_api_key {
    // Note: setting process-wide env var; prefer passing credentials via config
    #[allow(deprecated)]
    unsafe {
        std::env::set_var("RUSTGRAPH_API_KEY", key);
    }
    tracing::debug!("API key set from CLI argument");
}
```

Note: In Rust 1.88+, `set_var` may require `unsafe` depending on edition. Check and adjust accordingly. If it doesn't require unsafe yet, just add the log message.

**Step 2: Run tests**

```bash
cargo test -p datasynth-cli
```

---

### Task 21: Tier 4 — Fix unused _credit_amount in AR generator

**Files:**
- Modify: `crates/datasynth-generators/src/subledger/ar_generator.rs:185`

**Step 1: Remove the unused variable**

Delete the line:
```rust
let _credit_amount = (inv_line.net_amount * percent_of_invoice).round_dp(2);
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-generators -- ar_generator
```

---

### Task 22: Tier 4 — Fix JE generator COA empty fallback

**Files:**
- Modify: `crates/datasynth-generators/src/je_generator.rs:1736,1753`

**Step 1: Add warning before fallback**

```rust
all.choose(&mut self.rng)
    .copied()
    .unwrap_or_else(|| {
        tracing::warn!("Account selection returned empty list, falling back to first COA account");
        &self.coa.accounts[0]
    })
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-generators -- je_generator
```

---

### Task 23: Tier 4 — Fix NaN handling in mixture samplers

**Files:**
- Modify: `crates/datasynth-core/src/distributions/mixture.rs:320,436`

**Step 1: Add NaN check before binary search**

For both GaussianMixtureSampler (line 320) and LogNormalMixtureSampler (line 436), replace:

```rust
.binary_search_by(|w| w.partial_cmp(&p).unwrap_or(std::cmp::Ordering::Equal))
```

With:

```rust
.binary_search_by(|w| {
    w.partial_cmp(&p).unwrap_or_else(|| {
        tracing::debug!("NaN detected in mixture weight comparison (w={}, p={})", w, p);
        std::cmp::Ordering::Less
    })
})
```

Using `Less` instead of `Equal` for NaN ensures we fall through to the last component rather than selecting a potentially wrong one.

**Step 2: Run tests**

```bash
cargo test -p datasynth-core -- mixture
```

---

### Task 24: Tier 4 — Fix silent LLM parse error fallback

**Files:**
- Modify: `crates/datasynth-core/src/llm/nl_config.rs:70`

**Step 1: Log the error before fallback**

```rust
match llm_intent {
    Ok(llm) => Ok(Self::merge_intents(llm, keyword_intent)),
    Err(e) => {
        tracing::warn!("LLM-based config parsing failed, falling back to keyword parsing: {}", e);
        Ok(keyword_intent)
    }
}
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-core -- nl_config
```

---

### Task 25: Tier 4 — Fix test utility RwLock unwraps

**Files:**
- Modify: `crates/datasynth-test-utils/src/mocks.rs:25,33,49,60,69`

**Step 1: Replace `.unwrap()` with descriptive `.expect()`**

```rust
// Line 25:
self.balances.read().expect("MockBalanceTracker RwLock read should not be poisoned")
// Line 33:
let mut balances = self.balances.write().expect("MockBalanceTracker RwLock write should not be poisoned");
// Line 49:
self.balances.read().expect("MockBalanceTracker RwLock read should not be poisoned")
// Line 60:
self.balances.read().expect("MockBalanceTracker RwLock read should not be poisoned")
// Line 69:
self.balances.write().expect("MockBalanceTracker RwLock write should not be poisoned").clear();
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-test-utils
```

---

### Task 26: Final verification, version bump, commit

**Step 1: Run full test suite**

```bash
cargo test --workspace
```

**Step 2: Run clippy**

```bash
cargo clippy --workspace
```

**Step 3: Run fmt**

```bash
cargo fmt --all
```

**Step 4: Bump version to 0.9.3**

Update Cargo.toml workspace version, all changelogs, docs badge, LaTeX overview, Python pyproject.toml.

**Step 5: Commit all changes**

```bash
git add -A && git commit -m "fix: v0.9.3 codebase quality fixes — division-by-zero, ghost edges, unwraps, dead code"
```
