// Scripted GUI check for the Q2 fixes, over the Chrome DevTools Protocol.
// It drives the real app against a ProPresenter 7 REST mock: no microphone,
// no keystroke driver. Node 22+ (global fetch and WebSocket).
//
// Setup (Windows; WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS is ignored by wry):
//   1. Add "additionalBrowserArgs": "--remote-debugging-port=9222" to both
//      windows in src-tauri/tauri.conf.json, `cargo build`, copy
//      target/debug/voxxa.exe aside, and restore tauri.conf.json.
//   2. Start the mock (`python scripts/qa/mock_pro7.py 18025`), the dev
//      server (`npx vite --port 5173 --strictPort`) and the copied exe.
//   3. node scripts/qa/cdp-smoke.mjs <repo root> [results.json]
// Stop what you started by PID afterwards.
//
// Q2-18: with the local +1 in the Next handler put back, "Next x2 -> preview
// on slide 3" fails (the preview shows slide 4).
import fs from "node:fs";
import net from "node:net";

const ROOT = process.argv[2];
const MOCK = "http://127.0.0.1:18025";
const results = [];
const ok = (name, pass, detail = "") => {
  results.push({ name, pass, detail });
  console.log(`${pass ? "PASS" : "FAIL"} ${name}${detail ? " :: " + detail : ""}`);
};
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function target(match) {
  const list = await (await fetch("http://127.0.0.1:9222/json")).json();
  return list.find((t) => t.type === "page" && match(t.url));
}

function session(wsUrl) {
  const ws = new WebSocket(wsUrl);
  let id = 0;
  const pending = new Map();
  ws.onmessage = (m) => {
    const msg = JSON.parse(m.data);
    if (msg.id && pending.has(msg.id)) {
      pending.get(msg.id)(msg);
      pending.delete(msg.id);
    }
  };
  const ready = new Promise((r) => (ws.onopen = r));
  const send = async (method, params = {}) => {
    await ready;
    const i = ++id;
    ws.send(JSON.stringify({ id: i, method, params }));
    return new Promise((r) => pending.set(i, r));
  };
  const ev = async (expr) => {
    const r = await send("Runtime.evaluate", {
      expression: `(async () => { ${expr} })()`,
      awaitPromise: true,
      returnByValue: true,
    });
    if (r.result?.exceptionDetails) {
      return { error: r.result.exceptionDetails.exception?.description || JSON.stringify(r.result.exceptionDetails) };
    }
    return { value: r.result?.result?.value };
  };
  return { ev, close: () => ws.close() };
}

const mockLog = async () => (await (await fetch(`${MOCK}/_log`)).json());
const mockSelect = (name) => fetch(`${MOCK}/_select?name=${encodeURIComponent(name)}`, { method: "POST" });

const mainT = await target((u) => !u.includes("stage.html"));
const stageT = await target((u) => u.includes("stage.html"));
const main = session(mainT.webSocketDebuggerUrl);

// Q2-09: listen() resolves in both windows.
let r = await main.ev(`const un = await window.__TAURI__.event.listen('x', () => {}); un(); return 'resolved';`);
ok("Q2-09 main: event.listen resolves", r.value === "resolved", r.error || r.value);
if (stageT) {
  const stage = session(stageT.webSocketDebuggerUrl);
  r = await stage.ev(`const un = await window.__TAURI__.event.listen('x', () => {}); un(); return 'resolved';`);
  ok("Q2-09 stage: event.listen resolves", r.value === "resolved", r.error || r.value);
  stage.close();
} else {
  ok("Q2-09 stage: target found", false, "no stage.html target");
}

// Presenter -> local Pro7 REST mock (never the keystroke driver).
r = await main.ev(`return await window.__TAURI__.core.invoke('connect_presenter', { args: { kind: 'pro_presenter7_rest', host: '127.0.0.1', port: 18025 } });`);
ok("connect Pro7 REST mock", !r.error && r.value?.connected, r.error || JSON.stringify(r.value));

// Load setlist.example.json through the GUI's file input.
const setlist = fs.readFileSync(`${ROOT}/setlist.example.json`, "utf8");
r = await main.ev(`
  const input = document.getElementById('file-input');
  const dt = new DataTransfer();
  dt.items.add(new File([${JSON.stringify(setlist)}], 'setlist.example.json', { type: 'application/json' }));
  input.files = dt.files;
  input.dispatchEvent(new Event('change'));
  await new Promise(r => setTimeout(r, 800));
  return document.querySelectorAll('#song-strip .song-chip').length;`);
ok("setlist loaded (3 chips)", r.value === 3, r.error || String(r.value));

// Q2-18 + Q2-05: chip Amazing Grace (selected in the mock), then Next twice.
await mockSelect("Amazing Grace");
r = await main.ev(`
  document.querySelectorAll('#song-strip .song-chip')[0].click();
  await new Promise(r => setTimeout(r, 800));
  return document.getElementById('current-slide-text').textContent;`);
ok("chip -> operator preview shows Amazing Grace v1", (r.value || "").startsWith("Amazing grace"), r.error || r.value);
let log = await mockLog();
ok("Q2-05 chip sends slide_in_song 0 after confirming", log.includes("/v1/presentation/slide_index") && log.includes("/v1/presentation/active/0/trigger"), JSON.stringify(log));
await fetch(`${MOCK}/_clear`, { method: "POST" });
r = await main.ev(`
  const next = document.getElementById('next-btn');
  next.click(); await new Promise(r => setTimeout(r, 600));
  next.click(); await new Promise(r => setTimeout(r, 800));
  return [document.getElementById('current-slide-text').textContent, document.getElementById('slide-counter').textContent];`);
log = await mockLog();
const triggers = log.filter((p) => p.includes("/trigger"));
ok("Q2-18 Next x2 -> preview on slide 3", (r.value?.[0] || "").startsWith("Through many dangers"), r.error || JSON.stringify(r.value));
ok("Q2-18 Next x2 -> exactly two slide requests", triggers.length === 2 && triggers[1] === "/v1/presentation/active/2/trigger", JSON.stringify(log));

// Q2-05: song change while the app has another presentation selected.
await fetch(`${MOCK}/_clear`, { method: "POST" });
r = await main.ev(`
  document.querySelectorAll('#song-strip .song-chip')[1].click();
  await new Promise(r => setTimeout(r, 800));
  return Array.from(document.querySelectorAll('.toast')).map(t => t.textContent).join(' | ');`);
log = await mockLog();
ok("Q2-05 unconfirmed song change blanks, no trigger", log.includes("/v1/clear/layer/slide") && !log.some((p) => p.includes("/trigger")), JSON.stringify(log) + " toast=" + (r.value || r.error));
await mockSelect("Be Thou My Vision");
r = await main.ev(`document.querySelectorAll('#song-strip .song-chip')[1].click(); await new Promise(r => setTimeout(r, 800)); return 1;`);
log = await mockLog();
ok("Q2-05 confirmed song change -> trigger 0 (not global 4)", log.includes("/v1/presentation/active/0/trigger") && !log.includes("/v1/presentation/active/4/trigger"), JSON.stringify(log));

// Q2-01: malformed PDFs return an error and the app stays alive.
for (const name of ["unknown-font.pdf", "mediabox-3.pdf"]) {
  const b64 = fs.readFileSync(`${ROOT}/src-tauri/tests/fixtures/pdf/${name}`).toString("base64");
  r = await main.ev(`try { await window.__TAURI__.core.invoke('parse_song_bytes', { bytesB64: '${b64}', format: 'pdf', fallbackTitle: 'x' }); return 'parsed?'; } catch (e) { return 'err: ' + e; }`);
  ok(`Q2-01 ${name} -> error`, (r.value || "").startsWith("err:"), r.error || r.value);
  await sleep(300);
  r = await main.ev(`const s = await window.__TAURI__.core.invoke('get_status'); return 'alive ' + s.total_slides;`);
  ok(`Q2-01 app alive after ${name}`, (r.value || "").startsWith("alive"), r.error || r.value);
}
r = await main.ev(`
  const b64 = '${fs.readFileSync(`${ROOT}/src-tauri/tests/fixtures/pdf/three-pages.pdf`).toString("base64")}';
  const s = await window.__TAURI__.core.invoke('parse_song_bytes', { bytesB64: b64, format: 'pdf', fallbackTitle: 'x' });
  return s.slides.length;`);
ok("Q2-06 3-page PDF -> 3 slides via the command", r.value === 3, r.error || String(r.value));

// Q2-07: inline edit keeps the position.
r = await main.ev(`
  const before = await window.__TAURI__.core.invoke('get_status');
  await window.__TAURI__.core.invoke('update_slide_text', { globalIndex: before.current_slide, text: 'Be Thou my Vision O Lord of my heart' });
  const after = await window.__TAURI__.core.invoke('get_status');
  return [before.current_slide, after.current_slide, after.song_title];`);
ok("Q2-07 update_slide_text keeps current slide", r.value && r.value[0] === r.value[1] && r.value[0] !== null, r.error || JSON.stringify(r.value));

// Q2-04: Stage Display does not take focus.
r = await main.ev(`return await window.__TAURI__.core.invoke('toggle_stage_display');`);
await sleep(800);
const stageT2 = await target((u) => u.includes("stage.html"));
if (stageT2) {
  const stage = session(stageT2.webSocketDebuggerUrl);
  const f = await stage.ev(`return document.hasFocus();`);
  ok("Q2-04 stage shown without focus (known residual on Windows, ticketed)", r.value === true && f.value === false, `visible=${r.value} stageFocus=${f.value}`);
  stage.close();
}
await main.ev(`return await window.__TAURI__.core.invoke('toggle_stage_display');`);

// Q2-13: HTTP API.
r = await main.ev(`return await window.__TAURI__.core.invoke('start_http_api', { port: 17575, token: null });`);
const token = r.value;
ok("Q2-13 blank token -> generated 32-hex", /^[0-9a-f]{32}$/.test(token || ""), r.error || token);
const raw = (req) =>
  new Promise((res) => {
    const s = net.connect(17575, "127.0.0.1", () => s.write(req));
    let d = "";
    s.on("data", (c) => (d += c));
    s.on("end", () => res(d.split("\r\n")[0]));
    s.on("error", (e) => res("error " + e.message));
  });
const auth = `Authorization: Bearer ${token}\r\n`;
const st = await raw(`GET /api/v1/state HTTP/1.1\r\nHost: 127.0.0.1:17575\r\n${auth}Connection: close\r\n\r\n`);
ok("Q2-13 right token -> 200", st.includes("200"), st);
const noTok = await raw(`GET /api/v1/state HTTP/1.1\r\nHost: 127.0.0.1:17575\r\nConnection: close\r\n\r\n`);
ok("Q2-13 no token -> 401", noTok.includes("401"), noTok);
const evil = await raw(`GET /api/v1/state HTTP/1.1\r\nHost: evil.example:17575\r\n${auth}Connection: close\r\n\r\n`);
ok("Q2-13 foreign Host with token -> 403", evil.includes("403"), evil);
await fetch(`${MOCK}/_clear`, { method: "POST" });
const csrf = await raw(`POST /api/v1/blank HTTP/1.1\r\nHost: 127.0.0.1:17575\r\nOrigin: https://evil.example\r\nContent-Type: text/plain\r\nContent-Length: 1\r\nConnection: close\r\n\r\nx`);
log = await mockLog();
ok("Q2-13 cross-origin POST /blank -> 403, presenter untouched", csrf.includes("403") && log.length === 0, csrf + " " + JSON.stringify(log));
const blank = await raw(`POST /api/v1/blank HTTP/1.1\r\nHost: localhost:17575\r\n${auth}Content-Length: 0\r\nConnection: close\r\n\r\n`);
ok("Q2-13 token POST /blank -> 204", blank.includes("204"), blank);
await main.ev(`return await window.__TAURI__.core.invoke('stop_http_api');`);

main.close();
fs.writeFileSync(process.argv[3] || "smoke-results.json", JSON.stringify(results, null, 2));
console.log(`${results.filter((x) => x.pass).length}/${results.length} passed`);
