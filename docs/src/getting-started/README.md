# Getting Started

Welcome to SyntheticData! This section will help you get up and running quickly.

## What You'll Learn

- **[Installation](installation.md)**: Set up SyntheticData on your system
- **[Quick Start](quick-start.md)**: Generate your first synthetic dataset
- **[Demo Mode](demo-mode.md)**: Explore SyntheticData with built-in demo presets

## Prerequisites

Before you begin, ensure you have:

- **Rust 1.88+**: SyntheticData is written in Rust and requires the Rust toolchain
- **Git**: For cloning the repository
- **(Optional) Node.js 18+**: Required only for the desktop UI

## Installation Overview

```bash
# Clone and build
git clone https://github.com/ey-asu-rnd/SyntheticData.git
cd SyntheticData
cargo build --release

# The binary is at target/release/datasynth-data
```

## First Steps

The fastest way to explore SyntheticData is through demo mode:

```bash
datasynth-data generate --demo --output ./demo-output
```

This generates a complete set of synthetic financial data using sensible defaults.

## Architecture at a Glance

SyntheticData generates interconnected financial data:

```
┌─────────────────────────────────────────────────────────────┐
│                    Configuration (YAML)                      │
├─────────────────────────────────────────────────────────────┤
│                    Generation Pipeline                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │  Master  │→│ Document │→│  Journal │→│  Output  │     │
│  │   Data   │  │  Flows   │  │ Entries  │  │  Files   │     │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
├─────────────────────────────────────────────────────────────┤
│  Output: CSV, JSON, Neo4j, PyTorch Geometric, ACDOCA        │
└─────────────────────────────────────────────────────────────┘
```

## Next Steps

1. Follow the [Installation Guide](installation.md) to set up your environment
2. Work through the [Quick Start Tutorial](quick-start.md)
3. Explore [Demo Mode](demo-mode.md) for a hands-on introduction
4. Review the [CLI Reference](../user-guide/cli-reference.md) for all available commands

## Getting Help

- Check the [User Guide](../user-guide/README.md) for detailed usage instructions
- Review [Configuration](../configuration/README.md) for all available options
- See [Use Cases](../use-cases/README.md) for real-world examples
