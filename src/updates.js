// Advanced > Settings > Updates (ADR-020 § 1). Runs only when the operator presses a button:
// nothing here checks at startup or on a timer. It calls Voxxa's own commands; the webview has
// no updater permission. The backend refuses while Voxxa is listening, before any network call.
import { afterError, afterOutcome, updateView } from "./updates-logic.js";

const { invoke } = window.__TAURI__.core;
const text = document.getElementById("update-text");
const actions = document.getElementById("update-actions");
let state = { state: "idle" };

function render() {
  const view = updateView(state);
  text.textContent = view.text;
  text.className = `helper-sub update-${view.kind}`;
  actions.replaceChildren(...view.buttons.map(([id, label]) => {
    const b = document.createElement("button");
    b.id = id;
    b.textContent = label;
    b.className = id === "update-install" || id === "update-confirm" ? "primary-btn" : "secondary-btn";
    b.addEventListener("click", HANDLERS[id]);
    return b;
  }));
}

async function run(busy, cmd, args) {
  state = busy;
  render();
  try { state = afterOutcome(await invoke(cmd, args)); } catch (err) { state = afterError(err); }
  render();
}

const HANDLERS = {
  "update-check": () => run({ state: "checking" }, "check_for_update"),
  "update-install": () => { state = { ...state, state: "confirm" }; render(); },
  "update-cancel": () => { state = { ...state, state: "available" }; render(); },
  "update-confirm": () => run({ ...state, state: "installing" }, "install_update", { version: state.version }),
};

if (text && actions) render();
