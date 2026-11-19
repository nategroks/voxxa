// First-run flow (ADR-017 § 4): microphone -> presentation app -> voice model ->
// 20-second test. It replaces the welcome overlay and keeps its localStorage key,
// so an operator who dismissed the welcome is not asked again. Existing commands only.
import { testVerdict } from "./logic.js";

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const DONE_KEY = "voxxa.firstLaunchDone";
const DEFAULT_MODEL = "Tiny"; // the model the eval measures (ADR-015, ADR-016)
const STEPS = [
  ["device", "Which microphone?"],
  ["presenter", "Find your presentation app"],
  ["model", "Get the voice model"],
  ["test", "20-second test"],
];
const $ = (id) => document.getElementById(id);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const read = (k) => { try { return JSON.parse(localStorage.getItem(k)); } catch { return null; } };
const write = (k, v) => { try { localStorage.setItem(k, JSON.stringify(v)); } catch {} };

const box = document.createElement("div");
box.id = "firstrun";
box.className = "fr";
box.hidden = true;
box.innerHTML = `<div class="fr-card" role="dialog" aria-modal="true" aria-labelledby="fr-title">
  <div class="fr-step" id="fr-step"></div><h2 id="fr-title"></h2>
  <p id="fr-msg"></p><div class="fr-extra" id="fr-extra"></div>
  <div class="fr-actions"><button class="s-link" id="fr-skip">Skip setup</button><button class="primary-btn" id="fr-next">Next</button></div>
</div>`;
document.body.appendChild(box);
box.addEventListener("keydown", (e) => e.stopPropagation()); // main.js's shortcuts (Space = Listen) stay out of the guide

let step = -1;
let onNext = () => {};
const say = (text) => { $("fr-msg").textContent = text; };
function button(label, onclick, cls = "secondary-btn") {
  const b = document.createElement("button");
  b.className = cls;
  b.textContent = label;
  b.onclick = onclick;
  $("fr-extra").appendChild(b);
  return b;
}

export function openFirstRun() { box.hidden = false; go(0); }
export function maybeFirstRun() { if (read(DONE_KEY) !== 1) openFirstRun(); } // stored as "1"
function finish() { write(DONE_KEY, 1); box.hidden = true; step = -1; }
$("fr-skip").onclick = finish;
$("fr-next").onclick = () => onNext();

function go(i) {
  step = i;
  $("fr-step").textContent = `Step ${i + 1} of ${STEPS.length}`;
  $("fr-title").textContent = STEPS[i][1];
  $("fr-next").textContent = "Next";
  $("fr-extra").innerHTML = "";
  say("");
  onNext = () => go(i + 1);
  ({ device, presenter, model, test })[STEPS[i][0]]();
}
const still = (i) => step === i && !box.hidden;

// The device chosen here (or in Settings) is applied at every start, not only when Settings opens.
export async function applySavedDevice() {
  const dev = read("voxxa.audioDevice");
  try { if (dev && (await invoke("list_audio_devices")).includes(dev)) await invoke("select_audio_device", { device: dev }); } catch {}
}

// The presentation app chosen here (or in Settings) is reconnected at every start, apart from
// the device restore so a failed device list cannot skip it. Never the keystroke driver (it is
// the default) and never a form that needs a secret: passwords are never saved (main.js:1131-
// 1133), so that reconnect would fail every launch; row 6 of the light shows it instead.
export const savedPresenter = () => read("voxxa.presenterForm");
export function reconnectPresenter() {
  const form = savedPresenter();
  if (!form || !form.kind || form.kind === "keystroke" || /_ws$/.test(form.kind) || form.username) return;
  invoke("connect_presenter", { args: form }).catch(() => {});
}

async function device() {
  say("Pick the input that carries the music, usually the sound desk's USB or line input.");
  const sel = document.createElement("select");
  sel.id = "fr-device";
  sel.add(new Option("Default", ""));
  $("fr-extra").appendChild(sel);
  const saved = read("voxxa.audioDevice") || "";
  let devices = [], failed = null;
  try { devices = await invoke("list_audio_devices"); } catch (err) { failed = err; }
  if (!still(0)) return; // skipped or moved on while the list loaded: keep that step's Next
  if (failed) say("Couldn't list the microphones: " + failed);
  for (const d of devices) sel.add(new Option(d, d, false, d === saved));
  onNext = async () => {
    try { await invoke("select_audio_device", { device: sel.value || null }); } catch {}
    write("voxxa.audioDevice", sel.value);
    go(1);
  };
}

async function presenter() {
  const i = step;
  say("Looking for ProPresenter, FreeShow, OpenLP and OpenSong on this computer and network…");
  let found = [];
  try { found = (await invoke("discover_presenters", { timeoutMs: 2500 })) || []; } catch {}
  if (!still(i)) return;
  if (!found.length) {
    say("No presentation app found. Voxxa will press the arrow keys instead, so keep your presentation app in front during the service.");
    button("Search again", () => go(i));
    return;
  }
  say("Found. Pick yours:");
  for (const svc of found) {
    button(`${svc.name} · ${svc.host}:${svc.port}`, async () => {
      const args = { kind: svc.kind, host: svc.host, port: svc.port };
      try {
        await invoke("connect_presenter", { args });
        write("voxxa.presenterForm", args);
        say(`Connected to ${svc.name}.`);
      } catch (err) { say("Couldn't connect: " + err); }
    }, "discovered-row");
  }
}

async function model() {
  const i = step;
  let models = [];
  try { models = await invoke("get_model_status"); } catch {}
  if (!still(i)) return;
  const loaded = models.find((m) => m.loaded);
  if (loaded) return say(`Voice model ready: ${loaded.display_name}.`);
  const have = models.find((m) => m.downloaded);
  const m = have || models.find((x) => x.name === DEFAULT_MODEL) || { name: DEFAULT_MODEL, display_name: DEFAULT_MODEL };
  say(have ? `${m.display_name} is on this computer.` : `Voxxa needs a voice model, ${m.display_name}. It downloads once and stays on this computer.`);
  const b = button(have ? "Use it" : "Download", async () => {
    b.disabled = true;
    const bar = document.createElement("progress");
    bar.max = 100;
    bar.value = 0;
    $("fr-extra").appendChild(bar);
    const off = await listen("download-progress", (e) => { bar.value = e.payload.percent; });
    try {
      await invoke(have ? "load_model" : "download_model", { modelName: m.name });
      write("voxxa.lastModel", m.name);
      if (still(i)) go(i);
    } catch (err) { say("That didn't work: " + err); b.disabled = false; }
    off();
  });
}

async function test() {
  const i = step;
  $("fr-next").textContent = "Finish anyway";
  onNext = finish;
  let st = {}, p = {};
  try { st = await invoke("get_status"); } catch {}
  try { p = await invoke("get_presenter_info"); } catch {}
  if (!still(i)) return;
  if (!st.total_slides) {
    say("Load this Sunday's songs first. Then play 20 seconds of any song from the list.");
    button("Choose song files", () => $("file-input").click());
    document.addEventListener("voxxa-setlist", () => { if (still(i)) go(i); }, { once: true });
    return;
  }
  if (!st.model_loaded) {
    say("The voice model isn't ready yet.");
    return button("Back to the voice model", () => go(i - 1));
  }
  // Keystrokes are refused while Voxxa's own window has the focus (#44).
  say("Play 20 seconds of any song from the list through the speakers. If Voxxa recognises it, your presentation app moves like it would on Sunday." +
    (p.kind === "keystroke" ? " If Voxxa presses keys for you, click into your presentation app right after you start." : ""));
  button("Start the 20-second test", () => run(i, st.is_running), "primary-btn");
}

async function run(i, wasRunning) {
  $("fr-extra").innerHTML = "";
  const t = { started: wasRunning, lost: false, maxRms: 0, words: 0, advanced: 0, refused: 0 };
  const offs = await Promise.all([
    listen("mic-level", (e) => { t.maxRms = Math.max(t.maxRms, e.payload.rms || 0); }),
    listen("transcription", (e) => { if (e.payload.text) t.words += 1; }),
    listen("listening-stopped", (e) => { if (e.payload && e.payload.disconnected) t.lost = true; }),
    listen("slide-advanced", () => { t.advanced += 1; }), // a slide change the presenter took
    listen("presenter-error", () => { t.refused += 1; }),
  ]);
  // The Listen click has settled when main.js's Listen button turns "active" (started) or
  // main.js shows the refusal as an error toast. Opening a USB or Bluetooth input can take
  // seconds, so "started" is decided only then, and whatever the test started it stops, even
  // a start that ends after the verdict window or after the guide was closed.
  const active = () => $("listen-btn").classList.contains("active");
  let up = false, refusedStart = false;
  const watch = new MutationObserver((ms) => { if (ms.some((m) => [...m.addedNodes].some((n) => n.classList && n.classList.contains("toast-error")))) refusedStart = true; });
  const settle = async (ms) => { for (let w = 0; w < ms && !up && !refusedStart; w += 200) { await sleep(200); up = active(); } };
  if (!wasRunning) {
    watch.observe(document.body, { childList: true, subtree: true });
    $("listen-btn").click();
    say("Starting…");
    await settle(15000);
    t.started = up;
  }
  for (let left = 20; t.started && left > 0 && still(i) && !t.lost; left--) {
    say(`Listening… ${left} s`);
    await sleep(1000);
  }
  if (!wasRunning) {
    await settle(15000); // a start still in flight: wait for it, so it is stopped too
    watch.disconnect();
    if (active()) $("listen-btn").click();
    for (let w = 0; w < 5000 && active(); w += 200) await sleep(200); // stopped before "Try again" shows
  }
  offs.forEach((off) => off());
  if (!still(i)) return;
  const [pass, text] = testVerdict(t);
  say(text);
  box.dataset.verdict = pass ? "pass" : "fail";
  if (pass) {
    $("fr-next").textContent = "Finish";
  } else {
    button("Try again", async () => { let on = false; try { on = (await invoke("get_status")).is_running; } catch {} run(i, on); }, "primary-btn");
  }
}
