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

// Hymnary.org public-domain hymn lookup.
const hymnaryInput = document.getElementById("hymnary-q");
const hymnarySearchBtn = document.getElementById("hymnary-search-btn");
const hymnaryResults = document.getElementById("hymnary-results");

if (hymnarySearchBtn) {
  hymnarySearchBtn.addEventListener("click", runHymnarySearch);
}
if (hymnaryInput) {
  hymnaryInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      runHymnarySearch();
    }
  });
}

async function runHymnarySearch() {
  const q = (hymnaryInput?.value || "").trim();
  if (!q) return;
  hymnaryResults.innerHTML = '<div class="discovered-empty">Searching…</div>';
  hymnarySearchBtn.disabled = true;
  try {
    const results = await invoke("hymnary_search", { query: q });
    renderHymnaryResults(results || []);
  } catch (err) {
    console.error("hymnary_search:", err);
    hymnaryResults.innerHTML =
      '<div class="discovered-empty">Search failed: ' + escapeHtml(String(err)) + "</div>";
  } finally {
    hymnarySearchBtn.disabled = false;
  }
}

function renderHymnaryResults(results) {
  hymnaryResults.innerHTML = "";
  if (!results.length) {
    hymnaryResults.innerHTML =
      '<div class="discovered-empty">No matches. Try a shorter query.</div>';
    return;
  }
  for (const r of results.slice(0, 20)) {
    const row = document.createElement("button");
    row.className = "discovered-row";
    const meta = [r.author, r.year].filter(Boolean).join(" · ");
    row.innerHTML =
      '<div class="discovered-name">' + escapeHtml(r.title) + "</div>" +
      (meta ? '<div class="discovered-meta">' + escapeHtml(meta) + "</div>" : "");
    row.addEventListener("click", () => loadHymnaryHymn(r));
    hymnaryResults.appendChild(row);
  }
}

async function loadHymnaryHymn(result) {
  try {
    const song = await invoke("hymnary_fetch", { slug: result.slug });
    if (!song) return;
    let id = 0;
    for (const slide of song.slides) slide.id = id++;
    await loadSetlist(JSON.stringify({ setlist: [song] }));
  } catch (err) {
    console.error("hymnary_fetch:", err);
    toast("Couldn't load this hymn: " + err, "error");
  }
}

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
    // EasyWorship 6 .db files are whole multi-song libraries; route to the
    // dedicated import_easyworship_db command which returns Vec<Song>.
    if (lower.endsWith(".db")) {
      try {
        const bytesB64 = await readFileAsBase64(files[0]);
        const songs = await invoke("import_easyworship_db", { bytesB64 });
        if (songs && songs.length) {
          let id = 0;
          for (const song of songs) for (const slide of song.slides) slide.id = id++;
          await loadSetlist(JSON.stringify({ setlist: songs }));
          return;
        }
        showImportError("EasyWorship database contained no usable songs.");
      } catch (err) {
        showImportError("EasyWorship import failed: " + err);
      }
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

  if (lower.endsWith(".pptx") || lower.endsWith(".pdf") || lower.endsWith(".pro7")) {
    const bytesB64 = await readFileAsBase64(file);
    let fmt;
    if (lower.endsWith(".pdf")) fmt = "pdf";
    else if (lower.endsWith(".pro7")) fmt = "pro7";
    else fmt = "pptx";
    return invoke("parse_song_bytes", {
      bytesB64,
      format: fmt,
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
    songOffsets = [];
    let title = "";
    for (const song of songs) {
      title = song.title;
      songOffsets.push(slides.length);
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
    renderSongStrip();
    updateSlideDisplay();
    // Persist a snapshot so a restart mid-service can pick up where we
    // were — file paths or Planning Center sessions may not be reachable
    // by then. Skip empty setlists.
    if (songs.length > 0) saveRecentSetlist(songs);
  } catch (err) {
    console.error("Failed to load setlist:", err);
    showImportError("Failed to load setlist: " + err);
  }
}

// --- Recent setlists ---
const RECENT_KEY = "voxxa.recentSetlists";
const RECENT_LIMIT = 8;
// Hard cap total stored payload at ~1 MB to keep localStorage from getting
// fat on big Planning Center plans.
const RECENT_MAX_BYTES = 1_000_000;
const recentSection = document.getElementById("recent-section");
const recentList = document.getElementById("recent-list");

function saveRecentSetlist(songs) {
  try {
    const summary = songs.length === 1
      ? songs[0].title
      : `${songs.length} songs · ${songs[0].title}…`;
    const entry = {
      summary,
      saved_at: Date.now(),
      setlist: songs,
    };
    const stored = readJsonStorage(RECENT_KEY) || [];
    // Dedupe by summary so re-loading the same setlist updates the timestamp.
    const filtered = stored.filter((e) => e.summary !== summary);
    filtered.unshift(entry);
    // Trim by count, then by total bytes if still too large.
    let trimmed = filtered.slice(0, RECENT_LIMIT);
    while (trimmed.length > 1) {
      const size = JSON.stringify(trimmed).length;
      if (size <= RECENT_MAX_BYTES) break;
      trimmed.pop();
    }
    writeJsonStorage(RECENT_KEY, trimmed);
    renderRecentSetlists();
  } catch (err) {
    console.warn("saveRecentSetlist:", err);
  }
}

function renderRecentSetlists() {
  if (!recentList || !recentSection) return;
  const stored = readJsonStorage(RECENT_KEY) || [];
  if (!stored.length) {
    recentSection.hidden = true;
    return;
  }
  recentSection.hidden = false;
  recentList.innerHTML = "";
  for (const entry of stored) {
    const row = document.createElement("button");
    row.className = "recent-row";
    const when = new Date(entry.saved_at).toLocaleString();
    row.innerHTML =
      '<div class="recent-row-title">' + escapeHtml(entry.summary) + "</div>" +
      '<div class="recent-row-when">' + escapeHtml(when) + "</div>";
    row.addEventListener("click", async () => {
      await loadSetlist(JSON.stringify({ setlist: entry.setlist }));
    });
    recentList.appendChild(row);
  }
}

renderRecentSetlists();

// Song-jump strip: one button per song in the loaded setlist. Hidden for
// single-song setlists since there's nothing to navigate.
const songStripEl = document.getElementById("song-strip");
let songOffsets = [];

function renderSongStrip() {
  if (!songStripEl) return;
  songStripEl.innerHTML = "";
  if (currentSongs.length <= 1) {
    songStripEl.hidden = true;
    return;
  }
  songStripEl.hidden = false;
  currentSongs.forEach((song, idx) => {
    const btn = document.createElement("button");
    btn.className = "song-chip";
    btn.textContent = song.title || `Song ${idx + 1}`;
    btn.title = `Jump to ${song.title}`;
    btn.dataset.songIdx = String(idx);
    btn.addEventListener("click", () => jumpToSong(idx));
    songStripEl.appendChild(btn);
  });
  highlightActiveSong();
}

function highlightActiveSong() {
  if (!songStripEl) return;
  // Active song = the song whose offset range contains currentSlideIndex.
  let active = 0;
  for (let i = 0; i < songOffsets.length; i++) {
    if (songOffsets[i] <= currentSlideIndex) active = i;
    else break;
  }
  Array.from(songStripEl.children).forEach((el, i) => {
    el.classList.toggle("active", i === active);
  });
}

async function jumpToSong(songIndex) {
  try {
    await invoke("jump_to_song", { songIndex });
    // Local state will sync via the slide-advanced event the dispatcher emits.
  } catch (err) {
    console.error("jump_to_song:", err);
    toast("Failed to jump: " + err, "error");
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
  highlightActiveSong();
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
    toast("Failed to save edit: " + err, "error");
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
    toast(String(err), "error", 6000);
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
    toast("Next slide failed: " + err, "error");
  }
});

if (blankBtn) {
  blankBtn.addEventListener("click", async () => {
    try {
      await invoke("blank_manual");
    } catch (err) {
      console.error(err);
      toast("Blank failed: " + err, "error");
    }
  });
}

const helpBtn = document.getElementById("help-btn");
if (helpBtn) helpBtn.addEventListener("click", showShortcutHelp);

const stageBtn = document.getElementById("stage-btn");
if (stageBtn) {
  stageBtn.addEventListener("click", async () => {
    try {
      const visible = await invoke("toggle_stage_display");
      stageBtn.classList.toggle("active", visible);
    } catch (err) {
      console.error("toggle_stage_display:", err);
      toast("Stage Display failed: " + err, "error");
    }
  });
}

// --- Events ---
const micMeterFill = document.getElementById("mic-meter-fill");
const micMeterPeak = document.getElementById("mic-meter-peak");
let micPeakHold = 0;
let micPeakDecayTimer = null;

const detectSection = document.getElementById("detect-section");
const detectRows = document.getElementById("detect-rows");

function renderDetection(rows) {
  if (!detectSection || !detectRows) return;
  // Show top 3 only — the conductor sends 5; the long tail is noise.
  const top = rows.slice(0, 3);
  if (!top.length) {
    detectSection.hidden = true;
    return;
  }
  detectSection.hidden = false;
  detectRows.innerHTML = "";
  for (const r of top) {
    const row = document.createElement("div");
    row.className = "detect-row";
    const pct = Math.round((r.probability || 0) * 100);
    row.innerHTML =
      '<div class="detect-name">' + escapeHtml(r.song_title || "(untitled)") + "</div>" +
      '<div class="detect-bar"><div class="detect-bar-fill" style="width: ' + pct + '%"></div></div>' +
      '<div class="detect-pct">' + pct + "%</div>";
    detectRows.appendChild(row);
  }
}

async function setupListeners() {
  await listen("transcription", (event) => {
    const { text } = event.payload;
    if (text) {
      heardText.textContent = text;
    }
  });

  await listen("song-detection", (event) => {
    renderDetection(event.payload.rows || []);
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
      refreshHttpApiStatus();
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

      const statusText = model.loaded
        ? "In use"
        : (model.downloaded ? "Downloaded" : "Not downloaded");
      const info = document.createElement("div");
      info.innerHTML = `
        <div class="model-name">${escapeHtml(model.display_name)}</div>
        <div class="model-status">${statusText}</div>
        <div class="progress-bar" id="progress-${escapeHtml(model.name)}" hidden>
          <div class="fill" style="width: 0%"></div>
        </div>
      `;

      const btn = document.createElement("button");
      // Three states: download (not downloaded), use (downloaded but not loaded),
      // in-use (loaded). Each maps to a different label + action.
      if (model.loaded) {
        btn.className = "download-btn downloaded";
        btn.textContent = "In use";
        btn.disabled = true;
      } else if (model.downloaded) {
        btn.className = "download-btn use";
        btn.textContent = "Use";
        btn.addEventListener("click", () => loadModelByName(model.name));
      } else {
        btn.className = "download-btn";
        btn.textContent = "Download";
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

async function loadModelByName(modelName) {
  try {
    await invoke("load_model", { modelName });
    await loadModels();
    writeJsonStorage("voxxa.lastModel", modelName);
  } catch (err) {
    console.error("load_model:", err);
    toast("Failed to load model: " + err, "error");
  }
}

async function saveLanguageChoice() {
  const sel = document.getElementById("select-language");
  if (!sel) return;
  const language = sel.value || null;
  try {
    await invoke("set_language", { language });
    writeJsonStorage("voxxa.language", sel.value);
  } catch (err) {
    console.error("set_language:", err);
  }
}

// On startup, re-apply the last language choice. Auto-reload of the model
// is intentionally NOT done — model files can be large and reloading them
// every launch is expensive; the user clicks Use once and then Voxxa
// remembers it for the next session via writeJsonStorage above.
async function restoreLastSession() {
  const lang = readJsonStorage("voxxa.language");
  if (typeof lang === "string") {
    try { await invoke("set_language", { language: lang || null }); } catch {}
  }
  const lastModel = readJsonStorage("voxxa.lastModel");
  if (typeof lastModel === "string" && lastModel) {
    // Try to load — silently no-op if the file was deleted.
    try { await invoke("load_model", { modelName: lastModel }); } catch (e) {
      console.warn("could not restore last model:", e);
    }
  }
}

let currentDownloadModel = null;

async function downloadModel(modelName) {
  currentDownloadModel = modelName;
  const bar = document.getElementById(`progress-${modelName}`);
  if (bar) bar.hidden = false;
  try {
    await invoke("download_model", { modelName });
    // Backend auto-loads the model on first download; reflect that here.
    writeJsonStorage("voxxa.lastModel", modelName);
    loadModels();
  } catch (err) {
    console.error("Download failed:", err);
    toast("Download failed: " + err, "error");
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
    const [devices, models, settings, currentLang] = await Promise.all([
      invoke("list_audio_devices"),
      invoke("get_model_status"),
      invoke("get_settings"),
      invoke("get_language"),
    ]);

    const deviceSelect = document.getElementById("select-device");
    deviceSelect.innerHTML = '<option value="">Default</option>';
    const savedDevice = readJsonStorage("voxxa.audioDevice") || "";
    devices.forEach((d) => {
      const opt = document.createElement("option");
      opt.value = d;
      opt.textContent = d;
      if (savedDevice === d || (!savedDevice && settings.device === d)) {
        opt.selected = true;
      }
      deviceSelect.appendChild(opt);
    });
    // Apply the saved choice on every Settings tab open so a freshly-attached
    // mic (which only appears in the list after enumerating) gets picked up.
    if (savedDevice && devices.includes(savedDevice)) {
      try { await invoke("select_audio_device", { device: savedDevice }); } catch {}
    }

    // Active model is display-only — the user picks one from the Models tab.
    const modelSelect = document.getElementById("select-model");
    modelSelect.innerHTML = "";
    const loaded = models.find((m) => m.loaded);
    if (loaded) {
      const opt = document.createElement("option");
      opt.value = loaded.name;
      opt.textContent = loaded.display_name;
      opt.selected = true;
      modelSelect.appendChild(opt);
    } else {
      const opt = document.createElement("option");
      opt.textContent = models.some((m) => m.downloaded)
        ? "None loaded — pick one in the Models tab"
        : "No models downloaded";
      opt.disabled = true;
      modelSelect.appendChild(opt);
    }

    const langSelect = document.getElementById("select-language");
    if (langSelect) {
      langSelect.value = currentLang || "";
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

// --- Toast notifications ---
// Mid-service `alert()` blocks the UI thread and pulls focus; for operator
// workflows that's unacceptable. Toast notifications stack non-blocking.
function ensureToastContainer() {
  let c = document.getElementById("toast-container");
  if (!c) {
    c = document.createElement("div");
    c.id = "toast-container";
    document.body.appendChild(c);
  }
  return c;
}

function toast(message, kind = "info", timeoutMs = 4000) {
  const container = ensureToastContainer();
  const el = document.createElement("div");
  el.className = `toast toast-${kind}`;
  el.textContent = String(message);
  // Click to dismiss early.
  el.addEventListener("click", () => dismissToast(el));
  container.appendChild(el);
  if (timeoutMs > 0) {
    setTimeout(() => dismissToast(el), timeoutMs);
  }
  return el;
}

function dismissToast(el) {
  if (!el || el.classList.contains("fading")) return;
  el.classList.add("fading");
  setTimeout(() => el.remove(), 300);
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

const langSelectOnce = document.getElementById("select-language");
if (langSelectOnce) langSelectOnce.addEventListener("change", saveLanguageChoice);

const deviceSelectOnce = document.getElementById("select-device");
if (deviceSelectOnce) {
  deviceSelectOnce.addEventListener("change", async () => {
    const device = deviceSelectOnce.value || null;
    try {
      await invoke("select_audio_device", { device });
      writeJsonStorage("voxxa.audioDevice", deviceSelectOnce.value);
    } catch (err) {
      console.error("select_audio_device:", err);
    }
  });
}

// --- Local HTTP API toggle ---
const httpApiToggle = document.getElementById("http-api-toggle");
const httpApiDetail = document.getElementById("http-api-detail");
const httpApiPort = document.getElementById("http-api-port");
const httpApiToken = document.getElementById("http-api-token");
const httpApiStatus = document.getElementById("http-api-status");
const HTTP_API_STORAGE_KEY = "voxxa.httpApi";

async function refreshHttpApiStatus() {
  if (!httpApiStatus) return;
  try {
    const s = await invoke("get_http_api_status");
    httpApiStatus.textContent = s.running
      ? `Listening on http://127.0.0.1:${s.port}`
      : "Off";
    httpApiStatus.classList.toggle("ok", s.running);
    httpApiToggle.checked = s.running;
    httpApiDetail.hidden = !s.running && !httpApiToggle.checked;
  } catch (err) {
    console.error("get_http_api_status:", err);
  }
}

if (httpApiToggle) {
  httpApiToggle.addEventListener("change", async () => {
    httpApiDetail.hidden = !httpApiToggle.checked;
    if (httpApiToggle.checked) {
      const port = parseInt(httpApiPort.value, 10) || 7575;
      const token = httpApiToken.value || null;
      try {
        await invoke("start_http_api", { port, token });
        writeJsonStorage(HTTP_API_STORAGE_KEY, { port, enabled: true });
      } catch (err) {
        console.error("start_http_api:", err);
        httpApiStatus.textContent = "Failed: " + err;
        httpApiStatus.classList.add("err");
        httpApiToggle.checked = false;
        httpApiDetail.hidden = true;
      }
    } else {
      try {
        await invoke("stop_http_api");
        writeJsonStorage(HTTP_API_STORAGE_KEY, { port: parseInt(httpApiPort.value, 10) || 7575, enabled: false });
      } catch (err) {
        console.error("stop_http_api:", err);
      }
    }
    await refreshHttpApiStatus();
  });
}

// Restore non-secret HTTP API config — bearer token is intentionally not
// persisted, same policy as presenter passwords.
(function restoreHttpApi() {
  const saved = readJsonStorage(HTTP_API_STORAGE_KEY);
  if (saved && saved.port && httpApiPort) httpApiPort.value = saved.port;
})();

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
      toast("Failed to generate report: " + err, "error");
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

// --- Keyboard shortcuts ---
// These fire only when the main window has focus and no text input is active.
// Mid-service operators tend to keep their hands on the keyboard; the plan §6.3
// calls out the need for a panic-friendly control surface.
document.addEventListener("keydown", (e) => {
  // Don't intercept when typing in a text field, contenteditable, or modal.
  const target = e.target;
  const tag = target && target.tagName ? target.tagName.toLowerCase() : "";
  if (tag === "input" || tag === "textarea" || tag === "select") return;
  if (target && target.isContentEditable) return;
  // Modifier-key combos belong to the OS / Tauri menus.
  if (e.metaKey || e.ctrlKey || e.altKey) return;

  switch (e.key) {
    case " ": // Space toggles listening
      e.preventDefault();
      toggleListening();
      break;
    case "ArrowRight":
    case "n":
    case "N":
      e.preventDefault();
      nextBtn?.click();
      break;
    case "ArrowLeft":
    case "p":
    case "P":
      e.preventDefault();
      prevBtn?.click();
      break;
    case "b":
    case "B":
    case ".":
      e.preventDefault();
      blankBtn?.click();
      break;
    case "s":
    case "S":
      e.preventDefault();
      stageBtn?.click();
      break;
    case "?":
      e.preventDefault();
      showShortcutHelp();
      break;
  }
});

function showShortcutHelp() {
  const lines = [
    "Space — Start / stop listening",
    "← / P — Previous slide",
    "→ / N — Next slide",
    ".  / B — Blank output",
    "S — Toggle Stage Display",
    "?  — Show this help",
  ];
  // Long-timeout info toast — stays visible long enough to read but doesn't
  // pull focus the way alert() would.
  toast("Shortcuts: " + lines.join("  ·  "), "info", 10000);
}

// --- Init ---
setupListeners();
loadPresenterPanel();
restoreLastSession();
maybeShowWelcome();
