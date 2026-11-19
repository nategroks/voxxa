// Call for help (ADR-018): shows every file of the help bundle before anything is written,
// writes the zip only when the operator confirms, then opens the e-mail draft and the folder.
// Nothing is sent: the operator attaches the file and sends the e-mail.
import { settingsExport } from "./logic.js";

const { invoke } = window.__TAURI__.core;
const $ = (id) => document.getElementById(id);
const WORDS = "words.txt";

const box = document.createElement("div");
box.id = "help";
box.className = "fr";
box.hidden = true;
box.innerHTML = `<div class="fr-card" role="dialog" aria-modal="true" aria-labelledby="help-title">
  <h2 id="help-title">Call for help</h2><p id="help-msg"></p><div class="help-files" id="help-files"></div>
  <div class="fr-actions"><button class="s-link" id="help-cancel">Cancel</button><button class="primary-btn" id="help-save">Save and open e-mail</button></div>
</div>`;
document.body.appendChild(box);
// While it is shown, main.js's shortcuts (Space = Listen, S = stage) never fire, as in the guide (Q8-10).
window.addEventListener("keydown", (e) => { if (!box.hidden) e.stopPropagation(); }, true);
$("help-cancel").onclick = () => { box.hidden = true; };

let files = [], words = null, folder = "";
const say = (text) => { $("help-msg").textContent = text; };
function show(name, text) {
  const d = document.createElement("details");
  d.innerHTML = "<summary></summary><pre></pre>";
  d.querySelector("summary").textContent = name;
  d.querySelector("pre").textContent = text; // text, never markup
  d.open = name === WORDS;
  $("help-files").appendChild(d);
}

export async function openHelp() {
  box.hidden = false;
  $("help-files").innerHTML = "";
  $("help-save").hidden = true;
  $("help-cancel").textContent = "Cancel";
  say("Getting the help file ready…");
  let report, preview;
  try {
    [report, preview] = await Promise.all([invoke("generate_diagnostic_report"), invoke("help_preview")]);
  } catch (err) { return say("Couldn't get the help file ready: " + err); }
  files = [["report.json", JSON.stringify(report, null, 2)], ["settings.json", JSON.stringify(settingsExport(localStorage), null, 2)]];
  ({ words, folder } = preview);
  say(`Nothing is saved or sent yet. Voxxa will save these files as one zip file in ${folder}, then open an e-mail for you to attach it to.`);
  for (const [name, text] of files) show(name, text);
  if (words) {
    $("help-files").insertAdjacentHTML("beforeend", '<label class="help-check"><input type="checkbox" id="help-words" checked> Include the last 2 minutes of words the app heard</label>');
    show(WORDS, words);
  } else {
    $("help-files").insertAdjacentHTML("beforeend", "<p>No recent words were recorded, so there is no words file.</p>");
  }
  $("help-save").hidden = false;
}

$("help-save").onclick = async () => {
  $("help-save").disabled = true;
  const chosen = words && $("help-words").checked ? [...files, [WORDS, words]] : files; // unchecked: no words file at all
  let name;
  try {
    name = (await invoke("write_help_bundle", { files: chosen })).split(/[\\/]/).pop();
    $("help-files").innerHTML = "";
    $("help-save").hidden = true;
    $("help-cancel").textContent = "Close";
    say(`Saved ${name} in ${folder}. Attach it to the e-mail that opens, then send it.`);
  } catch (err) { say("Couldn't save the help file: " + err); }
  $("help-save").disabled = false;
  if (name) invoke("open_help_email", { file: name }).catch((err) => say(`Saved ${name} in ${folder}. The e-mail didn't open (${err}): attach the file to an e-mail yourself.`));
};
