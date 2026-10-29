// Unit tests for the pure studio logic. Run with `pnpm test` (node --test; Node 24 strips types).
import assert from "node:assert/strict";
import { test } from "node:test";
import type { Item, Lock, Project } from "./generated/types";
import {
  FEED_LIMIT,
  agentLocked,
  agentStopped,
  canEdit,
  editableFields,
  fieldPatch,
  initial,
  isLive,
  locate,
  reduce,
} from "./state.ts";

const base = { purpose: "", briefDeps: [], rev: 1 };

const title: Item = {
  ...base,
  id: "title",
  name: "Hook title",
  type: "clip",
  at: 0,
  dur: 2,
  props: {},
};

const music: Item = {
  ...base,
  id: "music",
  name: "Music",
  type: "audio",
  src: "s1",
  at: 0,
  ranges: [],
  volume: 1,
  duck: true,
};

const caption: Item = {
  ...base,
  id: "cap",
  name: "Captions",
  type: "caption",
  from: "voice",
  ref: "v1",
  style: "default",
};

function project(rev = 3): Project {
  return {
    schemaVersion: 1,
    id: "p",
    name: "Demo",
    rev,
    settings: { fps: 30, width: 1920, height: 1080 },
    mode: "generate",
    brief: { fields: {}, status: "proposed", checkpoints: [] },
    plan: null,
    tokens: {},
    sources: [],
    tracks: [
      { id: "v", kind: "overlay", items: [title] },
      { id: "a", kind: "audio", items: [caption, music] },
    ],
    voiceSlots: [],
    comments: [],
    assetsUsed: [],
    derived: [],
    locks: [],
    branches: [],
    pipeline: { stage: "passes", pass: "text" },
    agent: { status: "working", label: "Text pass", activeLayers: ["title"] },
  };
}

const lock = (
  layerId: string,
  holder: Lock["holder"],
  heartbeatAtMs: number,
): Lock => ({
  layerId,
  holder,
  acquiredAtMs: heartbeatAtMs,
  heartbeatAtMs,
  ttlMs: 1000,
});

test("snapshot sets the project and its locks", () => {
  const p = { ...project(), locks: [lock("title", "agent", 0)] };
  const s = reduce(initial, { type: "snapshot", project: p });
  assert.equal(s.project, p);
  assert.deepEqual(s.locks, p.locks);
});

test("locks event replaces the lock list", () => {
  const locks = [lock("music", "agent", 5)];
  const s = reduce(initial, { type: "locks", locks });
  assert.deepEqual(s.locks, locks);
});

test("revision events feed the activity list newest first, capped", () => {
  let s = initial;
  for (let rev = 1; rev <= FEED_LIMIT + 5; rev++) {
    s = reduce(s, {
      type: "revision",
      rev,
      author: "agent",
      label: `step ${rev}`,
      touched: ["title"],
    });
  }
  assert.equal(s.feed.length, FEED_LIMIT);
  assert.equal(s.feed[0]?.rev, FEED_LIMIT + 5);
  assert.equal(s.lastRevision?.label, `step ${FEED_LIMIT + 5}`);
});

test("lock liveness matches the server: live strictly before heartbeat + ttl", () => {
  const l = lock("title", "agent", 100);
  assert.equal(isLive(l, 1099), true);
  assert.equal(isLive(l, 1100), false);
});

test("only live agent locks block human edits", () => {
  const locks = [
    lock("title", "agent", 0),
    lock("music", "human", 0),
    lock("cap", "agent", -5000),
  ];
  assert.deepEqual([...agentLocked(locks, 500)], ["title"]);
  assert.equal(canEdit(locks, "title", 500), false);
  assert.equal(canEdit(locks, "music", 500), true);
  assert.equal(canEdit(locks, "cap", 500), true);
  assert.equal(canEdit(locks, "title", 2000), true);
});

test("agent counts as stopped only when it had locks and all lapsed", () => {
  assert.equal(agentStopped([], 0), false);
  assert.equal(agentStopped([lock("title", "agent", 0)], 500), false);
  assert.equal(agentStopped([lock("title", "agent", 0)], 1000), true);
  assert.equal(agentStopped([lock("title", "human", 0)], 5000), false);
});

test("locate finds an item's track and index", () => {
  const at = locate(project(), "music");
  assert.equal(at?.trackIndex, 1);
  assert.equal(at?.itemIndex, 1);
  assert.equal(locate(project(), "nope"), null);
});

test("editable fields follow the item type", () => {
  assert.deepEqual(editableFields(title), ["name", "purpose", "at", "dur"]);
  assert.deepEqual(editableFields(music), ["name", "purpose", "at"]);
  assert.deepEqual(editableFields(caption), ["name", "purpose"]);
});

test("field patch is one replace against the seen revisions, authored by the human", () => {
  const req = fieldPatch(project(7), "music", "at", 1.5);
  assert.deepEqual(req, {
    baseRev: 7,
    layerRevs: { music: 1 },
    ops: [{ op: "replace", path: "/tracks/1/items/1/at", value: 1.5 }],
    author: "human",
    label: "Edit at of Music",
  });
});

test("field patch refuses unknown layers and fields that do not apply", () => {
  assert.equal(fieldPatch(project(), "nope", "name", "x"), null);
  assert.equal(fieldPatch(project(), "cap", "at", 1), null);
  assert.equal(fieldPatch(project(), "music", "dur", 1), null);
});

test("asks events replace the open questions", () => {
  const asks = [{ id: "q1", question: "Which?", options: ["A", "B"] }];
  const s = reduce(initial, { type: "asks", asks });
  assert.deepEqual(s.asks, asks);
  assert.deepEqual(reduce(s, { type: "asks", asks: [] }).asks, []);
});
