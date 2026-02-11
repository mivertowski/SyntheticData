# Industry-Specific Features

SyntheticData includes industry-specific transaction modeling with authentic terminology, master data structures, and anomaly patterns. Three industries have full generator implementations (Manufacturing, Retail, Healthcare), while three additional industries (Technology, Financial Services, Professional Services) are available as configuration presets with industry-appropriate GL structures and anomaly rates.

## Overview

Each industry module provides:

- **Industry-specific transactions**: Authentic transaction types using correct terminology
- **Master data structures**: Industry-specific entities (BOM, routings, clinical codes, etc.)
- **Anomaly patterns**: Industry-authentic fraud and error patterns
- **GL account structures**: Industry-appropriate chart of accounts
- **Configuration options**: Fine-grained control over industry characteristics

## Implementation Status

| Industry | Status | Transaction Types | Master Data | Anomaly Patterns | Benchmarks |
|----------|--------|-------------------|-------------|-----------------|------------|
| **Manufacturing** | Full generator | 13 types | BOM, routings, work centers | 5 patterns | Yes |
| **Retail** | Full generator | 11 types | Stores, POS, loyalty | 6 patterns | Yes |
| **Healthcare** | Full generator | 9 types | ICD-10, CPT, DRG, payers | 6 patterns | Yes |
| **Technology** | Config preset | Config-only | — | 3 anomaly rates | Yes |
| **Financial Services** | Config preset | Config-only | — | 3 anomaly rates | Yes |
| **Professional Services** | Config preset | Config-only | — | 3 anomaly rates | No |

**Full generator** industries have dedicated Rust enum types with per-transaction generation logic, dedicated master data structures, and industry-specific anomaly injection. **Config preset** industries use the standard generator pipeline but apply industry-appropriate GL account structures, transaction distributions, and anomaly rates through configuration.

## Manufacturing

### Transaction Types

```rust
pub enum ManufacturingTransaction {
    // Production
    WorkOrderIssuance,      // Create production order
    MaterialRequisition,    // Issue materials to production
    LaborBooking,           // Record labor hours
    OverheadAbsorption,     // Apply manufacturing overhead
    ScrapReporting,         // Report production scrap
    ReworkOrder,            // Create rework order
    ProductionVariance,     // Record variances

    // Inventory
    RawMaterialReceipt,     // Receive raw materials
    WipTransfer,            // Transfer between work centers
    FinishedGoodsTransfer,  // Move to finished goods
    CycleCountAdjustment,   // Inventory adjustments

    // Costing
    StandardCostRevaluation,  // Update standard costs
    PurchasePriceVariance,    // Record PPV
}
```

### Master Data

```yaml
manufacturing:
  bom:
    depth: 4                    # BOM levels (3-7 typical)
    yield_rate: 0.97            # Expected yield
    scrap_factor: 0.02          # Scrap percentage
  routings:
    operations_per_product: 5   # Average operations
    setup_time_minutes: 30      # Default setup time
  work_centers:
    count: 20
    capacity_hours: 8
    efficiency: 0.85
```

### Anomaly Patterns

| Anomaly | Description | Detection Method |
|---------|-------------|------------------|
| Yield Manipulation | Reported yield differs from actual | Variance analysis |
| Labor Misallocation | Labor charged to wrong order | Cross-reference |
| Phantom Production | Production orders with no output | Data analytics |
| Obsolete Inventory | Aging inventory not written down | Aging analysis |
| Standard Cost Manipulation | Inflated standard costs | Trend analysis |

### Configuration

```yaml
industry_specific:
  enabled: true
  manufacturing:
    enabled: true
    bom_depth: 4
    just_in_time: false
    production_order_types:
      - standard
      - rework
      - prototype
    quality_framework: iso_9001
    supplier_tiers: 2
    standard_cost_frequency: quarterly
    target_yield_rate: 0.97
    scrap_alert_threshold: 0.03
    anomaly_rates:
      yield_manipulation: 0.005
      labor_misallocation: 0.008
      phantom_production: 0.002
      obsolete_inventory: 0.01
```

## Retail

### Transaction Types

```rust
pub enum RetailTransaction {
    // Point of Sale
    PosSale,                // Register sale
    ReturnRefund,           // Customer return
    VoidTransaction,        // Voided sale
    EmployeeDiscount,       // Staff discount
    LoyaltyRedemption,      // Points redemption

    // Inventory
    InventoryReceipt,       // Receive from DC
    StoreTransfer,          // Store-to-store
    MarkdownRecording,      // Price reductions
    ShrinkageAdjustment,    // Inventory loss

    // Cash Management
    CashDrop,               // Safe deposit
    RegisterReconciliation, // Drawer count
}
```

### Store Types

```yaml
retail:
  stores:
    types:
      - flagship      # High-volume, full assortment
      - standard      # Normal retail store
      - express       # Small format, convenience
      - outlet        # Discount/clearance
      - warehouse     # Bulk/club format
      - pop_up        # Temporary locations
      - digital       # E-commerce only
```

### Anomaly Patterns

| Anomaly | Description | Detection Method |
|---------|-------------|------------------|
| Sweethearting | Not scanning items | Video analytics |
| Skimming | Cash theft from register | Cash variance |
| Refund Fraud | Fake returns | Return pattern |
| Receiving Fraud | Short shipment theft | 3-way match |
| Coupon Fraud | Invalid coupon use | Coupon validation |
| Employee Discount Abuse | Unauthorized discounts | Policy review |

### Configuration

```yaml
industry_specific:
  enabled: true
  retail:
    enabled: true
    store_types:
      - standard
      - express
      - outlet
    shrinkage_rate: 0.015
    return_rate: 0.08
    markdown_frequency: weekly
    loss_prevention:
      camera_coverage: 0.85
      eas_enabled: true
    pos_anomaly_rates:
      sweethearting: 0.002
      skimming: 0.001
      refund_fraud: 0.003
```

## Healthcare

### Transaction Types

```rust
pub enum HealthcareTransaction {
    // Revenue Cycle
    PatientRegistration,    // Register patient
    ChargeCapture,          // Record charges
    ClaimSubmission,        // Submit to payer
    PaymentPosting,         // Record payment
    DenialManagement,       // Handle denials

    // Clinical
    ProcedureCoding,        // CPT codes
    DiagnosisCoding,        // ICD-10 codes
    SupplyConsumption,      // Medical supplies
    PharmacyDispensing,     // Medications
}
```

### Coding Systems

```yaml
healthcare:
  coding:
    icd10: true         # Diagnosis codes
    cpt: true           # Procedure codes
    drg: true           # Diagnosis Related Groups
    hcpcs: true         # Supplies/equipment
```

### Payer Mix

```yaml
healthcare:
  payer_mix:
    medicare: 0.40
    medicaid: 0.20
    commercial: 0.30
    self_pay: 0.10
```

### Compliance Frameworks

```yaml
healthcare:
  compliance:
    hipaa: true           # Privacy rules
    stark_law: true       # Physician referrals
    anti_kickback: true   # AKS compliance
    false_claims_act: true
```

### Anomaly Patterns

| Anomaly | Description | Detection Method |
|---------|-------------|------------------|
| Upcoding | Higher-level code than justified | Code validation |
| Unbundling | Splitting bundled services | Bundle analysis |
| Phantom Billing | Billing for unrendered services | Audit |
| Duplicate Billing | Same service billed twice | Duplicate check |
| Kickbacks | Physician referral payments | Relationship analysis |
| HIPAA Violations | Unauthorized data access | Access logs |

### Configuration

```yaml
industry_specific:
  enabled: true
  healthcare:
    enabled: true
    facility_type: hospital  # hospital, physician_practice, etc.
    payer_mix:
      medicare: 0.40
      medicaid: 0.20
      commercial: 0.30
      self_pay: 0.10
    coding_system:
      icd10: true
      cpt: true
      drg: true
    compliance:
      hipaa: true
      stark_law: true
      anti_kickback: true
    avg_daily_encounters: 200
    avg_charges_per_encounter: 8
    anomaly_rates:
      upcoding: 0.02
      unbundling: 0.015
      phantom_billing: 0.005
      duplicate_billing: 0.008
```

## Technology

### Transaction Types

- License revenue recognition
- Subscription billing
- Professional services
- R&D capitalization
- Deferred revenue

### Configuration

```yaml
industry_specific:
  enabled: true
  technology:
    enabled: true
    revenue_model: subscription  # license, subscription, usage
    subscription_revenue_percent: 0.70
    professional_services_percent: 0.20
    license_revenue_percent: 0.10
    r_and_d_capitalization_rate: 0.15
    deferred_revenue_months: 12
    anomaly_rates:
      premature_revenue: 0.008
      channel_stuffing: 0.003
      improper_capitalization: 0.005
```

## Financial Services

### Transaction Types

- Loan origination
- Interest accrual
- Fee income
- Trading transactions
- Customer deposits
- Wire transfers

### Configuration

```yaml
industry_specific:
  enabled: true
  financial_services:
    enabled: true
    institution_type: commercial_bank
    regulatory_framework: us  # us, eu, uk
    loan_portfolio_size: 1000
    avg_loan_amount: 250000
    loan_loss_provision_rate: 0.02
    fee_income_percent: 0.15
    trading_volume_daily: 50000000
    anomaly_rates:
      loan_fraud: 0.003
      trading_fraud: 0.001
      account_takeover: 0.002
```

## Professional Services

### Transaction Types

- Time and billing
- Engagement management
- Trust account transactions
- Expense reimbursement
- Partner distributions

### Configuration

```yaml
industry_specific:
  enabled: true
  professional_services:
    enabled: true
    billing_model: hourly  # hourly, fixed_fee, contingency
    avg_hourly_rate: 350
    utilization_target: 0.75
    realization_rate: 0.92
    trust_accounting: true
    engagement_types:
      - audit
      - tax
      - advisory
      - litigation
    anomaly_rates:
      billing_fraud: 0.004
      trust_misappropriation: 0.001
      expense_fraud: 0.008
```

## Industry Benchmarks

SyntheticData provides pre-configured ML benchmarks for each industry:

```rust
// Get industry-specific benchmark
let bench = get_industry_benchmark(IndustrySector::Healthcare);

// Available benchmarks
let manufacturing = manufacturing_fraud_5k();
let retail = retail_fraud_10k();
let healthcare = healthcare_fraud_5k();
let technology = technology_fraud_3k();
let financial = financial_services_fraud_5k();
```

### Benchmark Features

Each industry benchmark includes:

- Industry-specific transaction features
- Relevant anomaly types
- Appropriate cost matrices
- Industry-specific evaluation metrics

## Best Practices

1. **Match industry to use case**: Select the industry that matches your target domain
2. **Use industry presets first**: Start with default settings before customizing
3. **Enable industry-specific anomalies**: These provide realistic fraud patterns
4. **Consider regulatory context**: Enable compliance frameworks relevant to your industry
5. **Use industry benchmarks**: Evaluate models against industry-specific baselines

## Output Files

| File | Description |
|------|-------------|
| `industry_transactions.csv` | Industry-specific transaction log |
| `industry_master_data.json` | Industry-specific entities |
| `industry_anomalies.csv` | Industry-specific anomaly labels |
| `industry_gl_accounts.csv` | Industry-specific chart of accounts |
