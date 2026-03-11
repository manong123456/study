# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-03-11

### Added

- Initial release of CapeOS backend.
- `capeos-gateway`: API gateway with dynamic route registration and reverse proxy.
- `capeos-message-bus`: Event pub/sub with broadcast channels and WebSocket.
- `capeos-user-service`: User authentication with ECDSA JWT (P-256) and Argon2.
- `capeos-main`: System monitoring (sysinfo), file management, health checks.
- `capeos-local-storage`: Disk listing via sysinfo.
- `capeos-app-management`: Docker container management via bollard.
- `capeos-common`: Shared library with models, middleware, service discovery.
- `capeos-cli`: CLI diagnostic tool with health/version/status commands.
- SQLite database via rusqlite + tokio-rusqlite.
- systemd service files for all services.
- Cross-compilation support for Raspberry Pi 4+ (aarch64) and Pi 3 (armv7).
- 62 unit tests across all 8 crates.
