# datasynth-audit-fsm

YAML-driven audit FSM engine for methodology-based audit trail and artifact generation.

## Overview

`datasynth-audit-fsm` loads audit methodology blueprints as event-sourced finite state machines and generates deterministic event trails with concrete ISA-compliant artifacts.

- **Two-layer architecture**: Blueprints define *what happens* (procedures, phases, state machines), overlays control *how* (probabilities, timing, anomaly rates)
- **10 builtin blueprints**: FSA, IA, KPMG, PwC, Deloitte, EY GAM Lite, SOC 2, PCAOB, Regulatory, plus GAM via filesystem
- **14 generators**: StepDispatcher maps 135+ step commands to concrete artifact generators
- **judgment_level on every step**: Each step carries a judgment level for risk-based procedure selection
- **Deterministic**: ChaCha8Rng-seeded -- same seed produces identical output

## Built-in Blueprints

| Blueprint | Procedures | Phases | Steps | Events | Artifacts |
|-----------|-----------|--------|-------|--------|-----------|
| FSA (ISA) | 9 | 3 | 24 | 51 | 1,916 |
| IA (IIA-GIAS) | 34 | 9 | 82 | 368 | 1,891 |
| KPMG FSA | Firm-specific ISA methodology | | | | |
| PwC FSA | Firm-specific ISA methodology | | | | |
| Deloitte FSA | Firm-specific ISA methodology | | | | |
| EY GAM Lite | EY Global Audit Methodology (lite) | | | | |
| SOC 2 Type II | Trust Services Criteria | | | | |
| PCAOB Integrated | AS 2201 integrated audit | | | | |
| Regulatory Exam | Regulatory examination | | | | |

Custom blueprints: [SyntheticDataBlueprints](https://github.com/mivertowski/SyntheticDataBlueprints)

## Key Features

- 8-state C2CE (Condition-Criteria-Cause-Effect) lifecycle for finding development
- Self-loop handling with configurable max iterations
- Continuous phase support (parallel execution for ethics, governance, quality)
- Discriminator-based procedure filtering (categories, risk ratings, engagement types)
- Generation overlay presets: `default`, `thorough`, `rushed`
- 6 export formats: JSON, CSV (Disco/Celonis), XES 2.0 (ProM/pm4py), OCEL 2.0, Celonis, Parquet
- Analytics inventory integration (FSA, IA, SOC 2, PCAOB, Regulatory)
- ContentGenerator trait with pluggable implementations (template + Claude CLI adapter via `claude-content` feature)
- Streaming execution with live anomaly injection
- Benchmark and curriculum dataset generation

## Usage

```rust
use datasynth_audit_fsm::loader::{BlueprintWithPreconditions, load_overlay, OverlaySource, BuiltinOverlay};
use datasynth_audit_fsm::engine::AuditFsmEngine;
use datasynth_audit_fsm::context::EngagementContext;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
let overlay = load_overlay(&OverlaySource::Builtin(BuiltinOverlay::Default)).unwrap();
let mut engine = AuditFsmEngine::new(bwp, overlay, ChaCha8Rng::seed_from_u64(42));
let result = engine.run_engagement(&EngagementContext::demo()).unwrap();

// result.event_log -- ordered audit events
// result.artifacts -- typed artifacts (engagements, materiality, risks, workpapers, ...)
```

## Configuration

```yaml
audit:
  enabled: true
  fsm:
    enabled: true
    blueprint: builtin:fsa      # builtin:fsa, builtin:ia, builtin:kpmg, builtin:pwc, etc.
    overlay: builtin:default    # builtin:default, builtin:thorough, builtin:rushed
```

## Modules

| Module | Purpose |
|--------|---------|
| `schema` | Blueprint and overlay Rust types (deserialized from YAML) |
| `loader` | YAML parsing, validation, DAG topological sort, builtin resolution |
| `engine` | FSM execution engine with DAG walk and event emission |
| `dispatch` | StepDispatcher mapping step commands to 14 audit generators |
| `artifact` | ArtifactBag accumulator for generated audit artifacts |
| `context` | EngagementContext with financial, team, and reference data |
| `event` | AuditEvent, AuditEventBuilder, anomaly types |
| `content` | ContentGenerator trait and template-based default |
| `content_claude` | Claude CLI content adapter (behind `claude-content` feature) |
| `analytics_inventory` | Data requirements and analytical procedures (FSA, IA, SOC 2, PCAOB, Regulatory) |
| `export` | JSON, CSV, XES, OCEL, Celonis, Parquet exporters |
| `streaming` | Streaming execution for incremental event processing |
| `live_injection` | Live anomaly injection during execution |
| `benchmark` | Benchmark and curriculum dataset generation |
| `error` | AuditFsmError with validation, parse, and runtime variants |

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
