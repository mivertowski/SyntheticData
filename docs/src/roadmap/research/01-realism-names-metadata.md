# Research: Realism in Names, Descriptions, and Metadata

## Current State Analysis

### Entity Name Generation

The current system uses basic name generation across multiple entity types:

| Entity Type | Current Approach | Realism Level |
|-------------|------------------|---------------|
| Vendors | "Vendor_{id}" or template-based | Low |
| Customers | "Customer_{id}" or template-based | Low |
| Employees | First/Last name pools | Medium |
| Materials | "Material_{id}" with category prefix | Low |
| Cost Centers | "{dept}_{code}" pattern | Medium |
| GL Accounts | Numeric codes with descriptions | High |
| Companies | Configurable but often generic | Medium |

### Description Generation

Current descriptions follow predictable patterns:
- Journal entries: "{type} for {entity}"
- Invoices: "Invoice for {goods/services}"
- Payments: "Payment for Invoice {ref}"

### Metadata Patterns

- **Timestamps**: Well-distributed but lack system-specific quirks
- **User IDs**: Sequential or simple patterns
- **References**: Deterministic but predictable formats

---

## Improvement Recommendations

### 1. Culturally-Aware Name Generation

#### 1.1 Regional Name Pools

**Implementation**: Create region-specific name databases with appropriate cultural distributions.

```yaml
# Proposed configuration structure
name_generation:
  strategy: regional_weighted
  regions:
    - region: north_america
      weight: 0.45
      subregions:
        - country: US
          weight: 0.85
          cultural_mix:
            - origin: anglo
              weight: 0.55
            - origin: hispanic
              weight: 0.25
            - origin: asian
              weight: 0.12
            - origin: other
              weight: 0.08
        - country: CA
          weight: 0.10
        - country: MX
          weight: 0.05
    - region: europe
      weight: 0.30
    - region: asia_pacific
      weight: 0.25
```

#### 1.2 Company Name Patterns by Industry

**Retail**:
- Pattern: `{Founder} {Product}` → "Johnson's Hardware"
- Pattern: `{Adjective} {Category}` → "Premier Electronics"
- Pattern: `{Location} {Type}` → "Westside Grocers"

**Manufacturing**:
- Pattern: `{Name} {Industry} {Suffix}` → "Anderson Steel Corporation"
- Pattern: `{Acronym} {Type}` → "ACM Industries"
- Pattern: `{Technical} {Systems}` → "Precision Machining Systems"

**Professional Services**:
- Pattern: `{Partner1}, {Partner2} & {Partner3}` → "Smith, Chen & Associates"
- Pattern: `{Name} {Specialty} {Type}` → "Hartwell Tax Advisors"
- Pattern: `{Adjective} {Service} {Suffix}` → "Strategic Consulting Group"

**Financial Services**:
- Pattern: `{Location} {Type} {Entity}` → "Pacific Coast Credit Union"
- Pattern: `{Founder} {Service}` → "Morgan Wealth Management"
- Pattern: `{Region} {Specialty}` → "Midwest Commercial Lending"

#### 1.3 Vendor Name Realism

**Current**: `Vendor_00042` or simple templates

**Proposed**: Industry-appropriate vendor names based on spend category:

```rust
// Conceptual structure
pub struct VendorNameGenerator {
    category_templates: HashMap<SpendCategory, Vec<NameTemplate>>,
    regional_styles: HashMap<Region, NamingConvention>,
    legal_suffixes: HashMap<Country, Vec<String>>,
}

impl VendorNameGenerator {
    pub fn generate(&self, category: SpendCategory, region: Region) -> VendorName {
        // Select template based on category
        // Apply regional naming conventions
        // Add appropriate legal suffix (Inc., LLC, GmbH, Ltd., S.A., etc.)
    }
}
```

**Examples by Category**:

| Category | Example Names |
|----------|---------------|
| Office Supplies | Staples, Office Depot, ULINE, Quill Corporation |
| IT Services | Accenture Technology, Cognizant Solutions, InfoSys Systems |
| Raw Materials | Alcoa Aluminum, US Steel Supply, Nucor Materials |
| Utilities | Pacific Gas & Electric, ConEdison, Duke Energy |
| Professional Services | Deloitte & Touche, KPMG Advisory, BDO Consulting |
| Logistics | FedEx Freight, UPS Supply Chain, XPO Logistics |
| Facilities | ABM Industries, CBRE Services, JLL Facilities |

---

### 2. Realistic Description Generation

#### 2.1 Journal Entry Descriptions

**Current Pattern**: Generic, formulaic

**Proposed**: Context-aware, varied descriptions with realistic abbreviations and typos

```yaml
journal_entry_descriptions:
  revenue:
    templates:
      - "Revenue recognition - {customer} - {contract_ref}"
      - "Rev rec {period} - {product_line}"
      - "Sales revenue {region} Q{quarter}"
      - "Earned revenue - PO# {po_number}"
    abbreviations:
      enabled: true
      probability: 0.3
      mappings:
        Revenue: ["Rev", "REV"]
        recognition: ["rec", "recog"]
        Quarter: ["Q", "Qtr"]
    variations:
      case_variation: 0.1
      typo_rate: 0.02

  expense:
    templates:
      - "AP invoice - {vendor} - {invoice_ref}"
      - "{expense_category} - {cost_center}"
      - "Accrued {expense_type} {period}"
      - "{vendor_short} inv {invoice_num}"
    context_aware:
      include_approver: 0.2
      include_po_reference: 0.7
      include_department: 0.4
```

#### 2.2 Invoice Line Item Descriptions

**Goods**:
```
- "Qty {quantity} {product_name} @ ${unit_price}/ea"
- "{product_sku} - {product_description}"
- "{quantity}x {product_short_name}"
- "Lot# {lot_number} {product_name}"
```

**Services**:
```
- "Professional services - {date_range}"
- "Consulting fees - {project_name}"
- "Retainer - {month} {year}"
- "{hours} hrs @ ${rate}/hr - {service_type}"
```

#### 2.3 Payment Descriptions

**Current**: "Payment for Invoice INV-00123"

**Proposed variations**:
```
- "Pmt INV-00123"
- "ACH payment - {vendor} - {invoice_ref}"
- "Wire transfer ref {bank_ref}"
- "Check #{check_number} - {vendor}"
- "EFT {date} {vendor_short}"
- "Batch payment - {batch_id}"
```

---

### 3. Enhanced Metadata Generation

#### 3.1 User ID Patterns

**Current**: Sequential or simple random

**Proposed**: Realistic corporate patterns

```yaml
user_id_patterns:
  format: "{first_initial}{last_name}{disambiguator}"
  examples:
    - "jsmith"
    - "jsmith2"
    - "john.smith"
    - "smithj"

  system_accounts:
    - prefix: "SVC_"
      examples: ["SVC_BATCH", "SVC_INTERFACE", "SVC_RECON"]
    - prefix: "SYS_"
      examples: ["SYS_AUTO", "SYS_SCHEDULER"]
    - prefix: "INT_"
      examples: ["INT_SAP", "INT_ORACLE", "INT_SALESFORCE"]

  admin_accounts:
    - pattern: "admin_{system}"
    - examples: ["admin_gl", "admin_ap", "admin_ar"]
```

#### 3.2 Reference Number Formats

**Realistic patterns by document type**:

```yaml
reference_formats:
  purchase_order:
    patterns:
      - "PO-{year}{seq:06}"        # PO-2024000142
      - "4500{seq:06}"              # SAP-style: 4500000142
      - "{plant}-{year}-{seq:05}"   # CHI-2024-00142

  invoice:
    vendor_patterns:
      - "INV-{seq:08}"
      - "{vendor_prefix}-{date}-{seq:04}"
      - "{random_alpha:3}{seq:06}"
    internal_patterns:
      - "VINV-{year}{seq:06}"
      - "{company_code}-AP-{seq:07}"

  journal_entry:
    patterns:
      - "{year}{period:02}{seq:06}"   # 202401000142
      - "JE-{date}-{seq:04}"          # JE-20240115-0142
      - "{company}-{year}-{seq:07}"   # C001-2024-0000142

  bank_reference:
    patterns:
      - "{date}{random:10}"           # Bank statement ref
      - "TRN{seq:12}"                 # Transaction ID
      - "{swift_code}{date}{seq:06}"  # SWIFT format
```

#### 3.3 Timestamp Realism

**System-specific posting behaviors**:

```yaml
timestamp_patterns:
  batch_processing:
    typical_times: ["02:00", "06:00", "22:00"]
    duration_minutes: 30-180
    day_pattern: "business_days"

  manual_posting:
    peak_hours: [9, 10, 11, 14, 15, 16]
    off_peak_probability: 0.15
    lunch_dip: [12, 13]
    lunch_probability: 0.3

  interface_posting:
    patterns:
      - hourly: ":00", ":15", ":30", ":45"
      - real_time: random within seconds
    source_systems:
      - name: "SAP_INTERFACE"
        posting_lag_hours: 0-4
      - name: "LEGACY_BATCH"
        posting_time: "23:30"
        posting_day: "next_business_day"

  period_end_crunch:
    enabled: true
    days_before_close: 3
    extended_hours: true
    weekend_activity: 0.3
```

---

### 4. Address and Contact Information

#### 4.1 Realistic Address Generation

**Current Gap**: Generic or missing addresses

**Proposed**: Region-appropriate address formats

```yaml
address_generation:
  us:
    format: "{street_number} {street_name} {street_type}\n{city}, {state} {zip}"
    components:
      street_numbers:
        residential: 100-9999
        commercial: 1-500
        distribution: "log_normal"
      street_names:
        sources: ["census_data", "common_names"]
        include_directional: 0.3  # "N", "S", "E", "W"
      street_types:
        distribution:
          Street: 0.25
          Avenue: 0.15
          Road: 0.12
          Drive: 0.12
          Boulevard: 0.08
          Lane: 0.08
          Way: 0.08
          Court: 0.05
          Place: 0.04
          Circle: 0.03
      cities:
        source: "population_weighted"
        major_metro_weight: 0.6
    commercial_patterns:
      suite_probability: 0.4
      floor_probability: 0.2
      building_name_probability: 0.15

  de:
    format: "{street_name} {street_number}\n{postal_code} {city}"
    # German addresses put number after street name

  jp:
    format: "〒{postal_code}\n{prefecture}{city}{ward}\n{block}-{building}-{unit}"
    # Japanese addressing system
```

#### 4.2 Phone Number Formats

```yaml
phone_generation:
  formats:
    us: "+1 ({area_code}) {exchange}-{subscriber}"
    uk: "+44 {area_code} {local_number}"
    de: "+49 {area_code} {subscriber}"

  area_codes:
    us:
      source: "valid_area_codes"
      weight_by_population: true
      exclude_toll_free: true
      business_toll_free_rate: 0.3
```

#### 4.3 Email Patterns

```yaml
email_generation:
  corporate:
    patterns:
      - "{first}.{last}@{company_domain}"
      - "{first_initial}{last}@{company_domain}"
      - "{first}_{last}@{company_domain}"
    domain_generation:
      from_company_name: true
      tld_distribution:
        ".com": 0.75
        ".net": 0.10
        ".io": 0.05
        ".co": 0.05
        country_tld: 0.05

  vendor_contacts:
    patterns:
      - "accounts.payable@{domain}"
      - "ar@{domain}"
      - "billing@{domain}"
      - "{first}.{last}@{domain}"
    generic_rate: 0.4
```

---

### 5. Material and Product Naming

#### 5.1 SKU Patterns

```yaml
sku_generation:
  patterns:
    category_prefix:
      format: "{category:3}-{subcategory:3}-{sequence:06}"
      example: "ELE-CPT-000142"  # Electronics-Components

    alphanumeric:
      format: "{alpha:2}{numeric:6}{check_digit}"
      example: "AB123456C"

    hierarchical:
      format: "{division}-{family}-{class}-{item}"
      example: "01-234-567-8901"
```

#### 5.2 Product Descriptions

**By Category**:

```yaml
product_descriptions:
  raw_materials:
    templates:
      - "{material_type}, {grade}, {specification}"
      - "{chemical_formula} {purity}% pure"
      - "{material} {form} - {dimensions}"
    examples:
      - "Steel Coil, Grade 304, 1.2mm thickness"
      - "Aluminum Sheet 6061-T6, 4' x 8' x 0.125\""
      - "Polyethylene Pellets, HDPE, 50lb bag"

  finished_goods:
    templates:
      - "{brand} {product_line} {model}"
      - "{product_type} - {feature1}, {feature2}"
      - "{category} {variant} ({color}/{size})"
    examples:
      - "Acme Pro Series 5000X Widget"
      - "Heavy-Duty Industrial Pump - 2HP, 120V"
      - "Office Chair Ergonomic Mesh (Black/Large)"

  services:
    templates:
      - "{service_type} - {duration} {frequency}"
      - "Professional {service} Services"
      - "{specialty} Consultation - {scope}"
    examples:
      - "HVAC Maintenance - Annual Contract"
      - "Professional IT Support Services"
      - "Legal Consultation - Contract Review"
```

---

### 6. Implementation Priority

| Enhancement | Effort | Impact | Priority |
|-------------|--------|--------|----------|
| Regional name pools | Medium | High | P1 |
| Industry-specific vendor names | Medium | High | P1 |
| Varied JE descriptions | Low | Medium | P1 |
| Reference number formats | Low | High | P1 |
| User ID patterns | Low | Medium | P2 |
| Address generation | High | Medium | P2 |
| Product descriptions | Medium | Medium | P2 |
| Email patterns | Low | Low | P3 |
| Phone formatting | Low | Low | P3 |

---

### 7. Data Sources

**Recommended External Data Sources**:

1. **Name Data**:
   - US Census Bureau name frequency data
   - International name databases (regional)
   - Industry-specific company name patterns

2. **Address Data**:
   - OpenAddresses project
   - Census TIGER/Line files
   - Postal code databases by country

3. **Reference Patterns**:
   - ERP documentation (SAP, Oracle, NetSuite)
   - Industry EDI standards
   - Banking reference formats (SWIFT, ACH)

4. **Product Data**:
   - UNSPSC category codes
   - Industry classification systems
   - Standard material specifications

---

### 8. Configuration Example

```yaml
# Enhanced name and metadata configuration
realism:
  names:
    strategy: culturally_aware
    primary_region: north_america
    diversity_index: 0.4

  vendors:
    naming_style: industry_appropriate
    include_legal_suffix: true
    regional_distribution:
      domestic: 0.7
      international: 0.3

  descriptions:
    variation_enabled: true
    abbreviation_rate: 0.25
    typo_injection_rate: 0.01

  references:
    format_style: erp_realistic
    include_check_digits: true

  timestamps:
    system_behavior_modeling: true
    batch_window_realism: true

  addresses:
    format: regional_appropriate
    commercial_indicators: true
```

---

## Next Steps

1. Create name pool data files for major regions
2. Implement `NameGenerator` trait with regional strategies
3. Build description template engine with variation support
4. Add reference format configurations to schema
5. Integrate address generation with Faker-like libraries

---

*See also*: [02-statistical-distributions.md](02-statistical-distributions.md) for numerical realism
