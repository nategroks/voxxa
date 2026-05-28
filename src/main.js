const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// --- State ---
let isListening = false;
let slides = [];
let currentSlideIndex = 0;

// --- DOM ---
const statusBadge = document.getElementById("status-badge");
const setlistLoader = document.getElementById("setlist-loader");
const slideView = document.getElementById("slide-view");
const songTitle = document.getElementById("song-title");
const currentSlideText = document.getElementById("current-slide-text");
const nextSlideText = document.getElementById("next-slide-text");
const slideCounter = document.getElementById("slide-counter");
const slideProgressFill = document.getElementById("slide-progress-fill");
const heardText = document.getElementById("heard-text");
const listenBtn = document.getElementById("listen-btn");
const listenLabel = document.getElementById("listen-label");
const prevBtn = document.getElementById("prev-btn");
const nextBtn = document.getElementById("next-btn");
const fileInput = document.getElementById("file-input");
const tabs = document.querySelectorAll(".tab");
const modelList = document.getElementById("model-list");
const blankBtn = document.getElementById("blank-btn");
const selectPresenter = document.getElementById("select-presenter");
const presenterKeystrokeConfig = document.getElementById("presenter-keystroke-config");
const presenterRestConfig = document.getElementById("presenter-rest-config");
const presenterOpenLpConfig = document.getElementById("presenter-openlp-config");
const selectKeystrokeProfile = document.getElementById("select-keystroke-profile");
const inputPresenterHost = document.getElementById("input-presenter-host");
const inputPresenterPort = document.getElementById("input-presenter-port");
const restHelper = document.getElementById("rest-helper");
const inputOpenLpHost = document.getElementById("input-openlp-host");
const inputOpenLpPort = document.getElementById("input-openlp-port");
const inputOpenLpUsername = document.getElementById("input-openlp-username");
const inputOpenLpPassword = document.getElementById("input-openlp-password");
const connectPresenterBtn = document.getElementById("connect-presenter-btn");
const presenterStatus = document.getElementById("presenter-status");

// --- Setlist Loading ---
fileInput.addEventListener("change", async (e) => {
  const file = e.target.files[0];
  if (!file) return;
  const text = await file.text();
  await loadSetlist(text);
});

// Drag and drop
setlistLoader.addEventListener("dragover", (e) => {
  e.preventDefault();
  setlistLoader.style.borderColor = "var(--accent)";
});
setlistLoader.addEventListener("dragleave", () => {
  setlistLoader.style.borderColor = "";
});
setlistLoader.addEventListener("drop", async (e) => {
  e.preventDefault();
  setlistLoader.style.borderColor = "";
  const file = e.dataTransfer.files[0];
  if (file && file.name.endsWith(".json")) {
    const text = await file.text();
    await loadSetlist(text);
  }
});

async function loadSetlist(jsonText) {
  try {
    const songs = await invoke("load_setlist", { setlistJson: jsonText });

    // Flatten slides for display
    slides = [];
    let title = "";
    for (const song of songs) {
      title = song.title;
      for (const slide of song.slides) {
        slides.push(slide);
      }
    }

    currentSlideIndex = 0;
    songTitle.textContent = title;
    setlistLoader.hidden = true;
    slideView.hidden = false;
    updateSlideDisplay();
  } catch (err) {
    console.error("Failed to load setlist:", err);
    alert("Failed to load setlist: " + err);
  }
}

function updateSlideDisplay() {
  const curr = slides[currentSlideIndex];
  const next = slides[currentSlideIndex + 1];

  currentSlideText.textContent = curr ? curr.text : "—";
  nextSlideText.textContent = next ? next.text : "(End of setlist)";
  slideCounter.textContent = `${currentSlideIndex + 1} / ${slides.length}`;

  const pct = slides.length > 1
    ? (currentSlideIndex / (slides.length - 1)) * 100
    : 100;
  slideProgressFill.style.width = `${pct}%`;
}

// --- Listening Toggle ---
listenBtn.addEventListener("click", toggleListening);

async function toggleListening() {
  try {
    if (isListening) {
      await invoke("stop_listening");
      setListeningState(false);
    } else {
      await invoke("start_listening");
      setListeningState(true);
    }
  } catch (err) {
    console.error("Listen toggle error:", err);
    alert(String(err));
    setListeningState(false);
  }
}

function setListeningState(listening) {
  isListening = listening;
  listenBtn.classList.toggle("active", listening);
  statusBadge.classList.toggle("listening", listening);
  statusBadge.classList.toggle("ready", !listening);
  statusBadge.textContent = listening ? "Listening" : "Ready";
  listenLabel.textContent = listening ? "Stop" : "Start Listening";
  heardText.textContent = listening ? "Waiting for audio..." : "—";
}

// --- Manual Slide Controls ---
prevBtn.addEventListener("click", async () => {
  try {
    await invoke("prev_slide_manual");
    if (currentSlideIndex > 0) {
      currentSlideIndex--;
      updateSlideDisplay();
    }
  } catch (err) {
    console.error(err);
  }
});

nextBtn.addEventListener("click", async () => {
  try {
    await invoke("next_slide_manual");
    if (currentSlideIndex < slides.length - 1) {
      currentSlideIndex++;
      updateSlideDisplay();
    }
  } catch (err) {
    console.error(err);
    alert("Next slide failed: " + err);
  }
});

if (blankBtn) {
  blankBtn.addEventListener("click", async () => {
    try {
      await invoke("blank_manual");
    } catch (err) {
      console.error(err);
      alert("Blank failed: " + err);
    }
  });
}

// --- Events ---
async function setupListeners() {
  await listen("transcription", (event) => {
    const { text } = event.payload;
    if (text) {
      heardText.textContent = text;
    }
  });

  await listen("slide-advanced", (event) => {
    const { slide_index } = event.payload;
    currentSlideIndex = slide_index;
    updateSlideDisplay();
  });

  await listen("download-progress", (event) => {
    updateDownloadProgress(event.payload.percent);
  });

  await listen("tray-toggle-recording", () => {
    toggleListening();
  });
}

// --- Tab Navigation ---
const homeSection = document.getElementById("section-home");
const modelsSection = document.getElementById("section-models");
const settingsSection = document.getElementById("section-settings");

tabs.forEach((tab) => {
  tab.addEventListener("click", () => {
    tabs.forEach((t) => t.classList.remove("active"));
    tab.classList.add("active");

    const name = tab.dataset.tab;
    homeSection.hidden = name !== "home";
    modelsSection.hidden = name !== "models";
    settingsSection.hidden = name !== "settings";

    if (name === "models") loadModels();
    if (name === "settings") {
      loadSettings();
      loadPresenterPanel();
    }
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
  const bar = document.getElementById(`progress-${modelName}`);
  if (bar) bar.hidden = false;
  try {
    await invoke("download_model", { modelName });
    loadModels();
  } catch (err) {
    console.error("Download failed:", err);
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

    const deviceSelect = document.getElementById("select-device");
    deviceSelect.innerHTML = '<option value="">Default</option>';
    devices.forEach((d) => {
      const opt = document.createElement("option");
      opt.value = d;
      opt.textContent = d;
      if (settings.device === d) opt.selected = true;
      deviceSelect.appendChild(opt);
    });

    const modelSelect = document.getElementById("select-model");
    modelSelect.innerHTML = "";
    models.forEach((m) => {
      if (m.downloaded) {
        const opt = document.createElement("option");
        opt.value = m.name;
        opt.textContent = m.display_name;
        modelSelect.appendChild(opt);
      }
    });
    if (modelSelect.options.length === 0) {
      const opt = document.createElement("option");
      opt.textContent = "No models downloaded";
      opt.disabled = true;
      modelSelect.appendChild(opt);
    }

    const threshold = document.getElementById("input-threshold");
    const thresholdVal = document.getElementById("threshold-val");
    threshold.value = settings.similarity_threshold;
    thresholdVal.textContent = settings.similarity_threshold + "%";
    threshold.addEventListener("input", () => {
      thresholdVal.textContent = threshold.value + "%";
    });

    const margin = document.getElementById("input-margin");
    const marginVal = document.getElementById("margin-val");
    margin.value = settings.margin;
    marginVal.textContent = settings.margin;
    margin.addEventListener("input", () => {
      marginVal.textContent = margin.value;
    });
  } catch (err) {
    console.error("Failed to load settings:", err);
  }
}

// --- Presenter (Connection panel) ---
async function loadPresenterPanel() {
  if (!selectPresenter) return;
  try {
    const [kinds, info] = await Promise.all([
      invoke("list_presenters"),
      invoke("get_presenter_info"),
    ]);

    if (selectPresenter.options.length === 0) {
      kinds.forEach((k) => {
        const opt = document.createElement("option");
        opt.value = k.kind;
        opt.textContent = k.display_name;
        selectPresenter.appendChild(opt);
      });
    }

    selectPresenter.value = info.kind;
    showPresenterConfigFor(info.kind);
    updatePresenterStatus(info);
  } catch (err) {
    console.error("loadPresenterPanel:", err);
  }
}

function showPresenterConfigFor(kind) {
  presenterKeystrokeConfig.hidden = kind !== "keystroke";
  // The host/port pair is reused for both ProPresenter REST and FreeShow; the helper
  // copy and the default port are the only differences.
  const usesHostPort = kind === "pro_presenter7_rest" || kind === "free_show";
  presenterRestConfig.hidden = !usesHostPort;
  if (kind === "pro_presenter7_rest") {
    inputPresenterPort.value = "1025";
    if (restHelper) restHelper.textContent = "Set in ProPresenter → Settings → Network. Default 1025.";
  } else if (kind === "free_show") {
    inputPresenterPort.value = "5506";
    if (restHelper) restHelper.textContent = "Enable FreeShow → Settings → Connections. Default 5506 (HTTP).";
  }
  presenterOpenLpConfig.hidden = kind !== "open_lp_v2";
}

function updatePresenterStatus(info) {
  if (!presenterStatus) return;
  if (info && info.connected) {
    presenterStatus.textContent = "Connected: " + info.display_name;
    presenterStatus.classList.add("ok");
    presenterStatus.classList.remove("err");
  } else {
    presenterStatus.textContent = "Not connected";
    presenterStatus.classList.remove("ok", "err");
  }
}

if (selectPresenter) {
  selectPresenter.addEventListener("change", () => {
    showPresenterConfigFor(selectPresenter.value);
  });
}

if (connectPresenterBtn) {
  connectPresenterBtn.addEventListener("click", async () => {
    const kind = selectPresenter.value;
    const args = { kind };
    if (kind === "keystroke") {
      args.keystroke_profile = selectKeystrokeProfile.value;
    } else if (kind === "pro_presenter7_rest" || kind === "free_show") {
      args.host = inputPresenterHost.value.trim() || "127.0.0.1";
      args.port = parseInt(inputPresenterPort.value, 10) ||
        (kind === "pro_presenter7_rest" ? 1025 : 5506);
    } else if (kind === "open_lp_v2") {
      args.host = inputOpenLpHost.value.trim() || "127.0.0.1";
      args.port = parseInt(inputOpenLpPort.value, 10) || 4316;
      const user = inputOpenLpUsername.value.trim();
      const pass = inputOpenLpPassword.value;
      if (user) args.username = user;
      if (pass) args.password = pass;
    }
    connectPresenterBtn.disabled = true;
    presenterStatus.textContent = "Connecting...";
    presenterStatus.classList.remove("ok", "err");
    try {
      const info = await invoke("connect_presenter", { args });
      updatePresenterStatus(info);
    } catch (err) {
      console.error("connect_presenter:", err);
      presenterStatus.textContent = "Failed: " + err;
      presenterStatus.classList.add("err");
      presenterStatus.classList.remove("ok");
    } finally {
      connectPresenterBtn.disabled = false;
    }
  });
}

// --- Helpers ---
function escapeHtml(str) {
  const div = document.createElement("div");
  div.textContent = str;
  return div.innerHTML;
}

// --- Init ---
setupListeners();
loadPresenterPanel();
