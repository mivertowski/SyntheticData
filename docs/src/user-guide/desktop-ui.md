# Desktop UI

DataSynth includes a cross-platform desktop application built with Tauri and SvelteKit.

## Overview

The desktop UI provides:
- Visual configuration editing
- Industry preset selection
- Real-time generation monitoring
- Configuration validation feedback

## Installation

### Prerequisites

| Requirement | Version |
|-------------|---------|
| Node.js | 18+ |
| npm | 9+ |
| Rust | 1.88+ |
| Platform dependencies | See below |

### Platform Dependencies

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get install libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

**macOS:**
No additional dependencies required.

**Windows:**
WebView2 runtime (usually pre-installed on Windows 10/11).

### Running in Development

```bash
cd crates/datasynth-ui
npm install
npm run tauri dev
```

### Building for Production

```bash
cd crates/datasynth-ui
npm run tauri build
```

Build outputs are in `crates/datasynth-ui/src-tauri/target/release/bundle/`.

## Application Layout

### Dashboard

The main dashboard provides:
- Quick stats overview
- Recent generation history
- System status

### Configuration Editor

Access via the sidebar. Configuration is organized into sections:

| Section | Contents |
|---------|----------|
| Global | Industry, dates, seed, performance |
| Companies | Company definitions and weights |
| Transactions | Target count, line items, amounts |
| Master Data | Vendors, customers, materials |
| Document Flows | P2P, O2C configuration |
| Financial | Balance, subledger, FX, period close |
| Compliance | Fraud, controls, approval |
| Analytics | Graph export, anomaly, data quality |
| Output | Formats, compression |

### Configuration Sections

#### Global Settings

- **Industry**: Select from presets (manufacturing, retail, etc.)
- **Start Date**: Beginning of simulation period
- **Period Months**: Duration (1-120 months)
- **Group Currency**: Base currency for consolidation
- **Random Seed**: For reproducible generation

#### Chart of Accounts

- **Complexity**: Small (~100), Medium (~400), Large (~2500) accounts
- **Structure**: Industry-specific account hierarchies

#### Transactions

- **Target Count**: Number of journal entries to generate
- **Line Item Distribution**: Configure line count probabilities
- **Amount Distribution**: Log-normal parameters, round number bias

#### Master Data

Configure generation parameters for:
- Vendors (count, payment terms, intercompany flags)
- Customers (count, credit terms, payment behavior)
- Materials (count, valuation methods)
- Fixed Assets (count, depreciation methods)
- Employees (count, hierarchy depth)

#### Document Flows

- **P2P (Procure-to-Pay)**: PO → GR → Invoice → Payment rates
- **O2C (Order-to-Cash)**: SO → Delivery → Invoice → Receipt rates
- **Three-Way Match**: Tolerance settings

#### Financial Settings

- **Balance**: Opening balance configuration
- **Subledger**: AR, AP, FA, Inventory settings
- **FX**: Currency pairs, rate volatility
- **Period Close**: Accrual, depreciation, closing settings

#### Compliance

- **Fraud**: Enable/disable, fraud rate, fraud types
- **Controls**: Internal control definitions
- **Approval**: Threshold configuration, SoD rules

#### Analytics

- **Graph Export**: Format selection (PyTorch Geometric, Neo4j, DGL)
- **Anomaly Injection**: Rate, types, labeling
- **Data Quality**: Missing values, format variations, duplicates

#### Output Settings

- **Format**: CSV or JSON
- **Compression**: None, gzip, or zstd
- **File Organization**: Directory structure options

### Preset Selector

Quickly load industry presets:

1. Click "Load Preset" in the header
2. Select industry
3. Choose complexity level
4. Click "Apply"

### Real-time Streaming

During generation, view:
- Progress bar with percentage
- Entries per second
- Memory usage
- Recent entries table

Access streaming view via "Generate" → "Stream".

### Validation

The UI validates configuration in real-time:
- Required fields are highlighted
- Invalid values show error messages
- Distribution weights are checked
- Constraints are enforced

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl/Cmd + S` | Save configuration |
| `Ctrl/Cmd + G` | Start generation |
| `Ctrl/Cmd + ,` | Open settings |
| `Escape` | Close modal |

## Configuration Files

The UI stores configurations in:

| Platform | Location |
|----------|----------|
| Linux | `~/.config/datasynth-data/` |
| macOS | `~/Library/Application Support/datasynth-data/` |
| Windows | `%APPDATA%\datasynth-data\` |

## Exporting Configuration

To use your configuration with the CLI:

1. Configure in the UI
2. Click "Export" → "Export YAML"
3. Save the `.yaml` file
4. Use with CLI: `datasynth-data generate --config exported.yaml --output ./output`

## Development

### Project Structure

```
crates/datasynth-ui/
├── src/                      # SvelteKit frontend
│   ├── routes/               # Page routes
│   │   ├── +page.svelte      # Dashboard
│   │   ├── generate/         # Generation views
│   │   └── config/           # Configuration pages
│   └── lib/
│       ├── stores/           # State management
│       └── components/       # Reusable components
├── src-tauri/                # Rust backend
│   └── src/
│       └── main.rs           # Tauri commands
├── package.json
└── tauri.conf.json
```

### Adding a Configuration Page

1. Create route in `src/routes/config/<section>/+page.svelte`
2. Add form components
3. Connect to config store
4. Add navigation link

### Debugging

```bash
# Enable Tauri dev tools
npm run tauri dev

# View browser console (Ctrl/Cmd + Shift + I in dev mode)
```

## Troubleshooting

### UI Doesn't Start

```bash
# Check Node dependencies
npm install

# Rebuild native modules
npm run tauri clean
npm run tauri build
```

### Configuration Not Saving

Check file permissions in the config directory.

### WebSocket Connection Failed

Ensure the server is running if using streaming features:
```bash
cargo run -p datasynth-server -- --port 3000
```

## See Also

- [CLI Reference](cli-reference.md)
- [Server API](server-api.md)
- [Configuration](../configuration/README.md)
