# yoinkctl 🎨

**yoinkctl** is a fast, minimal, global-hotkey color picker for Linux written in **Rust** using **egui**.
Press a hotkey, click anywhere on your screen, and the color under your cursor is instantly copied to your clipboard.

Designed to be:

* 🚀 Fast & lightweight
* 🎯 Precise (pixel-level picking with magnifier)
* 🧠 Always available (background hotkey daemon)
* 🖥️ Multi-monitor aware

---

## ✨ Features

* **Global hotkey color picking**
* **Background daemon** (runs silently)
* **Fullscreen transparent picker**
* **Smooth magnifier with crosshair**
* **Clipboard copy (HEX)**
* **Display formats**

  * HEX
  * RGB
  * HSL
* **Configurable**

  * Hotkey
  * Preview size
  * Visible color formats
* **GUI config app**
* **Debounced hotkey spawning**
* **Single-instance picker lock**

---

## 📸 How It Works

1. Start the **daemon**
2. Press your configured hotkey
3. Screen freezes with a magnifier
4. Click anywhere → color copied
5. Picker exits instantly

---

## 🧠 Architecture Overview

```text
yoinkctl
├── Config GUI        (default launch)
├── Hotkey Daemon     (yoinkctl daemon)
└── Picker Overlay    (yoinkctl pick)
```

### Modes

| Command           | Purpose                      |
| ----------------- | ---------------------------- |
| `yoinkctl`        | Open settings GUI            |
| `yoinkctl daemon` | Run background hotkey daemon |
| `yoinkctl pick`   | Launch picker overlay        |

---

## 🛠️ Installation

### Requirements

* Linux (tested)
* Rust 1.70+
* X11 / Wayland compatible compositor

### Build from source

```bash
git clone https://github.com/yourusername/yoinkctl.git
cd yoinkctl
cargo build --release
```

Binary will be located at:

```text
target/release/yoinkctl
```

(Optional)

```bash
sudo cp target/release/yoinkctl /usr/local/bin/
```

---

## 🚀 Usage

### 1️⃣ Launch Config GUI

```bash
yoinkctl
```

From here you can:

* Start / stop the daemon
* Change hotkey
* Toggle color formats
* Adjust magnifier size

---

### 2️⃣ Start the Daemon

```bash
yoinkctl daemon
```

This registers the global hotkey and runs in the background.

---

### 3️⃣ Pick a Color

Press your configured hotkey (default):

```
Super + Shift + A
```

Click anywhere → color is copied to clipboard.

---

## ⌨️ Default Hotkey

```
Super + Shift + A
```

Supports:

* Super
* Shift
* Ctrl
* Alt
* A–Z keys

> ⚠️ Restart the daemon after changing hotkeys.

---

## ⚙️ Configuration

Config file location:

```text
~/.config/yoinkctl/config.json
```

Example:

```json
{
  "hotkey": "Super+Shift+A",
  "show_hex": true,
  "show_rgb": true,
  "show_hsl": true,
  "preview_size": 120
}
```

---

## 🧪 Manual Picker Launch (Debug)

```bash
yoinkctl pick
```

Useful for testing without the daemon.

---

## 🧩 Tech Stack

* **Rust**
* **egui / eframe** — UI
* **xcap** — Screen capture
* **arboard** — Clipboard
* **global-hotkey** — System hotkeys
* **serde / serde_json** — Config

---

## 🐧 Platform Support

| OS      | Status            |
| ------- | ----------------- |
| Linux   | ✅ Fully supported |
| macOS   | ⚠️ Untested       |
| Windows | ⚠️ Untested       |

> Daemon management (`pgrep`, `pkill`, `nohup`) is Linux-specific.

---

## 🔒 Single Instance Guarantee

* Picker uses a file lock in `/tmp`
* Prevents double spawning
* Cleans up safely on exit

---

## 📄 License

MIT License
Feel free to fork, modify, and distribute.

---

## 🤝 Contributing

Pull requests welcome.

Ideas:

* Windows support
* Wayland-specific optimizations
* Palette history
* Auto-copy RGB/HSL
* Tray icon

---

## 🧠 Inspiration

Built for developers, designers, and anyone tired of opening heavy apps just to copy a color.

> *Click. Yoink. Done.*
