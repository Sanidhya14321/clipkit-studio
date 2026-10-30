import assert from "node:assert/strict";
import test from "node:test";
import {
  PANELS,
  WORKSPACES,
  ZONES,
  defaultLayout,
  movePanel,
  parseLayout,
} from "./layout.ts";
import { executeCommand, matchesCommand, type Command } from "./commands.ts";

test("every workspace places each existing panel exactly once", () => {
  for (const workspace of WORKSPACES) {
    const layout = defaultLayout(workspace);
    assert.deepEqual(
      ZONES.flatMap((z) => layout.groups[z]).sort(),
      Object.keys(PANELS).sort(),
    );
    assert.deepEqual(parseLayout(JSON.stringify(layout), workspace), layout);
  }
});

test("closing, reopening and docking cannot duplicate or strand an active panel", () => {
  const original = defaultLayout("Edit");
  const closed = movePanel(original, "program", null);
  assert.equal(closed.active.center, "source");
  assert.deepEqual(original.groups.center, ["program", "source"]);
  const docked = movePanel(
    movePanel(closed, "program", "right"),
    "program",
    "left",
  );
  assert.equal(docked.active.left, "program");
  assert.equal(
    ZONES.flatMap((z) => docked.groups[z]).filter((id) => id === "program")
      .length,
    1,
  );
});

test("untrusted or corrupt stored layouts reset and valid sizes are bounded", () => {
  const fallback = defaultLayout("Review");
  for (const raw of [
    "{",
    "null",
    "[]",
    '{"groups":{}}',
    JSON.stringify({
      ...fallback,
      active: { ...fallback.active, right: null },
    }),
    JSON.stringify({
      ...fallback,
      groups: { ...fallback.groups, center: ["toString"] },
      active: { ...fallback.active, center: "toString" },
    }),
    JSON.stringify({
      ...fallback,
      groups: { ...fallback.groups, center: ["unknown"] },
    }),
    JSON.stringify({
      ...fallback,
      groups: { ...fallback.groups, center: ["timeline"] },
    }),
    JSON.stringify({
      ...fallback,
      active: { ...fallback.active, right: "program" },
    }),
  ]) {
    assert.deepEqual(parseLayout(raw, "Review"), fallback);
  }
  const bounded = parseLayout(
    JSON.stringify({ ...fallback, left: -100, right: 99999, bottom: 0 }),
    "Review",
  );
  assert.equal(bounded.left, 200);
  assert.equal(bounded.right, 480);
  assert.equal(bounded.bottom, 160);
});

test("commands share search, obey capability refusals and surface API errors", async () => {
  let calls = 0;
  const failures: string[] = [];
  const command: Command = {
    id: "undo",
    group: "Edit",
    label: "Undo revision",
    run: () => {
      calls++;
    },
  };
  assert.equal(matchesCommand(command, " edit undo "), true);
  assert.equal(matchesCommand(command, "export"), false);
  await executeCommand({ ...command, reason: "Agent lock" }, (e) =>
    failures.push(e),
  );
  assert.equal(calls, 0);
  await executeCommand(command, (e) => failures.push(e));
  assert.equal(calls, 1);
  await executeCommand(
    {
      ...command,
      run: async () => {
        throw new Error("E_PATCH_STALE: refresh revision");
      },
    },
    (e) => failures.push(e),
  );
  assert.deepEqual(failures, ["E_PATCH_STALE: refresh revision"]);
});
