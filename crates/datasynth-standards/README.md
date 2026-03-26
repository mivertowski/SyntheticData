# datasynth-standards

Accounting and audit standards framework for synthetic data generation.

## Overview

`datasynth-standards` implements major accounting, auditing, and regulatory frameworks used in financial reporting:

- **Accounting standards**: US GAAP (ASC 606, 842, 820, 360) and IFRS (IFRS 15, 16, 13, IAS 36)
- **Audit standards**: 34 ISA standards (ISA 200-720) and PCAOB (AS 2201, 2110, 3101)
- **Regulatory frameworks**: SOX Section 302/404 with deficiency classification
- **Framework selection**: US GAAP, IFRS, French GAAP, German GAAP, or dual reporting

## Key Types

| Category | Types |
|----------|-------|
| Revenue | `CustomerContract`, `PerformanceObligation` (ASC 606 / IFRS 15) |
| Leases | `Lease`, `ROUAsset`, `LeaseLiability` (ASC 842 / IFRS 16) |
| Fair Value | `FairValueMeasurement` with Level 1/2/3 hierarchy (ASC 820 / IFRS 13) |
| Impairment | `ImpairmentTest` (ASC 360 / IAS 36) |
| Audit | `IsaStandard`, `IsaRequirement`, `AnalyticalProcedure`, `ExternalConfirmation` |
| Opinions | `AuditOpinion`, `KeyAuditMatter` (ISA 700/705/706/701) |
| Regulatory | `Sox302Certification`, `Sox404Assessment`, `DeficiencyMatrix` |

## Usage

```rust
use datasynth_standards::framework::AccountingFramework;
use datasynth_standards::audit::isa_reference::IsaStandard;

let framework = AccountingFramework::UsGaap;
let standard = IsaStandard::Isa315;
```

## Modules

| Module | Purpose |
|--------|---------|
| `framework` | `AccountingFramework` enum and `FrameworkSettings` |
| `accounting` | Revenue recognition, leases, fair value, impairment |
| `audit` | ISA references, analytical procedures, confirmations, opinions, PCAOB mappings, audit trail |

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
