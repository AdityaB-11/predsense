#!/usr/bin/env bash
# install.sh – build & install Predator Control
# Run as a normal user; will prompt for sudo only where needed.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "==> Building Predator Control (release)…"
cargo build --release

BINARY="$SCRIPT_DIR/target/release/predator-control"

echo "==> Installing binary to /usr/local/bin/predator-control"
sudo install -Dm755 "$BINARY" /usr/local/bin/predator-control

echo "==> Installing polkit policy"
sudo install -Dm644 \
    policy/com.aditya.predator-control.policy \
    /usr/share/polkit-1/actions/com.aditya.predator-control.policy

echo "==> Installing .desktop entry"
sudo install -Dm644 \
    assets/predator-control.desktop \
    /usr/share/applications/predator-control.desktop

echo ""
echo "✅  Predator Control installed successfully."
echo "   Launch it from your app menu or run: predator-control"
