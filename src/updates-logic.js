// Check for updates (ADR-020 § 1): what Advanced > Settings > Updates shows in each state.
// Pure (no DOM, no Tauri), so tests/gui runs it in Node.

export const IDLE = "Voxxa looks for a new version only when you press Check for updates, never while it is listening.";
export const LISTENING = "Stop listening first. Voxxa never checks for updates while it is listening.";
const CHECK = ["update-check", "Check for updates"];

// The group for a state: { kind, text, buttons: [[id, label], ...] }. kind is busy, info, ok or error.
export function updateView(s) {
  switch (s.state) {
    case "checking":
      return { kind: "busy", text: "Checking for updates…", buttons: [] };
    case "listening":
      return { kind: "info", text: LISTENING, buttons: [CHECK] };
    case "up_to_date":
      return { kind: "ok", text: `Voxxa ${s.current} is up to date.`, buttons: [CHECK] };
    case "available":
      return { kind: "info", text: `Voxxa ${s.version} is available. You have ${s.current}.`, buttons: [["update-install", `Install ${s.version}…`], CHECK] };
    case "confirm":
      return { kind: "info", text: `Install Voxxa ${s.version} now? Voxxa will close, install the update and start again.`,
        buttons: [["update-confirm", "Install and restart"], ["update-cancel", "Not now"]] };
    case "installing":
      return { kind: "busy", text: `Downloading and installing Voxxa ${s.version}. Voxxa will start again by itself.`, buttons: [] };
    case "error":
      return { kind: "error", text: s.error, buttons: [CHECK] };
    default:
      return { kind: "info", text: IDLE, buttons: [CHECK] };
  }
}

// The state after check_for_update or install_update returned (Rust's updater::Outcome).
export function afterOutcome(outcome) {
  const known = ["listening", "up_to_date", "available"];
  if (!outcome || !known.includes(outcome.status)) return afterError(`Unexpected answer: ${JSON.stringify(outcome)}`);
  return { ...outcome, state: outcome.status };
}

// The state after a command failed. A network failure gets the one thing the operator can do.
export function afterError(err) {
  const text = String(err || "Something went wrong.").trim();
  return { state: "error", error: /^Could not/.test(text) ? `${text.replace(/\.$/, "")}. Check the internet connection and try again.` : text };
}
