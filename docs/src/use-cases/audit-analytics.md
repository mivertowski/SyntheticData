# Audit Analytics

Test audit procedures and analytical tools with realistic data.

## Overview

DataSynth generates comprehensive datasets for audit analytics:

- Complete document trails
- Known control exceptions
- Benford's Law compliant amounts
- Realistic temporal patterns

## Configuration

```yaml
global:
  seed: 42
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 12

transactions:
  target_count: 100000

  benford:
    enabled: true                    # Realistic first-digit distribution

  temporal:
    month_end_spike: 2.5
    quarter_end_spike: 3.0
    year_end_spike: 4.0

document_flows:
  p2p:
    enabled: true
    flow_rate: 0.35
    three_way_match:
      quantity_tolerance: 0.02
      price_tolerance: 0.01
  o2c:
    enabled: true
    flow_rate: 0.35

master_data:
  vendors:
    count: 200
  customers:
    count: 500

internal_controls:
  enabled: true

anomaly_injection:
  enabled: true
  total_rate: 0.03
  generate_labels: true

  categories:
    fraud: 0.20
    error: 0.50
    process_issue: 0.30

output:
  format: csv
```

## Audit Procedures

### 1. Benford's Law Analysis

Test first-digit distribution of amounts:

```python
import pandas as pd
import numpy as np
from scipy import stats

# Load data
entries = pd.read_csv('output/transactions/journal_entries.csv')

# Extract first digits
amounts = entries['debit_amount'] + entries['credit_amount']
amounts = amounts[amounts > 0]
first_digits = amounts.apply(lambda x: int(str(x)[0]))

# Calculate observed distribution
observed = first_digits.value_counts().sort_index()
observed_freq = observed / observed.sum()

# Expected Benford distribution
benford = {d: np.log10(1 + 1/d) for d in range(1, 10)}

# Chi-square test
chi_stat, p_value = stats.chisquare(
    observed.values,
    [benford[d] * observed.sum() for d in range(1, 10)]
)

print(f"Chi-square: {chi_stat:.2f}, p-value: {p_value:.4f}")
```

### 2. Three-Way Match Testing

Verify PO, GR, and Invoice alignment:

```python
# Load documents
po = pd.read_csv('output/documents/purchase_orders.csv')
gr = pd.read_csv('output/documents/goods_receipts.csv')
inv = pd.read_csv('output/documents/vendor_invoices.csv')

# Join on references
matched = po.merge(gr, left_on='po_number', right_on='po_reference')
matched = matched.merge(inv, left_on='po_number', right_on='po_reference')

# Calculate variances
matched['qty_variance'] = abs(matched['gr_quantity'] - matched['po_quantity']) / matched['po_quantity']
matched['price_variance'] = abs(matched['inv_unit_price'] - matched['po_unit_price']) / matched['po_unit_price']

# Identify exceptions
qty_exceptions = matched[matched['qty_variance'] > 0.02]
price_exceptions = matched[matched['price_variance'] > 0.01]

print(f"Quantity exceptions: {len(qty_exceptions)}")
print(f"Price exceptions: {len(price_exceptions)}")
```

### 3. Duplicate Payment Detection

Find potential duplicate payments:

```python
# Load payments and invoices
payments = pd.read_csv('output/documents/payments.csv')
invoices = pd.read_csv('output/documents/vendor_invoices.csv')

# Group by vendor and amount
potential_dups = invoices.groupby(['vendor_id', 'total_amount']).filter(
    lambda x: len(x) > 1
)

# Check payment dates
duplicates = []
for (vendor, amount), group in potential_dups.groupby(['vendor_id', 'total_amount']):
    if len(group) > 1:
        duplicates.append({
            'vendor': vendor,
            'amount': amount,
            'count': len(group),
            'invoices': group['invoice_number'].tolist()
        })

print(f"Potential duplicate payments: {len(duplicates)}")
```

### 4. Journal Entry Testing

Analyze manual journal entries:

```python
# Load entries
entries = pd.read_csv('output/transactions/journal_entries.csv')

# Filter manual entries
manual = entries[entries['source'] == 'Manual']

# Analyze characteristics
print(f"Manual entries: {len(manual)}")
print(f"Weekend entries: {manual['is_weekend'].sum()}")
print(f"Month-end entries: {manual['is_month_end'].sum()}")

# Top accounts with manual entries
top_accounts = manual.groupby('account_number').size().sort_values(ascending=False).head(10)
```

### 5. Cutoff Testing

Verify transactions recorded in correct period:

```python
# Identify late postings
entries['posting_date'] = pd.to_datetime(entries['posting_date'])
entries['document_date'] = pd.to_datetime(entries['document_date'])
entries['posting_lag'] = (entries['posting_date'] - entries['document_date']).dt.days

# Find entries posted after period end
late_postings = entries[entries['posting_lag'] > 5]
print(f"Late postings: {len(late_postings)}")

# Check year-end cutoff
year_end = entries['posting_date'].dt.year.max()
cutoff_issues = entries[
    (entries['document_date'].dt.year < year_end) &
    (entries['posting_date'].dt.year == year_end + 1)
]
```

### 6. Segregation of Duties

Check for SoD violations:

```python
# Load controls data
sod_rules = pd.read_csv('output/controls/sod_rules.csv')
entries = pd.read_csv('output/transactions/journal_entries.csv')

# Find entries with SoD violations
violations = entries[entries['sod_violation'] == True]
print(f"SoD violations: {len(violations)}")

# Analyze by conflict type
violation_types = violations.groupby('sod_conflict_type').size()
```

## Audit Analytics Dashboard

### Key Metrics

| Metric | Query | Expected |
|--------|-------|----------|
| Benford Chi-square | First-digit test | < 15.51 (p > 0.05) |
| Match exceptions | Three-way match | < 2% |
| Duplicate indicators | Amount/vendor matching | < 0.5% |
| Late postings | Document vs posting date | < 1% |
| SoD violations | Control violations | Known from labels |

### Population Statistics

```python
# Summary statistics
print("=== Audit Population Summary ===")
print(f"Total transactions: {len(entries):,}")
print(f"Total amount: ${entries['debit_amount'].sum():,.2f}")
print(f"Unique vendors: {entries['vendor_id'].nunique()}")
print(f"Unique customers: {entries['customer_id'].nunique()}")
print(f"Date range: {entries['posting_date'].min()} to {entries['posting_date'].max()}")
```

### 7. Financial Statement Analytics (v0.6.0)

Analyze generated financial statements for consistency, trend analysis, and ratio testing:

```python
import pandas as pd

# Load financial statements
balance_sheet = pd.read_csv('output/financial_reporting/balance_sheet.csv')
income_stmt = pd.read_csv('output/financial_reporting/income_statement.csv')
cash_flow = pd.read_csv('output/financial_reporting/cash_flow.csv')

# Verify accounting equation holds
for _, row in balance_sheet.iterrows():
    assets = row['total_assets']
    liabilities = row['total_liabilities']
    equity = row['total_equity']
    imbalance = abs(assets - (liabilities + equity))
    assert imbalance < 0.01, f"A=L+E violation: {imbalance}"

# Analytical procedures: ratio analysis
ratios = pd.DataFrame({
    'period': balance_sheet['period'],
    'current_ratio': balance_sheet['current_assets'] / balance_sheet['current_liabilities'],
    'gross_margin': income_stmt['gross_profit'] / income_stmt['revenue'],
    'debt_to_equity': balance_sheet['total_liabilities'] / balance_sheet['total_equity'],
})

# Flag unusual ratio movements (> 2 std devs from mean)
for col in ['current_ratio', 'gross_margin', 'debt_to_equity']:
    mean = ratios[col].mean()
    std = ratios[col].std()
    outliers = ratios[abs(ratios[col] - mean) > 2 * std]
    if len(outliers) > 0:
        print(f"Unusual {col} in periods: {outliers['period'].tolist()}")
```

### Budget Variance Analysis

When budgets are enabled, compare budget to actual for each account:

```python
# Load budget vs actual data
budget = pd.read_csv('output/financial_reporting/budget_vs_actual.csv')

# Calculate variance percentage
budget['variance_pct'] = (budget['actual'] - budget['budget']) / budget['budget']

# Identify material variances (> 10%)
material = budget[abs(budget['variance_pct']) > 0.10]
print(f"Material variances: {len(material)} accounts")
print(material[['account', 'budget', 'actual', 'variance_pct']].to_string())

# Favorable vs unfavorable analysis
favorable = budget[
    ((budget['account_type'] == 'revenue') & (budget['variance_pct'] > 0)) |
    ((budget['account_type'] == 'expense') & (budget['variance_pct'] < 0))
]
print(f"Favorable variances: {len(favorable)}")
```

### Management KPI Trend Analysis

```python
# Load KPI data
kpis = pd.read_csv('output/financial_reporting/management_kpis.csv')

# Check for declining trends
for kpi_name in kpis['kpi_name'].unique():
    series = kpis[kpis['kpi_name'] == kpi_name].sort_values('period')
    values = series['value'].values
    # Simple trend check: are last 3 periods declining?
    if len(values) >= 3 and all(values[-3+i] > values[-3+i+1] for i in range(2)):
        print(f"Declining trend: {kpi_name}")
```

### Payroll Audit Testing (v0.6.0)

When the HR module is enabled, test payroll data for anomalies:

```python
# Load payroll data
payroll = pd.read_csv('output/hr/payroll_entries.csv')

# Ghost employee check: employees with pay but no time entries
time_entries = pd.read_csv('output/hr/time_entries.csv')
paid_employees = set(payroll['employee_id'].unique())
active_employees = set(time_entries['employee_id'].unique())
no_time = paid_employees - active_employees
print(f"Employees paid without time entries: {len(no_time)}")

# Payroll amount reasonableness
payroll_summary = payroll.groupby('employee_id')['gross_pay'].sum()
mean_pay = payroll_summary.mean()
std_pay = payroll_summary.std()
outliers = payroll_summary[payroll_summary > mean_pay + 3 * std_pay]
print(f"Unusually high total pay: {len(outliers)} employees")

# Expense policy violation detection
expenses = pd.read_csv('output/hr/expense_reports.csv')
violations = expenses[expenses['policy_violation'] == True]
print(f"Expense policy violations: {len(violations)}")
```

## Sampling

### Statistical Sampling

```python
from scipy import stats

# Calculate sample size for attribute testing
population_size = len(entries)
confidence_level = 0.95
tolerable_error_rate = 0.05
expected_error_rate = 0.01

# Sample size formula
z_score = stats.norm.ppf(1 - (1 - confidence_level) / 2)
sample_size = int(
    (z_score ** 2 * expected_error_rate * (1 - expected_error_rate)) /
    (tolerable_error_rate ** 2)
)

print(f"Recommended sample size: {sample_size}")

# Random sample
sample = entries.sample(n=sample_size, random_state=42)
```

### Stratified Sampling

```python
# Stratify by amount
entries['amount_stratum'] = pd.qcut(
    entries['debit_amount'] + entries['credit_amount'],
    q=5,
    labels=['Very Low', 'Low', 'Medium', 'High', 'Very High']
)

# Sample from each stratum
stratified_sample = entries.groupby('amount_stratum').apply(
    lambda x: x.sample(n=min(100, len(x)), random_state=42)
)
```

## Enterprise Group Audit Simulation (v1.3.0)

DataSynth v1.3.0 introduces a complete group audit simulation capability following ISA, IFRS, US GAAP, and local regulations.

### Generating a Full Audit Dataset

```bash
# Generate 113+ interconnected audit files with the audit-group preset
./target/release/datasynth-data generate --demo --preset audit-group --output ./audit-output
```

### What is Generated

| Category | Files and Content |
|----------|------------------|
| **ISA 600 Group Audit** | Component auditors, materiality allocation, scope assignment (full/specific/analytical), component instructions and reports |
| **Audit Opinion (ISA 700/705/706/701)** | Opinion conclusion derived from findings severity and going concern assessment, Key Audit Matters, PCAOB ICFR opinion |
| **Combined Risk Assessment (ISA 315)** | Per account area and assertion: inherent risk, control risk, detection risk, overall audit risk |
| **Materiality (ISA 320)** | Performance materiality, clearly trivial threshold, component materiality allocation |
| **Sampling (ISA 530)** | Monetary unit sampling, attribute sampling, sample items with deviation tracking |
| **SCOTS Classification** | Significant classes of transactions with assertion-level risk ratings |
| **Unusual Items (ISA 315)** | Journal entry flags linked to source JEs with risk indicators |
| **Analytical Relationships (ISA 520)** | Expectation vs. actual with threshold-driven unusual fluctuation flags |
| **Accounting Standards** | Deferred tax (IAS 12/ASC 740), ECL (IFRS 9/ASC 326), provisions (IAS 37), pensions (IAS 19), stock comp (ASC 718/IFRS 2), business combinations (IFRS 3/ASC 805), segment reporting (IFRS 8/ASC 280) |
| **Consolidated Financial Statements** | Standalone per-entity + consolidated group with elimination schedules, CTA, NCI |
| **SOX Compliance** | Section 302 certifications, Section 404 ICFR assessments with deficiency classification |

### Internal Coherence

All audit data is causally linked:

1. **CRA drives sampling** — higher risk areas receive larger sample sizes
2. **Sampling drives misstatements** — deviation rates correlate with risk levels
3. **Misstatements drive findings** — findings reference sampled items and CRA areas
4. **Findings drive the opinion** — aggregate misstatements determine opinion type (unmodified/qualified/adverse/disclaimer)
5. **Going concern affects the opinion** — financial indicator scores feed into ISA 570 assessment

### Graph Export for ML and AI Agents

The audit simulation adds 28 entity types and 27 edge types to the graph export:

```python
import torch

# Load audit graph for GNN training
graph = torch.load('output/graphs/pytorch_geometric/audit_graph.pt')

# Entity types include: CombinedRiskAssessment, MaterialityCalculation,
# SamplingPlan, SampledItem, AuditOpinion, KeyAuditMatter, Sox404Assessment,
# SignificantClassOfTransactions, UnusualItemFlag, AnalyticalRelationship,
# ComponentAuditor, GroupAuditPlan, ComponentInstruction, ComponentAuditorReport

# Edge types include: cra_to_account, opinion_to_engagement, kam_to_opinion,
# sampling_to_cra, unusual_to_journal_entry, finding_to_sampled_item
print(f"Audit nodes: {graph.num_nodes}")
print(f"Audit edges: {graph.num_edges}")
```

### Loading Audit Output Files

```python
import pandas as pd

# Core audit methodology
cra = pd.read_csv('output/audit/combined_risk_assessments.csv')
materiality = pd.read_csv('output/audit/materiality_calculations.csv')
sampling = pd.read_csv('output/audit/sampling_plans.csv')
sampled_items = pd.read_csv('output/audit/sampled_items.csv')

# Findings and opinion
findings = pd.read_csv('output/audit/audit_findings.csv')
opinions = pd.read_csv('output/audit/audit_opinions.csv')
kams = pd.read_csv('output/audit/key_audit_matters.csv')

# ISA 600 group audit
component_auditors = pd.read_csv('output/audit/component_auditors.csv')
group_plans = pd.read_csv('output/audit/group_audit_plans.csv')
instructions = pd.read_csv('output/audit/component_instructions.csv')

# SOX compliance
sox302 = pd.read_csv('output/audit/sox_302_certifications.csv')
sox404 = pd.read_csv('output/audit/sox_404_assessments.csv')

# Verify opinion is consistent with findings
for _, opinion in opinions.iterrows():
    eng_findings = findings[findings['engagement_id'] == opinion['engagement_id']]
    has_material = eng_findings['is_material'].any()
    if has_material:
        assert opinion['opinion_type'] != 'Unmodified', \
            f"Material findings should not yield unmodified opinion for {opinion['engagement_id']}"

print("All opinions are consistent with findings severity.")
```

## FSM Engine for Audit Process Analytics

The `datasynth-audit-fsm` engine generates event-sourced audit trails that are well suited for audit process efficiency analysis, planning optimization, and process mining.

### Event Trail Analysis

The FSM engine produces a flat JSON event trail where each event captures a state transition or procedure step with timestamps, actor IDs, and anomaly flags. This enables audit process efficiency analysis:

```python
import json
import pandas as pd

# Load the FSM event trail
with open('output/audit/fsm_event_trail.json') as f:
    events = json.load(f)

df = pd.DataFrame(events)
df['timestamp'] = pd.to_datetime(df['timestamp'])

# Time spent per procedure
procedure_times = df.groupby('procedure_id')['timestamp'].agg(['min', 'max'])
procedure_times['duration_hours'] = (procedure_times['max'] - procedure_times['min']).dt.total_seconds() / 3600
print("Procedure durations:")
print(procedure_times['duration_hours'].sort_values(ascending=False))

# Identify revision hotspots (under_review -> in_progress loops)
revisions = df[(df['from_state'] == 'under_review') & (df['to_state'] == 'in_progress')]
revision_counts = revisions.groupby('procedure_id').size().sort_values(ascending=False)
print(f"\nRevision hotspots:\n{revision_counts}")

# Anomaly detection in the audit process itself
anomalies = df[df['is_anomaly'] == True]
print(f"\nProcess anomalies: {len(anomalies)}")
print(anomalies[['procedure_id', 'anomaly_type', 'timestamp']].to_string())
```

### Monte Carlo Simulation for Audit Planning

The `datasynth-audit-optimizer` crate runs Monte Carlo simulations over the FSM engine to identify bottleneck procedures, estimate engagement duration distributions, and find the happy path:

```python
# Load Monte Carlo report (generated by the optimizer)
import json

with open('output/audit/monte_carlo_report.json') as f:
    mc = json.load(f)

print(f"Average engagement duration: {mc['avg_duration_hours']:.0f} hours")
print(f"Average events per engagement: {mc['avg_events']:.0f}")

# Top bottleneck procedures
print("\nBottleneck procedures (most events):")
for proc_id, avg_events in mc['bottleneck_procedures']:
    print(f"  {proc_id}: {avg_events:.1f} avg events")

# Revision hotspots (procedures most likely to require rework)
print("\nRevision hotspots:")
for proc_id, avg_revisions in mc['revision_hotspots']:
    print(f"  {proc_id}: {avg_revisions:.1f} avg revisions")

# Optimal procedure ordering
print(f"\nHappy path: {' -> '.join(mc['happy_path'])}")
```

### OCEL 2.0 Integration with Process Mining

The FSM event trail can be projected to OCEL 2.0 format for use with process mining tools. Each audit procedure becomes an object type, and evidence references become linked objects:

```python
import pm4py

# Load the OCEL 2.0 projection of the audit trail
ocel = pm4py.read.read_ocel2_json("output/audit/fsm_ocel.json")

# Discover audit process model per procedure type
for obj_type in ocel.object_types:
    filtered = pm4py.filtering.filter_ocel_object_types(ocel, [obj_type])
    print(f"\n{obj_type}: {len(filtered.events)} events, {len(filtered.objects)} objects")

# Detect conformance deviations (e.g., skipped steps)
# OCEL process discovery reveals actual vs expected audit workflows
```

### Analyzing Overlay Impact

Compare different overlay presets to understand their impact on audit quality metrics:

```python
# Run generation with each overlay preset and compare
# default: balanced (1,916 artifacts, ~776h duration, ~2 anomalies)
# thorough: high quality (~3,200 artifacts, ~1,195h duration, ~0 anomalies)
# rushed: lower quality (~900 artifacts, ~284h duration, ~7 anomalies)

presets = {
    'default':  {'artifacts': 1916, 'duration_h': 776, 'anomalies': 2},
    'thorough': {'artifacts': 3200, 'duration_h': 1195, 'anomalies': 0},
    'rushed':   {'artifacts': 900,  'duration_h': 284,  'anomalies': 7},
}

comparison = pd.DataFrame(presets).T
print("Overlay preset comparison:")
print(comparison.to_string())

# Use this to calibrate the quality/efficiency tradeoff in audit planning
```

For full details on the FSM engine, blueprints, and overlays, see the [Audit FSM Engine deep dive](../advanced/audit-fsm-engine.md).

## See Also

- [Anomaly Injection](../advanced/anomaly-injection.md)
- [Document Flows](../configuration/document-flows.md)
- [SOX Compliance](sox-compliance.md)
- [Audit FSM Engine](../advanced/audit-fsm-engine.md)
- [datasynth-audit-fsm](../crates/datasynth-audit-fsm.md)
- [datasynth-audit-optimizer](../crates/datasynth-audit-optimizer.md)
