// U5-1 spike runner: frame accuracy, render time, footprint. Writes results.json to WORK.
import { spawnSync } from "node:child_process";
import { readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { FIXTURES } from "./fixtures.mjs";
import { WORK, ensureFootage, hfHtml, hyperframes, pixelHash, remotion } from "./adapters.mjs";

const pngs = (d) => readdirSync(d).filter((n) => n.endsWith(".png")).sort().map((n) => join(d, n));
async function frames(e, c, list, tag) {
  const d = join(WORK, "frames", e.name, c.id ?? tag, tag);
  rmSync(d, { recursive: true, force: true });
  await e.renderFrames(c, list, d);
  const files = pngs(d).sort((a, b) => parseInt(a.split("/").pop().match(/\d+/)) - parseInt(b.split("/").pop().match(/\d+/)));
  const h = {};
  for (const [i, n] of list.entries()) { const x = await pixelHash(files[i]); (h[n] ??= []).push(x); }
  return h;
}
await ensureFootage();
const res = { fixtures: {} };
for (const f of Object.values(FIXTURES)) {
  const row = (res.fixtures[f.id] = {});
  for (const e of [remotion, hyperframes]) {
    const c = await e.compile(f); c.id = f.id;
    const list = f.probeFrames, sorted = [...new Set(list)].sort((a, b) => a - b);
    const a = await frames(e, c, list, "a"), b = await frames(e, c, sorted, "b");
    const mismatched = sorted.filter((n) => new Set([...a[n], ...b[n]]).size !== 1);
    const distinct = new Set(sorted.map((n) => a[n][0])).size;
    const r = { outOfOrderIdentical: mismatched.length === 0, mismatched, distinctFrames: `${distinct}/${sorted.length}` };
    const out = join(WORK, `${f.id}-${e.name}.mp4`), t0 = performance.now();
    await e.renderVideo(c, out);
    r.renderSeconds = +((performance.now() - t0) / 1000).toFixed(1); r.mp4Bytes = statSync(out).size;
    if (e.renderSequence && f.id !== "kinetic") {
      const s1 = await e.renderSequence(c, join(WORK, "seq", f.id, "1")), s2 = await e.renderSequence(c, join(WORK, "seq", f.id, "2"));
      let diff = 0; for (let i = 0; i < s1.length; i += 7) if ((await pixelHash(s1[i])) !== (await pixelHash(s2[i]))) diff++;
      r.sequenceRunsDiffer = diff; r.sequenceFrames = s1.length;
      const n = sorted[1]; r.snapshotMatchesRender = (await pixelHash(s1[n])) === a[n][0];
    }
    row[e.name] = r; console.log(f.id, e.name, JSON.stringify(r));
  }
}
res.authoringChars = { hyperframesHtmlFixture1: hfHtml(FIXTURES.kinetic).length };
writeFileSync(join(WORK, "results.json"), JSON.stringify(res, null, 2));
const p = spawnSync("node", [new URL("./preview.mjs", import.meta.url).pathname], { encoding: "utf8", env: process.env });
console.log(p.stdout.trim().slice(-600), p.stderr.slice(-600));
