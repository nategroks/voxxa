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

// --- static server with the shim injected, under the app's own CSP (ADR-020) ---
const CSP = JSON.parse(fs.readFileSync(path.join(ROOT, "src-tauri/tauri.conf.json"), "utf8")).app.security.csp;
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
  res.writeHead(200, { "Content-Type": MIME[path.extname(file)] || "application/octet-stream", "Content-Security-Policy": CSP });
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
  // Every CSP report the browser logs, over all page loads (the check is at the end).
  const cspLog = [];
  ws.onmessage = (m) => { const msg = JSON.parse(m.data); if (msg.id && pending.has(msg.id)) { pending.get(msg.id)(msg); pending.delete(msg.id); }
    if (msg.method === "Log.entryAdded" && /Content Security Policy/.test(msg.params.entry.text)) cspLog.push(msg.params.entry.text); };
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
  // The colour a volunteer sees, not only the class: grey and amber are fixed, green and red
  // are main.css's --success and --danger as the browser resolves them; all four differ.
  const COLOURS = `(() => { const probe = (c) => { const e = document.createElement("i"); e.style.color = c; document.body.appendChild(e); const v = getComputedStyle(e).color; e.remove(); return v; };
    return { grey: "rgb(107, 114, 128)", amber: "rgb(245, 158, 11)", green: probe("var(--success)"), red: probe("var(--danger)") }; })()`;
  const light = async (want, name, re) => {
    const ok = await waitFor(`const c = ${COLOURS}; return new Set(Object.values(c)).size === 4 && document.querySelector("#s-light").classList.contains(${JSON.stringify(want)})
      && getComputedStyle(document.getElementById("s-light")).backgroundColor === c[${JSON.stringify(want)}] && ${re}.test(document.getElementById("s-sentence").textContent)`, 3000);
    check(`light ${name}`, ok, `${await sentence()} | ${await ev(`return getComputedStyle(document.getElementById("s-light")).backgroundColor`)}`);
  };
  // The "What to do" link under the sentence (ADR-018 § 7): shown on the runbook rows only;
  // a click asks the backend to open that runbook (the mock records the name).
  const runbook = async (row, want) => {
    const shown = await waitFor(`return document.getElementById("s-runbook").checkVisibility() === ${!!want}`, 2000);
    if (!want) return check(`${row}: no "What to do" link`, shown);
    await click("#s-runbook");
    check(`${row}: "What to do" opens ${want}`, shown && (await waitFor(`return __mock.st.runbooks.at(-1) === ${JSON.stringify(want)}`, 2000)), await ev(`return JSON.stringify(__mock.st.runbooks)`));
  };
  // What a user can see: not hidden, transparent or invisible, and inside the window.
  const SEEN = `const seen = (e) => { if (!e.checkVisibility({ opacityProperty: true, visibilityProperty: true })) return false;
    const r = e.getBoundingClientRect(); return r.width > 0 && r.height > 0 && r.top >= 0 && r.left >= 0 && r.bottom <= innerHeight && r.right <= innerWidth; };
    const scroll = (e) => { for (let p = e.parentElement; p && p !== document.body; p = p.parentElement) { const o = getComputedStyle(p).overflowY;
      if ((o === "auto" || o === "scroll") && p.scrollHeight > p.clientHeight) { const a = p.getBoundingClientRect(), r = e.getBoundingClientRect(); p.scrollTop += Math.max(0, r.bottom - a.bottom) - Math.max(0, a.top - r.top); } } };`;
  // Clicks only what a user could click: element.click() also works on hidden, transparent or
  // off-screen elements, so a hidden Skip, Help or Advanced control would otherwise pass
  // unnoticed. An element below the fold of a scrolling panel is scrolled to, as a user would.
  const PRESS = `${SEEN} const press = (e, what) => { if (!e) throw new Error("no element: " + what); if (!seen(e)) scroll(e); if (!seen(e)) throw new Error("not visible, a user could not click it (scrolling its panel included): " + what); e.click(); return true; };`;
  const click = (sel) => ev(`${PRESS} return press(document.querySelector(${JSON.stringify(sel)}), ${JSON.stringify(sel)});`);
  const clickButton = (container, re) => ev(`${PRESS} return press([...document.querySelectorAll(${JSON.stringify(container + " button")})].find((b) => ${re}.test(b.textContent)), ${JSON.stringify(container + " " + re)});`);
  // Simple mode shows nothing else: no visible control outside #simple (one clipped below the
  // fold by body{overflow:hidden} is still in the focus order), and nothing else on screen.
  const leaks = () => ev(`const simple = document.getElementById("simple"); const name = (e) => e.tagName.toLowerCase() + (e.id ? "#" + e.id : "") + (e.className && typeof e.className === "string" ? "." + e.className.trim().split(/\\s+/).join(".") : "");
    const toast = (e) => !!e.closest("#toast-container");
    const controls = [...document.querySelectorAll("button, a, input, select, textarea, [tabindex]")].filter((e) => e.checkVisibility() && !simple.contains(e) && !toast(e)).map(name);
    const onScreen = [...document.body.querySelectorAll("*")].filter((e) => { if (simple.contains(e) || e.contains(simple) || toast(e) || !e.checkVisibility()) return false;
      const r = e.getBoundingClientRect(); return r.width > 0 && r.height > 0 && r.bottom > 0 && r.right > 0 && r.top < innerHeight && r.left < innerWidth; }).map(name);
    return [...new Set([...controls, ...onScreen])];`);
  const onlySimple = async (when) => { const l = await leaks(); check(`only Simple mode is shown (${when})`, l.length === 0, JSON.stringify(l)); };
  const visible = (sel) => ev(`const e = document.querySelector(${JSON.stringify(sel)}); return !!e && e.checkVisibility();`);
  const calls = (cmd) => ev(`return __mock.calls.filter((c) => c.cmd === ${JSON.stringify(cmd)}).length`);
  // A real Space key press on a focused element, as the operator's keyboard sends it.
  const space = async (sel) => {
    await ev(`document.querySelector(${JSON.stringify(sel)}).focus(); return true`);
    const key = { key: " ", code: "Space", windowsVirtualKeyCode: 32, nativeVirtualKeyCode: 32 };
    await send("Input.dispatchKeyEvent", { type: "keyDown", text: " ", ...key });
    await send("Input.dispatchKeyEvent", { type: "keyUp", ...key });
  };
  // A key pressed while nothing has focus (the page body), as when the guide opens or after a step clears its buttons.
  const keyOnBody = async (k) => {
    check(`focus on the page before "${k}"`, await ev(`if (document.activeElement) document.activeElement.blur(); return document.activeElement === document.body`));
    const key = k === " " ? { key: " ", code: "Space", windowsVirtualKeyCode: 32, nativeVirtualKeyCode: 32 } : { key: k, code: "Key" + k.toUpperCase(), windowsVirtualKeyCode: k.toUpperCase().charCodeAt(0), nativeVirtualKeyCode: k.toUpperCase().charCodeAt(0) };
    await send("Input.dispatchKeyEvent", { type: "keyDown", text: k, ...key });
    await send("Input.dispatchKeyEvent", { type: "keyUp", ...key });
  };
  await send("Page.enable");
  await send("Runtime.enable");
  await send("Log.enable");
  await send("Emulation.setDeviceMetricsOverride", { width: 420, height: 600, deviceScaleFactor: 1, mobile: false });

  // 1. Fresh profile: the first-run flow opens over Simple mode.
  await load();
  check("first run opens on a fresh profile", await visible("#firstrun"));
  check("first run step 1 is the microphone", (await ev(`return document.getElementById("fr-title").textContent`)) === "Which microphone?");
  check("Skip is visible on step 1", await visible("#fr-skip"));
  await shot("01-first-run-device");
  await click("#fr-skip");
  check("Skip closes it", !(await visible("#firstrun")));
  check("Skip is remembered", (await ev(`return localStorage.getItem("voxxa.firstLaunchDone")`)) === "1");
  const stageAfterSkip = await calls("toggle_stage_display");
  await keyOnBody("s");
  await new Promise((r) => setTimeout(r, 300));
  check("guide closed: s reaches main.js again", (await calls("toggle_stage_display")) === stageAfterSkip + 1);
  await keyOnBody("s"); // close the stage window again

  // 2. The default view: three controls, the light, the song line.
  const controls = await ev(`${SEEN} return [...document.querySelectorAll("#simple button, #simple a, #simple input, #simple select")]
    .filter(seen).map((e) => e.closest(".s-switch") ? "mode switch" : e.textContent.trim())
    .filter((v, i, a) => a.indexOf(v) === i)`);
  check("default view shows exactly three controls", JSON.stringify(controls) === JSON.stringify(["mode switch", "Call for help", "Advanced"]), JSON.stringify(controls));
  check("mode switch has Off, Songs, Songs + Scripture", (await ev(`return [...document.querySelectorAll(".s-switch button")].map((b) => b.textContent + (b.disabled ? " (disabled)" : "")).join(", ")`)) === "Off, Songs, Songs + Scripture (disabled)");
  check("old panels are hidden", !(await visible("#app")) && !(await visible("#diag-overlay")));
  await onlySimple("first view");
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
  await onlySimple("after a reload");
  await click("#s-advanced");
  check("Advanced: one click shows the old window", (await visible("#app")) && !(await visible("#simple")));
  await click("#s-guide");
  check("Set-up guide re-opens the first run", await visible("#firstrun"));

  // The guide stops main.js's shortcut keys wherever focus is (ADR-017 § 1, Q8-10).
  const stageBefore = await calls("toggle_stage_display");
  const startsBeforeBody = await calls("start_listening");
  await keyOnBody(" ");
  await keyOnBody("s");
  await new Promise((r) => setTimeout(r, 300));
  check("guide open, focus on the page: Space starts nothing", (await calls("start_listening")) === startsBeforeBody && !(await ev(`return __mock.st.running`)));
  check("guide open, focus on the page: s opens no stage window", (await calls("toggle_stage_display")) === stageBefore);

  // 4. First-run flow: microphone.
  check("devices listed, Default first", (await waitFor(`return [...document.querySelectorAll("#fr-device option")].map((o) => o.textContent).join("|")`)) === "Default|Desk USB input|Laptop microphone");
  await ev(`document.getElementById("fr-device").value = "Desk USB input"; return true`);
  const startsBeforeSpace = await calls("start_listening");
  await space("#fr-next"); // Space presses Next; it must not reach main.js's Listen shortcut
  check("Space on Next: step 2", await waitFor(`return document.getElementById("fr-step").textContent === "Step 2 of 4"`));
  check("Skip is visible on step 2", await visible("#fr-skip"));
  check("Space on Next: no start_listening", (await calls("start_listening")) === startsBeforeSpace && !(await ev(`return __mock.st.running`)));
  check("device saved for Settings", (await waitFor(`return localStorage.getItem("voxxa.audioDevice")`)) === '"Desk USB input"');
  check("select_audio_device called with it", await ev(`return __mock.calls.some((c) => c.cmd === "select_audio_device" && c.args.device === "Desk USB input")`));

  // Presenter: nothing found first, then the mock app.
  check("presenter step: nothing found says keystrokes", await waitFor(`return /arrow keys/.test(document.getElementById("fr-msg").textContent)`, 3000));
  await shot("04-first-run-no-presenter");
  await ev(`__mock.st.found = [{ kind: "pro_presenter7_rest", host: "127.0.0.1", port: 1025, name: "ProPresenter on this computer", source: "localhost" }]; return true`);
  await clickButton("#fr-extra", /^Search again$/);
  check("presenter step: app found", await waitFor(`return !!document.querySelector("#fr-extra .discovered-row")`, 3000));
  await shot("05-first-run-presenter-found");
  await click("#fr-extra .discovered-row");
  check("connect_presenter called with the found app", await waitFor(`return __mock.calls.some((c) => c.cmd === "connect_presenter" && c.args.args.port === 1025)`));
  check("presenter form saved for Settings", JSON.parse(await ev(`return localStorage.getItem("voxxa.presenterForm")`) || "{}").kind === "pro_presenter7_rest");
  await click("#fr-next");

  // Model: download Tiny with progress.
  check("Skip is visible on step 3", await visible("#fr-skip"));
  check("model step offers the download", await waitFor(`return /Tiny \\(75MB\\)/.test(document.getElementById("fr-msg").textContent)`));
  await clickButton("#fr-extra", /^Download$/);
  check("progress bar shown", await waitFor(`return !!document.querySelector("#fr-extra progress")`));
  check("model ready after download", await waitFor(`return /ready: Tiny/.test(document.getElementById("fr-msg").textContent)`, 3000));
  check("download_model called for Tiny", await ev(`return __mock.calls.some((c) => c.cmd === "download_model" && c.args.modelName === "Tiny")`));
  await shot("06-first-run-model-ready");
  await click("#fr-next");

  // Test: songs first, then a silent test (fail), then a test with sound and words (pass).
  check("Skip is visible on step 4", await visible("#fr-skip"));
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
  await clickButton("#fr-extra", /20-second/);
  check("test starts listening through the Listen button", await waitFor(`return __mock.st.running`, 2000));
  await shot("08-first-run-test-running");
  const stopsDuringTest = await calls("stop_listening");
  const stageDuringTest = await calls("toggle_stage_display");
  await keyOnBody(" ");
  await keyOnBody("s");
  await new Promise((r) => setTimeout(r, 300));
  check("20-second test, focus on the page: Space does not stop it", (await ev(`return __mock.st.running`)) && (await calls("stop_listening")) === stopsDuringTest);
  check("20-second test, focus on the page: s opens no stage window", (await calls("toggle_stage_display")) === stageDuringTest);
  check("silent test fails", await waitFor(`return document.getElementById("firstrun").dataset.verdict === "fail"`, 25000), await ev(`return document.getElementById("fr-msg").textContent`));
  check("test stopped listening", !(await ev(`return __mock.st.running`)) && (await calls("stop_listening")) === 1);
  await shot("09-first-run-test-silent-fail");
  // A start that is refused after a while (the device is busy): "didn't start", nothing listening.
  await ev(`__mock.st.startDelay = 1600; __mock.st.startError = "mock: the device is busy"; return true`);
  await clickButton("#fr-extra", /^Try again$/);
  check("slow refused start: didn't start", await waitFor(`return document.getElementById("firstrun").dataset.verdict === "fail" && /didn't start/.test(document.getElementById("fr-msg").textContent)`, 8000), await ev(`return document.getElementById("fr-msg").textContent`));
  await sleep(1000);
  check("slow refused start: nothing listening", !(await ev(`return __mock.st.running`)) && !(await ev(`return document.getElementById("listen-btn").classList.contains("active")`)));
  // A start slower than a second (some USB or Bluetooth inputs): the test waits for it, runs,
  // and stops it; nothing is left listening after the verdict.
  await ev(`__mock.st.startError = null; delete document.getElementById("firstrun").dataset.verdict; return true`);
  const stopsBeforeSlow = await calls("stop_listening");
  await clickButton("#fr-extra", /^Try again$/);
  check("slow start: the test waits for it", await waitFor(`return document.getElementById("firstrun").dataset.verdict === "fail"`, 30000) && !/didn't start/.test(await ev(`return document.getElementById("fr-msg").textContent`)), await ev(`return document.getElementById("fr-msg").textContent`));
  await sleep(2500);
  check("slow start: nothing listening after the verdict", !(await ev(`return __mock.st.running`)) && !(await ev(`return document.getElementById("listen-btn").classList.contains("active")`)) && (await calls("stop_listening")) === stopsBeforeSlow + 1);
  await ev(`__mock.st.startDelay = 300; return true`);
  await clickButton("#fr-extra", /^Try again$/);
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
  await onlySimple("following a song");
  check("no slide text in Simple mode", !(await ev(`return document.getElementById("simple").textContent.includes("echo foxtrot") || document.getElementById("simple").textContent.includes("golf hotel")`)));
  await shot("13-simple-green-following");
  await runbook("row 9 (green)", null);
  await ev(`__mock.machine("blank_hold", true); return true`);
  await light("amber", "row 7 (talking)", /talking/);
  await shot("14-simple-amber-talking");
  await runbook("row 7 (amber)", null);
  await ev(`__mock.machine("singing", true); return true`);
  await light("amber", "row 8 (blanked)", /screen is blank/);
  // The keystroke driver's refusal while Voxxa has the focus (src-tauri/src/dispatch.rs:24).
  await ev(`__mock.presenterError("focus your presentation app: Voxxa's own window has the keyboard"); return true`);
  await light("red", "row 5 (presenter error, with the reason)", /ProPresenter 7\.9\+ didn't take the last slide change: focus your presentation app: Voxxa's own window has the keyboard\.$/);
  await shot("15-simple-red-presenter-error");
  await runbook("row 5 (red)", "presentation-app-not-reachable");
  await ev(`__mock.goto(4); return true`);
  await light("green", "a delivered slide clears row 5", /slide 3 of 3/);
  await ev(`__mock.st.presenter.connected = false; return true`);
  await light("red", "row 6 (presenter not found)", /not found/);
  await runbook("row 6 (red)", "presentation-app-not-reachable");
  await ev(`__mock.st.presenter.connected = true; return true`);
  await ev(`__mock.micLost(); return true`);
  await light("red", "row 1 (mic lost)", /Can't hear the mic/);
  check("mic lost: switch back on Off", await waitFor(`return document.querySelector('[data-mode="off"]').getAttribute("aria-checked") === "true"`));
  await shot("16-simple-red-mic-lost");
  await runbook("row 1 (red)", "no-audio-device");
  await click('[data-mode="songs"]');
  await light("amber", "starting again clears row 1", /Waiting for a song/);
  await click('[data-mode="off"]');
  await light("grey", "Off stops listening", /Choose Songs/);
  check("Off called stop_listening", (await calls("stop_listening")) === 4 && !(await ev(`return __mock.st.running`)));
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
  check("Off 150 ms after Songs: stopped", await waitFor(`return __mock.calls.filter((c) => c.cmd === "stop_listening").length === 6 && !__mock.st.running`, 4000));
  await sleep(1500);
  check("Off 150 ms after Songs: still stopped, switch on Off", !(await ev(`return __mock.st.running`)) && (await calls("start_listening")) === starts + 2 && (await ev(`return document.querySelector('[data-mode="off"]').getAttribute("aria-checked")`)) === "true");
  await light("grey", "Off 150 ms after Songs: grey", /Choose Songs/);
  await runbook("row 4 (grey)", null);
  // Row 3, the voice model missing (grey, not red): its link is the model runbook (ADR-018 § 7).
  await ev(`__mock.models.forEach((m) => (m.loaded = false)); return true`);
  await light("grey", "row 3 (no model)", /Download the voice model/);
  await runbook("row 3 (grey)", "model-missing");
  await shot("17-simple-grey-model-missing");
  await ev(`__mock.models[0].loaded = true; return true`);
  await light("grey", "model back: row 4", /Choose Songs/);

  // 6. Call for help (ADR-018): every file is shown before anything is written; Save writes
  // the zip (without the words file when its box is unchecked), then opens the e-mail draft.
  const helpShown = () => ev(`return document.getElementById("help").checkVisibility()`);
  const helpFiles = () => ev(`return [...document.querySelectorAll("#help-files summary")].map((s) => s.textContent.split(" (")[0]).join("|")`);
  const helpMsg = () => ev(`return document.getElementById("help-msg").textContent`);
  const helpReady = (n) => waitFor(`return !document.getElementById("help-save").disabled && document.querySelectorAll("#help-files li").length === ${n}`);
  const ALL = "diagnostic-report.json|operator-settings.json|words-last-2-minutes.txt";
  const startsBeforeHelp = await calls("start_listening");
  await space("#s-help"); // Space presses Call for help; it must not reach the Listen shortcut
  check("Space on Call for help opens the review screen", await waitFor(`return document.getElementById("help").checkVisibility()`));
  check("Space on Call for help: no start_listening", (await calls("start_listening")) === startsBeforeHelp && !(await ev(`return __mock.st.running`)));
  check("Save waits for the files", await ev(`return document.getElementById("help-save").disabled`));
  // Tab cannot reach Simple mode's switch behind the review, so Enter on it cannot start listening.
  check("review open: Simple mode behind it is inert", await ev(`return document.getElementById("simple").inert`));
  const tab = { key: "Tab", code: "Tab", windowsVirtualKeyCode: 9, nativeVirtualKeyCode: 9 };
  let tabbedIn = 0;
  for (let i = 1; i <= 8 && !tabbedIn; i++) {
    await send("Input.dispatchKeyEvent", { type: "keyDown", ...tab });
    await send("Input.dispatchKeyEvent", { type: "keyUp", ...tab });
    if (await ev(`return document.getElementById("simple").contains(document.activeElement)`)) tabbedIn = i;
  }
  check("review open: Tab never reaches Simple mode", !tabbedIn, tabbedIn ? `after ${tabbedIn} Tab presses` : "");
  check("the review screen lists every file", (await helpReady(3)) && (await helpFiles()) === ALL, await helpFiles());
  check("each file shows its size", await ev(`return [...document.querySelectorAll("#help-files summary")].every((s) => / [(][0-9]+[.][0-9] KB[)]$/.test(s.textContent))`));
  check("the words file's content is shown", await ev(`const d = document.querySelectorAll("#help-files details")[2]; return d.open && d.querySelector("pre").checkVisibility() && /alpha bravo charlie delta/.test(d.querySelector("pre").textContent)`));
  check("the other files' content is one click away", await ev(`const d = document.querySelectorAll("#help-files details"); return !d[0].open && !d[1].open && d[0].querySelector("pre").textContent.includes("voxxa_version")`));
  check("the words box: on by default, the agreed label", await ev(`const b = document.getElementById("help-words"); return b.checked && b.closest("label").textContent.trim() === "Include the last 2 minutes of words the app heard"`));
  check("the review says nothing is sent", /Nothing is sent/.test(await helpMsg()), await helpMsg());
  check("the operator's localStorage goes to the preview", await ev(`const s = __mock.st.help.storage; return !!s && s["voxxa.firstLaunchDone"] === "1" && s["voxxa.audioDevice"] === '"Desk USB input"'`));
  check("nothing written before Save", (await calls("write_help_bundle")) === 0 && (await calls("open_help_email")) === 0);
  await shot("18-help-review");
  // The dialog stops main.js's shortcut keys wherever focus is, like the guide (Q8-10).
  const stageBeforeHelpKeys = await calls("toggle_stage_display");
  await keyOnBody(" ");
  await keyOnBody("s");
  await sleep(300);
  check("review open, focus on the page: Space starts nothing", (await calls("start_listening")) === startsBeforeHelp && !(await ev(`return __mock.st.running`)));
  check("review open, focus on the page: s opens no stage window", (await calls("toggle_stage_display")) === stageBeforeHelpKeys);
  await click("#help-cancel");
  check("Cancel closes the review and writes nothing", !(await helpShown()) && !(await ev(`return document.getElementById("simple").inert`)) && (await calls("write_help_bundle")) === 0 && (await calls("open_help_email")) === 0);
  await onlySimple("after Cancel");
  await click("#s-help");
  await helpReady(3);
  await ev(`document.activeElement.blur(); return true`);
  const esc = { key: "Escape", code: "Escape", windowsVirtualKeyCode: 27, nativeVirtualKeyCode: 27 };
  await send("Input.dispatchKeyEvent", { type: "keyDown", ...esc });
  await send("Input.dispatchKeyEvent", { type: "keyUp", ...esc });
  check("Escape closes the review and writes nothing", (await waitFor(`return !document.getElementById("help").checkVisibility()`, 2000)) && (await calls("write_help_bundle")) === 0);

  // Unchecked: the words file is left out of the zip; a slow save cannot be pressed twice.
  await click("#s-help");
  await helpReady(3);
  await click("#help-words");
  check("unchecked: the words file is marked as left out", await ev(`return !document.getElementById("help-words").checked && document.getElementById("help-words").closest("li").classList.contains("off")`));
  await shot("19-help-words-unchecked");
  await ev(`__mock.st.help.writeDelay = 1200; return true`);
  await click("#help-save");
  await sleep(200);
  check("saving: Save is disabled", await ev(`return document.getElementById("help-save").disabled`));
  await ev(`document.getElementById("help-save").click(); return true`); // a second press while it saves
  check("unchecked: the zip has no words file", (await waitFor(`return __mock.st.help.written.length === 1`, 4000)) && (await ev(`return __mock.st.help.written[0].join("|")`)) === "diagnostic-report.json|operator-settings.json", await ev(`return JSON.stringify(__mock.st.help.written)`));
  check("Save is sent once and asks for no words", (await calls("write_help_bundle")) === 1 && (await ev(`return __mock.calls.find((c) => c.cmd === "write_help_bundle").args.includeWords`)) === false);
  check("after saving: the e-mail draft and folder are opened once", (await waitFor(`return __mock.st.help.opened === 1`)) && (await ev(`const c = __mock.calls.map((c) => c.cmd); return c.lastIndexOf("open_help_email") > c.lastIndexOf("write_help_bundle")`)));
  check("after saving: the path and what to do next are shown", (await helpMsg()).startsWith("Saved C:\\Users\\booth\\Desktop\\voxxa-help-20261119-101500.zip. Your e-mail program opens now: type your tech lead's address, drag that file"), await helpMsg());
  check("after saving: only Close is left", !(await visible("#help-save")) && (await ev(`return document.getElementById("help-cancel").textContent`)) === "Close" && (await helpFiles()) === "");
  await shot("20-help-saved");
  await click("#help-cancel");
  await ev(`__mock.st.help.writeDelay = 200; return true`);

  // Checked (the default): the zip has the words file.
  await click("#s-help");
  await helpReady(3);
  await click("#help-save");
  check("checked: the zip has all three files", (await waitFor(`return __mock.st.help.written.length === 2`, 3000)) && (await ev(`return __mock.st.help.written[1].join("|")`)) === ALL);
  check("checked: Save asks for the words", (await ev(`return __mock.calls.filter((c) => c.cmd === "write_help_bundle")[1].args.includeWords`)) === true);
  await waitFor(`return __mock.st.help.opened === 2`);
  await click("#help-cancel");

  // A refused save: the reason is shown, Save works again, and no e-mail is opened.
  await ev(`__mock.st.help.writeError = "No Desktop folder"; return true`);
  await click("#s-help");
  await helpReady(3);
  await click("#help-save");
  check("refused save: the reason is shown", await waitFor(`return /couldn't save the file: No Desktop folder/.test(document.getElementById("help-msg").textContent)`, 3000), await helpMsg());
  check("refused save: Save works again, no e-mail", !(await ev(`return document.getElementById("help-save").disabled`)) && (await ev(`return __mock.st.help.opened`)) === 2);
  await ev(`__mock.st.help.writeError = null; return true`);
  await click("#help-cancel");

  // The e-mail step fails: the backend opens the folder first, so the message says the folder is
  // open unless the error starts with "folder: " (ADR-018 § 5).
  for (const [err, want] of [["no mail program", /Its folder is open, but Voxxa couldn't open your e-mail program \(no mail program\)\. Send that file/],
    ["folder: no Explorer", /Voxxa couldn't open its folder \(folder: no Explorer\)\. Send that file/]]) {
    await ev(`__mock.st.help.openError = ${JSON.stringify(err)}; return true`);
    await click("#s-help");
    await helpReady(3);
    await click("#help-save");
    check(`e-mail step fails (${err}): the message says what is open`, await waitFor(`return ${want}.test(document.getElementById("help-msg").textContent)`, 3000), await helpMsg());
    await click("#help-cancel");
  }
  await ev(`__mock.st.help.openError = null; return true`);

  // No session log: no words file, no checkbox, and the review says so.
  await ev(`__mock.st.help.words = null; return true`);
  await click("#s-help");
  await helpReady(2);
  check("no session log: two files and no words box", (await helpFiles()) === "diagnostic-report.json|operator-settings.json" && !(await ev(`return !!document.getElementById("help-words")`)));
  check("no session log: the review says so", /no record of words it heard/.test(await helpMsg()), await helpMsg());
  await shot("21-help-no-words");
  await click("#help-cancel");
  check("Call for help never writes without Save", (await calls("write_help_bundle")) === 5 && (await ev(`return __mock.st.help.written.length`)) === 4);
  await onlySimple("after Call for help");

  // 7. Advanced: every old panel, unchanged, one click away.
  await click("#s-advanced");
  check("Slides panel shown", await visible("#section-home"));
  for (const [sel, what] of [["#slide-view", "slide previews"], ["#listen-btn", "Listen"], ["#stage-btn", "stage display"], ["#blank-btn", "Blank"], ["#export-setlist-btn", "setlist export"]]) {
    check(`Advanced → Slides: ${what}`, await visible(sel));
  }
  await ev(`__mock.emit("song-detection", { rows: [{ song_title: "Alpha Hymn", probability: 0.7 }] }); return true`);
  check("Advanced → Slides: confidence bars", await waitFor(`return document.getElementById("detect-section").checkVisibility()`));
  await shot("22-advanced-slides");
  await click('.tab[data-tab="models"]');
  check("Advanced → Models: model list", await waitFor(`return document.querySelectorAll("#model-list .model-item").length === 3`));
  await shot("23-advanced-models");
  await click('.tab[data-tab="settings"]');
  for (const [sel, what] of [["#select-presenter", "presenter connection"], ["#discover-btn", "auto-detect"], ["#select-device", "audio device"], ["#select-language", "language"], ["#smart-silence", "smart blanking"], ["#privacy-card", "privacy counter"], ["#http-api-toggle", "HTTP API"], ["#diag-btn", "diagnostics"]]) {
    check(`Advanced → Settings: ${what}`, await visible(sel));
  }
  check("Settings shows the device chosen in the first run", await waitFor(`return document.getElementById("select-device").value === "Desk USB input"`));
  await shot("24-advanced-settings");
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
  await shot("25-advanced-loader");
  await click("#s-simple");
  check("Simple mode link goes back", (await visible("#simple")) && !(await visible("#app")));
  await onlySimple("back from Advanced");

  // 8. A webview reload mid-song: the backend keeps its setlist and keeps following, the
  // page has lost its song list. Row 9 has no position then: "Following the song."
  await load();
  await ev(`__mock.st.songs = ${JSON.stringify(setlist.setlist)}; __mock.st.running = true; __mock.st.machine = "listening"; return true`);
  await light("amber", "after a reload mid-song: listening", /Waiting for a song/);
  await ev(`__mock.goto(3); return true`);
  await light("green", "row 9 after a reload: no position", /^Following the song\.$/);
  check("after a reload: the song's title, no slide count", (await ev(`return document.getElementById("s-song").textContent + " / " + document.getElementById("s-slide").textContent`)) === "Bravo Hymn / ");
  await shot("26-simple-reload-mid-song");

  // 9. WorshipTools Presenter (ADR-019): found by its process, used at once in step 2, the two
  // amber asks word for word, no blank, and its key profile reconnected after a restart.
  // The mock presses no key; titles are placeholders.
  const WT = { kind: "keystroke", keystroke_profile: "worship_tools_presenter", host: "", port: 0, name: "WorshipTools Presenter", source: "process" };
  const WT_SAYS = "Found WorshipTools Presenter. It has no remote control, so Voxxa presses its keys for you. Keep WorshipTools Presenter in front during the service.";
  const wtConnects = () => ev(`return __mock.calls.filter((c) => c.cmd === "connect_presenter" && c.args.args.kind === "keystroke" && c.args.args.keystroke_profile === "worship_tools_presenter" && !("host" in c.args.args)).length`);
  await ev(`__mock.st.found = [${JSON.stringify(WT)}]; return true`);
  await click("#s-advanced");
  await click("#s-guide");
  await click("#fr-next");
  check("WorshipTools: step 2 names it in plain words", await waitFor(`return document.getElementById("fr-msg").textContent === ${JSON.stringify(WT_SAYS)}`, 3000), await ev(`return document.getElementById("fr-msg").textContent`));
  check("WorshipTools: used at once with its key profile, no host", (await wtConnects()) === 1 && (await ev(`return __mock.st.presenter.capabilities.can_blank`)) === false);
  check("WorshipTools: saved for the next start", JSON.stringify(JSON.parse(await ev(`return localStorage.getItem("voxxa.presenterForm")`))) === JSON.stringify({ kind: "keystroke", keystroke_profile: "worship_tools_presenter" }));
  await shot("27-first-run-worshiptools-found");
  // Found next to a network app: a pick, its button without host:port, and the same sentence.
  await ev(`__mock.st.found = [{ kind: "pro_presenter7_rest", host: "127.0.0.1", port: 1025, name: "ProPresenter on this computer", source: "localhost" }, ${JSON.stringify(WT)}]; return true`);
  await clickButton("#fr-extra", /^Search again$/);
  check("WorshipTools next to another app: a pick", (await waitFor(`return [...document.querySelectorAll("#fr-extra .discovered-row")].map((b) => b.textContent).join("|")`, 3000)) === "ProPresenter on this computer · 127.0.0.1:1025|WorshipTools Presenter");
  await clickButton("#fr-extra", /^WorshipTools Presenter$/);
  check("WorshipTools picked: the same sentence", await waitFor(`return document.getElementById("fr-msg").textContent === ${JSON.stringify(WT_SAYS)}`, 2000) && (await wtConnects()) === 2);
  await click("#fr-skip");
  await click("#s-simple");
  // The page has no song list since the reload in section 8, so row 9 has no position.
  await ev(`__mock.st.songs = ${JSON.stringify(setlist.setlist)}; __mock.st.running = true; __mock.goto(0); return true`);
  await light("green", "WorshipTools: following", /^Following the song\.$/);
  await ev(`__mock.presenterError("Bring WorshipTools Presenter to the front."); return true`);
  await light("amber", "WorshipTools: the guard's sentence, word for word", /^Bring WorshipTools Presenter to the front\.$/);
  await runbook("WorshipTools guard (amber)", null);
  // No red toast anywhere: the class main.js gives an error toast is "toast-error".
  const noRedToast = async (after) => check(`WorshipTools: no red toast after ${after}`, await ev(`return document.querySelectorAll(".toast.toast-error").length === 0`),
    await ev(`return [...document.querySelectorAll(".toast")].map((t) => t.className + ": " + t.textContent).join(" | ")`));
  await noRedToast("the guard's sentence");
  await shot("28-simple-amber-bring-to-front");
  await ev(`__mock.goto(1); return true`);
  await light("green", "a delivered slide clears the guard's sentence", /^Following the song\.$/);
  // A song change, as commands::dispatch now sends it: the new song's first slide recorded and
  // told (slide-advanced), then the ask. The light shows the ask; the song line moves on.
  await ev(`__mock.goto(2); __mock.presenterError('Put "Bravo Hymn" live in Presenter.'); return true`);
  await light("amber", "WorshipTools: the song-change ask, word for word", /^Put "Bravo Hymn" live in Presenter\.$/);
  check("WorshipTools: the song line follows the new song with the ask", await waitFor(`return document.getElementById("s-song").textContent === "Bravo Hymn"`, 2000), await ev(`return document.getElementById("s-song").textContent`));
  await noRedToast("the song-change ask");
  await runbook("WorshipTools song change (amber)", null);
  check("WorshipTools: the asks' toasts are not error toasts", await ev(`const t = [...document.querySelectorAll(".toast")].filter((e) => /^(Bring WorshipTools|Put ")/.test(e.textContent));
    return t.length === 2 && t.every((e) => e.classList.contains("toast-info") && !e.classList.contains("toast-error"))`));
  await shot("29-simple-amber-put-song-live");
  await ev(`__mock.goto(3); return true`);
  await light("green", "the next slide inside the song clears the ask", /^Following the song\.$/);
  // A prayer: the backend skips the blank (no presenter-error); the light says the screen stays.
  await ev(`__mock.machine("blank_hold", true); return true`);
  await light("amber", "WorshipTools: can't clear the screen", /^Hearing talking, not a song\. Voxxa can't clear WorshipTools Presenter's screen, so the slide stays up\.$/);
  await shot("30-simple-amber-cannot-blank");
  // Advanced, the operator's song chip and Next with WorshipTools: the backend's ask and the guard's
  // sentence come back as the command's error. Shown bare as info, never "Failed to jump: …".
  await click("#s-advanced");
  await ev(`const input = document.getElementById("file-input"); const dt = new DataTransfer();
    dt.items.add(new File([${JSON.stringify(JSON.stringify(setlist))}], "placeholder.json", { type: "application/json" }));
    input.files = dt.files; input.dispatchEvent(new Event("change")); return true`);
  check("Advanced: the song strip shows the three songs", await waitFor(`return document.querySelectorAll("#song-strip .song-chip").length === 3`, 3000));
  await ev(`__mock.st.askOnSongChange = true; return true`);
  await click('#song-strip .song-chip[data-song-idx="2"]');
  const toastIs = (text) => ev(`const t = [...document.querySelectorAll(".toast")].filter((e) => e.textContent === ${JSON.stringify(text)}); return t.length > 0 && t.every((e) => e.classList.contains("toast-info"))`);
  check("song chip: the ask is an info toast, word for word", await waitFor(`return [...document.querySelectorAll(".toast")].some((e) => e.textContent === 'Put "Charlie Hymn" live in Presenter.')`, 2000) && (await toastIs('Put "Charlie Hymn" live in Presenter.')));
  check("song chip: the strip follows the new song with the ask", await waitFor(`const a = document.querySelector("#song-strip .song-chip.active"); return !!a && a.dataset.songIdx === "2"`, 2000));
  await ev(`__mock.st.stepError = "Bring WorshipTools Presenter to the front."; return true`);
  await click("#next-btn");
  check("Next: the guard's sentence is an info toast, word for word", await waitFor(`return [...document.querySelectorAll(".toast")].some((e) => e.textContent === "Bring WorshipTools Presenter to the front.")`, 2000) && (await toastIs("Bring WorshipTools Presenter to the front.")));
  check("song chip and Next: no \"failed\" toast", await ev(`return ![...document.querySelectorAll(".toast")].some((e) => /Failed to jump|slide failed/.test(e.textContent))`));
  await noRedToast("the song chip and Next");
  await shot("31-advanced-worshiptools-asks-as-info");
  await ev(`__mock.st.stepError = null; __mock.st.askOnSongChange = false; return true`);
  await click("#s-simple");
  // A restart: the saved WorshipTools profile is reconnected (the backend starts on universal keys).
  await load();
  check("restart: the WorshipTools profile is reconnected", (await waitFor(`return __mock.calls.some((c) => c.cmd === "connect_presenter" && c.args.args.keystroke_profile === "worship_tools_presenter")`)) && (await ev(`return __mock.st.presenter.capabilities.can_blank`)) === false);
  // The universal keystroke form is not: the backend already starts on it.
  await ev(`localStorage.setItem("voxxa.presenterForm", JSON.stringify({ kind: "keystroke", keystroke_profile: "universal" })); return true`);
  await load();
  await sleep(500);
  check("restart: a universal keystroke form is not reconnected", (await calls("connect_presenter")) === 0);
  // Advanced → Auto-detect: the WorshipTools row sets the keystroke driver and its own key
  // profile (main.js useDiscovered), so Connect never sends universal keys with no guard.
  // The selects start elsewhere, so only the row's click can set them.
  await ev(`__mock.st.found = [${JSON.stringify(WT)}]; return true`);
  await click("#s-advanced");
  await click('.tab[data-tab="settings"]');
  await ev(`const p = document.getElementById("select-presenter"); p.value = "pro_presenter7_rest"; p.dispatchEvent(new Event("change"));
    document.getElementById("select-keystroke-profile").value = "universal"; return true`);
  await click("#discover-btn");
  check("Advanced → Auto-detect: the WorshipTools row, no host:port for a process", (await waitFor(`return [...document.querySelectorAll("#discovered-list .discovered-row")].map((b) => b.querySelector(".discovered-meta").textContent).join("|")`, 3000)) === "process");
  await clickButton("#discovered-list", /^WorshipTools Presenter/);
  const advSelects = () => ev(`return document.getElementById("select-presenter").value + "/" + document.getElementById("select-keystroke-profile").value`);
  check("Advanced → Auto-detect row: keystroke with the WorshipTools key profile", (await advSelects()) === "keystroke/worship_tools_presenter", await advSelects());
  await click("#connect-presenter-btn");
  check("Advanced → Connect sends the WorshipTools key profile", await waitFor(`return __mock.calls.some((c) => c.cmd === "connect_presenter" && c.args.args.kind === "keystroke" && c.args.args.keystroke_profile === "worship_tools_presenter")`, 2000),
    await ev(`return JSON.stringify(__mock.calls.filter((c) => c.cmd === "connect_presenter").map((c) => c.args.args))`));
  await shot("32-advanced-autodetect-worshiptools");
  await click("#s-simple");

  // Updates (ADR-020): only in Advanced → Settings, only when pressed. The states are scripted.
  await load();
  check("update: no check at startup, no button in Simple mode", (await calls("check_for_update")) === 0 && !(await visible("#update-check-btn")));
  await click("#s-advanced");
  await click('.tab[data-tab="settings"]');
  check("update: Settings shows Check for updates; opening it checks nothing", (await visible("#update-check-btn")) && (await calls("check_for_update")) === 0);
  const updateIs = (text) => waitFor(`return document.getElementById("update-status").textContent === ${JSON.stringify(text)}`, 2000);
  const updateText = () => ev(`return document.getElementById("update-status").textContent`);
  const offered = () => visible("#update-confirm");
  await click("#update-check-btn");
  check("update: checking", (await updateText()) === "Checking for updates…" && (await ev(`return document.getElementById("update-check-btn").disabled`)));
  check("update: up to date", (await updateIs("Voxxa is up to date.")) && !(await offered()), await updateText());
  await shot("33-advanced-update-up-to-date");
  await ev(`__mock.st.update.error = "Stop listening before checking for updates."; return true`);
  await click("#update-check-btn");
  check("update: refused while listening, word for word, nothing offered", (await updateIs("Stop listening before checking for updates.")) && !(await offered()), await updateText());
  await shot("34-advanced-update-refused-listening");
  await ev(`__mock.st.update.error = "Could not check for updates: placeholder error"; return true`);
  await click("#update-check-btn");
  check("update: an error is shown, nothing offered", (await updateIs("Could not check for updates: placeholder error")) && !(await offered()), await updateText());
  await shot("35-advanced-update-error");
  await ev(`__mock.st.update.error = null; __mock.st.update.version = "0.2.0"; return true`);
  await click("#update-check-btn");
  check("update: available, and the question says Voxxa restarts", (await updateIs("Voxxa 0.2.0 is available.")) && (await offered())
    && (await ev(`return document.getElementById("update-question").textContent`)) === "Install Voxxa 0.2.0 now? Voxxa will close and restart.");
  check("update: nothing installed before the operator confirms", (await calls("install_update")) === 0);
  await shot("36-advanced-update-available");
  await click("#update-cancel-btn");
  check("update: Not now installs nothing", (await updateIs("Update not installed.")) && !(await offered()) && (await calls("install_update")) === 0);
  await click("#update-check-btn");
  await waitFor(`return !document.getElementById("update-confirm").hidden`, 2000);
  await click("#update-install-btn");
  check("update: Install and restart installs the confirmed version", (await updateIs("Installing Voxxa 0.2.0… Voxxa will restart.")) && (await ev(`return JSON.stringify(__mock.st.update.installed)`)) === '["0.2.0"]');
  await click("#s-simple");

  // The CSP (tauri.conf.json, sent by this server) refused nothing in the whole run, and it is enforced:
  // an inline style attribute is refused and reported.
  check("CSP: no violation in the whole run", cspLog.length === 0, JSON.stringify(cspLog));
  await ev(`const d = document.createElement("div"); d.setAttribute("style", "color: red"); document.body.appendChild(d); return true`);
  for (let i = 0; i < 30 && cspLog.length === 0; i++) await sleep(100);
  check("CSP: enforced (a planted inline style is refused)", cspLog.length === 1 && /style-src/.test(cspLog[0]), JSON.stringify(cspLog));
} catch (err) {
  check("e2e ran to the end", false, String(err && err.stack || err));
}

const failed = results.filter((r) => !r.pass).length;
fs.writeFileSync(path.join(OUT, "results.json"), JSON.stringify(results, null, 2));
console.log(`\n${results.length - failed} passed, ${failed} failed; screenshots in ${OUT}`);
await shutdown(failed ? 1 : 0);
