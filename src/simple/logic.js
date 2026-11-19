// Pure Simple-mode logic (ADR-017): no DOM, no Tauri, so tests/gui runs it in Node.

// "ProPresenter 7.9+ (REST API)" -> "ProPresenter 7.9+".
export const appName = (p) => ((p && p.display_name) || "Your presentation app").replace(/\s*\(.*\)\s*$/, "");

// The status light: ADR-017 § 2, first matching row wins. A third item names the runbook
// the light links to, docs/runbooks/<name>.md (ADR-018).
const APP_RUNBOOK = "presentation-app-not-reachable";
export function statusLight(s) {
  const following = s.state === "singing" || s.state === "inter_verse_silence";
  if (s.micLost) return ["red", "Can't hear the mic. Check the cable, then choose Songs again.", "no-audio-device"];
  if (!s.running && !s.setlist) return ["grey", "Off. Load this week's songs in Advanced first."];
  if (!s.running && !s.model) return ["grey", "Off. Download the voice model in Advanced first.", "model-missing"];
  if (!s.running) return ["grey", "Off. Choose Songs to start following."];
  if (s.presenterError) {
    const why = typeof s.presenterError === "string" && s.presenterError.trim();
    return ["red", why ? `${appName(s.presenter)} didn't take the last slide change: ${why.replace(/\.?$/, ".")}` : `${appName(s.presenter)} didn't take the last slide change. Check that it is open.`, APP_RUNBOOK];
  }
  // Connected, and to the app last chosen: a failed reconnect leaves the keystroke default.
  const chosenOk = !s.chosen || (!!s.presenter && s.presenter.kind === s.chosen);
  if (!s.presenter || !s.presenter.connected || !chosenOk) return ["red", `${appName(chosenOk ? s.presenter : null)} not found. Voxxa can't change slides.`, APP_RUNBOOK];
  if (s.state === "blank_hold") return ["amber", "Hearing talking, not a song. The screen stays blank."];
  if (following && s.isBlank) return ["amber", "The screen is blank. It comes back when the song does."];
  if (following && s.song && s.song.slides) return ["green", `Following ${s.song.title}, slide ${s.song.slide} of ${s.song.slides}.`];
  if (following) return ["green", "Following the song."];
  return ["amber", "Listening. Waiting for a song from the list."];
}

// Global slide index -> position inside its song (1-based), or null.
export function songPosition(songs, index, fallbackTitle) {
  if (index === null || index === undefined || index < 0) return null;
  let start = 0;
  for (let i = 0; i < (songs || []).length; i++) {
    const n = songs[i].slides.length;
    if (index < start + n) {
      return { song: i + 1, songs: songs.length, title: songs[i].title || fallbackTitle || `Song ${i + 1}`, slide: index - start + 1, slides: n };
    }
    start += n;
  }
  return fallbackTitle ? { song: 0, songs: 0, title: fallbackTitle, slide: 0, slides: 0 } : null;
}

// The 20-second test (ADR-017 § 4). Counts only: no heard text is kept. Green needs a slide
// change that reached the presenter inside the window (`slide-advanced`), not only words.
export const SOUND_RMS = 0.01;
export function testVerdict(t) {
  if (!t.started) return [false, "Listening didn't start. Read the message at the bottom of the window, then try again."];
  if (t.lost) return [false, "The microphone stopped. Check the cable and try again."];
  if (t.maxRms < SOUND_RMS) return [false, "I couldn't hear anything. Check the microphone (step 1) and its volume."];
  if (t.words < 1) return [false, "I hear sound but no words. Move the mic closer to the speakers or turn it up."];
  if (!(t.advanced >= 1) && t.refused > 0) return [false, "Voxxa recognised the song, but your presentation app didn't take the slide. Click into your presentation app right after you start (and check that it is open), then try again."];
  if (!(t.advanced >= 1)) return [false, "I hear words but didn't recognise a song from the list. Play a song that is loaded."];
  return [true, "Voxxa followed the song and changed the slide. You're ready."];
}

// Call for help (ADR-018): the operator's saved settings for the bundle. A secret is redacted
// wherever it sits (a name like token, secret, password, credential, app id or key), and the
// recent setlists are left out: they hold whole songs.
const SECRET = /token|secret|passw|credential|app_?id|key(?!stroke)/i;
export function settingsExport(storage) {
  const out = {};
  for (const [k, v] of Object.entries(storage)) {
    let value;
    try { value = JSON.parse(v, (key, x) => (SECRET.test(key) ? "[redacted]" : x)); } catch { value = v; }
    out[k] = SECRET.test(k) ? "[redacted]" : k === "voxxa.recentSetlists" ? "[left out: song text]" : value;
  }
  return out;
}
