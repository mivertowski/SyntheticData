# datasynth-standards

The `datasynth-standards` crate provides comprehensive support for major accounting and auditing standards frameworks including IFRS, US GAAP, French GAAP (PCG), German GAAP (HGB), ISA, SOX, and PCAOB.

## Overview

This crate contains domain models and business logic for:

- **Accounting Standards**: Revenue recognition, lease accounting, fair value measurement, impairment testing
- **Audit Standards**: ISA requirements, analytical procedures, confirmations, audit opinions
- **Regulatory Frameworks**: SOX 302/404 compliance, PCAOB standards

## Modules

### `framework`

Core accounting framework selection and settings.

```rust
use datasynth_standards::framework::{AccountingFramework, FrameworkSettings};

// Select framework
let framework = AccountingFramework::UsGaap;
assert!(framework.allows_lifo());
assert!(!framework.allows_impairment_reversal());

// German GAAP specifics
let hgb = AccountingFramework::GermanGaap;
assert!(!hgb.allows_lifo());
assert!(hgb.allows_impairment_reversal());       // Mandatory under §253(5)
assert!(hgb.requires_pending_loss_provisions());  // HGB-specific
assert!(hgb.allows_low_value_asset_expensing());  // GWG ≤ 800 EUR

// Framework-specific settings
let settings = FrameworkSettings::us_gaap();
assert!(settings.validate().is_ok());
```

### `accounting`

Accounting standards models:

| Module | Standards | Key Types |
|--------|-----------|-----------|
| `revenue` | ASC 606 / IFRS 15 | `CustomerContract`, `PerformanceObligation`, `VariableConsideration` |
| `leases` | ASC 842 / IFRS 16 | `Lease`, `ROUAsset`, `LeaseLiability`, `LeaseAmortizationEntry` |
| `fair_value` | ASC 820 / IFRS 13 | `FairValueMeasurement`, `FairValueHierarchyLevel` |
| `impairment` | ASC 360 / IAS 36 | `ImpairmentTest`, `RecoverableAmountMethod` |
| `differences` | Dual Reporting | `FrameworkDifferenceRecord` |

### `audit`

Audit standards models:

| Module | Standards | Key Types |
|--------|-----------|-----------|
| `isa_reference` | ISA 200-720 | `IsaStandard`, `IsaRequirement`, `IsaProcedureMapping` |
| `analytical` | ISA 520 | `AnalyticalProcedure`, `VarianceInvestigation` |
| `confirmation` | ISA 505 | `ExternalConfirmation`, `ConfirmationResponse` |
| `opinion` | ISA 700/705/706/701 | `AuditOpinion`, `KeyAuditMatter`, `OpinionModification` |
| `audit_trail` | Traceability | `AuditTrail`, `TrailGap` |
| `pcaob` | PCAOB AS | `PcaobStandard`, `PcaobIsaMapping` |

### `regulatory`

Regulatory compliance models:

| Module | Standards | Key Types |
|--------|-----------|-----------|
| `sox` | SOX 302/404 | `Sox302Certification`, `Sox404Assessment`, `DeficiencyMatrix`, `MaterialWeakness` |

## Usage Examples

### Revenue Recognition

```rust
use datasynth_standards::accounting::revenue::{
    CustomerContract, PerformanceObligation, ObligationType, SatisfactionPattern,
};
use datasynth_standards::framework::AccountingFramework;
use rust_decimal_macros::dec;

// Create a customer contract under US GAAP
let mut contract = CustomerContract::new(
    "C001".to_string(),
    "CUST001".to_string(),
    dec!(100000),
    AccountingFramework::UsGaap,
);

// Add performance obligations
let po = PerformanceObligation::new(
    "PO001".to_string(),
    ObligationType::Good,
    SatisfactionPattern::PointInTime,
    dec!(60000),
);
contract.add_performance_obligation(po);
```

### Lease Accounting

```rust
use datasynth_standards::accounting::leases::{Lease, LeaseAssetClass, LeaseClassification};
use datasynth_standards::framework::AccountingFramework;
use chrono::NaiveDate;
use rust_decimal_macros::dec;

// Create a lease
let lease = Lease::new(
    "L001".to_string(),
    LeaseAssetClass::RealEstate,
    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    60,                    // 5-year term
    dec!(10000),          // Monthly payment
    0.05,                  // Discount rate
    AccountingFramework::UsGaap,
);

// Classify under US GAAP bright-line tests
let classification = lease.classify_us_gaap(
    72,                    // Asset useful life (months)
    dec!(600000),         // Fair value
    dec!(550000),         // Present value of payments
);
```

### ISA Standards

```rust
use datasynth_standards::audit::isa_reference::{
    IsaStandard, IsaRequirement, IsaRequirementType,
};

// Reference an ISA standard
let standard = IsaStandard::Isa315;
assert_eq!(standard.number(), "315");
assert!(standard.title().contains("Risk"));

// Create a requirement
let requirement = IsaRequirement::new(
    IsaStandard::Isa500,
    "12".to_string(),
    IsaRequirementType::Requirement,
    "Design and perform audit procedures".to_string(),
);
```

### SOX Compliance

```rust
use datasynth_standards::regulatory::sox::{
    Sox404Assessment, DeficiencyMatrix, DeficiencyLikelihood, DeficiencyMagnitude,
};
use uuid::Uuid;

// Create a SOX 404 assessment
let assessment = Sox404Assessment::new(
    Uuid::new_v4(),
    2024,
    true, // ICFR effective
);

// Classify a deficiency
let deficiency = DeficiencyMatrix::new(
    DeficiencyLikelihood::Probable,
    DeficiencyMagnitude::Material,
);
assert!(deficiency.is_material_weakness());
```

## Framework Validation

The crate validates framework-specific rules:

```rust
use datasynth_standards::framework::{AccountingFramework, FrameworkSettings};

// LIFO is not permitted under IFRS
let mut settings = FrameworkSettings::ifrs();
settings.use_lifo_inventory = true;
assert!(settings.validate().is_err());

// PPE revaluation is not permitted under US GAAP
let mut settings = FrameworkSettings::us_gaap();
settings.use_ppe_revaluation = true;
assert!(settings.validate().is_err());

// German GAAP: LIFO also prohibited
let mut settings = FrameworkSettings::german_gaap();
settings.use_lifo_inventory = true;
assert!(settings.validate().is_err());
```

## Dependencies

```toml
[dependencies]
datasynth-standards = "0.2.3"
```

## Feature Flags

Currently, no optional features are defined. All functionality is included by default.

## See Also

- [Accounting Standards Guide](../advanced/accounting-standards.md) - Detailed usage guide
- [Configuration Reference](../configuration/yaml-schema.md) - YAML configuration options
- [datasynth-eval](./datasynth-eval.md) - Standards compliance evaluation
