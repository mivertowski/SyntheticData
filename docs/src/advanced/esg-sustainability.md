# ESG / Sustainability

*New in v0.7.0*

DataSynth generates comprehensive Environmental, Social, and Governance (ESG) data aligned with major reporting frameworks including GRI, SASB, and TCFD.

## Overview

The ESG module covers three pillars:

### Environmental
- **GHG Emissions** — Scope 1 (direct), Scope 2 (purchased energy), Scope 3 (value chain) following the GHG Protocol
- **Energy Consumption** — By type (electricity, gas, diesel, solar, wind) with renewable tracking
- **Water Usage** — Withdrawal, discharge, and consumption by source
- **Waste Management** — By type and disposal method with diversion targets

### Social
- **Workforce Diversity** — Gender, ethnicity, age, disability, and veteran dimensions by organizational level
- **Pay Equity** — Pay gap analysis with group comparisons and ratios
- **Safety** — Individual incidents (injury, illness, near-miss, fatality) and aggregate metrics (TRIR, LTIR, DART rates)

### Governance
- **Board Metrics** — Composition, independence, diversity, and ethics metrics
- **Disclosures** — GRI/SASB/TCFD framework disclosures with assurance status
- **Supply Chain ESG** — Supplier assessments and ratings
- **Climate Scenarios** — Scenario impact analysis

## Data Models

| Category | Models |
|----------|--------|
| Environmental | `EmissionRecord`, `EnergyConsumption`, `WaterUsage`, `WasteRecord` |
| Social | `WorkforceDiversityMetric`, `PayEquityMetric`, `SafetyIncident`, `SafetyMetric` |
| Governance | `GovernanceMetric`, `EsgDisclosure`, `SupplierEsgAssessment`, `ClimateScenarioAnalysis` |

## Configuration

```yaml
esg:
  enabled: true
  environmental:
    scope1:
      enabled: true
      regions: [US, EU, APAC]
    scope2:
      enabled: true
      regions: [US, EU, APAC]
    scope3:
      enabled: true
      categories: [purchased_goods, business_travel, commuting]
    energy:
      enabled: true
      facility_count: 5
      renewable_target: 0.30
    water:
      enabled: true
      facility_count: 3
    waste:
      enabled: true
      diversion_target: 0.50
  social:
    diversity:
      enabled: true
      dimensions: [gender, ethnicity, age, disability, veteran]
    pay_equity:
      enabled: true
    safety:
      enabled: true
      target_trir: 1.5
  governance:
    enabled: true
    board_size: 12
    independence_target: 0.75
  supply_chain_esg:
    enabled: true
    assessment_coverage: 0.80
  reporting:
    frameworks: [gri, sasb, tcfd]
    assurance_level: limited       # limited, reasonable
  climate_scenarios:
    enabled: true
    scenarios: [below_2c, net_zero_2050, business_as_usual]
  anomaly_rate: 0.02
```

## Output Files

| File | Description |
|------|-------------|
| `emission_records.csv` | GHG emissions (Scope 1/2/3) |
| `energy_consumption.csv` | Energy by source & facility |
| `water_usage.csv` | Water withdrawal/discharge/consumption |
| `waste_records.csv` | Waste by type & disposal |
| `workforce_diversity_metrics.csv` | Diversity by dimension & org level |
| `pay_equity_metrics.csv` | Pay gap analysis |
| `safety_incidents.csv` | Individual safety incidents |
| `safety_metrics.csv` | Aggregate safety rates (TRIR/LTIR/DART) |
| `governance_metrics.csv` | Board composition |
| `esg_disclosures.csv` | Disclosure & assurance records |
| `supplier_esg_assessments.csv` | Supply chain ratings |
| `climate_scenarios.csv` | Scenario impact analysis |
| `esg_anomaly_labels.csv` | Data quality labels |

## Reporting Frameworks

| Framework | Coverage |
|-----------|----------|
| **GRI** | Global Reporting Initiative Standards — environmental, social, governance disclosures |
| **SASB** | Sustainability Accounting Standards Board — industry-specific metrics |
| **TCFD** | Task Force on Climate-related Financial Disclosures — climate risk and scenario analysis |

## Process Mining (OCPM)

The ESG module contributes 3 object types and 3 activities:

- **Object Types**: `esg_data_point`, `emission_record`, `esg_disclosure`
- **Activities**: ESG data collection, emission calculation, disclosure submission
- **Lifecycle**: Data points follow collected → validated → reported; Disclosures follow draft → submitted → assured
