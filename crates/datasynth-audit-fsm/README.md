# datasynth-audit-fsm

YAML-driven audit FSM engine for methodology-based audit trail and artifact generation.

## Overview

`datasynth-audit-fsm` loads audit methodology blueprints (ISA, IIA-GIAS) as event-sourced finite state machines and generates both deterministic event trails and concrete ISA-compliant artifacts.

- **Two-layer architecture**: Blueprints define *what happens* (procedures, phases, state machines), overlays control *how* (probabilities, timing, anomaly rates)
- **Two built-in blueprints**: Financial Statement Audit (FSA) and Internal Audit (IA)
- **14 generators**: StepDispatcher maps 135 step commands to concrete artifact generators
- **Deterministic**: ChaCha8Rng-seeded — same seed produces identical output

## Built-in Blueprints

| Blueprint | Procedures | Phases | Steps | Events | Artifacts |
|-----------|-----------|--------|-------|--------|-----------|
| FSA (ISA) | 9 | 3 | 24 | 51 | 1,916 |
| IA (IIA-GIAS) | 34 | 9 | 82 | 368 | 1,891 |

Custom blueprints: [SyntheticDataBlueprints](https://github.com/mivertowski/SyntheticDataBlueprints)

## Key Features

- 8-state C2CE (Condition-Criteria-Cause-Effect) lifecycle for finding development
- Self-loop handling with configurable max iterations
- Continuous phase support (parallel execution for ethics, governance, quality)
- Discriminator-based procedure filtering (categories, risk ratings, engagement types)
- Generation overlay presets: `default`, `thorough`, `rushed`
- Flat JSON audit event trail + OCEL 2.0 projection exports

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
let result = engine.run_engagement(&EngagementContext::test_default()).unwrap();

// result.event_log — 51 ordered audit events
// result.artifacts — 1,916 typed artifacts (engagements, materiality, risks, workpapers, ...)
```

## Configuration

```yaml
audit:
  enabled: true
  fsm:
    enabled: true
    blueprint: builtin:fsa      # builtin:fsa, builtin:ia, or path to custom YAML
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
| `export` | Flat JSON event trail and OCEL 2.0 projection exporters |
| `error` | AuditFsmError with validation, parse, and runtime variants |

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
