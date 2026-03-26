# Big 4 Cross-Firm Methodology Benchmark

**Date**: 2026-03-25
**Seed**: 42, **Overlay**: default

## Results

| Firm | Phases | Procs | Steps | Events | Artifacts | Hours | Anomalies | Compl% | Data% | AI% | Human% |
|------|--------|-------|-------|--------|-----------|-------|-----------|--------|-------|-----|--------|
| Generic ISA | 3 | 9 | 24 | 51 | 2,012 | 731 | 2 | 100% | 17% | 29% | 54% |
| KPMG Clara | 5 | 10 | 26 | 56 | 1,923 | 816 | 3 | 100% | 12% | 31% | 58% |
| PwC Aura | 7 | 9 | 22 | 49 | 1,919 | 666 | 1 | 100% | 14% | 23% | 64% |
| Deloitte Omnia | 8 | 10 | 24 | 54 | 1,921 | 760 | 0 | 100% | 12% | 25% | 62% |
| IIA-GIAS | 9 | 34 | 82 | 205 | 3,814 | 2,737 | 12 | 100% | 6% | 67% | 27% |
| EY GAM | 8 | 1,182 | 3,035 | 7,731 | 367,090 | N/A | 372 | 100% | ~40% | ~36% | ~24% |

## Structural Analysis

### Phase Granularity
- **PwC** has the most phases (7) but fewest steps (22) — more granular phase gates, leaner execution
- **KPMG** adds a standalone EQR procedure (10 total vs 9 for others)
- **Deloitte** has the most phases (8) with a separate Communication phase
- **Generic ISA** is the most compact (3 phases)

### Judgment Level Distribution

The judgment_level classification reveals firm methodology culture:

- **PwC Aura**: 64% human-required — partner-heavy, conservative review approach
- **KPMG Clara**: 31% AI-assistable — aligns with their AI agent integration strategy
- **Deloitte Omnia**: 62% human-required — emphasis on professional judgment with cognitive tech support
- **IIA-GIAS**: 67% AI-assistable, only 27% human-required — internal audit is more process-driven
- **EY GAM**: ~40% data-only — the full methodology has more computational/verification steps

### LLM Integration Potential

Across all FSA-level blueprints:
- **~15% data-only** — fully automatable by generators (recalculation, three-way match, Benford's)
- **~30% AI-assistable** — Claude can draft narratives, generate evidence summaries, prepare workpapers
- **~55% human-required** — professional skepticism needed, but AI supports with data gathering and preliminary analysis

### Key Differences by Firm

| Aspect | KPMG | PwC | Deloitte |
|--------|------|-----|----------|
| Distinctive phase | N/A | Separate Reporting phase | Separate Communication phase |
| Extra procedure | Standalone EQR | N/A | Omnia collaboration checkpoint |
| Analytics approach | Per-procedure (Clara AI agents) | Dedicated Halo analytics steps | Cognitive technology embedded |
| QC emphasis | Professional Judgement Framework | Multi-level partner review | Real-time collaboration gates |
| AI integration posture | Aggressive (31% AI-assistable) | Conservative (64% human) | Moderate (62% human) |
