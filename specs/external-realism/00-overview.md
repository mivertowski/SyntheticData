# External Realism Enhancement Suite — Overview

## Purpose

This specification suite defines enhancements to the DataSynth framework that introduce
**external factor modeling** — macroeconomic conditions, geopolitical events, natural
disasters, regulatory changes, supply chain disruptions, and climate scenarios — to
dramatically increase the realism of generated synthetic enterprise financial data.

## Motivation

Current synthetic data generators (including DataSynth's existing drift/regime models)
typically treat enterprises as isolated systems with simple sinusoidal economic cycles.
Real enterprise data, however, reflects a complex web of **exogenous factors** that
simultaneously affect transaction volumes, amounts, timing, counterparty behavior, and
risk profiles. Academic research consistently shows that ignoring these correlations
produces synthetic data that is easily distinguishable from real data by even simple
statistical tests (Park et al., 2018; Assefa et al., 2020).

## Current State Assessment

DataSynth already has foundational infrastructure in:

- **drift.rs** — Economic cycle (sinusoidal), regime changes, parameter drifts
- **market_drift.rs** — Commodity pricing, price shocks, industry cycles
- **behavioral_drift.rs** — Entity-level responses to economic context
- **seasonality.rs** — Industry-specific seasonal events
- **event_timeline.rs** — Organizational events (M&A, restructuring)

**Key gaps** this suite addresses:

| Gap | Impact |
|-----|--------|
| No correlated macro factors | GDP, rates, inflation move independently |
| No interest rate term structure | Static rates unrealistic for treasury/banking |
| No credit cycle dynamics | Missing credit deterioration cascades |
| No geopolitical event modeling | Cannot simulate sanctions, trade wars |
| No natural disaster impacts | Missing catastrophic business interruption |
| No supply chain contagion | Disruptions don't propagate through network |
| No regulatory change modeling | Compliance cost shocks absent |
| No climate/ESG scenarios | Missing transition risk dynamics |
| No scenario orchestration | Cannot replay historical crises coherently |
| No cross-factor coherence validation | Generated scenarios may be internally inconsistent |

## Specification Structure

| Spec | Title | Scope |
|------|-------|-------|
| [01](./01-macroeconomic-factor-engine.md) | Macroeconomic Factor Engine | GDP, interest rates, inflation, unemployment, credit spreads |
| [02](./02-external-shock-event-system.md) | External Shock & Event System | Shock taxonomy, propagation, lifecycle, recovery curves |
| [03](./03-geopolitical-regulatory-risk.md) | Geopolitical & Regulatory Risk | Sanctions, trade policy, regulatory changes, political instability |
| [04](./04-natural-disaster-climate-impact.md) | Natural Disaster & Climate Impact | Catastrophes, physical risk, transition risk, carbon pricing |
| [05](./05-supply-chain-contagion.md) | Supply Chain Contagion Network | Multi-tier disruption propagation, bullwhip effect |
| [06](./06-scenario-orchestration.md) | Scenario Orchestration & Stress Testing | Historical replays, regulatory stress tests, scenario library |
| [07](./07-cross-factor-correlation.md) | Cross-Factor Correlation & Coherence | VAR-based factor models, copula correlations, coherence checks |
| [08](./08-realism-validation-metrics.md) | Enhanced Realism Validation & Metrics | Statistical tests, divergence metrics, benchmarking |

## Design Principles

1. **Additive, not breaking** — All enhancements are optional configuration sections;
   existing configs continue to work unchanged.
2. **Deterministic reproducibility** — All stochastic models use the existing ChaCha8 RNG
   infrastructure with configurable seeds.
3. **Composable factors** — External factors compose multiplicatively/additively through
   the existing `DriftAdjustments` pipeline.
4. **Scientifically grounded** — Each model cites academic literature and uses established
   econometric approaches.
5. **Performance-conscious** — Factor computations are O(1) per period lookup after
   initialization; pre-computed time series avoid per-record overhead.
6. **Industry-aware** — Factor sensitivities are calibrated per industry using empirical
   elasticities from economic literature.

## Integration Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Scenario Orchestrator (Spec 06)               │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐   │
│  │Historical │ │Stress    │ │Custom    │ │Climate/ESG       │   │
│  │Replays   │ │Tests     │ │Narrative │ │Scenarios         │   │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────────┬─────────┘   │
│       └─────────────┴────────────┴────────────────┘             │
│                              │                                   │
│              ┌───────────────▼───────────────┐                   │
│              │  Cross-Factor Correlation     │                   │
│              │  Engine (Spec 07)             │                   │
│              │  VAR / Copula / Factor Model  │                   │
│              └───────────────┬───────────────┘                   │
│                              │                                   │
│       ┌──────────────────────┼──────────────────────┐           │
│       ▼                      ▼                      ▼           │
│ ┌──────────┐ ┌───────────────────────┐ ┌──────────────────┐    │
│ │Macro     │ │External Shocks        │ │Supply Chain      │    │
│ │Factor    │ │(Spec 02)              │ │Contagion         │    │
│ │Engine    │ │                       │ │(Spec 05)         │    │
│ │(Spec 01) │ │ ┌─────┐ ┌──────────┐ │ │                  │    │
│ │          │ │ │Geo-  │ │Natural   │ │ │                  │    │
│ │GDP,Rates,│ │ │polit.│ │Disaster  │ │ │Tier propagation  │    │
│ │Inflation,│ │ │Reg.  │ │Climate   │ │ │Bullwhip effect   │    │
│ │Unemploy.,│ │ │(03)  │ │(04)      │ │ │                  │    │
│ │Credit    │ │ └──────┘ └──────────┘ │ │                  │    │
│ └────┬─────┘ └──────────┬────────────┘ └────────┬─────────┘    │
│      └──────────────────┼───────────────────────┘              │
│                         ▼                                       │
│              ┌──────────────────────┐                           │
│              │  Existing Pipeline   │                           │
│              │  DriftAdjustments    │                           │
│              │  + BehavioralDrift   │                           │
│              │  + Seasonality       │                           │
│              └──────────┬───────────┘                           │
│                         ▼                                       │
│              ┌──────────────────────┐                           │
│              │  Generators          │                           │
│              │  (JE, DocFlow, etc.) │                           │
│              └──────────┬───────────┘                           │
│                         ▼                                       │
│              ┌──────────────────────┐                           │
│              │  Realism Validation  │                           │
│              │  (Spec 08)           │                           │
│              └──────────────────────┘                           │
└─────────────────────────────────────────────────────────────────┘
```

## Configuration Preview

```yaml
# Top-level additions to DataSynth config
external_realism:
  enabled: true

  macroeconomic:
    enabled: true
    model: var          # var, independent, scenario_driven
    # ... see Spec 01

  external_shocks:
    enabled: true
    # ... see Spec 02

  geopolitical:
    enabled: true
    # ... see Spec 03

  natural_disasters:
    enabled: true
    # ... see Spec 04

  supply_chain_contagion:
    enabled: true
    # ... see Spec 05

  scenarios:
    enabled: true
    preset: "2008_financial_crisis"
    # ... see Spec 06

  cross_factor_correlation:
    enabled: true
    # ... see Spec 07

  realism_validation:
    enabled: true
    # ... see Spec 08
```

## References (Cross-Cutting)

- Assefa, S. A., Dervovic, D., Mahlich, M., et al. (2020). "Generating Synthetic Data in Finance." *Alan Turing Institute*.
- Park, N., Mohammadi, M., Gorde, K., et al. (2018). "Data Synthesis based on Generative Adversarial Networks." *PVLDB*, 11(10).
- Hamilton, J. D. (1989). "A New Approach to the Economic Analysis of Nonstationary Time Series." *Econometrica*, 57(2), 357-384.
- Sims, C. A. (1980). "Macroeconomics and Reality." *Econometrica*, 48(1), 1-48.
- Nelson, C. R. & Siegel, A. F. (1987). "Parsimonious Modeling of Yield Curves." *Journal of Business*, 60(4), 473-489.
