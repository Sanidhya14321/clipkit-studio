// Remotion adapter's composition: interprets a neutral fixture from the frame clock.
import { useLayoutEffect, useRef } from "react";
import { AbsoluteFill, Composition, Easing, Html5Video, Sequence, interpolate, registerRoot, staticFile, useCurrentFrame, useVideoConfig } from "remotion";
import gsap from "gsap";
import { animate } from "motion";
import { FIXTURES } from "./fixtures.mjs";

const FONT = "Helvetica Neue, Arial, sans-serif";
const clamp = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;

function Layer({ l, t }: { l: any; t: number }) {
  const local = t - l.at;
  const on = local >= 0 && local < l.dur;
  const tag = { "data-ck-layer": l.id, "data-ck-path": `layers/${l.id}` };
  if (l.type === "whip") {
    const d = (t - l.pivot) / 0.25;
    const x = interpolate(d, [-1, 1], [0, -1920], { ...clamp, easing: Easing.inOut(Easing.cubic) });
    const blur = interpolate(Math.abs(d), [0, 1], [40, 0], clamp);
    return <div {...tag} style={{ position: "absolute", top: 0, left: 0, width: 3840, height: 1080, transform: `translateX(${x}px)`, filter: `blur(${blur}px)`,
      background: "linear-gradient(90deg,#14213d 0%,#14213d 50%,#7b2cbf 50%,#7b2cbf 100%)" }} />;
  }
  if (l.type === "slam") {
    const s = interpolate(local, [0, 0.12], [3, 1], { ...clamp, easing: Easing.out(Easing.back(1.7)) });
    return <div {...tag} style={{ position: "absolute", inset: 0, display: on ? "flex" : "none", alignItems: "center", justifyContent: "center", color: "#fff", fontFamily: FONT, fontWeight: 900, fontSize: 220, transform: `scale(${s})` }}>{l.text}</div>;
  }
  if (l.type === "panel")
    return <div {...tag} style={{ position: "absolute", inset: 0, background: "#f5c518", display: on ? "block" : "none" }} />;
  if (l.type === "word3d") {
    const a = interpolate(local, [0, l.dur], [-50, 50], clamp);
    return <div {...tag} style={{ position: "absolute", inset: 0, display: on ? "flex" : "none", alignItems: "center", justifyContent: "center", perspective: 1000 }}>
      <div style={{ fontFamily: FONT, fontWeight: 900, fontSize: 240, color: "#14213d", transform: `rotateY(${a}deg)`, textShadow: "6px 6px 0 #7b2cbf,12px 12px 0 #5a189a" }}>{l.text}</div></div>;
  }
  if (l.type === "caption")
    return <div {...tag} style={{ position: "absolute", left: 0, right: 0, bottom: 90, textAlign: "center", fontFamily: FONT, fontWeight: 800, fontSize: 56, color: "#fff" }}>
      {l.words.map((w: any, i: number) => <span key={i} style={{ margin: "0 10px", color: t >= w.at && t < w.at + 0.8 ? "#f5c518" : "#fff" }}>{w.w}</span>)}</div>;
  if (l.type === "lowerThird") {
    const x = interpolate(local, [0, 0.4, l.dur - 0.4, l.dur], [-700, 0, 0, -700], clamp);
    return <div {...tag} style={{ position: "absolute", left: 80, top: 760, padding: "18px 34px", background: "#7b2cbf", color: "#fff", fontFamily: FONT, fontSize: 44, fontWeight: 700, transform: `translateX(${x}px)` }}>{l.text}</div>;
  }
  return null;
}

function Web({ t }: { t: number }) {
  const g = useRef<HTMLDivElement>(null), m = useRef<HTMLDivElement>(null);
  const tl = useRef<any>(null), c = useRef<any>(null);
  useLayoutEffect(() => {
    if (!tl.current) {
      tl.current = gsap.timeline({ paused: true }).to(g.current, { x: 1300, rotation: 360, duration: 6, ease: "power2.inOut" });
      c.current = animate(m.current!, { x: [0, 1300], rotate: [0, -360] }, { duration: 6, ease: "easeInOut" });
      c.current.pause();
    }
    tl.current.seek(t, false);
    c.current.time = t;
  }, [t]);
  const box = { position: "absolute", left: 200, width: 220, height: 220, borderRadius: 24 } as const;
  return <><div ref={g} data-ck-layer="gsap-box" data-ck-path="layers/gsap-box" style={{ ...box, top: 220, background: "#88ce02" }} />
    <div ref={m} data-ck-layer="motion-box" data-ck-path="layers/motion-box" style={{ ...box, top: 640, background: "#ff0088" }} /></>;
}

export function Scene({ fixture }: { fixture: any }) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const t = frame / fps;
  return <AbsoluteFill style={{ background: "#0b0b12", overflow: "hidden" }}>
    {fixture.layers.filter((l: any) => l.type === "video").map((l: any) => (
      <Sequence key={l.id} from={Math.round(l.at * fps)} durationInFrames={Math.round(l.dur * fps)} layout="none">
        <AbsoluteFill data-ck-layer={l.id} data-ck-path={`layers/${l.id}`}><Html5Video muted src={staticFile("footage.mp4")} trimBefore={Math.round(l.srcIn * fps)} style={{ width: "100%", height: "100%" }} /></AbsoluteFill>
      </Sequence>))}
    {fixture.id === "web" ? <Web t={t} /> : fixture.layers.map((l: any) => <Layer key={l.id} l={l} t={t} />)}
  </AbsoluteFill>;
}

const Root = () => <>{Object.values(FIXTURES).map((f: any) => (
  <Composition key={f.id} id={f.id} component={Scene as any} durationInFrames={f.duration * f.fps} fps={f.fps} width={f.width} height={f.height} defaultProps={{ fixture: f }} />))}</>;
if (typeof window !== "undefined" && !(window as any).__ckPreview) registerRoot(Root);
