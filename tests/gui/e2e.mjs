// Simple mode in a browser with a mocked backend (ADR-017 § 6, GUI testing way (a)).
// Serves the built dist/ with tests/gui/mock-tauri.js injected (never shipped), drives
// headless Edge or Chrome over the DevTools protocol, checks the default view, the
// status light, the Advanced move and the first-run flow including the 20-second test,
// and saves screenshots. Listen and the 20-second test run only here, against the mock.
//
//   npm run build && node tests/gui/e2e.mjs [screenshot dir]
// Browser: $BROWSER, else Edge or Chrome at their usual Windows, macOS or Linux paths.
// It stops the browser and the server it starts. Node 22+ (fetch, WebSocket). Mock data
// uses placeholder words only.
import fs from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";

const ROOT = path.resolve(import.meta.dirname, "../..");
const DIST = path.join(ROOT, "dist");
const OUT = path.resolve(process.argv[2] || path.join(os.tmpdir(), "voxxa-gui-e2e"));
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const results = [];
const check = (name, pass, detail = "") => {
  results.push({ name, pass: !!pass, detail });
  console.log(`${pass ? "PASS" : "FAIL"} ${name}${detail ? " :: " + detail : ""}`);
};

// --- the shipped bundle must not contain the shim ---
const bundle = fs.readdirSync(path.join(DIST, "assets")).map((f) => fs.readFileSync(path.join(DIST, "assets", f), "utf8")).join("\n");
check("shipped bundle has no mock", !bundle.includes("__mock") && !fs.readFileSync(path.join(DIST, "index.html"), "utf8").includes("mock-tauri"));

// --- static server with the shim injected ---
const MIME = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".png": "image/png", ".svg": "image/svg+xml", ".ico": "image/x-icon" };
const server = http.createServer((req, res) => {
  const url = new URL(req.url, "http://x").pathname;
  if (url === "/__mock/mock-tauri.js") {
    res.writeHead(200, { "Content-Type": MIME[".js"] });
    return res.end(fs.readFileSync(path.join(ROOT, "tests/gui/mock-tauri.js")));
  }
  const file = path.join(DIST, url === "/" ? "index.html" : url);
  if (!file.startsWith(DIST) || !fs.existsSync(file)) { res.writeHead(404); return res.end(); }
  let body = fs.readFileSync(file);
  if (file.endsWith("index.html")) body = body.toString().replace("<head>", '<head><script src="/__mock/mock-tauri.js"></script>');
  res.writeHead(200, { "Content-Type": MIME[path.extname(file)] || "application/octet-stream" });
  res.end(body);
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const APP = `http://127.0.0.1:${server.address().port}/`;

// --- browser ---
const candidates = [process.env.BROWSER,
  "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe",
  "C:/Program Files/Google/Chrome/Application/chrome.exe", "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  "/usr/bin/google-chrome", "/usr/bin/chromium", "/usr/bin/chromium-browser", "/usr/bin/microsoft-edge"].filter(Boolean);
const exe = candidates.find((p) => fs.existsSync(p));
if (!exe) { console.error("no Edge or Chrome found; set BROWSER"); server.close(); process.exit(2); }
const profile = fs.mkdtempSync(path.join(os.tmpdir(), "voxxa-e2e-profile-"));
const browser = spawn(exe, ["--headless=new", "--remote-debugging-port=0", `--user-data-dir=${profile}`, "--window-size=420,600",
  "--no-first-run", "--no-default-browser-check", "--disable-extensions", "--disable-gpu", "about:blank"], { stdio: "ignore" });
console.log(`browser pid ${browser.pid}, server ${APP}`);

let ws;
async function shutdown(code) {
  try { ws && ws.close(); } catch {}
  try {
    const port = fs.readFileSync(path.join(profile, "DevToolsActivePort"), "utf8").split("\n")[0];
    const v = await (await fetch(`http://127.0.0.1:${port}/json/version`)).json();
    const b = new WebSocket(v.webSocketDebuggerUrl);
    await new Promise((r) => (b.onopen = r));
    b.send(JSON.stringify({ id: 1, method: "Browser.close" }));
    await sleep(1000);
  } catch {}
  if (browser.exitCode === null) browser.kill();
  server.close();
  // Edge's helper processes can hold the profile for a few seconds after the browser exits.
  for (let i = 0; i < 20 && fs.existsSync(profile); i++) { try { fs.rmSync(profile, { recursive: true, force: true }); } catch { await sleep(1000); } }
  if (fs.existsSync(profile)) console.log(`could not delete the browser profile ${profile}`);
  process.exit(code);
}

try {
  let port;
  for (let i = 0; i < 50 && !port; i++) {
    try { port = fs.readFileSync(path.join(profile, "DevToolsActivePort"), "utf8").split("\n")[0]; } catch { await sleep(200); }
  }
  const page = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((t) => t.type === "page");
  ws = new WebSocket(page.webSocketDebuggerUrl);
  let id = 0;
  const pending = new Map();
  ws.onmessage = (m) => { const msg = JSON.parse(m.data); if (msg.id && pending.has(msg.id)) { pending.get(msg.id)(msg); pending.delete(msg.id); } };
  await new Promise((r) => (ws.onopen = r));
  const send = (method, params = {}) => new Promise((r) => { const i = ++id; pending.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
  const ev = async (expr) => {
    const r = await send("Runtime.evaluate", { expression: `(async () => { ${expr} })()`, awaitPromise: true, returnByValue: true });
    if (r.result && r.result.exceptionDetails) throw new Error(r.result.exceptionDetails.exception?.description || "evaluate failed");
    return r.result.result.value;
  };
  const waitFor = async (expr, ms = 5000) => { const end = Date.now() + ms; while (Date.now() < end) { const v = await ev(expr); if (v) return v; await sleep(100); } return false; };
  fs.mkdirSync(OUT, { recursive: true });
  const shot = async (name) => {
    const r = await send("Page.captureScreenshot", { format: "png" });
    fs.writeFileSync(path.join(OUT, `${name}.png`), Buffer.from(r.result.data, "base64"));
  };
  const load = async () => { await send("Page.navigate", { url: APP }); await waitFor(`return !!document.getElementById("simple") && !!window.__mock`); await sleep(300); };
  const sentence = () => ev(`return document.querySelector("#s-light").className.split(" ")[1] + " | " + document.getElementById("s-sentence").textContent`);
  const light = async (want, name, re) => {
    const ok = await waitFor(`return document.querySelector("#s-light").classList.contains(${JSON.stringify(want)}) && ${re}.test(document.getElementById("s-sentence").textContent)`, 3000);
    check(`light ${name}`, ok, await sentence());
  };
  const click = (sel) => ev(`document.querySelector(${JSON.stringify(sel)}).click(); return true;`);
  const visible = (sel) => ev(`const e = document.querySelector(${JSON.stringify(sel)}); return !!e && e.checkVisibility();`);
  const calls = (cmd) => ev(`return __mock.calls.filter((c) => c.cmd === ${JSON.stringify(cmd)}).length`);
  // A real Space key press on a focused element, as the operator's keyboard sends it.
  const space = async (sel) => {
    await ev(`document.querySelector(${JSON.stringify(sel)}).focus(); return true`);
    const key = { key: " ", code: "Space", windowsVirtualKeyCode: 32, nativeVirtualKeyCode: 32 };
    await send("Input.dispatchKeyEvent", { type: "keyDown", text: " ", ...key });
    await send("Input.dispatchKeyEvent", { type: "keyUp", ...key });
  };
  await send("Page.enable");
  await send("Runtime.enable");
  await send("Emulation.setDeviceMetricsOverride", { width: 420, height: 600, deviceScaleFactor: 1, mobile: false });

  // 1. Fresh profile: the first-run flow opens over Simple mode.
  await load();
  check("first run opens on a fresh profile", await visible("#firstrun"));
  check("first run step 1 is the microphone", (await ev(`return document.getElementById("fr-title").textContent`)) === "Which microphone?");
  await shot("01-first-run-device");
  await click("#fr-skip");
  check("Skip closes it", !(await visible("#firstrun")));
  check("Skip is remembered", (await ev(`return localStorage.getItem("voxxa.firstLaunchDone")`)) === "1");

  // 2. The default view: three controls, the light, the song line.
  const controls = await ev(`return [...document.querySelectorAll("#simple button, #simple a, #simple input, #simple select")]
    .filter((e) => e.checkVisibility()).map((e) => e.closest(".s-switch") ? "mode switch" : e.textContent.trim())
    .filter((v, i, a) => a.indexOf(v) === i)`);
  check("default view shows exactly three controls", JSON.stringify(controls) === JSON.stringify(["mode switch", "Call for help", "Advanced"]), JSON.stringify(controls));
  check("mode switch has Off, Songs, Songs + Scripture", (await ev(`return [...document.querySelectorAll(".s-switch button")].map((b) => b.textContent + (b.disabled ? " (disabled)" : "")).join(", ")`)) === "Off, Songs, Songs + Scripture (disabled)");
  check("old panels are hidden", !(await visible("#app")) && !(await visible("#diag-overlay")));
  check("Off is selected", (await ev(`return document.querySelector('[data-mode="off"]').getAttribute("aria-checked")`)) === "true");
  await light("grey", "row 2 (no songs)", /Load this week's songs/);
  check("song line before any song", (await ev(`return document.getElementById("s-song").textContent`)) === "No song on screen yet");
  await shot("02-simple-no-songs");
  await click('[data-mode="songs"]');
  check("Songs without songs: start refused, switch back on Off", await waitFor(`return document.querySelector('[data-mode="off"]').getAttribute("aria-checked") === "true" && __mock.calls.filter((c) => c.cmd === "start_listening").length === 1`, 4000));
  check("the refusal is shown", await waitFor(`return [...document.querySelectorAll(".toast")].some((t) => /No setlist/.test(t.textContent))`, 2000));
  await shot("03-simple-songs-refused");
  await click('[data-mode="scripture"]');
  check("Songs + Scripture does nothing", (await calls("start_listening")) === 1);

  // 3. Reload: the first run stays closed; re-open it from Advanced.
  await load();
  check("first run remembered after reload", !(await visible("#firstrun")));
  await click("#s-advanced");
  check("Advanced: one click shows the old window", (await visible("#app")) && !(await visible("#simple")));
  await click("#s-guide");
  check("Set-up guide re-opens the first run", await visible("#firstrun"));

  // 4. First-run flow: microphone.
  check("devices listed, Default first", (await waitFor(`return [...document.querySelectorAll("#fr-device option")].map((o) => o.textContent).join("|")`)) === "Default|Desk USB input|Laptop microphone");
  await ev(`document.getElementById("fr-device").value = "Desk USB input"; return true`);
  const startsBeforeSpace = await calls("start_listening");
  await space("#fr-next"); // Space presses Next; it must not reach main.js's Listen shortcut
  check("Space on Next: step 2", await waitFor(`return document.getElementById("fr-step").textContent === "Step 2 of 4"`));
  check("Space on Next: no start_listening", (await calls("start_listening")) === startsBeforeSpace && !(await ev(`return __mock.st.running`)));
  check("device saved for Settings", (await waitFor(`return localStorage.getItem("voxxa.audioDevice")`)) === '"Desk USB input"');
  check("select_audio_device called with it", await ev(`return __mock.calls.some((c) => c.cmd === "select_audio_device" && c.args.device === "Desk USB input")`));

  // Presenter: nothing found first, then the mock app.
  check("presenter step: nothing found says keystrokes", await waitFor(`return /arrow keys/.test(document.getElementById("fr-msg").textContent)`, 3000));
  await shot("04-first-run-no-presenter");
  await ev(`__mock.st.found = [{ kind: "pro_presenter7_rest", host: "127.0.0.1", port: 1025, name: "ProPresenter on this computer", source: "localhost" }]; return true`);
  await ev(`[...document.querySelectorAll("#fr-extra button")].find((b) => b.textContent === "Search again").click(); return true`);
  check("presenter step: app found", await waitFor(`return !!document.querySelector("#fr-extra .discovered-row")`, 3000));
  await shot("05-first-run-presenter-found");
  await click("#fr-extra .discovered-row");
  check("connect_presenter called with the found app", await waitFor(`return __mock.calls.some((c) => c.cmd === "connect_presenter" && c.args.args.port === 1025)`));
  check("presenter form saved for Settings", JSON.parse(await ev(`return localStorage.getItem("voxxa.presenterForm")`) || "{}").kind === "pro_presenter7_rest");
  await click("#fr-next");

  // Model: download Tiny with progress.
  check("model step offers the download", await waitFor(`return /Tiny \\(75MB\\)/.test(document.getElementById("fr-msg").textContent)`));
  await ev(`[...document.querySelectorAll("#fr-extra button")].find((b) => b.textContent === "Download").click(); return true`);
  check("progress bar shown", await waitFor(`return !!document.querySelector("#fr-extra progress")`));
  check("model ready after download", await waitFor(`return /ready: Tiny/.test(document.getElementById("fr-msg").textContent)`, 3000));
  check("download_model called for Tiny", await ev(`return __mock.calls.some((c) => c.cmd === "download_model" && c.args.modelName === "Tiny")`));
  await shot("06-first-run-model-ready");
  await click("#fr-next");

  // Test: songs first, then a silent test (fail), then a test with sound and words (pass).
  check("test step asks for songs first", await waitFor(`return /Load this Sunday's songs/.test(document.getElementById("fr-msg").textContent)`));
  await shot("07-first-run-needs-songs");
  const setlist = { setlist: [
    { title: "Alpha Hymn", slides: [{ id: 0, text: "alpha bravo" }, { id: 1, text: "charlie delta" }] },
    { title: "Bravo Hymn", slides: [{ id: 2, text: "echo foxtrot" }, { id: 3, text: "golf hotel" }, { id: 4, text: "india juliet" }] },
    { title: "Charlie Hymn", slides: [{ id: 5, text: "kilo lima" }] },
  ] };
  await ev(`__mock.st.presenter = { kind: "keystroke", display_name: "Keystroke (any app)", connected: true }; return true`);
  await ev(`const input = document.getElementById("file-input"); const dt = new DataTransfer();
    dt.items.add(new File([${JSON.stringify(JSON.stringify(setlist))}], "placeholder.json", { type: "application/json" }));
    input.files = dt.files; input.dispatchEvent(new Event("change")); return true`);
  check("songs loaded through the existing file input", await waitFor(`return [...document.querySelectorAll("#fr-extra button")].some((b) => /20-second/.test(b.textContent))`, 3000));
  check("keystroke users are told to click into their presentation app", /If Voxxa presses keys for you, click into your presentation app right after you start\.$/.test(await ev(`return document.getElementById("fr-msg").textContent`)));
  await ev(`__mock.st.presenter = { kind: "pro_presenter7_rest", display_name: "ProPresenter 7.9+ (REST API)", connected: true }; return true`);
  await ev(`[...document.querySelectorAll("#fr-extra button")].find((b) => /20-second/.test(b.textContent)).click(); return true`);
  check("test starts listening through the Listen button", await waitFor(`return __mock.st.running`, 2000));
  await shot("08-first-run-test-running");
  check("silent test fails", await waitFor(`return document.getElementById("firstrun").dataset.verdict === "fail"`, 25000), await ev(`return document.getElementById("fr-msg").textContent`));
  check("test stopped listening", !(await ev(`return __mock.st.running`)) && (await calls("stop_listening")) === 1);
  await shot("09-first-run-test-silent-fail");
  await ev(`[...document.querySelectorAll("#fr-extra button")].find((b) => b.textContent === "Try again").click(); return true`);
  await waitFor(`return __mock.st.running`, 2000);
  // Sound, words, and the conductor's slide change reaching the presenter (slide-advanced).
  for (let i = 0; i < 6; i++) { await sleep(1500); await ev(`__mock.hear(0.2, "alpha bravo"); ${i === 3 ? "__mock.goto(0); " : ""}return true`); }
  check("test with sound and words passes", await waitFor(`return document.getElementById("firstrun").dataset.verdict === "pass"`, 25000), await ev(`return document.getElementById("fr-msg").textContent`));
  check("no heard text shown in the verdict", !(await ev(`return document.getElementById("firstrun").textContent.includes("alpha bravo")`)));
  await shot("10-first-run-test-pass");
  await click("#fr-next");
  check("Finish closes the first run", !(await visible("#firstrun")) && (await ev(`return localStorage.getItem("voxxa.firstLaunchDone")`)) === "1");
  await click("#s-simple");

  // 5. The status light, row by row, through the real events.
  await light("grey", "row 4 (ready, off)", /Choose Songs/);
  await shot("11-simple-off-ready");
  await click('[data-mode="songs"]');
  await light("amber", "row 10 (listening)", /Waiting for a song/);
  check("Songs selected while listening", (await ev(`return document.querySelector('[data-mode="songs"]').getAttribute("aria-checked")`)) === "true");
  await shot("12-simple-amber-listening");
  // The status is aria-live: a poll that changes nothing must not rewrite its text.
  await ev(`window.__writes = 0; new MutationObserver((m) => (window.__writes += m.length)).observe(document.querySelector(".s-status"), { childList: true, characterData: true, subtree: true }); return true`);
  await sleep(2500);
  check("an unchanged status is not rewritten each second", (await ev(`return window.__writes`)) === 0, `${await ev(`return window.__writes`)} writes`);
  await ev(`__mock.goto(3); return true`);
  await light("green", "row 9 (following)", /Following Bravo Hymn, slide 2 of 3/);
  check("song and slide shown", (await ev(`return document.getElementById("s-song").textContent + " / " + document.getElementById("s-slide").textContent`)) === "Song 2 of 3 · Bravo Hymn / Slide 2 of 3");
  check("no slide text in Simple mode", !(await ev(`return document.getElementById("simple").textContent.includes("echo foxtrot") || document.getElementById("simple").textContent.includes("golf hotel")`)));
  await shot("13-simple-green-following");
  await ev(`__mock.machine("blank_hold", true); return true`);
  await light("amber", "row 7 (talking)", /talking/);
  await shot("14-simple-amber-talking");
  await ev(`__mock.machine("singing", true); return true`);
  await light("amber", "row 8 (blanked)", /screen is blank/);
  // The keystroke driver's refusal while Voxxa has the focus (src-tauri/src/dispatch.rs:24).
  await ev(`__mock.presenterError("focus your presentation app: Voxxa's own window has the keyboard"); return true`);
  await light("red", "row 5 (presenter error, with the reason)", /ProPresenter 7\.9\+ didn't take the last slide change: focus your presentation app: Voxxa's own window has the keyboard\.$/);
  await shot("15-simple-red-presenter-error");
  await ev(`__mock.goto(4); return true`);
  await light("green", "a delivered slide clears row 5", /slide 3 of 3/);
  await ev(`__mock.st.presenter.connected = false; return true`);
  await light("red", "row 6 (presenter not found)", /not found/);
  await ev(`__mock.st.presenter.connected = true; return true`);
  await ev(`__mock.micLost(); return true`);
  await light("red", "row 1 (mic lost)", /Can't hear the mic/);
  check("mic lost: switch back on Off", await waitFor(`return document.querySelector('[data-mode="off"]').getAttribute("aria-checked") === "true"`));
  await shot("16-simple-red-mic-lost");
  await click('[data-mode="songs"]');
  await light("amber", "starting again clears row 1", /Waiting for a song/);
  await click('[data-mode="off"]');
  await light("grey", "Off stops listening", /Choose Songs/);
  check("Off called stop_listening", (await calls("stop_listening")) === 3 && !(await ev(`return __mock.st.running`)));
  // One toggle at a time (start_listening takes 300 ms in the mock): two clicks on Songs
  // in one tick start once; Off clicked 150 ms after Songs still ends stopped.
  const starts = await calls("start_listening");
  await ev(`const b = document.querySelector('[data-mode="songs"]'); b.click(); b.click(); return true`);
  check("Songs shows at once while starting", (await ev(`return document.querySelector('[data-mode="songs"]').getAttribute("aria-checked")`)) === "true");
  await light("amber", "two clicks on Songs: listening", /Waiting for a song/);
  check("two clicks on Songs: one start_listening", (await calls("start_listening")) === starts + 1);
  await click('[data-mode="off"]');
  check("two clicks on Songs, then Off: stopped", await waitFor(`return !__mock.st.running && document.querySelector('[data-mode="off"]').getAttribute("aria-checked") === "true"`, 4000));
  await sleep(1500);
  check("two clicks on Songs, then Off: one start_listening, still stopped", (await calls("start_listening")) === starts + 1 && !(await ev(`return __mock.st.running`)));
  await ev(`document.querySelector('[data-mode="songs"]').click(); setTimeout(() => document.querySelector('[data-mode="off"]').click(), 150); return true`);
  check("Off 150 ms after Songs: stopped", await waitFor(`return __mock.calls.filter((c) => c.cmd === "stop_listening").length === 5 && !__mock.st.running`, 4000));
  await sleep(1500);
  check("Off 150 ms after Songs: still stopped, switch on Off", !(await ev(`return __mock.st.running`)) && (await calls("start_listening")) === starts + 2 && (await ev(`return document.querySelector('[data-mode="off"]').getAttribute("aria-checked")`)) === "true");
  await light("grey", "Off 150 ms after Songs: grey", /Choose Songs/);

  // 6. Call for help: a message, no command.
  const before = await ev(`return __mock.calls.filter((c) => !["get_status", "get_presenter_info"].includes(c.cmd)).length`);
  const startsBeforeHelp = await calls("start_listening");
  await space("#s-help"); // Space presses Call for help; it must not reach the Listen shortcut
  await sleep(300);
  check("Space on Call for help: no start_listening", (await calls("start_listening")) === startsBeforeHelp && !(await ev(`return __mock.st.running`)));
  check("Call for help shows the next-update message", (await visible("#s-help-msg")) && /next update/.test(await ev(`return document.getElementById("s-help-msg").textContent`)));
  check("Call for help calls no command", (await ev(`return __mock.calls.filter((c) => !["get_status", "get_presenter_info"].includes(c.cmd)).length`)) === before && (await calls("generate_diagnostic_report")) === 0);
  await shot("17-simple-call-for-help");

  // 7. Advanced: every old panel, unchanged, one click away.
  await click("#s-advanced");
  check("Slides panel shown", await visible("#section-home"));
  for (const [sel, what] of [["#slide-view", "slide previews"], ["#listen-btn", "Listen"], ["#stage-btn", "stage display"], ["#blank-btn", "Blank"], ["#export-setlist-btn", "setlist export"]]) {
    check(`Advanced → Slides: ${what}`, await visible(sel));
  }
  await ev(`__mock.emit("song-detection", { rows: [{ song_title: "Alpha Hymn", probability: 0.7 }] }); return true`);
  check("Advanced → Slides: confidence bars", await waitFor(`return document.getElementById("detect-section").checkVisibility()`));
  await shot("18-advanced-slides");
  await click('.tab[data-tab="models"]');
  check("Advanced → Models: model list", await waitFor(`return document.querySelectorAll("#model-list .model-item").length === 3`));
  await shot("19-advanced-models");
  await click('.tab[data-tab="settings"]');
  for (const [sel, what] of [["#select-presenter", "presenter connection"], ["#discover-btn", "auto-detect"], ["#select-device", "audio device"], ["#select-language", "language"], ["#smart-silence", "smart blanking"], ["#privacy-card", "privacy counter"], ["#http-api-toggle", "HTTP API"], ["#diag-btn", "diagnostics"]]) {
    check(`Advanced → Settings: ${what}`, await visible(sel));
  }
  check("Settings shows the device chosen in the first run", await waitFor(`return document.getElementById("select-device").value === "Desk USB input"`));
  await shot("20-advanced-settings");
  await click("#diag-btn");
  check("diagnostic modal opens outside #app", await waitFor(`return document.getElementById("diag-overlay").checkVisibility()`));
  await click("#diag-close");
  await click('.tab[data-tab="home"]');
  // The setlist loader with its import paths, once no setlist is loaded. The reload also
  // reconnects the presenter chosen in the first run (the mock starts on the keystroke driver).
  await load();
  check("reload reconnects the saved presenter", await waitFor(`return __mock.calls.some((c) => c.cmd === "connect_presenter" && c.args.args.kind === "pro_presenter7_rest" && c.args.args.port === 1025) && __mock.st.presenter.kind === "pro_presenter7_rest"`));
  await click("#s-advanced");
  for (const [sel, what] of [["#setlist-loader", "imports"], ["#hymnary-section", "Hymnary"], ["#pco-section", "Planning Center"], ["#recent-section", "recent setlists"]]) {
    check(`Advanced → Slides: ${what}`, await visible(sel));
  }
  await shot("21-advanced-loader");
  await click("#s-simple");
  check("Simple mode link goes back", (await visible("#simple")) && !(await visible("#app")));

  // 8. A webview reload mid-song: the backend keeps its setlist and keeps following, the
  // page has lost its song list. Row 9 has no position then: "Following the song."
  await load();
  await ev(`__mock.st.songs = ${JSON.stringify(setlist.setlist)}; __mock.st.running = true; __mock.st.machine = "listening"; return true`);
  await light("amber", "after a reload mid-song: listening", /Waiting for a song/);
  await ev(`__mock.goto(3); return true`);
  await light("green", "row 9 after a reload: no position", /^Following the song\.$/);
  check("after a reload: the song's title, no slide count", (await ev(`return document.getElementById("s-song").textContent + " / " + document.getElementById("s-slide").textContent`)) === "Bravo Hymn / ");
  await shot("22-simple-reload-mid-song");
} catch (err) {
  check("e2e ran to the end", false, String(err && err.stack || err));
}

const failed = results.filter((r) => !r.pass).length;
fs.writeFileSync(path.join(OUT, "results.json"), JSON.stringify(results, null, 2));
console.log(`\n${results.length - failed} passed, ${failed} failed; screenshots in ${OUT}`);
await shutdown(failed ? 1 : 0);
