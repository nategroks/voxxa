// Q10 / ADR-020 § 4: the strict CSP in the REAL app, guarded (GUI testing way (b)).
// Reloads the main window and the hidden stage window with a CSP-violation listener installed,
// walks Simple mode, the first-run Skip, Advanced and its tabs, Call for help's review screen
// (Cancel, nothing written), and shows the Updates group. Then it counts CSP violations from
// the DevTools log and from the page's securitypolicyviolation events: there must be none.
// Last, it plants one inline style to prove the CSP is enforced and the counter works.
//
// Never clicked: Check for updates (it would reach the network), Listen, the 20-second test,
// Next/Previous/Blank, the stage toggle, the HTTP API, a presenter, a model, Save. No key events.
// The amber light and the info toast are drawn by giving elements the classes the app's code
// gives them (their code paths need listening, forbidden here; the mocked e2e runs them under
// the same CSP).
//
// Setup (Windows):
//   1. A scratch Tauri config OUTSIDE git: "identifier": "com.voxxa.qa<lane>", and on both
//      windows "additionalBrowserArgs": "--remote-debugging-port=<port>" (as
//      scripts/qa/simple-mode-real-app.mjs). The CSP is the committed one.
//   2. npm run tauri build -- --no-bundle --config <scratch config>
//   3. Start voxxa.exe with VOXXA_HELP_DIR=<empty scratch folder> (Call for help then reads and
//      writes nothing outside it), then: VOXXA_CDP_PORT=<port> node scripts/qa/csp-real-app.mjs <dir>
// Afterwards stop the app by PID and delete %LOCALAPPDATA%\com.voxxa.qa<lane>.
import fs from "node:fs";
import path from "node:path";

const OUT = path.resolve(process.argv[2] || "csp-real-app");
const CDP = `http://127.0.0.1:${process.env.VOXXA_CDP_PORT || 9371}`;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const results = [];
const check = (name, pass, detail = "") => {
  results.push({ name, pass: !!pass, detail });
  console.log(`${pass ? "PASS" : "FAIL"} ${name}${detail ? " :: " + detail : ""}`);
};
fs.mkdirSync(OUT, { recursive: true });

const violations = [];
const cspText = (t) => /Content Security Policy|CSP violation/i.test(t || "");
const WATCH = `document.addEventListener("securitypolicyviolation", (e) => console.error("CSP violation: " + e.violatedDirective + " " + (e.blockedURI || "inline")));`;

async function attach(target, label) {
  const ws = new WebSocket(target.webSocketDebuggerUrl);
  let id = 0;
  const pending = new Map();
  ws.onmessage = (m) => {
    const msg = JSON.parse(m.data);
    if (msg.method === "Log.entryAdded" && cspText(msg.params.entry.text)) violations.push(`${label}: ${msg.params.entry.text}`);
    if (msg.method === "Runtime.consoleAPICalled") {
      const t = (msg.params.args || []).map((a) => a.value).join(" ");
      if (cspText(t)) violations.push(`${label}: ${t}`);
    }
    if (msg.id && pending.has(msg.id)) { pending.get(msg.id)(msg); pending.delete(msg.id); }
  };
  await new Promise((r) => (ws.onopen = r));
  const send = (method, params = {}) => new Promise((r) => { const i = ++id; pending.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
  const ev = async (expr) => {
    const r = await send("Runtime.evaluate", { expression: `(async () => { ${expr} })()`, awaitPromise: true, returnByValue: true });
    if (r.result && r.result.exceptionDetails) throw new Error(r.result.exceptionDetails.exception?.description || "evaluate failed");
    return r.result.result.value;
  };
  await send("Page.enable");
  await send("Runtime.enable");
  await send("Log.enable");
  await send("Page.addScriptToEvaluateOnNewDocument", { source: WATCH });
  return { ws, send, ev };
}

const targets = await (await fetch(`${CDP}/json`)).json();
const mainT = targets.find((t) => t.type === "page" && !t.url.includes("stage.html"));
const stageT = targets.find((t) => t.type === "page" && t.url.includes("stage.html"));
const main = await attach(mainT, "main");
const stage = stageT ? await attach(stageT, "stage") : null;
const { send, ev } = main;
const waitFor = async (expr, ms = 6000) => { const end = Date.now() + ms; while (Date.now() < end) { const v = await ev(expr); if (v) return v; await sleep(150); } return false; };
const shot = async (name) => { const r = await send("Page.captureScreenshot", { format: "png" }); fs.writeFileSync(path.join(OUT, `${name}.png`), Buffer.from(r.result.data, "base64")); };
const visible = (sel) => ev(`const e = document.querySelector(${JSON.stringify(sel)}); return !!e && e.checkVisibility();`);
const SAFE = new Set(["#fr-skip", "#s-advanced", "#s-simple", "#s-help", "#help-cancel", '.tab[data-tab="models"]', '.tab[data-tab="settings"]', '.tab[data-tab="home"]']);
const click = (sel) => { if (!SAFE.has(sel)) throw new Error(`refusing to click ${sel}`); return ev(`document.querySelector(${JSON.stringify(sel)}).click(); return true;`); };
const invoke = (cmd) => ev(`return await window.__TAURI__.core.invoke(${JSON.stringify(cmd)})`);

try {
  check("guard: this is the real app, not the mock", await ev(`return !!window.__TAURI__ && !window.__mock`));
  check("guard: not listening", (await invoke("get_status")).is_running === false);
  const outbound0 = (await invoke("get_network_stats")).outbound_requests;
  check("guard: no outbound request since start (no update check at startup)", outbound0 === 0, String(outbound0));

  // Log.enable replays entries the page logged before this run (a previous run's planted style,
  // say): drop them, then reload both pages with the listener in place, so every load-time
  // violation of this run is seen.
  await sleep(300);
  violations.length = 0;
  await send("Log.clear");
  if (stage) await stage.send("Log.clear");
  await send("Page.reload", { ignoreCache: true });
  if (stage) await stage.send("Page.reload", { ignoreCache: true });
  await sleep(2500);
  await waitFor(`return !!document.getElementById("simple")`);
  check("main window loaded under the CSP", await ev(`return document.readyState === "complete" && !!document.getElementById("simple")`));
  check("stage window loaded under the CSP", !stage || (await stage.ev(`return document.readyState === "complete" && !!document.getElementById("stage-current-text")`)));
  const policy = await ev(`return [...document.querySelectorAll('meta[http-equiv="Content-Security-Policy"]')].map((m) => m.content).join(" | ")`);
  console.log(`CSP meta in the page: ${policy || "(none; sent as a header)"}`);

  if (await visible("#firstrun")) { await click("#fr-skip"); await sleep(300); }
  check("Simple mode is shown", await visible("#simple"));
  check("Simple mode has no update control", !(await visible("#update-check")));
  await shot("01-simple");

  // The amber light and an info toast, drawn with the classes the app's code uses.
  await ev(`const l = document.getElementById("s-light"); l.dataset.was = l.className; l.className = l.className.replace(/\\b(grey|amber|green|red)\\b/, "") + " amber";
    document.getElementById("s-sentence").dataset.was = document.getElementById("s-sentence").textContent;
    document.getElementById("s-sentence").textContent = "Bring WorshipTools Presenter to the front.";
    let c = document.getElementById("toast-container"); if (!c) { c = document.createElement("div"); c.id = "toast-container"; document.body.appendChild(c); }
    const t = document.createElement("div"); t.className = "toast toast-info"; t.id = "qa-toast"; t.textContent = 'Put "Alpha Hymn" live in Presenter.'; c.appendChild(t); return true`);
  // Read in the same turn: Simple mode's status poll redraws the light from the real state.
  const amber = await ev(`const l = document.getElementById("s-light"); l.className = l.className.replace(/\\b(grey|amber|green|red)\\b/, "") + " amber"; return getComputedStyle(l).backgroundColor`);
  await shot("02a-simple-amber");
  check("amber light renders (stylesheet applied)", amber === "rgb(245, 158, 11)", amber);
  await sleep(300);
  check("info toast renders (styled, on screen)", await ev(`const t = document.getElementById("qa-toast"); return t.checkVisibility() && t.getBoundingClientRect().height > 0 && getComputedStyle(t).backgroundColor !== "rgba(0, 0, 0, 0)"`),
    await ev(`return getComputedStyle(document.getElementById("qa-toast")).backgroundColor`));
  await shot("02-simple-amber-and-toast");
  await ev(`document.getElementById("qa-toast").remove(); const l = document.getElementById("s-light"); l.className = l.dataset.was; const s = document.getElementById("s-sentence"); s.textContent = s.dataset.was; return true`);

  // Call for help's review screen (reads in memory; Cancel writes nothing).
  await click("#s-help");
  check("Call for help's dialog renders", await waitFor(`return document.getElementById("help-title") && document.getElementById("help-title").checkVisibility() && document.querySelectorAll("#help-files li").length > 0`, 8000));
  await shot("03-call-for-help-review");
  await click("#help-cancel");
  await sleep(300);

  // Advanced, every tab, and the Updates group (seen, never clicked).
  await click("#s-advanced");
  for (const tab of ["home", "models", "settings"]) { await click(`.tab[data-tab="${tab}"]`); await sleep(500); }
  await ev(`document.getElementById("update-group").scrollIntoView(); return true`);
  await sleep(300);
  check("Advanced > Settings shows Check for updates", (await visible("#update-check")) && (await ev(`return document.getElementById("update-check").textContent`)) === "Check for updates");
  check("Updates says when Voxxa checks", /only when you press Check for updates/.test(await ev(`return document.getElementById("update-text").textContent`)));
  await shot("04-advanced-settings-updates");
  await click("#s-simple");
  const outbound1 = (await invoke("get_network_stats")).outbound_requests;
  check("still no outbound request (nothing checked for updates)", outbound1 === 0, String(outbound1));

  check("CSP: zero violations in the main and stage windows", violations.length === 0, violations.join(" | "));

  // The CSP is enforced: a planted inline style is refused and reported, then left out.
  const before = violations.length;
  await ev(`document.body.insertAdjacentHTML("beforeend", '<div id="qa-planted" style="color: rgb(255, 0, 0)">x</div>'); return true`);
  await sleep(600);
  const color = await ev(`const e = document.getElementById("qa-planted"); const c = getComputedStyle(e).color; e.remove(); return c`);
  check("CSP enforced in the real app: a planted inline style is refused and reported", color !== "rgb(255, 0, 0)" && violations.length > before, `${color} | ${violations.slice(before).join(" | ")}`);
} catch (err) {
  check("ran to the end", false, String(err && err.stack || err));
}
const failed = results.filter((r) => !r.pass).length;
fs.writeFileSync(path.join(OUT, "results.json"), JSON.stringify(results, null, 2));
console.log(`\n${results.length - failed} passed, ${failed} failed; screenshots in ${OUT}`);
main.ws.close();
if (stage) stage.ws.close();
process.exit(failed ? 1 : 0);
