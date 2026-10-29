import { interpolate, useCurrentFrame } from "remotion";

/** A scrolling news ticker band for the lower third. */
export interface NewsTickerProps {
  headline: string;
  speed?: number;
}

export const NewsTicker = ({ headline, speed = 4 }: NewsTickerProps) => {
  const frame = useCurrentFrame();
  const x = interpolate(frame, [0, 300], [0, -300 * speed]);
  return <div style={{ position: "absolute", bottom: 80, transform: `translateX(${x}px)` }}>{headline}</div>;
};
