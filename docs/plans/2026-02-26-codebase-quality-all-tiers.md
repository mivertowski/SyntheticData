# Codebase Quality Fixes — All Tiers Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix all 61 identified issues across Tiers 1-6 to reach professional production quality.

**Architecture:** Fix bugs and security issues first (Tier 1), then build centralized account classification (Tier 3) that unblocks many downstream fixes, then address stubs/missing implementations (Tier 2), silent errors (Tier 4), performance (Tier 5), and quality concerns (Tier 6). Group changes by crate to minimize recompilation.

**Tech Stack:** Rust, `subtle` crate for constant-time comparison, `rand_distr::Beta` for proper Beta distribution

---

### Task 1: Tier 1 — Fix employee_generator last_mut() logic bug

**Files:**
- Modify: `crates/datasynth-generators/src/master_data/employee_generator.rs:404-418`

**Step 1: Fix the ordering — add employee BEFORE setting manager_id via last_mut()**

Move `pool.add_employee(cfo)` before the `last_mut()` call (same for COO):

```rust
// CFO
let cfo = self.generate_executive(company_code, "CFO", start_date);
let cfo_id = cfo.employee_id.clone();
pool.add_employee(cfo);
pool.employees
    .last_mut()
    .expect("just added CFO")
    .manager_id = Some(ceo_id.clone());

// COO
let coo = self.generate_executive(company_code, "COO", start_date);
let coo_id = coo.employee_id.clone();
pool.add_employee(coo);
pool.employees
    .last_mut()
    .expect("just added COO")
    .manager_id = Some(ceo_id.clone());
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-generators -- employee
```

---

### Task 2: Tier 1 — Fix banking customer_generator determinism (thread_rng)

**Files:**
- Modify: `crates/datasynth-banking/src/generators/customer_generator.rs:837,857`

**Step 1: Replace `rand::rng()` with `self.rng` and `rand::random()` with `self.rng.random()`**

Line 837: change `let mut thread_rng = rand::rng();` and `formats.choose(&mut thread_rng)` to `formats.choose(&mut self.rng)`.

Line 857: change `rand::random::<u8>() % 10` to `self.rng.random::<u8>() % 10`.

Remove the `let mut thread_rng = rand::rng();` line entirely.

**Step 2: Run tests**

```bash
cargo test -p datasynth-banking -- customer
```

---

### Task 3: Tier 1 — Fix gRPC auth to use constant-time comparison

**Files:**
- Modify: `crates/datasynth-server/src/grpc/auth_interceptor.rs:41`
- Modify: `crates/datasynth-server/Cargo.toml` (add `subtle = "2"`)

**Step 1: Add `subtle` dependency and use constant-time comparison**

In `auth_interceptor.rs`, add `use subtle::ConstantTimeEq;` and change the `validate_token` method:

```rust
pub fn validate_token(&self, token: &str) -> bool {
    if !self.enabled {
        return true;
    }
    let token_bytes = token.as_bytes();
    self.api_keys.iter().any(|k| {
        let key_bytes = k.as_bytes();
        // Constant-time comparison — prevent timing attacks
        key_bytes.len() == token_bytes.len()
            && key_bytes.ct_eq(token_bytes).into()
    })
}
```

Update the comment on `api_keys` field to remove the "plaintext for gRPC" note.

**Step 2: Run tests**

```bash
cargo test -p datasynth-server
```

---

### Task 4: Tier 1 — Fix CLI verify command count mismatch bug

**Files:**
- Modify: `crates/datasynth-cli/src/main.rs:1441-1448`

**Step 1: Change mismatch branch to increment `failed` and set `all_pass = false`**

```rust
} else {
    println!(
        "  [FAIL] {} count: expected {}, found {}",
        file_info.path, expected_count, line_count
    );
    failed += 1;
    all_pass = false;
}
```

Also fix the missing-file branch (~line 1449-1452) to increment `failed`:

```rust
} else {
    println!("  [FAIL] {} file missing for count check", file_info.path);
    failed += 1;
    all_pass = false;
}
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-cli
```

---

### Task 5: Tier 1 — Fix DGL heterogeneous graph node type in Python loader

**Files:**
- Modify: `crates/datasynth-graph/src/exporters/dgl.rs:665-667`

**Step 1: Use per-node type lookup instead of hardcoded `node_type_names[0]`**

In the Python loader string, replace the edge_dict construction to use the node_types array:

```python
    # Build edge dict for heterogeneous graph
    edge_dict = {{}}
    for etype_idx, etype_name in enumerate(edge_type_names):
        mask = edge_types == etype_idx
        if mask.any():
            src = edge_index[mask, 0]
            dst = edge_index[mask, 1]
            # Determine src/dst node types from the per-node type array
            if len(node_type_names) > 1:
                src_types = node_types[src]
                dst_types = node_types[dst]
                # Use the most common src/dst type for this edge type
                src_type = node_type_names[int(np.bincount(src_types).argmax())]
                dst_type = node_type_names[int(np.bincount(dst_types).argmax())]
            else:
                src_type = node_type_names[0] if node_type_names else 'node'
                dst_type = node_type_names[0] if node_type_names else 'node'
            edge_dict[(src_type, etype_name, dst_type)] = (src, dst)
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-graph -- dgl
```

---

### Task 6: Tier 3 — Centralized framework-aware account classification

**Files:**
- Modify: `crates/datasynth-core/src/framework_accounts.rs` (add classification methods + IFRS support)
- Modify: `crates/datasynth-core/src/models/balance/account_balance.rs:215-225`
- Modify: `crates/datasynth-core/src/models/balance/trial_balance.rs:387-406`

**Step 1: Add IFRS constructor and `classify_account_type` method to `FrameworkAccounts`**

In `framework_accounts.rs`, add:

1. An `ifrs()` constructor (initially same as `us_gaap()` — IFRS uses similar numbering conventions, the key difference is a proper explicit match rather than falling through to `_`)
2. Update `for_framework` to handle `"ifrs"`, `"us_gaap"`, `"dual_reporting"` explicitly
3. Add a `classify_account_type(&self, account_code: &str) -> AccountType` method that uses the framework's known account ranges
4. Add a `classify_trial_balance_category(&self, account_code: &str) -> TrialBalanceCategory` method

```rust
pub fn for_framework(framework: &str) -> Self {
    match framework {
        "us_gaap" | "UsGaap" => Self::us_gaap(),
        "ifrs" | "Ifrs" => Self::ifrs(),
        "french_gaap" | "FrenchGaap" => Self::french_gaap(),
        "german_gaap" | "GermanGaap" | "hgb" => Self::german_gaap(),
        "dual_reporting" | "DualReporting" => Self::us_gaap(), // primary framework
        other => {
            tracing::warn!("Unknown accounting framework '{}', defaulting to US GAAP", other);
            Self::us_gaap()
        }
    }
}
```

**Step 2: Update `AccountBalanceType::from_account_code` to accept an optional framework parameter**

Add a new `from_account_code_with_framework(code, framework)` method that dispatches based on framework, keeping the old method as a backward-compatible default.

**Step 3: Update `TrialBalanceCategory::from_account_code` similarly**

**Step 4: Run tests**

```bash
cargo test -p datasynth-core -- account_balance && cargo test -p datasynth-core -- trial_balance
```

---

### Task 7: Tier 3 — Update generators to use framework-aware classification

**Files:**
- Modify: `crates/datasynth-generators/src/balance/balance_tracker.rs:314-322`
- Modify: `crates/datasynth-generators/src/balance/trial_balance_generator.rs:472-484`
- Modify: `crates/datasynth-generators/src/fx/currency_translator.rs:108-122`
- Modify: `crates/datasynth-generators/src/intercompany/ic_generator.rs:394-429`

**Step 1: Update `balance_tracker.rs` to use FrameworkAccounts classification**

Replace the first-digit heuristic with a call to `FrameworkAccounts::classify_account_type()`. The `BalanceTracker` should accept a framework string on construction and store the appropriate `FrameworkAccounts` instance.

**Step 2: Update `trial_balance_generator.rs` similarly**

Replace the numeric-prefix match with framework-aware classification.

**Step 3: Update `currency_translator.rs` `is_monetary` function**

Add framework awareness — at minimum, add a `framework` parameter that allows dispatch. For the catch-all, log a warning instead of silently returning `true`.

**Step 4: Update `ic_generator.rs` to use FrameworkAccounts for IC account codes**

Replace hardcoded `"4100"`, `"5100"`, `"1310"`, `"2110"` etc. with lookups from FrameworkAccounts (add IC-specific account fields to the struct if needed, or use the existing revenue/expense/receivable/payable fields).

**Step 5: Run tests**

```bash
cargo test -p datasynth-generators -- balance && cargo test -p datasynth-generators -- trial_balance && cargo test -p datasynth-generators -- currency && cargo test -p datasynth-generators -- intercompany
```

---

### Task 8: Tier 3 — Update graph crate account classification

**Files:**
- Modify: `crates/datasynth-graph/src/builders/transaction_graph.rs:288-337`
- Modify: `crates/datasynth-graph/src/models/nodes.rs:265-268`

**Step 1: Update `infer_account_type` and related functions**

Add a `framework` parameter to these functions. For the graph module, since it may not have direct access to the config framework, accept an optional `&str` framework parameter defaulting to `"us_gaap"`.

**Step 2: Fix `nodes.rs` account code feature**

Parse the full account code as a numeric feature (not just the first character). Use a normalized representation — e.g., parse up to 4 characters and divide by 10000 to get a [0,1] range.

**Step 3: Run tests**

```bash
cargo test -p datasynth-graph
```

---

### Task 9: Tier 2 — Wire Neo4j and DGL graph exports in orchestrator

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs:6643-6650`

**Step 1: Replace warn-and-skip stubs with actual exporter calls**

Import `Neo4jExporter` and `DglExporter` from `datasynth-graph` and call them in the respective match arms, following the same pattern used for the PyTorch Geometric exporter arm above.

**Step 2: Run tests**

```bash
cargo test -p datasynth-runtime -- graph
```

---

### Task 10: Tier 2 — Fix streaming orchestrator no-op phases

**Files:**
- Modify: `crates/datasynth-runtime/src/streaming_orchestrator.rs:279-303`

**Step 1: Add meaningful implementations or demote to explicit warnings**

For phases that genuinely cannot stream (BalanceValidation, Complete), leave the debug log but add an explicit `info!` that these phases are not applicable in streaming mode.

For `AnomalyInjection` and `DataQuality`, add a `warn!` that these phases require post-processing and are skipped in streaming mode.

For `OcpmEvents`, add a `warn!` that OCPM is not yet supported in streaming.

**Step 2: Fix the 30-day month approximation (line 499)**

Replace `chrono::Duration::days((config.global.period_months as i64) * 30)` with the proper calculation:

```rust
use chrono::Months;
let end_date = start_date
    .checked_add_months(Months::new(config.global.period_months))
    .unwrap_or(start_date + chrono::Duration::days(365));
```

**Step 3: Fix the hardcoded department (lines 436-443)**

Read from `config.departments` if available, falling back to the hardcoded default with a warning:

```rust
let dept = if let Some(first_dept) = config.departments.first() {
    first_dept.clone()
} else {
    warn!("No departments configured, using default 'General' department");
    DepartmentDefinition { code: "1000".to_string(), ... }
};
```

**Step 4: Run tests**

```bash
cargo test -p datasynth-runtime -- streaming
```

---

### Task 11: Tier 2 — Fix server stubs (stream, reload_config, proto conversion)

**Files:**
- Modify: `crates/datasynth-server/src/rest/routes.rs:818-835` (start_stream)
- Modify: `crates/datasynth-server/src/rest/routes.rs:1052-1065` (reload_config)
- Modify: `crates/datasynth-server/src/grpc/service.rs:308-312,342-343` (proto fields)
- Modify: `crates/datasynth-server/src/config_loader.rs:39-46` (URL stub)

**Step 1: Use StreamRequest fields in start_stream**

Read `_req` fields and store them in the server state (add fields to `ServerState` if needed):

```rust
async fn start_stream(
    State(state): State<AppState>,
    Json(req): Json<StreamRequest>,
) -> Json<StreamResponse> {
    if let Some(eps) = req.events_per_second {
        state.server_state.events_per_second.store(eps, Ordering::Relaxed);
    }
    // ... rest
}
```

**Step 2: Fix reload_config to use config_loader**

```rust
async fn reload_config(State(state): State<AppState>) -> ... {
    let source = state.server_state.config_source.read().await;
    match crate::config_loader::load_config(&source).await {
        Ok(new_config) => { ... }
        Err(e) => { ... return error ... }
    }
}
```

**Step 3: Map proto fields from domain model**

```rust
vendor_id: line.vendor_id.clone(),
customer_id: line.customer_id.clone(),
material_id: line.material_id.clone(),
text: line.line_text.clone(),
// ...
generate_master_data: config.master_data.enabled,
generate_document_flows: config.document_flows.p2p.enabled || config.document_flows.o2c.enabled,
```

**Step 4: Remove ConfigSource::Url or implement with reqwest**

Since the server already depends on `reqwest` (or `hyper`), implement URL config loading:

```rust
ConfigSource::Url { url } => {
    let resp = reqwest::get(url).await.map_err(|e| ConfigLoadError::Io(e.to_string()))?;
    let content = resp.text().await.map_err(|e| ConfigLoadError::Io(e.to_string()))?;
    let config: GeneratorConfig = serde_yaml::from_str(&content)?;
    Ok(config)
}
```

**Step 5: Run tests**

```bash
cargo test -p datasynth-server
```

---

### Task 12: Tier 2 — Fix remaining stubs (CLI quality gate, fingerprint, counterfactual, approval graph)

**Files:**
- Modify: `crates/datasynth-cli/src/main.rs:1221-1227` (quality gate)
- Modify: `crates/datasynth-cli/src/main.rs:1539-1541` (--sign)
- Modify: `crates/datasynth-cli/src/main.rs:418-424` (fingerprint manifest)
- Modify: `crates/datasynth-generators/src/counterfactual/mod.rs:468-471`
- Modify: `crates/datasynth-graph/src/builders/approval_graph.rs:60-68`

**Step 1: Quality gate — log clear warning that evaluation is not yet wired**

```rust
warn!("Quality gate evaluation uses placeholder data — full integration pending");
```

**Step 2: Fingerprint --sign — log warning at warn level instead of info**

```rust
if sign {
    warn!("Fingerprint signing is not yet implemented; writing unsigned fingerprint");
}
```

**Step 3: Fingerprint manifest — extract actual config from orchestrator when possible**

If the orchestrator exposes its config, use it. Otherwise, add a clear `warn!`:

```rust
ConfigOrOrchestrator::Orchestrator(ref orch) => {
    warn!("Fingerprint-based generation: manifest uses approximate config metadata");
    create_safe_demo_preset()  // keep fallback but with warning
}
```

**Step 4: Counterfactual — implement AddLineItem and RemoveLineItem**

For `CounterfactualSpec::AddLineItem`, create the injection strategy that actually adds a line to the JE. For `RemoveLineItem`, create one that removes a line by index.

**Step 5: Approval graph — remove dead `include_hierarchy` config or document as future feature**

Add a `warn!` when `include_hierarchy` is true:

```rust
if self.config.include_hierarchy {
    warn!("include_hierarchy requires manager_id field on User model — not yet supported");
}
```

**Step 6: Run tests**

```bash
cargo test -p datasynth-cli && cargo test -p datasynth-generators -- counterfactual && cargo test -p datasynth-graph -- approval
```

---

### Task 13: Tier 2 — Clean up dead code and unimplemented traits

**Files:**
- Modify: `crates/datasynth-core/src/distributions/behavioral_drift.rs:78-85` (dead trait)
- Modify: `crates/datasynth-generators/src/industry/common.rs:111-131` (unimplemented trait)
- Modify: `crates/datasynth-generators/src/project_accounting/revenue_generator.rs:5-8` (dead_code allow)
- Modify: `crates/datasynth-eval/src/gates/engine.rs:372-378,541-547` (always-None gates)
- Modify: `crates/datasynth-fingerprint/src/federated/protocol.rs:159-178` (empty mins/maxs)
- Modify: `crates/datasynth-generators/src/standards/mod.rs:3` (stale comment)
- Modify: `crates/datasynth-output/src/streaming/parquet_sink.rs:190-212` (dead GenericParquetRecord)
- Modify: `crates/datasynth-config/src/validation.rs:560-569` (unused validate_positive)

**Step 1: Remove `BehavioralDrift` trait (dead code)**

Delete lines 78-85. Keep the concrete implementations on the structs.

**Step 2: Add `#[allow(unused)]` with doc comment on `IndustryTransactionGenerator` trait**

```rust
/// Industry-specific transaction generator trait.
///
/// Note: Not yet implemented by concrete generators. Retained as the
/// intended public API for industry-specific generation modules.
#[allow(unused)]
pub trait IndustryTransactionGenerator: Send + Sync { ... }
```

**Step 3: Remove `#![allow(dead_code)]` from revenue_generator.rs, add targeted allows**

Replace the blanket `#![allow(dead_code)]` with specific `#[allow(dead_code)]` on the struct and its impl, and update the comment.

**Step 4: Add warnings for always-None gates**

```rust
QualityMetric::CorrelationPreservation => {
    tracing::warn!("CorrelationPreservation gate metric is not yet available in ComprehensiveEvaluation");
    (None, "correlation preservation metric not available".to_string())
}
```

**Step 5: Add `mins`/`maxs`/`correlations` parameters to `create_partial`**

```rust
pub fn create_partial(
    source_id: &str, columns: Vec<String>, record_count: u64,
    means: Vec<f64>, stds: Vec<f64>,
    mins: Vec<f64>, maxs: Vec<f64>, correlations: Vec<f64>,
    epsilon: f64,
) -> PartialFingerprint { ... }
```

**Step 6: Remove stale comment from standards/mod.rs**

Change line 3 from `mod revenue_recognition_generator; // will be created by another agent` to just `mod revenue_recognition_generator;`.

**Step 7: Remove dead `GenericParquetRecord` from streaming parquet_sink.rs**

Delete lines 190-212.

**Step 8: Either use `validate_positive` or remove it**

Wire it into `validate_global_settings` for validating positive numeric fields, or remove if not needed.

**Step 9: Run tests**

```bash
cargo test --workspace 2>&1 | tail -5
```

---

### Task 14: Tier 4 — Fix silent date fallbacks in banking

**Files:**
- Modify: `crates/datasynth-banking/src/generators/transaction_generator.rs:33-34`
- Modify: `crates/datasynth-banking/src/generators/customer_generator.rs:34`
- Modify: `crates/datasynth-banking/src/typologies/injector.rs:189-194,252-257,313-318,368-373`

**Step 1: Replace silent fallbacks with explicit errors or warnings**

Create a helper function in the banking crate:

```rust
fn parse_start_date(date_str: &str) -> NaiveDate {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap_or_else(|e| {
        tracing::warn!(
            "Failed to parse start_date '{}': {}. Defaulting to 2024-01-01",
            date_str, e
        );
        NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date")
    })
}
```

Replace all 6 occurrences with calls to this helper.

**Step 2: Run tests**

```bash
cargo test -p datasynth-banking
```

---

### Task 15: Tier 4 — Fix CLI silent defaults and safety limits

**Files:**
- Modify: `crates/datasynth-cli/src/main.rs:1279-1293` (init defaults)
- Modify: `crates/datasynth-cli/src/main.rs:2014-2034` (safety limits)

**Step 1: Add warnings for unrecognized init values**

```rust
_ => {
    eprintln!("Warning: unrecognized industry '{}', defaulting to manufacturing", industry);
    IndustrySector::Manufacturing
}
// ... same for complexity
```

**Step 2: Add warnings when safety limits truncate values**

```rust
fn apply_safety_limits(config: &mut GeneratorConfig) {
    if config.global.period_months > 12 {
        tracing::warn!(
            "Safety limit: period_months capped from {} to 12 for demo mode",
            config.global.period_months
        );
        config.global.period_months = 12;
    }
    // ... similar for transaction volume and banking population
}
```

**Step 3: Run tests**

```bash
cargo test -p datasynth-cli
```

---

### Task 16: Tier 4 — Fix config validation gaps

**Files:**
- Modify: `crates/datasynth-config/src/validation.rs`

**Step 1: Add `start_date` format validation**

```rust
fn validate_global_settings(config: &GeneratorConfig) -> SynthResult<()> {
    // Existing period_months validation...

    // Validate start_date format
    if NaiveDate::parse_from_str(&config.global.start_date, "%Y-%m-%d").is_err() {
        return Err(SynthError::validation(format!(
            "Invalid start_date format '{}', expected YYYY-MM-DD",
            config.global.start_date
        )));
    }
    Ok(())
}
```

**Step 2: Add company.country and company.name validation**

```rust
if company.name.is_empty() {
    return Err(SynthError::validation(format!(
        "Company name cannot be empty for company '{}'", company.code
    )));
}
if company.country.len() != 2 {
    return Err(SynthError::validation(format!(
        "Invalid country code '{}' for company '{}', expected 2-letter ISO code",
        company.country, company.code
    )));
}
```

**Step 3: Run tests**

```bash
cargo test -p datasynth-config
```

---

### Task 17: Tier 4 — Fix GoBD tax amount and contra account

**Files:**
- Modify: `crates/datasynth-output/src/formats/gobd.rs:65-80`

**Step 1: Compute tax amount from line data when tax_code is present**

```rust
let steuer_betrag = if line.tax_code.is_some() {
    // Compute tax from the line's tax amount field if available
    line.tax_amount.map(|t| t.to_string()).unwrap_or_else(|| "0.00".to_string())
} else {
    "0.00".to_string()
};
```

**Step 2: Improve contra account for multi-line entries**

For multi-line entries, use the primary contra account (the first line on the opposite side):

```rust
let contra_for = |idx: usize| -> String {
    if je.lines.len() == 2 {
        let other = if idx == 0 { 1 } else { 0 };
        je.lines[other].gl_account.clone()
    } else {
        // For multi-line: find the first line on the opposite side
        let is_debit = je.lines[idx].is_debit();
        je.lines.iter()
            .find(|l| l.is_debit() != is_debit)
            .map(|l| l.gl_account.clone())
            .unwrap_or_default()
    }
};
```

**Step 3: Run tests**

```bash
cargo test -p datasynth-output -- gobd
```

---

### Task 18: Tier 4 — Fix remaining silent incorrect behaviors

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs:148,160,186` (hardcoded rates — add warn comments)
- Modify: `crates/datasynth-core/src/llm/nl_config.rs:169` (warn on unknown features)
- Modify: `crates/datasynth-core/src/templates/provider.rs:358,383` (warn on unknown industry)
- Modify: `crates/datasynth-generators/src/audit/evidence_generator.rs:470,473` (use config date)
- Modify: `crates/datasynth-server/src/grpc/service.rs:190-197` (return error for unknown enums)

**Step 1: Add warnings for hardcoded orchestrator rates**

The rates that bypass config should have clear `// TODO: wire to config schema` comments and `tracing::debug!` when used.

**Step 2: Warn on unknown NL config features**

```rust
_ => {
    tracing::warn!("Unknown feature '{}' in natural language config, ignoring", feature);
}
```

**Step 3: Warn on unknown template industry**

```rust
_ => {
    tracing::debug!("No vendor names template for category '{}', using manufacturing", category);
    Self::embedded_vendor_names_manufacturing()
}
```

**Step 4: Use config date in evidence_generator instead of hardcoded 2025-12-31**

Use the generator's period end date from config context.

**Step 5: Return gRPC error for unknown enum strings**

```rust
_ => {
    return Err(Status::invalid_argument(format!(
        "Unknown industry '{}'", req.industry
    )));
}
```

**Step 6: Run tests**

```bash
cargo test --workspace 2>&1 | tail -5
```

---

### Task 19: Tier 5 — Fix server performance issues

**Files:**
- Modify: `crates/datasynth-server/src/grpc/service.rs:575-600` (orchestrator per iteration)
- Modify: `crates/datasynth-server/src/rest/websocket.rs:175-205` (take(1) waste)

**Step 1: Move orchestrator creation outside the streaming loop**

```rust
let mut orchestrator = EnhancedOrchestrator::new(config.clone(), phase_config.clone())?;
loop {
    let result = orchestrator.generate()?;
    // ... stream results
}
```

**Step 2: Fix WebSocket to stream all entries, not just take(1)**

```rust
for entry in &result.journal_entries {
    sequence += 1;
    // ... emit each entry
}
```

**Step 3: Run tests**

```bash
cargo test -p datasynth-server
```

---

### Task 20: Tier 6 — Fix production unwrap/expect calls

**Files:**
- Modify: `crates/datasynth-core/src/templates/names.rs:1019,1023,1026` (name pool expects)
- Modify: `crates/datasynth-core/src/streaming/channel.rs:204,237,244` (mutex expects)
- Modify: `crates/datasynth-generators/src/document_flow/document_chain_manager.rs:227,230`
- Modify: `crates/datasynth-generators/src/je_generator.rs:1042`
- Modify: `crates/datasynth-runtime/src/run_manifest.rs:230,241`
- Modify: `crates/datasynth-fingerprint/src/certificates/certificate.rs:209`

**Step 1: Fix name pool — validate at construction, not at use**

Add validation in `NamePool::new()` or the builder that ensures lists are non-empty. Keep the `expect()` calls but with better messages.

**Step 2: Fix streaming channel — return SynthError instead of panicking**

Replace `lock().expect("mutex poisoned")` with `lock().map_err(|e| SynthError::generation(format!("mutex poisoned: {}", e)))?`.

**Step 3: Fix document_chain_manager — return Result or skip with warning**

```rust
let Some(vendors) = vendors_by_company.get(company_code) else {
    warn!("No vendor pool for company {}, skipping", company_code);
    continue;
};
```

**Step 4: Fix je_generator batch_state — use if-let or return error**

```rust
let Some(batch) = self.batch_state.clone() else {
    return Err(SynthError::generation("batch_state must be set before calling generate_batched_entry"));
};
```

**Step 5: Fix run_manifest — use `let now` pattern and propagate serialization error**

```rust
fn hash_config(config: &GeneratorConfig) -> String {
    let json = serde_json::to_string(config).unwrap_or_else(|e| {
        tracing::warn!("Failed to serialize config for hashing: {}", e);
        String::new()
    });
    // ...
}

pub fn complete(&mut self, statistics: EnhancedGenerationStatistics) {
    let now = Utc::now();
    self.completed_at = Some(now);
    self.duration_seconds = Some((now - self.started_at).num_milliseconds() as f64 / 1000.0);
}
```

**Step 6: Fix certificate signable_content — propagate error**

```rust
fn signable_content(certificate: &SyntheticDataCertificate) -> Result<String, serde_json::Error> {
    let mut cert_copy = certificate.clone();
    cert_copy.signature = None;
    serde_json::to_string(&cert_copy)
}
```

Update callers to handle the Result.

**Step 7: Run tests**

```bash
cargo test --workspace 2>&1 | tail -5
```

---

### Task 21: Tier 6 — Fix dead variable, Debug-format matching, header unwraps

**Files:**
- Modify: `crates/datasynth-core/src/causal/validation.rs:86,112` (dead _correct_signs)
- Modify: `crates/datasynth-graph/src/builders/banking_graph.rs:142-160,409-421` (Debug format matching)
- Modify: `crates/datasynth-server/src/rest/security_headers.rs:24-37` (parse().unwrap())
- Modify: `crates/datasynth-server/src/rest/request_id.rs:31`
- Modify: `crates/datasynth-server/src/rest/rate_limit.rs:186-190`

**Step 1: Remove `_correct_signs` or use it**

Either remove the dead variable entirely, or use it in the result to report the success ratio.

**Step 2: Replace Debug-format matching with proper enum matching**

```rust
// Instead of: match format!("{:?}", kyc.expected_monthly_turnover).as_str() { "Low" => ... }
// Use: match kyc.expected_monthly_turnover { MonthlyTurnover::Low => 2.0, ... }
```

**Step 3: Replace `.parse().unwrap()` with `HeaderValue::from_static()`**

```rust
headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
```

For dynamic values (request_id, rate limit numbers), use `HeaderValue::try_from()` with proper error handling.

**Step 4: Run tests**

```bash
cargo test -p datasynth-core -- causal && cargo test -p datasynth-graph -- banking && cargo test -p datasynth-server
```

---

### Task 22: Tier 6 — Extract shared NPY code in graph crate

**Files:**
- Create: `crates/datasynth-graph/src/exporters/npy_writer.rs`
- Modify: `crates/datasynth-graph/src/exporters/mod.rs`
- Modify: `crates/datasynth-graph/src/exporters/pytorch_geometric.rs`
- Modify: `crates/datasynth-graph/src/exporters/dgl.rs`

**Step 1: Create `npy_writer.rs` with shared functions**

Extract the duplicated `write_npy_header`, `write_npy_1d_f32`, `write_npy_1d_i64`, `write_npy_2d_i64`, `write_npy_1d_bool`, `export_masks` functions into a shared module.

**Step 2: Update both exporters to use the shared module**

Remove the duplicated functions and import from `npy_writer`.

**Step 3: Run tests**

```bash
cargo test -p datasynth-graph
```

---

### Task 23: Tier 6 — Fix remaining quality issues

**Files:**
- Modify: `crates/datasynth-core/src/causal/scm.rs:94-105` (Beta distribution)
- Modify: `crates/datasynth-core/src/traits/registry.rs:124-141` (empty version/description)
- Modify: `crates/datasynth-eval/src/banking/aml_detectability.rs:176-179` (trivial detection)
- Modify: `crates/datasynth-eval/src/statistical/anderson_darling.rs:416-423` (generic p-value)
- Modify: `crates/datasynth-eval/src/coherence/subledger.rs:196` (unknown = reconciled)

**Step 1: Use proper Beta distribution from rand_distr**

```rust
"beta" => {
    let alpha = var.params.get("alpha").copied().unwrap_or(2.0);
    let beta_param = var.params.get("beta_param").copied().unwrap_or(2.0);
    if let Ok(d) = rand_distr::Beta::new(alpha, beta_param) {
        d.sample(rng)
    } else {
        // Fallback for invalid parameters
        alpha / (alpha + beta_param)
    }
}
```

**Step 2: Add `version()` and `description()` to OutputSink trait or use defaults**

If the trait can't be extended (breaking change), use a sensible default like `"1.0.0"` and the sink's name.

**Step 3: Improve AML detectability for unknown typologies**

```rust
_ => {
    // Check for basic pattern: multiple suspicious transactions in a case
    let suspicious_count = txns.iter().filter(|t| t.is_suspicious).count();
    suspicious_count > 0 && suspicious_count as f64 / txns.len().max(1) as f64 > 0.3
}
```

**Step 4: Improve A-D generic p-value with conservative approach**

```rust
_ => {
    // Conservative: report as significant if A² > 1.0
    // (no valid statistical approximation available for this distribution)
    tracing::debug!("No A-D p-value table for distribution type, using conservative threshold");
    if a2 < 1.0 { 0.1 } else { 0.01 }
}
```

**Step 5: Fix subledger unknown type — return unreconciled with warning**

```rust
_ => {
    tracing::warn!("Unknown account type '{}', cannot verify reconciliation", account_type);
    (false, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO)
}
```

**Step 6: Run tests**

```bash
cargo test --workspace 2>&1 | tail -5
```

---

### Task 24: Final verification

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

**Step 4: Commit all changes**

```bash
git add -A && git commit -m "fix: comprehensive codebase quality fixes (Tiers 1-6)"
```
