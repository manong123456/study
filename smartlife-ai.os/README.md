# smartlife-ai.os

CapeOS backend - an edge AI platform operating system built with Rust.

## Architecture

CapeOS uses a microservice gateway architecture. All services communicate through
CapeOS-Gateway (reverse proxy) and CapeOS-MessageBus (event pub/sub).

```
Browser --> CapeOS-Gateway (port 80) --> Backend Microservices (127.0.0.1)
```

### Services

| Service | Binary | Description |
|---------|--------|-------------|
| capeos-gateway | `capeos-gateway` | API gateway / reverse proxy |
| capeos-message-bus | `capeos-message-bus` | Event pub/sub message bus |
| capeos-user-service | `capeos-user-service` | User auth, JWT, JWKS |
| capeos-local-storage | `capeos-local-storage` | Disk and storage management |
| capeos-app-management | `capeos-app-management` | Docker app lifecycle |
| capeos (main) | `capeos` | File manager, system monitor, Samba, cloud storage |
| capeos-cli | `capeos-cli` | CLI diagnostic tool |

### Tech Stack

- **Language**: Rust (Edition 2021)
- **Web Framework**: Axum 0.8
- **Async Runtime**: Tokio
- **Database**: SQLite via rusqlite + tokio-rusqlite
- **Auth**: ECDSA JWT (P-256) + JWKS + Argon2
- **System Info**: sysinfo crate
- **Docker**: bollard crate

## Project Structure

```
smartlife-ai.os/
  crates/
    capeos-common/        # Shared library (models, middleware, utils)
    capeos-gateway/       # API gateway
    capeos-message-bus/   # Event message bus
    capeos-user-service/  # User authentication
    capeos-local-storage/ # Disk management
    capeos-app-management/# Docker app management
    capeos-main/          # Core service (files, system, samba)
    capeos-cli/           # CLI tool
  config/                 # Configuration templates
  migrations/             # SQL migrations
  deploy/                 # systemd units and install scripts
```

## Build

### Prerequisites

- Rust 1.75+ (install via https://rustup.rs)
- For Raspberry Pi cross-compilation:
  ```bash
  sudo apt install gcc-aarch64-linux-gnu
  rustup target add aarch64-unknown-linux-gnu
  ```

### Native Build

```bash
cargo build --release
```

### Cross-compile for Raspberry Pi 4 (aarch64)

```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

### Cross-compile for Raspberry Pi 3 (armv7)

```bash
sudo apt install gcc-arm-linux-gnueabihf
rustup target add armv7-unknown-linux-gnueabihf
cargo build --release --target armv7-unknown-linux-gnueabihf
```

## Install on Raspberry Pi

1. Build for the target architecture (or cross-compile)
2. Copy the release binaries to the Pi
3. Run the install script:

```bash
sudo bash deploy/scripts/install.sh
```

4. Start services:

```bash
sudo systemctl start capeos-gateway
sudo systemctl start capeos-message-bus
sudo systemctl start capeos-user-service
sudo systemctl start capeos-local-storage
sudo systemctl start capeos-app-management
sudo systemctl start capeos
```

5. Access CapeOS at `http://<raspberry-pi-ip>:80`

## Development

Run individual services locally:

```bash
# Terminal 1: Gateway
GATEWAY_PORT=8080 cargo run -p capeos-gateway

# Terminal 2: Message Bus
cargo run -p capeos-message-bus

# Terminal 3: User Service
cargo run -p capeos-user-service

# Terminal 4: Main Service
cargo run -p capeos-main
```

## API Overview

| Service | Base Path | Key Endpoints |
|---------|-----------|--------------|
| Gateway | `/v1/gateway` | `POST /routes`, `GET /port` |
| UserService | `/v1/user_service` | `POST /users/login`, `POST /users/register` |
| LocalStorage | `/v1/local_storage` | `GET /disks`, `GET /storage` |
| AppManagement | `/v1/app_management` | `GET /container`, `GET /compose` |
| MessageBus | `/v1/message_bus` | `POST /event_types`, `GET /subscribe` (WS) |
| Main | `/v1/sys` | `GET /utilization`, `GET /hardware` |
| Main | `/v1/file` | `GET /content`, `POST /upload` |
| Main | `/v1/capeos` | `GET /health/services` |

## License

MIT
