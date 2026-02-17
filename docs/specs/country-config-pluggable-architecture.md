# Country Configuration Pluggable Architecture — Specification

> **Status:** Draft
> **Date:** 2026-02-16
> **Scope:** Extract all hardcoded country-specific values into pluggable JSON country packs; enable open-source minimal set and commercial full-coverage model

---

## 1. Executive Summary

### 1.1 Problem Statement

The codebase currently embeds country-specific knowledge — personal names, holidays, tax rates, phone formats, address structures, legal entity suffixes, currency conventions, and more — directly in Rust source code and scattered YAML template files. This creates several problems:

1. **Adding a new country requires code changes** across multiple crates (`datasynth-core`, `datasynth-generators`, `datasynth-banking`, `datasynth-config`)
2. **No single source of truth** — country data is fragmented across `holidays.rs`, `timezone.rs`, `names.rs`, `customer_generator.rs`, `presets.rs`, and five regional YAML templates
3. **Cannot ship country-specific updates independently** of code releases
4. **No clear boundary** between open-source baseline and commercially-licensable country depth
5. **US-centric defaults** are baked in (USD, America/New_York, T+2 settlement, Saturday/Sunday weekends) with no structured override path

### 1.2 Goal

Design a **pluggable country-pack architecture** where:

1. Every country-specific value is externalized into a standardized **JSON country-pack file**
2. Country packs are **self-contained** — one file per country holds all locale data needed by every generator
3. The runtime **discovers and loads** packs from a configurable directory, with graceful fallback
4. **Open-source ships with a minimal set** (e.g., US + 2-3 others) while **commercial packs** provide depth for 50+ countries
5. Third parties can **author custom packs** for proprietary or niche locales
6. **Zero code changes** required to add a new country — drop a JSON file, reference the country code in config

### 1.3 Current Hardcoded Country Data — Inventory

| Data Category | Current Location(s) | Countries Covered | Lines of Code |
|---|---|---|---|
| Public holidays | `datasynth-core/src/distributions/holidays.rs` | US, DE, GB, CN, JP, IN, BR, MX, AU, SG, KR (11) | ~1,450 |
| Person names (first/last, gendered) | `datasynth-core/src/templates/names.rs` + 5 YAML templates | US, DE, FR, CN, JP, IN, BR, GB (8 cultures) | ~600 Rust + ~1,200 YAML |
| Timezone presets | `datasynth-core/src/distributions/timezone.rs` | US, EU, APAC preset groups | ~40 |
| Phone number formats | `datasynth-banking/src/generators/customer_generator.rs` | US/CA, GB, fallback | ~20 |
| Date/amount locale formats | `datasynth-config/src/schema.rs` (DataQuality section) | US, EU (ISO implicit) | ~80 |
| Company/industry presets | `datasynth-config/src/presets.rs` | US, DE, CN, GB, FR, CH, IE, JP (by industry) | ~50 |
| Vendor/customer name templates | 5 YAML files in `examples/templates/` | BR, DE, JP, GB, IN | ~500 YAML each |
| Currency defaults | `datasynth-config/src/schema.rs` | USD default, per-company override | ~5 |
| GL account numbering | `datasynth-core/src/accounts.rs` | Generic (no country variation yet) | ~120 |
| Accounting/audit standards | `datasynth-standards/src/` | US GAAP, IFRS, ISA, PCAOB, SOX | ~3,000 |
| Business-day/settlement rules | `datasynth-config/src/schema.rs` | Generic with US defaults | ~100 |
| Lunar calendar calculations | `datasynth-core/src/distributions/holidays.rs` | CN, IN, SG, KR | ~200 |

**Total:** ~7,000+ lines of country-specific data embedded in source code and templates.

---

## 2. Architecture

### 2.1 Design Principles

| Principle | Rationale |
|---|---|
| **One file per country** | Self-contained, easy to version, diff, and distribute |
| **JSON format** | Language-agnostic, schema-validatable, no code execution risk |
| **ISO 3166-1 alpha-2 keying** | `US.json`, `DE.json`, `BR.json` — universally understood |
| **Layered defaults** | `_default.json` → `{country}.json` → user config YAML overrides |
| **Lazy loading** | Only load packs for countries referenced in the generation config |
| **Schema-versioned** | `"schema_version": "1.0"` in every pack for forward compatibility |
| **Deterministic** | No external network calls; all data is local files |
| **Graceful degradation** | Missing optional sections fall back to `_default.json`; missing required sections produce clear errors |

### 2.2 Component Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                        User Config (YAML)                        │
│  companies:                                                      │
│    - code: C001, country: "US"                                   │
│    - code: C002, country: "DE"                                   │
│    - code: C003, country: "BR"                                   │
└─────────────────────────┬────────────────────────────────────────┘
                          │ references country codes
                          ▼
┌──────────────────────────────────────────────────────────────────┐
│                   CountryPackRegistry                             │
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │ _default.json│  │   US.json   │  │   DE.json   │  ...         │
│  │  (fallback)  │  │  (bundled)  │  │ (commercial)│              │
│  └──────┬───────┘  └──────┬──────┘  └──────┬──────┘              │
│         │                 │                 │                     │
│         └────────┬────────┘─────────────────┘                    │
│                  ▼                                                │
│         MergedCountryConfig                                      │
│         (default ← country ← user overrides)                     │
└─────────────────────────┬────────────────────────────────────────┘
                          │ provides locale data
                          ▼
┌────────────┬────────────┬────────────┬────────────┬──────────────┐
│  Name Gen  │ Holiday Cal│  Tax Calc  │ Address Gen│  Phone Gen   │
│  Vendor Gen│ FX Service │  Doc Flow  │ Banking Gen│  Payroll Gen │
└────────────┴────────────┴────────────┴────────────┴──────────────┘
```

### 2.3 Discovery & Loading

```
Search order (first match wins, layers merge bottom-up):

1. Built-in embedded packs (compiled into binary via include_str!)
   └── crates/datasynth-core/country-packs/          ← open-source minimal set
       ├── _default.json
       ├── US.json
       └── ... (2-3 more)

2. External pack directory (runtime, filesystem)
   └── $DATASYNTH_COUNTRY_PACKS_DIR/                 ← commercial / custom packs
       ├── DE.json
       ├── BR.json
       └── ...

3. User config inline overrides (YAML)
   └── country_overrides:
         US:
           tax.corporate_rate: 0.21
```

**Resolution algorithm:**

```
fn resolve_country(code: &str) -> MergedCountryConfig {
    let base    = load_embedded("_default.json");
    let country = load_embedded(code)
                    .or_else(|| load_external(code))
                    .unwrap_or_default();
    let user    = config.country_overrides.get(code);

    base.deep_merge(country).deep_merge(user)
}
```

### 2.4 Crate Ownership

| Concern | Crate | Role |
|---|---|---|
| Country pack schema, loader, registry, merge logic | `datasynth-core` | New module: `country/` |
| Embedded open-source packs | `datasynth-core` | `country-packs/*.json` compiled in |
| Config schema for `country_packs_dir` + `country_overrides` | `datasynth-config` | New config fields |
| Consuming country data in generators | `datasynth-generators`, `datasynth-banking`, etc. | Accept `&CountryPack` parameter |
| Validation that referenced countries have packs | `datasynth-config` | Validation pass |
| Commercial pack distribution | Out-of-repo packaging | Separate artifact / crate feature |

---

## 3. Country Pack JSON Schema

### 3.1 Top-Level Structure

```jsonc
{
  "schema_version": "1.0",
  "country_code": "DE",                    // ISO 3166-1 alpha-2
  "country_name": "Germany",
  "region": "EMEA",                        // Grouping: AMERICAS, EMEA, APAC

  "locale":          { /* §3.2 */ },
  "names":           { /* §3.3 */ },
  "holidays":        { /* §3.4 */ },
  "tax":             { /* §3.5 */ },
  "address":         { /* §3.6 */ },
  "phone":           { /* §3.7 */ },
  "banking":         { /* §3.8 */ },
  "business_rules":  { /* §3.9 */ },
  "legal_entities":  { /* §3.10 */ },
  "accounting":      { /* §3.11 */ },
  "payroll":         { /* §3.12 */ },
  "vendor_templates":   { /* §3.13 */ },
  "customer_templates": { /* §3.14 */ },
  "material_templates": { /* §3.15 */ },
  "document_texts":     { /* §3.16 */ }
}
```

### 3.2 Locale

```jsonc
{
  "locale": {
    "language_code": "de",                  // ISO 639-1
    "language_name": "German",
    "default_currency": "EUR",              // ISO 4217
    "currency_symbol": "€",
    "currency_decimal_places": 2,
    "number_format": {
      "decimal_separator": ",",
      "thousands_separator": ".",
      "grouping": [3]                       // Indian: [3, 2]
    },
    "date_format": {
      "short": "DD.MM.YYYY",
      "long": "D. MMMM YYYY",
      "iso": "YYYY-MM-DD"
    },
    "default_timezone": "Europe/Berlin",
    "weekend_days": ["Saturday", "Sunday"], // Middle East: ["Friday", "Saturday"]
    "fiscal_year": {
      "start_month": 1,
      "start_day": 1,
      "variant": "calendar"                 // "calendar" | "custom" | "four_four_five"
    }
  }
}
```

### 3.3 Names

```jsonc
{
  "names": {
    "cultures": [
      {
        "culture_id": "german",
        "weight": 0.85,                     // 85% of generated names use this culture
        "male_first_names":   ["Hans", "Klaus", "Wolfgang", "Stefan", "Michael", "..."],
        "female_first_names": ["Anna", "Maria", "Elisabeth", "Petra", "Sabine", "..."],
        "last_names":         ["Müller", "Schmidt", "Schneider", "Fischer", "Weber", "..."],
        "name_order": "western",            // "western" (first last) | "eastern" (last first)
        "titles": {
          "male":   ["Herr"],
          "female": ["Frau"]
        },
        "academic_titles": ["Dr.", "Prof.", "Prof. Dr.", "Dipl.-Ing."]
      },
      {
        "culture_id": "turkish_german",
        "weight": 0.10,                     // Minority culture representation
        "male_first_names":   ["Mehmet", "Ali", "Mustafa", "..."],
        "female_first_names": ["Ayşe", "Fatma", "Emine", "..."],
        "last_names":         ["Yılmaz", "Kaya", "Demir", "..."],
        "name_order": "western"
      }
    ],
    "email_domains": ["example.de", "firma.de", "unternehmen.de", "beispiel.de"],
    "username_patterns": [
      "{first_initial}{last_name}",
      "{first_name}.{last_name}",
      "{last_name}{employee_number}"
    ]
  }
}
```

### 3.4 Holidays

```jsonc
{
  "holidays": {
    "calendar_type": "gregorian",           // "gregorian" | "lunar" | "hybrid"
    "fixed": [
      {
        "name": "Neujahr",
        "name_en": "New Year's Day",
        "month": 1,
        "day": 1,
        "activity_multiplier": 0.05
      },
      {
        "name": "Tag der Arbeit",
        "name_en": "Labour Day",
        "month": 5,
        "day": 1,
        "activity_multiplier": 0.05
      },
      {
        "name": "Tag der Deutschen Einheit",
        "name_en": "German Unity Day",
        "month": 10,
        "day": 3,
        "activity_multiplier": 0.05
      }
      // ... full list
    ],
    "easter_relative": [
      { "name": "Karfreitag",       "name_en": "Good Friday",    "offset_days": -2, "activity_multiplier": 0.05 },
      { "name": "Ostermontag",      "name_en": "Easter Monday",  "offset_days": 1,  "activity_multiplier": 0.05 },
      { "name": "Christi Himmelfahrt", "name_en": "Ascension Day", "offset_days": 39, "activity_multiplier": 0.1 },
      { "name": "Pfingstmontag",    "name_en": "Whit Monday",    "offset_days": 50, "activity_multiplier": 0.1 }
    ],
    "lunar": [],                            // For CN, KR, IN, SG: lunar-calendar-based holidays
    "regional_holidays": {
      "enabled": false,                     // Commercial: sub-national holidays (e.g., Bavaria, Texas)
      "regions": {}
    },
    "holiday_seasons": [
      {
        "name": "Weihnachtszeit",
        "name_en": "Christmas Season",
        "start": { "month": 12, "day": 24 },
        "end":   { "month": 12, "day": 26 },
        "activity_multiplier": 0.02,
        "description": "Extended holiday period with near-zero business activity"
      }
    ]
  }
}
```

### 3.5 Tax

```jsonc
{
  "tax": {
    "corporate_income_tax": {
      "standard_rate": 0.15,
      "trade_tax_rate": 0.14,               // Germany-specific Gewerbesteuer
      "solidarity_surcharge": 0.055,         // Applied on top of corporate tax
      "effective_combined_rate": 0.2975,
      "small_business_threshold": null
    },
    "vat": {
      "standard_rate": 0.19,
      "reduced_rates": [
        { "rate": 0.07, "label": "reduced", "applies_to": ["food", "books", "public_transport"] }
      ],
      "zero_rated": ["exports", "intra_eu_supplies"],
      "exempt": ["financial_services", "insurance", "healthcare", "education"],
      "registration_threshold": null,        // null = mandatory; number = voluntary below
      "filing_frequency": "monthly",         // "monthly" | "quarterly" | "annual"
      "reverse_charge_applicable": true
    },
    "withholding_tax": {
      "dividends_domestic": 0.25,
      "dividends_foreign_default": 0.2638,
      "interest": 0.0,
      "royalties": 0.15,
      "services": 0.0
    },
    "payroll_tax": {
      "income_tax_brackets": [
        { "up_to": 11604,  "rate": 0.0 },
        { "up_to": 17005,  "rate": 0.14 },
        { "up_to": 66760,  "rate": 0.2397 },
        { "up_to": 277825, "rate": 0.42 },
        { "above": 277825, "rate": 0.45 }
      ],
      "social_security": {
        "pension":        { "employee_rate": 0.093, "employer_rate": 0.093, "ceiling_annual": 90600 },
        "health":         { "employee_rate": 0.073, "employer_rate": 0.073, "ceiling_annual": 62100 },
        "unemployment":   { "employee_rate": 0.013, "employer_rate": 0.013, "ceiling_annual": 90600 },
        "long_term_care": { "employee_rate": 0.017, "employer_rate": 0.017, "ceiling_annual": 62100 }
      },
      "church_tax_rate": 0.08               // Optional, Bavaria: 0.08, other states: 0.09
    },
    "transfer_pricing": {
      "documentation_required": true,
      "methods": ["CUP", "RPM", "CPLM", "TNMM", "PSM"],
      "safe_harbor_rules": false
    }
  }
}
```

### 3.6 Address

```jsonc
{
  "address": {
    "format_template": "{street} {building_number}\n{postal_code} {city}",
    "components": {
      "street_names":    ["Hauptstraße", "Bahnhofstraße", "Berliner Straße", "Industriestraße", "..."],
      "city_names":      ["München", "Berlin", "Hamburg", "Frankfurt", "Stuttgart", "Düsseldorf", "..."],
      "state_names":     ["Bayern", "Baden-Württemberg", "Nordrhein-Westfalen", "Hessen", "..."],
      "state_codes":     ["BY", "BW", "NW", "HE", "..."]
    },
    "postal_code": {
      "format": "NNNNN",                   // N=digit, A=alpha
      "regex": "^[0-9]{5}$",
      "ranges": [
        { "from": "01000", "to": "99999" }
      ]
    },
    "building_number": {
      "format": "numeric",                  // "numeric" | "alphanumeric" | "numeric_suffix"
      "range": [1, 200]
    },
    "country_calling_code": "+49"
  }
}
```

### 3.7 Phone

```jsonc
{
  "phone": {
    "country_calling_code": "+49",
    "formats": {
      "landline": "+49-{area_code}-{subscriber}",
      "mobile":   "+49-1{n2}-{n7}",
      "freephone": "+49-800-{n7}"
    },
    "area_codes": ["30", "40", "69", "89", "211", "221", "341", "351", "511", "711"],
    "subscriber_length": { "min": 6, "max": 8 },
    "display_format": "0{area_code} {subscriber}"
  }
}
```

### 3.8 Banking

```jsonc
{
  "banking": {
    "account_format": "IBAN",               // "IBAN" | "domestic" | "both"
    "iban": {
      "country_prefix": "DE",
      "length": 22,
      "bban_structure": "{bank_code:8}{account_number:10}",
      "check_digit_algorithm": "mod97"
    },
    "domestic_format": null,                // US: routing + account number
    "bank_names": [
      "Deutsche Bank AG",
      "Commerzbank AG",
      "DZ Bank AG",
      "KfW Bankengruppe",
      "Bayerische Landesbank",
      "Landesbank Baden-Württemberg"
    ],
    "swift_prefix": "DE",
    "payment_systems": ["SEPA", "TARGET2"],
    "settlement_rules": {
      "domestic_transfer_days": 1,
      "international_transfer_days": 2,
      "wire_cutoff_time": "14:00",
      "direct_debit_lead_days": 5
    },
    "kyc_requirements": {
      "id_document_types": ["Personalausweis", "Reisepass", "Aufenthaltserlaubnis"],
      "pep_screening_required": true,
      "beneficial_ownership_threshold": 0.25,
      "enhanced_due_diligence_triggers": ["high_risk_country", "pep", "complex_structure"]
    }
  }
}
```

### 3.9 Business Rules

```jsonc
{
  "business_rules": {
    "invoice": {
      "numbering_format": "RE-{YYYY}-{SEQ:6}",
      "mandatory_fields": ["tax_id", "vat_number", "sequential_number", "issue_date"],
      "retention_years": 10,
      "electronic_invoice_mandatory": true,
      "e_invoice_format": "ZUGFeRD"         // Country-specific: "ZUGFeRD", "FatturaPA", "Factur-X", "UBL"
    },
    "payment_terms": {
      "default_days": 30,
      "common_terms": [14, 30, 45, 60, 90],
      "early_payment_discount": {
        "common_rate": 0.02,
        "common_days": 10
      },
      "late_payment_interest": {
        "statutory_rate": 0.0912,            // Germany: 9.12% p.a. (base + 8pp for B2B)
        "base_rate_reference": "ECB"
      }
    },
    "approval_thresholds": {
      "currency": "EUR",
      "levels": [
        { "up_to": 5000,   "approver": "team_lead" },
        { "up_to": 25000,  "approver": "department_head" },
        { "up_to": 100000, "approver": "director" },
        { "above": 100000, "approver": "board" }
      ]
    },
    "data_privacy": {
      "regulation": "GDPR",
      "pseudonymization_required": true,
      "retention_limits": {
        "employee_data_years": 3,
        "financial_records_years": 10,
        "tax_records_years": 10
      }
    }
  }
}
```

### 3.10 Legal Entities

```jsonc
{
  "legal_entities": {
    "entity_types": [
      { "code": "GmbH",  "name": "Gesellschaft mit beschränkter Haftung", "name_en": "Limited Liability Company", "weight": 0.40 },
      { "code": "AG",    "name": "Aktiengesellschaft",                    "name_en": "Stock Corporation",         "weight": 0.15 },
      { "code": "KG",    "name": "Kommanditgesellschaft",                 "name_en": "Limited Partnership",       "weight": 0.10 },
      { "code": "OHG",   "name": "Offene Handelsgesellschaft",            "name_en": "General Partnership",       "weight": 0.05 },
      { "code": "e.K.",  "name": "eingetragener Kaufmann",                "name_en": "Registered Merchant",       "weight": 0.15 },
      { "code": "GmbH & Co. KG", "name": "GmbH & Co. Kommanditgesellschaft", "name_en": "Limited Partnership with GmbH as GP", "weight": 0.15 }
    ],
    "tax_id_format": {
      "name": "Steuernummer",
      "format": "NNN/NNN/NNNNN",
      "regex": "^[0-9]{3}/[0-9]{3}/[0-9]{5}$"
    },
    "vat_id_format": {
      "name": "Umsatzsteuer-Identifikationsnummer",
      "prefix": "DE",
      "format": "DE NNNNNNNNN",
      "regex": "^DE[0-9]{9}$"
    },
    "registration_authority": "Handelsregister",
    "registration_format": "HRB {NNNNN}"
  }
}
```

### 3.11 Accounting

```jsonc
{
  "accounting": {
    "framework": "ifrs",                    // Primary: "us_gaap" | "ifrs" | "local_gaap"
    "secondary_framework": null,            // For dual-reporting: "us_gaap" | "ifrs"
    "local_gaap_name": "HGB",              // Handelsgesetzbuch
    "chart_of_accounts": {
      "standard": "SKR04",                  // Germany: SKR03, SKR04; France: PCG; US: generic
      "numbering_length": 4,
      "account_ranges": {
        "assets":      { "from": "0", "to": "1" },
        "liabilities": { "from": "2", "to": "3" },
        "revenue":     { "from": "4", "to": "4" },
        "expenses":    { "from": "5", "to": "7" },
        "closing":     { "from": "9", "to": "9" }
      }
    },
    "audit_framework": "isa",               // "isa" | "pcaob" | "dual"
    "regulatory": {
      "sox_applicable": false,
      "local_regulations": ["BilMoG", "HGB"],
      "filing_requirements": [
        { "name": "Jahresabschluss", "name_en": "Annual Financial Statements", "deadline_months_after_ye": 12 },
        { "name": "Konzernabschluss", "name_en": "Consolidated Statements", "deadline_months_after_ye": 4 }
      ]
    }
  }
}
```

### 3.12 Payroll

```jsonc
{
  "payroll": {
    "pay_frequency": "monthly",             // "weekly" | "biweekly" | "semi_monthly" | "monthly"
    "currency": "EUR",
    "statutory_deductions": [
      { "code": "LOHNST",  "name": "Lohnsteuer",             "name_en": "Income Tax",        "type": "progressive" },
      { "code": "SOLI",    "name": "Solidaritätszuschlag",   "name_en": "Solidarity Surcharge", "type": "percentage", "rate": 0.055 },
      { "code": "KiSt",    "name": "Kirchensteuer",          "name_en": "Church Tax",        "type": "percentage", "rate": 0.08, "optional": true },
      { "code": "RV",      "name": "Rentenversicherung",     "name_en": "Pension Insurance",  "type": "percentage", "rate": 0.093 },
      { "code": "KV",      "name": "Krankenversicherung",    "name_en": "Health Insurance",   "type": "percentage", "rate": 0.073 },
      { "code": "AV",      "name": "Arbeitslosenversicherung", "name_en": "Unemployment Insurance", "type": "percentage", "rate": 0.013 },
      { "code": "PV",      "name": "Pflegeversicherung",     "name_en": "Long-Term Care Insurance", "type": "percentage", "rate": 0.017 }
    ],
    "employer_contributions": [
      { "code": "RV_AG",   "name": "Rentenversicherung (AG)", "rate": 0.093 },
      { "code": "KV_AG",   "name": "Krankenversicherung (AG)", "rate": 0.073 },
      { "code": "AV_AG",   "name": "Arbeitslosenversicherung (AG)", "rate": 0.013 },
      { "code": "PV_AG",   "name": "Pflegeversicherung (AG)", "rate": 0.017 },
      { "code": "BG",      "name": "Berufsgenossenschaft",    "rate": 0.013, "description": "Employer-only: occupational insurance" }
    ],
    "minimum_wage": {
      "hourly": 12.41,
      "currency": "EUR",
      "effective_date": "2024-01-01"
    },
    "working_hours": {
      "standard_weekly": 40.0,
      "max_daily": 10.0,
      "statutory_annual_leave_days": 20,
      "common_annual_leave_days": 30
    },
    "thirteenth_month": false,              // true for many European/Latin American countries
    "severance": {
      "statutory": true,
      "formula": "0.5 * monthly_salary * years_of_service"
    }
  }
}
```

### 3.13 Vendor Templates

```jsonc
{
  "vendor_templates": {
    "name_patterns": [
      "{industry_word} {entity_suffix}",
      "{city} {industry_word} {entity_suffix}",
      "{last_name} & {last_name} {entity_suffix}"
    ],
    "industry_words": {
      "manufacturing": ["Maschinenbau", "Präzisionsteile", "Werkzeug", "Fertigungstechnik", "Metallverarbeitung"],
      "services":      ["Beratung", "Dienstleistungen", "Logistik", "Consulting"],
      "technology":    ["Systemtechnik", "Softwareentwicklung", "Datentechnik", "Digitale Lösungen"],
      "supplies":      ["Bürobedarf", "Industriebedarf", "Verbrauchsmaterial"]
    }
  }
}
```

### 3.14 Customer Templates

```jsonc
{
  "customer_templates": {
    "name_patterns": [
      "{industry_word} {entity_suffix}",
      "{city} {industry_word}",
      "{last_name} {entity_suffix}"
    ],
    "industry_words": {
      "retail":        ["Handelsgesellschaft", "Warenhaus", "Einzelhandel", "Versandhaus"],
      "enterprise":    ["Konzern", "Holding", "Gruppe", "Unternehmensgruppe"],
      "public_sector": ["Stadtverwaltung", "Landratsamt", "Behörde"]
    }
  }
}
```

### 3.15 Material Templates

```jsonc
{
  "material_templates": {
    "categories": {
      "raw_materials":   ["Stahlblech", "Aluminiumprofile", "Kunststoffgranulat", "Kupferdraht"],
      "components":      ["Getriebe", "Hydraulikzylinder", "Steuerungsmodul", "Sensoreinheit"],
      "finished_goods":  ["Werkzeugmaschine", "Förderanlage", "Prüfstand"],
      "consumables":     ["Kühlschmierstoff", "Schleifmittel", "Dichtungen", "Schrauben"],
      "office_supplies": ["Druckerpapier", "Toner", "Büromöbel"]
    },
    "unit_of_measure_labels": {
      "each": "Stück",
      "kg":   "Kilogramm",
      "liter": "Liter",
      "meter": "Meter",
      "box":  "Karton"
    }
  }
}
```

### 3.16 Document Texts

```jsonc
{
  "document_texts": {
    "purchase_order": {
      "header_templates": [
        "Bestellung Nr. {po_number} — {vendor_name}",
        "Rahmenbestellung gemäß Vertrag {contract_ref}"
      ],
      "line_descriptions": [
        "Lieferung {material} gemäß Spezifikation",
        "Wartungsservice für {asset}",
        "Beratungsleistung {service_description}"
      ]
    },
    "invoice": {
      "header_templates": [
        "Rechnung Nr. {invoice_number}",
        "Gutschrift Nr. {credit_note_number}"
      ]
    },
    "journal_entry": {
      "posting_texts": [
        "Buchung {doc_type} {doc_number}",
        "Abgrenzung {period}",
        "Periodenabschluss {period}"
      ]
    }
  }
}
```

---

## 4. Open-Source vs. Commercial Tier Strategy

### 4.1 Tier Definitions

| Aspect | Open-Source (Community) | Commercial (Enterprise) |
|---|---|---|
| **Bundled countries** | US, DE, GB (3 packs compiled in) | 50+ country packs as downloadable add-on |
| **Pack depth** | Full schema coverage for bundled countries | Full schema + regional sub-divisions (e.g., US state taxes, German Bundesland holidays) |
| **Fallback pack** | `_default.json` with English-generic values | Same, plus `_regional_defaults/` (EMEA, APAC, AMERICAS) |
| **Custom packs** | Users can author and load their own JSON files | Same, plus GUI pack editor (future) |
| **Tax data** | Simplified (headline rates only) | Detailed brackets, treaty rates, special regimes |
| **Payroll** | Basic statutory deductions | Full payroll calculation with ceiling, brackets, exceptions |
| **Updates** | Community-contributed, best-effort | Commercially maintained, annual updates for rate changes |
| **Validation** | Schema validation only | Schema + cross-field consistency checks (e.g., VAT rate matches tax bracket structure) |
| **Support** | GitHub Issues | Dedicated support channel |

### 4.2 Open-Source Bundled Packs

The following three countries provide representative coverage across major regions and accounting regimes:

| Country | Rationale |
|---|---|
| **US** | Largest user base, US GAAP, USD, English baseline |
| **DE** | IFRS/HGB, EUR, European regulatory complexity (GDPR, ZUGFeRD, SEPA), non-English locale |
| **GB** | IFRS, GBP, Commonwealth conventions, different date/number formats from US |

Plus `_default.json` as the universal fallback providing English-language generic values for every schema section.

### 4.3 Commercial Pack Roadmap

**Phase 1 — Major economies (15 countries):**
US, GB, DE, FR, NL, CH, JP, CN, IN, AU, CA, BR, MX, SG, KR

**Phase 2 — Extended coverage (25 more):**
IT, ES, PT, AT, BE, SE, NO, DK, FI, PL, CZ, IE, IL, AE, SA, ZA, NZ, TH, MY, ID, PH, VN, TW, HK, CL

**Phase 3 — Comprehensive (10+ more):**
Remaining G20 + OECD + high-demand markets

### 4.4 Licensing Model

```
datasynth-data (OSS, Apache-2.0)
├── Built-in: _default.json, US.json, DE.json, GB.json
└── Loads from: $DATASYNTH_COUNTRY_PACKS_DIR/

datasynth-country-packs-enterprise (Commercial license)
├── 50+ country JSON files
├── Annual update subscription
└── Installed to: /opt/datasynth/country-packs/ (or user-chosen path)
```

---

## 5. Runtime Integration

### 5.1 New Config Fields

```yaml
# Added to global config section
country_packs:
  # Directory containing additional country pack JSON files
  # Overrides/supplements built-in packs
  external_dir: "/opt/datasynth/country-packs"

  # Explicit inline overrides per country (highest priority)
  overrides:
    US:
      tax:
        corporate_income_tax:
          standard_rate: 0.21
    DE:
      payroll:
        minimum_wage:
          hourly: 12.82
          effective_date: "2025-01-01"
```

### 5.2 Core Data Structures (Rust)

```rust
// crates/datasynth-core/src/country/mod.rs

/// Registry that manages loaded country packs.
pub struct CountryPackRegistry {
    packs: HashMap<CountryCode, CountryPack>,
    default_pack: CountryPack,
    external_dir: Option<PathBuf>,
}

/// ISO 3166-1 alpha-2 country code.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CountryCode(pub String);  // e.g., "US", "DE"

/// Complete country configuration, deserialized from JSON.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CountryPack {
    pub schema_version: String,
    pub country_code: String,
    pub country_name: String,
    pub region: String,

    pub locale: LocaleConfig,
    pub names: NamesConfig,
    pub holidays: HolidaysConfig,
    pub tax: TaxConfig,
    pub address: AddressConfig,
    pub phone: PhoneConfig,
    pub banking: BankingConfig,
    pub business_rules: BusinessRulesConfig,
    pub legal_entities: LegalEntitiesConfig,
    pub accounting: AccountingConfig,
    pub payroll: PayrollConfig,
    pub vendor_templates: TemplatesConfig,
    pub customer_templates: TemplatesConfig,
    pub material_templates: MaterialTemplatesConfig,
    pub document_texts: DocumentTextsConfig,
}

impl CountryPackRegistry {
    /// Load built-in packs + external directory + user overrides.
    pub fn new(
        external_dir: Option<&Path>,
        overrides: &HashMap<String, serde_json::Value>,
    ) -> Result<Self, CountryPackError>;

    /// Get the merged config for a country code.
    /// Falls back to _default for missing sections.
    pub fn get(&self, code: &CountryCode) -> &CountryPack;

    /// List all available country codes.
    pub fn available_countries(&self) -> Vec<&CountryCode>;

    /// Validate that all countries referenced in config have packs.
    pub fn validate_config_countries(
        &self,
        companies: &[CompanyConfig],
    ) -> Result<(), Vec<CountryPackError>>;
}
```

### 5.3 Generator Integration Pattern

Before (hardcoded):
```rust
// datasynth-banking/src/generators/customer_generator.rs
fn generate_phone(&self, country: &str) -> String {
    match country {
        "US" | "CA" => format!("+1-555-{:03}-{:04}", ...),
        "GB" => format!("+44-7{:03}-{:06}", ...),
        _ => format!("+{}-{:010}", ...),
    }
}
```

After (country-pack driven):
```rust
fn generate_phone(&self, country_pack: &CountryPack, rng: &mut impl Rng) -> String {
    let phone = &country_pack.phone;
    let area_code = phone.area_codes.choose(rng).unwrap_or(&"000".to_string());
    let subscriber_len = rng.gen_range(phone.subscriber_length.min..=phone.subscriber_length.max);
    let subscriber: String = (0..subscriber_len).map(|_| rng.gen_range(0..=9).to_string()).collect();

    phone.formats.mobile
        .replace("{area_code}", area_code)
        .replace("{subscriber}", &subscriber)
}
```

### 5.4 Holiday Calendar Integration

Before (hardcoded):
```rust
// holidays.rs — 1,450 lines of match arms
fn get_holidays(&self, region: Region, year: i32) -> Vec<Holiday> {
    match region {
        Region::US => vec![ /* 15 hardcoded holidays */ ],
        Region::DE => vec![ /* 12 hardcoded holidays */ ],
        // ... 9 more regions
    }
}
```

After (country-pack driven):
```rust
fn get_holidays(&self, country_pack: &CountryPack, year: i32) -> Vec<Holiday> {
    let mut holidays = Vec::new();

    // Fixed holidays
    for h in &country_pack.holidays.fixed {
        holidays.push(Holiday {
            date: NaiveDate::from_ymd_opt(year, h.month, h.day).unwrap(),
            name: h.name.clone(),
            activity_multiplier: h.activity_multiplier,
        });
    }

    // Easter-relative holidays
    let easter = compute_easter(year);
    for h in &country_pack.holidays.easter_relative {
        holidays.push(Holiday {
            date: easter + Duration::days(h.offset_days as i64),
            name: h.name.clone(),
            activity_multiplier: h.activity_multiplier,
        });
    }

    // Lunar holidays (delegated to lunar calendar calculator)
    for h in &country_pack.holidays.lunar {
        holidays.extend(resolve_lunar_holiday(h, year));
    }

    holidays
}
```

### 5.5 Name Generation Integration

Before:
```rust
// names.rs — hardcoded arrays per NameCulture enum variant
const WESTERN_US_MALE: &[&str] = &["James", "John", "Robert", ...];
```

After:
```rust
fn generate_name(&self, country_pack: &CountryPack, rng: &mut impl Rng) -> PersonName {
    // Pick culture based on weights
    let culture = country_pack.names.cultures
        .choose_weighted(rng, |c| c.weight)
        .unwrap();

    let is_male = rng.gen_bool(0.5);
    let first_names = if is_male { &culture.male_first_names } else { &culture.female_first_names };
    let first = first_names.choose(rng).unwrap().clone();
    let last = culture.last_names.choose(rng).unwrap().clone();

    let display = match culture.name_order.as_str() {
        "eastern" => format!("{} {}", last, first),
        _ => format!("{} {}", first, last),
    };

    PersonName { first_name: first, last_name: last, display_name: display, culture: culture.culture_id.clone(), is_male }
}
```

---

## 6. Migration Plan

### 6.1 Phases

| Phase | Scope | Effort Estimate | Breaking Changes |
|---|---|---|---|
| **Phase 0: Schema & Loader** | Define JSON schema, implement `CountryPackRegistry`, create `_default.json` | Foundation | None — additive only |
| **Phase 1: Extract US** | Convert all US-specific hardcoded values to `US.json`, replace code references | First country pack | Internal refactor, public API unchanged |
| **Phase 2: Extract DE + GB** | Convert German and British data, retire YAML templates | Complete OSS set | Template file paths change |
| **Phase 3: Generator Refactor** | Update all generators to accept `&CountryPack` instead of raw strings/enums | Core integration | Trait signatures change (internal) |
| **Phase 4: Validation & CLI** | `datasynth-data validate` checks country pack presence; `datasynth-data info --countries` lists available packs | User-facing tooling | None |
| **Phase 5: Commercial Packs** | Author packs for Phase 1 countries (15), set up distribution | Commercial offering | None |
| **Phase 6: Template Migration** | Migrate remaining YAML templates (BR, JP, IN) to JSON packs | Cleanup | Template directory restructured |

### 6.2 Backward Compatibility

- **Config files**: Existing YAML configs without `country_packs:` section continue to work; built-in packs are used automatically based on each company's `country:` field
- **Template files**: Existing YAML templates in `examples/templates/` remain functional during migration; both systems coexist with country packs taking priority when present
- **API**: `NameCulture` enum remains available but delegates to country pack data internally
- **CLI**: New `--country-packs-dir` flag added; environment variable `DATASYNTH_COUNTRY_PACKS_DIR` supported as alternative

### 6.3 File Layout After Migration

```
crates/datasynth-core/
├── src/
│   └── country/
│       ├── mod.rs              // CountryPackRegistry, loader, merge logic
│       ├── schema.rs           // CountryPack struct and sub-structs (Deserialize)
│       ├── validation.rs       // JSON schema validation, cross-field checks
│       ├── lunar.rs            // Lunar calendar holiday resolution (extracted from holidays.rs)
│       └── easter.rs           // Easter date computation (extracted from holidays.rs)
└── country-packs/              // Embedded at compile time
    ├── _schema.json            // JSON Schema for validation / editor autocomplete
    ├── _default.json           // Universal fallback
    ├── US.json                 // United States
    ├── DE.json                 // Germany
    └── GB.json                 // United Kingdom
```

---

## 7. JSON Schema Validation

A formal JSON Schema (`_schema.json`) is provided alongside the packs to enable:

1. **Editor autocomplete** when authoring custom packs (VS Code, IntelliJ)
2. **CI validation** of contributed packs
3. **Runtime validation** before generation starts

```jsonc
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://datasynth.dev/schemas/country-pack/v1.json",
  "title": "DataSynth Country Pack",
  "type": "object",
  "required": ["schema_version", "country_code", "country_name", "locale", "names", "holidays"],
  "properties": {
    "schema_version": { "type": "string", "const": "1.0" },
    "country_code":   { "type": "string", "pattern": "^[A-Z]{2}$" },
    "country_name":   { "type": "string" },
    "region":         { "type": "string", "enum": ["AMERICAS", "EMEA", "APAC"] },
    "locale":         { "$ref": "#/$defs/locale" },
    "names":          { "$ref": "#/$defs/names" },
    "holidays":       { "$ref": "#/$defs/holidays" },
    "tax":            { "$ref": "#/$defs/tax" },
    "address":        { "$ref": "#/$defs/address" },
    "phone":          { "$ref": "#/$defs/phone" },
    "banking":        { "$ref": "#/$defs/banking" },
    "business_rules": { "$ref": "#/$defs/business_rules" },
    "legal_entities": { "$ref": "#/$defs/legal_entities" },
    "accounting":     { "$ref": "#/$defs/accounting" },
    "payroll":        { "$ref": "#/$defs/payroll" }
  },
  "$defs": {
    "locale": {
      "type": "object",
      "required": ["language_code", "default_currency", "default_timezone"],
      "properties": {
        "language_code":    { "type": "string", "pattern": "^[a-z]{2}$" },
        "default_currency": { "type": "string", "pattern": "^[A-Z]{3}$" },
        "default_timezone": { "type": "string" },
        "weekend_days":     { "type": "array", "items": { "type": "string", "enum": ["Monday","Tuesday","Wednesday","Thursday","Friday","Saturday","Sunday"] } }
      }
    }
    // ... remaining $defs for each section (omitted for brevity)
  }
}
```

---

## 8. CLI & Tooling Integration

### 8.1 New CLI Commands

```bash
# List available country packs and their sources (built-in vs external)
datasynth-data info --countries

# Output:
# Available Country Packs:
#   US  United States     [built-in]  v1.0  16 sections
#   DE  Germany           [built-in]  v1.0  16 sections
#   GB  United Kingdom    [built-in]  v1.0  16 sections
#   BR  Brazil            [external]  v1.0  16 sections  /opt/datasynth/country-packs/BR.json
#   JP  Japan             [external]  v1.0  14 sections  /opt/datasynth/country-packs/JP.json

# Validate a country pack file
datasynth-data validate --country-pack ./custom/MY.json

# Generate a skeleton country pack for a new country
datasynth-data init --country-pack MY --output ./MY.json
# Creates MY.json pre-filled with _default.json values and country_code set to "MY"

# Show which country-specific values a config references
datasynth-data validate --config config.yaml --check-countries
```

### 8.2 Environment Variables

| Variable | Purpose | Example |
|---|---|---|
| `DATASYNTH_COUNTRY_PACKS_DIR` | Directory for external packs | `/opt/datasynth/country-packs` |
| `DATASYNTH_COUNTRY_PACKS_STRICT` | Fail if a referenced country has no pack (`true`/`false`) | `true` |

---

## 9. Testing Strategy

### 9.1 Unit Tests

| Test | Scope |
|---|---|
| `test_default_pack_loads` | `_default.json` parses without error |
| `test_all_builtin_packs_valid` | Each built-in pack passes schema validation |
| `test_merge_override` | Country pack overrides default values correctly |
| `test_deep_merge_preserves_arrays` | Array fields replace (not append) during merge |
| `test_missing_optional_section_fallback` | Missing `payroll` in country pack falls back to default |
| `test_holiday_resolution` | Fixed, Easter-relative, and lunar holidays resolve to correct dates |
| `test_phone_format_generation` | Phone numbers match the declared regex for each country |
| `test_address_format` | Generated addresses match postal code regex |
| `test_tax_rate_bounds` | All rates are 0.0..1.0, brackets are ascending |
| `test_unknown_country_uses_default` | Requesting a non-existent country code returns default pack |

### 9.2 Integration Tests

| Test | Scope |
|---|---|
| `test_generate_with_us_pack` | Full generation with US company produces US-formatted output |
| `test_generate_multi_country` | Config with US + DE companies uses correct pack per company |
| `test_external_pack_loading` | Pack placed in external dir is discovered and used |
| `test_user_override_applies` | YAML `country_overrides` section modifies pack values |
| `test_backward_compat_no_country_packs_section` | Config without `country_packs:` works with built-in defaults |

### 9.3 Property Tests

| Test | Scope |
|---|---|
| `proptest_any_valid_pack_roundtrips` | Serialize → deserialize → serialize produces identical JSON |
| `proptest_merged_pack_has_all_required_fields` | No merge combination produces a pack with missing required fields |

---

## 10. Open Questions & Future Considerations

| # | Question | Proposed Resolution |
|---|---|---|
| 1 | **Should packs support versioned tax rates?** (e.g., VAT rate changed from 19% to 21% on date X) | Phase 2: Add `effective_date_ranges` to tax brackets. For now, packs represent "current" rates and users override for historical scenarios. |
| 2 | **Sub-national variation?** (US state taxes, German Bundesland holidays, Canadian provincial rules) | Commercial packs include `regional_holidays` and `regional_tax` sections. Open-source packs set these to `null`/`enabled: false`. |
| 3 | **Lunar calendar accuracy** — current approximations in `holidays.rs` drift over time. | Extract lunar algorithms to `country/lunar.rs`; keep algorithmic computation for Chinese/Korean/Hindu calendars rather than hardcoding year-by-year tables. Accuracy to ±1 day is acceptable for synthetic data. |
| 4 | **Should `_default.json` use English-US or truly generic values?** | English-US. It serves as both fallback and the "no country specified" baseline. Naming it "generic" would still effectively be US-English. |
| 5 | **Pack signing for commercial distribution?** | Future: Ed25519 signature in `_signature` field, verified by loader. Not in v1.0. |
| 6 | **How to handle accounting chart-of-accounts country standards (SKR03/04, PCG)?** | Country packs declare the standard and account range mapping. Actual COA generation remains in `coa_generator` but uses the pack's ranges to adjust numbering. |
| 7 | **YAML vs JSON for pack files?** | JSON. Reasons: no code execution risk (unlike YAML anchors/aliases), strict schema validation, smaller parse dependency, and clear boundary between "config" (YAML) and "data" (JSON). |
| 8 | **Should existing YAML templates coexist permanently?** | No. After Phase 6, YAML templates are deprecated. A migration tool converts them to country pack JSON sections. Transition period: 2 major versions. |

---

## 11. Summary

This specification defines a clean separation between **code** (generation logic, orchestration, output formatting) and **country data** (names, holidays, tax rates, formats, legal conventions). The pluggable architecture enables:

- **Open-source users**: Full functionality with US/DE/GB, ability to author custom packs
- **Commercial customers**: Drop-in country packs for 50+ jurisdictions with annual rate updates
- **Contributors**: Add a country by submitting a single JSON file — no Rust knowledge required
- **Maintainability**: Tax rate changes, new holidays, and format updates require no code changes or recompilation

The migration is designed to be **incremental and backward-compatible**, with no breaking changes to the public API or existing configuration files.
