import { interpolate, useCurrentFrame, Easing } from "remotion";

/** Slides its children in from the left. */
export type SlideInProps = {
  children: React.ReactNode;
  distance: number;
  durationInFrames?: number;
};

export function SlideIn({ children, distance, durationInFrames = 20 }: SlideInProps) {
  const frame = useCurrentFrame();
  const x = interpolate(frame, [0, durationInFrames], [-distance, 0], {
    easing: Easing.out(Easing.cubic),
    extrapolateRight: "clamp",
  });
  return <div style={{ transform: `translateX(${x}px)` }}>{children}</div>;
}
