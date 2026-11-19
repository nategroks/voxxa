// Q8 / ADR-017: Simple mode and the first-run flow in the REAL app, guarded
// (GUI testing way (b)). Safe actions only, over the DevTools protocol:
//   never Listen, never the 20-second test, never Next/Prev/Blank/Stage, no key
//   events, never the keystroke presenter (auto-detect finds a local ProPresenter
//   mock and the flow connects to it), never the HTTP API, never "Use"/"Download"
//   a model (a failed load renames the model file aside, Q2-50).
// Device enumeration and the presenter auto-detect are allowed. Device names are
// counted, never printed.
//
// Setup (Windows; the WebView2 env var is ignored, Q2-53):
//   1. A scratch Tauri config OUTSIDE git with "identifier": "com.voxxa.qa<lane>"
//      and, on both windows, "additionalBrowserArgs": "--disable-features=msWebOOUI,
//      msPdfOOUI,msSmartScreenProtection --remote-debugging-port=9341".
//   2. npm run tauri build -- --no-bundle --config <scratch config>
//   3. python scripts/qa/mock_pro7.py 1025   (only if 1025 is free; auto-detect probes it)
//   4. Start target/release/voxxa.exe, then: node scripts/qa/simple-mode-real-app.mjs <screenshot dir>
// Stop the app and the mock by PID afterwards and delete %LOCALAPPDATA%\com.voxxa.qa<lane>.
import fs from "node:fs";
import path from "node:path";

const OUT = path.resolve(process.argv[2] || "simple-mode-real-app");
const CDP = "http://127.0.0.1:9341";
const MOCK = "http://127.0.0.1:1025";
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const results = [];
const check = (name, pass, detail = "") => {
  results.push({ name, pass: !!pass, detail });
  console.log(`${pass ? "PASS" : "FAIL"} ${name}${detail ? " :: " + detail : ""}`);
};
fs.mkdirSync(OUT, { recursive: true });

const targets = await (await fetch(`${CDP}/json`)).json();
const main = targets.find((t) => t.type === "page" && !t.url.includes("stage.html"));
const ws = new WebSocket(main.webSocketDebuggerUrl);
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
const waitFor = async (expr, ms = 6000) => { const end = Date.now() + ms; while (Date.now() < end) { const v = await ev(expr); if (v) return v; await sleep(150); } return false; };
const shot = async (name) => { const r = await send("Page.captureScreenshot", { format: "png" }); fs.writeFileSync(path.join(OUT, `${name}.png`), Buffer.from(r.result.data, "base64")); };
const visible = (sel) => ev(`const e = document.querySelector(${JSON.stringify(sel)}); return !!e && e.checkVisibility();`);
// Only these elements may be clicked: navigation and the first-run flow's safe buttons.
const SAFE = new Set(["#fr-next", "#fr-skip", "#s-advanced", "#s-simple", "#s-guide", "#s-help", ".tab[data-tab=\"models\"]", ".tab[data-tab=\"settings\"]", ".tab[data-tab=\"home\"]", "#fr-extra .discovered-row"]);
const click = (sel) => { if (!SAFE.has(sel)) throw new Error(`refusing to click ${sel}`); return ev(`document.querySelector(${JSON.stringify(sel)}).click(); return true;`); };
const running = () => ev(`return (await window.__TAURI__.core.invoke("get_status")).is_running`);

try {
  await send("Runtime.enable");
  check("guard: not listening at start", (await running()) === false);
  const p0 = await ev(`return (await window.__TAURI__.core.invoke("get_presenter_info")).kind`);
  check("guard: default presenter is the keystroke driver (never dispatched to)", p0 === "keystroke", p0);

  // First run on a fresh identifier.
  check("first run opens on a fresh identifier", await waitFor(`return document.getElementById("firstrun") && document.getElementById("firstrun").checkVisibility()`));
  const n = await waitFor(`const o = document.querySelectorAll("#fr-device option"); return o.length > 1 ? o.length : 0`, 4000);
  check("step 1 lists the real input devices (Default + n)", n > 1, `${n - 1} device(s), names not printed`);
  await shot("r01-first-run-device");
  await click("#fr-next");
  check("step 2 runs auto-detect", await waitFor(`return /Looking|Found|No presentation app/.test(document.getElementById("fr-msg").textContent)`));
  const found = await waitFor(`return document.querySelector("#fr-extra .discovered-row") ? "found" : /No presentation app/.test(document.getElementById("fr-msg").textContent) ? "none" : ""`, 8000);
  await shot("r02-first-run-presenter");
  if (found === "found") {
    const label = await ev(`return document.querySelector("#fr-extra .discovered-row").textContent`);
    check("auto-detect found the local mock", /127\.0\.0\.1:1025/.test(label), label);
    await click("#fr-extra .discovered-row");
    check("connected to the mock", await waitFor(`return /Connected to/.test(document.getElementById("fr-msg").textContent)`), await ev(`return document.getElementById("fr-msg").textContent`));
    const p1 = await ev(`return (await window.__TAURI__.core.invoke("get_presenter_info"))`);
    check("presenter is now ProPresenter REST, not keystroke", p1.kind === "pro_presenter7_rest" && p1.connected, JSON.stringify(p1));
    try { const log = await (await fetch(`${MOCK}/_log`)).json(); check("mock saw the probe", log.includes("/v1/version"), JSON.stringify(log)); } catch (e) { check("mock log readable", false, String(e)); }
    await shot("r03-first-run-presenter-connected");
  } else {
    check("auto-detect: no app found (no mock on 1025)", found === "none");
  }
  await click("#fr-next");
  check("step 3 reports the model state", await waitFor(`return /model|Tiny|Base|ready|on this computer/i.test(document.getElementById("fr-msg").textContent)`), await ev(`return document.getElementById("fr-msg").textContent`));
  await shot("r04-first-run-model");
  await click("#fr-next");
  check("step 4 waits for songs (test not started)", await waitFor(`return /songs first|isn't ready/.test(document.getElementById("fr-msg").textContent)`), await ev(`return document.getElementById("fr-msg").textContent`));
  check("guard: still not listening", (await running()) === false);
  await shot("r05-first-run-test-step");
  await click("#fr-next"); // "Finish anyway"
  check("first run finished and remembered", !(await visible("#firstrun")) && (await ev(`return localStorage.getItem("voxxa.firstLaunchDone")`)) === "1");

  // The default view.
  const controls = await ev(`return [...document.querySelectorAll("#simple button, #simple a, #simple input, #simple select")]
    .filter((e) => e.checkVisibility()).map((e) => e.closest(".s-switch") ? "mode switch" : e.textContent.trim()).filter((v, i, a) => a.indexOf(v) === i)`);
  check("default view shows exactly three controls", JSON.stringify(controls) === JSON.stringify(["mode switch", "Call for help", "Advanced"]), JSON.stringify(controls));
  check("Songs + Scripture disabled", await ev(`return document.querySelector('[data-mode="scripture"]').disabled`));
  check("light is grey while off", await waitFor(`return document.getElementById("s-light").classList.contains("grey")`), await ev(`return document.getElementById("s-sentence").textContent`));
  check("old panels hidden", !(await visible("#app")) && !(await visible("#diag-overlay")));
  await shot("r06-simple-default");
  await click("#s-help");
  check("Call for help message", await visible("#s-help-msg"));
  await shot("r07-simple-call-for-help");

  // Advanced: the old window, one click away.
  await click("#s-advanced");
  check("Advanced shows the old window", (await visible("#app")) && (await visible("#setlist-loader")) && !(await visible("#simple")));
  await shot("r08-advanced-slides");
  await click(".tab[data-tab=\"models\"]");
  check("Models panel", await waitFor(`return document.querySelectorAll("#model-list .model-item").length > 0`));
  await shot("r09-advanced-models");
  await click(".tab[data-tab=\"settings\"]");
  check("Settings panel: presenter, device, HTTP API, diagnostics", (await visible("#select-presenter")) && (await visible("#select-device")) && (await visible("#http-api-toggle")) && (await visible("#diag-btn")));
  check("HTTP API stays off", !(await ev(`return (await window.__TAURI__.core.invoke("get_http_api_status")).running`)));
  await shot("r10-advanced-settings");
  await click(".tab[data-tab=\"home\"]");
  await click("#s-guide");
  check("Set-up guide re-opens the first run", await visible("#firstrun"));
  await click("#fr-skip");
  await click("#s-simple");
  check("back to Simple mode", (await visible("#simple")) && !(await visible("#app")));

  // Remembered across a reload.
  await send("Page.reload");
  await sleep(2500);
  check("first run not shown again after reload", !(await visible("#firstrun")) && (await visible("#simple")));
  await shot("r11-simple-after-reload");
  check("guard: never listened", (await running()) === false);
} catch (err) {
  check("ran to the end", false, String(err && err.stack || err));
}
ws.close();
fs.writeFileSync(path.join(OUT, "results.json"), JSON.stringify(results, null, 2));
const failed = results.filter((r) => !r.pass).length;
console.log(`\n${results.length - failed} passed, ${failed} failed; screenshots in ${OUT}`);
process.exit(failed ? 1 : 0);
