# Rust Code Standards

This document defines the coding standards for CapeOS. All contributors must follow
these rules before submitting code.

---

## 1. Lint & Formatting

### rustfmt

All code must be formatted with `rustfmt`. CI will reject unformatted code.

```bash
# Format all code
cargo fmt

# Check formatting without modifying files (used in CI)
cargo fmt --check
```

Configuration is in `rustfmt.toml`:

```toml
edition = "2021"
max_width = 100
use_field_init_shorthand = true
```

### clippy

Static analysis with clippy. All warnings are treated as errors.

```bash
# Run clippy with warnings as errors
cargo clippy -- -D warnings
```

Configuration is in `clippy.toml`:

```toml
too-many-arguments-threshold = 8
```

### deny.toml

Dependency license and vulnerability auditing via `cargo deny`.

```bash
# Install cargo-deny
cargo install cargo-deny

# Run audit
cargo deny check
```

Configuration is in `deny.toml`:

- **Licenses**: Only MIT, Apache-2.0, BSD-2/3-Clause, ISC, Zlib, CC0-1.0, MPL-2.0 allowed
- **Vulnerabilities**: Deny known vulnerabilities
- **Bans**: Warn on multiple versions of the same crate; deny wildcard dependencies

---

## 2. Comments & Documentation

### Module-level documentation (`//!`)

Every `.rs` file must start with a `//!` doc comment describing the module purpose:

```rust
//! JWT token issuance and JWKS distribution.
//!
//! Uses ES256 (ECDSA P-256) for signing. Generates a new key pair on startup.
```

### Public API documentation (`///`)

Every `pub` item (function, struct, enum, trait, type alias, constant, module) must
have a `///` doc comment:

```rust
/// Issues and signs JWT tokens; provides JWKS for verification.
pub struct JwtIssuer {
    /// The private key used for signing tokens.
    encoding_key: EncodingKey,
    /// Pre-serialized JWKS JSON for the public key endpoint.
    jwks_json: String,
}
```

### Function documentation

Public functions must document:

1. **What** the function does (first line)
2. **`# Arguments`** section for each parameter
3. **`# Returns`** section describing the return value
4. **`# Errors`** section if the function returns `Result`
5. **`# Examples`** section with runnable code (auto-tested as doctests)

```rust
/// Polls until a service becomes available or retries are exhausted.
///
/// # Arguments
///
/// * `runtime_path` - Base directory for runtime files (e.g., `/var/run/capeos`).
/// * `filename` - Name of the file containing the service URL.
/// * `retries` - Maximum number of attempts before giving up.
///
/// # Returns
///
/// The service URL when it becomes available.
///
/// # Errors
///
/// Returns an error if the service is not reachable after all retries.
///
/// # Examples
///
/// ```no_run
/// use capeos_common::utils::service_discovery::wait_for_service;
///
/// # async fn example() -> anyhow::Result<()> {
/// let addr = wait_for_service("/var/run/capeos", "gateway.url", 10).await?;
/// println!("Gateway at: {}", addr);
/// # Ok(())
/// # }
/// ```
pub async fn wait_for_service(runtime_path: &str, filename: &str, retries: u32) -> Result<String> {
    // ...
}
```

### Inline comments

- Explain **why**, not **what** the code does
- Complex algorithms, non-obvious trade-offs, and workarounds must have inline comments
- Do not add comments that merely restate the code:

```rust
// BAD: restates the code
let port = 80; // set port to 80

// GOOD: explains the reason
// Port 80 is the default HTTP port used by the gateway when no
// GATEWAY_PORT environment variable is set.
let port = 80;
```

### Documentation verification

```bash
# Build documentation (must succeed without warnings)
cargo doc --no-deps

# Run doc-tests
cargo test --doc
```

---

## 3. Testing

### Test coverage targets

| Project Stage | Minimum Coverage |
|---------------|-----------------|
| Early / library | >= 70% |
| Mature / core library | >= 85% |
| Security-critical paths | >= 95% |

Measure coverage with:

```bash
# Install coverage tool
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --lcov --output-path lcov.info

# Generate HTML report
cargo llvm-cov --html
```

### Test layers

All five layers of testing are expected:

#### Unit tests (`#[test]`)

Each module should contain a `#[cfg(test)] mod tests` block testing internal logic:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_serialization() {
        let route = Route {
            path: "/v1/sys".to_string(),
            target: "http://127.0.0.1:3000".to_string(),
        };
        let json = serde_json::to_string(&route).unwrap();
        let parsed: Route = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.path, route.path);
    }

    #[tokio::test]
    async fn test_async_operation() {
        // Use #[tokio::test] for async tests
    }
}
```

#### Integration tests (`tests/` directory)

Cross-crate behavior and public API integration tests go in the workspace `tests/` directory:

```
tests/
  gateway_integration.rs   # Route model cross-crate serialization
  event_integration.rs     # Event system model validation
  error_integration.rs     # Error handling cross-crate behavior
```

#### Doc-tests (`# Examples` blocks)

Code examples in doc comments are automatically tested:

```rust
/// Creates a success response.
///
/// # Examples
///
/// ```
/// use capeos_common::models::ApiResponse;
///
/// let resp = ApiResponse::ok("hello");
/// assert_eq!(resp.success, 200);
/// ```
pub fn ok(data: T) -> Self { ... }
```

#### Property-based tests (proptest / quickcheck)

For complex logic with many edge cases, use property-based testing:

```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_port_range(port in 1024u16..65535) {
            // Port availability check should never panic
            let _ = is_port_available(port);
        }
    }
}
```

#### Benchmark tests (criterion)

For performance-sensitive paths:

```rust
// benches/routing_bench.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_route_lookup(c: &mut Criterion) {
    // ...
}

criterion_group!(benches, bench_route_lookup);
criterion_main!(benches);
```

### Running tests

```bash
# Run all tests (unit + integration + doctests)
cargo test --all

# Run tests for a specific crate
cargo test -p capeos-gateway

# Run a specific test
cargo test test_longest_prefix_match

# Run with output
cargo test -- --nocapture
```

---

## 4. Error Handling

### Rules

1. **Public APIs** must return `Result<T, E>` with a custom error type
2. **Never use `unwrap()`** in production code; use `expect()` with descriptive messages
   or proper `?` error propagation
3. **`unwrap()` is acceptable** in test code only
4. Use `thiserror` for error type definitions
5. Use `anyhow` only for application-level binary entry points, not libraries

### Error type pattern

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("internal: {0}")]
    Internal(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
```

### Error conversion for Axum

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            // ...
        };
        (status, Json(json!({"success": status.as_u16(), "message": message}))).into_response()
    }
}
```

---

## 5. Architecture & Documentation

### README.md required sections

Every repository README must contain:

1. **Project description** - what the project does
2. **Core features** - key capabilities
3. **Getting Started** - prerequisites, installation, first run
4. **Architecture overview** - module diagram or description
5. **Build instructions** - native and cross-compilation
6. **API overview** - key endpoints table
7. **License** - MIT
8. **Contributing** - link to CONTRIBUTING.md

### Architecture documentation

- Small libraries: module description in README is sufficient
- Medium/large projects: `docs/` directory with architecture decision records (ADR)
- Generate module dependency graphs: `cargo modules generate graph`
- Use text diagrams for architecture in Markdown:

```
Browser --> Gateway (port 80) --> Backend Services (127.0.0.1)
```

### CHANGELOG.md

Follow [Keep a Changelog](https://keepachangelog.com/) format with
[Semantic Versioning](https://semver.org/):

```markdown
## [0.2.0] - 2026-04-01

### Added
- New storage management API

### Changed
- Improved JWT validation performance

### Fixed
- Gateway route matching for trailing slashes
```

---

## 6. CI/CD Minimum Standard

### GitHub Actions workflow

Every push and pull request must pass:

```yaml
jobs:
  check:
    steps:
      - cargo fmt --check        # Formatting
      - cargo clippy -- -D warnings  # Lint
      - cargo test --all         # All tests
      - cargo doc --no-deps      # Documentation

  coverage:
    steps:
      - cargo llvm-cov --lcov --output-path lcov.info
      # Upload to codecov.io

  cross-compile:
    strategy:
      matrix:
        target: [aarch64-unknown-linux-gnu, armv7-unknown-linux-gnueabihf]
    steps:
      - cargo build --target ${{ matrix.target }}
```

---

## 7. Dependency Management

| Requirement | Tool / Standard |
|-------------|----------------|
| Security audit | `cargo audit` / `cargo deny` |
| Minimize dependencies | Control feature flags; avoid unnecessary crates |
| MSRV | Declare `rust-version` in `Cargo.toml` |
| License compliance | Only use crates with compatible licenses (MIT/Apache-2.0/BSD) |
| Lock file | Commit `Cargo.lock` for binary projects |

---

## 8. Platform Compatibility

### Supported targets

| Target | Hardware | Status |
|--------|----------|--------|
| `x86_64-unknown-linux-gnu` | Development / servers | Primary |
| `aarch64-unknown-linux-gnu` | Raspberry Pi 4/5 | Primary |
| `armv7-unknown-linux-gnueabihf` | Raspberry Pi 3 | Secondary |

### Cross-compilation

```bash
# Install cross-compiler toolchain
sudo apt install gcc-aarch64-linux-gnu

# Add Rust target
rustup target add aarch64-unknown-linux-gnu

# Build
cargo build --release --target aarch64-unknown-linux-gnu
```

---

## 9. Quick Reference Checklist

Before submitting a pull request, verify:

```bash
# 1. Format
cargo fmt

# 2. Lint (zero warnings)
cargo clippy -- -D warnings

# 3. All tests pass
cargo test --all

# 4. Docs build
cargo doc --no-deps

# 5. No unwrap() in production code
grep -rn "\.unwrap()" crates/ --include="*.rs" | grep -v "#\[cfg(test)\]" | grep -v "mod tests"
```

---

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [The Rust Reference](https://doc.rust-lang.org/reference/)
- [Clippy Lint List](https://rust-lang.github.io/rust-clippy/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Semantic Versioning](https://semver.org/)
