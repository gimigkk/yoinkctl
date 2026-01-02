#!/bin/bash

# yoinkctl installer - MAXIMUM user friendliness!

set -e

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "🎨 Installing yoinkctl - The Friendly Color Picker!"
echo ""

# Check for wmctrl (needed for sticky windows)
if ! command -v wmctrl &> /dev/null; then
    echo "⚠️  wmctrl not found - needed for multi-workspace support"
    echo ""
    echo "Install it with:"
    if command -v dnf &> /dev/null; then
        echo "  sudo dnf install wmctrl"
    elif command -v apt &> /dev/null; then
        echo "  sudo apt install wmctrl"
    elif command -v pacman &> /dev/null; then
        echo "  sudo pacman -S wmctrl"
    else
        echo "  (search for wmctrl in your package manager)"
    fi
    echo ""
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Build release version
echo "⚙️  Building yoinkctl..."
cargo build --release 2>&1 | grep -v "warning:" || true

# Stop any running instances before installing
echo "🛑 Stopping any running instances..."
pkill -f "yoinkctl" 2>/dev/null || true
sleep 1

# Install binary to user's local bin
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
cp "$SCRIPT_DIR/target/release/yoinkctl" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/yoinkctl"

echo "✅ Binary installed to $INSTALL_DIR/yoinkctl"

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo ""
    echo "📝 Adding $INSTALL_DIR to PATH..."
    
    # Detect shell and add to appropriate rc file
    if [ -n "$BASH_VERSION" ]; then
        SHELL_RC="$HOME/.bashrc"
    elif [ -n "$ZSH_VERSION" ]; then
        SHELL_RC="$HOME/.zshrc"
    else
        SHELL_RC="$HOME/.profile"
    fi
    
    if ! grep -q ".local/bin" "$SHELL_RC" 2>/dev/null; then
        echo "" >> "$SHELL_RC"
        echo "# Added by yoinkctl installer" >> "$SHELL_RC"
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$SHELL_RC"
        echo "✅ Added to $SHELL_RC"
    fi
    
    # Also export for current session
    export PATH="$HOME/.local/bin:$PATH"
fi

echo ""
echo "🧹 Cleaning up old configurations..."

# Remove old KDE shortcuts to prevent conflicts
if [ "$XDG_CURRENT_DESKTOP" = "KDE" ] || [ -n "$KDE_SESSION_VERSION" ]; then
    echo "   Removing old KDE hotkey configurations..."
    
    # Find kwriteconfig
    KWRITE=""
    if command -v kwriteconfig6 &> /dev/null; then
        KWRITE="kwriteconfig6"
    elif command -v kwriteconfig5 &> /dev/null; then
        KWRITE="kwriteconfig5"
    elif command -v kwriteconfig &> /dev/null; then
        KWRITE="kwriteconfig"
    fi
    
    if [ -n "$KWRITE" ]; then
        # Remove from kglobalshortcutsrc
        $KWRITE --file kglobalshortcutsrc --group "yoinkctl-pick.desktop" --delete 2>/dev/null || true
        
        # Remove all yoinkctl entries from khotkeysrc
        KHOTKEYS_RC="$HOME/.config/khotkeysrc"
        if [ -f "$KHOTKEYS_RC" ]; then
            # Create a backup
            cp "$KHOTKEYS_RC" "$KHOTKEYS_RC.backup-$(date +%s)"
            
            # Remove yoinkctl sections using sed
            sed -i '/\[Data_[0-9]*\]/,/^$/{ /yoinkctl/,/^$/d; }' "$KHOTKEYS_RC" 2>/dev/null || true
            sed -i '/yoinkctl/d' "$KHOTKEYS_RC" 2>/dev/null || true
        fi
        
        echo "   ✓ Removed old KDE shortcuts"
        
        # Reload KDE shortcuts
        QDBUS_CMD=""
        if command -v qdbus6 &> /dev/null; then
            QDBUS_CMD="qdbus6"
        elif command -v qdbus &> /dev/null; then
            QDBUS_CMD="qdbus"
        fi
        
        if [ -n "$QDBUS_CMD" ]; then
            # Unload and reload khotkeys to clear old shortcuts
            $QDBUS_CMD org.kde.kded6 /kded org.kde.kded6.unloadModule khotkeys 2>/dev/null || \
            $QDBUS_CMD org.kde.kded5 /kded org.kde.kded5.unloadModule khotkeys 2>/dev/null || true
            
            sleep 0.5
            
            $QDBUS_CMD org.kde.kded6 /kded org.kde.kded6.loadModule khotkeys 2>/dev/null || \
            $QDBUS_CMD org.kde.kded5 /kded org.kde.kded5.loadModule khotkeys 2>/dev/null || true
            
            echo "   ✓ Reloaded KDE shortcuts service"
        fi
    fi
fi

# Remove old desktop file
rm -f "$HOME/.local/share/applications/yoinkctl-pick.desktop" 2>/dev/null || true

# Remove old xbindkeys config if it exists
if [ -f "$HOME/.xbindkeysrc" ]; then
    sed -i '/# yoinkctl/,+2d' "$HOME/.xbindkeysrc" 2>/dev/null || true
fi

echo "✅ Old configurations cleaned"

echo ""
echo "🚀 Starting hotkey daemon..."

# Kill any existing daemon
pkill -f "yoinkctl daemon" 2>/dev/null || true
sleep 0.5

# Start the daemon in the background
nohup "$INSTALL_DIR/yoinkctl" daemon >/dev/null 2>&1 &
DAEMON_PID=$!

sleep 1

# Check if daemon is running
if kill -0 $DAEMON_PID 2>/dev/null; then
    echo "✅ Daemon started successfully (PID: $DAEMON_PID)"
else
    echo "⚠️  Daemon failed to start, trying again..."
    "$INSTALL_DIR/yoinkctl" daemon &
    sleep 1
fi

# Create autostart entry so daemon starts on login
AUTOSTART_DIR="$HOME/.config/autostart"
mkdir -p "$AUTOSTART_DIR"

cat > "$AUTOSTART_DIR/yoinkctl.desktop" << EOF
[Desktop Entry]
Type=Application
Name=yoinkctl Hotkey Daemon
Comment=Global hotkey daemon for yoinkctl color picker
Exec=$INSTALL_DIR/yoinkctl daemon
Icon=color-picker
Terminal=false
Categories=Utility;
X-GNOME-Autostart-enabled=true
EOF

echo "✅ Autostart configured (will start on next login)"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  ✨ Installation Complete! ✨"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  🎯 Your hotkey is ready: Meta+Shift+A"
echo ""
echo "  Try it RIGHT NOW! Press Meta+Shift+A"
echo "  (Meta is usually the Windows/Super key)"
echo ""
echo "  ⚠️  If you see TWO pickers, please log out"
echo "      and log back in to clear old shortcuts"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Other commands:"
echo "  • yoinkctl          → Open settings GUI"
echo "  • yoinkctl pick     → Launch picker manually"
echo "  • yoinkctl daemon   → Start hotkey daemon"
echo ""
echo "The daemon will auto-start when you log in!"
echo ""