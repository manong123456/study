# Contributing to CapeOS

Thank you for your interest in contributing!

## Development Setup

1. Install Rust via [rustup](https://rustup.rs):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Clone the repository:
   ```bash
   git clone https://github.com/smartlife-ai/smartlife-ai.os.git
   cd smartlife-ai.os
   ```

3. Build and test:
   ```bash
   cargo build
   cargo test --all
   ```

## Code Quality Requirements

Before submitting a pull request, ensure:

```bash
# Format code
cargo fmt

# No clippy warnings
cargo clippy -- -D warnings

# All tests pass
cargo test --all

# Documentation builds
cargo doc --no-deps
```

## Coding Standards

- **Formatting**: Use `rustfmt` (run `cargo fmt`).
- **Linting**: All clippy warnings must be resolved.
- **Documentation**: All public APIs must have `///` doc comments.
- **Error handling**: Use `Result<T, E>` with proper error types. No `unwrap()` in production code.
- **Testing**: New features must include unit tests. Target >= 70% coverage.

## Pull Request Process

1. Fork the repository and create a feature branch from `main`.
2. Make your changes with clear, descriptive commits.
3. Ensure all CI checks pass (format, clippy, test, doc).
4. Submit a PR with a description of the changes.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
