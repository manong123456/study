# smartlife-ai.view

CapeOS frontend - built with Leptos (Rust WASM).

## Tech Stack

- **Framework**: Leptos 0.7 (CSR mode)
- **Language**: Rust, compiled to WebAssembly
- **UI**: Custom CSS (responsive design)
- **Build**: trunk

## Pages

| Page | Route | Description |
|------|-------|-------------|
| Login | `/login` | User authentication |
| Dashboard | `/` | System stats overview |
| Files | `/files` | File manager |

## Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install trunk (WASM bundler)
cargo install trunk
```

## Development

```bash
trunk serve --open
```

This starts a dev server at http://localhost:8080 with hot-reload.

## Build for Production

```bash
trunk build --release
```

Output is in the `dist/` directory. Serve it with any static file server
or embed it into the CapeOS gateway.

## Deploy to Raspberry Pi

1. Build the frontend:
   ```bash
   trunk build --release
   ```

2. Copy `dist/` to the Pi:
   ```bash
   scp -r dist/ pi@<pi-ip>:/var/lib/capeos/www/
   ```

3. Configure CapeOS-Gateway to serve static files from `/var/lib/capeos/www/`.

## Project Structure

```
smartlife-ai.view/
  src/
    main.rs           # App entry point and router
    api.rs            # API client utilities
    pages/
      login.rs        # Login page
      dashboard.rs    # Dashboard page
      files.rs        # File manager page
    components/
      layout.rs       # App layout (sidebar + content)
  index.html          # HTML template
  style.css           # Global styles
```

## License

MIT
