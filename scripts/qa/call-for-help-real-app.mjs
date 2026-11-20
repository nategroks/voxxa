// Q9 / ADR-018: Call for help in the REAL app, guarded (GUI testing way (b)).
// Opens Call for help up to its review screen, then writes the bundle through the real
// backend with write_help_bundle called directly over the DevTools protocol, once with the
// words file and once without. It never presses Save, so open_help_email (the e-mail draft
// and the folder) is never called; it never presses Listen, a presenter or a model button,
// and never opens a runbook (only an unknown name, which the backend refuses).
//
// Setup (Windows):
//   1. A scratch Tauri config OUTSIDE git: "identifier": "com.voxxa.qa<lane>", and on both
//      windows "additionalBrowserArgs": "--remote-debugging-port=<port>" (see
//      scripts/qa/simple-mode-real-app.mjs for the full flag list).
//   2. npm run tauri build -- --no-bundle --config <scratch config>
//   3. A scratch folder <dir> with <dir>/sessions/session-YYYYMMDD-HHMM.jsonl holding
//      placeholder words only. Start voxxa.exe with VOXXA_HELP_DIR=<dir> in its environment:
//      the zip is then written to <dir>, logs are read from <dir>/sessions, nothing is opened.
//   4. VOXXA_CDP_PORT=<port> VOXXA_HELP_DIR=<dir> node scripts/qa/call-for-help-real-app.mjs <screenshot dir>
// Afterwards stop the app by PID, delete %LOCALAPPDATA%\com.voxxa.qa<lane> and <dir>.
import fs from "node:fs";
import path from "node:path";

const OUT = path.resolve(process.argv[2] || "call-for-help-real-app");
const CDP = `http://127.0.0.1:${process.env.VOXXA_CDP_PORT || 9351}`;
const DIR = process.env.VOXXA_HELP_DIR;
if (!DIR) { console.error("set VOXXA_HELP_DIR to the scratch folder the app was started with"); process.exit(2); }
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
const waitFor = async (expr, ms = 8000) => { const end = Date.now() + ms; while (Date.now() < end) { const v = await ev(expr); if (v) return v; await sleep(150); } return false; };
const shot = async (name) => { const r = await send("Page.captureScreenshot", { format: "png" }); fs.writeFileSync(path.join(OUT, `${name}.png`), Buffer.from(r.result.data, "base64")); };
// Click only what a user could see and click.
const click = (sel) => ev(`const e = document.querySelector(${JSON.stringify(sel)}); if (!e || !e.checkVisibility()) throw new Error("not visible: ${sel}"); e.click(); return true`);
const ready = (n) => waitFor(`return !document.getElementById("help-save").disabled && document.querySelectorAll("#help-files li").length === ${n}`);

try {
  await send("Page.enable");
  await send("Runtime.enable");
  check("real app: Simple mode is up", await waitFor(`return !!document.getElementById("simple")`));
  if (await ev(`return document.getElementById("firstrun").checkVisibility()`)) await click("#fr-skip"); // Skip starts nothing
  // Placeholder settings with secrets, in this scratch profile only (deleted afterwards).
  await ev(`localStorage.setItem("voxxa.presenterForm", JSON.stringify({ kind: "open_lp_v2", host: "127.0.0.1", port: 4316, username: "booth", password: "placeholder-secret-1" }));
    localStorage.setItem("voxxa.pcoSecret", JSON.stringify("placeholder-secret-2"));
    localStorage.setItem("voxxa.pco", JSON.stringify({ application_id: "placeholder-secret-4", clientId: "placeholder-secret-5", apiKeys: ["placeholder-secret-6"] }));
    localStorage.setItem("voxxa.httpApi", JSON.stringify({ port: 7575, enabled: false, token: "placeholder-secret-3" }));
    localStorage.setItem("voxxa.recentSetlists", JSON.stringify([{ summary: "Alpha Hymn", setlist: [{ title: "Alpha Hymn", slides: [{ id: 0, text: "alpha bravo placeholder slide" }] }] }]));
    return true`);
  await shot("01-real-simple");
  const before = fs.readdirSync(DIR).filter((f) => f.endsWith(".zip")).length;
  await click("#s-help");
  check("real app: the review lists three files", await ready(3), await ev(`return document.getElementById("help-msg").textContent`));
  const files = await ev(`return [...document.querySelectorAll("#help-files summary")].map((s) => s.textContent)`);
  console.log("review screen files: " + JSON.stringify(files));
  check("real app: the words file is shown and its box is on", await ev(`const d = document.querySelectorAll("#help-files details")[2]; return d.open && document.getElementById("help-words").checked`));
  const settings = await ev(`return document.querySelectorAll("#help-files pre")[1].textContent`);
  check("real app: no secret and no slide text on the review screen", !/placeholder-secret|placeholder slide/.test(settings) && settings.includes("[redacted]") && settings.includes("[left out: whole setlists]"));
  check("real app: nothing written before Save", fs.readdirSync(DIR).filter((f) => f.endsWith(".zip")).length === before);
  await shot("02-real-review");
  // Write through the real backend (not the Save button, so no e-mail step is called).
  const withWords = await ev(`return await window.__TAURI__.core.invoke("write_help_bundle", { includeWords: true })`);
  console.log("written (box checked): " + withWords);
  check("real app: the zip is in the scratch folder", path.resolve(path.dirname(withWords)) === path.resolve(DIR));
  await click("#help-cancel");
  await sleep(1100); // a new file name each second
  await click("#s-help");
  await ready(3);
  await click("#help-words");
  await shot("03-real-review-unchecked");
  const without = await ev(`return await window.__TAURI__.core.invoke("write_help_bundle", { includeWords: false })`);
  console.log("written (box unchecked): " + without);
  const again = await ev(`try { await window.__TAURI__.core.invoke("write_help_bundle", { includeWords: true }); return "wrote"; } catch (e) { return String(e); }`);
  check("real app: a second write without a new review writes nothing", again === "Press Call for help again", again);
  // A zip stores its entry names uncompressed: look for the words file's name in each zip.
  const holdsWords = (zip) => fs.readFileSync(zip).includes("words-last-2-minutes.txt");
  check("real app: box checked, the zip holds the words file", holdsWords(withWords));
  check("real app: box unchecked, the zip has no words file", !holdsWords(without));
  const refused = await ev(`try { await window.__TAURI__.core.invoke("open_runbook", { name: "../../evil" }); return "opened"; } catch (e) { return String(e); }`);
  check("real app: open_runbook refuses an unknown name", refused === "no runbook named ../../evil", refused);
  await click("#help-cancel");
  check("real app: the review closes", !(await ev(`return document.getElementById("help").checkVisibility()`)));
  check("real app: never listening", !(await ev(`return (await window.__TAURI__.core.invoke("get_status")).is_running`)));
  fs.writeFileSync(path.join(OUT, "written.json"), JSON.stringify({ withWords, without }, null, 2));
} catch (err) {
  check("real-app run reached the end", false, String(err && err.stack || err));
}
ws.close();
const failed = results.filter((r) => !r.pass).length;
console.log(`\n${results.length - failed} passed, ${failed} failed; screenshots in ${OUT}`);
process.exit(failed ? 1 : 0);
