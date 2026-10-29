// Unit tests for the Brief page logic. Run with `pnpm test`.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import type { Project } from "./generated/types";
import { PACING, confirmPatch, display, pacingLevel } from "./brief.ts";

function project(): Project {
  return {
    rev: 7,
    brief: {
      fields: {
        length: { value: 15, source: "inferred", confidence: 0.8 },
        platform: { value: "youtube", source: "inferred", confidence: 0.9 },
      },
      status: "proposed",
      checkpoints: [],
      questions: [{ id: "price", text: "Show the price?", affects: ["cta"] }],
    },
  } as unknown as Project;
}

test("pacing levels read numbers and names, otherwise the middle", () => {
  assert.equal(pacingLevel(4), 4);
  assert.equal(pacingLevel("Calm"), 1);
  assert.equal(pacingLevel("brisk"), 3);
  assert.equal(pacingLevel(9), 3);
  assert.equal(pacingLevel(undefined), 3);
});

test("the pacing scale matches the taste file", () => {
  const md = readFileSync(
    new URL("../../assets/taste/pacing.md", import.meta.url),
    "utf8",
  );
  for (const p of PACING) {
    const row = `| ${p.level} | ${p.name} | ${p.avgShot.toFixed(1)} | ${p.minShot.toFixed(1)} | ${p.maxShot.toFixed(1)} | ${p.cutsPer15s} |`;
    assert.ok(md.includes(row), `missing row: ${row}`);
  }
});

test("display shows scalars plainly and objects as JSON", () => {
  assert.equal(display(undefined), "");
  assert.equal(display(15), "15");
  assert.equal(display("x"), "x");
  assert.equal(display({ a: 1 }), '{"a":1}');
});

test("confirm marks only changed fields as user, answers questions, sets status", () => {
  const req = confirmPatch(
    project(),
    { length: 15, pacing: 4, "a/b": "x" },
    { price: " yes ", other: "ignored" },
  );
  assert.equal(req.baseRev, 7);
  assert.equal(req.author, "human");
  assert.deepEqual(req.ops, [
    {
      op: "add",
      path: "/brief/fields/pacing",
      value: { value: 4, source: "user", confidence: 1 },
    },
    {
      op: "add",
      path: "/brief/fields/a~1b",
      value: { value: "x", source: "user", confidence: 1 },
    },
    { op: "add", path: "/brief/questions/0/answer", value: "yes" },
    { op: "replace", path: "/brief/status", value: "confirmed" },
  ]);
});

test("confirm with no edits only sets the status", () => {
  assert.deepEqual(confirmPatch(project(), {}).ops, [
    { op: "replace", path: "/brief/status", value: "confirmed" },
  ]);
});
