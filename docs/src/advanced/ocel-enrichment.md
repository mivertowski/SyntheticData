# OCEL 2.0 Enrichment

DataSynth enriches OCEL 2.0 event logs with lifecycle state machines, multi-object correlation events, and resource pool modeling for realistic process mining datasets.

## Lifecycle State Machines

Each document type follows a probabilistic state machine:

### PurchaseOrder States
`Created -> Approved -> PartiallyReceived -> FullyReceived -> Closed`

### SalesOrder States
`Created -> Confirmed -> PartiallyDelivered -> FullyDelivered -> Invoiced -> Closed`

### VendorInvoice States
`Received -> Verified -> Approved -> Scheduled -> Paid`

State transitions include probabilistic branching (e.g., 10% rejection rate, 20% partial delivery).

## Correlation Events

Multi-object correlation events link related process instances:

| Event | Objects Linked | Description |
|-------|---------------|-------------|
| ThreeWayMatch | PO + GR + Invoice | Links purchase order, goods receipt, and vendor invoice |
| PaymentAllocation | Payment + Invoices | Links payment to one or more invoices |
| BankReconciliation | Statement + Payments | Links bank statement lines to internal payments |

## Resource Pools

Resource pools model workload distribution across processors:

| Strategy | Description |
|----------|-------------|
| RoundRobin | Cycle through resources equally |
| LeastBusy | Assign to resource with lowest current workload |
| SkillBased | Match resource skills to event requirements |

## Enriched Event Fields

Each OCEL event includes:
- `from_state` / `to_state` -- lifecycle state transition
- `resource_id` -- assigned resource from pool
- `resource_workload` -- current workload of assigned resource
- `correlation_id` -- links related events across objects

## Configuration

```yaml
ocpm:
  enabled: true
  lifecycle_state_machines:
    enabled: true
  resource_pools:
    enabled: true
    pool_size: 10
    assignment_strategy: least_busy
  correlation_events:
    three_way_match: true
    payment_allocation: true
    bank_reconciliation: true
  coverage_threshold: 0.95
```

## Evaluation

The OCEL enrichment quality evaluator checks:
- State transition coverage (% of valid transitions observed)
- Correlation event completeness
- Resource utilization distribution
