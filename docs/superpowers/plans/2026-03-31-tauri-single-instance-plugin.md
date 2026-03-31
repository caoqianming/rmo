# Tauri Single Instance Plugin Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the hand-written Windows mutex guard with Tauri's official single-instance plugin so `rmo` always runs as a single instance and re-focuses the existing window on a second launch.

**Architecture:** Keep single-instance enforcement in the Tauri host bootstrap, but delegate instance detection to `tauri-plugin-single-instance`. Register the plugin before other app setup so the second launch is redirected to the already-running process, then reuse the existing `show_window` helper to reveal and focus the main window.

**Tech Stack:** Rust, Tauri 2, `tauri-plugin-single-instance`, Node.js source checks

---

## File Map

| File | Responsibility |
| --- | --- |
| `src-tauri/Cargo.toml` | Remove the ad-hoc Windows API dependency and add the official Tauri single-instance plugin |
| `src-tauri/src/main.rs` | Register the plugin, remove the mutex code, and focus the existing window on re-launch |
| `tests/ui-shell-check.mjs` | Lock in the plugin-based startup contract instead of the Win32 mutex contract |

### Task 1: Lock In Plugin-Based Expectations

**Files:**
- Modify: `tests/ui-shell-check.mjs`
- Test: `tests/ui-shell-check.mjs`

- [ ] **Step 1: Write the failing test**

```js
assert.match(
  rust,
  /tauri_plugin_single_instance::init/,
  "expected Rust startup to register the Tauri single-instance plugin"
);
assert.match(
  rust,
  /\.plugin\(tauri_plugin_single_instance::init\(/,
  "expected the Tauri builder to install the single-instance plugin"
);
assert.match(
  rust,
  /show_window\(&app\)/,
  "expected a second launch to reveal the existing main window"
);
assert.doesNotMatch(
  rust,
  /CreateMutexW|GetLastError|ERROR_ALREADY_EXISTS|acquire_single_instance_guard/,
  "expected the old Win32 mutex guard to be removed"
);
```

- [ ] **Step 2: Run test to verify it fails**

Run: `node tests/ui-shell-check.mjs`
Expected: `FAIL` because `main.rs` still contains `acquire_single_instance_guard` and does not yet register `tauri_plugin_single_instance::init`.

- [ ] **Step 3: Commit**

```bash
git add tests/ui-shell-check.mjs
git commit -m "test: cover tauri single-instance plugin startup"
```

### Task 2: Replace The Mutex Guard With The Plugin

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/main.rs`
- Test: `tests/ui-shell-check.mjs`

- [ ] **Step 1: Write the minimal dependency change**

```toml
[dependencies]
dirs-next = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sysinfo = "0.33"
tauri = { version = "2", features = ["tray-icon", "image-ico"] }
tauri-plugin-single-instance = "2"
```

- [ ] **Step 2: Replace the startup logic with the plugin**

```rust
fn main() {
    let initial_config = Config::load();
    let mut collector = MetricsCollector::new();
    let initial_snapshot = collector.refresh(
        initial_config.show_network,
        initial_config.show_disk_io,
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            let _ = show_window(app);
        }))
        .manage(AppState::new(
            initial_config,
            collector,
            initial_snapshot,
        ))
```

- [ ] **Step 3: Remove the old mutex code**

```rust
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Position, State, WebviewWindow};
```

Expected edit: remove the `SingleInstanceGuard` structs, `Drop` impl, and both `acquire_single_instance_guard` functions from `src-tauri/src/main.rs`.

- [ ] **Step 4: Run source-level regression test**

Run: `node tests/ui-shell-check.mjs`
Expected: `ui shell checks passed`

- [ ] **Step 5: Run Rust verification**

Run: `cargo check`
Expected: `Finished` for the `src-tauri` crate with no single-instance related errors

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/main.rs tests/ui-shell-check.mjs
git commit -m "fix: use tauri single-instance plugin"
```

### Task 3: Build The Executable And Verify Runtime Behavior

**Files:**
- Modify: none
- Test: `src-tauri/target/debug/rmo.exe`

- [ ] **Step 1: Build the desktop binary**

Run: `cargo build`
Expected: successful debug build for `rmo`

- [ ] **Step 2: Verify the second launch reuses the first instance**

Run:

```powershell
$exe = Resolve-Path 'src-tauri\target\debug\rmo.exe'
Get-Process rmo -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 500
Start-Process -FilePath $exe
Start-Sleep -Seconds 2
Start-Process -FilePath $exe
Start-Sleep -Seconds 2
Get-Process rmo -ErrorAction SilentlyContinue | Measure-Object
```

Expected: only one `rmo` process remains after the second launch, and the existing window is shown/focused instead of spawning a second process.
