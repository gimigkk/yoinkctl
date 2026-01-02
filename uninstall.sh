#!/usr/bin/env bash
set -e

BIN="yoinkctl"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
CONFIG_DIR="$HOME/.config/yoinkctl"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${YELLOW}ℹ${NC} $1"; }
log_success() { echo -e "${GREEN}✓${NC} $1"; }
log_error() { echo -e "${RED}✗${NC} $1"; }

echo "Uninstalling yoinkctl..."

# Stop and disable systemd service
if command -v systemctl >/dev/null 2>&1; then
    if systemctl --user is-active yoinkctl.service >/dev/null 2>&1; then
        log_info "Stopping daemon..."
        systemctl --user stop yoinkctl.service
    fi
    
    if systemctl --user is-enabled yoinkctl.service >/dev/null 2>&1; then
        log_info "Disabling autostart..."
        systemctl --user disable yoinkctl.service
    fi
    
    SERVICE_FILE="$HOME/.config/systemd/user/yoinkctl.service"
    if [[ -f "$SERVICE_FILE" ]]; then
        rm "$SERVICE_FILE"
        systemctl --user daemon-reload
        log_success "Removed systemd service"
    fi
fi

# Remove binary
if [[ -f "$INSTALL_DIR/$BIN" ]]; then
    rm "$INSTALL_DIR/$BIN"
    log_success "Removed binary from $INSTALL_DIR"
else
    log_info "Binary not found at $INSTALL_DIR/$BIN"
fi

# Ask about config
if [[ -d "$CONFIG_DIR" ]]; then
    read -p "Remove configuration directory? (includes your settings) [y/N] " yn
    if [[ "$yn" =~ ^[Yy]$ ]]; then
        rm -rf "$CONFIG_DIR"
        log_success "Removed config directory"
    else
        log_info "Kept config at $CONFIG_DIR"
    fi
fi

log_success "yoinkctl uninstalled"