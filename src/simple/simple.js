// Simple mode (ADR-017): the default view, in front of the existing panels (#app,
// "Advanced"). It reads existing commands and events only and starts or stops
// listening through the existing Listen button, so main.js stays in step.
import "./simple.css";
import { statusLight, songPosition } from "./logic.js";
import { openFirstRun, maybeFirstRun, applySavedDevice } from "./firstrun.js";

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);
const advanced = $("app");

const root = document.createElement("main");
root.id = "simple";
root.innerHTML = `
  <h1 class="logo">Voxxa</h1>
  <div class="s-switch" role="radiogroup" aria-label="Mode">
    <button role="radio" data-mode="off">Off</button>
    <button role="radio" data-mode="songs">Songs</button>
    <button role="radio" data-mode="scripture" disabled title="Scripture comes in a later update">Songs + Scripture</button>
  </div>
  <div class="s-status" role="status" aria-live="polite"><span class="s-light" id="s-light"></span><span id="s-sentence"></span></div>
  <div class="s-now"><div id="s-song"></div><div id="s-slide"></div></div>
  <button class="s-help" id="s-help">Call for help</button>
  <p class="s-help-msg" id="s-help-msg" hidden>Call for help comes in the next update. Until then, call your tech lead and read them the sentence next to the light.</p>
  <button class="s-link" id="s-advanced">Advanced</button>`;
document.body.insertBefore(root, advanced);
advanced.querySelector(".header").insertAdjacentHTML("beforeend",
  '<span class="s-back"><button class="s-link" id="s-guide">Set-up guide</button><button class="s-link" id="s-simple">Simple mode</button></span>');

const show = (adv) => { advanced.hidden = !adv; root.hidden = adv; };
$("s-advanced").onclick = () => show(true);
$("s-simple").onclick = () => show(false);
$("s-guide").onclick = openFirstRun;
$("s-help").onclick = () => { $("s-help-msg").hidden = false; };

// Everything the light needs (ADR-017 § 2), from existing commands and events.
const s = { running: false, setlist: false, model: false, state: null, isBlank: false, presenter: null, presenterError: null, micLost: false, song: null };
let songs = [];

function render() {
  const [light, sentence] = statusLight(s);
  $("s-light").className = `s-light ${light}`;
  root.dataset.light = light;
  $("s-sentence").textContent = sentence;
  for (const b of root.querySelectorAll("[data-mode]")) b.setAttribute("aria-checked", String(b.dataset.mode === (s.running ? "songs" : "off")));
  const p = s.song;
  $("s-song").textContent = !p ? "No song on screen yet" : p.songs ? `Song ${p.song} of ${p.songs} · ${p.title}` : p.title;
  $("s-slide").textContent = p && p.slides ? `Slide ${p.slide} of ${p.slides}` : "";
}

let polling = false;
async function refresh() {
  if (polling) return;
  polling = true;
  try {
    const [st, presenter] = await Promise.all([invoke("get_status"), invoke("get_presenter_info")]);
    if (st.is_running && !s.running) s.micLost = s.presenterError = false; // a new session
    Object.assign(s, { running: st.is_running, setlist: st.total_slides > 0, model: st.model_loaded, state: st.machine_state, isBlank: !!st.is_blank, presenter });
    s.song = songPosition(songs, st.current_slide, st.song_title);
  } catch (err) {
    console.error("simple mode status:", err);
  } finally {
    polling = false;
  }
  render();
}

// Off stops and Songs starts listening through the existing Listen button.
root.querySelector(".s-switch").addEventListener("click", (e) => {
  const mode = e.target.dataset && e.target.dataset.mode;
  if (!mode || e.target.disabled || (mode === "songs") === s.running) return;
  if (mode === "songs") s.micLost = s.presenterError = false;
  $("listen-btn").click();
  setTimeout(refresh, 400);
});

listen("machine-state", (e) => { s.state = e.payload.state; s.isBlank = e.payload.is_blank; render(); });
listen("listening-stopped", (e) => { if (e.payload && e.payload.disconnected) s.micLost = true; refresh(); });
listen("presenter-error", () => { s.presenterError = true; render(); });
listen("slide-advanced", () => { s.presenterError = false; refresh(); });
document.addEventListener("voxxa-setlist", (e) => { songs = e.detail || []; refresh(); });

render();
refresh();
setInterval(refresh, 1000);
applySavedDevice();
maybeFirstRun();
