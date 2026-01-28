<div align="center">

# 🪟 Where Is My Window?

**Never lose track of your focused window on multi-monitor setups**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Windows](https://img.shields.io/badge/Windows-0078D6?style=flat-square&logo=windows&logoColor=white)](https://www.microsoft.com/windows)

[Download](https://github.com/Jeffrey0117/whereismywindow/releases) • [Features](#-features) • [Usage](#-usage)

</div>

---

## 😫 The Problem

You have 2-3 monitors. A dozen windows open. You start typing and...

**Wait, which window has focus?**

You Alt-Tab, something pops up on the wrong screen. You click, but the cursor was on a different monitor. Sound familiar?

## ✨ The Solution

**Where Is My Window** highlights the active window with a colored border and shows which monitor is active.

<table>
<tr>
<td align="center">
<strong>🔵 Blue Border</strong><br>
Follows your focused window in real-time
</td>
<td align="center">
<strong>🔢 Monitor Badges</strong><br>
Numbered labels show active monitor
</td>
<td align="center">
<strong>⚡ Zero Impact</strong><br>
~0% CPU, <10MB RAM
</td>
</tr>
</table>

---

## 🎯 Features

- **Smart Border** - Solid or Glow style, click-through
- **Monitor Badges** - Active monitor highlighted in blue
- **Flash on Switch** - Optional screen edge flash when switching monitors
- **Hotkey Reveal** - `Ctrl+Shift+F` shows monitor layout
- **System Tray** - Lives in tray, no window clutter
- **Lightweight** - Pure Rust + Win32 APIs, no Electron bloat

---

## 🚀 Installation

### Download

Grab the `.exe` from [Releases](https://github.com/Jeffrey0117/whereismywindow/releases)

### Build from Source

```bash
git clone https://github.com/Jeffrey0117/whereismywindow.git
cd whereismywindow
cargo build --release
```

Run `target\release\whereismywindow.exe`

---

## 📖 Usage

**The app runs in the system tray.** Right-click the tray icon for options:

| Option | Description |
|--------|-------------|
| **Border: ON/OFF** | Toggle focus border |
| **Style: Solid/Glow** | Switch border style |
| **Flash: ON/OFF** | Flash screen on monitor switch |
| **Indicator: ON/OFF** | Toggle monitor badges |
| **Quit** | Exit app |

### Keyboard Shortcut

- `Ctrl+Shift+F` - Show monitor layout info

---

## 🛠️ How It Works

Built with **Rust + Direct2D + Win32 APIs**:

- **Focus Tracking** - Windows event hooks detect active window changes
- **Overlay Rendering** - Transparent topmost window with color-key transparency
- **Real-time Updates** - Border follows window position/size changes instantly

No framework, no runtime, no bloat. Just pure system integration.

---

## 💡 Why This Exists

Multi-monitor setups are amazing until you lose track of focus. This app solves that with zero learning curve:

✅ Glance at screen → See blue border → Know where you are
✅ No config files, no settings to learn
✅ Just works

---

## ⚙️ Requirements

- Windows 10 or 11
- Multiple monitors (works on single monitor too, but kinda pointless)

---

## 🗺️ Roadmap

- [ ] Customizable border colors
- [ ] Per-app border rules (e.g., red for terminal, green for browser)
- [ ] Auto-start on boot option
- [ ] Portable mode (no installer)

---

## 🤝 Contributing

PRs welcome! Keep it simple and lightweight.

---

## 📄 License

MIT License - Use it, fork it, modify it.

---

<div align="center">

**If this saved you from typing in the wrong window, give it a ⭐!**

Made with 💜 by [Jeffrey0117](https://github.com/Jeffrey0117)

</div>
