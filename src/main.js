const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// --- State ---
let isListening = false;
let slides = [];
let currentSlideIndex = 0;
// Full setlist (array of {title, slides: [...]}) — kept so we can re-export
// what was loaded.
let currentSongs = [];

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
const presenterOpenSongConfig = document.getElementById("presenter-opensong-config");
const selectKeystrokeProfile = document.getElementById("select-keystroke-profile");
const inputPresenterHost = document.getElementById("input-presenter-host");
const inputPresenterPort = document.getElementById("input-presenter-port");
const restHelper = document.getElementById("rest-helper");
const inputOpenLpHost = document.getElementById("input-openlp-host");
const inputOpenLpPort = document.getElementById("input-openlp-port");
const inputOpenLpUsername = document.getElementById("input-openlp-username");
const inputOpenLpPassword = document.getElementById("input-openlp-password");
const inputOpenSongHost = document.getElementById("input-opensong-host");
const inputOpenSongPort = document.getElementById("input-opensong-port");
const inputOpenSongKey = document.getElementById("input-opensong-key");
const presenterPro7WsConfig = document.getElementById("presenter-pro7ws-config");
const inputPro7WsHost = document.getElementById("input-pro7ws-host");
const inputPro7WsPort = document.getElementById("input-pro7ws-port");
const inputPro7WsPassword = document.getElementById("input-pro7ws-password");
const connectPresenterBtn = document.getElementById("connect-presenter-btn");
const discoverBtn = document.getElementById("discover-btn");
const discoveredList = document.getElementById("discovered-list");
const presenterStatus = document.getElementById("presenter-status");

// --- Setlist Loading ---
const importError = document.getElementById("import-error");

// Planning Center (in-memory credentials only).
const pcoAppId = document.getElementById("pco-app-id");
const pcoSecret = document.getElementById("pco-secret");
const pcoConnectBtn = document.getElementById("pco-connect-btn");
const pcoStatus = document.getElementById("pco-status");
const pcoStepService = document.getElementById("pco-step-service");
const pcoServiceType = document.getElementById("pco-service-type");
const pcoStepPlan = document.getElementById("pco-step-plan");
const pcoPlan = document.getElementById("pco-plan");
const pcoImportBtn = document.getElementById("pco-import-btn");
let pcoCreds = null;

if (pcoConnectBtn) {
  pcoConnectBtn.addEventListener("click", pcoConnect);
}
if (pcoServiceType) {
  pcoServiceType.addEventListener("change", pcoLoadPlans);
}
if (pcoImportBtn) {
  pcoImportBtn.addEventListener("click", pcoImportSelectedPlan);
}

async function pcoConnect() {
  const appId = (pcoAppId?.value || "").trim();
  const secret = pcoSecret?.value || "";
  if (!appId || !secret) {
    setPcoStatus("Enter both an application ID and secret.", "err");
    return;
  }
  pcoConnectBtn.disabled = true;
  setPcoStatus("Verifying...");
  try {
    const name = await invoke("pco_verify", { appId, secret });
    pcoCreds = { appId, secret };
    setPcoStatus(`Connected as ${name}`, "ok");
    const types = await invoke("pco_list_service_types", { appId, secret });
    pcoServiceType.innerHTML = "";
    if (!types.length) {
      setPcoStatus("No service types on this account.", "err");
      return;
    }
    for (const t of types) {
      const opt = document.createElement("option");
      opt.value = t.id;
      opt.textContent = t.name;
      pcoServiceType.appendChild(opt);
    }
    pcoStepService.hidden = false;
    await pcoLoadPlans();
  } catch (err) {
    console.error("PCO connect:", err);
    setPcoStatus("Failed: " + err, "err");
  } finally {
    pcoConnectBtn.disabled = false;
  }
}

async function pcoLoadPlans() {
  if (!pcoCreds || !pcoServiceType.value) return;
  pcoStepPlan.hidden = true;
  pcoPlan.innerHTML = "";
  setPcoStatus("Loading plans...");
  try {
    const plans = await invoke("pco_list_plans", {
      appId: pcoCreds.appId,
      secret: pcoCreds.secret,
      serviceTypeId: pcoServiceType.value,
    });
    if (!plans.length) {
      setPcoStatus("No future plans on this service type.", "err");
      return;
    }
    for (const p of plans) {
      const opt = document.createElement("option");
      opt.value = p.id;
      const label = p.dates ? `${p.dates} — ${p.title || "(untitled)"}` : (p.title || p.id);
      opt.textContent = label;
      pcoPlan.appendChild(opt);
    }
    pcoStepPlan.hidden = false;
    setPcoStatus(`${plans.length} plan${plans.length === 1 ? "" : "s"} found.`, "ok");
  } catch (err) {
    console.error("PCO plans:", err);
    setPcoStatus("Failed: " + err, "err");
  }
}

async function pcoImportSelectedPlan() {
  if (!pcoCreds || !pcoServiceType.value || !pcoPlan.value) return;
  pcoImportBtn.disabled = true;
  setPcoStatus("Importing songs...");
  try {
    const songs = await invoke("pco_import_plan", {
      appId: pcoCreds.appId,
      secret: pcoCreds.secret,
      serviceTypeId: pcoServiceType.value,
      planId: pcoPlan.value,
    });
    if (!songs || !songs.length) {
      setPcoStatus("Plan imported but contained no usable songs.", "err");
      return;
    }
    let id = 0;
    for (const song of songs) {
      for (const slide of song.slides) slide.id = id++;
    }
    await loadSetlist(JSON.stringify({ setlist: songs }));
    setPcoStatus(`Imported ${songs.length} song${songs.length === 1 ? "" : "s"}.`, "ok");
  } catch (err) {
    console.error("PCO import:", err);
    setPcoStatus("Failed: " + err, "err");
  } finally {
    pcoImportBtn.disabled = false;
  }
}

function setPcoStatus(msg, kind) {
  if (!pcoStatus) return;
  pcoStatus.textContent = msg;
  pcoStatus.classList.remove("ok", "err");
  if (kind) pcoStatus.classList.add(kind);
}

fileInput.addEventListener("change", async (e) => {
  await importFiles(Array.from(e.target.files || []));
  // Clear the input so the same file can be re-selected.
  fileInput.value = "";
});

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
  await importFiles(Array.from(e.dataTransfer.files || []));
});

async function importFiles(files) {
  if (!files.length) return;
  if (importError) {
    importError.hidden = true;
    importError.textContent = "";
  }

  // Single .json or .voxxa-set replaces the entire setlist via the existing
  // path. Both formats are JSON; .voxxa-set just adds a wrapping envelope.
  if (files.length === 1) {
    const lower = files[0].name.toLowerCase();
    if (lower.endsWith(".json") || lower.endsWith(".voxxa-set")) {
      const text = await files[0].text();
      await loadSetlist(unwrapVoxxaSet(text));
      return;
    }
  }

  const collected = [];
  const failures = [];
  for (const file of files) {
    try {
      const song = await parseOneFile(file);
      if (song) collected.push(song);
    } catch (err) {
      console.error(`Failed to parse ${file.name}:`, err);
      failures.push(`${file.name}: ${err}`);
    }
  }
  if (!collected.length) {
    showImportError(`No songs imported. ${failures.join("; ") || ""}`);
    return;
  }
  // Re-id slides globally so the conductor's ids line up with the frontend's flat view.
  let id = 0;
  for (const song of collected) {
    for (const slide of song.slides) {
      slide.id = id++;
    }
  }
  await loadSetlist(JSON.stringify({ setlist: collected }));
  if (failures.length) {
    showImportError(`Imported ${collected.length}, skipped: ${failures.join("; ")}`);
  }
}

async function parseOneFile(file) {
  const lower = file.name.toLowerCase();
  const fallbackTitle = file.name.replace(/\.[^.]+$/, "");

  if (lower.endsWith(".pptx") || lower.endsWith(".pdf")) {
    const bytesB64 = await readFileAsBase64(file);
    return invoke("parse_song_bytes", {
      bytesB64,
      format: lower.endsWith(".pdf") ? "pdf" : "pptx",
      fallbackTitle,
    });
  }

  // Single .json file shouldn't get here, but multiple-with-json means treat
  // each .json as a Song-shaped or Setlist-shaped object.
  if (lower.endsWith(".json")) {
    const text = await file.text();
    const parsed = JSON.parse(text);
    if (parsed.setlist && Array.isArray(parsed.setlist)) {
      // Flatten into one virtual song — usually .json shouldn't be combined
      // with other files, but if the user does it, treat each contained song
      // as its own entry.
      return parsed.setlist;
    }
    if (parsed.title && Array.isArray(parsed.slides)) {
      return parsed;
    }
    throw new Error("unrecognised JSON shape");
  }

  const text = await file.text();
  let format;
  if (lower.endsWith(".txt")) {
    format = "txt";
  } else if (lower.endsWith(".openlyrics")) {
    format = "open_lyrics";
  } else if (lower.endsWith(".cho") || lower.endsWith(".chordpro") || lower.endsWith(".pro")) {
    format = "chord_pro";
  } else if (lower.endsWith(".xml")) {
    // Disambiguate by namespace / root element.
    format = sniffXmlFormat(text);
  } else {
    throw new Error("unknown extension");
  }
  return invoke("parse_song_text", {
    content: text,
    format,
    fallbackTitle,
  });
}

function sniffXmlFormat(xml) {
  const head = xml.slice(0, 1024).toLowerCase();
  if (head.includes("openlyrics.info")) return "open_lyrics";
  // OpenSong files have a bare `<song>` root with no namespace.
  if (/\<song(\s|>)/.test(head)) return "open_song";
  // Default to OpenLyrics — at least it has a stricter parser.
  return "open_lyrics";
}

function readFileAsBase64(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      // FileReader's data URL is "data:...;base64,XXXX"; strip the prefix.
      const url = reader.result;
      const comma = url.indexOf(",");
      resolve(comma >= 0 ? url.slice(comma + 1) : url);
    };
    reader.onerror = () => reject(reader.error || new Error("read failed"));
    reader.readAsDataURL(file);
  });
}

function showImportError(msg) {
  if (!importError) return;
  importError.textContent = msg;
  importError.hidden = false;
}

async function loadSetlist(jsonText) {
  try {
    const songs = await invoke("load_setlist", { setlistJson: jsonText });
    currentSongs = songs;
    slides = [];
    let title = "";
    for (const song of songs) {
      title = song.title;
      for (const slide of song.slides) {
        slides.push(slide);
      }
    }
    currentSlideIndex = 0;
    songTitle.textContent = songs.length > 1
      ? `${songs.length} songs · starting ${title}`
      : title;
    setlistLoader.hidden = true;
    slideView.hidden = false;
    updateSlideDisplay();
  } catch (err) {
    console.error("Failed to load setlist:", err);
    showImportError("Failed to load setlist: " + err);
  }
}

// --- Setlist export (.voxxa-set) ---
const exportSetlistBtn = document.getElementById("export-setlist-btn");
if (exportSetlistBtn) {
  exportSetlistBtn.addEventListener("click", exportCurrentSetlist);
}

function exportCurrentSetlist() {
  if (!currentSongs.length) return;
  const payload = {
    voxxa_set_version: 1,
    exported_at: new Date().toISOString(),
    setlist: currentSongs,
  };
  const blob = new Blob([JSON.stringify(payload, null, 2)], {
    type: "application/json",
  });
  const a = document.createElement("a");
  const stamp = new Date().toISOString().slice(0, 10);
  const stem = currentSongs.length === 1
    ? sanitizeFileName(currentSongs[0].title)
    : `setlist-${stamp}`;
  a.href = URL.createObjectURL(blob);
  a.download = `${stem}.voxxa-set`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  // Releasing the object URL frees the in-memory blob once the download
  // dialog has consumed it.
  setTimeout(() => URL.revokeObjectURL(a.href), 1000);
}

function sanitizeFileName(name) {
  return (name || "setlist").replace(/[^a-z0-9_\- ]/gi, "").trim() || "setlist";
}

// Strip the .voxxa-set envelope and return the inner setlist JSON unchanged.
// A bare {setlist:[...]} (the original format) passes through.
function unwrapVoxxaSet(text) {
  try {
    const parsed = JSON.parse(text);
    if (parsed && parsed.voxxa_set_version && parsed.setlist) {
      return JSON.stringify({ setlist: parsed.setlist });
    }
  } catch {
    // Not JSON or malformed — let the backend error.
  }
  return text;
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

// --- Inline slide editor ---
function attachSlideEditor(el) {
  if (!el) return;
  el.addEventListener("dblclick", () => beginEditSlide(el));
}
attachSlideEditor(currentSlideText);
attachSlideEditor(nextSlideText);

function beginEditSlide(el) {
  const offset = parseInt(el.dataset.slideOffset || "0", 10);
  const slideIdx = currentSlideIndex + offset;
  const slide = slides[slideIdx];
  if (!slide) return;

  el.contentEditable = "true";
  el.classList.add("editing");
  // Place the caret at the end of the existing text.
  el.focus();
  const range = document.createRange();
  range.selectNodeContents(el);
  range.collapse(false);
  const sel = window.getSelection();
  sel.removeAllRanges();
  sel.addRange(range);

  const finish = async (commit) => {
    el.removeEventListener("blur", onBlur);
    el.removeEventListener("keydown", onKey);
    el.contentEditable = "false";
    el.classList.remove("editing");
    if (!commit) {
      el.textContent = slide.text;
      return;
    }
    const newText = el.innerText.replace(/ /g, " ").trim();
    if (newText === slide.text || !newText) {
      el.textContent = slide.text;
      return;
    }
    // Mutating the slide also updates currentSongs because slides[] holds
    // references to the same Slide objects.
    slide.text = newText;
    await reloadConductorPreservingPosition();
  };

  const onBlur = () => finish(true);
  const onKey = (e) => {
    if (e.key === "Escape") {
      e.preventDefault();
      finish(false);
    } else if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      finish(true);
    }
  };
  el.addEventListener("blur", onBlur);
  el.addEventListener("keydown", onKey);
}

// Push the in-memory currentSongs back into the conductor. The conductor
// resets to slide 0 on load, so we manually re-advance to where we were.
async function reloadConductorPreservingPosition() {
  if (!currentSongs.length) return;
  const targetIdx = currentSlideIndex;
  try {
    await invoke("load_setlist", {
      setlistJson: JSON.stringify({ setlist: currentSongs }),
    });
    // Rebuild the flat slides[] from the (possibly edited) currentSongs.
    slides = [];
    for (const song of currentSongs) {
      for (const slide of song.slides) slides.push(slide);
    }
    currentSlideIndex = Math.min(targetIdx, slides.length - 1);
    updateSlideDisplay();
  } catch (err) {
    console.error("Failed to apply slide edit:", err);
    alert("Failed to save edit: " + err);
  }
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
const micMeterFill = document.getElementById("mic-meter-fill");
const micMeterPeak = document.getElementById("mic-meter-peak");
let micPeakHold = 0;
let micPeakDecayTimer = null;

async function setupListeners() {
  await listen("transcription", (event) => {
    const { text } = event.payload;
    if (text) {
      heardText.textContent = text;
    }
  });

  await listen("mic-level", (event) => {
    const { peak, rms } = event.payload;
    if (micMeterFill) {
      // RMS-based fill maps to perceptual "loudness" better than peak.
      // Visual scale: 0–0.3 covers normal speech.
      const pct = Math.min(rms / 0.3, 1) * 100;
      micMeterFill.style.width = `${pct}%`;
    }
    if (micMeterPeak) {
      // Peak indicator with slow decay so transients are visible.
      if (peak > micPeakHold) {
        micPeakHold = peak;
        clearTimeout(micPeakDecayTimer);
        micPeakDecayTimer = setTimeout(() => {
          micPeakHold = 0;
          if (micMeterPeak) micMeterPeak.style.left = "0%";
        }, 600);
      }
      const peakPct = Math.min(micPeakHold / 0.5, 1) * 100;
      micMeterPeak.style.left = `${peakPct}%`;
    }
  });

  await listen("slide-advanced", (event) => {
    const { slide_index, song_title } = event.payload;
    currentSlideIndex = slide_index;
    if (song_title && songTitle) songTitle.textContent = song_title;
    updateSlideDisplay();
  });

  await listen("machine-state", (event) => {
    const { state, is_blank } = event.payload;
    if (!statusBadge) return;
    // Show LISTENING / SINGING / BLANK_HOLD / INTER_VERSE_SILENCE in the header.
    const label =
      state === "singing"
        ? "Singing"
        : state === "blank_hold"
        ? "Blank (no match)"
        : state === "inter_verse_silence"
        ? "Holding"
        : "Listening";
    statusBadge.textContent = is_blank ? `${label} · BLANK` : label;
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
      loadSmartConfig();
      refreshPrivacyCount();
    }
  });
});

// Poll the outbound-request counter while the Settings tab is visible. 5 s is
// generous — the counter only moves when the user triggers an action, so
// faster polling would be wasted IPC.
const privacyCount = document.getElementById("privacy-count");
const privacyCard = document.getElementById("privacy-card");
async function refreshPrivacyCount() {
  if (!privacyCount) return;
  try {
    const stats = await invoke("get_network_stats");
    const n = stats.outbound_requests;
    privacyCount.textContent =
      n === 0
        ? "0 outbound network requests this session"
        : `${n} outbound request${n === 1 ? "" : "s"} this session (all to apps you connected to)`;
    if (privacyCard) {
      privacyCard.classList.toggle("zero", n === 0);
    }
  } catch (err) {
    console.error("get_network_stats:", err);
  }
}
setInterval(() => {
  // Only refresh if Settings panel is the visible tab — cheap DOM check.
  if (!settingsSection.hidden) refreshPrivacyCount();
}, 5000);

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

    // Restore last-used non-secret form fields. Passwords are never persisted
    // and the user re-enters them each session.
    const saved = readJsonStorage(PRESENTER_STORAGE_KEY);
    const kindToShow = (saved && saved.kind) || info.kind;
    selectPresenter.value = kindToShow;
    if (saved) restorePresenterForm(saved);
    showPresenterConfigFor(kindToShow);
    updatePresenterStatus(info);
  } catch (err) {
    console.error("loadPresenterPanel:", err);
  }
}

function restorePresenterForm(saved) {
  if (saved.host) {
    if (inputPresenterHost) inputPresenterHost.value = saved.host;
    if (inputOpenLpHost) inputOpenLpHost.value = saved.host;
    if (inputOpenSongHost) inputOpenSongHost.value = saved.host;
    if (inputPro7WsHost) inputPro7WsHost.value = saved.host;
  }
  if (saved.port) {
    if (inputPresenterPort) inputPresenterPort.value = saved.port;
    if (inputOpenLpPort) inputOpenLpPort.value = saved.port;
    if (inputOpenSongPort) inputOpenSongPort.value = saved.port;
    if (inputPro7WsPort) inputPro7WsPort.value = saved.port;
  }
  if (saved.username && inputOpenLpUsername) {
    inputOpenLpUsername.value = saved.username;
  }
  if (saved.keystroke_profile && selectKeystrokeProfile) {
    selectKeystrokeProfile.value = saved.keystroke_profile;
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
  if (presenterOpenSongConfig) presenterOpenSongConfig.hidden = kind !== "open_song";
  // Same WS config form (host/port/password) for both Pro7 7.0-7.8 and Pro6.
  if (presenterPro7WsConfig) {
    presenterPro7WsConfig.hidden =
      kind !== "pro_presenter7_ws" && kind !== "pro_presenter6_ws";
  }
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

if (discoverBtn) {
  discoverBtn.addEventListener("click", async () => {
    discoverBtn.disabled = true;
    discoverBtn.textContent = "Scanning...";
    discoveredList.hidden = true;
    discoveredList.innerHTML = "";
    try {
      const found = await invoke("discover_presenters", { timeoutMs: 2500 });
      renderDiscovered(found || []);
    } catch (err) {
      console.error("discover_presenters:", err);
      discoveredList.hidden = false;
      discoveredList.innerHTML =
        '<div class="discovered-empty">Discovery failed: ' + escapeHtml(String(err)) + "</div>";
    } finally {
      discoverBtn.disabled = false;
      discoverBtn.textContent = "Auto-detect";
    }
  });
}

function renderDiscovered(found) {
  discoveredList.hidden = false;
  discoveredList.innerHTML = "";
  if (!found.length) {
    discoveredList.innerHTML =
      '<div class="discovered-empty">No presentation apps found. ' +
      "Make sure the app is running and its remote API is enabled.</div>";
    return;
  }
  for (const svc of found) {
    const row = document.createElement("button");
    row.className = "discovered-row";
    row.innerHTML =
      '<div class="discovered-name">' + escapeHtml(svc.name) + "</div>" +
      '<div class="discovered-meta">' +
      escapeHtml(svc.host) + ":" + svc.port + " · " + escapeHtml(svc.source) +
      "</div>";
    row.addEventListener("click", () => useDiscovered(svc));
    discoveredList.appendChild(row);
  }
}

function useDiscovered(svc) {
  selectPresenter.value = svc.kind;
  showPresenterConfigFor(svc.kind);
  if (svc.kind === "pro_presenter7_rest" || svc.kind === "free_show") {
    inputPresenterHost.value = svc.host;
    inputPresenterPort.value = svc.port;
  } else if (svc.kind === "open_lp_v2") {
    inputOpenLpHost.value = svc.host;
    inputOpenLpPort.value = svc.port;
  } else if (svc.kind === "open_song") {
    inputOpenSongHost.value = svc.host;
    inputOpenSongPort.value = svc.port;
  }
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
    } else if (kind === "open_song") {
      args.host = inputOpenSongHost.value.trim() || "127.0.0.1";
      args.port = parseInt(inputOpenSongPort.value, 10) || 8082;
      const key = inputOpenSongKey.value;
      if (key) args.password = key;
    } else if (kind === "pro_presenter7_ws" || kind === "pro_presenter6_ws") {
      args.host = inputPro7WsHost.value.trim() || "127.0.0.1";
      args.port = parseInt(inputPro7WsPort.value, 10) || 50001;
      args.password = inputPro7WsPassword.value || "";
    }
    connectPresenterBtn.disabled = true;
    presenterStatus.textContent = "Connecting...";
    presenterStatus.classList.remove("ok", "err");
    try {
      const info = await invoke("connect_presenter", { args });
      updatePresenterStatus(info);
      // Persist non-secret form values so the next launch starts here.
      writeJsonStorage(PRESENTER_STORAGE_KEY, {
        kind: args.kind,
        host: args.host,
        port: args.port,
        username: args.username,
        keystroke_profile: args.keystroke_profile,
      });
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

// --- First-launch welcome ---
const FIRST_LAUNCH_KEY = "voxxa.firstLaunchDone";
const welcomeOverlay = document.getElementById("welcome-overlay");
const welcomeDetect = document.getElementById("welcome-detect");
const welcomeModel = document.getElementById("welcome-model");
const welcomeSetlist = document.getElementById("welcome-setlist");
const welcomeDismiss = document.getElementById("welcome-dismiss");

function dismissWelcome() {
  if (welcomeOverlay) welcomeOverlay.hidden = true;
  try {
    localStorage.setItem(FIRST_LAUNCH_KEY, "1");
  } catch {
    // Ignore — localStorage can be disabled in some WebKit configurations.
  }
}

function maybeShowWelcome() {
  let seen = false;
  try {
    seen = localStorage.getItem(FIRST_LAUNCH_KEY) === "1";
  } catch {
    seen = false;
  }
  if (!seen && welcomeOverlay) {
    welcomeOverlay.hidden = false;
  }
}

if (welcomeDismiss) welcomeDismiss.addEventListener("click", dismissWelcome);

if (welcomeDetect) {
  welcomeDetect.addEventListener("click", () => {
    dismissWelcome();
    // Switch to Settings tab and trigger discovery.
    const settingsTab = document.querySelector('.tab[data-tab="settings"]');
    if (settingsTab) settingsTab.click();
    if (discoverBtn) discoverBtn.click();
  });
}

if (welcomeModel) {
  welcomeModel.addEventListener("click", () => {
    dismissWelcome();
    const modelsTab = document.querySelector('.tab[data-tab="models"]');
    if (modelsTab) modelsTab.click();
  });
}

if (welcomeSetlist) {
  welcomeSetlist.addEventListener("click", () => {
    dismissWelcome();
    if (fileInput) fileInput.click();
  });
}

// --- Smart-blanking threshold tuning ---
const smartSilence = document.getElementById("smart-silence");
const smartSilenceVal = document.getElementById("smart-silence-val");
const smartUnrec = document.getElementById("smart-unrecognized");
const smartUnrecVal = document.getElementById("smart-unrecognized-val");
const smartConf = document.getElementById("smart-confidence");
const smartConfVal = document.getElementById("smart-confidence-val");
const smartDwell = document.getElementById("smart-dwell");
const smartDwellVal = document.getElementById("smart-dwell-val");
let smartConfig = null;

async function loadSmartConfig() {
  if (!smartSilence) return;
  try {
    smartConfig = await invoke("get_smart_config");
    // Persisted overrides shadow the backend defaults so the user's last
    // session-tuned values come back automatically.
    const stored = readJsonStorage(SMART_STORAGE_KEY);
    if (stored && typeof stored === "object") {
      smartConfig = { ...smartConfig, ...stored };
      // Push back to the backend in case it started with defaults.
      try { await invoke("set_smart_config", { cfg: smartConfig }); } catch {}
    }
    smartSilence.value = smartConfig.silence_to_blank_secs;
    smartUnrec.value = smartConfig.unrecognized_speech_to_blank_secs;
    smartConf.value = Math.round(smartConfig.song_confidence_floor * 100);
    smartDwell.value = smartConfig.min_song_dwell_secs;
    updateSmartLabels();
  } catch (err) {
    console.error("get_smart_config:", err);
  }
}

const SMART_STORAGE_KEY = "voxxa.smartConfig";
const PRESENTER_STORAGE_KEY = "voxxa.presenterForm";

function readJsonStorage(key) {
  try {
    const raw = localStorage.getItem(key);
    return raw ? JSON.parse(raw) : null;
  } catch {
    return null;
  }
}

function writeJsonStorage(key, value) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // Storage quota exceeded or disabled — non-fatal.
  }
}

function updateSmartLabels() {
  smartSilenceVal.textContent = parseFloat(smartSilence.value).toFixed(1) + " s";
  smartUnrecVal.textContent = parseFloat(smartUnrec.value).toFixed(1) + " s";
  smartConfVal.textContent = smartConf.value + " %";
  smartDwellVal.textContent = parseFloat(smartDwell.value).toFixed(1) + " s";
}

// Debounce config writes so dragging a slider doesn't flood IPC.
let smartSaveTimer = null;
function saveSmartConfigSoon() {
  updateSmartLabels();
  clearTimeout(smartSaveTimer);
  smartSaveTimer = setTimeout(async () => {
    if (!smartConfig) return;
    const cfg = {
      ...smartConfig,
      silence_to_blank_secs: parseFloat(smartSilence.value),
      unrecognized_speech_to_blank_secs: parseFloat(smartUnrec.value),
      song_confidence_floor: parseInt(smartConf.value, 10) / 100,
      min_song_dwell_secs: parseFloat(smartDwell.value),
    };
    try {
      await invoke("set_smart_config", { cfg });
      smartConfig = cfg;
      writeJsonStorage(SMART_STORAGE_KEY, cfg);
    } catch (err) {
      console.error("set_smart_config:", err);
    }
  }, 250);
}

[smartSilence, smartUnrec, smartConf, smartDwell].forEach((el) => {
  if (el) el.addEventListener("input", saveSmartConfigSoon);
});

// --- Diagnostics ---
const diagBtn = document.getElementById("diag-btn");
const diagOverlay = document.getElementById("diag-overlay");
const diagClose = document.getElementById("diag-close");
const diagText = document.getElementById("diag-text");
const diagCopy = document.getElementById("diag-copy");

if (diagBtn) {
  diagBtn.addEventListener("click", async () => {
    try {
      const report = await invoke("generate_diagnostic_report");
      diagText.value = JSON.stringify(report, null, 2);
      diagOverlay.hidden = false;
    } catch (err) {
      console.error("diagnostic report:", err);
      alert("Failed to generate report: " + err);
    }
  });
}
if (diagClose) diagClose.addEventListener("click", () => diagOverlay.hidden = true);
if (diagOverlay) {
  diagOverlay.addEventListener("click", (e) => {
    // Click outside the card closes; click inside doesn't.
    if (e.target === diagOverlay) diagOverlay.hidden = true;
  });
}
if (diagCopy) {
  diagCopy.addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(diagText.value);
      diagCopy.textContent = "Copied";
      setTimeout(() => (diagCopy.textContent = "Copy to clipboard"), 1500);
    } catch (err) {
      // Fallback for environments without async clipboard.
      diagText.select();
      document.execCommand("copy");
    }
  });
}

// --- Init ---
setupListeners();
loadPresenterPanel();
maybeShowWelcome();
