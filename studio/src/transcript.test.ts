import assert from "node:assert/strict";
import { test } from "node:test";
import type { Project, Word } from "./generated/types";
import { deletedWords, timelineTime, toggled } from "./transcript.ts";

const words: Word[] = [
  { i: 0, text: "Welcome", start: 1.0, end: 1.4 },
  { i: 1, text: "um", start: 1.45, end: 1.7 },
  { i: 2, text: "to", start: 1.75, end: 1.85 },
];

function project(segs: [number, number, number][]): Project {
  const items = segs.map(([srcIn, srcOut, at], n) => ({
    id: `s-${n}`,
    name: "",
    purpose: "",
    briefDeps: [],
    rev: 0,
    type: "footage" as const,
    src: "src-1",
    srcIn,
    srcOut,
    at,
    speed: 1,
    volume: 1,
  }));
  return {
    tracks: [{ id: "video-1", kind: "video", items }],
  } as unknown as Project;
}

test("a word is deleted when no segment keeps its midpoint", () => {
  const p = project([
    [0, 1.45, 0],
    [1.75, 12, 1.45],
  ]);
  assert.deepEqual([...deletedWords(p, "src-1", words)], [1]);
  assert.equal(deletedWords(project([]), "src-1", words).size, 0);
});

test("toggling deletes or restores one word in the whole set", () => {
  assert.deepEqual(toggled(new Set([1]), 2), [1, 2]);
  assert.deepEqual(toggled(new Set([1, 2]), 1), [2]);
});

test("source time maps through the segments", () => {
  const p = project([
    [0, 1.45, 0],
    [1.75, 12, 1.45],
  ]);
  assert.equal(timelineTime(p, "src-1", 1.0), 1.0);
  assert.equal(timelineTime(p, "src-1", 1.5), null);
  assert.ok(Math.abs((timelineTime(p, "src-1", 2.0) ?? 0) - 1.7) < 1e-9);
  assert.equal(timelineTime(project([]), "src-1", 3), 3);
});

test("media URLs carry the token for video sources with a proxy only", async () => {
  const { mediaUrls } = await import("./timeline.ts");
  const p = {
    sources: [
      { id: "v", kind: "video", path: "a.mp4", proxyPath: "x", hash: "ab" },
      { id: "a", kind: "audio", path: "a.wav", hash: "cd" },
    ],
  } as unknown as Project;
  assert.deepEqual(mediaUrls(p, "t k"), { v: "/media/ab.mp4?token=t%20k" });
  assert.deepEqual(mediaUrls(null, "t"), {});
});
