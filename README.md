# yoinkctl 🎨

<div align="center">

**A lightning-fast, pixel-perfect color picker for Linux**

*Press a hotkey anywhere, click a color, done. No windows, no hassle.*

[Features](#-features) • [Installation](#-installation) • [Quick Start](#-quick-start) • [Configuration](#%EF%B8%8F-configuration) • [Contributing](#-contributing)

</div>

---

## 🎯 What is yoinkctl?

**yoinkctl** is a modern color picker built for speed and precision. It runs silently in the background, ready to capture any color on your screen with a single hotkey press. Whether you're a designer matching brand colors, a developer debugging CSS, or just someone who loves pixel-perfect accuracy, yoinkctl gets out of your way and lets you work.

**No windows to manage. No apps to open. Just colors.**

### Why yoinkctl?

- ⚡ **Instant access** — Global hotkey from anywhere
- 🎯 **Pixel-perfect** — Magnified preview with crosshair targeting
- 🪶 **Lightweight** — Minimal resource usage, native performance
- 📊 **Multiple formats** — HEX, RGB, and HSL at a glance
- 📜 **Smart history** — Recent colors saved and searchable
- 🎨 **Beautiful UI** — Clean, modern interface with dark theme
- 🚀 **Zero latency** — Rust-powered performance

---

## ✨ Features

### Core Functionality

- **🔥 Global Hotkey Activation** — Press your custom hotkey combo from any app
- **📸 Fullscreen Picker** — Transparent overlay captures your entire workspace
- **🔍 5x Magnifier** — Smooth, anti-aliased zoom for precise color selection
- **📋 Instant Clipboard Copy** — Colors automatically copied on click
- **🎨 Color History** — Last 50 colors saved with click-to-copy
- **🖥️ Multi-Monitor Support** — Works seamlessly across all displays

### Display Options

Choose which color formats to show:
- **HEX** — `#FF5733` (web standard)
- **RGB** — `RGB(255, 87, 51)` (design apps)
- **HSL** — `HSL(9, 100%, 60%)` (color theory)

### Configuration

- ⌨️ **Customizable Hotkeys** — Set any modifier+key combination
- 🔧 **Adjustable Magnifier** — Preview size from 50px to 200px
- 🚀 **Autostart Daemon** — Launch at system boot
- 💾 **Persistent Settings** — Config saved in `~/.config/yoinkctl`

---

## 📦 Installation

### Quick Install (Recommended)

Download and install in one command:

```bash
curl -fsSL https://raw.githubusercontent.com/gimigkk/yoinkctl/main/install.sh | bash
```

This will:
- ✅ Download the latest release for your architecture
- ✅ Install to `~/.local/bin/yoinkctl`
- ✅ Set up desktop entry for application menu
- ✅ Optionally enable autostart

### Build from Source

**Requirements:**
- Rust 1.70+ ([rustup.rs](https://rustup.rs))
- X11 development libraries

**Ubuntu/Debian:**
```bash
sudo apt install libx11-dev libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

**Fedora:**
```bash
sudo dnf install libX11-devel libxcb-devel
```

**Build & Install:**
```bash
git clone https://github.com/gimigkk/yoinkctl.git
cd yoinkctl
./install.sh
```

### Manual Installation

```bash
# Build
cargo build --release

# Copy binary
cp target/release/yoinkctl ~/.local/bin/

# Make sure ~/.local/bin is in your PATH
export PATH="$HOME/.local/bin:$PATH"
```

---

## 🚀 Quick Start

### 1. Launch the Config App

Search for "yoinkctl" in your application menu, or run:

```bash
yoinkctl
```

You'll see the main control panel with:
- **Daemon status** — Start/stop the background service
- **Quick launch** — Test the picker without hotkey
- **Color history** — View and copy recent colors

### 2. Start the Daemon

Click the **Start** button in the Daemon card, or run:

```bash
yoinkctl daemon
```

The daemon registers your hotkey and runs silently in the background.

### 3. Pick a Color!

Press the default hotkey:

```
Super + Shift + A
```

1. Screen freezes with magnifier overlay
2. Move cursor to desired pixel
3. Click to copy color
4. Press `Esc` to cancel

The color is now in your clipboard! 📋

---

## ⚙️ Configuration

### Changing the Hotkey

1. Open **yoinkctl** settings (⚙️ icon)
2. Select modifiers: Super, Shift, Ctrl, Alt
3. Choose a letter key (A-Z)
4. Click **Save Settings**
5. **Restart the daemon** for changes to take effect

**Example combinations:**
- `Super+Shift+C` — Quick and easy
- `Ctrl+Alt+P` — For non-tiling WM users
- `Super+Shift+X` — One-handed convenience

> ⚠️ At least one modifier is required to prevent conflicts

### Display Preferences

Toggle which color formats appear in the picker:

- ✅ **Show HEX** — Standard web format
- ✅ **Show RGB** — Red, green, blue values
- ✅ **Show HSL** — Hue, saturation, lightness

### Magnifier Size

Adjust the preview zoom window from **50px to 200px**. Larger sizes are easier to target but take up more screen space.

### Autostart

Enable **"Launch daemon at startup"** to have yoinkctl ready when you log in. Works with systemd-based systems.

---

## 📁 Project Structure

```
yoinkctl/
├── src/
│   ├── main.rs          # Entry point & mode routing
│   ├── picker.rs        # Color picker overlay UI
│   ├── gui.rs           # Config app & main window
│   ├── config.rs        # Settings management
│   ├── history.rs       # Color history storage
│   └── autostart.rs     # System integration
├── assets/              # Icons & desktop files
├── install.sh           # Installation script
└── Cargo.toml
```

### Architecture

**yoinkctl** operates in three modes:

| Command           | Purpose                             |
|-------------------|-------------------------------------|
| `yoinkctl`        | Launch settings GUI (default)       |
| `yoinkctl daemon` | Run background hotkey service       |
| `yoinkctl pick`   | Show color picker overlay           |

The daemon monitors for your hotkey and spawns picker instances on demand. Each picker uses a file lock to prevent double-spawning.

---

## 🧪 Technical Details

### Built With

- **[Rust](https://www.rust-lang.org/)** — Systems programming language
- **[egui/eframe](https://github.com/emilk/egui)** — Immediate mode GUI
- **[xcap](https://github.com/nashaofu/xcap)** — Cross-platform screen capture
- **[global-hotkey](https://github.com/tauri-apps/global-hotkey)** — System-wide hotkey registration
- **[arboard](https://github.com/1Password/arboard)** — Clipboard management

### Platform Support

| Platform | Status              | Notes                          |
|----------|---------------------|--------------------------------|
| Linux    | ✅ Fully Supported  | X11 and Wayland                |
| macOS    | ⚠️ Experimental     | Hotkey support may vary        |
| Windows  | ⚠️ Experimental     | Daemon management differs      |

> **Note:** Daemon process management (`pgrep`, `pkill`, `nohup`) is optimized for Linux. MacOS and Windows support is theoretical but untested.

---

## 🐛 Troubleshooting

### Daemon won't start

**Check if already running:**
```bash
pgrep -f "yoinkctl daemon"
```

**View daemon logs:**
```bash
journalctl --user -u yoinkctl -f
```

### Hotkey not working

1. Verify daemon is running: look for "● Running" in the GUI
2. Check for hotkey conflicts with your desktop environment
3. Try a different key combination
4. Restart the daemon after changing settings

### Colors not copying

Ensure `arboard` has clipboard access. On Wayland, you may need `wl-clipboard`:

```bash
sudo apt install wl-clipboard  # Ubuntu/Debian
sudo dnf install wl-clipboard  # Fedora
```

### Picker appears on wrong monitor

This is a known issue with multi-monitor setups using different scales. The picker captures the primary monitor. Future updates will add better multi-monitor targeting.

---

## 🗺️ Roadmap

- [ ] **Wayland native support** — Better compositor integration
- [ ] **Color palette management** — Save and organize color schemes
- [ ] **Export formats** — CSS variables, SCSS, Tailwind configs
- [ ] **System tray icon** — Quick access without opening GUI
- [ ] **Keyboard-only mode** — Navigate picker with arrow keys
- [ ] **Color gradients** — Pick multiple colors for smooth transitions
- [ ] **Contrast checker** — WCAG compliance for accessibility
- [ ] **macOS & Windows builds** — Full cross-platform support

---

## 🤝 Contributing

Contributions are welcome! Whether it's bug reports, feature requests, or code improvements, your input helps make yoinkctl better.

### How to Contribute

1. **Fork the repository**
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Commit your changes** (`git commit -m 'Add amazing feature'`)
4. **Push to the branch** (`git push origin feature/amazing-feature`)
5. **Open a Pull Request**

### Development Setup

```bash
git clone https://github.com/gimigkk/yoinkctl.git
cd yoinkctl
cargo build
cargo run  # Launch config GUI
```

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Run clippy before submitting (`cargo clippy`)
- Add tests for new features when applicable

---

## 📄 License

**MIT License** — Free to use, modify, and distribute.

See [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- **egui community** — For the excellent immediate mode GUI framework
- **Rust ecosystem** — For making systems programming accessible
- **Linux community** — For providing a platform where tools like this can thrive

---

## 💬 Support

- 🐛 **Bug Reports:** [GitHub Issues](https://github.com/gimigkk/yoinkctl/issues)
- 💡 **Feature Requests:** [GitHub Discussions](https://github.com/gimigkk/yoinkctl/discussions)
- ⭐ **Star the repo** if you find it useful!

---

<div align="center">

**Built with ❤️ by developers, for developers**

*Click. Yoink. Done.*

[⬆ Back to Top](#yoinkctl-)

</div>
