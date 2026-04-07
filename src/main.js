const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// --- State ---
let isRecording = false;

// --- DOM Elements ---
const recordBtn = document.getElementById("record-btn");
const recordHint = document.getElementById("record-hint");
const statusBadge = document.getElementById("status-badge");
const waveform = document.getElementById("waveform");
const transcriptionOutput = document.getElementById("transcription-output");
const modelList = document.getElementById("model-list");
const tabs = document.querySelectorAll(".tab");

const HINT_IDLE = 'Press <kbd>Ctrl+Shift+Space</kbd> or click to record';
const HINT_RECORDING = "Recording... click or press hotkey to stop";

// --- Recording Toggle ---
async function toggleRecording() {
  try {
    if (isRecording) {
      await invoke("stop_recording");
      setRecordingState(false);
    } else {
      await invoke("start_recording");
      setRecordingState(true);
    }
  } catch (err) {
    console.error("Recording toggle error:", err);
    showError(String(err));
  }
}

function setRecordingState(recording) {
  isRecording = recording;
  recordBtn.classList.toggle("recording", recording);
  statusBadge.classList.toggle("recording", recording);
  statusBadge.classList.toggle("ready", !recording);
  statusBadge.textContent = recording ? "Recording" : "Ready";
  waveform.hidden = !recording;
  recordHint.innerHTML = recording ? HINT_RECORDING : HINT_IDLE;
}

// --- Transcription Events ---
async function setupListeners() {
  await listen("transcription", (event) => {
    const result = event.payload;
    appendTranscription(result.text);
  });

  await listen("download-progress", (event) => {
    const { percent } = event.payload;
    updateDownloadProgress(percent);
  });

  await listen("tray-toggle-recording", () => {
    toggleRecording();
  });
}

function appendTranscription(text) {
  if (!text) return;

  // Remove placeholder
  const placeholder = transcriptionOutput.querySelector(".placeholder");
  if (placeholder) placeholder.remove();

  const p = document.createElement("p");
  p.textContent = text;
  p.style.marginBottom = "8px";
  transcriptionOutput.appendChild(p);
  transcriptionOutput.scrollTop = transcriptionOutput.scrollHeight;
}

function showError(message) {
  statusBadge.textContent = message || "Error";
  statusBadge.className = "status-badge";
  statusBadge.style.color = "var(--danger)";
  statusBadge.style.borderColor = "var(--danger)";
  setTimeout(() => {
    statusBadge.style.color = "";
    statusBadge.style.borderColor = "";
    statusBadge.textContent = isRecording ? "Recording" : "Ready";
    statusBadge.classList.toggle("recording", isRecording);
    statusBadge.classList.toggle("ready", !isRecording);
  }, 3000);
}

// --- Tab Navigation ---
const recordingSection = document.querySelector(".recording-section");
const transcriptionSection = document.querySelector(".transcription-section");

tabs.forEach((tab) => {
  tab.addEventListener("click", () => {
    tabs.forEach((t) => t.classList.remove("active"));
    tab.classList.add("active");

    const tabName = tab.dataset.tab;
    const isHome = tabName === "home";

    // Hide all panels
    document.querySelectorAll(".panel").forEach((p) => (p.hidden = true));

    // Show/hide main sections
    recordingSection.hidden = !isHome;
    transcriptionSection.hidden = !isHome;

    if (!isHome) {
      const panel = document.getElementById(`panel-${tabName}`);
      if (panel) panel.hidden = false;
    }

    // Load data for tabs
    if (tabName === "models") loadModels();
    if (tabName === "settings") loadSettings();
  });
});

// --- Models ---
async function loadModels() {
  try {
    const models = await invoke("get_model_status");
    modelList.innerHTML = "";
    models.forEach((model) => {
      const item = document.createElement("div");
      item.className = "model-item";

      const info = document.createElement("div");
      info.innerHTML = `
        <div class="model-name">${escapeHtml(model.display_name)}</div>
        <div class="model-status">${model.downloaded ? "Downloaded" : "Not downloaded"}</div>
        <div class="progress-bar" id="progress-${escapeHtml(model.name)}" hidden>
          <div class="fill" style="width: 0%"></div>
        </div>
      `;

      const btn = document.createElement("button");
      btn.className = "download-btn" + (model.downloaded ? " downloaded" : "");
      btn.textContent = model.downloaded ? "Ready" : "Download";
      btn.disabled = model.downloaded;
      if (!model.downloaded) {
        btn.addEventListener("click", () => downloadModel(model.name));
      }

      item.appendChild(info);
      item.appendChild(btn);
      modelList.appendChild(item);
    });
  } catch (err) {
    console.error("Failed to load models:", err);
  }
}

let currentDownloadModel = null;

async function downloadModel(modelName) {
  currentDownloadModel = modelName;
  const progressBar = document.getElementById(`progress-${modelName}`);
  if (progressBar) progressBar.hidden = false;

  try {
    await invoke("download_model", { modelName });
    loadModels(); // Refresh list
  } catch (err) {
    console.error("Download failed:", err);
    showError("Download failed");
  }
  currentDownloadModel = null;
}

function updateDownloadProgress(percent) {
  if (!currentDownloadModel) return;
  const bar = document.querySelector(`#progress-${currentDownloadModel} .fill`);
  if (bar) bar.style.width = `${percent}%`;
}

// --- Settings ---
async function loadSettings() {
  try {
    const [devices, models, settings] = await Promise.all([
      invoke("list_audio_devices"),
      invoke("get_model_status"),
      invoke("get_settings"),
    ]);

    // Audio devices
    const deviceSelect = document.getElementById("select-device");
    deviceSelect.innerHTML = '<option value="">Default</option>';
    devices.forEach((d) => {
      const opt = document.createElement("option");
      opt.value = d;
      opt.textContent = d;
      if (settings.device === d) opt.selected = true;
      deviceSelect.appendChild(opt);
    });

    // Whisper models
    const modelSelect = document.getElementById("select-model");
    modelSelect.innerHTML = "";
    models.forEach((m) => {
      if (m.downloaded) {
        const opt = document.createElement("option");
        opt.value = m.name;
        opt.textContent = m.display_name;
        if (settings.model === m.name) opt.selected = true;
        modelSelect.appendChild(opt);
      }
    });
    if (modelSelect.options.length === 0) {
      const opt = document.createElement("option");
      opt.value = "";
      opt.textContent = "No models downloaded";
      opt.disabled = true;
      modelSelect.appendChild(opt);
    }

    document.getElementById("input-hotkey").value = settings.hotkey;
    document.getElementById("toggle-vad").checked = settings.vad_enabled;
    document.getElementById("toggle-overlay").checked = settings.show_overlay;
    document.getElementById("toggle-autopaste").checked = settings.auto_paste;
  } catch (err) {
    console.error("Failed to load settings:", err);
  }
}

// --- Helpers ---
function escapeHtml(str) {
  const div = document.createElement("div");
  div.textContent = str;
  return div.innerHTML;
}

// --- Init ---
recordBtn.addEventListener("click", toggleRecording);
setupListeners();
