//! End-to-end evaluation of the FSM engine output.
//!
//! Loads both FSA and IA blueprints, runs engagements, and prints summary
//! statistics, OCEL projections, shortest-path analysis, and Monte Carlo
//! results.
//!
//! Run with:
//!   cargo test -p datasynth-audit-fsm --test evaluate_output -- --nocapture --test-threads=4

use std::collections::HashSet;

use datasynth_audit_fsm::{
    context::EngagementContext,
    engine::AuditFsmEngine,
    export::{
        flat_log::{export_events_to_file, export_events_to_json},
        ocel::project_to_ocel,
    },
    loader::{default_overlay, BlueprintWithPreconditions},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_fsa_engine(seed: u64) -> AuditFsmEngine {
    let bwp =
        BlueprintWithPreconditions::load_builtin_fsa().expect("builtin FSA blueprint must load");
    let overlay = default_overlay();
    let rng = ChaCha8Rng::seed_from_u64(seed);
    AuditFsmEngine::new(bwp, overlay, rng)
}

fn build_ia_engine(seed: u64) -> AuditFsmEngine {
    let bwp =
        BlueprintWithPreconditions::load_builtin_ia().expect("builtin IA blueprint must load");
    let overlay = default_overlay();
    let rng = ChaCha8Rng::seed_from_u64(seed);
    AuditFsmEngine::new(bwp, overlay, rng)
}

// ---------------------------------------------------------------------------
// Test
// ---------------------------------------------------------------------------

#[test]
fn evaluate_fsm_engine_output() {
    let ctx = EngagementContext::test_default();

    // ===================================================================
    // 1. Load both blueprints and run engagements
    // ===================================================================
    println!("\n{}", "=".repeat(70));
    println!("  FSM Engine End-to-End Evaluation");
    println!("{}\n", "=".repeat(70));

    // -- FSA --
    let mut fsa_engine = build_fsa_engine(42);
    let fsa_result = fsa_engine
        .run_engagement(&ctx)
        .expect("FSA engagement must succeed");

    // -- IA --
    let mut ia_engine = build_ia_engine(42);
    let ia_result = ia_engine
        .run_engagement(&ctx)
        .expect("IA engagement must succeed");

    // ===================================================================
    // 2. Summary statistics — FSA
    // ===================================================================
    let fsa_events = &fsa_result.event_log;
    let fsa_procedures_executed = fsa_result.procedure_states.len();
    let fsa_phases_completed = fsa_result.phases_completed.len();
    let fsa_anomalies = fsa_result.anomalies.len();
    let fsa_duration = fsa_result.total_duration_hours;

    let fsa_self_loops = fsa_events
        .iter()
        .filter(|e| e.from_state.is_some() && e.from_state == e.to_state)
        .count();

    let fsa_unique_event_types: HashSet<&str> =
        fsa_events.iter().map(|e| e.event_type.as_str()).collect();

    println!("--- FSA Blueprint Summary ---");
    println!("  Events generated:      {}", fsa_events.len());
    println!("  Procedures executed:   {}", fsa_procedures_executed);
    println!("  Phases completed:      {}", fsa_phases_completed);
    println!("  Anomalies injected:    {}", fsa_anomalies);
    println!("  Duration (hours):      {:.1}", fsa_duration);
    println!("  Self-loop events:      {}", fsa_self_loops);
    println!("  Unique event types:    {}", fsa_unique_event_types.len());
    println!("  Event types:           {:?}", {
        let mut v: Vec<&str> = fsa_unique_event_types.iter().copied().collect();
        v.sort();
        v
    });
    println!();

    // ===================================================================
    // 2b. Summary statistics — IA
    // ===================================================================
    let ia_events = &ia_result.event_log;
    let ia_procedures_executed = ia_result.procedure_states.len();
    let ia_phases_completed = ia_result.phases_completed.len();
    let ia_anomalies = ia_result.anomalies.len();
    let ia_duration = ia_result.total_duration_hours;

    let ia_self_loops = ia_events
        .iter()
        .filter(|e| e.from_state.is_some() && e.from_state == e.to_state)
        .count();

    let ia_unique_event_types: HashSet<&str> =
        ia_events.iter().map(|e| e.event_type.as_str()).collect();

    println!("--- IA Blueprint Summary ---");
    println!("  Events generated:      {}", ia_events.len());
    println!("  Procedures executed:   {}", ia_procedures_executed);
    println!("  Phases completed:      {}", ia_phases_completed);
    println!("  Anomalies injected:    {}", ia_anomalies);
    println!("  Duration (hours):      {:.1}", ia_duration);
    println!("  Self-loop events:      {}", ia_self_loops);
    println!("  Unique event types:    {}", ia_unique_event_types.len());
    println!("  Event types:           {:?}", {
        let mut v: Vec<&str> = ia_unique_event_types.iter().copied().collect();
        v.sort();
        v
    });
    println!();

    // ===================================================================
    // 2c. Artifact Generation Summary
    // ===================================================================
    let fsa_bag = &fsa_result.artifacts;
    let ia_bag = &ia_result.artifacts;

    println!("--- FSA Artifact Generation ---");
    println!("  Engagements:           {}", fsa_bag.engagements.len());
    println!(
        "  Engagement letters:    {}",
        fsa_bag.engagement_letters.len()
    );
    println!(
        "  Materiality calcs:     {}",
        fsa_bag.materiality_calculations.len()
    );
    println!(
        "  Risk assessments:      {}",
        fsa_bag.risk_assessments.len()
    );
    println!(
        "  Combined risk assess:  {}",
        fsa_bag.combined_risk_assessments.len()
    );
    println!("  Workpapers:            {}", fsa_bag.workpapers.len());
    println!("  Evidence:              {}", fsa_bag.evidence.len());
    println!("  Findings:              {}", fsa_bag.findings.len());
    println!("  Sampling plans:        {}", fsa_bag.sampling_plans.len());
    println!(
        "  Analytical results:    {}",
        fsa_bag.analytical_results.len()
    );
    println!(
        "  Going concern:         {}",
        fsa_bag.going_concern_assessments.len()
    );
    println!(
        "  Subsequent events:     {}",
        fsa_bag.subsequent_events.len()
    );
    println!("  Audit opinions:        {}", fsa_bag.audit_opinions.len());
    println!(
        "  Key audit matters:     {}",
        fsa_bag.key_audit_matters.len()
    );
    println!("  Confirmations:         {}", fsa_bag.confirmations.len());
    println!("  TOTAL ARTIFACTS:       {}", fsa_bag.total_artifacts());
    println!();

    println!("--- IA Artifact Generation ---");
    println!("  Engagements:           {}", ia_bag.engagements.len());
    println!("  Workpapers:            {}", ia_bag.workpapers.len());
    println!("  Risk assessments:      {}", ia_bag.risk_assessments.len());
    println!("  Findings:              {}", ia_bag.findings.len());
    println!("  Sampling plans:        {}", ia_bag.sampling_plans.len());
    println!("  TOTAL ARTIFACTS:       {}", ia_bag.total_artifacts());
    println!();

    assert!(
        fsa_bag.total_artifacts() > 0,
        "FSA should produce artifacts"
    );
    // IA uses different step commands than FSA — most fall through to
    // generic workpaper generation which requires an engagement in the bag.
    // Full IA command mapping is planned for a future iteration.
    println!(
        "  (IA artifact mapping covers {} of 82 steps)",
        ia_bag.total_artifacts()
    );

    // ===================================================================
    // 3. Export FSA event trail to temp file, read back, print first 3
    // ===================================================================
    let tmp_dir = std::env::temp_dir();
    let tmp_path = tmp_dir.join(format!("evaluate_output_fsa_{}.json", std::process::id()));

    export_events_to_file(&fsa_result.event_log, &tmp_path)
        .expect("export_events_to_file must succeed");

    let contents = std::fs::read_to_string(&tmp_path).expect("temp file must be readable");
    let parsed: Vec<serde_json::Value> =
        serde_json::from_str(&contents).expect("must be valid JSON");

    println!("--- FSA Export: first 3 events (of {}) ---", parsed.len());
    for (i, event) in parsed.iter().take(3).enumerate() {
        println!(
            "  [{}] type={}, proc={}, cmd={}, ts={}",
            i,
            event["event_type"].as_str().unwrap_or("?"),
            event["procedure_id"].as_str().unwrap_or("?"),
            event["command"].as_str().unwrap_or("?"),
            event["timestamp"].as_str().unwrap_or("?"),
        );
    }
    println!();

    // Clean up temp file.
    let _ = std::fs::remove_file(&tmp_path);

    // Verify round-trip count
    assert_eq!(
        parsed.len(),
        fsa_events.len(),
        "round-tripped JSON length must match event count"
    );

    // ===================================================================
    // 4. OCEL projection
    // ===================================================================
    let fsa_ocel = project_to_ocel(&fsa_result.event_log);
    let ia_ocel = project_to_ocel(&ia_result.event_log);

    println!("--- OCEL 2.0 Projection ---");
    println!(
        "  FSA: {} object types, {} events, {} objects",
        fsa_ocel.object_types.len(),
        fsa_ocel.events.len(),
        fsa_ocel.objects.len(),
    );
    println!(
        "  IA:  {} object types, {} events, {} objects",
        ia_ocel.object_types.len(),
        ia_ocel.events.len(),
        ia_ocel.objects.len(),
    );
    println!("  FSA object types: {:?}", fsa_ocel.object_types);
    println!("  IA  object types: {:?}", ia_ocel.object_types);
    println!();

    assert!(!fsa_ocel.object_types.is_empty());
    assert!(!ia_ocel.object_types.is_empty());

    // ===================================================================
    // 5. Shortest-path analysis (via optimizer crate types replicated here
    //    since we can only depend on datasynth-audit-fsm; we do a manual
    //    BFS-based analysis using the blueprint directly)
    // ===================================================================
    // We perform a simplified shortest-path analysis using the blueprint's
    // procedure aggregates directly (the optimizer crate does the same logic).
    let fsa_bwp = BlueprintWithPreconditions::load_builtin_fsa().expect("FSA blueprint must load");
    let ia_bwp = BlueprintWithPreconditions::load_builtin_ia().expect("IA blueprint must load");

    let fsa_min_transitions = count_total_min_transitions(&fsa_bwp);
    let ia_min_transitions = count_total_min_transitions(&ia_bwp);

    println!("--- Shortest Path Analysis ---");
    println!("  FSA total minimum transitions: {}", fsa_min_transitions);
    println!("  IA  total minimum transitions: {}", ia_min_transitions);
    println!();

    assert!(
        fsa_min_transitions > 0,
        "FSA should have at least one minimum transition"
    );
    assert!(
        ia_min_transitions > 0,
        "IA should have at least one minimum transition"
    );

    // ===================================================================
    // 6. Monte Carlo (10 iterations) on FSA
    // ===================================================================
    let mc_iterations = 10;
    let mc_seed = 42u64;
    let mc_ctx = EngagementContext::test_default();

    let mut total_events: u64 = 0;
    let mut total_duration: f64 = 0.0;
    let mut happy_path: Vec<String> = Vec::new();

    for i in 0..mc_iterations {
        let iter_seed = mc_seed.wrapping_add(i as u64);
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().expect("FSA must load");
        let overlay = default_overlay();
        let rng = ChaCha8Rng::seed_from_u64(iter_seed);
        let mut engine = AuditFsmEngine::new(bwp, overlay, rng);

        let result = engine
            .run_engagement(&mc_ctx)
            .expect("MC iteration must succeed");

        total_events += result.event_log.len() as u64;
        total_duration += result.total_duration_hours;

        if i == 0 {
            // Build happy path from first iteration.
            let mut seen = HashSet::new();
            for event in &result.event_log {
                if event.to_state.as_deref() == Some("completed")
                    && seen.insert(event.procedure_id.clone())
                {
                    happy_path.push(event.procedure_id.clone());
                }
            }
        }
    }

    let avg_events = total_events as f64 / mc_iterations as f64;
    let avg_duration = total_duration / mc_iterations as f64;

    println!("--- Monte Carlo ({} iterations, FSA) ---", mc_iterations);
    println!("  Avg events per run:    {:.1}", avg_events);
    println!("  Avg duration (hours):  {:.1}", avg_duration);
    println!("  Happy path:            {}", happy_path.join(" -> "));
    println!();

    assert!(avg_events > 0.0, "avg_events should be positive");
    assert!(avg_duration > 0.0, "avg_duration should be positive");
    assert!(!happy_path.is_empty(), "happy_path should be non-empty");

    // ===================================================================
    // JSON export sanity check
    // ===================================================================
    let json = export_events_to_json(&fsa_result.event_log).expect("JSON export must succeed");
    let re_parsed: Vec<serde_json::Value> =
        serde_json::from_str(&json).expect("JSON must re-parse");
    assert_eq!(re_parsed.len(), fsa_events.len());

    println!("--- All checks passed ---\n");
}

// ---------------------------------------------------------------------------
// Simplified shortest-path BFS (mirrors optimizer logic without depending on
// the optimizer crate).
// ---------------------------------------------------------------------------

fn count_total_min_transitions(bwp: &BlueprintWithPreconditions) -> usize {
    use std::collections::{HashMap, VecDeque};

    let mut total = 0usize;

    for phase in &bwp.blueprint.phases {
        for procedure in &phase.procedures {
            let agg = &procedure.aggregate;
            if agg.transitions.is_empty() || agg.initial_state.is_empty() {
                continue;
            }

            // Build adjacency.
            let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
            for state in &agg.states {
                adj.entry(state.as_str()).or_default();
            }
            for t in &agg.transitions {
                adj.entry(t.from_state.as_str()).or_default();
                adj.entry(t.to_state.as_str()).or_default();
                adj.entry(t.from_state.as_str())
                    .or_default()
                    .push(t.to_state.as_str());
            }

            let initial = agg.initial_state.as_str();
            if !adj.contains_key(initial) {
                continue;
            }

            // Terminal states.
            let terminals: HashSet<&str> = adj
                .iter()
                .filter(|(_, neighbours)| neighbours.is_empty())
                .map(|(s, _)| *s)
                .collect();

            if terminals.is_empty() {
                continue;
            }

            // BFS.
            let mut visited = HashSet::new();
            visited.insert(initial);
            let mut queue: VecDeque<(&str, usize)> = VecDeque::new();
            queue.push_back((initial, 0));

            while let Some((current, depth)) = queue.pop_front() {
                if terminals.contains(current) {
                    total += depth;
                    break;
                }
                if let Some(neighbours) = adj.get(current) {
                    for &next in neighbours {
                        if visited.insert(next) {
                            queue.push_back((next, depth + 1));
                        }
                    }
                }
            }
        }
    }

    total
}
