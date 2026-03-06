# ERP Output Formats

DataSynth can export data in native ERP table formats, enabling direct load testing and integration validation against SAP S/4HANA, Oracle EBS, and NetSuite environments.

## Overview

The `datasynth-output` crate provides three ERP-specific exporters alongside the standard CSV/JSON/Parquet sinks. Each exporter transforms the internal data model into the target ERP's table schema with correct field names, data types, and referential integrity.

| ERP System | Exporter | Tables |
|------------|----------|--------|
| SAP S/4HANA | `SapExporter` | BKPF, BSEG, ACDOCA, LFA1, KNA1, MARA, CSKS, CEPC |
| Oracle EBS | `OracleExporter` | GL_JE_HEADERS, GL_JE_LINES, GL_JE_BATCHES |
| NetSuite | `NetSuiteExporter` | Journal entries with subsidiary, multi-book, custom fields |

## SAP S/4HANA

### Supported Tables

| Table | Description | Source Data |
|-------|-------------|-------------|
| **BKPF** | Document Header | Journal entry headers |
| **BSEG** | Document Line Items | Journal entry line items |
| **ACDOCA** | Universal Journal | Full ACDOCA event records |
| **LFA1** | Vendor Master | Vendor records |
| **KNA1** | Customer Master | Customer records |
| **MARA** | Material Master | Material records |
| **CSKS** | Cost Center Master | Cost center assignments |
| **CEPC** | Profit Center Master | Profit center assignments |

### BKPF Fields (Document Header)

| SAP Field | Description | Example |
|-----------|-------------|---------|
| `MANDT` | Client | `100` |
| `BUKRS` | Company Code | `1000` |
| `BELNR` | Document Number | `0100000001` |
| `GJAHR` | Fiscal Year | `2024` |
| `BLART` | Document Type | `SA` (G/L posting) |
| `BLDAT` | Document Date | `2024-01-15` |
| `BUDAT` | Posting Date | `2024-01-15` |
| `MONAT` | Fiscal Period | `1` |
| `CPUDT` | Entry Date | `2024-01-15` |
| `CPUTM` | Entry Time | `10:30:00` |
| `USNAM` | User Name | `JSMITH` |

### BSEG Fields (Line Items)

| SAP Field | Description | Example |
|-----------|-------------|---------|
| `MANDT` | Client | `100` |
| `BUKRS` | Company Code | `1000` |
| `BELNR` | Document Number | `0100000001` |
| `GJAHR` | Fiscal Year | `2024` |
| `BUZEI` | Line Item | `001` |
| `BSCHL` | Posting Key | `40` (debit) / `50` (credit) |
| `HKONT` | GL Account | `1100` |
| `DMBTR` | Amount in Local Currency | `1000.00` |
| `WRBTR` | Amount in Doc Currency | `1000.00` |
| `KOSTL` | Cost Center | `CC100` |
| `PRCTR` | Profit Center | `PC100` |

### ACDOCA Fields (Universal Journal)

The ACDOCA format includes all standard SAP Universal Journal fields plus simulation metadata:

| Field | Description |
|-------|-------------|
| `RCLNT` | Client |
| `RLDNR` | Ledger |
| `RBUKRS` | Company Code |
| `GJAHR` | Fiscal Year |
| `BELNR` | Document Number |
| `DOCLN` | Line Item |
| `POPER` | Posting Period |
| `RACCT` | Account |
| `DRCRK` | Debit/Credit Indicator |
| `HSL` | Amount in Local Currency |
| `ZSIM_*` | Simulation metadata fields |

### Configuration

```yaml
output:
  format: sap
  sap:
    tables:
      - bkpf
      - bseg
      - acdoca
      - lfa1
      - kna1
      - mara
    client: "100"
    ledger: "0L"
```

---

## Oracle EBS

### Supported Tables

| Table | Description | Source Data |
|-------|-------------|-------------|
| **GL_JE_HEADERS** | Journal Entry Headers | Journal entry headers |
| **GL_JE_LINES** | Journal Entry Lines | Journal entry line items |
| **GL_JE_BATCHES** | Journal Entry Batches | Batch groupings |

### GL_JE_HEADERS Fields

| Oracle Field | Description | Example |
|-------------|-------------|---------|
| `JE_HEADER_ID` | Unique Header ID | `10001` |
| `LEDGER_ID` | Ledger ID | `1` |
| `JE_BATCH_ID` | Batch ID | `5001` |
| `PERIOD_NAME` | Period Name | `JAN-24` |
| `NAME` | Journal Name | `Manual Entry 001` |
| `JE_CATEGORY` | Category | `MANUAL`, `ADJUSTMENT`, `PAYABLES` |
| `JE_SOURCE` | Source | `MANUAL`, `PAYABLES`, `RECEIVABLES` |
| `CURRENCY_CODE` | Currency | `USD` |
| `ACTUAL_FLAG` | Type | `A` (Actual), `B` (Budget), `E` (Encumbrance) |
| `STATUS` | Status | `P` (Posted), `U` (Unposted) |
| `DEFAULT_EFFECTIVE_DATE` | Effective Date | `2024-01-15` |
| `RUNNING_TOTAL_DR` | Total Debits | `10000.00` |
| `RUNNING_TOTAL_CR` | Total Credits | `10000.00` |
| `PARENT_JE_HEADER_ID` | Parent (for reversals) | `null` |
| `ACCRUAL_REV_FLAG` | Reversal Flag | `Y` / `N` |

### GL_JE_LINES Fields

| Oracle Field | Description | Example |
|-------------|-------------|---------|
| `JE_HEADER_ID` | Header Reference | `10001` |
| `JE_LINE_NUM` | Line Number | `1` |
| `CODE_COMBINATION_ID` | Account Combo ID | `10110` |
| `ENTERED_DR` | Entered Debit | `1000.00` |
| `ENTERED_CR` | Entered Credit | `0.00` |
| `ACCOUNTED_DR` | Accounted Debit | `1000.00` |
| `ACCOUNTED_CR` | Accounted Credit | `0.00` |
| `DESCRIPTION` | Line Description | `Customer payment` |
| `EFFECTIVE_DATE` | Effective Date | `2024-01-15` |

### Configuration

```yaml
output:
  format: oracle
  oracle:
    ledger_id: 1
    set_of_books_id: 1
```

---

## NetSuite

### Journal Entry Fields

NetSuite export includes support for subsidiaries, multi-book accounting, and custom fields:

| NetSuite Field | Description | Example |
|----------------|-------------|---------|
| `INTERNAL_ID` | Internal ID | `50001` |
| `EXTERNAL_ID` | External ID (for import) | `DS-JE-001` |
| `TRAN_ID` | Transaction Number | `JE00001` |
| `TRAN_DATE` | Transaction Date | `2024-01-15` |
| `POSTING_PERIOD` | Period ID | `Jan 2024` |
| `SUBSIDIARY` | Subsidiary ID | `1` |
| `CURRENCY` | Currency Code | `USD` |
| `EXCHANGE_RATE` | Exchange Rate | `1.000000` |
| `MEMO` | Memo | `Monthly accrual` |
| `APPROVED` | Approval Status | `true` |
| `REVERSAL_DATE` | Reversal Date | `2024-02-01` |
| `DEPARTMENT` | Department ID | `100` |
| `CLASS` | Class ID | `1` |
| `LOCATION` | Location ID | `1` |
| `TOTAL_DEBIT` | Total Debits | `5000.00` |
| `TOTAL_CREDIT` | Total Credits | `5000.00` |

### NetSuite Line Fields

| Field | Description |
|-------|-------------|
| `ACCOUNT` | Account internal ID |
| `DEBIT` | Debit amount |
| `CREDIT` | Credit amount |
| `MEMO` | Line memo |
| `DEPARTMENT` | Department |
| `CLASS` | Class segment |
| `LOCATION` | Location segment |
| `ENTITY` | Customer/Vendor reference |
| `CUSTOM_FIELDS` | Additional custom field map |

### Configuration

```yaml
output:
  format: netsuite
  netsuite:
    subsidiary_id: 1
    include_custom_fields: true
```

---

## Usage Examples

### SAP Load Testing

Generate data for SAP S/4HANA load testing with full table coverage:

```yaml
global:
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 12

transactions:
  target_count: 100000

output:
  format: sap
  sap:
    tables: [bkpf, bseg, acdoca, lfa1, kna1, mara, csks, cepc]
    client: "100"
```

### Oracle EBS Migration Validation

Generate journal entries in Oracle EBS format for migration testing:

```yaml
output:
  format: oracle
  oracle:
    ledger_id: 1
```

### NetSuite Integration Testing

Generate multi-subsidiary data with custom fields:

```yaml
output:
  format: netsuite
  netsuite:
    subsidiary_id: 1
    include_custom_fields: true
```

## Output Files

| Format | Output Files |
|--------|-------------|
| SAP | `sap_bkpf.csv`, `sap_bseg.csv`, `sap_acdoca.csv`, `sap_lfa1.csv`, `sap_kna1.csv`, `sap_mara.csv`, `sap_csks.csv`, `sap_cepc.csv` |
| Oracle | `oracle_gl_je_headers.csv`, `oracle_gl_je_lines.csv`, `oracle_gl_je_batches.csv` |
| NetSuite | `netsuite_journal_entries.csv`, `netsuite_journal_lines.csv` |

## See Also

- [Output Formats](output-formats.md) — Standard CSV/JSON/Parquet output
- [Streaming Output](streaming-output.md) — Real-time streaming sinks
- [Output Settings](../configuration/output-settings.md) — Configuration reference
- [ERP Load Testing](../use-cases/erp-testing.md) — ERP testing use case
- [datasynth-output](../crates/datasynth-output.md) — Crate reference
