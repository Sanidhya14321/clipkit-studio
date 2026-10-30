// Preview responsiveness and hover-select, in system Chrome. "Patch" = one word changes; then
// seek and wait for two animation frames with the new text in the DOM.
import { createServer } from "node:http";
import { copyFileSync, cpSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, extname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";
import { chromium } from "playwright-core";
import { FIXTURES } from "./fixtures.mjs";
import { WORK, hfHtml, hyperframes } from "./adapters.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);
const CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const median = (a) => [...a].sort((x, y) => x - y)[Math.floor(a.length / 2)];
const web = join(WORK, "preview");
mkdirSync(web, { recursive: true });
const MIME = { ".html": "text/html", ".js": "text/javascript", ".mp4": "video/mp4" };
const server = createServer((q, s) => {
  try { const p = join(web, decodeURIComponent(q.url.split("?")[0]));
    s.setHeader("content-type", MIME[extname(p)] ?? "application/octet-stream"); s.end(readFileSync(p));
  } catch { s.statusCode = 404; s.end(); }
}).listen(0);
const base = `http://127.0.0.1:${server.address().port}`;
const f = FIXTURES.kinetic;

// Remotion: the Player with the same Scene the render uses.
writeFileSync(join(web, "entry.tsx"), `
(window as any).__ckPreview = true;
import { createRoot } from "react-dom/client"; import { createElement as h, createRef } from "react";
import { Player } from "@remotion/player"; import { Scene } from "${here}/RemotionScene.tsx"; import { FIXTURES } from "${here}/fixtures.mjs";
const f = JSON.parse(JSON.stringify(FIXTURES.kinetic)); const r = createRoot(document.getElementById("root")!); const ref = createRef<any>();
const draw = (fx) => r.render(h(Player, { ref, component: Scene as any, inputProps: { fixture: fx }, durationInFrames: 450, fps: 30, compositionWidth: 1920, compositionHeight: 1080, style: { width: 960 } }));
draw(f); (window as any).__patch = (text, frame) => new Promise((ok) => { const n = JSON.parse(JSON.stringify(f)); n.layers[1].text = text; draw(n); ref.current.seekTo(frame); const w = () => document.body.innerText.includes(text) ? requestAnimationFrame(() => requestAnimationFrame(ok)) : requestAnimationFrame(w); w(); });`);
await build({ entryPoints: [join(web, "entry.tsx")], bundle: true, outfile: join(web, "rem.js"), loader: { ".tsx": "tsx" }, jsx: "automatic", nodePaths: [resolve(here, "../node_modules")], define: { "process.env.NODE_ENV": '"production"' }, logLevel: "error" });
writeFileSync(join(web, "rem.html"), `<!doctype html><div id=root></div><script src=rem.js></script>`);

// HyperFrames: the player element. A word change reloads the composition document (srcdoc is
// the documented in-place route, and the HTML must be rebuilt by the compiler either way).
const c = await hyperframes.compile(f);
cpSync(c.dir, join(web, "hf"), { recursive: true });
copyFileSync(require.resolve("@hyperframes/player").replace(/hyperframes-player\.c?js$/, "hyperframes-player.global.js"), join(web, "player.js"));
writeFileSync(join(web, "hf.html"), `<!doctype html><script src=player.js></script><hyperframes-player id=p src="hf/index.html" style="width:960px;height:540px;display:block"></hyperframes-player>
<script>window.__patch=(text,t)=>new Promise(ok=>{const p=document.getElementById("p");const u="hf/v"+text+".html";
p.addEventListener("ready",function g(){p.removeEventListener("ready",g);p.seek(t);requestAnimationFrame(()=>requestAnimationFrame(ok))});p.setAttribute("src",u)});</script>`);
const browser = await chromium.launch({ executablePath: CHROME, headless: true });
const out = {};
try {
  const rp = await browser.newPage(); await rp.goto(`${base}/rem.html`); await rp.waitForFunction(() => window.__patch);
  const rs = []; for (let i = 0; i < 21; i++) rs.push(await rp.evaluate(async ([i]) => { const a = performance.now(); await window.__patch("REM" + i, 3); return performance.now() - a; }, [i]));
  out.remotionSeekMs = Math.round(median(rs.slice(1)));
  out.remotionHover = await rp.evaluate(() => document.querySelectorAll("[data-ck-layer]").length);
  await rp.close();
  for (let i = 0; i < 21; i++) { const n = structuredClone(f); n.layers[1].text = "HF" + i; writeFileSync(join(web, "hf", `vHF${i}.html`), hfHtml(n)); }
  const hp = await browser.newPage(); await hp.goto(`${base}/hf.html`); await hp.waitForFunction(() => document.getElementById("p")?.ready);
  const hs = []; for (let i = 0; i < 21; i++) hs.push(await hp.evaluate(async ([i]) => { const a = performance.now(); await window.__patch("HF" + i, 0.1); return performance.now() - a; }, [i]));
  out.hyperframesSeekMs = Math.round(median(hs.slice(1)));
  const ss = []; for (let i = 0; i < 21; i++) ss.push(await hp.evaluate(async ([t]) => { const a = performance.now(); document.getElementById("p").seek(t); await new Promise((k) => requestAnimationFrame(() => requestAnimationFrame(k))); return performance.now() - a; }, [(i % 14) + 0.3]));
  out.hyperframesSeekOnlyMs = Math.round(median(ss.slice(1)));
  out.hyperframesHover = await hp.evaluate(() => document.querySelector("hyperframes-player").iframeElement.contentDocument.querySelectorAll("[data-ck-layer]").length);
  out.expectedLayers = f.layers.length;
} finally { await browser.close(); server.close(); }
console.log(JSON.stringify(out));
