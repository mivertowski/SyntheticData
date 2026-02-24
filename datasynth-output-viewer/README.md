# DataSynth Output Viewer

A React + Vite app to visualize generated DataSynth output: journal entries (JEC), FEC when applicable, master data, fraud/anomaly labels, trial balance, and general ledger.

## Features

- **Journal (JEC / FEC)** – Table of journal entry lines from `journal_entries.csv`. When `fec.csv` exists (French GAAP), a dedicated FEC view (Fichier des Écritures Comptables) is available.
- **Master Data** – Tabs for Vendors, Customers, Materials, Fixed Assets, and Employees from `master_data/*.json`.
- **Fraud & Anomalies** – Table of anomaly/fraud labels plus pie chart by category and bar chart by type (from `labels/anomaly_labels`, `labels/fraud_labels`).
- **Trial Balance** – Period trial balances from `period_close/trial_balances.json` with a **date (period) selector**.
- **General Ledger** – Transactions **per GL account** from journal entries, with account dropdown.

## Prerequisites

- Node.js 18+
- Generated output from `datasynth-data generate` (e.g. in `./output` or `../output_fr`).

## Quick start

1. **Generate data** (from repo root):

   ```bash
   datasynth-data generate --demo --output ./output
   # or with config:
   datasynth-data generate --config config.yaml --output ./output
   ```

2. **Launch the viewer** (from repo root):

   ```bash
   ./scripts/launch_output_viewer.sh          # uses ./output
   ./scripts/launch_output_viewer.sh ../output_fr   # custom output dir
   ```

   Or from this directory:

   ```bash
   cd datasynth-output-viewer
   npm install
   OUTPUT_DIR=../output npm run dev:with-data
   ```

   This copies the output into `public/data` and starts Vite. Open http://localhost:5173.

3. **Alternative (manual load):**

   ```bash
   OUTPUT_DIR=../output npm run load-data   # copy once
   npm run dev                              # then start dev server
   ```

## Scripts

| Script | Description |
|--------|-------------|
| `npm run dev` | Start Vite dev server (data must already be in `public/data`) |
| `npm run dev:with-data` | Run `load-data` then `dev` (use `OUTPUT_DIR` to point to output folder) |
| `npm run load-data` | Copy output files into `public/data` (default: `./output`; set `OUTPUT_DIR`) |
| `npm run build` | Production build |
| `npm run preview` | Preview production build |
| `npm run preview:with-data` | Load data, build, then preview |

## Data directory

- **Default:** The app expects data under `/data` (i.e. files in `public/data/` after `load-data`).
- **Custom path:** Set `OUTPUT_DIR` when running `load-data`, e.g. `OUTPUT_DIR=../output_fr npm run load-data`.

Expected structure (after copying into `public/data`):

- `journal_entries.csv` – flat journal entry lines
- `fec.csv` – optional; French FEC export
- `master_data/vendors.json`, `customers.json`, `materials.json`, `fixed_assets.json`, `employees.json`
- `labels/anomaly_labels.csv` or `labels/anomaly_labels.json`
- `labels/fraud_labels.csv` or `labels/fraud_labels.json` (optional)
- `period_close/trial_balances.json` – for Trial Balance tab

## Tech stack

- React 19, TypeScript, Vite 7
- Recharts (charts), PapaParse (CSV)
