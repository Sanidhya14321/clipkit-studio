import assert from "node:assert/strict";
import { test } from "node:test";
import { parseCommand, parseTime } from "./voiceCommands.ts";

test("simple commands run locally", () => {
  assert.deepEqual(parseCommand("Play."), { kind: "play" });
  assert.deepEqual(parseCommand("pause the video"), { kind: "pause" });
  assert.deepEqual(parseCommand("Undo that"), { kind: "undo" });
  assert.deepEqual(parseCommand("redo"), { kind: "redo" });
  assert.deepEqual(parseCommand("split here"), { kind: "split" });
  assert.deepEqual(parseCommand("go to 12 seconds"), {
    kind: "seek",
    seconds: 12,
  });
  assert.deepEqual(parseCommand("jump to 1:05"), { kind: "seek", seconds: 65 });
});

test("anything else becomes a comment", () => {
  assert.deepEqual(parseCommand("Make the title bigger"), {
    kind: "comment",
    text: "Make the title bigger",
  });
  assert.deepEqual(parseCommand("go to the end"), {
    kind: "comment",
    text: "go to the end",
  });
  assert.equal(parseCommand("   "), null);
});

test("times parse in several spoken forms", () => {
  assert.equal(parseTime("12.5s"), 12.5);
  assert.equal(parseTime("1 minute 5 seconds"), 65);
  assert.equal(parseTime("2 min"), 120);
  assert.equal(parseTime("soon"), null);
});
