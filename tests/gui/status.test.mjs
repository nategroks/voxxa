// ADR-017's status-light table, row by row. Run: node --test "tests/gui/*.test.mjs"
import test from "node:test";
import assert from "node:assert/strict";
import { light } from "../../src/simple/status.js";

const on = { listening: true, connected: true };
const color = (s) => light(s)[0];

test("off is grey", () => {
  assert.deepEqual(light({ ...on, listening: false }), ["grey", "Off. Switch to Songs when the music starts."]);
});

test("listening with no state, or the conductor's listening state, is amber", () => {
  assert.equal(color({ ...on, machine: null }), "amber");
  assert.equal(color({ ...on, machine: "listening" }), "amber");
});

test("singing and inter-verse silence are green", () => {
  assert.equal(color({ ...on, machine: "singing" }), "green");
  assert.equal(color({ ...on, machine: "inter_verse_silence" }), "green");
});

test("blank hold is amber with its own sentence", () => {
  assert.deepEqual(light({ ...on, machine: "blank_hold" }), ["amber", "Screen blanked: what I hear doesn't match your songs."]);
});

test("a lost microphone is red, even after listening stopped", () => {
  assert.equal(color({ ...on, listening: false, micLost: true }), "red");
  assert.match(light({ ...on, machine: "singing", micLost: true })[1], /microphone/);
});

test("a refused start is red and carries the backend's message", () => {
  assert.deepEqual(light({ ...on, listening: false, startError: "No setlist loaded." }), ["red", "Couldn't start: No setlist loaded."]);
});

test("a presenter error or a lost connection is red", () => {
  const red = ["red", "Your presentation app isn't answering. Check that it is open, or connect it under Advanced."];
  assert.deepEqual(light({ ...on, machine: "singing", presenterError: "refused" }), red);
  assert.deepEqual(light({ ...on, listening: false, connected: false }), red);
});

test("rows are ordered: microphone, start, presenter, off, then the machine state", () => {
  const all = { ...on, micLost: true, startError: "e", presenterError: "p", machine: "singing" };
  assert.match(light(all)[1], /microphone/);
  assert.match(light({ ...all, micLost: false })[1], /^Couldn't start/);
  assert.match(light({ ...all, micLost: false, startError: "" })[1], /isn't answering/);
  assert.equal(color({ ...all, micLost: false, startError: "", presenterError: "", listening: false }), "grey");
});
