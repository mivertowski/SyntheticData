# Contributing to SyntheticData

Thank you for your interest in contributing to SyntheticData! This document provides guidelines and information for contributors.

## Quick Links

- [Development Setup](docs/src/contributing/development-setup.md)
- [Code Style Guide](docs/src/contributing/code-style.md)
- [Testing Guidelines](docs/src/contributing/testing.md)
- [Pull Request Guide](docs/src/contributing/pull-requests.md)

## Getting Started

### Prerequisites

- Rust 1.88 or later (install via [rustup](https://rustup.rs/))
- Git
- Cargo (included with Rust)

### Setup

```bash
# Clone the repository
git clone <repository-url>
cd SyntheticData

# Build the project
cargo build

# Run tests
cargo test

# Run the CLI
cargo run --release -- generate --demo --output ./output
```

## Ways to Contribute

### Code Contributions

- **Bug fixes**: Fix issues from the GitHub issue tracker
- **New features**: Implement new generators, output formats, or analysis tools
- **Performance**: Optimize generation speed or memory usage
- **Documentation**: Improve or expand documentation

### Non-Code Contributions

- **Bug reports**: Report issues with detailed reproduction steps
- **Feature requests**: Suggest new features or improvements
- **Documentation feedback**: Point out unclear or missing documentation
- **Testing**: Test pre-release versions and report issues

## Development Workflow

### 1. Fork and Branch

```bash
# Fork the repository on GitHub, then:
git clone <your-fork-url>
cd SyntheticData
git checkout -b feature/my-feature
```

### 2. Make Changes

Follow the code style guidelines:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test
```

### 3. Submit Pull Request

- Push your branch to your fork
- Open a pull request against the `main` branch
- Fill out the PR template
- Wait for review

## Code Style

- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Use `rust_decimal::Decimal` for all financial amounts
- Follow Rust naming conventions
- Add documentation for public APIs

## Testing

- Write unit tests for new functionality
- Ensure all existing tests pass
- Add integration tests for complex features
- Use deterministic seeds for reproducibility

## Pull Request Guidelines

### PR Checklist

- [ ] Code follows project style guidelines
- [ ] Tests added/updated as needed
- [ ] Documentation updated
- [ ] All CI checks pass
- [ ] Self-review completed

### PR Title Format

```
<type>: <short description>
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `perf`, `chore`

Examples:
- `feat: Add OCEL 2.0 export format`
- `fix: Correct decimal serialization in JSON output`
- `docs: Update configuration reference`

## Project Structure

```
SyntheticData/
├── crates/
│   ├── datasynth-cli/          # CLI binary (datasynth-data)
│   ├── datasynth-core/         # Core models and traits
│   ├── datasynth-config/       # Configuration schema
│   ├── datasynth-generators/   # Data generators
│   ├── datasynth-output/       # Output sinks (CSV, JSON, Parquet)
│   ├── datasynth-graph/        # Graph/ML export (PyTorch Geometric, Neo4j)
│   ├── datasynth-runtime/      # Orchestration layer
│   ├── datasynth-server/       # REST/gRPC/WebSocket server
│   ├── datasynth-ui/           # Tauri/SvelteKit desktop UI
│   ├── datasynth-eval/         # Evaluation framework
│   ├── datasynth-ocpm/         # OCEL 2.0 process mining
│   ├── datasynth-banking/      # KYC/AML banking transactions
│   └── datasynth-test-utils/   # Test utilities and fixtures
├── benches/                    # Criterion benchmarks
├── docs/                       # Documentation (mdBook)
└── scripts/                    # Build and utility scripts
```

## Getting Help

- Open an issue for bugs or feature requests
- Check existing issues before creating new ones
- Review the documentation in `docs/src/`

## Code of Conduct

We are committed to providing a welcoming environment. Please:

- Be respectful and constructive
- Focus on technical merits
- Help newcomers learn
- Report unacceptable behavior

## License

By contributing, you agree that your contributions will be licensed under the project's Apache 2.0 License.

---

For more detailed information, see the [contributing guide](docs/src/contributing/README.md).
