# datasynth-ui

Cross-platform desktop application for synthetic data generation.

## Overview

`datasynth-ui` provides a graphical interface for DataSynth:

- **Visual Configuration**: Comprehensive UI for all configuration sections
- **Real-time Streaming**: Live generation viewer with WebSocket
- **Preset Management**: One-click industry preset application
- **Validation Feedback**: Real-time configuration validation

## Technology Stack

| Component | Technology |
|-----------|------------|
| Backend | Tauri 2.0 (Rust) |
| Frontend | SvelteKit + Svelte 5 |
| Styling | TailwindCSS |
| State | Svelte stores with runes |

## Prerequisites

### Linux (Ubuntu/Debian)

```bash
sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev \
    libappindicator3-dev librsvg2-dev
```

### Linux (Fedora)

```bash
sudo dnf install gtk3-devel webkit2gtk4.1-devel \
    libappindicator-gtk3-devel librsvg2-devel
```

### Linux (Arch)

```bash
sudo pacman -S webkit2gtk-4.1 base-devel curl wget file \
    openssl appmenu-gtk-module gtk3 librsvg libvips
```

### macOS

No additional dependencies required (uses built-in WebKit).

### Windows

WebView2 runtime (usually pre-installed on Windows 10/11).

## Development

```bash
cd crates/datasynth-ui

# Install dependencies
npm install

# Frontend development (no desktop features)
npm run dev

# Desktop app development
npm run tauri dev

# Production build
npm run build
npm run tauri build
```

## Project Structure

```
datasynth-ui/
├── src/                    # Svelte frontend
│   ├── routes/             # SvelteKit pages
│   │   ├── +page.svelte    # Dashboard
│   │   ├── config/         # Configuration pages (15+ sections)
│   │   │   ├── global/
│   │   │   ├── transactions/
│   │   │   ├── master-data/
│   │   │   └── ...
│   │   └── generate/
│   │       └── stream/     # Generation streaming viewer
│   └── lib/
│       ├── components/     # Reusable UI components
│       │   ├── forms/      # Form components
│       │   └── config/     # Config-specific components
│       ├── stores/         # Svelte stores
│       └── utils/          # Utilities
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── lib.rs          # Tauri commands
│   │   └── main.rs         # App entry point
│   └── Cargo.toml
├── e2e/                    # Playwright E2E tests
├── package.json
└── tauri.conf.json
```

## Configuration Sections

| Section | Description |
|---------|-------------|
| Global | Industry, dates, seed, performance |
| Transactions | Line items, amounts, sources |
| Master Data | Vendors, customers, materials |
| Document Flows | P2P, O2C configuration |
| Financial | Balance, subledger, FX, period close |
| Compliance | Fraud, controls, approval |
| Analytics | Graph export, anomaly, data quality |
| Output | Formats, compression |

## Key Components

### Config Store

```typescript
// src/lib/stores/config.ts
import { writable } from 'svelte/store';

export const config = writable<Config>(defaultConfig);
export const isDirty = writable(false);

export function updateConfig(section: string, value: any) {
    config.update(c => ({...c, [section]: value}));
    isDirty.set(true);
}
```

### Form Components

```svelte
<!-- src/lib/components/forms/InputNumber.svelte -->
<script lang="ts">
  export let value: number;
  export let min: number = 0;
  export let max: number = Infinity;
  export let label: string;
</script>

<label>
  {label}
  <input type="number" bind:value {min} {max} />
</label>
```

### Tauri Commands

```rust
// src-tauri/src/lib.rs
#[tauri::command]
async fn save_config(config: Config) -> Result<(), String> {
    // Save configuration
}

#[tauri::command]
async fn start_generation(config: Config) -> Result<(), String> {
    // Start generation via datasynth-runtime
}
```

## Server Connection

The UI connects to `datasynth-server` for streaming:

```bash
# Start server first
cargo run -p datasynth-server

# Then run UI
npm run tauri dev
```

Default server URL: `http://localhost:3000`

## Testing

```bash
# Unit tests
npm test

# E2E tests with Playwright
npx playwright test

# E2E with UI
npx playwright test --ui
```

## Build Output

Production builds create platform-specific packages:

| Platform | Output |
|----------|--------|
| Windows | `.msi`, `.exe` |
| macOS | `.dmg`, `.app` |
| Linux | `.deb`, `.AppImage`, `.rpm` |

Located in: `src-tauri/target/release/bundle/`

## UI Features

### Dashboard

- System overview
- Quick stats
- Recent generations

### Configuration Editor

- Visual form editors for all sections
- Real-time validation
- Dirty state tracking
- Export to YAML/JSON

### Streaming Viewer

- Real-time progress
- Entry preview table
- Memory usage graph
- Pause/resume controls

### Preset Selector

- Industry presets
- Complexity levels
- One-click application

## See Also

- [Desktop UI Guide](../user-guide/desktop-ui.md)
- [datasynth-server](datasynth-server.md)
- [Configuration](../configuration/README.md)
