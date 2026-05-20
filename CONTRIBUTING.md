# Contributing to wzllama

Thanks for your interest in contributing to wzllama! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful and constructive. We're building a community around local AI tools.

## How to Contribute

### Reporting Bugs

1. Check existing issues to avoid duplicates
2. Use the bug report template
3. Include:
   - Your OS and version
   - wzllama version (`wzllama --version`)
   - Steps to reproduce
   - Expected vs actual behavior

### Suggesting Features

1. Check existing feature requests
2. Use the feature request template
3. Describe the problem and proposed solution
4. Explain the use case

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Commit with conventional commits
6. Push and create a PR

## Development Setup

### Prerequisites

- Rust 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Ollama installed (optional, for local model testing)

### Building

```bash
# Clone and build
git clone https://github.com/olivierlevy/wzllama.git
cd wzllama
cargo build --release

# Run
./target/release/wzllama
```

### Running Tests

```bash
cargo test
cargo clippy  # Check for warnings
```

## Project Structure

```
wzllama/
├── src/
│   ├── core/          # Core logic (hardware, models, API)
│   ├── wizard/        # Interactive wizards
│   └── tools/         # Tool integrations
├── config/            # i18n translations
└── tests/             # Integration tests
```

## Commit Convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `refactor:` Code refactoring
- `test:` Test additions/changes
- `chore:` Maintenance tasks

## Coding Standards

- Run `cargo fmt` before committing
- Fix clippy warnings
- Add comments for complex logic
- Keep functions focused and small

## License

By contributing, you agree that your contributions will be licensed under the MIT License.