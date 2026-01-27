# Where Is My Window?

A tiny Windows app that solves one annoying problem: **on multi-monitor setups, you lose track of which window has focus.**

You're working across 2-3 screens, a dozen windows open, you start typing and... wait, where's my cursor? Which monitor am I even on? You Alt-Tab and something pops up on the wrong screen. Sound familiar?

This app fixes that. It runs in the background and does two things:

- **Blue border** around the focused window -- you always know exactly which one is active
- **Monitor badges** -- small numbered labels at the bottom-left of each screen showing which monitor is active

That's it. No config files, no UI to learn, no Electron bloat. Just a system tray icon that sits there and works.

## Features

- Colored border follows your focused window in real-time
- Border styles: **Solid** or **Glow** (toggle from tray menu)
- Numbered badge on each monitor (active = blue, inactive = gray)
- Optional full-screen flash on monitor switch
- Click-through -- never steals focus, never blocks your mouse
- Hotkey reveal: `Ctrl+Shift+F` shows monitor layout info
- ~0% CPU idle, <10MB RAM

## Install

Grab the `.exe` from [Releases](https://github.com/Jeffrey0117/whereismywindow/releases) or build from source:

```
cargo build --release
```

Run `target\release\whereismywindow.exe`. It goes straight to the system tray.

## Usage

Right-click the tray icon:

| Option | What it does |
|--------|-------------|
| Border: ON/OFF | Toggle the focus border |
| Style: Solid/Glow | Switch border style |
| Flash: ON/OFF | Flash screen edge on monitor switch |
| Indicator: ON/OFF | Toggle monitor badges |
| Quit | Exit |

## Requirements

- Windows 10/11
- That's it

## Why this exists

Every multi-monitor user has been there. You've got Slack on one screen, VS Code on another, browser on a third. You click somewhere, start typing, and the text goes into the wrong window. Or you Alt-Tab expecting a window on your left monitor but it pops up on your right.

This app makes the active window obvious at a glance. The border is subtle enough to not be annoying, but visible enough that you always know where you are.

## Tech

Built in Rust with raw Win32 APIs. No framework, no runtime. Direct2D for rendering, system event hooks for focus tracking. The overlay is a transparent topmost window with color-key transparency -- it's literally invisible except for the border pixels.

## License

MIT
