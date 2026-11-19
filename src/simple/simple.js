// Simple mode (ADR-017): the default view, in front of the existing panels (#app,
// "Advanced"). It reads existing commands and events only and starts or stops
// listening through the existing Listen button, so main.js stays in step.
import "./simple.css";
import { statusLight, songPosition } from "./logic.js";
import { openFirstRun, maybeFirstRun, applySavedDevice, reconnectPresenter, savedPresenter } from "./firstrun.js";

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
// Space or Enter on Simple mode's own buttons presses the button, not main.js's shortcut.
root.addEventListener("keydown", (e) => { if (e.target.tagName === "BUTTON" && (e.key === " " || e.key === "Enter")) e.stopPropagation(); });

// Everything the light needs (ADR-017 § 2), from existing commands and events.
const s = { running: false, setlist: false, model: false, state: null, isBlank: false, presenter: null, presenterError: null, micLost: false, song: null };
let songs = [];
// One toggle at a time: `pending` is the running state asked for until get_status shows
// it (or 3 polls pass); a click meanwhile is kept in `want` and sent once it settled.
let pending = null, want = null, polls = 0;

// Write text only when it changes: the status is aria-live and render() runs every second,
// so rewriting the same sentence could make a screen reader announce it again.
const put = (el, t) => { if (el.textContent !== t) el.textContent = t; };

function render() {
  const [light, sentence] = statusLight(s);
  $("s-light").className = `s-light ${light}`;
  root.dataset.light = light;
  put($("s-sentence"), sentence);
  const on = want ?? pending ?? s.running; // the user's choice shows until the backend settles
  for (const b of root.querySelectorAll("[data-mode]")) b.setAttribute("aria-checked", String(b.dataset.mode === (on ? "songs" : "off")));
  const p = s.song;
  put($("s-song"), !p ? "No song on screen yet" : p.songs ? `Song ${p.song} of ${p.songs} · ${p.title}` : p.title);
  put($("s-slide"), p && p.slides ? `Slide ${p.slide} of ${p.slides}` : "");
}

let polling = false;
async function refresh() {
  if (polling) return;
  polling = true;
  try {
    const [st, presenter] = await Promise.all([invoke("get_status"), invoke("get_presenter_info")]);
    if (st.is_running && !s.running) s.micLost = s.presenterError = false; // a new session
    const chosen = (savedPresenter() || {}).kind || null; // connected means: to the app last chosen
    Object.assign(s, { running: st.is_running, setlist: st.total_slides > 0, model: st.model_loaded, state: st.machine_state, isBlank: !!st.is_blank, presenter, chosen });
    s.song = songPosition(songs, st.current_slide, st.song_title);
    // Settled when the backend and main.js's Listen button (its "active" class) both show it.
    if (pending !== null && st.is_running === pending && $("listen-btn").classList.contains("active") === pending) {
      const next = want;
      pending = want = null;
      if (next !== null && next !== st.is_running) toggle(next); // the click made while it settled
    } else if (pending !== null && ++polls >= 3) pending = want = null; // refused or slow: show the real state
  } catch (err) {
    console.error("simple mode status:", err);
  } finally {
    polling = false;
  }
  render();
}

// Off stops and Songs starts listening through the existing Listen button.
function toggle(on) {
  pending = on;
  polls = 0;
  if (on) s.micLost = s.presenterError = false;
  $("listen-btn").click();
  render();
  setTimeout(refresh, 400);
}
root.querySelector(".s-switch").addEventListener("click", (e) => {
  const mode = e.target.dataset && e.target.dataset.mode;
  if (!mode || e.target.disabled) return;
  if (pending !== null) { want = mode === "songs"; return render(); } // no second toggle in flight
  if ((mode === "songs") !== s.running) toggle(mode === "songs");
});

listen("machine-state", (e) => { s.state = e.payload.state; s.isBlank = e.payload.is_blank; render(); });
listen("listening-stopped", (e) => { if (e.payload && e.payload.disconnected) s.micLost = true; refresh(); });
listen("presenter-error", (e) => { s.presenterError = e.payload ? String(e.payload) : true; render(); }); // the reason names titles only
listen("slide-advanced", () => { s.presenterError = false; refresh(); });
document.addEventListener("voxxa-setlist", (e) => { songs = e.detail || []; refresh(); });

render();
refresh();
setInterval(refresh, 1000);
applySavedDevice();
reconnectPresenter();
maybeFirstRun();
