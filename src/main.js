const tauri = window.__TAURI__;
const invoke = tauri?.core?.invoke;
const listen = tauri?.event?.listen;
const currentWindow = tauri?.window?.getCurrentWindow?.();

const state = {
  config: null,
  controlsOpen: false,
  windowShown: false,
};

const elements = {
  monitor: document.getElementById("monitor"),
  controlPanel: document.getElementById("control-panel"),
  settingsError: document.getElementById("settings-error"),
  cpuValue: document.getElementById("cpu-value"),
  memValue: document.getElementById("mem-value"),
  netUpValue: document.getElementById("net-up-value"),
  netDownValue: document.getElementById("net-down-value"),
  diskReadValue: document.getElementById("disk-read-value"),
  diskWriteValue: document.getElementById("disk-write-value"),
  refreshInterval: document.getElementById("refresh-interval"),
  refreshOutput: document.getElementById("refresh-output"),
  opacity: document.getElementById("opacity"),
  opacityOutput: document.getElementById("opacity-output"),
  showCpu: document.getElementById("show-cpu"),
  showMemory: document.getElementById("show-memory"),
  showNetwork: document.getElementById("show-network"),
  showDisk: document.getElementById("show-disk"),
  hideControls: document.getElementById("hide-controls"),
  exitApp: document.getElementById("exit-app"),
};

function formatPercent(value) {
  return `${Math.round(value)}%`;
}

function formatSpeed(value) {
  const kilobytesPerSec = value / 1024;
  if (kilobytesPerSec < 1024) {
    return `${kilobytesPerSec.toFixed(1)} KB/s`;
  }
  return `${(kilobytesPerSec / 1024).toFixed(1)} MB/s`;
}

function thresholdClass(value) {
  if (value < 60) {
    return "";
  }
  if (value < 85) {
    return "yellow";
  }
  return "red";
}

function setMetricVisibility(metric, visible) {
  const row = document.querySelector(`[data-metric="${metric}"]`);
  if (!row) {
    return;
  }
  row.classList.toggle("hidden", !visible);
}

function renderSnapshot(snapshot) {
  elements.cpuValue.textContent = formatPercent(snapshot.cpu_pct);
  elements.memValue.textContent = formatPercent(snapshot.mem_pct);
  elements.netUpValue.textContent = formatSpeed(snapshot.net_up_bps);
  elements.netDownValue.textContent = formatSpeed(snapshot.net_down_bps);
  elements.diskReadValue.textContent = formatSpeed(snapshot.disk_read_bps);
  elements.diskWriteValue.textContent = formatSpeed(snapshot.disk_write_bps);

  elements.cpuValue.className = `metric-value ${thresholdClass(snapshot.cpu_pct)}`.trim();
  elements.memValue.className = `metric-value ${thresholdClass(snapshot.mem_pct)}`.trim();
}

function activeMetricCount(config) {
  return [
    config.show_cpu,
    config.show_memory,
    config.show_network,
    config.show_disk_io,
  ].filter(Boolean).length;
}

function syncConfigUi(config) {
  state.config = config;
  elements.refreshInterval.value = String(config.refresh_interval_secs);
  elements.refreshOutput.textContent = `${config.refresh_interval_secs.toFixed(1)}s`;
  elements.opacity.value = String(config.opacity);
  elements.opacityOutput.textContent = config.opacity.toFixed(2);
  elements.showCpu.checked = config.show_cpu;
  elements.showMemory.checked = config.show_memory;
  elements.showNetwork.checked = config.show_network;
  elements.showDisk.checked = config.show_disk_io;

  document.documentElement.style.setProperty("--panel-alpha", config.opacity.toFixed(2));

  setMetricVisibility("cpu", config.show_cpu);
  setMetricVisibility("memory", config.show_memory);
  setMetricVisibility("network-up", config.show_network);
  setMetricVisibility("network-down", config.show_network);
  setMetricVisibility("disk-read", config.show_disk_io);
  setMetricVisibility("disk-write", config.show_disk_io);

  const count = activeMetricCount(config);
  elements.showCpu.disabled = count === 1 && config.show_cpu;
  elements.showMemory.disabled = count === 1 && config.show_memory;
  elements.showNetwork.disabled = count === 1 && config.show_network;
  elements.showDisk.disabled = count === 1 && config.show_disk_io;
}

function resizeWindowToContent() {
  if (!invoke) {
    return;
  }

  window.requestAnimationFrame(async () => {
    const width = Math.ceil(elements.monitor.scrollWidth);
    const height = Math.ceil(elements.monitor.scrollHeight);

    try {
      await invoke("resize_window_to_content", { width, height });
      if (!state.windowShown) {
        await invoke("show_main_window");
        state.windowShown = true;
      }
    } catch (error) {
      console.error("Failed to resize window:", error);
    }
  });
}

function toggleControlPanel(forceOpen) {
  state.controlsOpen = typeof forceOpen === "boolean" ? forceOpen : !state.controlsOpen;
  elements.controlPanel.classList.toggle("hidden", !state.controlsOpen);
  elements.settingsError.classList.add("hidden");
  resizeWindowToContent();
}

async function pushConfigUpdate(patch) {
  if (!state.config || !invoke) {
    return;
  }

  const nextConfig = {
    ...state.config,
    ...patch,
  };

  try {
    const saved = await invoke("update_config", { config: nextConfig });
    elements.settingsError.classList.add("hidden");
    syncConfigUi(saved);
    resizeWindowToContent();
  } catch (error) {
    elements.settingsError.textContent = String(error);
    elements.settingsError.classList.remove("hidden");
    syncConfigUi(state.config);
    resizeWindowToContent();
  }
}

async function init() {
  if (!invoke || !listen) {
    console.error("Global Tauri API not available.");
    return;
  }

  const config = await invoke("get_config");
  const snapshot = await invoke("get_metrics_snapshot");
  syncConfigUi(config);
  renderSnapshot(snapshot);
  resizeWindowToContent();

  await listen("metrics://updated", (event) => {
    renderSnapshot(event.payload);
    resizeWindowToContent();
  });
}

elements.monitor.addEventListener("contextmenu", (event) => {
  event.preventDefault();
  toggleControlPanel();
});

elements.monitor.addEventListener("mousedown", async (event) => {
  if (state.controlsOpen || event.button !== 0) {
    return;
  }

  const dragRegion = event.target.closest("[data-tauri-drag-region]");
  if (!dragRegion || !currentWindow) {
    return;
  }

  event.preventDefault();

  try {
    await invoke("set_auto_positioning", { enabled: false });
    await currentWindow.startDragging();
  } catch (error) {
    console.error("Failed to start dragging:", error);
  }
});

elements.hideControls.addEventListener("click", () => {
  toggleControlPanel(false);
});

elements.exitApp.addEventListener("click", () => invoke("exit_app"));

elements.refreshInterval.addEventListener("input", () => {
  elements.refreshOutput.textContent = `${Number(elements.refreshInterval.value).toFixed(1)}s`;
});

elements.opacity.addEventListener("input", () => {
  const value = Number(elements.opacity.value);
  elements.opacityOutput.textContent = value.toFixed(2);
  document.documentElement.style.setProperty("--panel-alpha", value.toFixed(2));
});

elements.refreshInterval.addEventListener("change", () => {
  pushConfigUpdate({ refresh_interval_secs: Number(elements.refreshInterval.value) });
});

elements.opacity.addEventListener("change", () => {
  pushConfigUpdate({ opacity: Number(elements.opacity.value) });
});

elements.showCpu.addEventListener("change", () => {
  pushConfigUpdate({ show_cpu: elements.showCpu.checked });
});

elements.showMemory.addEventListener("change", () => {
  pushConfigUpdate({ show_memory: elements.showMemory.checked });
});

elements.showNetwork.addEventListener("change", () => {
  pushConfigUpdate({ show_network: elements.showNetwork.checked });
});

elements.showDisk.addEventListener("change", () => {
  pushConfigUpdate({ show_disk_io: elements.showDisk.checked });
});

window.addEventListener("resize", resizeWindowToContent);

init().catch((error) => {
  console.error("Failed to initialize rmo:", error);
});
