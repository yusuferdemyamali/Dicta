import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

type Settings = {
  startWithWindows: boolean;
};

type AppStatus = {
  state: string;
  lastError: string | null;
};

const startWithWindowsEl = document.getElementById("startWithWindows") as HTMLInputElement;
const saveBtn = document.getElementById("saveBtn") as HTMLButtonElement;
const closeBtn = document.getElementById("closeBtn") as HTMLButtonElement;
const appStatusEl = document.getElementById("appStatus") as HTMLDivElement;
const lastErrorEl = document.getElementById("lastError") as HTMLDivElement;
const autostartStatusEl = document.getElementById("autostartStatus") as HTMLDivElement;
const hotkeyStatusEl = document.getElementById("hotkeyStatus") as HTMLDivElement;

async function loadSettings(): Promise<void> {
  try {
    const settings = await invoke<Settings>("get_settings");
    startWithWindowsEl.checked = settings.startWithWindows;
    autostartStatusEl.textContent = "";
  } catch (e) {
    autostartStatusEl.textContent = `Failed to load settings: ${String(e)}`;
  }
}

async function loadStatus(): Promise<void> {
  try {
    const status = await invoke<AppStatus>("get_app_status");
    appStatusEl.textContent = status.state;
    lastErrorEl.textContent = status.lastError ?? "";
    // hotkey status is conveyed via lastError when registration failed
    if (status.lastError && status.lastError.toLowerCase().includes("hotkey")) {
      hotkeyStatusEl.textContent = status.lastError;
    } else {
      hotkeyStatusEl.textContent = "";
    }
  } catch (e) {
    appStatusEl.textContent = "Unknown";
    lastErrorEl.textContent = String(e);
  }
}

saveBtn.addEventListener("click", async () => {
  saveBtn.disabled = true;
  autostartStatusEl.textContent = "Saving...";
  try {
    await invoke("save_settings", {
      settings: { startWithWindows: startWithWindowsEl.checked },
    });
    autostartStatusEl.textContent = "Saved.";
    await loadStatus();
  } catch (e) {
    autostartStatusEl.textContent = `Save failed: ${String(e)}`;
  } finally {
    saveBtn.disabled = false;
    setTimeout(() => {
      if (autostartStatusEl.textContent === "Saved.") autostartStatusEl.textContent = "";
    }, 2000);
  }
});

closeBtn.addEventListener("click", async () => {
  await getCurrentWindow().hide();
});

window.addEventListener("DOMContentLoaded", async () => {
  await loadSettings();
  await loadStatus();
  // poll status every 1.5s for minimal feedback without excessive polling
  setInterval(loadStatus, 1500);
});
