#!/usr/bin/env bash
set -euo pipefail

echo "=== CapeOS Installation Script ==="

# Create system directories
mkdir -p /etc/capeos
mkdir -p /var/lib/capeos/db
mkdir -p /var/lib/capeos/files
mkdir -p /var/log/capeos
mkdir -p /var/run/capeos

# Copy binaries
BINS=(capeos-gateway capeos-message-bus capeos-user-service capeos-local-storage capeos-app-management capeos capeos-cli)
for bin in "${BINS[@]}"; do
    if [ -f "./target/release/$bin" ]; then
        cp "./target/release/$bin" "/usr/bin/$bin"
        chmod +x "/usr/bin/$bin"
        echo "Installed $bin"
    fi
done

# Copy config
if [ ! -f /etc/capeos/capeos.toml ]; then
    cp config/capeos.toml.sample /etc/capeos/capeos.toml
fi

# Install systemd services
cp deploy/systemd/*.service /usr/lib/systemd/system/ 2>/dev/null || cp deploy/systemd/*.service /etc/systemd/system/
systemctl daemon-reload

# Enable services
for bin in "${BINS[@]}"; do
    if [ "$bin" != "capeos-cli" ]; then
        systemctl enable "$bin.service"
    fi
done

echo ""
echo "=== Installation complete ==="
echo "Start all services: systemctl start capeos-gateway"
echo "Check status: capeos-cli health"
