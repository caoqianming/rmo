# rmo Implementation Plan

> For agentic workers: required sub-skill: use `subagent-driven-development` (recommended) or `executing-plans` to implement this plan task by task.

**Goal:** Build `rmo`, a lightweight Windows system monitor using `Rust + Tauri`, with a borderless always-on-top floating window at the bottom-right of the screen and a native `HTML/CSS/JS` UI.

**Architecture:** Tauri owns the desktop shell and window lifecycle, Rust owns metrics/config/state, and the frontend uses plain HTML/CSS/JS for rendering and interaction. Config lives at `%APPDATA%\rmo\config.json`.

**Tech Stack:** Tauri 2, Rust, sysinfo, serde, serde_json, native HTML/CSS/JS

---

## File Map

| File | Responsibility |
|------|----------------|
| `src-tauri/Cargo.toml` | Rust/Tauri dependencies |
| `src-tauri/tauri.conf.json` | Window/app configuration |
| `src-tauri/src/main.rs` | Tauri bootstrap, commands, event emission, window setup |
| `src-tauri/src/config.rs` | `Config` struct, load/save/clamp |
| `src-tauri/src/metrics.rs` | `MetricsSnapshot`, `MetricsCollector`, rate calculation |
| `src-tauri/src/state.rs` | Shared app state for config, collector, latest snapshot |
| `src/index.html` | Floating monitor markup |
| `src/styles.css` | Floating window, context menu, settings modal styles |
| `src/main.js` | Tauri invoke/listen bridge, DOM rendering, interaction wiring |

---

## Task 1: Scaffold the Tauri App

**Files:**
- Create or update: `src-tauri/Cargo.toml`
- Create or update: `src-tauri/tauri.conf.json`
- Create or update: `src-tauri/src/main.rs`
- Create or update: `src/index.html`
- Create or update: `src/styles.css`
- Create or update: `src/main.js`

- [ ] Initialize or normalize the project as a Tauri app
- [ ] Ensure the frontend is plain HTML/CSS/JS, not Vue or another framework
- [ ] Configure a single main window for Windows desktop use
- [ ] Confirm the app boots and shows a placeholder monitor shell

Expected outcome:

- Tauri launches successfully
- A single floating window opens
- Frontend assets are loaded from native HTML/CSS/JS entry files

---

## Task 2: Implement Config Persistence in Rust

**Files:**
- Create or update: `src-tauri/src/config.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] Add `Config` with defaults:
  - `version`
  - `refresh_interval_secs`
  - `opacity`
  - `show_cpu`
  - `show_memory`
  - `show_network`
  - `show_disk_io`
- [ ] Implement `load()`, `save()`, `clamp()`, and active-metric helper methods
- [ ] Persist config to `%APPDATA%\rmo\config.json`
- [ ] Fall back to defaults on missing or malformed config
- [ ] Keep unknown field handling forward-compatible

Expected outcome:

- Config is created automatically on first run
- Invalid values are clamped
- At least one metric can always remain enabled

---

## Task 3: Implement Metrics Collection in Rust

**Files:**
- Create or update: `src-tauri/src/metrics.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] Add `MetricsSnapshot` for CPU, memory, network, and disk values
- [ ] Add `MetricsCollector` using `sysinfo`
- [ ] Refresh CPU and memory every cycle
- [ ] Calculate network/disk rates from cumulative counters
- [ ] Format threshold rules and speed helper functions in Rust or shared frontend-safe helpers

Expected outcome:

- CPU and memory percentages are available every refresh
- Optional network and disk rates are zero-safe on first sample
- The collector can be reused by the Tauri runtime

---

## Task 4: Add Shared State and Tauri Commands

**Files:**
- Create or update: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] Create shared state for:
  - current config
  - metrics collector
  - latest metrics snapshot
- [ ] Register Tauri commands:
  - `get_config`
  - `update_config`
  - `get_metrics_snapshot`
  - `exit_app`
- [ ] Validate and save config in Rust
- [ ] Return serialized data structures the frontend can consume directly

Expected outcome:

- Frontend can fetch config and a metrics snapshot on demand
- Settings changes flow through Rust and persist correctly

---

## Task 5: Add Rust Refresh Loop and Event Push

**Files:**
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/state.rs`

- [ ] Start a background refresh loop driven by the configured interval
- [ ] Collect metrics using the current config
- [ ] Store the latest snapshot in shared state
- [ ] Emit a Tauri event such as `metrics://updated` to the main window
- [ ] Ensure config updates affect subsequent refresh timing and enabled metric collection

Expected outcome:

- The frontend does not need to own the authoritative polling timer
- Metric updates are pushed from Rust to the window

---

## Task 6: Build the Native Frontend Shell

**Files:**
- Modify: `src/index.html`
- Modify: `src/styles.css`
- Modify: `src/main.js`

- [ ] Create the compact monitor layout in plain HTML
- [ ] Add rows for CPU and memory by default
- [ ] Add optional rows for network and disk
- [ ] Apply threshold colors for CPU/MEM
- [ ] Keep the card compact, tool-like, and readable at small size
- [ ] Avoid framework-style component abstractions

Expected outcome:

- The window renders from static frontend assets only
- DOM updates reflect the latest pushed snapshot

---

## Task 7: Wire Frontend to Tauri

**Files:**
- Modify: `src/main.js`

- [ ] On startup, call `get_config`
- [ ] On startup, call `get_metrics_snapshot`
- [ ] Subscribe to `metrics://updated`
- [ ] Render the latest values into the DOM
- [ ] Show or hide optional rows based on config
- [ ] Reflect opacity changes in the monitor background

Expected outcome:

- Initial paint works before the first pushed refresh
- Later updates arrive through Tauri events

---

## Task 8: Implement Context Menu and Settings Overlay

**Files:**
- Modify: `src/index.html`
- Modify: `src/styles.css`
- Modify: `src/main.js`

- [ ] Add a custom right-click context menu in plain HTML/CSS/JS
- [ ] Include `Settings` and `Exit`
- [ ] Add a lightweight modal/overlay settings panel
- [ ] Add controls for refresh interval, opacity, and metric toggles
- [ ] Disable the last remaining active metric toggle
- [ ] Call `update_config` on changes
- [ ] Call `exit_app` from the context menu

Expected outcome:

- No native OS context menu is required
- Settings are visually consistent with the monitor shell

---

## Task 9: Implement Tauri Window Behavior

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/src/main.rs`
- Optionally modify: `src/main.js`

- [ ] Configure the window to be borderless
- [ ] Configure the window to stay always on top
- [ ] Disable resizing
- [ ] Position the window at the bottom-right on startup
- [ ] Support dragging through Tauri window drag behavior
- [ ] Do not persist manual drag position between launches

Expected outcome:

- The app feels like a small desktop utility rather than a standard browser window

---

## Task 10: Verification and Release Readiness

**Files:**
- Review all touched files

- [ ] Run Rust tests for config and metrics logic
- [ ] Run the Tauri app locally and verify the shell behavior
- [ ] Verify:
  - startup positioning
  - always-on-top
  - drag behavior
  - settings persistence
  - context menu actions
  - optional metric visibility
- [ ] Build a Windows release binary

Expected outcome:

- `rmo` is shippable as a Tauri desktop utility
- The implementation matches the spec and avoids extra frontend frameworks

---

## Verification Checklist

```text
[ ] Config file is created and rewritten safely
[ ] CPU/MEM render correctly
[ ] Network/disk rates do not spike on first sample
[ ] Opacity affects the card background, not text clarity
[ ] Right-click opens custom menu
[ ] Settings updates persist after restart
[ ] Window opens at bottom-right on every launch
[ ] Window remains always on top
[ ] No Vue/React/Svelte introduced
```
