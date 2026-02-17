# Companies

Company configuration defines the legal entities for data generation.

## Configuration

```yaml
companies:
  - code: "1000"
    name: "Headquarters"
    currency: USD
    country: US
    volume_weight: 0.6
    is_parent: true
    parent_code: null

  - code: "2000"
    name: "European Subsidiary"
    currency: EUR
    country: DE
    volume_weight: 0.4
    is_parent: false
    parent_code: "1000"
```

## Fields

### code

Unique identifier for the company.

| Property | Value |
|----------|-------|
| Type | `string` |
| Required | Yes |
| Constraints | Unique across all companies |

```yaml
companies:
  - code: "1000"      # Four-digit SAP-style
  - code: "US01"      # Region-based
  - code: "HQ"        # Abbreviated
```

### name

Display name for the company.

| Property | Value |
|----------|-------|
| Type | `string` |
| Required | Yes |

```yaml
companies:
  - name: "Headquarters"
  - name: "European Operations GmbH"
  - name: "Asia Pacific Holdings"
```

### currency

Local currency for the company.

| Property | Value |
|----------|-------|
| Type | `string` (ISO 4217) |
| Required | Yes |

```yaml
companies:
  - currency: USD
  - currency: EUR
  - currency: CHF
  - currency: JPY
```

**Used for:**
- Transaction amounts
- Local reporting
- FX translation

### country

Country code for the company.

| Property | Value |
|----------|-------|
| Type | `string` (ISO 3166-1 alpha-2) |
| Required | Yes |

```yaml
companies:
  - country: US
  - country: DE
  - country: CH
  - country: JP
```

**Affects:**
- **Country pack resolution** — the `CountryPackRegistry` automatically loads the matching country pack (e.g., `US.json` for `country: US`), providing holidays, names, tax rates, address formats, phone formats, payroll rules, and banking configuration. See [Country Packs](country-packs.md).
- Holiday calendars
- Tax calculations
- Regional templates

### volume_weight

Relative transaction volume for this company.

| Property | Value |
|----------|-------|
| Type | `f64` |
| Required | Yes |
| Range | 0.0 - 1.0 |
| Constraint | Sum across all companies = 1.0 |

```yaml
companies:
  - code: "1000"
    volume_weight: 0.5    # 50% of transactions

  - code: "2000"
    volume_weight: 0.3    # 30% of transactions

  - code: "3000"
    volume_weight: 0.2    # 20% of transactions
```

### is_parent

Whether this company is the consolidation parent.

| Property | Value |
|----------|-------|
| Type | `bool` |
| Required | No |
| Default | `false` |

```yaml
companies:
  - code: "1000"
    is_parent: true       # Consolidation parent

  - code: "2000"
    is_parent: false      # Subsidiary
```

**Notes:**
- Exactly one company should be `is_parent: true` for consolidation
- Parent receives elimination entries

### parent_code

Reference to parent company for subsidiaries.

| Property | Value |
|----------|-------|
| Type | `string` or `null` |
| Required | No |
| Default | `null` |

```yaml
companies:
  - code: "1000"
    is_parent: true
    parent_code: null     # No parent (is the parent)

  - code: "2000"
    is_parent: false
    parent_code: "1000"   # Owned by 1000

  - code: "3000"
    is_parent: false
    parent_code: "2000"   # Owned by 2000 (nested)
```

## Examples

### Single Company

```yaml
companies:
  - code: "1000"
    name: "Demo Company"
    currency: USD
    country: US
    volume_weight: 1.0
```

### Multi-National

```yaml
companies:
  - code: "1000"
    name: "Global Holdings Inc"
    currency: USD
    country: US
    volume_weight: 0.4
    is_parent: true

  - code: "2000"
    name: "European Operations GmbH"
    currency: EUR
    country: DE
    volume_weight: 0.25
    parent_code: "1000"

  - code: "3000"
    name: "UK Limited"
    currency: GBP
    country: GB
    volume_weight: 0.15
    parent_code: "2000"

  - code: "4000"
    name: "Asia Pacific Pte Ltd"
    currency: SGD
    country: SG
    volume_weight: 0.2
    parent_code: "1000"
```

### Regional Structure

```yaml
companies:
  - code: "HQ"
    name: "Headquarters"
    currency: USD
    country: US
    volume_weight: 0.3
    is_parent: true

  - code: "NA01"
    name: "North America Operations"
    currency: USD
    country: US
    volume_weight: 0.3
    parent_code: "HQ"

  - code: "EU01"
    name: "EMEA Operations"
    currency: EUR
    country: DE
    volume_weight: 0.25
    parent_code: "HQ"

  - code: "AP01"
    name: "APAC Operations"
    currency: JPY
    country: JP
    volume_weight: 0.15
    parent_code: "HQ"
```

## Validation

| Check | Rule |
|-------|------|
| `code` | Must be unique |
| `volume_weight` | Sum must equal 1.0 (±0.01) |
| `parent_code` | Must reference existing company or be null |
| `is_parent` | At most one true (if intercompany enabled) |

## Intercompany Implications

When multiple companies exist:
- Intercompany transactions generated between companies
- FX rates generated for currency pairs
- Elimination entries created for parent
- Transfer pricing applied

See [Intercompany Processing](../advanced/intercompany.md) for details.

## See Also

- [Global Settings](global-settings.md)
- [Intercompany Processing](../advanced/intercompany.md)
- [FX Settings](financial-settings.md)
