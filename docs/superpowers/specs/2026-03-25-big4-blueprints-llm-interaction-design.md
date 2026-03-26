# Big 4 Methodology Blueprints + LLM Interaction Points

**Date**: 2026-03-25
**Status**: Approved
**Scope**: Create KPMG, PwC, and Deloitte methodology blueprints derived from ISA standards. Add judgment_level classification to all blueprint steps. Implement cross-firm benchmark simulation.

## Research Summary

### Big 4 Audit Platforms and Methodologies

| Firm | Platform | Methodology | Key Differentiator |
|------|----------|-------------|-------------------|
| **EY** | EY Canvas / Atlas | GAM (Global Audit Methodology) | Most publicly documented; 1,182 procedures in SyntheticData |
| **KPMG** | KPMG Clara (KCw) | KAM (KPMG Audit Manual) | Professional Judgement Framework; AI agents for substantive procedures |
| **PwC** | Aura + Halo | PwC Audit Guide | Halo for data analytics; Aura for workflow standardization |
| **Deloitte** | Omnia | Deloitte Audit Approach | Cloud-based; cognitive technology integration |

Sources:
- [KPMG Clara](https://kpmg.com/ch/en/services/audit/auditing-software-kpmg-clara.html)
- [KPMG Methodology](https://kpmg.com/kh/en/home/services/audit/auditmethodology.html)
- [KPMG AI Integration](https://kpmg.com/xx/en/media/press-releases/2025/04/kpmg-advances-ai-integration-in-kpmg-clara-smart-audit-platform.html)
- [Big 4 Audit Tools](https://betachon.com/what-audit-tools-does-the-big-4-use/)
- [PwC Audit Technology](https://www.pwc.com/us/en/services/audit-assurance/financial-statement-audit/audit-technology.html)
- [PwC Aura Analysis](https://financialauditexpert.com/blog/pwc_s_aura_a_comparative_analysis_of_big_4_audit_software_ef.php)

### Key Insight

All Big 4 follow ISA. The procedure structure is fundamentally the same — differences are in:
1. **Naming conventions** (KPMG: "Business Process Understanding", PwC: "Understanding the Entity")
2. **Quality control layers** (different sign-off requirements, EQR triggers)
3. **Tool integration** (Clara, Halo, Omnia-specific steps)
4. **Emphasis areas** (KPMG emphasizes Professional Judgement Framework, PwC emphasizes data analytics via Halo)

### Approach

Rather than reverse-engineering proprietary methodologies (which would be legally questionable), we create **ISA-based firm-style blueprints** that:
- Follow the same ISA procedures as our FSA blueprint
- Use firm-specific naming conventions and organizational structure
- Model firm-specific quality control and review layers
- Are clearly labeled as "ISA-based, firm-style" (not proprietary)

## LLM Interaction Point Classification

### Framework

Every audit step falls into one of three categories based on ISA requirements:

| Level | Label | Definition | ISA Basis |
|-------|-------|-----------|-----------|
| 0 | `data_only` | Deterministic computation from data. No judgment needed. | ISA 520 (recalculation), ISA 530 (sampling execution) |
| 1 | `ai_assistable` | Pattern recognition + narrative generation. AI drafts, human reviews. | ISA 230 (documentation), ISA 500 (evidence gathering) |
| 2 | `human_required` | Professional skepticism and judgment. AI supports, human decides. | ISA 200 (professional skepticism), ISA 240 (fraud), ISA 700 (opinion) |

### Distribution (from GAM analysis)

| Level | Steps | % | Command Verbs |
|-------|-------|---|---------------|
| `data_only` | ~1,200 | 40% | perform, calculate, compute, verify, reperform, check, analyze |
| `ai_assistable` | ~1,100 | 36% | provide, document, prepare, describe, report, present, also, use |
| `human_required` | ~735 | 24% | evaluate, assess, consider, determine, exercise, discuss, approve, identify, review |

### Schema Addition

```yaml
# On BlueprintStep
judgment_level: ai_assistable    # data_only | ai_assistable | human_required
ai_capabilities:                 # what AI can do for this step
  - draft_narrative
  - analyze_data
  - flag_exceptions
human_responsibilities:          # what human must do
  - review_and_approve
  - apply_professional_skepticism
```

## Solution

### Part 1: Firm-Style Blueprint Generation

Create three new ISA-based blueprints styled after KPMG, PwC, and Deloitte methodologies. Each shares the same ISA procedure structure as FSA but with firm-specific characteristics.

#### KPMG Clara Style
- Phases: Client Acceptance → Planning → Risk Assessment → Design Responses → Execute Procedures → Evaluate Results → Form Opinion → Reporting
- Emphasis: Professional Judgement Framework checkpoints at key decision points
- Quality: KCw review gates, engagement quality reviewer (EQR) triggers for PIEs
- Naming: "Business Process Understanding", "Determine Strategy", "Evaluate Results"

#### PwC Aura Style
- Phases: Engagement Acceptance → Preliminary Activities → Risk Assessment → Audit Strategy → Execution → Completion → Reporting
- Emphasis: Data analytics via Halo at every substantive phase
- Quality: Aura workflow sign-offs, multi-level review (manager, director, partner)
- Naming: "Understanding the Entity", "Design Audit Approach", "Evaluate Audit Evidence"

#### Deloitte Omnia Style
- Phases: Engagement Setup → Planning → Risk Assessment → Response Design → Testing → Completion → Opinion → Communication
- Emphasis: Cognitive technology integration, risk-based workflows
- Quality: Omnia collaboration checkpoints, real-time data sharing gates
- Naming: "Establish Engagement", "Identify Risks", "Design Audit Procedures"

### Part 2: Judgment Level Classification

Add `judgment_level` field to `BlueprintStep` schema and classify all steps across all blueprints (FSA, IA, KPMG, PwC, Deloitte, GAM).

Classification rules (by command verb):
- `data_only`: perform_, calculate_, compute_, verify_, reperform_, check_, analyze_, test_, execute_
- `ai_assistable`: provide_, document_, prepare_, describe_, report_, present_, also_, use_, include_, may_, can_, record_, reflect_
- `human_required`: evaluate_, assess_, consider_, determine_, exercise_, discuss_, approve_, identify_, review_, sign_, authorize_, observe_, inquire_

### Part 3: Cross-Firm Benchmark Simulation

Run all 5 FSA-level blueprints (generic FSA, KPMG, PwC, Deloitte + IA and GAM) and compare:
- Event count, artifact count, duration
- Judgment level distribution
- Phase completion patterns
- Standards coverage
- Anomaly detection profiles under same overlay

## Files

### New blueprints (in datasynth-audit-fsm)
- `blueprints/kpmg_fsa.yaml`
- `blueprints/pwc_fsa.yaml`
- `blueprints/deloitte_fsa.yaml`

### Schema changes
- `schema.rs`: add `judgment_level: Option<String>` and `ai_capabilities: Vec<String>` to `BlueprintStep`
- `loader.rs`: wire through from raw types

### New module
- `benchmark_comparison.rs` in `datasynth-audit-optimizer`: cross-blueprint comparison framework

### Tests
- Each new blueprint loads and validates
- Each blueprint runs a full engagement
- Judgment level distribution matches expected ratios
- Cross-firm comparison produces meaningful differences
