import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";

/** A title that fades in with a soft glow. Use for hooks on dark backgrounds. */
export interface GlowTitleProps {
  text: string;
  color?: string;
  glow?: number;
}

export const GlowTitle: React.FC<GlowTitleProps> = ({ text, color = "#ffffff", glow = 24 }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const enter = spring({ frame, fps, config: { damping: 200 } });
  const opacity = interpolate(frame, [0, 15], [0, 1], { extrapolateRight: "clamp" });
  return (
    <AbsoluteFill style={{ justifyContent: "center", alignItems: "center" }}>
      <h1 style={{ color, opacity, transform: `scale(${0.9 + 0.1 * enter})`, textShadow: `0 0 ${glow}px ${color}` }}>
        {text}
      </h1>
    </AbsoluteFill>
  );
};
