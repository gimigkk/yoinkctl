#!/bin/bash

# yoinkctl Universal Installer
# Works with or without pre-built binary

set -e

REPO_URL="https://github.com/gimigkk/yoinkctl"
VERSION="1.0.0"
INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/yoinkctl"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

print_header() {
    echo ""
    echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

# Detect OS and architecture
detect_system() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    
    case $ARCH in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            print_error "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac
    
    print_info "Detected: $OS-$ARCH"
}

# Complete cleanup of old installations
cleanup_old_installations() {
    print_header "🧹 Cleaning Up Old Installations"
    
    # Stop all running instances
    print_info "Stopping all yoinkctl processes..."
    pkill -9 -f "yoinkctl" 2>/dev/null || true
    sleep 1
    
    # Remove from common installation locations
    local locations=(
        "$HOME/.local/bin/yoinkctl"
        "$HOME/bin/yoinkctl"
        "/usr/local/bin/yoinkctl"
        "$HOME/.cargo/bin/yoinkctl"
    )
    
    for loc in "${locations[@]}"; do
        if [ -f "$loc" ]; then
            rm -f "$loc" 2>/dev/null || true
            print_success "Removed $loc"
        fi
    done
    
    # Clear KDE shortcuts
    if [ "$XDG_CURRENT_DESKTOP" = "KDE" ] || [ -n "$KDE_SESSION_VERSION" ]; then
        print_info "Cleaning KDE shortcuts..."
        
        KWRITE=""
        if command -v kwriteconfig6 &> /dev/null; then
            KWRITE="kwriteconfig6"
        elif command -v kwriteconfig5 &> /dev/null; then
            KWRITE="kwriteconfig5"
        fi
        
        if [ -n "$KWRITE" ]; then
            $KWRITE --file kglobalshortcutsrc --group "yoinkctl-pick.desktop" --delete 2>/dev/null || true
            
            KHOTKEYS_RC="$HOME/.config/khotkeysrc"
            if [ -f "$KHOTKEYS_RC" ]; then
                cp "$KHOTKEYS_RC" "$KHOTKEYS_RC.backup-$(date +%s)" 2>/dev/null || true
                sed -i '/yoinkctl/d' "$KHOTKEYS_RC" 2>/dev/null || true
            fi
            
            # Reload KDE shortcuts
            QDBUS=""
            if command -v qdbus6 &> /dev/null; then
                QDBUS="qdbus6"
            elif command -v qdbus &> /dev/null; then
                QDBUS="qdbus"
            fi
            
            if [ -n "$QDBUS" ]; then
                $QDBUS org.kde.kded6 /kded org.kde.kded6.unloadModule khotkeys 2>/dev/null || \
                $QDBUS org.kde.kded5 /kded org.kde.kded5.unloadModule khotkeys 2>/dev/null || true
                sleep 0.3
                $QDBUS org.kde.kded6 /kded org.kde.kded6.loadModule khotkeys 2>/dev/null || \
                $QDBUS org.kde.kded5 /kded org.kde.kded5.loadModule khotkeys 2>/dev/null || true
            fi
            
            print_success "KDE shortcuts cleared"
        fi
    fi
    
    # Remove old desktop files
    rm -f "$HOME/.local/share/applications/yoinkctl"*.desktop 2>/dev/null || true
    
    # Remove old autostart
    rm -f "$HOME/.config/autostart/yoinkctl.desktop" 2>/dev/null || true
    
    # Clear xbindkeys config
    if [ -f "$HOME/.xbindkeysrc" ]; then
        sed -i '/# yoinkctl/,+2d' "$HOME/.xbindkeysrc" 2>/dev/null || true
    fi
    
    print_success "Cleanup complete"
}

# Check and install dependencies
check_dependencies() {
    print_header "📦 Checking Dependencies"
    
    local missing_deps=()
    
    # Check for wmctrl (recommended but not required)
    if ! command -v wmctrl &> /dev/null; then
        print_warning "wmctrl not found (recommended for multi-workspace support)"
        missing_deps+=("wmctrl")
    fi
    
    # Check for qdbus/qdbus6 on KDE
    if [ "$XDG_CURRENT_DESKTOP" = "KDE" ] || [ -n "$KDE_SESSION_VERSION" ]; then
        if ! command -v qdbus6 &> /dev/null && ! command -v qdbus &> /dev/null; then
            print_warning "qdbus not found (needed for KDE Wayland support)"
            missing_deps+=("qdbus" "or" "qdbus6")
        fi
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        echo ""
        print_info "Optional dependencies: ${missing_deps[*]}"
        print_info "Install commands:"
        
        if command -v apt &> /dev/null; then
            echo "  sudo apt install wmctrl qdbus-qt5"
        elif command -v dnf &> /dev/null; then
            echo "  sudo dnf install wmctrl qt5-qttools"
        elif command -v pacman &> /dev/null; then
            echo "  sudo pacman -S wmctrl qt5-tools"
        fi
        
        echo ""
        read -p "Continue without optional dependencies? [Y/n] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Nn]$ ]]; then
            exit 1
        fi
    else
        print_success "All dependencies found"
    fi
}

# Download pre-built binary
download_binary() {
    print_header "⬇️  Downloading yoinkctl"
    
    local binary_url="${REPO_URL}/releases/download/v${VERSION}/yoinkctl-${OS}-${ARCH}"
    local temp_file="/tmp/yoinkctl-download"
    
    print_info "Downloading from: $binary_url"
    
    if command -v curl &> /dev/null; then
        if curl -L -f -o "$temp_file" "$binary_url" 2>/dev/null; then
            print_success "Download complete"
            echo "$temp_file"
            return 0
        fi
    elif command -v wget &> /dev/null; then
        if wget -q -O "$temp_file" "$binary_url" 2>/dev/null; then
            print_success "Download complete"
            echo "$temp_file"
            return 0
        fi
    fi
    
    print_warning "Could not download pre-built binary"
    return 1
}

# Build from source
build_from_source() {
    print_header "🔨 Building from Source"
    
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found!"
        print_info "Install Rust: https://rustup.rs/"
        print_info "Or download a pre-built release from: $REPO_URL/releases"
        exit 1
    fi
    
    local script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
    
    print_info "Building release version..."
    cd "$script_dir"
    
    if cargo build --release 2>&1 | grep -v "warning:" | grep -v "Compiling" | grep -v "Finished"; then
        print_success "Build complete"
        echo "$script_dir/target/release/yoinkctl"
        return 0
    else
        print_error "Build failed"
        return 1
    fi
}

# Install binary
install_binary() {
    local binary_path="$1"
    
    print_header "📥 Installing yoinkctl"
    
    mkdir -p "$INSTALL_DIR"
    mkdir -p "$CONFIG_DIR"
    
    cp "$binary_path" "$INSTALL_DIR/yoinkctl"
    chmod +x "$INSTALL_DIR/yoinkctl"
    
    print_success "Installed to: $INSTALL_DIR/yoinkctl"
    
    # Ensure ~/.local/bin is in PATH
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        print_info "Adding $INSTALL_DIR to PATH..."
        
        local shell_rc=""
        if [ -n "$BASH_VERSION" ]; then
            shell_rc="$HOME/.bashrc"
        elif [ -n "$ZSH_VERSION" ]; then
            shell_rc="$HOME/.zshrc"
        else
            shell_rc="$HOME/.profile"
        fi
        
        if ! grep -q ".local/bin" "$shell_rc" 2>/dev/null; then
            echo "" >> "$shell_rc"
            echo "# Added by yoinkctl installer" >> "$shell_rc"
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$shell_rc"
            print_success "Added to $shell_rc"
        fi
        
        export PATH="$HOME/.local/bin:$PATH"
    fi
}

# Setup autostart
setup_autostart() {
    print_header "🚀 Setting Up Autostart"
    
    local autostart_dir="$HOME/.config/autostart"
    mkdir -p "$autostart_dir"
    
    cat > "$autostart_dir/yoinkctl.desktop" << EOF
[Desktop Entry]
Type=Application
Name=yoinkctl Hotkey Daemon
Comment=Global hotkey daemon for yoinkctl color picker
Exec=$INSTALL_DIR/yoinkctl daemon
Icon=color-picker
Terminal=false
Categories=Utility;
X-GNOME-Autostart-enabled=true
Hidden=false
EOF
    
    print_success "Autostart configured"
}

# Start daemon
start_daemon() {
    print_header "▶️  Starting Daemon"
    
    # Kill any existing daemon
    pkill -f "yoinkctl daemon" 2>/dev/null || true
    sleep 0.5
    
    # Start new daemon
    nohup "$INSTALL_DIR/yoinkctl" daemon > /tmp/yoinkctl-daemon.log 2>&1 &
    local daemon_pid=$!
    
    sleep 1
    
    if kill -0 $daemon_pid 2>/dev/null; then
        print_success "Daemon started (PID: $daemon_pid)"
        return 0
    else
        print_warning "Daemon may have failed to start"
        print_info "Check logs: /tmp/yoinkctl-daemon.log"
        return 1
    fi
}

# Main installation flow
main() {
    print_header "🎨 yoinkctl Installer v${VERSION}"
    
    detect_system
    cleanup_old_installations
    check_dependencies
    
    # Try to download pre-built binary first
    local binary_path=""
    
    if binary_path=$(download_binary); then
        print_success "Using pre-built binary"
    else
        print_info "Falling back to building from source..."
        if binary_path=$(build_from_source); then
            print_success "Built from source"
        else
            print_error "Installation failed"
            exit 1
        fi
    fi
    
    install_binary "$binary_path"
    setup_autostart
    start_daemon
    
    # Final success message
    print_header "✨ Installation Complete!"
    
    echo -e "${GREEN}${BOLD}"
    echo "  🎯 Hotkey: Meta+Shift+A"
    echo "     (Meta = Windows/Super key)"
    echo -e "${NC}"
    echo -e "${BLUE}Commands:${NC}"
    echo "  • yoinkctl          → Open settings GUI"
    echo "  • yoinkctl pick     → Manual color picker"
    echo "  • yoinkctl daemon   → Start daemon"
    echo ""
    echo -e "${YELLOW}⚠️  If the hotkey doesn't work immediately,${NC}"
    echo -e "${YELLOW}   please log out and log back in.${NC}"
    echo ""
    print_info "Try your hotkey now: Meta+Shift+A"
    echo ""
}

main "$@"