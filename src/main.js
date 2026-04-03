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
    showError(err);
  }
}

function setRecordingState(recording) {
  isRecording = recording;
  recordBtn.classList.toggle("recording", recording);
  statusBadge.classList.toggle("recording", recording);
  statusBadge.classList.toggle("ready", !recording);
  statusBadge.textContent = recording ? "Recording" : "Ready";
  waveform.hidden = !recording;
  recordHint.textContent = recording
    ? "Recording... click or press hotkey to stop"
    : "Press Ctrl+Shift+Space or click to record";
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
  statusBadge.textContent = "Error";
  statusBadge.style.color = "var(--danger)";
  setTimeout(() => {
    statusBadge.textContent = isRecording ? "Recording" : "Ready";
    statusBadge.style.color = "";
  }, 3000);
}

// --- Tab Navigation ---
tabs.forEach((tab) => {
  tab.addEventListener("click", () => {
    tabs.forEach((t) => t.classList.remove("active"));
    tab.classList.add("active");

    const tabName = tab.dataset.tab;

    // Hide all panels
    document.querySelectorAll(".panel").forEach((p) => (p.hidden = true));

    // Show/hide main sections
    const homeVisible = tabName === "home";
    document.querySelector(".recording-section").hidden = !homeVisible;
    document.querySelector(".transcription-section").hidden = !homeVisible;

    if (tabName !== "home") {
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
      item.innerHTML = `
        <div>
          <div class="model-name">${model.display_name}</div>
          <div class="model-status">${model.downloaded ? "Downloaded" : "Not downloaded"}</div>
          <div class="progress-bar" id="progress-${model.name}" hidden>
            <div class="fill" style="width: 0%"></div>
          </div>
        </div>
        <button class="download-btn ${model.downloaded ? "downloaded" : ""}"
                data-model="${model.name}"
                ${model.downloaded ? "disabled" : ""}>
          ${model.downloaded ? "✓ Ready" : "Download"}
        </button>
      `;
      modelList.appendChild(item);
    });

    // Download button handlers
    modelList.querySelectorAll(".download-btn:not(.downloaded)").forEach((btn) => {
      btn.addEventListener("click", () => downloadModel(btn.dataset.model));
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
    showError(err);
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
    const [devices, settings] = await Promise.all([
      invoke("list_audio_devices"),
      invoke("get_settings"),
    ]);

    const deviceSelect = document.getElementById("select-device");
    deviceSelect.innerHTML = '<option value="">Default</option>';
    devices.forEach((d) => {
      const opt = document.createElement("option");
      opt.value = d;
      opt.textContent = d;
      if (settings.device === d) opt.selected = true;
      deviceSelect.appendChild(opt);
    });

    document.getElementById("input-hotkey").value = settings.hotkey;
    document.getElementById("toggle-vad").checked = settings.vad_enabled;
    document.getElementById("toggle-overlay").checked = settings.show_overlay;
    document.getElementById("toggle-autopaste").checked = settings.auto_paste;
  } catch (err) {
    console.error("Failed to load settings:", err);
  }
}

// --- Init ---
recordBtn.addEventListener("click", toggleRecording);
setupListeners();
