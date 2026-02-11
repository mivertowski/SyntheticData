# Installation

This guide covers installing SyntheticData from source.

## Prerequisites

### Required

| Requirement | Version | Purpose |
|-------------|---------|---------|
| Rust | 1.88+ | Compilation |
| Git | Any recent | Clone repository |
| C compiler | gcc/clang | Native dependencies |

### Optional

| Requirement | Version | Purpose |
|-------------|---------|---------|
| Node.js | 18+ | Desktop UI |
| npm | 9+ | Desktop UI dependencies |

## Installing Rust

If you don't have Rust installed, use rustup:

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
# Download and run rustup-init.exe from https://rustup.rs

# Verify installation
rustc --version
cargo --version
```

## Building from Source

### Clone the Repository

```bash
git clone https://github.com/ey-asu-rnd/SyntheticData.git
cd SyntheticData
```

### Build Release Binary

```bash
# Build optimized release binary
cargo build --release

# The binary is at target/release/datasynth-data
```

### Verify Installation

```bash
# Check version
./target/release/datasynth-data --version

# View help
./target/release/datasynth-data --help

# Run quick validation
./target/release/datasynth-data info
```

## Adding to PATH

To run `datasynth-data` from anywhere:

### Linux/macOS

```bash
# Option 1: Symlink to local bin
ln -s $(pwd)/target/release/datasynth-data ~/.local/bin/datasynth-data

# Option 2: Copy to system bin (requires sudo)
sudo cp target/release/datasynth-data /usr/local/bin/

# Option 3: Add target/release to PATH in ~/.bashrc or ~/.zshrc
export PATH="$PATH:/path/to/SyntheticData/target/release"
```

### Windows

Add the `target/release` directory to your system PATH environment variable.

## Building the Desktop UI

The desktop UI requires additional setup:

```bash
# Navigate to UI crate
cd crates/datasynth-ui

# Install Node.js dependencies
npm install

# Run in development mode
npm run tauri dev

# Build production release
npm run tauri build
```

### Platform-Specific Dependencies

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
Install WebView2 runtime (usually pre-installed on Windows 10/11).

## Running Tests

Verify your installation by running the test suite:

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p datasynth-core
cargo test -p datasynth-generators

# Run with output
cargo test -- --nocapture
```

## Development Setup

For development work:

```bash
# Check code without building
cargo check

# Format code
cargo fmt

# Run lints
cargo clippy

# Build documentation
cargo doc --workspace --no-deps --open
```

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench generation_throughput
```

## Troubleshooting

### Build Failures

**Missing C compiler:**
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# macOS
xcode-select --install

# Fedora/RHEL
sudo dnf install gcc
```

**Out of memory during build:**
```bash
# Limit parallel jobs
cargo build --release -j 2
```

### Runtime Issues

**Permission denied:**
```bash
chmod +x target/release/datasynth-data
```

**Library not found (Linux):**
```bash
# Check for missing dependencies
ldd target/release/datasynth-data
```

## Next Steps

- Follow the [Quick Start Guide](quick-start.md) to generate your first dataset
- Explore [Demo Mode](demo-mode.md) for a hands-on introduction
- Review the [CLI Reference](../user-guide/cli-reference.md) for all commands
