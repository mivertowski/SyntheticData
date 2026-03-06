# SOX Compliance Testing

Test internal control monitoring systems.

## Overview

DataSynth generates data for SOX 404 compliance testing:

- Internal control definitions
- Control test evidence
- Segregation of Duties violations
- Approval workflow data

## Configuration

```yaml
global:
  seed: 42
  industry: financial_services
  start_date: 2024-01-01
  period_months: 12

transactions:
  target_count: 50000

internal_controls:
  enabled: true

  controls:
    - id: "CTL-001"
      name: "Payment Authorization"
      type: preventive
      frequency: continuous
      threshold: 10000
      assertions: [authorization, validity]

    - id: "CTL-002"
      name: "Journal Entry Review"
      type: detective
      frequency: daily
      assertions: [accuracy, completeness]

    - id: "CTL-003"
      name: "Bank Reconciliation"
      type: detective
      frequency: monthly
      assertions: [existence, completeness]

  sod_rules:
    - conflict_type: create_approve
      processes: [ap_invoice, ap_payment]
      description: "Cannot create and approve payments"

    - conflict_type: create_approve
      processes: [ar_invoice, ar_receipt]
      description: "Cannot create and approve receipts"

    - conflict_type: custody_recording
      processes: [cash_handling, cash_recording]
      description: "Cannot handle and record cash"

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

fraud:
  enabled: true
  fraud_rate: 0.005

  types:
    skipped_approval: 0.30
    threshold_manipulation: 0.30
    unauthorized_discount: 0.20
    duplicate_payment: 0.20

output:
  format: csv
```

## Control Testing

### 1. Control Evidence

```python
import pandas as pd

# Load control data
controls = pd.read_csv('output/controls/internal_controls.csv')
mappings = pd.read_csv('output/controls/control_account_mappings.csv')
entries = pd.read_csv('output/transactions/journal_entries.csv')

# Identify entries subject to each control
for _, control in controls.iterrows():
    control_id = control['control_id']
    threshold = control['threshold']

    # Filter entries in scope
    if pd.notna(threshold):
        in_scope = entries[
            (entries['control_ids'].str.contains(control_id)) &
            (entries['debit_amount'] >= threshold)
        ]
    else:
        in_scope = entries[entries['control_ids'].str.contains(control_id)]

    print(f"{control['name']}: {len(in_scope)} entries in scope")
```

### 2. Approval Testing

```python
# Load entries with approval data
entries = pd.read_csv('output/transactions/journal_entries.csv')

# Test approval compliance
approval_required = entries[entries['debit_amount'] >= 5000]
approved = approval_required[approval_required['approved_by'].notna()]
not_approved = approval_required[approval_required['approved_by'].isna()]

print(f"Requiring approval: {len(approval_required)}")
print(f"Properly approved: {len(approved)}")
print(f"Missing approval: {len(not_approved)}")

# Test approval levels
def check_approval_level(row):
    amount = row['debit_amount']
    if amount >= 100000:
        return row['approval_level'] >= 4
    elif amount >= 25000:
        return row['approval_level'] >= 3
    elif amount >= 5000:
        return row['approval_level'] >= 2
    return True

entries['approval_adequate'] = entries.apply(check_approval_level, axis=1)
inadequate = entries[~entries['approval_adequate']]
print(f"Inadequate approval level: {len(inadequate)}")
```

### 3. Segregation of Duties

```python
# Load SoD data
sod_rules = pd.read_csv('output/controls/sod_rules.csv')
entries = pd.read_csv('output/transactions/journal_entries.csv')

# Identify violations
violations = entries[entries['sod_violation'] == True]
print(f"Total SoD violations: {len(violations)}")

# Analyze by type
violation_summary = violations.groupby('sod_conflict_type').agg({
    'document_id': 'count',
    'debit_amount': 'sum'
}).rename(columns={'document_id': 'count', 'debit_amount': 'total_amount'})

print("\nViolations by type:")
print(violation_summary)

# Analyze by user
user_violations = violations.groupby('created_by').size().sort_values(ascending=False)
print("\nTop violators:")
print(user_violations.head(10))
```

### 4. Threshold Manipulation

```python
# Detect threshold-adjacent transactions
approval_threshold = 10000

entries['near_threshold'] = (
    (entries['debit_amount'] >= approval_threshold * 0.9) &
    (entries['debit_amount'] < approval_threshold)
)

near_threshold = entries[entries['near_threshold']]
print(f"Near-threshold entries: {len(near_threshold)}")

# Statistical analysis
expected_near = len(entries) * 0.10  # 10% would be in this range randomly
chi_stat = ((len(near_threshold) - expected_near) ** 2) / expected_near
print(f"Chi-square statistic: {chi_stat:.2f}")
```

## Control Matrix

### Generate RACM

```python
# Risk and Control Matrix
controls = pd.read_csv('output/controls/internal_controls.csv')
mappings = pd.read_csv('output/controls/control_account_mappings.csv')

racm = controls.merge(mappings, on='control_id')
racm = racm[[
    'control_id', 'name', 'control_type', 'frequency',
    'account_number', 'assertions'
]]

# Add testing results
racm['population'] = racm['account_number'].apply(
    lambda x: len(entries[entries['account_number'] == x])
)
racm['exceptions'] = racm['control_id'].apply(
    lambda x: len(entries[
        (entries['control_ids'].str.contains(x)) &
        (entries['is_anomaly'] == True)
    ])
)
racm['exception_rate'] = racm['exceptions'] / racm['population']

print(racm)
```

## Test Documentation

### Control Test Template

```python
def document_control_test(control_id, entries, sample_size=25):
    """Generate control test documentation."""
    control = controls[controls['control_id'] == control_id].iloc[0]

    # Get population
    population = entries[entries['control_ids'].str.contains(control_id)]

    # Sample
    sample = population.sample(n=min(sample_size, len(population)), random_state=42)

    # Test results
    exceptions = sample[sample['is_anomaly'] == True]

    return {
        'control_id': control_id,
        'control_name': control['name'],
        'control_type': control['control_type'],
        'frequency': control['frequency'],
        'population_size': len(population),
        'sample_size': len(sample),
        'exceptions_found': len(exceptions),
        'exception_rate': len(exceptions) / len(sample),
        'conclusion': 'Effective' if len(exceptions) == 0 else 'Exception Noted'
    }

# Test all controls
results = []
for control_id in controls['control_id']:
    result = document_control_test(control_id, entries)
    results.append(result)

test_results = pd.DataFrame(results)
test_results.to_csv('control_test_results.csv', index=False)
```

## Deficiency Assessment

```python
# Classify deficiencies
def assess_deficiency(exception_rate, amount_impact):
    if exception_rate > 0.10 or amount_impact > 1000000:
        return 'Material Weakness'
    elif exception_rate > 0.05 or amount_impact > 100000:
        return 'Significant Deficiency'
    elif exception_rate > 0:
        return 'Control Deficiency'
    return 'No Deficiency'

test_results['amount_impact'] = test_results['control_id'].apply(
    lambda x: entries[
        (entries['control_ids'].str.contains(x)) &
        (entries['is_anomaly'] == True)
    ]['debit_amount'].sum()
)

test_results['deficiency_classification'] = test_results.apply(
    lambda x: assess_deficiency(x['exception_rate'], x['amount_impact']),
    axis=1
)

print(test_results[['control_name', 'exception_rate', 'deficiency_classification']])
```

## See Also

- [Compliance Configuration](../configuration/compliance.md)
- [Audit Analytics](audit-analytics.md)
- [Anomaly Injection](../advanced/anomaly-injection.md)
