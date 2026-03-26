# Big 4 Cross-Firm Methodology Benchmark

**Date**: 2026-03-25
**Seed**: 42, **Overlay**: default

## Data Source Disclaimer

**EY GAM is the only fully populated model** in this benchmark. It was extracted directly from EY's Atlas platform via the AuditMethodology scraper, producing 1,182 procedures with 3,035 steps — a faithful representation of the actual EY Global Audit Methodology.

The **KPMG Clara, PwC Aura, and Deloitte Omnia** blueprints are **derived from public ISA standards** (37 standards, 702 requirement paragraphs) with **firm-specific flavours** layered on top based on publicly available information about each firm's audit platform, tools, and quality frameworks. They are **not** scraped from proprietary firm methodologies and should be understood as ISA-based approximations styled after each firm's publicly documented approach. The firm-specific extra procedures (7-9 per firm) reflect documented platform capabilities (Clara AI, Halo analytics, Spotlight, Argus, etc.) but are not verified against internal firm methodology documentation.

## Results (Firm-Enriched Blueprints)

### Cross-Firm Comparison (ISA-based, comparable scale)

| Firm | Phases | Procs | Steps | Events | Artifacts | Hours | Anomalies | Compl% | Data% | AI% | Human% |
|------|--------|-------|-------|--------|-----------|-------|-----------|--------|-------|-----|--------|
| Generic ISA | 3 | 9 | 24 | 51 | 2,012 | 731 | 2 | 100% | 17% | 29% | 54% |
| KPMG Clara | 7 | 44 | 728 | 891 | 30,371 | 4,013 | 77 | 100% | 11% | 13% | 75% |
| PwC Aura | 7 | 44 | 729 | 976 | 37,055 | 4,144 | 93 | 100% | 11% | 13% | 76% |
| Deloitte Omnia | 7 | 46 | 733 | 958 | 34,236 | 4,448 | 89 | 100% | 12% | 14% | 74% |
| **EY GAM Lite** | **7** | **52** | **757** | **955** | **44,641** | **4,587** | **80** | **100%** | **11%** | **14%** | **74%** |
| IIA-GIAS | 9 | 34 | 82 | 205 | 3,814 | 2,737 | 12 | 100% | 6% | 67% | 27% |

### EY GAM Full (proprietary, not shareable)

| Firm | Phases | Procs | Steps | Events | Artifacts | Hours | Anomalies | Compl% | Data% | AI% | Human% |
|------|--------|-------|-------|--------|-----------|-------|-----------|--------|-------|-----|--------|
| **EY GAM Full** | **8** | **1,182** | **3,035** | **7,731** | **367,090** | **N/A** | **372** | **100%** | **~40%** | **~36%** | **~24%** |

*EY GAM Full is scraped from EY Atlas and contains proprietary methodology content. It is not included in the shareable blueprint set. EY GAM Lite provides a comparable ISA-based + EY-specific alternative suitable for benchmarking and distribution.*

## Blueprint Composition

| Blueprint | ISA Base | Firm-Specific Extra | Source | Shareable? |
|-----------|----------|-------------------|--------|------------|
| Generic ISA | 9 hand-crafted procedures | — | Manual ISA mapping | Yes |
| KPMG Clara | 37 ISA standards (702 steps) | +7 procedures (26 steps): Sentinel, BPM, MindBridge scoring, SoD analysis, forensic analytics, EQCR, FRA | ISA standards + public KPMG documentation | Yes |
| PwC Aura | 37 ISA standards (702 steps) | +7 procedures (27 steps): FRISK 13-factor, Halo journal/population/3-way/outlier, QRP review, ECR | ISA standards + public PwC documentation | Yes |
| Deloitte Omnia | 37 ISA standards (702 steps) | +9 procedures (31 steps): Cortex, Argus, DARTbot, Spotlight scoring/JE/benchmarking, iConfirm, Omnia GenAI, Trustworthy AI | ISA standards + public Deloitte documentation | Yes |
| **EY GAM Lite** | **37 ISA standards (702 steps)** | **+8 procedures (29 steps): Canvas risk/materiality, Atlas methodology, Helix analytics, specialist coordination, EQR, digital audit, GAM compliance** | **ISA standards + public EY documentation** | **Yes** |
| IIA-GIAS | 34 hand-crafted procedures | — | Manual IIA-GIAS mapping (96.2% coverage) | Yes |
| **EY GAM Full** | **1,182 scraped procedures** | **N/A (native)** | **Extracted from EY Atlas platform** | **No (proprietary)** |

## Structural Analysis

### Firm-Specific Differentiation

The three Big 4 blueprints share the same ISA foundation (37 procedures, 702 steps) but diverge through firm-specific additions:

| Metric | KPMG | PwC | Deloitte | Why |
|--------|------|-----|----------|-----|
| **Procedures** | 44 | 44 | **46** | Deloitte has more named tools (Argus, Spotlight, Cortex, DARTbot, iConfirm) |
| **Steps** | 728 | 729 | **733** | Deloitte's 9 extra procedures add more steps |
| **Events** | 891 | **976** | 958 | PwC's FRISK has 6 steps generating more events |
| **Artifacts** | 30,371 | **37,055** | 34,236 | PwC's Halo analytics generate more workpapers |
| **Hours** | 4,013 | 4,144 | **4,448** | Deloitte's additional procedures add engagement time |
| **Anomalies** | 77 | **93** | 89 | More events = more anomaly injection opportunities |

### Firm-Specific Procedures

**KPMG Clara** (7 extra procedures):
- Sentinel independence check (pre-engagement)
- Business process mining via Clara Analytics (KCa)
- MindBridge full-population AI transaction scoring (100% population, rules + statistics + ML)
- SoD transaction-level analysis (identifies conflicts AND actual exercises with financial impact)
- Forensic fraud analytics (forensic test routines, fraud surveys)
- EQCR multi-point review (planning, interim, pre-completion, pre-issuance — veto authority)
- Financial Report Analyzer (FRA) AI disclosure checklist

**PwC Aura/Halo** (7 extra procedures):
- FRISK 13-factor pre-engagement risk assessment
- Halo journal pattern analysis (posting times, users, round amounts, period-end entries)
- Halo full-population testing (100% of transactions, not samples)
- Halo three-way match analytics (PO/GR/Invoice validation)
- Halo outlier visualization (interactive dashboards for anomaly detection)
- Quality Review Partner (QRP) hot review (independent challenge before report issuance)
- Engagement Compliance Review (ECR) (post-issuance inspection)

**Deloitte Omnia** (9 extra procedures):
- Cortex data ingestion and harmonization (multi-year, multi-system)
- Argus ML/NLP document extraction (leases, contracts, legal letters, board minutes)
- DARTbot GenAI accounting research consultation
- Spotlight transaction risk scoring (who/what/where/when/why/how)
- Spotlight full-population journal entry analysis (9-10 tests per population)
- Spotlight anonymous cross-client benchmarking
- iConfirm external confirmation automation
- Omnia GenAI documentation review (clarity, consistency, completeness)
- Trustworthy AI governance gate (7-dimension AI output validation)

### Judgment Level Distribution

| Category | Generic ISA | KPMG | PwC | Deloitte | IA | EY GAM |
|----------|------------|------|-----|----------|-----|--------|
| Data-only | 17% | 11% | 11% | 12% | 6% | ~40% |
| AI-assistable | 29% | 13% | 13% | 14% | 67% | ~36% |
| Human-required | 54% | 75% | 76% | 74% | 27% | ~24% |

**Observations**:
- ISA-complete blueprints have higher human-required % (74-76%) than the generic ISA (54%) because ISA requirement paragraphs are predominantly judgment-oriented
- Internal audit (IIA-GIAS) has the highest AI-assistable % (67%) — internal audit is inherently more process-driven
- EY GAM has the highest data-only % (~40%) because the full methodology includes many computational/verification steps not captured at the ISA requirement level

### LLM Integration Potential

Across all Big 4 FSA blueprints:
- **~12% data-only** — fully automatable (recalculation, matching, Benford's, sampling execution)
- **~13% AI-assistable** — Claude can draft narratives, prepare workpapers, generate evidence summaries
- **~75% human-required** — professional skepticism needed per ISA 200, but AI supports with data gathering, preliminary analysis, and draft recommendations

The firm-specific extra procedures shift the balance slightly:
- KPMG's MindBridge scoring adds data-only steps
- PwC's Halo analytics adds data-only steps
- Deloitte's Argus/Spotlight adds a mix of data-only (extraction) and AI-assistable (research)
- All firms' quality review procedures add human-required steps

## Methodology Notes

- All blueprints use the same 4-state FSM lifecycle: `not_started → in_progress → under_review → completed`
- All blueprints use the same generation overlay (default: revision_probability=0.15, iteration_limit=50)
- Artifacts are generated by the StepDispatcher with command prefix dispatch (100% coverage across all blueprints)
- The benchmark is deterministic (seed=42) and reproducible
