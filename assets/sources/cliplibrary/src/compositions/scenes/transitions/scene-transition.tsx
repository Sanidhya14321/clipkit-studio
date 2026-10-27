import type { CSSProperties } from 'react';
import { AbsoluteFill, Composition, interpolate, useCurrentFrame, useVideoConfig } from 'remotion';
import {
  type SceneTransitionProps,
  sceneTransitionDefaults,
  sceneTransitionMeta,
  sceneTransitionSchema,
} from '@/schemas';
import { resolveEasing } from '../../kit/easing';
import type { SceneModule } from '../../scene-module';

type IrisCenter = { x: number; y: number; reach: number };

const farthestCorner = (x: number, y: number, width: number, height: number) =>
  Math.hypot(Math.max(x, 1 - x) * width, Math.max(y, 1 - y) * height);

const irisCenter = (centerX: number, centerY: number, width: number, height: number) => ({
  x: centerX,
  y: centerY,
  reach:
    farthestCorner(centerX / 100, centerY / 100, width, height) /
    farthestCorner(0.5, 0.5, width, height),
});

const coverStyle = (
  kind: SceneTransitionProps['kind'],
  cover: number,
  open: number,
  iris: IrisCenter,
): CSSProperties => {
  const bars = (1 - cover) * 50 + open * 50;
  switch (kind) {
    case 'iris':
      return open > 0
        ? {
            maskImage: `radial-gradient(circle at ${iris.x}% ${iris.y}%, transparent ${open * 101}%, black ${open * 101 + 0.5}%)`,
          }
        : { clipPath: `circle(${cover * 101 * iris.reach}% at ${iris.x}% ${iris.y}%)` };
    case 'wipe-left':
      return { transform: `translateX(${(1 - cover) * 100 - open * 100}%)` };
    case 'wipe-right':
      return { transform: `translateX(${open * 100 - (1 - cover) * 100}%)` };
    case 'wipe-up':
      return { transform: `translateY(${(1 - cover) * 100 - open * 100}%)` };
    case 'wipe-down':
      return { transform: `translateY(${open * 100 - (1 - cover) * 100}%)` };
    case 'split':
      return { clipPath: `inset(${bars}% 0 ${bars}% 0)` };
    case 'split-vertical':
      return { clipPath: `inset(0 ${bars}% 0 ${bars}%)` };
    default:
      return {};
  }
};

export const SceneTransition = ({
  kind,
  color,
  easing,
  holdFrames,
  centerX,
  centerY,
}: SceneTransitionProps) => {
  const frame = useCurrentFrame();
  const { durationInFrames, width, height } = useVideoConfig();
  const hold = Math.max(0, Math.min(holdFrames, durationInFrames - 2));
  const coverEnd = (durationInFrames - hold) / 2;
  const openStart = coverEnd + hold;
  const curve = resolveEasing(easing);
  const clamp = { extrapolateLeft: 'clamp', extrapolateRight: 'clamp', easing: curve } as const;
  const cover = interpolate(frame, [0, coverEnd], [0, 1], clamp);
  const open = interpolate(frame, [openStart, durationInFrames], [0, 1], clamp);
  const iris = irisCenter(centerX, centerY, width, height);
  return <AbsoluteFill style={{ background: color, ...coverStyle(kind, cover, open, iris) }} />;
};

export const sceneTransitionScene: SceneModule = {
  id: 'scene-transition',
  component: SceneTransition,
  render: (key) => (
    <Composition
      key={key}
      id="scene-transition"
      component={SceneTransition}
      schema={sceneTransitionSchema}
      defaultProps={sceneTransitionDefaults}
      durationInFrames={sceneTransitionMeta.durationInFrames}
      fps={sceneTransitionMeta.fps}
      width={sceneTransitionMeta.width}
      height={sceneTransitionMeta.height}
    />
  ),
};
