# System Monitor - Design Spec
Date: 2026-03-30

## Overview

A lightweight Windows desktop system monitor built with Rust and Tauri. The app displays as an always-on-top, borderless floating window anchored to the bottom-right corner of the screen, showing real-time system metrics with a native HTML/CSS/JS UI.

## Tech Stack

| Component | Technology | Notes |
|-----------|------------|-------|
| Desktop shell / windowing | `tauri` | Window creation, positioning, always-on-top, app lifecycle |
| System metrics | `sysinfo` | CPU, memory, network, disk counters |
| Config persistence | `serde` + `serde_json` | JSON config stored in AppData |
| Frontend UI | native `HTML` + `CSS` + `JavaScript` | No Vue, React, or other frontend framework |

## Project Structure

```text
src-tauri/
|- Cargo.toml          # Rust dependencies for Tauri backend
|- tauri.conf.json     # Tauri window/app configuration
`- src/
   |- main.rs          # Tauri bootstrap, commands, window setup, event emission
   |- metrics.rs       # Metric collection via sysinfo
   |- config.rs        # Config struct, load/save to config.json
   `- state.rs         # Shared app state: config, collector, latest snapshot

src/
|- index.html          # Floating monitor UI
|- styles.css          # Window and settings styles
`- main.js             # DOM updates, Tauri invoke/listen, settings interactions
```

## Architecture

The application is split into two layers with a narrow bridge:

- Rust backend: owns system metric collection, config persistence, refresh scheduling, and desktop-window behavior that must use Tauri
- Frontend: renders the floating monitor and settings overlay using native DOM, CSS, and small amounts of JavaScript

The frontend must not contain business logic for metric collection, rate calculation, or config validation. Those rules live in Rust and are exposed through Tauri commands/events.

## Configuration

Persisted to `%APPDATA%\rmo\config.json`. Directory and file are created automatically on first run. Saved on any change.

```json
{
  "version": 1,
  "refresh_interval_secs": 2.0,
  "opacity": 0.85,
  "show_cpu": true,
  "show_memory": true,
  "show_network": false,
  "show_disk_io": false
}
```

- `refresh_interval_secs`: Rust type `f64`, stored as JSON float
- `opacity`: `f64`, valid range `[0.3, 1.0]`

After a successful load, values are clamped: `opacity` to `[0.3, 1.0]`, `refresh_interval_secs` to `[1.0, 10.0]`.

Schema evolution policy:

- Unknown fields are ignored on load
- Missing or malformed config falls back to defaults
- Default config is immediately rewritten to disk after fallback
- `version` is reserved for future migration work

Minimum metric constraint:

- At least one of `show_cpu`, `show_memory`, `show_network`, `show_disk_io` must remain enabled
- The settings UI disables the last active metric toggle

## Window Behavior

The main window is created and managed by Tauri.

- Borderless
- Always on top
- Non-resizable
- Positioned at bottom-right on every launch with a 12px margin
- Draggable in-session
- Position is not persisted; each app start resets to bottom-right

Required Tauri-owned behavior:

- Create and configure the main window
- Reposition on startup
- Exit the application
- Expose window drag capability to the frontend

Right-click menu behavior:

- Use a custom HTML/CSS/JS context menu instead of a native OS menu
- Menu entries: `Settings`, `Exit`
- This keeps the visual style consistent and avoids introducing more Tauri-native menu plumbing than needed

Transparency behavior:

- Do not rely on whole-window opacity APIs for the primary monitor effect
- Apply opacity to the rendered container background with CSS `rgba(...)`
- Text and metric values should remain visually crisp

## Metrics

### Default

| Label | Source | Color coding |
|-------|--------|--------------|
| CPU | `system.global_cpu_usage()` | threshold-based |
| MEM | `system.used_memory() / system.total_memory()` | threshold-based |

CPU may show `0%` on the first sample, which is acceptable.

### Optional

| Label | Description | Color |
|-------|-------------|-------|
| NET UP | Upload speed | white |
| NET DOWN | Download speed | white |
| DISK R | Disk read speed | white |
| DISK W | Disk write speed | white |

Network and disk metrics are derived from cumulative counters:

```text
speed = (current_bytes - previous_bytes) / elapsed_secs
```

On the first sample, all rate-based metrics display `0 KB/s`.

Speed unit formatting:

- Show `KB/s` below 1 MB/s
- Show `MB/s` at or above 1 MB/s

## Data Flow

Use a mixed command/event model:

1. App startup:
   - frontend loads
   - frontend calls `get_config`
   - frontend calls `get_metrics_snapshot`
2. Runtime refresh:
   - Rust refresh loop collects metrics using the configured interval
   - Rust emits a `metrics://updated` event with the latest snapshot
   - frontend listens and updates the DOM
3. Settings changes:
   - frontend calls `update_config`
   - Rust validates, clamps, saves, updates shared state
   - subsequent refreshes use the new config automatically

Recommended Tauri commands:

- `get_config`
- `update_config`
- `get_metrics_snapshot`
- `exit_app`

Optional command:

- `reposition_window` if startup-only positioning later proves insufficient

## UI Layout

No frontend framework and no canvas-based rendering are needed. The UI is a small text-first floating card with rows for enabled metrics.

Approximate layout:

```text
+----------------------+
| CPU        34%       |
| MEM        62%       |
| NET UP   1.2 MB/s    |
| NET DOWN 0.3 MB/s    |
| DISK R    50 MB/s    |
| DISK W    12 MB/s    |
+----------------------+
```

Guidelines:

- Compact width around 160px
- Height grows with enabled metrics
- CPU/MEM values use threshold colors
- Network/disk values use neutral text color
- Keep the visual language simple and tool-like

## Settings Panel

The settings panel is rendered in the frontend as a lightweight modal/overlay.

Controls:

- Refresh interval slider: `1.0` to `10.0`, step `0.5`
- Opacity slider: `0.3` to `1.0`, step `0.05`
- Checkboxes:
  - Show CPU
  - Show Memory
  - Show Network
  - Show Disk IO

Rules:

- Changes apply immediately after `update_config`
- Config is saved by Rust, not by frontend local storage
- The last enabled metric toggle is disabled
- Right-click context menu is suppressed while settings is open

## Error Handling

- Config read/parse failures fall back to defaults and are logged in Rust
- Metric collection failure should degrade to zeros rather than crash the UI
- Frontend command failures should show a minimal inline error state or silently retry on next refresh
- If optional metrics are unavailable on a given machine, display zero values rather than removing the row unexpectedly

## Refresh Loop

The refresh scheduler lives in Rust, not in the frontend.

- Default interval: `2.0s`
- Rust collects only the enabled optional metrics where possible
- After each collection, Rust emits an updated snapshot event
- Frontend does not own the authoritative timer

This keeps refresh timing, rate calculation, and config application in one place.

## Color Thresholds

```text
value < 60%          -> green  (#4ade80)
60% <= value < 85%   -> yellow (#facc15)
value >= 85%         -> red    (#f87171)
```

Applied to CPU and MEM only.

## Non-Goals

- No Vue, React, Svelte, or other frontend framework
- No local database
- No historical charting
- No tray-first workflow in v1 unless needed for exit/discoverability
- No persisted custom window position in v1
