import type { RibbonWordsProps } from '@/schemas';
import { displayFont } from '../promo/fonts';
import { type Curve, lengthToT, tToLength } from './ribbon-curve';
import {
  bendLimit,
  breathPhase,
  breathReach,
  buildRibbon,
  type RibbonLane,
  ribbonPoint,
  safeAmplitude,
} from './ribbon-path';
import { measureLine, splitHighlight } from './ribbon-text';
import { ribbonMotion } from './ribbon-timing';

export type StrandColors = {
  ribbon: string;
  text: string;
  accent: string;
};

type StrandProps = {
  props: RibbonWordsProps;
  lane: RibbonLane;
  text: string;
  delay: number;
  colors: StrandColors;
  frame: number;
  frameWidth: number;
  pathId: string;
};

const dash = (curve: Curve, from: number, to: number) => ({
  strokeDasharray: `${Math.max(0, to - from).toFixed(2)} ${(curve.length * 3).toFixed(0)}`,
  strokeDashoffset: (-from).toFixed(2),
});

const carryOffset = (from: Curve, to: Curve, distance: number): number => {
  if (distance <= 0) {
    return distance;
  }
  if (distance >= from.length) {
    return to.length + distance - from.length;
  }
  return tToLength(to, lengthToT(from, distance));
};

const MAIN_LANE: RibbonLane = { offsetY: 0, shift: 0 };

const REST_SAMPLES = 200;

const laneRest = (
  props: RibbonWordsProps,
  frameWidth: number,
  lane: RibbonLane,
  amplitude: number,
  center: Curve,
): number => {
  if (lane.offsetY === 0) {
    return props.timing.settleAt;
  }
  const main = buildRibbon(props, frameWidth, MAIN_LANE, amplitude, null, false).center;
  const anchor = ribbonPoint(
    props,
    frameWidth,
    MAIN_LANE,
    amplitude,
    lengthToT(main, props.timing.settleAt * main.length),
  );
  let best = 0;
  let bestDistance = Number.POSITIVE_INFINITY;
  for (let index = 0; index <= REST_SAMPLES; index++) {
    const point = ribbonPoint(props, frameWidth, lane, amplitude, index / REST_SAMPLES);
    const distance = Math.hypot(point.x - anchor.x, point.y - anchor.y);
    if (distance < bestDistance) {
      bestDistance = distance;
      best = index / REST_SAMPLES;
    }
  }
  return Math.min(0.95, Math.max(0.05, tToLength(center, best) / center.length));
};

export const RibbonStrand = ({
  props,
  lane,
  text,
  delay,
  colors,
  frame,
  frameWidth,
  pathId,
}: StrandProps) => {
  const { words, ribbon, highlight } = props;
  const outline = ribbon.style === 'outline';
  const amplitude = safeAmplitude(props, frameWidth, lane);
  const still = buildRibbon(props, frameWidth, lane, amplitude, null, false);
  const live = buildRibbon(props, frameWidth, lane, amplitude, breathPhase(props, frame), outline);
  const limit = bendLimit(props);
  const settleAt = laneRest(props, frameWidth, lane, amplitude, still.center);
  const timed = {
    ...props,
    timing: {
      ...props.timing,
      drawAt: props.timing.drawAt + delay,
      settleAt,
      drift: (props.timing.drift * (limit + breathReach(props))) / limit,
    },
  };
  const { head, tail, textCenter } = ribbonMotion(
    timed,
    { length: still.center.length, entry: still.entry, textWidth: measureLine(text, words) },
    frame,
  );
  const visible = head - tail > 0.5;
  const headT = lengthToT(still.center, head);
  const tailT = lengthToT(still.center, tail);
  const textOffset = carryOffset(still.center, live.center, textCenter);
  const parts = splitHighlight(text, highlight);
  return (
    <>
      <defs>
        <path id={pathId} d={live.center.d} />
      </defs>
      {visible && live.sides === null ? (
        <path
          d={live.center.d}
          fill="none"
          stroke={colors.ribbon}
          strokeWidth={ribbon.width}
          strokeLinecap="butt"
          {...dash(live.center, tToLength(live.center, tailT), tToLength(live.center, headT))}
        />
      ) : null}
      {visible && live.sides !== null
        ? live.sides.map((side, index) => (
            <path
              key={index === 0 ? 'left' : 'right'}
              d={side.d}
              fill="none"
              stroke={colors.ribbon}
              strokeWidth={ribbon.outline}
              strokeLinecap="butt"
              {...dash(side, tToLength(side, tailT), tToLength(side, headT))}
            />
          ))
        : null}
      <text
        fontFamily={displayFont}
        fontSize={words.size}
        fontWeight={words.weight}
        letterSpacing={`${words.tracking}em`}
        fill={colors.text}
        textRendering="geometricPrecision"
        style={{ whiteSpace: 'pre' }}
      >
        <textPath href={`#${pathId}`} startOffset={textOffset.toFixed(2)} textAnchor="middle">
          {parts.map((part, index) => (
            <tspan
              key={`${index}-${part.text}`}
              dy={index === 0 ? (words.size * words.centering).toFixed(2) : undefined}
              fill={part.highlighted ? colors.accent : undefined}
            >
              {part.text}
            </tspan>
          ))}
        </textPath>
      </text>
    </>
  );
};
