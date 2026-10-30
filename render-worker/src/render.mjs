// Render one job (spec 2.3 full export path). ck writes the job as JSON on stdin:
//   { bundle, out, project, looks, media, codec?, frameRange?, still?, stills? }
// `stills` is a list of {frame, out} rendered in the order given (the determinism check
// renders the same frame twice and frames out of order).
// and reads JSON lines on stdout: {"progress": 0..1} while rendering, then
// {"done": "<out>"} or {"error": "..."}. Video only (muted); ck mixes and muxes the audio.
// The job is data from ck; nothing in a project is executed beyond the composition itself.
import { renderMedia, renderStill, selectComposition } from "@remotion/renderer";

const line = (v) => process.stdout.write(JSON.stringify(v) + "\n");

async function readJob() {
  let text = "";
  for await (const chunk of process.stdin) text += chunk;
  return JSON.parse(text);
}

try {
  const job = await readJob();
  const inputProps = {
    project: job.project,
    looks: job.looks ?? [],
    agentLayers: [],
    flash: [],
    flashRev: 0,
    media: job.media ?? {},
  };
  const composition = await selectComposition({
    serveUrl: job.bundle,
    id: "project",
    inputProps,
  });
  if (Array.isArray(job.stills)) {
    for (const [i, s] of job.stills.entries()) {
      await renderStill({
        composition,
        serveUrl: job.bundle,
        output: s.out,
        frame: s.frame,
        inputProps,
        imageFormat: "png",
      });
      line({ progress: Math.round(((i + 1) / job.stills.length) * 100) / 100 });
    }
  } else if (job.still !== undefined) {
    await renderStill({
      composition,
      serveUrl: job.bundle,
      output: job.out,
      frame: job.still,
      inputProps,
      imageFormat: "png",
    });
  } else {
    await renderMedia({
      composition,
      serveUrl: job.bundle,
      codec: job.codec ?? "h264",
      outputLocation: job.out,
      inputProps,
      muted: true,
      frameRange: job.frameRange ?? null,
      onProgress: ({ progress }) => line({ progress: Math.round(progress * 100) / 100 }),
    });
  }
  line({ done: job.out });
} catch (e) {
  line({ error: e instanceof Error ? e.message : String(e) });
  process.exitCode = 1;
}
