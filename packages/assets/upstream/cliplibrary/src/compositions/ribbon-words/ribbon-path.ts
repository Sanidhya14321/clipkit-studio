import type { RibbonWordsProps } from '@/schemas';
import { TAU } from '../promo/timing';
import {
  buildCurve,
  type Curve,
  type CurveFn,
  minimumRadius,
  offsetFn,
  tToLength,
} from './ribbon-curve';

const SWOOP_GAIN = 2.2;
const RADIUS_SAMPLES = 240;
const CLAMP_STEPS = 14;

export type RibbonLane = {
  offsetY: number;
  shift: number;
};

const ribbonOvershoot = (props: RibbonWordsProps): number =>
  Math.ceil(props.ribbon.width / 2 + props.words.size + 60);

export const bendLimit = (props: RibbonWordsProps): number =>
  props.ribbon.width / 2 + props.words.size / 2;

export const breathReach = (props: RibbonWordsProps): number =>
  Math.min(props.path.breathe, bendLimit(props) * 0.5);

export const breathPhase = (props: RibbonWordsProps, frame: number): number =>
  (TAU * frame) / props.path.breatheFrames;

const shapeFn = (
  props: RibbonWordsProps,
  frameWidth: number,
  lane: RibbonLane,
  amplitude: number,
): CurveFn => {
  const { centerY, drop, waves, shift, shape } = props.path;
  const overshoot = ribbonOvershoot(props);
  const span = frameWidth + overshoot * 2;
  const phase = ((shift + lane.shift) / 360) * TAU;
  const omega = TAU * waves;
  const heading = Math.atan2(drop, frameWidth);
  const slideX = -lane.offsetY * Math.sin(heading);
  const slideY = lane.offsetY * Math.cos(heading);
  return (t) => {
    const x = -overshoot + span * t;
    const across = (x - slideX) / frameWidth;
    const base = centerY + slideY + (across - 0.5) * drop;
    if (shape === 'swoop') {
      return { x, y: base - amplitude * SWOOP_GAIN * Math.cos(phase) * Math.sin(Math.PI * across) };
    }
    const wave = amplitude * Math.sin(omega * across + phase);
    return { x, y: base + (shape === 'swell' ? wave * (0.2 + 0.8 * across) : wave) };
  };
};

const breathingFn = (
  props: RibbonWordsProps,
  frameWidth: number,
  base: CurveFn,
  phase: number,
): CurveFn => {
  const reach = breathReach(props);
  if (reach <= 0) {
    return base;
  }
  return offsetFn(base, (_, point) => reach * Math.sin(phase + (Math.PI * point.x) / frameWidth));
};

export const ribbonPoint = (
  props: RibbonWordsProps,
  frameWidth: number,
  lane: RibbonLane,
  amplitude: number,
  t: number,
) => shapeFn(props, frameWidth, lane, amplitude)(t);

export const safeAmplitude = (
  props: RibbonWordsProps,
  frameWidth: number,
  lane: RibbonLane,
): number => {
  const { amplitude } = props.path;
  const limit = bendLimit(props) + breathReach(props);
  const fits = (candidate: number) =>
    minimumRadius(shapeFn(props, frameWidth, lane, candidate), RADIUS_SAMPLES) >= limit;
  if (amplitude <= 0 || fits(amplitude)) {
    return amplitude;
  }
  let low = 0;
  let high = amplitude;
  for (let step = 0; step < CLAMP_STEPS; step++) {
    const middle = (low + high) / 2;
    if (fits(middle)) {
      low = middle;
    } else {
      high = middle;
    }
  }
  return low;
};

export type RibbonGeometry = {
  center: Curve;
  entry: number;
  sides: [Curve, Curve] | null;
};

export const buildRibbon = (
  props: RibbonWordsProps,
  frameWidth: number,
  lane: RibbonLane,
  amplitude: number,
  phase: number | null,
  withSides: boolean,
): RibbonGeometry => {
  const base = shapeFn(props, frameWidth, lane, amplitude);
  const fn = phase === null ? base : breathingFn(props, frameWidth, base, phase);
  const center = buildCurve(fn);
  const overshoot = ribbonOvershoot(props);
  const entryT = Math.max(0, overshoot - props.ribbon.width / 2 - 8) / (frameWidth + overshoot * 2);
  const reach = Math.max(0, props.ribbon.width / 2 - props.ribbon.outline / 2);
  return {
    center,
    entry: tToLength(center, entryT),
    sides: withSides
      ? [buildCurve(offsetFn(fn, () => reach)), buildCurve(offsetFn(fn, () => -reach))]
      : null,
  };
};
