import assert from "node:assert/strict";
import { test } from "node:test";
import type { Project } from "./generated/types";
import { chipEdits, chipPatch, impacted } from "./tune.ts";

function project(fields: Record<string, unknown>): Project {
  const brief = {
    status: "confirmed",
    fields: Object.fromEntries(
      Object.entries(fields).map(([k, v]) => [
        k,
        { value: v, source: "inferred", confidence: 0.8 },
      ]),
    ),
  };
  const items = [
    { id: "bg", briefDeps: ["look"] },
    { id: "head", briefDeps: ["pacing", "look"] },
    { id: "cta", briefDeps: ["cta"] },
  ];
  return {
    rev: 7,
    brief,
    tracks: [{ id: "t", kind: "overlay", items }],
  } as unknown as Project;
}

test("chips step pacing, shorten the length and move the look", () => {
  const p = project({ pacing: "lively", length: 15, look: "clean-product" });
  assert.deepEqual(chipEdits(p, "punchier"), { pacing: 4 });
  assert.deepEqual(chipEdits(p, "calmer"), { pacing: 2 });
  assert.deepEqual(chipEdits(p, "shorter"), { length: 12 });
  assert.deepEqual(chipEdits(p, "premium"), { look: "cinematic" });
});

test("chips at the end of their range change nothing", () => {
  const p = project({ pacing: 5, length: 1, look: "cinematic" });
  assert.equal(chipEdits(p, "punchier"), null);
  assert.equal(chipEdits(p, "shorter"), null);
  assert.equal(chipEdits(p, "premium"), null);
  assert.equal(chipEdits(project({ pacing: 1 }), "calmer"), null);
});

test("impact lists only the layers whose briefDeps the change hits", () => {
  const p = project({});
  assert.deepEqual(impacted(p, ["pacing"]), ["head"]);
  assert.deepEqual(impacted(p, ["look"]), ["bg", "head"]);
  assert.deepEqual(impacted(p, ["length"]), []);
});

test("the premium patch also writes the stored look", () => {
  const p = project({ look: "clean-product" });
  const req = chipPatch(p, "premium", { look: "cinematic" });
  assert.equal(req.baseRev, 7);
  assert.equal(req.author, "human");
  assert.deepEqual(req.ops, [
    {
      op: "add",
      path: "/brief/fields/look",
      value: { value: "cinematic", source: "user", confidence: 1 },
    },
    { op: "add", path: "/tokens/look", value: "cinematic" },
  ]);
});
