import type { ComingSoonFlipsProps, FlipBeat } from '@/schemas';

export type FlipCue =
  | { kind: 'word'; beat: FlipBeat; local: number; blank: boolean }
  | { kind: 'flash' }
  | { kind: 'finale'; local: number; length: number };

export const flipCue = (
  props: ComingSoonFlipsProps,
  frame: number,
  durationInFrames: number,
): FlipCue => {
  let at = 0;
  for (const beat of props.beats) {
    if (frame < at + beat.frames) {
      return { kind: 'word', beat, local: frame - at, blank: false };
    }
    at += beat.frames;
    if (frame < at + beat.blank) {
      return { kind: 'word', beat, local: frame - at + beat.frames, blank: true };
    }
    at += beat.blank;
  }
  if (frame < at + props.flashFrames) {
    return { kind: 'flash' };
  }
  at += props.flashFrames;
  return { kind: 'finale', local: frame - at, length: Math.max(1, durationInFrames - at) };
};

export const beamCenter = (
  beam: ComingSoonFlipsProps['beam'],
  local: number,
  lastFrame: number,
): number => {
  const turnAt = Math.min(Math.max(1, beam.turnAt), Math.max(1, lastFrame - 1));
  const backLength = Math.max(1, lastFrame - turnAt);
  const half = Math.min(beam.turnFrames / 2, turnAt, backLength);
  const forward = (t: number) => beam.start + (beam.peak - beam.start) * (t / turnAt);
  const back = (t: number) => beam.peak + (beam.end - beam.peak) * ((t - turnAt) / backLength);
  if (local <= turnAt - half) {
    return forward(local);
  }
  if (local >= turnAt + half) {
    return back(local);
  }
  const s = (local - turnAt + half) / (2 * half);
  const from = forward(turnAt - half);
  const to = back(turnAt + half);
  return (1 - s) ** 2 * from + 2 * s * (1 - s) * beam.peak + s ** 2 * to;
};

export const hash = (...values: number[]): number => {
  let state = 2166136261;
  for (const value of values) {
    state ^= Math.floor(value * 997) + 0x9e3779b9;
    state = Math.imul(state, 16777619);
    state ^= state >>> 13;
    state = Math.imul(state, 0x5bd1e995);
    state ^= state >>> 15;
  }
  return (state >>> 0) / 4294967296;
};
