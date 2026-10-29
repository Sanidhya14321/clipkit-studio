import assert from "node:assert/strict";
import { test } from "node:test";
import type { Project } from "./generated/types";
import { anchorFor, anchorLabel, kinds, type Pointer } from "./comments.ts";

const pointer: Pointer = {
  playhead: 2.3456,
  loopRange: [1, 3],
  selection: "title",
  element: { layerId: "title", path: "headline" },
};

test("the anchor kinds follow what the user points at", () => {
  assert.deepEqual(kinds(pointer), ["moment", "range", "layer", "element"]);
  assert.deepEqual(
    kinds({ playhead: 0, loopRange: null, selection: null, element: null }),
    ["moment"],
  );
});

test("each kind builds its anchor", () => {
  assert.deepEqual(anchorFor("moment", pointer), { tStart: 2.346 });
  assert.deepEqual(anchorFor("range", pointer), { tStart: 1, tEnd: 3 });
  assert.deepEqual(anchorFor("layer", pointer), { layerId: "title" });
  assert.deepEqual(anchorFor("element", pointer), {
    layerId: "title",
    propPath: "headline",
  });
});

test("labels name the layer, the element and the time", () => {
  const p = {
    tracks: [
      { id: "t", kind: "overlay", items: [{ id: "title", name: "Hook" }] },
    ],
  } as unknown as Project;
  assert.equal(
    anchorLabel(p, { layerId: "title", propPath: "headline" }),
    "Hook · headline",
  );
  assert.equal(anchorLabel(p, { tStart: 1, tEnd: 2.5 }), "1.0–2.5s");
  assert.equal(anchorLabel(p, {}), "project");
});
