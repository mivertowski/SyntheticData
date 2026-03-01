# Spec 03: Geopolitical & Regulatory Risk Module

**Status**: Draft
**Priority**: High
**Depends on**: Spec 01 (Macro Factor Engine), Spec 02 (External Shock System)
**Extends**: `market_drift.rs` (GeopoliticalEvent shock type)

---

## 1. Problem Statement

Enterprise financial data is profoundly affected by geopolitical events (wars, sanctions,
trade policy changes, political instability) and regulatory shifts (new compliance
requirements, tax reforms, accounting standard changes). The current framework has only
a generic `GeopoliticalEvent` shock type in `market_drift.rs` with no structured model
for how these events propagate through enterprise financials. Real-world data shows
distinct fingerprints — sanctions create counterparty gaps, trade wars shift sourcing
patterns, and regulatory changes create compliance cost step-functions.

## 2. Scientific Foundation

### 2.1 Geopolitical Risk Index (GPR)

Caldara & Iacoviello (2022) construct the GPR index by counting newspaper articles
related to geopolitical tensions. Key empirical findings:

- A 1-standard-deviation GPR increase → **investment declines 1.5%** over 4 quarters
- **Employment declines 0.5%** with a 2-quarter lag
- **Stock market declines 2-3%** within the same quarter
- GPR spikes are associated with **capital flight** from emerging markets to safe havens
- The index distinguishes **GPR Threats** (verbal threats, escalation) from **GPR Acts**
  (actual military/terrorist events)

For synthetic data, the GPR level modulates uncertainty premiums, investment decisions,
and cross-border transaction volumes.

**Reference**: Caldara, D. & Iacoviello, M. (2022). "Measuring Geopolitical Risk."
*American Economic Review*, 112(4), 1194-1225.

### 2.2 Economic Policy Uncertainty (EPU) Index

Baker, Bloom, & Davis (2016) measure policy uncertainty through newspaper coverage,
tax code provisions set to expire, and forecaster disagreement. Key findings:

- EPU spikes precede **GDP growth declines of 0.5-1.5%** within 6 months
- High EPU → firms **delay hiring by 3-6 months** and **reduce capex by 5-15%**
- EPU affects **working capital**: firms hold 8-12% more cash during high-EPU periods
- Sector sensitivity varies: **financials** and **industrials** most affected; **utilities**
  and **healthcare** least

**Reference**: Baker, S.R., Bloom, N., & Davis, S.J. (2016). "Measuring Economic
Policy Uncertainty." *Quarterly Journal of Economics*, 131(4), 1593-1636.

### 2.3 Sanctions Impact Research

Sanctions create measurable, structured financial impacts:

- **Primary sanctions**: Direct trade prohibition → immediate revenue loss from sanctioned
  counterparties (Hufbauer et al., 2007)
- **Secondary sanctions**: Compliance burden on third parties → increased KYC/AML costs,
  de-risking behavior
- **Sectoral sanctions**: Targeted at specific industries (energy, finance, technology) →
  asymmetric industry impacts
- **Duration**: Average sanctions episode lasts **7.3 years** (Peterson Institute data)
- **GDP impact on target**: Median **-2.4% of GDP** (Neuenkirch & Neumeier, 2015)
- **Compliance costs**: Estimated 3-5% of operating costs for affected financial institutions

**Reference**: Hufbauer, G.C., Schott, J.J., Elliott, K.A., & Oegg, B. (2007).
*Economic Sanctions Reconsidered*. 3rd ed. Peterson Institute.

### 2.4 Regulatory Change Cost Functions

Regulatory implementations follow a characteristic cost curve:

```
Cost(t) = C_fixed · pulse(t₀) + C_ongoing · step(t₀) + C_transition · bell(t₀, σ)
```

Where:
- `C_fixed` = one-time implementation cost (systems, training, consulting)
- `C_ongoing` = permanent incremental compliance cost
- `C_transition` = temporary elevated cost during transition period
- `pulse(t₀)` = Dirac delta at implementation date
- `step(t₀)` = Heaviside step function
- `bell(t₀, σ)` = Gaussian transition cost centered on go-live

Empirical examples:
- **Basel III implementation**: ~$70B compliance costs for top-20 banks; 3-5 year transition
- **IFRS 9/CECL adoption**: 15-30% increase in loan loss provisions at adoption
- **SOX 404 compliance**: Average $1.7M annual cost for mid-cap companies (SEC estimates)
- **GDPR implementation**: Median €1.3M for large enterprises (IAPP survey)

## 3. Proposed Design

### 3.1 Geopolitical Event Taxonomy

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeopoliticalEvent {
    /// International sanctions imposed/lifted
    Sanctions(SanctionsEvent),
    /// Trade policy changes (tariffs, quotas, embargoes)
    TradePolicy(TradePolicyEvent),
    /// Armed conflict or military escalation
    ArmedConflict(ConflictEvent),
    /// Political regime change or instability
    PoliticalInstability(PoliticalEvent),
    /// Terrorism or security event
    SecurityEvent(SecurityEventType),
    /// International agreement or treaty
    Treaty(TreatyEvent),
    /// Currency/capital controls imposed or removed
    CapitalControls(CapitalControlEvent),
    /// Election with policy-relevant outcome
    Election(ElectionEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionsEvent {
    /// Target country or entity group
    pub target: SanctionsTarget,
    /// Imposing authority (US OFAC, EU, UN, etc.)
    pub authority: String,
    /// Type of sanctions
    pub sanctions_type: SanctionsType,
    /// Affected sectors
    pub affected_sectors: Vec<IndustrySector>,
    /// Estimated compliance cost multiplier
    pub compliance_cost_multiplier: f64,
    /// Whether secondary sanctions apply (extraterritorial)
    pub secondary_sanctions: bool,
    /// Duration estimate (months, None = indefinite)
    pub estimated_duration_months: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SanctionsType {
    /// Full economic embargo
    Comprehensive,
    /// Targeted at specific sectors
    Sectoral { sectors: Vec<String> },
    /// Targeted at specific entities (SDN list)
    EntityBased { entity_count: u32 },
    /// Financial sanctions (asset freeze, payment restrictions)
    Financial,
    /// Technology export controls
    TechnologyExport,
    /// Travel bans and diplomatic sanctions
    Diplomatic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradePolicyEvent {
    /// Type of trade policy change
    pub policy_type: TradePolicyType,
    /// Affected trade corridors (country pairs)
    pub affected_corridors: Vec<TradeCorridorImpact>,
    /// Affected product categories
    pub affected_products: Vec<String>,
    /// Implementation timeline (gradual vs. immediate)
    pub implementation: TradeImplementation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradePolicyType {
    /// New tariff or tariff increase
    TariffIncrease { rate_pct: f64 },
    /// Tariff reduction or elimination
    TariffReduction { rate_pct: f64 },
    /// Import/export quota
    Quota { limit_pct_of_current: f64 },
    /// Trade embargo
    Embargo,
    /// Free trade agreement
    FreeTradeAgreement,
    /// Content/origin requirements
    LocalContentRequirement { threshold_pct: f64 },
    /// Anti-dumping duties
    AntiDumping { target_country: String, rate_pct: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeCorridorImpact {
    pub origin_region: String,
    pub destination_region: String,
    /// Percentage of entity's trade volume through this corridor
    pub trade_share: f64,
    /// Impact multiplier on costs
    pub cost_impact: f64,
    /// Impact on lead times (multiplier)
    pub lead_time_impact: f64,
}
```

### 3.2 Regulatory Change Framework

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegulatoryChange {
    /// Accounting standard change
    AccountingStandard(AccountingRegChange),
    /// Financial regulation (capital, liquidity, conduct)
    FinancialRegulation(FinRegChange),
    /// Tax policy change
    TaxReform(TaxReformEvent),
    /// Data protection / privacy regulation
    DataProtection(DataProtectionEvent),
    /// Environmental / ESG regulation
    EnvironmentalRegulation(EnvRegEvent),
    /// Labor / employment law change
    LaborRegulation(LaborRegEvent),
    /// Industry-specific regulation
    IndustryRegulation(IndustryRegEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingRegChange {
    pub standard: AccountingStandardChange,
    /// Transition method
    pub transition: TransitionMethod,
    /// Impact on financial statements
    pub balance_sheet_impact: Option<BalanceSheetImpact>,
    /// Implementation timeline
    pub effective_date_period: usize,
    pub early_adoption_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountingStandardChange {
    /// New revenue recognition (ASC 606 / IFRS 15)
    RevenueRecognition,
    /// Lease accounting (ASC 842 / IFRS 16)
    LeaseAccounting { rou_asset_increase_pct: f64 },
    /// Credit loss methodology (CECL / IFRS 9)
    ExpectedCreditLoss { provision_increase_pct: f64 },
    /// Fair value measurement changes
    FairValueChanges,
    /// Consolidation scope changes
    ConsolidationChanges,
    /// Custom standard change
    Custom { name: String, description: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxReformEvent {
    /// Type of tax change
    pub reform_type: TaxReformType,
    /// Effective period
    pub effective_period: usize,
    /// Transition provisions
    pub transition_periods: usize,
    /// Impact on effective tax rate
    pub etr_change_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaxReformType {
    /// Corporate rate change
    CorporateRateChange { old_rate: f64, new_rate: f64 },
    /// International tax reform (BEPS, Pillar Two)
    InternationalTaxReform { min_rate: f64 },
    /// Transfer pricing enforcement
    TransferPricingEnforcement { adjustment_risk_pct: f64 },
    /// R&D incentive change
    RdIncentiveChange { credit_rate_change: f64 },
    /// Carbon tax introduction/change
    CarbonTax { rate_per_ton: f64 },
    /// Digital services tax
    DigitalServicesTax { rate: f64, threshold: f64 },
}
```

### 3.3 Regulatory Impact Cost Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryImpactModel {
    /// One-time implementation costs
    pub implementation_costs: ImplementationCosts,
    /// Ongoing compliance costs
    pub ongoing_costs: OngoingCosts,
    /// Transition period dynamics
    pub transition: TransitionDynamics,
    /// Impact on financial statements
    pub financial_impact: FinancialStatementImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationCosts {
    /// System changes and IT costs (as % of revenue)
    pub system_cost_pct: f64,
    /// Training and consulting costs
    pub training_cost_pct: f64,
    /// Process redesign costs
    pub process_cost_pct: f64,
    /// Spread over N periods
    pub capitalized_over_periods: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionDynamics {
    /// Periods before effective date where preparation begins
    pub preparation_lead_periods: usize,
    /// Peak error rate during transition (multiplier over baseline)
    pub peak_error_multiplier: f64,
    /// How many periods until error rate normalizes
    pub learning_curve_periods: usize,
    /// Parallel run required (dual reporting)?
    pub parallel_run_periods: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialStatementImpact {
    /// Balance sheet reclassifications
    pub reclassifications: Vec<Reclassification>,
    /// P&L impact in transition period
    pub transition_pnl_impact_pct: f64,
    /// Equity adjustment (retained earnings impact)
    pub equity_adjustment_pct: f64,
    /// New line items or disclosures required
    pub new_disclosures: Vec<String>,
}
```

### 3.4 Geopolitical Uncertainty Index (Internal)

The module maintains a composite uncertainty index that modulates enterprise behavior:

```rust
pub struct GeopoliticalUncertaintyIndex {
    /// Base uncertainty level (0 = calm, 1 = extreme uncertainty)
    base_level: f64,
    /// Contributions from active events
    event_contributions: Vec<(String, f64)>,
    /// Time-weighted decay of past events
    memory_half_life_periods: usize,
}

impl GeopoliticalUncertaintyIndex {
    /// Compute current uncertainty level
    pub fn current_level(&self, period: usize) -> f64 {
        let event_sum: f64 = self.event_contributions.iter()
            .map(|(_, contribution)| contribution)
            .sum();
        (self.base_level + event_sum).min(1.0)
    }

    /// Behavioral effects of uncertainty
    pub fn behavioral_effects(&self, period: usize) -> UncertaintyEffects {
        let level = self.current_level(period);
        UncertaintyEffects {
            // Investment and hiring freeze at high uncertainty
            capex_multiplier: 1.0 - 0.15 * level,      // Up to 15% capex reduction
            hiring_multiplier: 1.0 - 0.05 * level,      // Up to 5% hiring reduction
            cash_hoarding_multiplier: 1.0 + 0.12 * level, // Up to 12% more cash held
            // Working capital tightening
            payment_acceleration_factor: 1.0 + 0.08 * level,
            // Inventory buildup (precautionary)
            inventory_buffer_multiplier: 1.0 + 0.20 * level,
            // Increased audit/control scrutiny
            control_strictness_multiplier: 1.0 + 0.10 * level,
        }
    }
}
```

## 4. Configuration Schema

```yaml
external_realism:
  geopolitical:
    enabled: true

    # Background uncertainty level (0-1)
    base_uncertainty: 0.15

    # Geopolitical events
    events:
      - type: sanctions
        target: { country: "RU" }
        authority: "US_OFAC"
        sanctions_type: sectoral
        affected_sectors: [energy, financial_services, technology]
        compliance_cost_multiplier: 1.04   # 4% increase in compliance costs
        secondary_sanctions: true
        onset_period: 12
        estimated_duration_months: 36
        impact:
          counterparty_loss_pct: 0.08       # 8% of counterparties affected
          trade_volume_decline_pct: 0.15    # 15% trade decline with target
          kyc_cost_increase_pct: 0.25       # 25% more KYC costs

      - type: trade_policy
        policy_type:
          tariff_increase:
            rate_pct: 0.25                   # 25% tariff
        affected_corridors:
          - origin: "CN"
            destination: "US"
            trade_share: 0.30
            cost_impact: 1.25
            lead_time_impact: 1.15
        onset_period: 18
        implementation: phased              # immediate | phased

    # Regulatory changes
    regulatory_changes:
      - type: accounting_standard
        standard:
          expected_credit_loss:
            provision_increase_pct: 0.25
        effective_date_period: 24
        transition:
          preparation_lead_periods: 12
          peak_error_multiplier: 1.8
          learning_curve_periods: 6
          parallel_run_periods: 3
        financial_impact:
          equity_adjustment_pct: -0.03       # 3% retained earnings hit

      - type: tax_reform
        reform:
          international_tax_reform:
            min_rate: 0.15                   # Pillar Two minimum rate
        effective_period: 36
        etr_change_pct: 0.02                # 2% effective rate increase
        transition_periods: 6

    # Uncertainty memory
    memory_half_life_periods: 12             # How long uncertainty lingers
```

## 5. Pre-Built Geopolitical Scenarios

### 5.1 Trade War Escalation

```yaml
scenario: trade_war_escalation
events:
  - type: trade_policy
    policy_type: { tariff_increase: { rate_pct: 0.10 } }
    onset_period: 6
    affected_products: ["manufactured_goods"]
  - type: trade_policy
    policy_type: { tariff_increase: { rate_pct: 0.25 } }
    onset_period: 12                          # Retaliatory escalation
  - type: trade_policy
    policy_type: { tariff_increase: { rate_pct: 0.50 } }
    onset_period: 18                          # Full escalation
effects:
  sourcing_shift_rate: 0.15                   # 15% of suppliers change per escalation
  inventory_buildup_periods: 3                # Pre-tariff hoarding
  cost_pass_through: 0.70                     # 70% of tariff passed to prices
```

### 5.2 Sanctions Regime

```yaml
scenario: comprehensive_sanctions
events:
  - type: sanctions
    target: { country: "TARGET" }
    sanctions_type: comprehensive
    onset_period: 12
    counterparty_exposure_pct: 0.12
effects:
  immediate_counterparty_loss: true
  payment_freeze_periods: 2
  compliance_cost_surge: 1.08
  kyc_enhanced_due_diligence: true
  correspondent_banking_restrictions: true
```

### 5.3 Regulatory Wave

```yaml
scenario: regulatory_convergence
regulatory_changes:
  - type: accounting_standard
    standard: { lease_accounting: { rou_asset_increase_pct: 0.06 } }
    effective_date_period: 12
  - type: financial_regulation
    regulation: { capital_requirements: { increase_pct: 0.015 } }
    effective_date_period: 24
  - type: data_protection
    regulation: { type: "gdpr_equivalent", fine_risk_pct: 0.04 }
    effective_date_period: 18
  - type: tax_reform
    reform: { carbon_tax: { rate_per_ton: 50.0 } }
    effective_date_period: 36
```

## 6. Downstream Effects

| Event Type | Primary Effect | Secondary Effects |
|-----------|---------------|-------------------|
| Sanctions | Counterparty loss, payment blocks | Sourcing shift, compliance costs, KYC burden |
| Tariffs | COGS increase, margin pressure | Supply chain reconfiguration, inventory buildup |
| Armed conflict | Trade route disruption | Energy prices, refugee labor, insurance costs |
| Tax reform | ETR change | Transfer pricing adjustment, legal restructuring |
| Accounting standard | Balance sheet reclassification | Audit costs, system changes, parallel reporting |
| GDPR/privacy | Compliance system costs | Data handling processes, consent management |

## 7. Testing Strategy

- **Sanctions effect**: Counterparty transactions drop to zero after sanctions onset
- **Tariff cost pass-through**: Verify COGS increases proportional to tariff × exposure
- **Regulatory cost curve**: Implementation costs follow bell curve, ongoing costs step up
- **Uncertainty behavioral effects**: High uncertainty → capex decline, cash increase
- **Multi-event interaction**: Simultaneous sanctions + tariff produces compound effects
- **Duration bounds**: Sanctions last within configured duration estimate

## References

1. Caldara, D. & Iacoviello, M. (2022). "Measuring Geopolitical Risk." *American Economic Review*, 112(4), 1194-1225.
2. Baker, S.R., Bloom, N., & Davis, S.J. (2016). "Measuring Economic Policy Uncertainty." *Quarterly Journal of Economics*, 131(4), 1593-1636.
3. Hufbauer, G.C., Schott, J.J., Elliott, K.A., & Oegg, B. (2007). *Economic Sanctions Reconsidered*. 3rd ed. Peterson Institute.
4. Neuenkirch, M. & Neumeier, F. (2015). "The Impact of UN and US Economic Sanctions on GDP Growth." *European Journal of Political Economy*, 40, 110-125.
5. Ahn, D.P. & Ludema, R.D. (2020). "The Sword and the Shield: The Economics of Targeted Sanctions." *European Economic Review*, 130, 103587.
6. Bloom, N. (2009). "The Impact of Uncertainty Shocks." *Econometrica*, 77(3), 623-685.
