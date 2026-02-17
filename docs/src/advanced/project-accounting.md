# Project Accounting

*New in v0.7.0*

DataSynth generates end-to-end project accounting data including WBS hierarchies, cost tracking, revenue recognition, earned value management, and change order workflows.

## Overview

The project accounting module simulates construction, engineering, and internal project cost management:

1. **Project Master Data** — Projects with WBS (Work Breakdown Structure) hierarchies
2. **Cost Tracking** — Cost lines by category (labor, material, subcontractor, overhead, equipment, travel)
3. **Revenue Recognition** — Percentage-of-completion (PoC) and ASC 606 revenue recognition with unbilled tracking
4. **Earned Value Management** — BCWS, BCWP, ACWP, SPI, CPI, EAC, ETC, TCPI metrics
5. **Change Orders** — Scope, cost, and schedule change management
6. **Retainage** — Payment hold and release tracking

## Data Models

| Model | Description |
|-------|-------------|
| `ProjectCostLine` | Individual cost postings (Labor/Material/Subcontractor/Overhead/Equipment/Travel) |
| `ProjectRevenue` | PoC / ASC 606 revenue recognition with unbilled tracking |
| `ProjectMilestone` | Project milestones with payment amounts & status |
| `ChangeOrder` | Scope/cost/schedule changes (Submitted → Approved → Rejected) |
| `Retainage` | Payment holds and releases |
| `EarnedValueMetrics` | BCWS/BCWP/ACWP/SPI/CPI/EAC/ETC/TCPI |

## Configuration

```yaml
project_accounting:
  enabled: true
  project_count: 10
  project_types:
    capital: 0.25
    internal: 0.20
    customer: 0.30
    r_and_d: 0.10
    maintenance: 0.10
    technology: 0.05
  wbs:
    max_depth: 3
    elements_per_level_min: 2
    elements_per_level_max: 6
  cost_allocation:
    time_entry_percentage: 0.60
    expense_percentage: 0.30
    po_percentage: 0.40
    vi_percentage: 0.35
  revenue_recognition:
    method: percentage_of_completion  # percentage_of_completion, completed_contract
    measure: cost_to_cost
  milestones:
    avg_per_project: 4
    payment_milestone_rate: 0.50
  change_orders:
    enabled: true
    approval_rate: 0.75
  retainage:
    enabled: true
    hold_percentage: 0.10
  earned_value:
    enabled: true
  anomaly_rate: 0.03
```

## Output Files

| File | Description |
|------|-------------|
| `projects.csv` | Project master data |
| `wbs_elements.csv` | WBS hierarchies |
| `project_cost_lines.csv` | Cost postings by category |
| `project_revenue.csv` | Revenue recognition records |
| `project_milestones.csv` | Milestones with status |
| `change_orders.csv` | Change order tracking |
| `retainage.csv` | Payment hold records |
| `earned_value_metrics.csv` | EVM calculations |
| `project_accounting_anomaly_labels.csv` | Data quality labels |

## Earned Value Management

The EVM metrics provide project performance indicators:

| Metric | Formula | Description |
|--------|---------|-------------|
| **SPI** | BCWP / BCWS | Schedule Performance Index (>1 = ahead) |
| **CPI** | BCWP / ACWP | Cost Performance Index (>1 = under budget) |
| **EAC** | BAC / CPI | Estimate at Completion |
| **ETC** | EAC - ACWP | Estimate to Complete |
| **TCPI** | (BAC - BCWP) / (BAC - ACWP) | To-Complete Performance Index |

## Process Mining (OCPM)

The project accounting module contributes 4 object types and 5 activities:

- **Object Types**: `project`, `project_cost_line`, `project_milestone`, `change_order`
- **Activities**: Project creation, activation, completion, cost posting, milestone completion, change order approval
- **Lifecycle**: Projects follow created → active → on_hold → completed → closed
