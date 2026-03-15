# Development Setup

Set up your local development environment for DataSynth.

## Prerequisites

### Required

- **Rust**: 1.88 or later (install via [rustup](https://rustup.rs/))
- **Git**: For version control
- **Cargo**: Included with Rust

### Optional

- **Node.js 18+**: For desktop UI development (datasynth-ui)
- **Protocol Buffers**: For gRPC development
- **mdBook**: For documentation development

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/mivertowski/SyntheticData.git
cd SyntheticData
```

### 2. Install Rust Toolchain

```bash
# Install rustup if not present
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install stable toolchain
rustup install stable
rustup default stable

# Add useful components
rustup component add clippy rustfmt
```

### 3. Build the Project

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check
```

### 4. Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific crate tests
cargo test -p datasynth-core
cargo test -p datasynth-generators
```

## IDE Setup

### VS Code

Recommended extensions:

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml",
    "serayuzgur.crates",
    "vadimcn.vscode-lldb"
  ]
}
```

Settings for the project:

```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true
}
```

### JetBrains (RustRover/IntelliJ)

1. Install the Rust plugin
2. Open the project directory
3. Configure Cargo settings under Preferences > Languages & Frameworks > Rust

## Desktop UI Setup

For developing the Tauri/SvelteKit desktop UI:

```bash
# Navigate to UI crate
cd crates/datasynth-ui

# Install Node.js dependencies
npm install

# Run development server
npm run dev

# Run Tauri desktop app
npm run tauri dev

# Build production
npm run build
```

## Documentation Setup

For working on documentation:

```bash
# Install mdBook
cargo install mdbook

# Build documentation
cd docs
mdbook build

# Serve with live reload
mdbook serve --open

# Generate Rust API docs
cargo doc --workspace --no-deps --open
```

## Project Structure

```
DataSynth/
├── crates/
│   ├── datasynth-cli/          # CLI binary
│   ├── datasynth-core/         # Core models and traits
│   ├── datasynth-config/       # Configuration schema
│   ├── datasynth-generators/   # Data generators
│   ├── datasynth-output/       # Output sinks
│   ├── datasynth-graph/        # Graph export
│   ├── datasynth-runtime/      # Orchestration
│   ├── datasynth-server/       # REST/gRPC server
│   ├── datasynth-ui/           # Desktop UI
│   └── datasynth-ocpm/         # OCEL 2.0 export
├── benches/                # Benchmarks
├── docs/                   # Documentation
├── configs/                # Example configs
└── templates/              # Data templates
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level (trace, debug, info, warn, error) | `info` |
| `SYNTH_CONFIG_PATH` | Default config search path | Current directory |
| `SYNTH_TEMPLATE_PATH` | Template files location | `./templates` |

## Debugging

### VS Code Launch Configuration

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug CLI",
      "cargo": {
        "args": ["build", "--bin=datasynth-data", "--package=datasynth-cli"]
      },
      "args": ["generate", "--demo", "--output", "./output"],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

### Logging

Enable debug logging:

```bash
RUST_LOG=debug cargo run --release -- generate --demo --output ./output
```

Module-specific logging:

```bash
RUST_LOG=synth_generators=debug,synth_core=info cargo run ...
```

## Common Issues

### Build Failures

```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
```

### Test Failures

```bash
# Run tests with backtrace
RUST_BACKTRACE=1 cargo test

# Run single test with output
cargo test test_name -- --nocapture
```

### Memory Issues

For large generation volumes, increase system limits:

```bash
# Linux: Increase open file limit
ulimit -n 65536

# Check memory usage during generation
/usr/bin/time -v datasynth-data generate --config config.yaml --output ./output
```

## Next Steps

- Review [Code Style](code-style.md) guidelines
- Read [Testing](testing.md) practices
- Learn the [Pull Request](pull-requests.md) process
