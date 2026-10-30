import assert from "node:assert/strict";
import test from "node:test";
import type { Project } from "../generated/types";
import {
  moveRequest,
  trimRequest,
  removeRequest,
  optimisticProject,
  snapTime,
  titleRequest,
} from "./edits.ts";
function fixture(): Project {
  return {
    schemaVersion: 1,
    id: "edit-test",
    name: "Edit fixture",
    rev: 4,
    settings: { fps: 30, width: 1920, height: 1080 },
    mode: "edit",
    brief: { status: "confirmed", fields: {}, checkpoints: [] },
    plan: null,
    tokens: {},
    sources: [
      {
        id: "source",
        kind: "video",
        path: "test.mp4",
        hash: "test",
        durationSec: 20,
      },
    ],
    tracks: [
      {
        id: "video",
        kind: "video",
        items: [
          {
            id: "a",
            name: "A",
            purpose: "",
            briefDeps: [],
            rev: 3,
            type: "footage",
            src: "source",
            srcIn: 2,
            srcOut: 8,
            at: 0,
            speed: 2,
            volume: 1,
          },
          {
            id: "b",
            name: "B",
            purpose: "",
            briefDeps: [],
            rev: 4,
            type: "clip",
            clipId: "kinetic-text",
            at: 3,
            dur: 2,
            props: { text: "B" },
          },
        ],
      },
    ],
    voiceSlots: [],
    comments: [],
    assetsUsed: [],
    derived: [],
    locks: [],
    branches: [],
    pipeline: { stage: "handoff" },
    agent: { status: "idle", label: "", activeLayers: [] },
  };
}
test("move commands preserve source timing, layer revision and server revision", () => {
  const p = fixture(),
    req = moveRequest(p, "a", 1)!;
  assert.equal(req.baseRev, 4);
  assert.deepEqual(req.layerRevs, { a: 3 });
  const next = optimisticProject(p, req);
  assert.equal((next.tracks[0]!.items[0] as { at: number }).at, 1);
  assert.equal((p.tracks[0]!.items[0] as { at: number }).at, 0);
  assert.equal(moveRequest(p, "a", -1), null);
});
test("footage trims use source speed and never exceed source bounds or one-frame minimum", () => {
  const p = fixture();
  const left = optimisticProject(p, trimRequest(p, "a", "start", 1)!);
  assert.equal((left.tracks[0]!.items[0] as { srcIn: number }).srcIn, 4);
  const right = optimisticProject(p, trimRequest(p, "a", "end", 5)!);
  assert.equal((right.tracks[0]!.items[0] as { srcOut: number }).srcOut, 12);
  assert.equal(trimRequest(p, "a", "end", 100), null);
  assert.equal(trimRequest(p, "a", "start", 3), null);
});
test("ripple delete moves later items before removing by index as one undoable patch", () => {
  const p = fixture(),
    req = removeRequest(p, "a", true)!;
  const next = optimisticProject(p, req);
  assert.equal(next.tracks[0]!.items.length, 1);
  assert.equal((next.tracks[0]!.items[0] as { at: number }).at, 0);
  assert.deepEqual(req.layerRevs, { a: 3, b: 4 });
});
test("snapping respects its zoom-dependent threshold and frame precision", () => {
  assert.equal(snapTime(1.98, [0, 2, 3], 30, 0.05), 2);
  assert.equal(snapTime(1.8, [2], 30, 0.05), 1.8);
  assert.equal(snapTime(-1, [], 30, 0.1), 0);
});
test("title add is a server patch to an overlay and leaves the authoritative project intact", () => {
  const p = fixture();
  const next = optimisticProject(p, titleRequest(p, 1, "Hello"));
  assert.equal(next.tracks.at(-1)!.kind, "overlay");
  assert.equal(next.tracks.at(-1)!.items[0]!.name, "Hello");
  assert.equal(p.tracks.length, 1);
});

test("unregistered or derived components are never treated as native rendered assets", async () => {
  const { unsupportedAssetLayers } = await import("./assets.ts");
  const p = fixture();
  assert.deepEqual(unsupportedAssetLayers(p), []);
  p.tracks[0]!.items.push({
    id: "derived",
    name: "External",
    purpose: "",
    briefDeps: [],
    rev: 0,
    type: "clip",
    derivedId: "external-card",
    at: 0,
    dur: 1,
    props: {},
  });
  assert.deepEqual(unsupportedAssetLayers(p), ["derived"]);
});
