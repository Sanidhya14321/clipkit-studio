// The three engine-neutral fixtures of UPDATE_5 section 2.2. Plain data: no engine concepts.
// Layers declare a runtime; each adapter compiles them. Times in seconds.
const W = ["SHIP", "FASTER", "CUT", "THE", "NOISE", "EVERY", "FRAME", "HITS", "THE", "BEAT"];
const slams = Array.from({ length: 20 }, (_, i) => ({
  id: `word-${i}`, type: "slam", runtime: "gsap", at: i * 0.5, dur: 0.5, text: W[i % W.length],
}));
const FPS = 30, width = 1920, height = 1080;
const base = { fps: FPS, width, height };
export const FIXTURES = {
  kinetic: {
    ...base, id: "kinetic", duration: 15, probeFrames: [12, 225, 330, 400, 12],
    layers: [
      { id: "strip", type: "whip", runtime: "gsap", at: 0, dur: 15, pivot: 7.5 },
      ...slams,
      { id: "colorflip", type: "panel", runtime: "gsap", at: 10, dur: 5 },
      { id: "word3d", type: "word3d", runtime: "gsap", at: 10, dur: 5, text: "CLIPKIT" },
    ],
  },
  footage: {
    ...base, id: "footage", duration: 9, probeFrames: [10, 95, 200, 95],
    layers: [
      { id: "shot-1", type: "video", runtime: "css", at: 0, dur: 3, srcIn: 0 },
      { id: "shot-2", type: "video", runtime: "css", at: 3, dur: 3, srcIn: 6 },
      { id: "shot-3", type: "video", runtime: "css", at: 6, dur: 3, srcIn: 2 },
      { id: "caption", type: "caption", runtime: "gsap", at: 0.5, dur: 8,
        words: "every word lights up exactly when it is spoken".split(" ").map((w, i) => ({ w, at: 0.5 + i * 0.8 })) },
      { id: "lower-third", type: "lowerThird", runtime: "gsap", at: 1, dur: 4, text: "Sanidhya, ClipKit" },
    ],
  },
  web: {
    ...base, id: "web", duration: 6, probeFrames: [30, 150, 90, 30],
    layers: [
      { id: "gsap-box", type: "box", runtime: "gsap", at: 0, dur: 6 },
      { id: "motion-box", type: "box", runtime: "motion", at: 0, dur: 6 },
    ],
  },
};
