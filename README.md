# yoinkctl - Blazing fast color picker

A fast, friendly color picker for Linux with global hotkey support.

## Features

- 🎯 **Global Hotkey**: Press `Meta+Shift+A` anywhere to pick colors
- �� **Magnifying Glass**: 11x11 pixel zoom for precise color selection
- ⚡ **Instant Copy**: Colors automatically copied to clipboard
- 🚀 **Built-in Daemon**: No configuration needed
- 💻 **Cross-platform**: Works on X11 and Wayland
- 🎨 **Clean UI**: Transparent overlay with color info

## Installation

### Prerequisites

**Fedora:**
```bash
sudo dnf install rust cargo wmctrl
```

**Arch:**
```bash
sudo pacman -S rust wmctrl
```

**Ubuntu/Debian:**
```bash
sudo apt install cargo wmctrl
```

### Install
```bash
./install.sh
```

The installer will:
- Build the application
- Install to `~/.local/bin`
- Start the hotkey daemon
- Configure autostart on login

## Usage

### Quick Start

Press **Meta+Shift+A** (Windows/Super + Shift + A) to launch the color picker!

### Manual Commands
```bash
# Open settings GUI
yoinkctl

# Launch color picker manually
yoinkctl pick

# Start the hotkey daemon
yoinkctl daemon
```

### Development
```bash
# Run the daemon in development
cargo run -- daemon

# Test the picker
cargo run -- pick

# Open settings
cargo run
```

## How It Works

1. Press `Meta+Shift+A` anywhere
2. Move your cursor over any color
3. Click to pick and copy to clipboard
4. Press `ESC` to cancel

Colors are copied in HEX format: `#RRGGBB`

## Troubleshooting

### Hotkey doesn't work after install
Log out and log back in, or manually start the daemon:
```bash
yoinkctl daemon
```

### Multiple pickers launching
Run the installer again to clean up old configurations:
```bash
./install.sh
```

## Tech Stack

- **Rust** - Fast, safe systems programming
- **egui** - Immediate mode GUI
- **global-hotkey** - Cross-platform hotkey registration
- **xcap** - Screen capture
- **arboard** - Clipboard handling

## License

MIT

## Contributing

Issues and pull requests welcome!
