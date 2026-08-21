import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

type Settings = {
  start_with_windows: boolean;
  model_id: string;
  // backward compat camel aliases may appear
  startWithWindows?: boolean;
  modelId?: string;
};

type AppStatus = {
  state: string;
  lastError: string | null;
};

const startWithWindowsEl = document.getElementById("startWithWindows") as HTMLInputElement;
const apiKeyEl = document.getElementById("apiKey") as HTMLInputElement;
const modelIdEl = document.getElementById("modelId") as HTMLInputElement;
const saveBtn = document.getElementById("saveBtn") as HTMLButtonElement;
const closeBtn = document.getElementById("closeBtn") as HTMLButtonElement;
const clearApiKeyBtn = document.getElementById("clearApiKeyBtn") as HTMLButtonElement;
const appStatusEl = document.getElementById("appStatus") as HTMLDivElement;
const lastErrorEl = document.getElementById("lastError") as HTMLDivElement;
const autostartStatusEl = document.getElementById("autostartStatus") as HTMLDivElement;
const hotkeyStatusEl = document.getElementById("hotkeyStatus") as HTMLDivElement;
const apiKeyStatusEl = document.getElementById("apiKeyStatus") as HTMLSpanElement;
const zenStatusEl = document.getElementById("zenStatus") as HTMLDivElement;

async function loadApiKeyStatus(): Promise<void> {
  try {
    const hasKey = await invoke<boolean>("has_api_key");
    // Also try alternative name
    apiKeyStatusEl.textContent = hasKey ? "Saved" : "Not set";
    apiKeyStatusEl.className = hasKey ? "hint saved" : "hint";
    // Never preload secret into input - keep input empty (write-only)
    // Show placeholder hint
    if (hasKey) {
      apiKeyEl.placeholder = "Saved — enter new key to update";
    } else {
      apiKeyEl.placeholder = "Enter OpenCode Zen API key";
    }
  } catch {
    // fallback to other command
    try {
      const hasKey = await invoke<boolean>("get_api_key_status");
      apiKeyStatusEl.textContent = hasKey ? "Saved" : "Not set";
      if (hasKey) apiKeyEl.placeholder = "Saved — enter new key to update";
    } catch {
      apiKeyStatusEl.textContent = "";
    }
  }
}

async function loadSettings(): Promise<void> {
  try {
    const settings = await invoke<Settings>("get_settings");
    const sww = (settings as any).start_with_windows ?? (settings as any).startWithWindows ?? true;
    const mid = (settings as any).model_id ?? (settings as any).modelId ?? "deepseek-v4-flash-free";
    startWithWindowsEl.checked = Boolean(sww);
    modelIdEl.value = String(mid);
    autostartStatusEl.textContent = "";
    zenStatusEl.textContent = "";
    await loadApiKeyStatus();
    // Ensure input is empty - never preload secret
    apiKeyEl.value = "";
  } catch (e) {
    autostartStatusEl.textContent = `Failed to load settings: ${String(e)}`;
  }
}

async function loadStatus(): Promise<void> {
  try {
    const status = await invoke<AppStatus>("get_app_status");
    appStatusEl.textContent = status.state;
    lastErrorEl.textContent = status.lastError ?? "";
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
  zenStatusEl.textContent = "";
  try {
    const modelId = modelIdEl.value.trim() || "deepseek-v4-flash-free";
    await invoke("save_settings", {
      settings: { start_with_windows: startWithWindowsEl.checked, model_id: modelId },
    });
    // Only update API key if user entered non-empty value
    const apiKey = apiKeyEl.value.trim();
    if (apiKey.length > 0) {
      await invoke("save_api_key", { apiKey });
      apiKeyEl.value = "";
    }
    autostartStatusEl.textContent = "Saved.";
    zenStatusEl.textContent = "";
    await loadApiKeyStatus();
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

clearApiKeyBtn.addEventListener("click", async () => {
  clearApiKeyBtn.disabled = true;
  try {
    await invoke("save_api_key", { apiKey: "" });
    apiKeyEl.value = "";
    zenStatusEl.textContent = "API key cleared.";
    await loadApiKeyStatus();
  } catch (e) {
    zenStatusEl.textContent = `Clear failed: ${String(e)}`;
  } finally {
    clearApiKeyBtn.disabled = false;
    setTimeout(() => {
      if (zenStatusEl.textContent === "API key cleared.") zenStatusEl.textContent = "";
    }, 2000);
  }
});

closeBtn.addEventListener("click", async () => {
  await getCurrentWindow().hide();
});

window.addEventListener("DOMContentLoaded", async () => {
  await loadSettings();
  await loadStatus();
  setInterval(loadStatus, 1500);
});
