#!/usr/bin/env bash
set -e

BIN="yoinkctl"
INSTALL_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor"
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

# Install desktop file
if [[ -f "assets/yoinkctl.desktop" ]]; then
  log "Installing desktop entry"
  mkdir -p "$DESKTOP_DIR"
  cp assets/yoinkctl.desktop "$DESKTOP_DIR/"
  
  # Update Exec path to use full path
  sed -i "s|Exec=yoinkctl|Exec=$INSTALL_DIR/yoinkctl|g" "$DESKTOP_DIR/yoinkctl.desktop"
  
  ok "Desktop entry installed"
  
  # Update desktop database
  if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
  fi
fi

# Install icon if it exists
for icon_file in assets/yoinkctl.{svg,png}; do
  if [[ -f "$icon_file" ]]; then
    log "Installing icon"
    if [[ "$icon_file" == *.svg ]]; then
      mkdir -p "$ICON_DIR/scalable/apps"
      cp "$icon_file" "$ICON_DIR/scalable/apps/"
    else
      mkdir -p "$ICON_DIR/48x48/apps"
      cp "$icon_file" "$ICON_DIR/48x48/apps/"
    fi
    ok "Icon installed"
    
    # Update icon cache
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
      gtk-update-icon-cache -f -t "$ICON_DIR" 2>/dev/null || true
    fi
    break
  fi
done

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

ok "Done! Search for 'yoinkctl' in your application menu"
ok "Or run: yoinkctl"