# Country Packs

Country packs provide pluggable, JSON-based country-specific configuration for holidays, names, tax rates, addresses, phone formats, payroll rules, banking, and more.

## Overview

DataSynth ships with a layered country pack system:

1. **`_default.json`** — baseline values for all countries
2. **Country pack** (e.g., `US.json`, `DE.json`, `GB.json`) — overrides specific to that country
3. **User overrides** — surgical patches applied via the `country_packs.overrides` config section

Objects merge recursively (overlay keys win), while arrays and scalars replace entirely. This means you can override a single tax rate without re-specifying the entire tax section.

## Built-in Packs

| Pack | Description |
|------|-------------|
| `_default.json` | Baseline defaults for all sections (English locale, USD currency, generic names) |
| `US.json` | United States — federal/state holidays, SSN format, US GAAP, state tax rates, USD formatting |
| `DE.json` | Germany — German holidays (incl. regional), IBAN/BIC, IFRS/HGB, trade tax, EUR formatting |
| `GB.json` | United Kingdom — UK bank holidays, NI number, FCA/IFRS, GBP formatting, PAYE payroll |

All built-in packs are embedded via `include_str!` — no external files needed.

## Configuration

```yaml
country_packs:
  external_dir: ./my-country-packs    # Optional: directory with additional JSON packs
  overrides:                           # Optional: surgical JSON overrides per country
    US:
      tax:
        corporate_income:
          standard_rate: 0.25
    DE:
      payroll:
        statutory_deductions:
          church_tax_rate: 0.09
```

### Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `external_dir` | `string` (path) | `null` | Directory containing additional `.json` country pack files |
| `overrides` | `map<string, object>` | `{}` | Per-country JSON overrides applied after pack loading |

Override keys must be 2-letter ISO 3166-1 alpha-2 country codes (e.g., `US`, `DE`, `GB`, `FR`).

## How Country Packs Are Resolved

When the orchestrator generates data for a company with `country: DE`:

1. Load `_default.json` as the base
2. Deep-merge `DE.json` on top (German holidays replace default holidays, German names extend defaults, etc.)
3. Apply any `overrides.DE` from config on top
4. Pass the resolved `CountryPack` to generators

If no pack exists for a country code, the default pack is used as-is.

## JSON Schema (16 Sections)

Each country pack JSON file contains up to 16 sections:

| Section | Description |
|---------|-------------|
| `locale` | Language, currency, number/date formats, timezone, fiscal year, weekend days |
| `names` | Person name pools by culture, email domains, username patterns |
| `holidays` | Fixed dates, Easter-relative, nth-weekday, last-weekday, and lunar holidays |
| `tax` | Corporate income, VAT/GST, withholding, payroll tax brackets, subnational taxes |
| `address` | Format template, components (street, city, state, postal), postal code patterns |
| `phone` | Country code, format patterns, area codes, subscriber digit lengths |
| `banking` | IBAN format, domestic bank format, settlement rules, KYC requirements, bank names |
| `business_rules` | Invoice rules, payment terms, approval thresholds, privacy regulations |
| `legal_entities` | Entity type suffixes (LLC, GmbH, Ltd), tax/VAT ID formats, registration |
| `accounting` | Framework (US GAAP / IFRS), COA standards, regulatory bodies |
| `payroll` | Pay frequency, statutory deductions, minimum wage, working hours, leave entitlements |
| `vendor_templates` | Name patterns, industry-specific words for vendor generation |
| `customer_templates` | Name patterns, industry-specific words for customer generation |
| `material_templates` | Material categories, unit-of-measure labels |
| `document_texts` | Invoice header templates, line descriptions, posting text patterns |

All sections use `#[serde(default)]`, so a country pack only needs to specify the sections it wants to override.

## Holiday Types

Country packs support 5 holiday resolution algorithms:

| Type | Example | Description |
|------|---------|-------------|
| **Fixed** | Jan 1 — New Year's Day | Same date every year |
| **Easter-relative** | Good Friday (Easter − 2) | Offset from Easter Sunday (computed via anonymous Gregorian algorithm) |
| **Nth weekday** | 3rd Monday in January — MLK Day | Nth occurrence of a weekday in a given month, with optional `offset_days` (e.g., Day after Thanksgiving = 4th Thursday + 1) |
| **Last weekday** | Last Monday in May — Memorial Day | Last occurrence of a weekday in a given month |
| **Lunar** | Chinese New Year, Diwali, Eid al-Fitr | Algorithmic computation via `lunar.rs` for Chinese, Hindu, Islamic calendars |

Each holiday can specify `observe_on_nearest_weekday: true` to shift Saturday holidays to Friday and Sunday holidays to Monday.

## Creating a Custom Country Pack

Create a JSON file named with the 2-letter country code (e.g., `FR.json`):

```json
{
  "schema_version": "1.0",
  "country_code": "FR",
  "country_name": "France",
  "region": "EMEA",
  "locale": {
    "language_code": "fr",
    "language_name": "French",
    "default_currency": "EUR",
    "currency_symbol": "€",
    "default_timezone": "Europe/Paris"
  },
  "holidays": {
    "fixed": [
      { "month": 1, "day": 1, "name": "Jour de l'An" },
      { "month": 5, "day": 1, "name": "Fête du Travail" },
      { "month": 7, "day": 14, "name": "Fête nationale" },
      { "month": 11, "day": 11, "name": "Armistice" },
      { "month": 12, "day": 25, "name": "Noël" }
    ],
    "easter_relative": [
      { "offset_days": -2, "name": "Vendredi saint" },
      { "offset_days": 1, "name": "Lundi de Pâques" },
      { "offset_days": 39, "name": "Ascension" },
      { "offset_days": 50, "name": "Lundi de Pentecôte" }
    ]
  },
  "tax": {
    "corporate_income": {
      "standard_rate": 0.25
    },
    "vat": {
      "standard_rate": 0.20,
      "reduced_rates": [0.10, 0.055, 0.021]
    }
  }
}
```

Place it in the directory specified by `country_packs.external_dir`, and it will be loaded automatically.

## Multi-Country Company Setup

```yaml
global:
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 12

companies:
  - code: "1000"
    name: "US Headquarters"
    currency: USD
    country: US            # → loads US.json country pack
    volume_weight: 0.4
    is_parent: true

  - code: "2000"
    name: "German Operations GmbH"
    currency: EUR
    country: DE            # → loads DE.json country pack
    volume_weight: 0.35
    parent_code: "1000"

  - code: "3000"
    name: "UK Limited"
    currency: GBP
    country: GB            # → loads GB.json country pack
    volume_weight: 0.25
    parent_code: "1000"

country_packs:
  overrides:
    DE:
      tax:
        corporate_income:
          trade_tax_rate: 0.14    # Override Gewerbesteuer rate
```

Each company automatically resolves its country pack from the `country` field. Generators receive the appropriate pack for culture-aware names, local holidays, tax rates, address formats, and payroll rules.

## Generator Integration

The following generators consume country pack data:

| Generator | Country Pack Usage |
|-----------|--------------------|
| `HolidayCalendar` | `from_country_pack()` — resolves all 5 holiday types |
| `MultiCultureNameGenerator` | `from_country_pack()` — culture-weighted person names |
| `TaxCodeGenerator` | `generate_from_country_pack()` — tax rates, jurisdictions, states |
| `PayrollGenerator` | `generate_with_country_pack()` — statutory deduction rates |
| `EmissionGenerator` | `spend_emission_factor_from_pack()` — country emission multipliers |
| `CustomerGenerator` | `generate_phone_from_pack()`, `generate_address_from_pack()`, `generate_national_id_from_pack()` |

## See Also

- [Companies](companies.md) — the `country` field drives country pack resolution
- [Global Settings](global-settings.md)
- [datasynth-core](../crates/datasynth-core.md) — `country/` module implementation details
