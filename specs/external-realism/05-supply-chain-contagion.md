# Spec 05: Supply Chain Contagion Network

**Status**: Draft
**Priority**: High
**Depends on**: Spec 02 (External Shock System), existing `vendor_network` config
**Extends**: `datasynth-generators/src/master_data/vendor_generator.rs`, `datasynth-core/src/models/vendor_network.rs`

---

## 1. Problem Statement

The existing `vendor_network` configuration generates tiered supplier networks
(Tier 1/2/3) with clusters and dependency metrics, but disruptions in the current model
are isolated events — a supplier failure doesn't propagate through the network. Real
supply chains exhibit **cascading failures** where a disruption at one node ripples
upstream and downstream, amplified by the **bullwhip effect**. The 2021 semiconductor
shortage, 2020 COVID supply freeze, and 2011 Tohoku earthquake all demonstrated
multi-tier cascade dynamics that current synthetic data cannot replicate.

## 2. Scientific Foundation

### 2.1 Epidemic Contagion Models (SIS/SIR)

Disruption propagation through supply networks can be modeled using compartmental
epidemic models adapted from epidemiology (Dolgui & Ivanov, 2021). Each firm transitions
between states:

- **S** (Susceptible): Operating normally but connected to disrupted nodes
- **I** (Infected/Disrupted): Experiencing operational disruption
- **R** (Recovered): Restored operations with potential immunity/resilience

The SIS model on a network with adjacency matrix **A**:

```
dp_i/dt = -δ_i · p_i + (1 - p_i) · β · Σ_j A_ij · p_j
```

Where:
- `β` = transmission rate (0.05-0.30 per period, depending on coupling tightness)
- `δ` = recovery rate (0.01-0.10 per period)
- `R₀ = β · ρ(A) / δ` — disruption persists when R₀ > 1
- `ρ(A)` = spectral radius of adjacency matrix

**Validated on 21 real supply chain networks** (EJOR, 2022): disruption magnitude depends
strongly on network topology, with upstream-centric networks experiencing lower risk.
Authority weight (HITS algorithm) was the most effective metric for identifying critical
nodes.

**Reference**: Dolgui, A. & Ivanov, D. (2021). "Ripple Effect and Supply Chain
Disruption Management." *International Journal of Production Research*.

### 2.2 Cascading Failure (Load-Capacity) Model

Each node `i` has load `L_i` and capacity `C_i = (1 + α) · L_i⁰`, where α is the
tolerance parameter. When node `i` fails, its load redistributes:

```
L_j_new = L_j + L_i · (k_j / Σ_neighbors k_n)
```

If `L_j_new > C_j`, node `j` fails, triggering the cascade. The tolerance parameter
α ∈ [0.1, 0.5] determines network fragility — lower values mean higher cascade risk.

**Key finding**: Barabási-Albert scale-free networks (power-law degree distribution
with γ ≈ 2.5-3.0) closely model real supply chain topology and exhibit characteristic
"hub vulnerability" — targeted attacks on high-degree nodes cause disproportionate
cascade damage.

**Reference**: Motter, A.E. & Lai, Y.C. (2002). "Cascade-Based Attacks on Complex
Networks." *Physical Review E*, 66(6), 065102.

### 2.3 Bullwhip Effect Quantification

The bullwhip effect amplifies demand variability upstream through the supply chain
(Chen et al., 2000). The variance amplification ratio:

```
BWE = Var(Orders_upstream) / Var(Demand_downstream)
```

For a chain with moving average forecasting over `p` periods and lead time `L`:

```
BWE ≥ 1 + 2L/p + 2L²/p²
```

For exponential smoothing with parameter `α_s`:

```
BWE ≥ 1 + 2α_s·L + (2α_s²·L²) / (2 - α_s)²
```

Empirical amplification: **1.5-5.0x** across 3-5 tiers in real supply chains.
Revenue loss per $1 at disrupted firm: **$2.40 loss at customer firms**.

**Reference**: Chen, F., Drezner, Z., Ryan, J.K., & Simchi-Levi, D. (2000).
"Quantifying the Bullwhip Effect in a Simple Supply Chain." *Management Science*,
46(3), 436-443.

### 2.4 Financial Impact Calibration

Empirical data from large-scale studies:

| Metric | Value | Source |
|--------|-------|--------|
| Revenue decline per disruption event | 6-20% | The Economist |
| EBITDA impact (short disruption, <30d) | 3-5% | McKinsey |
| EBITDA impact (prolonged disruption) | 30-50% annual | McKinsey |
| Recovery time (production restart) | ~50 trading days | Hendricks & Singhal |
| Post-disruption underperformance | Up to 2 years | Hendricks & Singhal |
| Inventory increase (resilience response) | +47% for critical items | Executive surveys |

**Reference**: Hendricks, K.B. & Singhal, V.R. (2005). "An Empirical Analysis of the
Effect of Supply Chain Disruptions on Long-Run Stock Price Performance." *Production
and Operations Management*, 14(1), 35-52.

## 3. Proposed Design

### 3.1 Supply Chain Network Graph

```rust
/// Supply chain network with contagion dynamics
#[derive(Debug, Clone)]
pub struct SupplyChainNetwork {
    /// Network graph (adjacency list with edge weights)
    graph: SupplyChainGraph,
    /// Node state (susceptible, disrupted, recovered)
    node_states: HashMap<NodeId, NodeState>,
    /// Contagion model parameters
    contagion: ContagionParams,
    /// Bullwhip effect parameters
    bullwhip: BullwhipParams,
    /// Financial impact calculator
    financial_impact: FinancialImpactModel,
}

#[derive(Debug, Clone)]
pub struct SupplyChainGraph {
    /// Nodes (suppliers, manufacturers, distributors, customers)
    pub nodes: Vec<SupplyChainNode>,
    /// Directed edges (supply relationships)
    pub edges: Vec<SupplyEdge>,
    /// Tier structure
    pub tiers: HashMap<NodeId, SupplyTier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyChainNode {
    pub id: NodeId,
    pub node_type: SupplyNodeType,
    /// Geographic region
    pub region: String,
    /// Industry/sector
    pub sector: IndustrySector,
    /// Criticality score (0.0-1.0)
    pub criticality: f64,
    /// Resilience score (0.0-1.0) — affects recovery speed
    pub resilience: f64,
    /// Current operational load (normalized)
    pub load: f64,
    /// Maximum capacity (load * (1 + tolerance))
    pub capacity: f64,
    /// Single-source dependency flag
    pub single_source: bool,
    /// Inventory buffer days (safety stock coverage)
    pub inventory_buffer_days: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SupplyNodeType {
    RawMaterialSupplier,
    ComponentManufacturer,
    SubassemblyMaker,
    FinalAssembler,
    Distributor,
    Retailer,
    ServiceProvider,
    LogisticsProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyEdge {
    pub from: NodeId,
    pub to: NodeId,
    /// Dependency strength (0.0-1.0) — fraction of input from this source
    pub dependency_weight: f64,
    /// Lead time (days)
    pub lead_time_days: f64,
    /// Substitutability (0.0 = irreplaceable, 1.0 = easily substituted)
    pub substitutability: f64,
    /// Geographic distance factor (affects transport disruption)
    pub geographic_risk: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SupplyTier {
    /// Direct suppliers
    Tier1,
    /// Suppliers' suppliers
    Tier2,
    /// Third-level suppliers
    Tier3,
    /// The focal company
    FocalCompany,
    /// Downstream customers
    Customer,
}
```

### 3.2 Contagion Dynamics Engine

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContagionParams {
    /// Base transmission probability per period per edge
    pub beta: f64,
    /// Recovery rate per period
    pub delta: f64,
    /// Model type
    pub model: ContagionModel,
    /// Maximum cascade depth (limits computation)
    pub max_cascade_depth: usize,
    /// Minimum disruption magnitude to propagate
    pub propagation_threshold: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ContagionModel {
    /// Susceptible-Infected-Susceptible (can be re-disrupted)
    SIS,
    /// Susceptible-Infected-Recovered (temporary immunity)
    SIR { immunity_periods: usize },
    /// Load redistribution cascading failure
    CascadingFailure { tolerance: f64 },
    /// Hybrid: SIS for gradual spread + cascade for sudden failures
    Hybrid { cascade_threshold: f64 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NodeState {
    /// Operating normally
    Susceptible,
    /// Disrupted — reduced or zero output
    Disrupted {
        severity: f64,             // 0.0 = minor, 1.0 = complete shutdown
        disrupted_since: usize,    // Period when disruption started
        source: DisruptionSource,  // Direct shock or contagion
    },
    /// Recovered — operational with potential residual effects
    Recovered {
        recovered_since: usize,
        residual_capacity_loss: f64,
    },
    /// Permanently failed (exited network)
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DisruptionSource {
    /// Direct external shock (natural disaster, sanctions, etc.)
    DirectShock,
    /// Upstream supply disruption (supplier failed/disrupted)
    UpstreamContagion { depth: usize },
    /// Downstream demand disruption (customer failed/disrupted)
    DownstreamContagion,
    /// Overload from load redistribution
    CascadeOverload,
}
```

### 3.3 Contagion Simulation

```rust
impl SupplyChainNetwork {
    /// Simulate one period of contagion dynamics
    pub fn step(&mut self, period: usize, rng: &mut ChaCha8Rng) {
        match self.contagion.model {
            ContagionModel::SIS => self.step_sis(period, rng),
            ContagionModel::SIR { .. } => self.step_sir(period, rng),
            ContagionModel::CascadingFailure { tolerance } => {
                self.step_cascade(period, tolerance, rng)
            }
            ContagionModel::Hybrid { cascade_threshold } => {
                self.step_sis(period, rng);
                // If any node reaches high severity, trigger cascade check
                let high_severity_nodes: Vec<_> = self.node_states.iter()
                    .filter(|(_, state)| matches!(state,
                        NodeState::Disrupted { severity, .. } if *severity > cascade_threshold
                    ))
                    .map(|(id, _)| *id)
                    .collect();
                for node_id in high_severity_nodes {
                    self.propagate_cascade(node_id, 0);
                }
            }
        }
    }

    /// SIS contagion step
    fn step_sis(&mut self, period: usize, rng: &mut ChaCha8Rng) {
        let mut new_states = self.node_states.clone();

        for node in &self.graph.nodes {
            match self.node_states[&node.id] {
                NodeState::Susceptible => {
                    // Check for infection from disrupted neighbors
                    let infection_pressure: f64 = self.graph.edges.iter()
                        .filter(|e| e.to == node.id) // incoming supply edges
                        .filter_map(|e| {
                            if let NodeState::Disrupted { severity, .. } = self.node_states[&e.from] {
                                Some(severity * e.dependency_weight * (1.0 - e.substitutability))
                            } else {
                                None
                            }
                        })
                        .sum();

                    let infection_prob = 1.0 - (-self.contagion.beta * infection_pressure).exp();
                    if rng.gen::<f64>() < infection_prob {
                        let severity = (infection_pressure * 0.7).min(1.0);
                        new_states.insert(node.id, NodeState::Disrupted {
                            severity,
                            disrupted_since: period,
                            source: DisruptionSource::UpstreamContagion {
                                depth: self.compute_cascade_depth(node.id),
                            },
                        });
                    }
                }
                NodeState::Disrupted { severity, disrupted_since, .. } => {
                    // Check for recovery
                    let recovery_prob = self.contagion.delta * node.resilience;
                    if rng.gen::<f64>() < recovery_prob {
                        let residual = severity * 0.05; // 5% residual capacity loss
                        new_states.insert(node.id, NodeState::Recovered {
                            recovered_since: period,
                            residual_capacity_loss: residual,
                        });
                    }
                }
                _ => {}
            }
        }

        self.node_states = new_states;
    }

    /// Inject an external shock into specific nodes
    pub fn inject_shock(
        &mut self,
        affected_nodes: &[NodeId],
        severity: f64,
        period: usize,
    ) {
        for node_id in affected_nodes {
            self.node_states.insert(*node_id, NodeState::Disrupted {
                severity,
                disrupted_since: period,
                source: DisruptionSource::DirectShock,
            });
        }
    }
}
```

### 3.4 Bullwhip Effect Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BullwhipParams {
    /// Forecasting method used by firms
    pub forecasting_method: ForecastingMethod,
    /// Batch ordering behavior
    pub order_batching: OrderBatchingConfig,
    /// Lead time variability (coefficient of variation)
    pub lead_time_cv: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ForecastingMethod {
    /// Moving average with window size p
    MovingAverage { window: usize },
    /// Exponential smoothing with parameter α
    ExponentialSmoothing { alpha: f64 },
    /// No forecasting (pass-through)
    PassThrough,
}

impl BullwhipParams {
    /// Compute the variance amplification ratio for a given tier
    pub fn amplification_ratio(&self, tier_depth: usize, lead_time: f64) -> f64 {
        let base_bwe = match self.forecasting_method {
            ForecastingMethod::MovingAverage { window } => {
                let p = window as f64;
                let l = lead_time;
                1.0 + (2.0 * l / p) + (2.0 * l * l / (p * p))
            }
            ForecastingMethod::ExponentialSmoothing { alpha } => {
                let l = lead_time;
                1.0 + (2.0 * alpha * l)
                    + (2.0 * alpha * alpha * l * l) / ((2.0 - alpha).powi(2))
            }
            ForecastingMethod::PassThrough => 1.0,
        };

        // Compound amplification across tiers
        base_bwe.powi(tier_depth as i32)
    }

    /// Compute order quantity variance for a node given downstream demand variance
    pub fn amplified_variance(
        &self,
        demand_variance: f64,
        tier_depth: usize,
        lead_time: f64,
    ) -> f64 {
        demand_variance * self.amplification_ratio(tier_depth, lead_time)
    }
}
```

### 3.5 Financial Impact Translation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialImpactModel {
    /// Revenue impact per unit of disruption severity
    pub revenue_sensitivity: f64,
    /// COGS impact from supply disruption
    pub cogs_sensitivity: f64,
    /// Expediting cost multiplier for rush orders
    pub expediting_cost_multiplier: f64,
    /// Inventory holding cost increase from safety stock buildup
    pub safety_stock_cost_multiplier: f64,
}

impl FinancialImpactModel {
    /// Translate network disruption state to generator-level adjustments
    pub fn compute_adjustments(
        &self,
        focal_company: &NodeId,
        network: &SupplyChainNetwork,
        period: usize,
    ) -> SupplyChainAdjustments {
        // Upstream disruption: affects procurement
        let upstream_disruption = network.upstream_disruption_level(focal_company);
        // Downstream disruption: affects demand
        let downstream_disruption = network.downstream_disruption_level(focal_company);

        SupplyChainAdjustments {
            procurement_volume_multiplier: 1.0 - upstream_disruption * self.revenue_sensitivity,
            procurement_cost_multiplier: 1.0 + upstream_disruption * self.cogs_sensitivity,
            sales_volume_multiplier: 1.0 - downstream_disruption * 0.5,
            lead_time_multiplier: 1.0 + upstream_disruption * 2.0,
            expediting_cost_active: upstream_disruption > 0.3,
            safety_stock_multiplier: 1.0 + upstream_disruption * self.safety_stock_cost_multiplier,
            supplier_substitution_active: upstream_disruption > 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SupplyChainAdjustments {
    pub procurement_volume_multiplier: f64,
    pub procurement_cost_multiplier: f64,
    pub sales_volume_multiplier: f64,
    pub lead_time_multiplier: f64,
    pub expediting_cost_active: bool,
    pub safety_stock_multiplier: f64,
    pub supplier_substitution_active: bool,
}
```

## 4. Configuration Schema

```yaml
external_realism:
  supply_chain_contagion:
    enabled: true

    # Network topology generation
    network:
      # Use existing vendor_network config or generate new
      source: vendor_network              # vendor_network | generated | custom
      # For generated networks:
      generated:
        topology: barabasi_albert          # barabasi_albert | erdos_renyi | small_world
        total_nodes: 200
        edges_per_new_node: 3              # For BA model
        power_law_exponent: 2.8
        tier_distribution:
          tier1: 0.25
          tier2: 0.40
          tier3: 0.30
          logistics: 0.05

    # Contagion model
    contagion:
      model: hybrid                        # sis | sir | cascading_failure | hybrid
      beta: 0.15                           # Transmission rate per period
      delta: 0.05                          # Recovery rate per period
      max_cascade_depth: 5
      propagation_threshold: 0.2           # Min severity to propagate

      # SIR immunity (if model = sir)
      immunity_periods: 12

      # Cascading failure tolerance (if model = cascading_failure or hybrid)
      cascade_tolerance: 0.3
      cascade_threshold: 0.7               # Severity threshold for cascade trigger

    # Bullwhip effect
    bullwhip:
      enabled: true
      forecasting_method:
        exponential_smoothing:
          alpha: 0.3
      lead_time_cv: 0.25                   # Lead time coefficient of variation

    # Node resilience distribution
    resilience:
      distribution: beta                   # beta | uniform | fixed
      beta_params: { alpha: 3.0, beta: 2.0 }  # Skewed toward higher resilience
      single_source_penalty: 0.3           # Resilience reduction for single-sourced

    # Financial impact parameters
    financial_impact:
      revenue_sensitivity: 0.8
      cogs_sensitivity: 0.5
      expediting_cost_multiplier: 1.5
      safety_stock_cost_multiplier: 0.47   # 47% inventory increase

    # Disruption scenarios (linked to Spec 02 external shocks)
    initial_disruptions: []                # Can also be triggered by shock system

    # Output
    export_network_graph: true             # Export network as edge list
    export_contagion_timeline: true        # Export state transitions per period
    export_disruption_metrics: true        # Export aggregate disruption levels
```

## 5. Integration with Existing Vendor Network

The module enriches the existing `vendor_network` config:

```rust
/// Bridge between existing VendorNetwork and contagion model
impl SupplyChainNetwork {
    pub fn from_vendor_network(
        vendor_network: &VendorNetwork,
        config: &SupplyChainContagionConfig,
    ) -> Self {
        // Map VendorRelationships to SupplyEdges
        let edges: Vec<SupplyEdge> = vendor_network.relationships.iter()
            .map(|rel| SupplyEdge {
                from: rel.vendor_id.into(),
                to: rel.customer_id.into(),
                dependency_weight: rel.spend_share,
                lead_time_days: rel.avg_lead_time_days,
                substitutability: 1.0 - rel.strategic_importance,
                geographic_risk: compute_geo_risk(&rel.vendor_region, &rel.customer_region),
            })
            .collect();

        // Map Vendors to SupplyChainNodes
        let nodes: Vec<SupplyChainNode> = vendor_network.vendors.iter()
            .map(|v| SupplyChainNode {
                id: v.id.into(),
                node_type: tier_to_node_type(&v.tier),
                region: v.region.clone(),
                sector: v.sector,
                criticality: v.quality_score.overall,
                resilience: compute_resilience(v, config),
                load: v.current_utilization,
                capacity: v.current_utilization * (1.0 + config.contagion.cascade_tolerance),
                single_source: v.is_sole_source,
                inventory_buffer_days: v.safety_stock_days,
            })
            .collect();

        Self::new(nodes, edges, config)
    }
}
```

## 6. Integration with External Shocks (Spec 02)

External shocks from Spec 02 can trigger supply chain disruptions:

```rust
impl ShockSequencer {
    /// When a supply chain shock fires, inject it into the network
    pub fn inject_into_network(
        &self,
        shock: &ExternalShock,
        network: &mut SupplyChainNetwork,
        period: usize,
    ) {
        match &shock.shock_type {
            ExternalShockType::SupplyChain(variant) => {
                let (affected_nodes, severity) = match variant {
                    SupplyShockVariant::RegionalDisruption { region, severity } => {
                        let nodes = network.nodes_in_region(region);
                        (nodes, *severity)
                    }
                    SupplyShockVariant::SingleSupplierFailure { supplier_id } => {
                        let node = supplier_id.as_ref()
                            .map(|id| vec![id.clone().into()])
                            .unwrap_or_else(|| network.random_critical_node());
                        (node, 1.0)
                    }
                    // ... other variants
                    _ => return,
                };
                network.inject_shock(&affected_nodes, severity, period);
            }
            ExternalShockType::NaturalDisaster(_) => {
                // Natural disasters disrupt nodes in the affected region
                let affected_region = shock.region();
                let nodes = network.nodes_in_region(&affected_region);
                let severity = shock.peak_severity();
                network.inject_shock(&nodes, severity, period);
            }
            _ => {}
        }
    }
}
```

## 7. Testing Strategy

### Unit Tests
- **SIS convergence**: With R₀ < 1, disruption dies out; with R₀ > 1, it persists
- **Cascade bounds**: Cascade respects max_cascade_depth limit
- **Bullwhip ratio**: Computed BWE matches analytical formula ± 1%
- **Node recovery**: Recovered nodes return to Susceptible (SIS) or stay immune (SIR)

### Integration Tests
- **End-to-end propagation**: Regional shock → network injection → contagion → financial impact
- **Tier amplification**: Tier 3 disruption > Tier 2 > Tier 1 (bullwhip)
- **Single-source vulnerability**: Single-sourced nodes have higher cascade impact
- **Recovery timeline**: Network returns to baseline within configured recovery periods

### Statistical Tests
- **Disruption frequency**: Poisson-distributed when driven by stochastic events
- **Impact distribution**: Financial impact follows heavy-tailed distribution
- **Network degree distribution**: BA-generated networks follow power law

## 8. Performance Considerations

- **Network simulation per period**: O(E) where E = edges (~1,000-10,000 for typical networks)
- **Cascade propagation**: O(N · D) where N = nodes, D = max cascade depth
- **Memory**: ~500 bytes per node, ~100 bytes per edge
- **Total for 200-node network, 120 periods**: ~2.4M operations (negligible)

## References

1. Dolgui, A. & Ivanov, D. (2021). "Ripple Effect and Supply Chain Disruption Management." *IJPR*.
2. Chen, F., Drezner, Z., Ryan, J.K., & Simchi-Levi, D. (2000). "Quantifying the Bullwhip Effect." *Management Science*, 46(3), 436-443.
3. Motter, A.E. & Lai, Y.C. (2002). "Cascade-Based Attacks on Complex Networks." *Physical Review E*, 66(6), 065102.
4. Hendricks, K.B. & Singhal, V.R. (2005). "Supply Chain Disruptions and Long-Run Stock Price Performance." *POM*, 14(1), 35-52.
5. Acemoglu, D., Carvalho, V.M., Ozdaglar, A., & Tahbaz-Salehi, A. (2012). "The Network Origins of Aggregate Fluctuations." *Econometrica*, 80(5), 1977-2016.
6. Barabási, A.L. & Albert, R. (1999). "Emergence of Scaling in Random Networks." *Science*, 286(5439), 509-512.
7. Craighead, C.W., Blackhurst, J., Rungtusanatham, M.J., & Handfield, R.B. (2007). "The Severity of Supply Chain Disruptions." *Decision Sciences*, 38(1), 131-156.
