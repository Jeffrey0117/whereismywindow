# whereismywindow

Windows system-tray utility that draws a border around the focused window and shows which monitor is active — so you never lose your cursor on a multi-monitor setup.

## Stack
- Rust 2021, single binary (`opt-level="s"`, LTO, stripped in release)
- `windows` 0.61 — raw Win32 APIs (WindowsAndMessaging, Accessibility/WinEvent hooks, Direct2D, DirectWrite, DWM, HiDPI, Shell, Registry)
- `tray-icon` 0.19 — system tray icon + menu
- `global-hotkey` 0.6 — Ctrl+Shift+F reveal hotkey
- `eframe`/egui 0.30 (glow) — settings window (runs as a subprocess)
- `serde` + `toml` — config persistence; `dirs` — config path; `log` + `env_logger`
- `winresource` (build dep) — embeds `assets/app.manifest` (DPI awareness)

## Directory structure
```
src/
  main.rs        ← Entry: message-only window + Win32 message loop, timers, tray/menu dispatch
  app.rs         ← App + FocusState top-level state, monitor-change detection
  config.rs      ← Config struct (toggles, border color/style, flash, poll interval) + defaults
  focus/         ← Foreground-window tracking
    tracker.rs     WinEvent hooks (WM_FOCUS_CHANGED / WM_LOCATION_CHANGED), msg_hwnd
    window_info.rs Foreground window title/exe/rect, DWM extended frame bounds
  monitor/       ← Monitor enumeration + geometry (best_monitor_index, rects)
  overlay/       ← Transparent topmost click-through overlays
    border.rs      Focus border (Solid/Glow) via Direct2D
    flash.rs       Full-screen flash on monitor switch
    indicator.rs   Numbered per-monitor badges (bottom-left, active=blue)
    window.rs      Shared overlay window creation
  tray/          ← icon.rs: tray icon, menu items, label updates
  hotkey/        ← reveal.rs: global hotkey registration + poll
  settings/      ← autostart (registry), persistence (toml), launch (subprocess),
                   ui.rs (egui panel), data.rs
assets/app.manifest   ← embedded manifest
build.rs              ← compiles/embeds manifest on Windows
```

## Key concepts
- **Message loop, no framework**: `main.rs` creates a message-only window (`HWND_MESSAGE`) and runs a classic `GetMessageW` loop. Win32 WinEvent hooks post `WM_FOCUS_CHANGED` / `WM_LOCATION_CHANGED`; timers handle polling, flash-hide, hotkey check, and settings-subprocess polling.
- **Overlays**: transparent, topmost, click-through windows. The border tracks the focused window in real time; rects are clamped to the monitor (`clamp_to_monitor`) so a maximized window's overscan doesn't leak onto the next screen. Overlay HWNDs are skipped during focus detection so the app never tracks itself.
- **Focus detection**: `window_info::get_foreground_window_info()` snapshots title/exe/rect; `get_extended_frame_bounds` (DWM) gives accurate borders. Monitor chosen via `geometry::best_monitor_index`.
- **Settings = subprocess**: `--settings` flag launches an egui window in a separate process (`settings::launch`). Parent saves config first, polls the child, and reloads `config.toml` when it exits with changes applied. Keeps the Win32 loop and the egui event loop isolated.
- **Config**: TOML at `dirs::config_dir()/whereismywindow/config.toml`. Compile-time defaults in `config.rs`; runtime changes via tray menu or settings panel. `auto_start` is synced with the Windows registry (`settings::autostart`), not just the file.
- **Tray menu**: Border ON/OFF, Style Solid/Glow, Flash ON/OFF, Indicator ON/OFF, Settings, Quit. Menu events polled each loop iteration.

## Commands
- Build (dev): `cargo build`
- Build (release): `cargo build --release` → `target\release\whereismywindow.exe`
- Run: `cargo run` (debug build shows a console; release uses `windows_subsystem = "windows"`)
- Open settings directly: `whereismywindow.exe --settings`
- Logging: set `RUST_LOG` (e.g. `RUST_LOG=info`); defaults to `info`

## Coding rules
- Windows-only; all platform calls go through the `windows` crate inside `unsafe` blocks — keep `unsafe` scoped and small.
- Win32 resources (timers, hooks, windows) are explicitly cleaned up on exit; pair every create with a teardown.
- Overlay HWNDs must be excluded from focus tracking to avoid self-selection.
- Config values have defaults in `config.rs`; new fields need `#[serde(default)]` for forward compatibility.
- Use `log::{info,warn,error}`; failures degrade gracefully (warn + continue) rather than panic.
