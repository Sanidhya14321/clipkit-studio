// EngineAdapter (UPDATE_5 2.1): compile, renderFrames, renderVideo, capabilities. The studio's
// preview mount is measured separately in preview.mjs.
import { execFile } from "node:child_process";
import { copyFileSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import { createRequire } from "node:module";
const run = promisify(execFile);
const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "..");
const require = createRequire(import.meta.url);
const pk = (n) => dirname(require.resolve(`${n}/package.json`));
export const WORK = process.env.SPIKE_WORK ?? join(root, "out");
const FOOTAGE = join(WORK, "footage.mp4");

export async function ensureFootage() {
  mkdirSync(join(WORK, "public"), { recursive: true });
  await run("ffmpeg", ["-y", "-f", "lavfi", "-i", "testsrc2=s=1280x720:r=30:d=12", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-g", "1", FOOTAGE]);
  copyFileSync(FOOTAGE, join(WORK, "public", "footage.mp4"));
}

// ---- Remotion ----
let serveUrl;
export const remotion = {
  name: "remotion",
  capabilities: () => ({ runtimes: ["remotion", "css", "gsap", "motion", "three", "lottie"], motionBlur: true }),
  async compile(f) {
    if (!serveUrl) {
      const { bundle } = await import("@remotion/bundler");
      serveUrl = await bundle({ entryPoint: join(here, "RemotionScene.tsx"), publicDir: join(WORK, "public"), outDir: join(WORK, "rem-bundle") });
    }
    const { selectComposition } = await import("@remotion/renderer");
    return { composition: await selectComposition({ serveUrl, id: f.id }) };
  },
  async renderFrames(c, frames, out) {
    const { renderStill } = await import("@remotion/renderer");
    mkdirSync(out, { recursive: true });
    const paths = [];
    for (const [i, frame] of frames.entries()) {
      const output = join(out, `${i}-${frame}.png`);
      await renderStill({ composition: c.composition, serveUrl, output, frame, imageFormat: "png" });
      paths.push(output);
    }
    return paths;
  },
  async renderVideo(c, out) {
    const { renderMedia } = await import("@remotion/renderer");
    await renderMedia({ composition: c.composition, serveUrl, codec: "h264", outputLocation: out, muted: true, imageFormat: "jpeg" });
  },
};

// ---- HyperFrames ----
const ck = (l) => `id="${l.id}" data-ck-layer="${l.id}" data-ck-path="layers/${l.id}"`;
const clip = (l, i) => `class="clip" data-start="${l.at}" data-duration="${l.dur}" data-track-index="${i}"`;
function layerHtml(l, i) {
  switch (l.type) {
    case "whip": return `<div ${ck(l)} ${clip(l, i)} style="position:absolute;top:0;left:0;width:3840px;height:1080px;background:linear-gradient(90deg,#14213d 0%,#14213d 50%,#7b2cbf 50%,#7b2cbf 100%)"></div>`;
    case "slam": return `<div ${ck(l)} ${clip(l, i)} style="position:absolute;inset:0;display:flex;align-items:center;justify-content:center;color:#fff;font:900 220px Helvetica Neue,Arial,sans-serif">${l.text}</div>`;
    case "panel": return `<div ${ck(l)} ${clip(l, i)} style="position:absolute;inset:0;background:#f5c518"></div>`;
    case "word3d": return `<div ${ck(l)} ${clip(l, i)} style="position:absolute;inset:0;display:flex;align-items:center;justify-content:center;perspective:1000px"><div id="w3d" style="font:900 240px Helvetica Neue,Arial,sans-serif;color:#14213d;text-shadow:6px 6px 0 #7b2cbf,12px 12px 0 #5a189a">${l.text}</div></div>`;
    case "video": return `<video ${ck(l)} ${clip(l, i)} data-media-start="${l.srcIn}" src="footage.mp4" muted playsinline style="position:absolute;inset:0;width:100%;height:100%"></video>`;
    case "caption": return `<div ${ck(l)} ${clip(l, i)} style="position:absolute;left:0;right:0;bottom:90px;text-align:center;font:800 56px Helvetica Neue,Arial,sans-serif;color:#fff">${l.words.map((w, k) => `<span id="cw${k}" style="margin:0 10px">${w.w}</span>`).join(" ")}</div>`;
    case "lowerThird": return `<div ${ck(l)} ${clip(l, i)} style="position:absolute;left:80px;top:760px;padding:18px 34px;background:#7b2cbf;color:#fff;font:700 44px Helvetica Neue,Arial,sans-serif">${l.text}</div>`;
    case "box": return `<div ${ck(l)} ${clip(l, i)} style="position:absolute;left:200px;top:${l.runtime === "gsap" ? 220 : 640}px;width:220px;height:220px;border-radius:24px;background:${l.runtime === "gsap" ? "#88ce02" : "#ff0088"}"></div>`;
  }
}
function tweens(l) {
  switch (l.type) {
    case "whip": return `tl.to("#strip",{x:-1920,duration:.5,ease:"power2.inOut"},${l.pivot - 0.25});tl.fromTo("#strip",{filter:"blur(0px)"},{filter:"blur(40px)",duration:.25,ease:"none"},${l.pivot - 0.25});tl.to("#strip",{filter:"blur(0px)",duration:.25,ease:"none"},${l.pivot});`;
    case "slam": return `tl.fromTo("#${l.id}",{scale:3},{scale:1,duration:.12,ease:"back.out(1.7)"},${l.at});`;
    case "word3d": return `tl.fromTo("#w3d",{rotationY:-50},{rotationY:50,duration:${l.dur},ease:"none"},${l.at});`;
    case "caption": return l.words.map((w, k) => `tl.set("#cw${k}",{color:"#f5c518"},${w.at});tl.set("#cw${k}",{color:"#fff"},${w.at + 0.8});`).join("");
    case "lowerThird": return `tl.fromTo("#${l.id}",{x:-700},{x:0,duration:.4,ease:"none"},${l.at});tl.to("#${l.id}",{x:-700,duration:.4,ease:"none"},${l.at + l.dur - 0.4});`;
    case "box": if (l.runtime === "gsap") return `tl.to("#gsap-box",{x:1300,rotation:360,duration:6,ease:"power2.inOut"},0);`;
      // Motion has no HyperFrames adapter: it is driven from the GSAP clock by a proxy tween.
      return `var mc=Motion.animate("#motion-box",{x:[0,1300],rotate:[0,-360]},{duration:6,ease:"easeInOut"});mc.pause();var p={t:0};tl.to(p,{t:6,duration:6,ease:"none",onUpdate:function(){mc.time=p.t}},0);`;
    default: return "";
  }
}
export function hfHtml(f) {
  return `<!doctype html><html><head><meta charset="utf-8"><style>html,body{margin:0;width:${f.width}px;height:${f.height}px;overflow:hidden;background:#0b0b12}</style></head><body>
<div id="stage" data-composition-id="${f.id}" data-start="0" data-duration="${f.duration}" data-width="${f.width}" data-height="${f.height}" style="position:relative;width:${f.width}px;height:${f.height}px;overflow:hidden;background:#0b0b12">
${f.layers.map(layerHtml).join("\n")}
<script src="gsap.min.js"></script><script src="motion.js"></script>
<script>var tl=gsap.timeline({paused:true});${f.layers.map(tweens).join("")}
window.__timelines=window.__timelines||{};window.__timelines[${JSON.stringify(f.id)}]=tl;</script>
</div></body></html>`;
}
const hf = (args, cwd) => run(join(root, "node_modules/.bin/hyperframes"), args, { cwd, maxBuffer: 1 << 26, env: { ...process.env, DO_NOT_TRACK: "1" } });
export const hyperframes = {
  name: "hyperframes",
  capabilities: () => ({ runtimes: ["html", "css", "gsap", "lottie", "three", "anime", "waapi"], motionBlur: false }),
  async compile(f) {
    const dir = join(WORK, "hf", f.id);
    mkdirSync(dir, { recursive: true });
    writeFileSync(join(dir, "index.html"), hfHtml(f));
    copyFileSync(join(pk("gsap"), "dist/gsap.min.js"), join(dir, "gsap.min.js"));
    copyFileSync(join(pk("motion"), "dist/motion.js"), join(dir, "motion.js"));
    if (f.id === "footage") copyFileSync(FOOTAGE, join(dir, "footage.mp4"));
    return { dir, fixture: f };
  },
  async renderFrames(c, frames, out) {
    mkdirSync(out, { recursive: true });
    const at = frames.map((n) => (n / c.fixture.fps).toFixed(6)).join(",");
    const { stdout } = await hf(["snapshot", c.dir, "--at", at, "--no-end", "-o", out, "--json"], c.dir);
    return JSON.parse(stdout.slice(stdout.indexOf("{")));
  },
  async renderVideo(c, out) {
    await hf(["render", c.dir, "-o", out, "-f", String(c.fixture.fps)], c.dir);
  },
  async renderSequence(c, out) {
    await hf(["render", c.dir, "--format", "png-sequence", "-o", out, "-f", String(c.fixture.fps)], c.dir);
    return readdirSync(out).filter((n) => n.endsWith(".png")).sort().map((n) => join(out, n));
  },
};
export const pixelHash = async (png) => (await run("ffmpeg", ["-v", "error", "-i", png, "-pix_fmt", "rgb24", "-f", "md5", "-"])).stdout.trim();
export const sourceBytes = (p) => readFileSync(p).length;
