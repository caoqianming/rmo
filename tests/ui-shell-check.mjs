import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const root = "D:/study/rmo";

const capability = JSON.parse(
  fs.readFileSync(path.join(root, "src-tauri", "capabilities", "default.json"), "utf8")
);
assert.ok(
  capability.permissions.includes("core:window:allow-start-dragging"),
  "expected dragging permission to stay enabled"
);

const html = fs.readFileSync(path.join(root, "src", "index.html"), "utf8");
assert.match(html, /id="control-panel"/, "expected inline control panel markup");
assert.doesNotMatch(html, /id="context-menu"/, "expected temporary exit-only menu to be removed");
assert.match(html, />内存</, "expected memory label to be localized");
assert.match(html, />上传</, "expected upload label to be localized");
assert.match(html, />下载</, "expected download label to be localized");
assert.match(html, />磁盘读</, "expected disk read label to be localized");
assert.match(html, />磁盘写</, "expected disk write label to be localized");
assert.match(html, />刷新间隔</, "expected refresh interval label to be localized");
assert.match(html, />透明度</, "expected opacity label to be localized");
assert.match(html, />显示项</, "expected visible metrics legend to be localized");
assert.match(html, />完成</, "expected done button to be present again");
assert.match(html, />退出</, "expected exit button to be localized");

const js = fs.readFileSync(path.join(root, "src", "main.js"), "utf8");
assert.match(js, /toggleControlPanel/, "expected direct control panel toggle logic");
assert.doesNotMatch(js, /toggleContextMenu/, "expected temporary exit-only menu logic to be removed");
assert.match(js, /invoke\("resize_window_to_content"/, "expected frontend to resize through a Rust command");
assert.match(js, /invoke\("show_main_window"\)/, "expected frontend to reveal the window after initial sizing");
assert.match(js, /invoke\("set_auto_positioning", \{ enabled: false \}\)/, "expected dragging to disable auto repositioning");
assert.match(js, /toggleControlPanel\(\)/, "expected right click to toggle the control panel open and closed");

const css = fs.readFileSync(path.join(root, "src", "styles.css"), "utf8");
assert.match(css, /\.control-panel/, "expected inline control panel styling");
assert.doesNotMatch(css, /\.context-menu/, "expected temporary exit-only menu styling to be removed");
assert.match(css, /min-width: 132px/, "expected a narrower monitor width");
assert.match(css, /width:\s*100%/, "expected the control panel width to stay aligned with the monitor width");
assert.doesNotMatch(css, /min-height:\s*100%/, "expected page height to shrink to content");
assert.match(
  css,
  /--panel-border:\s*rgba\(17,\s*24,\s*39,\s*var\(--panel-alpha\)\)/,
  "expected panel border color to match the panel background"
);
assert.match(css, /padding:\s*6px 7px;/, "expected tighter monitor padding");
assert.match(css, /box-shadow:\s*none;/, "expected monitor shadow to be removed");

const rust = fs.readFileSync(path.join(root, "src-tauri", "src", "main.rs"), "utf8");
assert.match(rust, /fn resize_window_to_content/, "expected Rust resize command");
assert.match(rust, /fn show_main_window/, "expected a Rust command that reveals the window after initial sizing");
assert.match(rust, /fn set_auto_positioning/, "expected Rust command to control auto positioning");
assert.match(rust, /set_background_color/, "expected window setup to force a transparent background");
assert.match(rust, /TrayIconBuilder/, "expected the Tauri shell to build a tray icon");
assert.doesNotMatch(rust, /show_window\(app\.handle\(\)\)\?;/, "expected setup to keep the window hidden until frontend sizing finishes");
assert.match(rust, /tray_exit/, "expected a tray menu item for exiting the app");
assert.doesNotMatch(rust, /tray_show/, "expected tray menu to only keep exit");
assert.match(rust, /show_menu_on_left_click\(false\)/, "expected left click to avoid opening the tray menu");

const tauriConfig = fs.readFileSync(path.join(root, "src-tauri", "tauri.conf.json"), "utf8");
assert.match(tauriConfig, /"width": 140/, "expected a narrower initial window width");
assert.match(tauriConfig, /"skipTaskbar": true/, "expected the window to stay out of the taskbar");

console.log("ui shell checks passed");
