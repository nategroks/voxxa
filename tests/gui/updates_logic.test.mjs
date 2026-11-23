// The Updates group's pure logic (ADR-020 § 1). Run: node --test tests/gui/updates_logic.test.mjs
import test from "node:test";
import assert from "node:assert/strict";
import { afterError, afterOutcome, updateView, IDLE, LISTENING } from "../../src/updates-logic.js";

const ids = (v) => v.buttons.map(([id]) => id);

test("idle: one button, and it says when Voxxa checks", () => {
  const v = updateView({ state: "idle" });
  assert.deepEqual(ids(v), ["update-check"]);
  assert.equal(v.text, IDLE);
  assert.match(v.text, /only when you press/);
});

test("checking and installing show no button to press twice", () => {
  assert.deepEqual(ids(updateView({ state: "checking" })), []);
  assert.deepEqual(ids(updateView({ state: "installing", version: "0.2.0" })), []);
  assert.match(updateView({ state: "installing", version: "0.2.0" }).text, /start again by itself/);
});

test("each backend outcome maps to its state", () => {
  assert.deepEqual(afterOutcome({ status: "listening" }), { status: "listening", state: "listening" });
  const up = updateView(afterOutcome({ status: "up_to_date", current: "0.1.0" }));
  assert.equal(up.text, "Voxxa 0.1.0 is up to date.");
  assert.equal(up.kind, "ok");
  const av = updateView(afterOutcome({ status: "available", current: "0.1.0", version: "0.2.0", notes: null }));
  assert.equal(av.text, "Voxxa 0.2.0 is available. You have 0.1.0.");
  assert.deepEqual(ids(av), ["update-install", "update-check"]);
});

test("refused while listening: the sentence, word for word, not an error", () => {
  const v = updateView(afterOutcome({ status: "listening" }));
  assert.equal(v.text, "Stop listening first. Voxxa never checks for updates while it is listening.");
  assert.equal(v.text, LISTENING);
  assert.equal(v.kind, "info");
});

test("install asks first and says Voxxa will restart", () => {
  const v = updateView({ state: "confirm", version: "0.2.0", current: "0.1.0" });
  assert.equal(v.text, "Install Voxxa 0.2.0 now? Voxxa will close, install the update and start again.");
  assert.deepEqual(v.buttons, [["update-confirm", "Install and restart"], ["update-cancel", "Not now"]]);
});

test("errors: a network failure says what to do; other errors are shown as they are", () => {
  const net = updateView(afterError("Could not check for updates: error sending request"));
  assert.equal(net.kind, "error");
  assert.equal(net.text, "Could not check for updates: error sending request. Check the internet connection and try again.");
  assert.deepEqual(ids(net), ["update-check"]);
  assert.equal(updateView(afterError("Could not download the update: timed out.")).text, "Could not download the update: timed out. Check the internet connection and try again.");
  assert.equal(updateView(afterError("The update on offer is now 0.3.0, not 0.2.0. Check again.")).text, "The update on offer is now 0.3.0, not 0.2.0. Check again.");
  assert.equal(updateView(afterError(undefined)).text, "Something went wrong.");
});

test("an unknown answer is an error, never an install button", () => {
  for (const odd of [null, {}, { status: "install_now" }, "available"]) {
    const v = updateView(afterOutcome(odd));
    assert.equal(v.kind, "error", JSON.stringify(odd));
    assert.ok(!ids(v).includes("update-install"));
  }
});
