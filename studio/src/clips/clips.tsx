// The 8 native core clips (spec 7: written in Remotion, fully instrumented). Each is a pure
// function of its props and the time into the clip. Every part a user may point at carries
// `data-ck-part`; the root carries `data-ck-layer` and `data-ck-clip` (set by ClipLayer).
// Media sources render as a framed panel naming the source until proxies reach the preview.
import type { CSSProperties, ReactNode } from "react";
import {
  type Box,
  type CoreClip,
  type Props,
  boxStyle,
  counterText,
  fontPx,
  list,
  num,
  presence,
  staggerDelay,
  str,
  textLines,
} from "./motion";

export type ClipArgs = {
  props: Props;
  /** Seconds into the clip. */
  t: number;
  /** Clip duration in seconds. */
  dur: number;
  width: number;
  height: number;
};

const family = (props: Props, key = "font") =>
  `${JSON.stringify(str(props, key, "Inter"))}, system-ui, sans-serif`;

const full: CSSProperties = { position: "absolute", inset: 0 };

/** Fade and rise by `p` (0 hidden, 1 in place). */
const rise = (p: number, height: number): CSSProperties => ({
  opacity: p,
  transform: `translateY(${((1 - p) * height * 0.02).toFixed(2)}px)`,
});

function Media({ source, radius }: { source: string; radius: number }) {
  return (
    <div
      data-ck-part="media"
      data-ck-source={source}
      style={{
        ...full,
        borderRadius: radius,
        background: "linear-gradient(135deg, #2a2f3a, #161a22)",
        color: "rgba(255,255,255,0.55)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        fontFamily: "system-ui, sans-serif",
        overflow: "hidden",
      }}
    >
      {source}
    </div>
  );
}

function Lines({
  lines,
  props,
  t,
  dur,
  height,
  part,
}: {
  lines: string[];
  props: Props;
  t: number;
  dur: number;
  height: number;
  part: string;
}) {
  return (
    <>
      {lines.map((line, i) => (
        <div
          key={i}
          data-ck-part={part}
          style={rise(presence(t, dur, props, staggerDelay(props, i)), height)}
        >
          {line}
        </div>
      ))}
    </>
  );
}

function Backdrop({ props, t, dur }: ClipArgs) {
  const color = str(props, "color", "#111");
  const accent = str(props, "accent");
  const source = str(props, "source");
  // Media fills the frame when given; otherwise the color with a slow accent glow.
  const drift = (t / Math.max(dur, 0.001)) * 10;
  return (
    <div data-ck-part="fill" style={{ ...full, background: color }}>
      {source ? <Media source={source} radius={0} /> : null}
      {accent && !source ? (
        <div
          data-ck-part="accent"
          style={{
            ...full,
            background: `radial-gradient(circle at ${30 + drift}% 70%, ${accent}33, transparent 60%)`,
          }}
        />
      ) : null}
    </div>
  );
}

function KineticText({ props, t, dur, height }: ClipArgs) {
  const size = fontPx(
    props,
    height,
    str(props, "size") === "small" ? 0.03 : 0.07,
  );
  const kicker = str(props, "kicker");
  const index = str(props, "index");
  const subtitle = str(props, "subtitle");
  return (
    <div
      style={{
        position: "absolute",
        ...boxStyle(props, { x: 0.08, y: 0.4, w: 0.84, h: 0.2 }),
        color: str(props, "color", "#fff"),
        fontFamily: family(props),
        fontSize: size,
        lineHeight: 1.15,
        textAlign: "center",
        display: "flex",
        flexDirection: "column",
        justifyContent: "center",
      }}
    >
      {kicker || index ? (
        <div
          data-ck-part="kicker"
          style={{
            fontSize: size * 0.4,
            ...rise(presence(t, dur, props), height),
          }}
        >
          {index ? `${index} ` : ""}
          {kicker}
        </div>
      ) : null}
      <Lines
        lines={textLines(props)}
        props={props}
        t={t}
        dur={dur}
        height={height}
        part="line"
      />
      {subtitle ? (
        <div
          data-ck-part="subtitle"
          style={{
            fontSize: size * 0.45,
            ...rise(presence(t, dur, props, staggerDelay(props, 2)), height),
          }}
        >
          {subtitle}
        </div>
      ) : null}
    </div>
  );
}

function LowerThird({ props, t, dur, width, height }: ClipArgs) {
  const p = presence(t, dur, props);
  const size = fontPx(props, height, 0.035);
  return (
    <div
      style={{
        position: "absolute",
        ...boxStyle(props, { x: 0.06, y: 0.74, w: 0.6, h: 0.14 }),
        fontFamily: family(props),
        display: "flex",
        flexDirection: "column",
        justifyContent: "center",
        transform: `translateX(${((p - 1) * width * 0.05).toFixed(2)}px)`,
        opacity: p,
      }}
    >
      <div
        data-ck-part="name"
        style={{
          alignSelf: "flex-start",
          background: str(props, "panel", "#fff"),
          color: str(props, "color", "#111"),
          fontSize: size,
          padding: "0.2em 0.5em",
        }}
      >
        {str(props, "name", str(props, "text"))}
      </div>
      {str(props, "role") ? (
        <div
          data-ck-part="role"
          style={{
            alignSelf: "flex-start",
            color: "#fff",
            background: "rgba(0,0,0,0.55)",
            fontSize: size * 0.65,
            padding: "0.2em 0.6em",
          }}
        >
          {str(props, "role")}
        </div>
      ) : null}
    </div>
  );
}

function BulletList({ props, t, dur, height }: ClipArgs) {
  const size = fontPx(props, height, 0.04);
  const marker = str(props, "marker", "#fff");
  const title = str(props, "title");
  return (
    <div
      style={{
        position: "absolute",
        ...boxStyle(props, { x: 0.1, y: 0.2, w: 0.8, h: 0.6 }),
        color: str(props, "color", "#fff"),
        fontFamily: family(props),
        fontSize: size,
        display: "flex",
        flexDirection: "column",
        justifyContent: "center",
        // `gap` is the look's spacing unit (fraction of height); 3 units between items.
        gap: num(props, "gap", size / height / 6) * height * 3,
      }}
    >
      {title ? (
        <div
          data-ck-part="title"
          style={{
            fontFamily: family(props, "titleFont"),
            fontSize: size * 1.4,
            ...rise(presence(t, dur, props), height),
          }}
        >
          {title}
        </div>
      ) : null}
      {list(props, "items").map((item, i) => (
        <div
          key={i}
          data-ck-part="item"
          style={{
            display: "flex",
            gap: "0.5em",
            alignItems: "baseline",
            ...rise(
              presence(t, dur, props, staggerDelay(props, i + 1)),
              height,
            ),
          }}
        >
          <span style={{ color: marker }}>■</span>
          {item}
        </div>
      ))}
    </div>
  );
}

function QuoteCard({ props, t, dur, width, height }: ClipArgs) {
  const p = presence(t, dur, props);
  const size = fontPx(props, height, 0.045);
  const radius = num(props, "radius", 0.02) * Math.min(width, height);
  return (
    <div
      data-ck-part="card"
      style={{
        position: "absolute",
        ...boxStyle(props, { x: 0.1, y: 0.25, w: 0.8, h: 0.5 }),
        background: str(props, "card", "#222"),
        borderRadius: radius,
        padding: size,
        boxSizing: "border-box",
        color: str(props, "color", "#fff"),
        fontFamily: family(props),
        display: "flex",
        flexDirection: "column",
        justifyContent: "center",
        opacity: p,
        transform: `scale(${(0.96 + 0.04 * p).toFixed(4)})`,
      }}
    >
      <div
        data-ck-part="mark"
        style={{
          color: str(props, "mark", "#fc0"),
          fontSize: size * 2,
          lineHeight: 0.6,
        }}
      >
        “
      </div>
      <div data-ck-part="quote" style={{ fontSize: size, lineHeight: 1.25 }}>
        {textLines(props, "quote").join(" ")}
      </div>
      <div
        data-ck-part="author"
        style={{
          fontSize: size * 0.55,
          marginTop: size * 0.6,
          ...rise(presence(t, dur, props, staggerDelay(props, 2)), height),
        }}
      >
        {str(props, "author")}
        {str(props, "detail") ? `, ${str(props, "detail")}` : ""}
      </div>
    </div>
  );
}

function SplitCompare({ props, t, dur, width, height }: ClipArgs) {
  // The divider sweeps from the left edge to the middle as the clip enters.
  const p = presence(t, dur, { ...props, exit: "linear" });
  const split = 0.5 * p;
  const size = fontPx(props, height, 0.035);
  const label = (
    text: string,
    part: string,
    side: "left" | "right",
  ): ReactNode => (
    <div
      data-ck-part={part}
      style={{
        position: "absolute",
        bottom: "8%",
        [side]: "4%",
        fontFamily: family(props),
        fontSize: size,
        color: str(props, "color", "#fff"),
        background: "rgba(0,0,0,0.5)",
        padding: "0.2em 0.5em",
      }}
    >
      {text}
    </div>
  );
  return (
    <div style={full}>
      <div
        data-ck-part="before"
        style={{ ...full, right: `${(1 - split) * 100}%`, overflow: "hidden" }}
      >
        <Media source={str(props, "beforeSource", "before")} radius={0} />
        {label(str(props, "beforeLabel"), "before-label", "left")}
      </div>
      <div
        data-ck-part="after"
        style={{ ...full, left: `${split * 100}%`, overflow: "hidden" }}
      >
        <Media source={str(props, "afterSource", "after")} radius={0} />
        {label(str(props, "afterLabel"), "after-label", "right")}
      </div>
      <div
        data-ck-part="divider"
        style={{
          position: "absolute",
          top: 0,
          bottom: 0,
          left: `calc(${split * 100}% - ${width * 0.002}px)`,
          width: width * 0.004,
          background: str(props, "divider", "#fff"),
        }}
      />
    </div>
  );
}

function StatCounter({ props, t, dur, height }: ClipArgs) {
  const size = fontPx(props, height, 0.12);
  const unit = str(props, "unit");
  const label = str(props, "label");
  return (
    <div
      style={{
        position: "absolute",
        ...boxStyle(props, { x: 0.1, y: 0.3, w: 0.8, h: 0.4 }),
        fontFamily: family(props),
        color: str(props, "color", "#fff"),
        textAlign: "center",
        display: "flex",
        flexDirection: "column",
        justifyContent: "center",
        ...rise(presence(t, dur, props), height),
      }}
    >
      <div data-ck-part="value" style={{ fontSize: size, lineHeight: 1 }}>
        {counterText(str(props, "value"), t, dur, props)}
        {unit ? (
          <span data-ck-part="unit" style={{ fontSize: size * 0.5 }}>
            {unit}
          </span>
        ) : null}
      </div>
      {label ? (
        <div
          data-ck-part="label"
          style={{
            fontSize: size * 0.3,
            color: str(props, "labelColor", str(props, "color", "#fff")),
          }}
        >
          {label}
        </div>
      ) : null}
    </div>
  );
}

function DeviceFrame({ props, t, dur, width, height }: ClipArgs) {
  const p = presence(t, dur, props);
  const portrait = height > width;
  // A phone in portrait frames, a laptop-like screen in landscape ones.
  const fallback: Box = portrait
    ? { x: 0.2, y: 0.12, w: 0.6, h: 0.6 }
    : { x: 0.2, y: 0.12, w: 0.6, h: 0.62 };
  const radius = num(props, "radius", 0.02) * Math.min(width, height) * 2;
  return (
    <div
      data-ck-part="device"
      style={{
        position: "absolute",
        ...boxStyle(props, fallback),
        border: `${Math.round(Math.min(width, height) * 0.012)}px solid #0b0b0d`,
        borderRadius: radius,
        boxSizing: "border-box",
        background: "#000",
        boxShadow: "0 30px 80px rgba(0,0,0,0.45)",
        overflow: "hidden",
        opacity: p,
        transform: `translateY(${((1 - p) * height * 0.08).toFixed(2)}px)`,
      }}
    >
      <Media source={str(props, "source", "screen")} radius={radius * 0.6} />
    </div>
  );
}

export const CLIPS: Record<CoreClip, (args: ClipArgs) => ReactNode> = {
  backdrop: Backdrop,
  "bullet-list": BulletList,
  "device-frame": DeviceFrame,
  "kinetic-text": KineticText,
  "lower-third": LowerThird,
  "quote-card": QuoteCard,
  "split-compare": SplitCompare,
  "stat-counter": StatCounter,
};
