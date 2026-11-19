// Simple mode's pure logic (ADR-017 § 2 and § 4). Run: node --test tests/gui/simple_logic.test.mjs
// Mock data uses placeholder words only.
import test from "node:test";
import assert from "node:assert/strict";
import { appName, statusLight, songPosition, testVerdict, SOUND_RMS } from "../../src/simple/logic.js";

const pro = { connected: true, display_name: "ProPresenter 7.9+ (REST API)" };
const song = { title: "Alpha Bravo", slide: 2, slides: 6 };
// A healthy listening session; each row below changes one input.
const base = { running: true, setlist: true, model: true, state: "listening", isBlank: true, presenter: pro, presenterError: null, micLost: false, song: null };
const light = (over) => statusLight({ ...base, ...over });

test("ADR-017 table, one case per row", () => {
  const rows = [
    [1, { micLost: true }, "red", "Can't hear the mic. Check the cable, then choose Songs again."],
    [2, { running: false, setlist: false }, "grey", "Off. Load this week's songs in Advanced first."],
    [3, { running: false, model: false }, "grey", "Off. Download the voice model in Advanced first."],
    [4, { running: false }, "grey", "Off. Choose Songs to start following."],
    [5, { presenterError: "refused" }, "red", "ProPresenter 7.9+ didn't take the last slide change. Check that it is open."],
    [6, { presenter: { ...pro, connected: false } }, "red", "ProPresenter 7.9+ not found. Voxxa can't change slides."],
    [7, { state: "blank_hold" }, "amber", "Hearing talking, not a song. The screen stays blank."],
    [8, { state: "singing", isBlank: true, song }, "amber", "The screen is blank. It comes back when the song does."],
    [9, { state: "singing", isBlank: false, song }, "green", "Following Alpha Bravo, slide 2 of 6."],
    [9, { state: "inter_verse_silence", isBlank: false, song }, "green", "Following Alpha Bravo, slide 2 of 6."],
    [9, { state: "singing", isBlank: false }, "green", "Following the song."],
    [10, { state: "listening" }, "amber", "Listening. Waiting for a song from the list."],
    [10, { state: null, isBlank: false }, "amber", "Listening. Waiting for a song from the list."],
  ];
  for (const [row, over, colour, sentence] of rows) {
    assert.deepEqual(light(over), [colour, sentence], `row ${row}: ${JSON.stringify(over)}`);
  }
});

test("first matching row wins", () => {
  // A lost mic beats being off, a presenter error and a song on screen.
  assert.equal(light({ micLost: true, running: false, presenterError: "x", state: "singing", song })[0], "red");
  assert.match(light({ micLost: true, running: false })[1], /mic/);
  // Off beats every listening-only row, even with stale errors and states.
  for (const over of [{ presenterError: "x" }, { presenter: { connected: false } }, { state: "singing", isBlank: false, song }, { state: "blank_hold" }]) {
    assert.equal(light({ running: false, ...over })[0], "grey", JSON.stringify(over));
  }
  // No setlist is reported before no model.
  assert.match(light({ running: false, setlist: false, model: false })[1], /songs/);
  // A refused send beats a disconnected presenter and a song on screen.
  assert.match(light({ presenterError: "x", presenter: { connected: false }, state: "singing", isBlank: false, song })[1], /didn't take/);
  // Not connected beats blank_hold and green.
  assert.equal(light({ presenter: { connected: false }, state: "singing", isBlank: false, song })[0], "red");
  // blank_hold is amber even when is_blank is false.
  assert.equal(light({ state: "blank_hold", isBlank: false, song })[0], "amber");
  // A missing presenter counts as not connected.
  assert.deepEqual(light({ presenter: null }), ["red", "Your presentation app not found. Voxxa can't change slides."]);
});

test("every light colour is one of the four", () => {
  const states = [null, "listening", "singing", "inter_verse_silence", "blank_hold"];
  for (const running of [false, true]) for (const state of states) for (const isBlank of [false, true]) for (const micLost of [false, true]) {
    const [c, s] = light({ running, state, isBlank, micLost, song });
    assert.ok(["grey", "amber", "green", "red"].includes(c));
    assert.ok(s.length > 10 && s.endsWith("."), s);
    if (!running && !micLost) assert.equal(c, "grey");
    if (c === "green") assert.ok(running && !isBlank && (state === "singing" || state === "inter_verse_silence"));
  }
});

test("appName drops the bracketed detail", () => {
  assert.equal(appName({ display_name: "OpenLP (Web Remote v2)" }), "OpenLP");
  assert.equal(appName({ display_name: "FreeShow" }), "FreeShow");
  assert.equal(appName(undefined), "Your presentation app");
});

test("songPosition counts slides inside the song", () => {
  const songs = [
    { title: "Alpha", slides: [{}, {}, {}] },
    { title: "Bravo", slides: [{}, {}] },
    { title: "", slides: [{}] },
  ];
  assert.deepEqual(songPosition(songs, 0), { song: 1, songs: 3, title: "Alpha", slide: 1, slides: 3 });
  assert.deepEqual(songPosition(songs, 2), { song: 1, songs: 3, title: "Alpha", slide: 3, slides: 3 });
  assert.deepEqual(songPosition(songs, 3), { song: 2, songs: 3, title: "Bravo", slide: 1, slides: 2 });
  assert.deepEqual(songPosition(songs, 4), { song: 2, songs: 3, title: "Bravo", slide: 2, slides: 2 });
  assert.deepEqual(songPosition(songs, 5), { song: 3, songs: 3, title: "Song 3", slide: 1, slides: 1 });
  assert.equal(songPosition(songs, null), null);
  assert.equal(songPosition(songs, undefined), null);
  assert.equal(songPosition(songs, 6), null);
  // No setlist known to the GUI (e.g. after a reload): the backend's title only.
  assert.deepEqual(songPosition([], 4, "Charlie"), { song: 0, songs: 0, title: "Charlie", slide: 5, slides: 0 });
});

test("testVerdict: the 20-second test", () => {
  const good = { started: true, lost: false, maxRms: 0.2, words: 3 };
  assert.deepEqual(testVerdict(good), [true, "I heard music and words. You're ready."]);
  assert.match(testVerdict({ ...good, started: false })[1], /didn't start/);
  assert.match(testVerdict({ ...good, lost: true })[1], /stopped/);
  assert.match(testVerdict({ ...good, maxRms: SOUND_RMS / 2 })[1], /couldn't hear/);
  assert.equal(testVerdict({ ...good, maxRms: SOUND_RMS })[0], true);
  assert.match(testVerdict({ ...good, words: 0 })[1], /no words/);
  // Order: not started beats everything; a lost mic beats silence.
  assert.match(testVerdict({ started: false, lost: true, maxRms: 0, words: 0 })[1], /didn't start/);
  assert.match(testVerdict({ started: true, lost: true, maxRms: 0, words: 0 })[1], /stopped/);
  assert.equal(testVerdict({ started: true, lost: false, maxRms: 0, words: 5 })[0], false);
});
