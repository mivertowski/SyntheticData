# Compliance

Compliance settings control fraud injection, internal controls, and approval workflows.

## Fraud Configuration

```yaml
fraud:
  enabled: true
  fraud_rate: 0.005

  types:
    fictitious_transaction: 0.15
    revenue_manipulation: 0.10
    expense_capitalization: 0.10
    split_transaction: 0.15
    round_tripping: 0.05
    kickback_scheme: 0.10
    ghost_employee: 0.05
    duplicate_payment: 0.15
    unauthorized_discount: 0.10
    suspense_abuse: 0.05
```

### Fraud Rate

Overall percentage of fraudulent transactions:

```yaml
fraud:
  enabled: true
  fraud_rate: 0.005    # 0.5% fraud rate
  fraud_rate: 0.01     # 1% fraud rate
  fraud_rate: 0.001    # 0.1% fraud rate
```

### Fraud Types

| Type | Description |
|------|-------------|
| `fictitious_transaction` | Completely fabricated entries |
| `revenue_manipulation` | Premature/delayed revenue recognition |
| `expense_capitalization` | Improper capitalization of expenses |
| `split_transaction` | Split to avoid approval thresholds |
| `round_tripping` | Circular transactions to inflate revenue |
| `kickback_scheme` | Vendor kickback arrangements |
| `ghost_employee` | Payments to non-existent employees |
| `duplicate_payment` | Same invoice paid multiple times |
| `unauthorized_discount` | Unapproved customer discounts |
| `suspense_abuse` | Hiding items in suspense accounts |

### Fraud Patterns

```yaml
fraud:
  patterns:
    threshold_adjacent:
      enabled: true
      threshold: 10000             # Approval threshold
      range: 0.1                   # % below threshold

    time_based:
      weekend_preference: 0.3      # Weekend entry rate
      after_hours_preference: 0.2  # After hours rate

    entity_targeting:
      repeat_offender_rate: 0.4    # Same user commits multiple
```

---

## Internal Controls Configuration

```yaml
internal_controls:
  enabled: true

  controls:
    - id: "CTL-001"
      name: "Payment Approval"
      type: preventive
      frequency: continuous
      assertions:
        - authorization
        - validity

  sod_rules:
    - conflict_type: create_approve
      processes: [ap_invoice, ap_payment]
```

### Control Definition

```yaml
internal_controls:
  controls:
    - id: "CTL-001"
      name: "Payment Approval"
      description: "Payments require manager approval"
      type: preventive              # preventive, detective
      frequency: continuous         # continuous, daily, weekly, monthly
      assertions:
        - authorization
        - validity
        - completeness
      accounts: ["2000"]            # Applicable accounts
      threshold: 5000               # Trigger threshold

    - id: "CTL-002"
      name: "Journal Entry Review"
      type: detective
      frequency: daily
      assertions:
        - accuracy
        - completeness
```

### Control Types

| Type | Description |
|------|-------------|
| `preventive` | Prevents errors/fraud before occurrence |
| `detective` | Detects errors/fraud after occurrence |

### Control Assertions

| Assertion | Description |
|-----------|-------------|
| `authorization` | Proper approval obtained |
| `validity` | Transaction is legitimate |
| `completeness` | All transactions recorded |
| `accuracy` | Amounts are correct |
| `cutoff` | Recorded in correct period |
| `classification` | Properly categorized |

### Segregation of Duties

```yaml
internal_controls:
  sod_rules:
    - conflict_type: create_approve
      processes: [ap_invoice, ap_payment]
      description: "Cannot create and approve payments"

    - conflict_type: create_approve
      processes: [ar_invoice, ar_receipt]

    - conflict_type: custody_recording
      processes: [cash_handling, cash_recording]

    - conflict_type: authorization_custody
      processes: [vendor_master, ap_payment]
```

### SoD Conflict Types

| Type | Description |
|------|-------------|
| `create_approve` | Create and approve same transaction |
| `custody_recording` | Physical custody and recording |
| `authorization_custody` | Authorization and physical access |
| `create_modify` | Create and modify master data |

---

## Approval Configuration

```yaml
approval:
  enabled: true

  thresholds:
    - level: 1
      name: "Clerk"
      max_amount: 5000
    - level: 2
      name: "Supervisor"
      max_amount: 25000
    - level: 3
      name: "Manager"
      max_amount: 100000
    - level: 4
      name: "Director"
      max_amount: 500000
    - level: 5
      name: "Executive"
      max_amount: null          # Unlimited
```

### Approval Thresholds

```yaml
approval:
  thresholds:
    - level: 1
      name: "Level 1 - Clerk"
      max_amount: 5000
      auto_approve: false

    - level: 2
      name: "Level 2 - Supervisor"
      max_amount: 25000
      auto_approve: false

    - level: 3
      name: "Level 3 - Manager"
      max_amount: 100000
      requires_dual: false        # Single approver

    - level: 4
      name: "Level 4 - Director"
      max_amount: 500000
      requires_dual: true         # Dual approval required
```

### Approval Process

```yaml
approval:
  process:
    workflow: hierarchical        # hierarchical, matrix
    escalation_days: 3            # Auto-escalate after N days
    reminder_days: 1              # Send reminder after N days

  exceptions:
    recurring_exempt: true        # Skip for recurring entries
    system_exempt: true           # Skip for system entries
```

---

## Combined Example

```yaml
fraud:
  enabled: true
  fraud_rate: 0.005
  types:
    fictitious_transaction: 0.15
    split_transaction: 0.20
    duplicate_payment: 0.15
    ghost_employee: 0.10
    kickback_scheme: 0.10
    revenue_manipulation: 0.10
    expense_capitalization: 0.10
    unauthorized_discount: 0.10

internal_controls:
  enabled: true
  controls:
    - id: "SOX-001"
      name: "Payment Authorization"
      type: preventive
      frequency: continuous
      threshold: 10000

    - id: "SOX-002"
      name: "JE Review"
      type: detective
      frequency: daily

  sod_rules:
    - conflict_type: create_approve
      processes: [ap_invoice, ap_payment]
    - conflict_type: create_approve
      processes: [ar_invoice, ar_receipt]
    - conflict_type: create_modify
      processes: [vendor_master, ap_invoice]

approval:
  enabled: true
  thresholds:
    - level: 1
      max_amount: 5000
    - level: 2
      max_amount: 25000
    - level: 3
      max_amount: 100000
    - level: 4
      max_amount: null
```

## Validation

| Check | Rule |
|-------|------|
| `fraud_rate` | 0.0 - 1.0 |
| `fraud.types` | Sum = 1.0 |
| `control.id` | Unique |
| `thresholds` | Strictly ascending |

## Synthetic Data Certificates (v0.5.0)

Certificates provide cryptographic proof of the privacy guarantees and quality metrics of generated data.

```yaml
certificates:
  enabled: true
  issuer: "DataSynth"
  include_quality_metrics: true
```

When enabled, a `certificate.json` file is produced alongside the output containing:

- **DP Guarantee**: Mechanism (Laplace/Gaussian), epsilon, delta, composition method
- **Quality Metrics**: Benford MAD, correlation preservation, statistical fidelity, MIA AUC
- **Config Hash**: SHA-256 hash of the generation configuration
- **Signature**: HMAC-SHA256 signature for tamper detection
- **Fingerprint Hash**: Hash of source fingerprint (if fingerprint-based generation)

The certificate can be embedded in Parquet file metadata or included as a separate JSON file.

```bash
# Generate with certificate
datasynth-data generate --config config.yaml --output ./output --certificate

# Certificate is written to ./output/certificate.json
```

## See Also

- [Anomaly Injection](../advanced/anomaly-injection.md)
- [Master Data](master-data.md)
- [SOX Compliance Use Case](../use-cases/sox-compliance.md)
