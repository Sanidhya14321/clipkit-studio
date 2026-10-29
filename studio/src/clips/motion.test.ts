// Unit tests for the core clips' motion helpers (pure; node --test).
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  CORE_CLIPS,
  EASINGS,
  EXIT_SEC,
  boxStyle,
  counterText,
  easing,
  isCoreClip,
  lookId,
  presence,
  resolveTokens,
  staggerDelay,
  textLines,
} from "./motion.ts";

test("CORE_CLIPS mirrors the Rust list exactly", () => {
  const rust = readFileSync(
    new URL("../../../crates/ck-assets/src/recipe.rs", import.meta.url),
    "utf8",
  );
  const block = /CORE_CLIPS: &\[&str\] = &\[([^\]]*)\]/.exec(rust)?.[1] ?? "";
  const ids = [...block.matchAll(/"([^"]+)"/g)].map((m) => m[1]);
  assert.deepEqual(ids, [...CORE_CLIPS]);
  assert.equal(isCoreClip("kinetic-text"), true);
  assert.equal(isCoreClip("nope"), false);
  assert.equal(isCoreClip(null), false);
});

test("every look easing starts at 0 and ends at 1; unknown names fall back", () => {
  for (const [name, f] of Object.entries(EASINGS)) {
    assert.ok(Math.abs(f(0)) < 1e-9, name);
    assert.ok(Math.abs(f(1) - 1) < 1e-9, name);
  }
  for (const look of [
    "clean-product",
    "calm-documentary",
    "cinematic",
    "bold-social",
  ]) {
    const json = JSON.parse(
      readFileSync(
        new URL(`../../../assets/looks/${look}.json`, import.meta.url),
        "utf8",
      ),
    ) as { motion: { enterEasing: string; exitEasing: string } };
    assert.ok(EASINGS[json.motion.enterEasing], `${look} enter`);
    assert.ok(EASINGS[json.motion.exitEasing], `${look} exit`);
  }
  assert.equal(easing("bogus"), EASINGS["cubic-out"]);
});

test("presence enters, holds and exits within [0, 1], and is deterministic", () => {
  const props = { easing: "expo-out" };
  assert.equal(presence(0, 3, props), 0);
  assert.equal(presence(1.5, 3, props), 1);
  assert.equal(presence(3, 3, props), 0);
  assert.ok(presence(3 - EXIT_SEC / 2, 3, props) < 1);
  for (let t = -1; t <= 4; t += 0.05) {
    const p = presence(t, 3, props);
    assert.ok(p >= 0 && p <= 1);
    assert.equal(p, presence(t, 3, props));
  }
  // A delayed part is still hidden when the first one is already showing.
  assert.ok(presence(0.2, 3, props, 0.3) < presence(0.2, 3, props));
});

test("stagger reads the look's seconds per step", () => {
  assert.ok(Math.abs(staggerDelay({ stagger: 0.06 }, 3) - 0.18) < 1e-9);
  assert.equal(staggerDelay({ stagger: "0.1" }, 2), 0.2);
  assert.equal(staggerDelay({ stagger: -1 }, 2), 0);
});

test("counter counts up keeping decimals; text passes through", () => {
  assert.equal(counterText("120", 0, 2, {}), "0");
  assert.equal(counterText("120", 2, 2, {}), "120");
  assert.equal(counterText("4.5", 2, 2, {}), "4.5");
  assert.equal(counterText("4.5", 0, 2, {}), "0.0");
  assert.equal(counterText("3x", 1, 2, {}), "3x");
});

test("token refs resolve against the look; unknown refs drop; brief picks the look", () => {
  const look = JSON.parse(
    readFileSync(
      new URL("../../../assets/looks/bold-social.json", import.meta.url),
      "utf8",
    ),
  ) as object;
  assert.deepEqual(
    resolveTokens(
      {
        color: "$colors.background",
        font: "$fonts.display",
        easing: "$motion.enterEasing",
        stagger: "$motion.stagger",
        radius: "$size.radius",
        text: "Hi",
        bad: "$colors.nope",
      },
      look,
    ),
    {
      color: "#0E0E12",
      font: "Archivo Black",
      easing: "expo-out",
      stagger: 0.06,
      radius: 0.02,
      text: "Hi",
    },
  );
  assert.deepEqual(resolveTokens({ color: "$colors.text" }, undefined), {});
  assert.equal(
    lookId({ brief: { fields: { look: { value: "cinematic" } } } }),
    "cinematic",
  );
  assert.equal(
    lookId({
      tokens: { look: "bold-social" },
      brief: { fields: { look: { value: "cinematic" } } },
    }),
    "bold-social",
  );
  assert.equal(lookId({ brief: { fields: {} } }), "clean-product");
});

test("box and lines come from the solver's props, with fallbacks", () => {
  assert.deepEqual(
    boxStyle(
      { box: { x: 0.1, y: 0.2, w: 0.5, h: 0.25 } },
      { x: 0, y: 0, w: 1, h: 1 },
    ),
    { left: "10.000%", top: "20.000%", width: "50.000%", height: "25.000%" },
  );
  assert.deepEqual(boxStyle({ box: { x: "a" } }, { x: 0, y: 0, w: 1, h: 1 }), {
    left: "0.000%",
    top: "0.000%",
    width: "100.000%",
    height: "100.000%",
  });
  assert.deepEqual(textLines({ text: "a b", lines: ["a", "b"] }), ["a", "b"]);
  assert.deepEqual(textLines({ text: "a b" }), ["a b"]);
  assert.deepEqual(textLines({}), []);
});
