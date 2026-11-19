// Call for help (ADR-018): show every file of the help bundle first; save the zip to the Desktop
// only on Save, without the words file when its box is unchecked; then open the e-mail draft
// and the zip's folder. Nothing is sent anywhere.
const { invoke } = window.__TAURI__.core;
const $ = (id) => document.getElementById(id);
const WORDS = "words-last-2-minutes.txt";

const box = document.createElement("div");
box.id = "help";
box.className = "fr";
box.hidden = true;
box.innerHTML = `<div class="fr-card" role="dialog" aria-modal="true" aria-labelledby="help-title">
  <h2 id="help-title">Call for help</h2><p id="help-msg"></p><ul class="help-files" id="help-files"></ul>
  <div class="fr-actions"><button class="s-link" id="help-cancel">Cancel</button><button class="primary-btn" id="help-save">Save to Desktop</button></div>
</div>`;
document.body.appendChild(box);
const say = (text) => { $("help-msg").textContent = text; };
// Simple mode behind it is inert while it is shown: Tab cannot reach the switch, so Enter cannot start listening.
const close = () => { box.hidden = true; $("simple").inert = false; };
// While it is shown, main.js's shortcuts (Space = Listen, S = stage) never fire (as in the guide, Q8-10).
window.addEventListener("keydown", (e) => { if (box.hidden) return; e.stopPropagation(); if (e.key === "Escape") close(); }, true);
$("help-cancel").onclick = close;

// One list item per file: its name and size, and its content (text only, never HTML).
function fileItem([name, content]) {
  const li = document.createElement("li");
  const details = document.createElement("details");
  const summary = document.createElement("summary");
  const pre = document.createElement("pre");
  summary.textContent = `${name} (${(new Blob([content]).size / 1024).toFixed(1)} KB)`;
  pre.textContent = content;
  details.append(summary, pre);
  li.appendChild(details);
  if (name !== WORDS) return li;
  details.open = true; // the words are what the user decides about, so they are shown
  const label = document.createElement("label");
  label.className = "help-words";
  label.innerHTML = '<input type="checkbox" id="help-words" checked> Include the last 2 minutes of words the app heard';
  li.prepend(label);
  label.firstChild.onchange = (e) => li.classList.toggle("off", !e.target.checked);
  return li;
}

let opened = 0; // a preview that returns after Cancel or a newer open is dropped
export async function openHelp() {
  const ticket = ++opened;
  box.hidden = false;
  $("simple").inert = true;
  $("help-files").replaceChildren();
  $("help-save").hidden = false;
  $("help-save").disabled = true;
  $("help-cancel").textContent = "Cancel";
  say("Getting the files ready…");
  const storage = {};
  try { for (let i = 0; i < localStorage.length; i++) storage[localStorage.key(i)] = localStorage.getItem(localStorage.key(i)); } catch {}
  let files;
  try { files = await invoke("help_bundle_preview", { storage }); } catch (err) { return ticket === opened && say("Voxxa couldn't get the files ready: " + err); }
  if (box.hidden || ticket !== opened) return;
  $("help-files").append(...files.map(fileItem));
  say("Voxxa will save these files in one zip file on your Desktop. Nothing is sent. Look them over, then press Save." +
    (files.some((f) => f[0] === WORDS) ? "" : " Voxxa found no record of words it heard, so there is no words file."));
  $("help-save").disabled = false;
}

$("help-save").onclick = async () => {
  $("help-save").disabled = true;
  const words = $("help-words");
  say("Saving…");
  let path;
  try { path = await invoke("write_help_bundle", { includeWords: !!(words && words.checked) }); } catch (err) {
    $("help-save").disabled = false;
    return say("Voxxa couldn't save the file: " + err);
  }
  $("help-files").replaceChildren();
  $("help-save").hidden = true;
  $("help-cancel").textContent = "Close";
  say(`Saved ${path}. Your e-mail program opens now: type your tech lead's address, drag that file into the message (its folder opens too) and send it.`);
  try { await invoke("open_help_email"); } catch (err) { say(`Saved ${path}. Voxxa couldn't open your e-mail program (${err}). Send that file to your tech lead yourself.`); }
};
