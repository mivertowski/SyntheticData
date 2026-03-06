# Contributing

Welcome to the DataSynth contributor guide.

## Overview

DataSynth is an open-source project and we welcome contributions from the community. This section covers everything you need to know to contribute effectively.

## Ways to Contribute

### Code Contributions

- **Bug fixes**: Fix issues reported in the GitHub issue tracker
- **New features**: Implement new generators, output formats, or analysis tools
- **Performance improvements**: Optimize generation speed or memory usage
- **Documentation**: Improve or expand the documentation

### Non-Code Contributions

- **Bug reports**: Report issues with detailed reproduction steps
- **Feature requests**: Suggest new features or improvements
- **Documentation feedback**: Point out unclear or missing documentation
- **Testing**: Test pre-release versions and report issues

## Quick Start

```bash
# Fork and clone the repository
git clone https://github.com/YOUR_USERNAME/SyntheticData.git
cd SyntheticData

# Create a feature branch
git checkout -b feature/my-feature

# Make your changes and run tests
cargo test

# Submit a pull request
```

## Contribution Guidelines

### Before You Start

1. **Check existing issues**: Look for related issues or discussions
2. **Open an issue first**: For significant changes, discuss before implementing
3. **Follow code style**: Run `cargo fmt` and `cargo clippy`
4. **Write tests**: All new features need test coverage
5. **Update documentation**: Keep docs in sync with code changes

### Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please:

- Be respectful and constructive in discussions
- Focus on the technical merits of contributions
- Help newcomers learn and contribute
- Report unacceptable behavior to the maintainers

## Getting Help

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Pull Request Reviews**: For feedback on your contributions

## In This Section

| Page | Description |
|------|-------------|
| [Development Setup](development-setup.md) | Set up your development environment |
| [Code Style](code-style.md) | Coding standards and conventions |
| [Testing](testing.md) | Testing guidelines and practices |
| [Pull Requests](pull-requests.md) | PR submission and review process |

## License

By contributing to DataSynth, you agree that your contributions will be licensed under the project's MIT License.
