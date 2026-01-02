#!/usr/bin/env bash
set -e

BIN="yoinkctl"
INSTALL_DIR="$HOME/.local/bin"
MODE="local"   # local | remote

log() { echo "➜ $1"; }
ok() { echo "✓ $1"; }
warn() { echo "⚠ $1"; }
die() { echo "✗ $1" >&2; exit 1; }

# Args
if [[ "$1" == "--remote" ]]; then
  MODE="remote"
fi

# Arch (only needed for remote)
ARCH=$(uname -m)
case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *) die "Unsupported architecture: $ARCH" ;;
esac

mkdir -p "$INSTALL_DIR"

log "Installing yoinkctl ($MODE mode)"

if [[ "$MODE" == "local" ]]; then
  log "Building release binary"
  cargo build --release

  [[ -f "target/release/$BIN" ]] || die "Build failed"

  cp target/release/$BIN "$INSTALL_DIR/$BIN"
else
  REPO="gimigkk/yoinkctl"
  URL="https://github.com/$REPO/releases/latest/download/$BIN-linux-$ARCH"

  log "Downloading from GitHub"
  curl -fsSL "$URL" -o "$INSTALL_DIR/$BIN" || die "Download failed"
fi

chmod +x "$INSTALL_DIR/$BIN"
ok "Installed to $INSTALL_DIR/$BIN"

# PATH check
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
  warn "$INSTALL_DIR not in PATH"
  echo "Add this to your shell config:"
  echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

# systemd user service
echo ""
if command -v systemctl >/dev/null 2>&1; then
  read -p "Enable yoinkctl daemon autostart? [Y/n] " yn
  if [[ ! "$yn" =~ ^[Nn]$ ]]; then
    mkdir -p ~/.config/systemd/user
    cat > ~/.config/systemd/user/yoinkctl.service <<EOF
[Unit]
Description=yoinkctl daemon

[Service]
ExecStart=$INSTALL_DIR/yoinkctl daemon
Restart=on-failure

[Install]
WantedBy=default.target
EOF
    systemctl --user daemon-reload
    systemctl --user enable --now yoinkctl.service
    ok "Daemon enabled"
  fi
fi

ok "Done! Try: yoinkctl"
