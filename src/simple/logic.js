// Pure Simple-mode logic (ADR-017): no DOM, no Tauri, so tests/gui runs it in Node.

// "ProPresenter 7.9+ (REST API)" -> "ProPresenter 7.9+".
export const appName = (p) => ((p && p.display_name) || "Your presentation app").replace(/\s*\(.*\)\s*$/, "");

// The status light: ADR-017 § 2, first matching row wins.
export function statusLight(s) {
  const following = s.state === "singing" || s.state === "inter_verse_silence";
  if (s.micLost) return ["red", "Can't hear the mic. Check the cable, then choose Songs again."];
  if (!s.running && !s.setlist) return ["grey", "Off. Load this week's songs in Advanced first."];
  if (!s.running && !s.model) return ["grey", "Off. Download the voice model in Advanced first."];
  if (!s.running) return ["grey", "Off. Choose Songs to start following."];
  if (s.presenterError) {
    const why = typeof s.presenterError === "string" && s.presenterError.trim();
    return ["red", why ? `${appName(s.presenter)} didn't take the last slide change: ${why.replace(/\.?$/, ".")}` : `${appName(s.presenter)} didn't take the last slide change. Check that it is open.`];
  }
  // Connected, and to the app last chosen: a failed reconnect leaves the keystroke default.
  const chosenOk = !s.chosen || (!!s.presenter && s.presenter.kind === s.chosen);
  if (!s.presenter || !s.presenter.connected || !chosenOk) return ["red", `${appName(chosenOk ? s.presenter : null)} not found. Voxxa can't change slides.`];
  if (s.state === "blank_hold") return ["amber", "Hearing talking, not a song. The screen stays blank."];
  if (following && s.isBlank) return ["amber", "The screen is blank. It comes back when the song does."];
  if (following && s.song) return ["green", `Following ${s.song.title}, slide ${s.song.slide} of ${s.song.slides}.`];
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
  return fallbackTitle ? { song: 0, songs: 0, title: fallbackTitle, slide: index + 1, slides: 0 } : null;
}

// The 20-second test (ADR-017 § 4). Counts only: no heard text is kept.
export const SOUND_RMS = 0.01;
export function testVerdict(t) {
  if (!t.started) return [false, "Listening didn't start. Read the message at the bottom of the window, then try again."];
  if (t.lost) return [false, "The microphone stopped. Check the cable and try again."];
  if (t.maxRms < SOUND_RMS) return [false, "I couldn't hear anything. Check the microphone (step 1) and its volume."];
  if (t.words < 1) return [false, "I hear sound but no words. Move the mic closer to the speakers or turn it up."];
  return [true, "I heard music and words. You're ready."];
}
