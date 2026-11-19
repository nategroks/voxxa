// Simple mode (ADR-010 Decision 4, ADR-017): the default view for a volunteer.
// Three things (mode switch, status light, Call for help), the current song and
// slide, and one link to the Advanced view, which holds every existing panel.
// It reads the commands and events the panels already use; no slide text is
// shown here, and this is the operator window only (ADR-011).
import {
  invoke, listen, isListening, toggleListening, currentSongs, songOffsets,
  toast, readJsonStorage, writeJsonStorage,
} from "../main.js";
import { light } from "./status.js";
import "./simple.css";

// The same localStorage keys the Advanced panels use, so both stay in step.
const FIRST_RUN_KEY = "voxxa.firstLaunchDone"; // the old welcome's key: whoever dismissed it is not asked again
const DEVICE_KEY = "voxxa.audioDevice";
const PRESENTER_KEY = "voxxa.presenterForm";

const s = { machine: null, micLost: false, startError: "", presenterError: "", connected: true, song: "" };
const app = document.getElementById("app");
const view = document.createElement("main");
view.id = "simple";
view.innerHTML = `
  <header class="header"><h1 class="logo">Voxxa</h1></header>
  <div class="mode" role="radiogroup" aria-label="Mode">
    <label><input type="radio" name="mode" value="off" checked><span>Off</span></label>
    <label><input type="radio" name="mode" value="songs"><span>Songs</span></label>
    <label title="Coming in a later update"><input type="radio" name="mode" value="scripture" disabled><span>Songs + Scripture</span></label>
  </div>
  <div class="light grey" role="status"><span class="dot"></span><span class="sentence"></span></div>
  <p class="now"></p>
  <button class="primary-btn help">Call for help</button>
  <button class="link-btn more">Advanced…</button>`;
document.body.prepend(view);
const button = (className, textContent, onclick) => Object.assign(document.createElement("button"), { className, textContent, onclick });
const say = (el, text) => (el.querySelector("p").textContent = text);

// --- Status light, mode switch, current song ---
let was = false;
let busy = false;
let shownSongs = currentSongs;
function render() {
  if (isListening && !was) newSession().finally(render); // Songs, Space, the tray or the Advanced button
  was = isListening;
  if (currentSongs !== shownSongs) [s.song, shownSongs] = ["", currentSongs]; // a new setlist: no song yet
  const [color, text] = light({ ...s, listening: isListening });
  view.querySelector(".light").className = `light ${color}`;
  const songs = currentSongs.length ? `${currentSongs.length} song${currentSongs.length === 1 ? "" : "s"} loaded` : "No songs loaded yet";
  // Write only changes, so a screen reader announces the status line once.
  const put = (el, t) => el.textContent !== t && (el.textContent = t);
  put(view.querySelector(".sentence"), text);
  document.querySelectorAll(".now").forEach((el) => put(el, s.song || songs));
  if (!busy) view.querySelector(`[value=${isListening ? "songs" : "off"}]`).checked = true;
  return color;
}

async function newSession() {
  Object.assign(s, { micLost: false, startError: "", presenterError: "" });
  s.machine = (await invoke("get_status")).machine_state;
  await presenterInfo();
}

// Connected to the app last chosen; after a failed reconnect the keystroke default is not it.
async function presenterInfo() {
  const p = await invoke("get_presenter_info");
  const chosen = readJsonStorage(PRESENTER_KEY);
  s.connected = p.connected && (!chosen || chosen.kind === p.kind);
}

// "Song 2: <title> · slide 3/6" from the global slide index the backend sends.
function songLine(i) {
  const n = songOffsets.findLastIndex((o) => o <= i);
  const song = currentSongs[n];
  return song ? `Song ${n + 1}: ${song.title} · slide ${i - songOffsets[n] + 1}/${song.slides.length}` : "";
}

// "click", not "change": choosing Off again after a refused start clears its red.
view.querySelector(".mode").addEventListener("click", async (e) => {
  if (!e.target.matches("input")) return;
  const want = e.target.value === "songs";
  busy = true;
  s.startError = want === isListening ? "" : (await toggleListening()) || "";
  busy = false;
  render();
});

listen("machine-state", (e) => { s.machine = e.payload.state; render(); });
listen("slide-advanced", (e) => { s.presenterError = ""; [s.song, shownSongs] = [songLine(e.payload.slide_index), currentSongs]; render(); });
listen("presenter-error", (e) => { s.presenterError = String(e.payload); render(); });
listen("listening-stopped", (e) => { s.micLost = !!e.payload?.disconnected; render(); });
setInterval(render, 500);

// Q9 wires this to the support bundle. The diagnostic report is not offered
// here yet: it carries what the GUI heard (Q7-08, #102).
view.querySelector(".help").onclick = () =>
  toast("Call for help arrives in the next update. Until then, call the person who set up Voxxa for your church.", "info", 10000);

// --- Advanced: every existing panel, one click away ---
const advanced = (on) => {
  app.hidden = !on;
  view.hidden = on;
  if (!on) presenterInfo().finally(render); // Settings may have connected another app
};
view.querySelector(".more").onclick = () => advanced(true);
document.querySelector("#app .header").append(button("link-btn", "← Simple", () => advanced(false)));
document.getElementById("section-settings").append(button("secondary-btn", "Run first-time setup again", () => setup(0)));

// --- First-run flow: device → presentation app → model → 20-second test ---
const overlay = Object.assign(document.createElement("div"), { className: "welcome-overlay", hidden: true });
document.body.append(overlay);
const steps = [chooseDevice, findPresenter, getModel, testRun];

async function setup(i) {
  overlay.hidden = false;
  overlay.innerHTML = `<div class="welcome-card">
    <h2 class="welcome-title">Set up Voxxa</h2>
    <div class="welcome-tag">Step ${i + 1} of ${steps.length}</div>
    <div class="step"><p class="welcome-tag"></p></div>
    <div class="welcome-footer"><button class="welcome-dismiss">Skip setup</button><button class="primary-btn">${i < steps.length - 1 ? "Next" : "Finish"}</button></div>
  </div>`;
  const [skip, next] = overlay.querySelectorAll(".welcome-footer button");
  skip.onclick = done;
  next.onclick = () => (i < steps.length - 1 ? setup(i + 1) : done());
  const el = overlay.querySelector(".step");
  try {
    await steps[i](el);
  } catch (err) {
    say(el, `Something went wrong: ${err}. You can skip this step and fix it later under Advanced.`);
  }
}

// Skipping counts as done too: the flow is re-opened from Advanced → Settings.
function done() {
  overlay.hidden = true;
  writeJsonStorage(FIRST_RUN_KEY, 1);
}

async function chooseDevice(el) {
  say(el, "Which input should Voxxa listen to? Pick the sound desk if it is plugged in.");
  el.insertAdjacentHTML("beforeend", `<div class="setting-group"><select aria-label="Audio input"></select></div>`);
  const sel = el.querySelector("select");
  const saved = readJsonStorage(DEVICE_KEY) || "";
  for (const d of ["", ...(await invoke("list_audio_devices"))]) sel.add(new Option(d || "Default", d, false, d === saved));
  sel.onchange = async () => {
    await invoke("select_audio_device", { device: sel.value || null });
    writeJsonStorage(DEVICE_KEY, sel.value);
  };
}

async function findPresenter(el) {
  say(el, "Looking for ProPresenter, FreeShow, OpenLP or OpenSong…");
  const [svc] = await invoke("discover_presenters", { timeoutMs: 2500 });
  if (!svc) {
    return say(el, "No presentation app found, so Voxxa will press the arrow keys for you: click into your presentation app before you switch to Songs. To connect one later, open Advanced → Settings.");
  }
  const args = { kind: svc.kind, host: svc.host, port: svc.port };
  await invoke("connect_presenter", { args });
  writeJsonStorage(PRESENTER_KEY, args);
  await presenterInfo();
  say(el, `Connected to ${svc.name}.`);
}

async function getModel(el) {
  say(el, "Checking the speech model…");
  const models = await invoke("get_model_status");
  const have = models.find((m) => m.loaded) || models.find((m) => m.downloaded);
  if (have) {
    if (!have.loaded) await invoke("load_model", { modelName: have.name });
    writeJsonStorage("voxxa.lastModel", have.name);
    return say(el, `Speech model ready: ${have.display_name}.`);
  }
  // shortcut: the download is not checked against a hash (#9); fix that in the backend command.
  const base = models.find((m) => m.name === "Base");
  say(el, `Voxxa needs a speech model, ${base.display_name}. It downloads once, then works offline.`);
  el.insertAdjacentHTML("beforeend", `<button class="primary-btn">Download</button><div class="progress-bar" hidden><div class="fill"></div></div>`);
  const btn = el.querySelector("button");
  const bar = el.querySelector(".progress-bar");
  btn.onclick = async () => {
    btn.disabled = true;
    bar.hidden = false;
    const stop = await listen("download-progress", (e) => (bar.firstChild.style.width = `${e.payload.percent}%`));
    try {
      await invoke("download_model", { modelName: base.name });
      writeJsonStorage("voxxa.lastModel", base.name);
      say(el, "Speech model ready.");
      btn.hidden = true;
    } catch (err) {
      say(el, `Download failed: ${err}`);
      btn.disabled = false;
    }
    stop();
  };
}

async function testRun(el) {
  say(el, "Last step: load this Sunday's songs, start the test, and play or sing 20 seconds of one of them. Voxxa should turn green.");
  el.insertAdjacentHTML("beforeend", `<p class="now"></p><div class="connect-row"></div>`);
  const row = el.querySelector(".connect-row");
  row.append(button("secondary-btn", "Choose songs", () => document.getElementById("file-input").click()));
  row.append(button("primary-btn", "Start the 20-second test", async () => {
    const buttons = overlay.querySelectorAll("button");
    buttons.forEach((b) => (b.disabled = true));
    s.startError = isListening ? "" : (await toggleListening()) || "";
    let green = false;
    for (let left = 20; left > 0 && isListening; left--) {
      say(el, `Listening… ${left} s left.`);
      await new Promise((r) => setTimeout(r, 1000));
      green = render() === "green" || green;
    }
    const text = light({ ...s, listening: isListening })[1];
    if (isListening) await toggleListening();
    say(el, green ? "Green: Voxxa followed your song. You are ready." : `Not green yet. ${text}`);
    buttons.forEach((b) => (b.disabled = false));
    render();
  }));
}

// --- Start-up: re-apply what the first run (or Advanced) chose ---
async function restore() {
  const device = readJsonStorage(DEVICE_KEY);
  if (device && (await invoke("list_audio_devices")).includes(device)) await invoke("select_audio_device", { device });
  const form = readJsonStorage(PRESENTER_KEY);
  if (form) await invoke("connect_presenter", { args: form }).catch(console.warn);
  await presenterInfo();
}
restore().catch(console.error).finally(render);
if (!readJsonStorage(FIRST_RUN_KEY)) setup(0);
