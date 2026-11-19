// Q8: drive Simple mode and the first-run flow in headless Edge/Chrome against
// the mocked backend (mock_tauri.js), with screenshots. No microphone, no
// presenter, no network: every command and event is scripted. Node 22+.
//
//   npm run build
//   node scripts/qa/simple-walkthrough.mjs <out dir> [browser exe]
//
// It serves dist/ on 127.0.0.1:18480, starts the browser with CDP on 9343,
// and stops the browser (by PID, with its children) and the server when done.
// Exit 1 if any check fails. Placeholder song words only.
import fs from "node:fs";
import http from "node:http";
import path from "node:path";
import { spawn, execFileSync } from "node:child_process";

const ROOT = path.resolve(import.meta.dirname, "../..");
const OUT = path.resolve(process.argv[2] || "simple-walkthrough");
const BROWSER = process.argv[3] || "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe";
const PORT = 18480;
const CDP = 9343;
fs.mkdirSync(OUT, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const results = [];
const ok = (name, pass, detail = "") => {
  results.push({ name, pass, detail });
  console.log(`${pass ? "PASS" : "FAIL"} ${name}${detail ? " :: " + detail : ""}`);
};

const types = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".svg": "image/svg+xml" };
const server = http.createServer((req, res) => {
  const file = path.join(ROOT, "dist", decodeURIComponent(new URL(req.url, "http://x").pathname));
  fs.readFile(file.endsWith(path.sep) ? file + "index.html" : file, (err, body) => {
    res.writeHead(err ? 404 : 200, { "Content-Type": types[path.extname(file)] || "application/octet-stream" });
    res.end(body);
  });
}).listen(PORT, "127.0.0.1");

const profile = fs.mkdtempSync(path.join(OUT, "profile-"));
const browser = spawn(BROWSER, ["--headless=new", `--remote-debugging-port=${CDP}`, `--user-data-dir=${profile}`,
  "--no-first-run", "--disable-extensions", "about:blank"], { stdio: "ignore" });
console.log(`browser PID ${browser.pid}`);

let page;
try {
  for (let i = 0; i < 50 && !page; i++) {
    await sleep(200);
    page = await fetch(`http://127.0.0.1:${CDP}/json`).then((r) => r.json()).then((l) => l.find((t) => t.type === "page")).catch(() => null);
  }
  await run(cdp(page.webSocketDebuggerUrl));
} catch (err) {
  ok("walkthrough ran to the end", false, String(err?.stack || err));
} finally {
  // Browser.close ends every browser process; the launcher PID may already be gone.
  const version = await fetch(`http://127.0.0.1:${CDP}/json/version`).then((r) => r.json()).catch(() => null);
  if (version) await cdp(version.webSocketDebuggerUrl).send("Browser.close").catch(() => {});
  await sleep(1000);
  try { execFileSync("taskkill", ["/PID", String(browser.pid), "/T", "/F"], { stdio: "ignore" }); } catch { /* already gone */ }
  server.close();
  fs.rmSync(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 500 });
  fs.writeFileSync(path.join(OUT, "results.json"), JSON.stringify(results, null, 2));
  const failed = results.filter((r) => !r.pass).length;
  console.log(`${results.length - failed}/${results.length} passed`);
  process.exitCode = failed ? 1 : 0;
}

function cdp(wsUrl) {
  const ws = new WebSocket(wsUrl);
  let id = 0;
  const pending = new Map();
  ws.onmessage = (m) => {
    const msg = JSON.parse(m.data);
    if (msg.id && pending.has(msg.id)) pending.get(msg.id)(msg), pending.delete(msg.id);
    if (msg.method === "Runtime.exceptionThrown") console.log("page exception:", msg.params.exceptionDetails.exception?.description);
  };
  const ready = new Promise((r) => (ws.onopen = r));
  const send = async (method, params = {}) => {
    await ready;
    const i = ++id;
    ws.send(JSON.stringify({ id: i, method, params }));
    return new Promise((r, reject) => {
      pending.set(i, r);
      setTimeout(() => reject(new Error(`CDP ${method} timed out`)), 30000);
    });
  };
  const ev = async (expr) => {
    const r = await send("Runtime.evaluate", { expression: `(async () => { ${expr} })()`, awaitPromise: true, returnByValue: true });
    if (r.result?.exceptionDetails) throw new Error(r.result.exceptionDetails.exception?.description || "evaluate failed");
    return r.result?.result?.value;
  };
  const shot = async (name) => {
    const r = await send("Page.captureScreenshot", { format: "png" });
    fs.writeFileSync(path.join(OUT, `${name}.png`), Buffer.from(r.result.data, "base64"));
  };
  return { send, ev, shot };
}

async function run({ send, ev, shot }) {
  await send("Runtime.enable");
  await send("Page.enable");
  await send("Emulation.setDeviceMetricsOverride", { width: 420, height: 600, deviceScaleFactor: 1, mobile: false });
  await send("Page.addScriptToEvaluateOnNewDocument", { source: fs.readFileSync(path.join(ROOT, "scripts/qa/mock_tauri.js"), "utf8") });
  const load = async () => {
    await send("Page.navigate", { url: `http://127.0.0.1:${PORT}/index.html` });
    await sleep(1500);
  };
  const visible = (sel) => `(() => { const e = document.querySelector(${JSON.stringify(sel)}); return !!e && e.getClientRects().length > 0; })()`;
  const called = (cmd) => ev(`return window.__mock.calls.filter(c => c[0] === ${JSON.stringify(cmd)}).map(c => c[1]);`);
  const lightNow = () => ev(`return [document.querySelector('#simple .light').className, document.querySelector('#simple .sentence').textContent, document.querySelector('#simple .now').textContent, document.querySelector('#simple input[value=songs]').checked];`);
  const stepText = () => ev(`return document.querySelector('.welcome-overlay .step p').textContent;`);
  const next = () => ev(`document.querySelector('.welcome-footer .primary-btn').click(); await new Promise(r => setTimeout(r, 400)); return 1;`);

  // --- First run: a fresh profile shows the flow, step 1 is the device ---
  await load();
  let r = await ev(`return [${visible(".welcome-overlay")}, document.querySelector('.welcome-overlay').textContent.includes('Step 1 of 4'), [...document.querySelectorAll('.step option')].map(o => o.textContent)];`);
  ok("first run: the flow opens on a fresh profile at step 1 (device)", r[0] && r[1], JSON.stringify(r));
  ok("first run: device list is Default plus the enumerated inputs", JSON.stringify(r[2]) === JSON.stringify(["Default", "Desk USB input (mock)", "Laptop microphone (mock)"]), JSON.stringify(r[2]));
  await ev(`const s = document.querySelector('.step select'); s.value = 'Desk USB input (mock)'; s.dispatchEvent(new Event('change')); await new Promise(r => setTimeout(r, 200)); return 1;`);
  r = [await called("select_audio_device"), await ev(`return localStorage.getItem('voxxa.audioDevice');`)];
  ok("first run: picking the desk input selects it and remembers it", JSON.stringify(r[0].at(-1)) === '{"device":"Desk USB input (mock)"}' && r[1] === '"Desk USB input (mock)"', JSON.stringify(r));
  await shot("01-first-run-device");

  // Step 2: auto-detect finds the (mocked) ProPresenter and connects to it.
  await ev(`window.__mock.state.discover = [{ kind: 'pro_presenter7_rest', host: '127.0.0.1', port: 1025, name: 'ProPresenter 7 (mock)', source: 'localhost' }]; return 1;`);
  await next();
  await sleep(300);
  r = [await stepText(), await called("connect_presenter"), await ev(`return localStorage.getItem('voxxa.presenterForm');`)];
  ok("first run: step 2 auto-detects and connects the presentation app", r[0].startsWith("Connected to ProPresenter 7 (mock)") && r[1].at(-1)?.args?.port === 1025 && r[2].includes("pro_presenter7_rest"), JSON.stringify(r));
  await shot("02-first-run-presenter");

  // Step 3: no model yet, download the default (Base) with a progress bar.
  await next();
  r = [await stepText(), await ev(`return document.querySelector('.step .primary-btn')?.textContent;`)];
  ok("first run: step 3 offers the model download", r[0].startsWith("Voxxa needs a speech model") && r[1] === "Download", JSON.stringify(r));
  await ev(`document.querySelector('.step .primary-btn').click(); return 1;`);
  await sleep(200);
  await shot("03a-first-run-model-downloading");
  await sleep(1000);
  r = [await stepText(), await called("download_model"), await ev(`return localStorage.getItem('voxxa.lastModel');`)];
  ok("first run: model downloaded and remembered", r[0] === "Speech model ready." && r[1][0]?.modelName === "Base" && r[2] === '"Base"', JSON.stringify(r));
  await shot("03b-first-run-model-ready");

  // Step 4: the 20-second test. Without songs it says why; then load songs.
  await next();
  await ev(`[...document.querySelectorAll('.step button')].find(b => b.textContent.startsWith('Start')).click(); await new Promise(r => setTimeout(r, 500)); return 1;`);
  r = await stepText();
  ok("first run: test without songs is refused in plain words", r === "Not green yet. Couldn't start: No setlist loaded. Load a setlist first.", r);
  await shot("04a-first-run-test-no-songs");
  const setlist = { setlist: [
    { title: "Alpha Song", slides: [{ id: 0, text: "alpha bravo" }, { id: 1, text: "charlie delta" }] },
    { title: "Echo Song", slides: [{ id: 2, text: "echo foxtrot" }, { id: 3, text: "golf hotel" }, { id: 4, text: "india juliet" }] },
  ] };
  await ev(`const input = document.getElementById('file-input'); const dt = new DataTransfer();
    dt.items.add(new File([${JSON.stringify(JSON.stringify(setlist))}], 'placeholder.json', { type: 'application/json' }));
    input.files = dt.files; input.dispatchEvent(new Event('change')); await new Promise(r => setTimeout(r, 900)); return 1;`);
  r = await ev(`return document.querySelector('.step .now').textContent;`);
  ok("first run: choosing songs shows how many loaded", r === "2 songs loaded", r);
  await ev(`[...document.querySelectorAll('.step button')].find(b => b.textContent.startsWith('Start')).click(); return 1;`);
  await sleep(3000);
  r = [await stepText(), await ev(`return [...document.querySelectorAll('.welcome-overlay button')].every(b => b.disabled);`)];
  ok("first run: the test counts down and locks the buttons", /^Listening… 1[0-9] s left\.$/.test(r[0]) && r[1], JSON.stringify(r));
  await ev(`window.__mock.state.machine = 'singing'; window.__mock.emit('machine-state', { state: 'singing', is_blank: false });
    window.__mock.emit('slide-advanced', { slide_index: 3, total_slides: 5, slide_text: 'golf hotel', next_slide_text: 'india juliet', song_title: 'Echo Song' }); return 1;`);
  await shot("04b-first-run-test-running");
  await sleep(18500);
  r = [await stepText(), await called("stop_listening"), await ev(`return window.__mock.state.listening;`)];
  ok("first run: green during the test passes it, and listening stops after 20 s", r[0] === "Green: Voxxa followed your song. You are ready." && r[1].length === 1 && r[2] === false, JSON.stringify(r));
  await shot("04c-first-run-test-green");
  await next();
  r = [await ev(`return ${visible(".welcome-overlay")};`), await ev(`return localStorage.getItem('voxxa.firstLaunchDone');`)];
  ok("first run: Finish closes the flow and remembers completion", r[0] === false && r[1] === "1", JSON.stringify(r));

  // --- The default view: three controls, everything else one click away ---
  r = await ev(`const v = document.getElementById('simple');
    return { advancedHidden: !${visible("#app")},
      radios: [...v.querySelectorAll('input[name=mode]')].map(i => i.parentElement.textContent.trim() + (i.disabled ? ' (disabled)' : '')),
      buttons: [...v.querySelectorAll('button')].map(b => b.textContent), light: !!v.querySelector('.light'),
      text: v.innerText };`);
  ok("default view: the Advanced panels are hidden", r.advancedHidden, JSON.stringify(r));
  ok("default view: mode switch Off / Songs / Songs + Scripture (disabled)", JSON.stringify(r.radios) === JSON.stringify(["Off", "Songs", "Songs + Scripture (disabled)"]), JSON.stringify(r.radios));
  ok("default view: one button (Call for help) and the Advanced link", JSON.stringify(r.buttons) === JSON.stringify(["Call for help", "Advanced…"]), JSON.stringify(r.buttons));
  ok("default view: no slide text in Simple mode (ADR-011)", !/alpha|charlie|echo f|golf|india/.test(r.text), r.text);
  r = await lightNow();
  ok("default view: song and slide shown as 'Song 2: Echo Song · slide 2/3'", r[2] === "Song 2: Echo Song · slide 2/3", JSON.stringify(r));
  ok("status: Off is grey", r[0] === "light grey" && r[1] === "Off. Switch to Songs when the music starts.", JSON.stringify(r));
  await shot("05-default-view-off");

  // --- Mode switch and every status-light row, from scripted events ---
  const songs = () => ev(`document.querySelector('#simple input[value=songs]').click(); await new Promise(r => setTimeout(r, 400)); return 1;`);
  const off = () => ev(`document.querySelector('#simple input[value=off]').click(); await new Promise(r => setTimeout(r, 400)); return 1;`);
  const fire = (name, payload) => ev(`window.__mock.emit(${JSON.stringify(name)}, ${JSON.stringify(payload)}); await new Promise(r => setTimeout(r, 100)); return 1;`);
  await ev(`window.__mock.state.machine = null; return 1;`);
  const before = (await called("start_listening")).length;
  await songs();
  r = await lightNow();
  ok("mode: Songs starts listening through start_listening", (await called("start_listening")).length === before + 1 && r[3] === true, JSON.stringify(r));
  ok("status: listening, nothing matched is amber", r[0] === "light amber" && r[1] === "Listening. Waiting for a song I know.", JSON.stringify(r));
  await shot("06-status-amber");
  await fire("machine-state", { state: "singing", is_blank: false });
  await fire("slide-advanced", { slide_index: 1, total_slides: 5, slide_text: "x", next_slide_text: null, song_title: "Alpha Song" });
  r = await lightNow();
  ok("status: singing is green, with the song and slide", r[0] === "light green" && r[1] === "Following the song." && r[2] === "Song 1: Alpha Song · slide 2/2", JSON.stringify(r));
  await shot("07-status-green");
  await fire("machine-state", { state: "inter_verse_silence", is_blank: false });
  ok("status: inter-verse silence stays green", (await lightNow())[0] === "light green");
  await fire("machine-state", { state: "blank_hold", is_blank: true });
  r = await lightNow();
  ok("status: blank hold is amber with its sentence", r[0] === "light amber" && r[1].startsWith("Screen blanked"), JSON.stringify(r));
  await fire("presenter-error", "ProPresenter refused the slide (mock)");
  r = await lightNow();
  ok("status: a presenter error is red", r[0] === "light red" && r[1].startsWith("Your presentation app isn't answering"), JSON.stringify(r));
  await shot("08-status-red-presenter");
  await fire("slide-advanced", { slide_index: 2, total_slides: 5, slide_text: "x", next_slide_text: null, song_title: "Echo Song" });
  await fire("machine-state", { state: "singing", is_blank: false });
  ok("status: the next slide sent clears the presenter error", (await lightNow())[0] === "light green");
  await ev(`window.__mock.state.listening = false; return 1;`);
  await fire("listening-stopped", { disconnected: true });
  await sleep(600);
  r = await lightNow();
  ok("status: a lost microphone is red and the switch returns to Off", r[0] === "light red" && r[1].startsWith("Can't hear the microphone") && r[3] === false, JSON.stringify(r));
  await shot("09-status-red-mic");
  await songs();
  r = await lightNow();
  ok("status: switching to Songs again clears the microphone error", r[0] !== "light red" && r[3] === true, JSON.stringify(r));
  await off();
  r = [await lightNow(), await called("stop_listening"), await ev(`return window.__mock.state.listening;`)];
  ok("mode: Off stops listening through stop_listening and the light is grey", r[0][0] === "light grey" && r[1].length === 2 && r[2] === false, JSON.stringify(r));
  await ev(`window.__mock.state.loaded = null; return 1;`);
  await songs();
  r = await lightNow();
  ok("status: a refused start is red with the reason, and the switch stays Off", r[0] === "light red" && r[1].startsWith("Couldn't start: No Whisper model loaded") && r[3] === false, JSON.stringify(r));
  await shot("10-status-red-start");
  await off();
  ok("status: choosing Off clears the start error", (await lightNow())[0] === "light grey");
  await ev(`window.__mock.state.loaded = 'Base'; return 1;`);
  r = await ev(`const s = document.querySelector('#simple input[value=scripture]'); s.click(); await new Promise(r => setTimeout(r, 300)); return [s.disabled, s.checked];`);
  ok("mode: Songs + Scripture cannot be chosen", r[0] === true && r[1] === false, JSON.stringify(r));
  await ev(`document.querySelector('#simple input[value=off]').focus(); return 1;`);
  for (const type of ["keyDown", "keyUp"]) await send("Input.dispatchKeyEvent", { type, key: "ArrowRight", code: "ArrowRight", windowsVirtualKeyCode: 39 });
  await sleep(600);
  r = await lightNow();
  ok("keyboard: the arrow key moves the switch from Off to Songs and starts listening", r[3] === true && r[0] === "light amber", JSON.stringify(r));
  await off();

  // --- Call for help ---
  r = await ev(`document.querySelector('#simple .help').click(); await new Promise(r => setTimeout(r, 300)); return [...document.querySelectorAll('.toast')].map(t => t.textContent).join(' | ');`);
  ok("Call for help says it arrives in the next update", r.includes("Call for help arrives in the next update"), r);
  await shot("11-call-for-help");
  await ev(`document.querySelectorAll('.toast').forEach(t => t.remove()); return 1;`);

  // --- Advanced: one click shows every existing panel ---
  await ev(`document.querySelector('#simple .more').click(); await new Promise(r => setTimeout(r, 300)); return 1;`);
  r = await ev(`return { app: ${visible("#app")}, simple: ${visible("#simple")},
    tabs: [...document.querySelectorAll('#app .tab')].map(t => t.textContent),
    slides: ${visible("#current-slide")}, listen: ${visible("#listen-btn")}, stage: ${visible("#stage-btn")}, blank: ${visible("#blank-btn")}, back: [...document.querySelectorAll('#app .header button')].map(b => b.textContent) };`);
  ok("Advanced: one click shows the existing view (Slides / Models / Settings)", r.app && !r.simple && JSON.stringify(r.tabs) === JSON.stringify(["Slides", "Models", "Settings"]), JSON.stringify(r));
  ok("Advanced: slide preview, Listen, Blank and Stage are there", r.slides && r.listen && r.blank && r.stage, JSON.stringify(r));
  await shot("12-advanced-slides");
  r = await ev(`document.querySelector('.tab[data-tab=settings]').click(); await new Promise(r => setTimeout(r, 400));
    return { presenter: ${visible("#select-presenter")}, detect: ${visible("#discover-btn")}, device: ${visible("#select-device")}, http: ${visible("#http-api-toggle")}, diag: ${visible("#diag-btn")}, smart: ${visible("#smart-silence")},
      again: [...document.querySelectorAll('#section-settings button')].some(b => b.textContent === 'Run first-time setup again') };`);
  ok("Advanced: Settings keeps presenter, devices, HTTP API, diagnostics, smart blanking", Object.values(r).every(Boolean), JSON.stringify(r));
  await shot("13-advanced-settings");
  r = await ev(`document.querySelector('.tab[data-tab=models]').click(); await new Promise(r => setTimeout(r, 300)); return document.querySelectorAll('#model-list .model-item').length;`);
  ok("Advanced: Models lists the models", r === 3, String(r));
  await ev(`document.querySelector('.tab[data-tab=home]').click(); return 1;`);
  r = await ev(`document.getElementById('listen-btn').click(); await new Promise(r => setTimeout(r, 700)); return window.__mock.state.listening;`);
  await ev(`[...document.querySelectorAll('#app .header button')].find(b => b.textContent.includes('Simple')).click(); await new Promise(r => setTimeout(r, 700)); return 1;`);
  const back = await lightNow();
  ok("Advanced: listening started there shows in Simple mode after '← Simple'", r === true && back[3] === true && back[0] === "light amber", JSON.stringify([r, back]));
  await off();

  // --- Re-open the first-run flow from Advanced, then skip it ---
  await ev(`document.querySelector('#simple .more').click(); document.querySelector('.tab[data-tab=settings]').click(); await new Promise(r => setTimeout(r, 300));
    [...document.querySelectorAll('#section-settings button')].find(b => b.textContent === 'Run first-time setup again').click(); await new Promise(r => setTimeout(r, 400)); return 1;`);
  r = await ev(`return [${visible(".welcome-overlay")}, document.querySelector('.welcome-overlay').textContent.includes('Step 1 of 4')];`);
  ok("first run: re-opened from Advanced → Settings at step 1", r[0] && r[1], JSON.stringify(r));
  await shot("14-first-run-reopened");
  await ev(`document.querySelector('.welcome-dismiss').click(); await new Promise(r => setTimeout(r, 200)); return 1;`);
  ok("first run: Skip setup closes it", (await ev(`return ${visible(".welcome-overlay")};`)) === false);

  // --- Restart: completion and the first-run choices are remembered ---
  await load();
  r = { overlay: await ev(`return ${visible(".welcome-overlay")};`), simple: await ev(`return ${visible("#simple")};`),
    device: await called("select_audio_device"), connect: await called("connect_presenter") };
  ok("restart: no first-run flow, Simple mode is the default view", r.overlay === false && r.simple, JSON.stringify(r));
  ok("restart: the remembered input and presentation app are re-applied", r.device[0]?.device === "Desk USB input (mock)" && r.connect[0]?.args?.kind === "pro_presenter7_rest", JSON.stringify(r));
  await shot("15-restart-default-view");

  // --- No presentation app found: the keystroke fallback says so ---
  await ev(`localStorage.clear(); return 1;`);
  await load();
  await next();
  await sleep(300);
  r = await stepText();
  ok("first run: nothing found falls back to keystrokes and says so", r.startsWith("No presentation app found, so Voxxa will press the arrow keys"), r);
  await shot("16-first-run-no-presenter");
}
