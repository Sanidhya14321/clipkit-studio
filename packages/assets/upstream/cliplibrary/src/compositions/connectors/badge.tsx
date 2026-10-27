import { Easing } from 'remotion';
import type { ConnectorBadge, ConnectorChalk } from '@/schemas';
import { smoothstep } from '../promo/ink-geometry';
import { glide, smooth } from '../promo/timing';
import { BADGE_DESIGNS } from './badge-designs';
import type { BadgePart, ShapePart, Tone, Window, WordPart } from './badge-parts';
import { BrushStroke } from './brush';
import { CONNECTOR_FONTS } from './fonts';

export const BADGE_WIDTH = 600;
export const BADGE_HEIGHT = 360;

const ERASE_LEAD = 0.3;
const overshoot = Easing.out(Easing.back(1.7));

const clamp01 = (value: number): number => Math.min(1, Math.max(0, value));

const within = (progress: number, [from, to]: Window): number =>
  clamp01((progress - from) / Math.max(0.0001, to - from));

const eraseWindow = (index: number, count: number): Window => {
  const start = (index / Math.max(1, count)) * ERASE_LEAD;
  return [start, start + 1 - ERASE_LEAD];
};

const TONE_COLORS: Record<Tone, (badge: ConnectorBadge) => string> = {
  line: (badge) => badge.line,
  fill: (badge) => badge.fill,
  text: (badge) => badge.text,
  shade: (badge) => `color-mix(in oklab, ${badge.fill}, #000 16%)`,
  deep: (badge) => `color-mix(in oklab, ${badge.fill}, #000 42%)`,
};

const toneColor = (badge: ConnectorBadge, tone: Tone): string => TONE_COLORS[tone](badge);

const ShapeView = ({
  part,
  id,
  color,
  shown,
  gone,
}: {
  part: ShapePart;
  id: string;
  color: string;
  shown: number;
  gone: number;
}) => {
  if (shown <= 0 || gone >= 1) {
    return null;
  }
  const rotate = part.rotate ?? 0;
  if (part.reveal === 'pop') {
    const grow = overshoot(shown) * (1 - glide(gone));
    const spin = (part.spin ?? 0) * (1 - smooth(shown));
    const [ox, oy] = part.origin;
    return (
      <g
        transform={`rotate(${rotate}) translate(${ox} ${oy}) rotate(${spin.toFixed(2)}) scale(${Math.max(0, grow).toFixed(4)}) translate(${-ox} ${-oy})`}
      >
        <path d={part.d} fill={color} />
      </g>
    );
  }
  const [left, right] = part.span ?? [-BADGE_WIDTH / 2, BADGE_WIDTH / 2];
  const width = right - left;
  const open = smooth(shown);
  const close = glide(gone);
  const clip =
    part.reveal === 'unroll'
      ? { x: left + width * close, w: width * (open - close) }
      : {
          x: left + (width * (1 - open * (1 - close))) / 2,
          w: width * open * (1 - close),
        };
  return (
    <g transform={`rotate(${rotate})`}>
      <clipPath id={id}>
        <rect x={clip.x} y={-BADGE_HEIGHT} width={Math.max(0, clip.w)} height={BADGE_HEIGHT * 2} />
      </clipPath>
      <path
        d={part.d}
        fill={color}
        clipPath={`url(#${id})`}
        opacity={1 - smoothstep(0.45, 0.88, gone)}
      />
    </g>
  );
};

const SOFT_EM = 0.6;
const INSET_EM = 0.3;

const write = Easing.bezier(0.3, 0.2, 0.5, 1);

const wipeStop = (progress: number): string =>
  `calc(${INSET_EM}em + (100% - ${(INSET_EM * 2 - SOFT_EM).toFixed(2)}em) * ${progress.toFixed(4)})`;

const ANCHOR_SHIFT = { start: '0%', center: '-50%', end: '-100%' } as const;

const ANCHOR_ORIGIN = { start: '0% 50%', center: '50% 50%', end: '100% 50%' } as const;

export const WordView = ({
  part,
  text,
  color,
  shown,
  gone,
}: {
  part: WordPart;
  text: string;
  color: string;
  shown: number;
  gone: number;
}) => {
  if (shown <= 0 || gone >= 1) {
    return null;
  }
  const outline = part.outline ?? 0;
  const base = {
    position: 'absolute' as const,
    left: BADGE_WIDTH / 2 + part.x,
    top: BADGE_HEIGHT / 2 + part.y,
    transform: `translate(${ANCHOR_SHIFT[part.anchor ?? 'center']}, -50%) rotate(${part.rotate ?? 0}deg)`,
    transformOrigin: ANCHOR_ORIGIN[part.anchor ?? 'center'],
    fontFamily: CONNECTOR_FONTS[part.font],
    fontSize: part.size,
    lineHeight: 1,
    letterSpacing: `${part.tracking ?? 0}em`,
    whiteSpace: 'pre' as const,
    color: outline > 0 ? 'transparent' : color,
    WebkitTextStroke: outline > 0 ? `${outline}px ${color}` : undefined,
    padding: '0.35em 0.45em',
  };
  if (part.reveal === 'wipe') {
    const head = write(shown) * (part.unwrite === true ? 1 - glide(gone) : 1);
    const tail = part.unwrite === true ? 0 : glide(gone);
    const mask = `linear-gradient(${part.angle ?? 100}deg, transparent calc(${wipeStop(tail)} - ${SOFT_EM}em), #000 ${wipeStop(tail)}, #000 calc(${wipeStop(head)} - ${SOFT_EM}em), transparent ${wipeStop(head)})`;
    return (
      <div
        style={{
          ...base,
          maskImage: mask,
          WebkitMaskImage: mask,
          maskRepeat: 'no-repeat',
          WebkitMaskRepeat: 'no-repeat',
        }}
      >
        {text}
      </div>
    );
  }
  const letters = [...text];
  const count = letters.length;
  return (
    <div style={base}>
      {letters.map((letter, index) => {
        const lead = count > 1 ? (index / (count - 1)) * 0.4 : 0;
        const pop = clamp01((shown - lead) / 0.6);
        const fall = clamp01((gone - lead * 0.6) / 0.76);
        const scale = (0.3 + 0.7 * overshoot(pop)) * (1 - glide(fall));
        const tilt = (index % 2 === 0 ? -14 : 12) * (1 - pop);
        return (
          <span
            key={`${index.toString()}${letter}`}
            style={{
              display: 'inline-block',
              opacity: Math.min(1, pop * 1.6),
              transform: `translateY(${((1 - pop) * 0.2).toFixed(3)}em) rotate(${tilt.toFixed(2)}deg) scale(${Math.max(0, scale).toFixed(4)})`,
            }}
          >
            {letter}
          </span>
        );
      })}
    </div>
  );
};

const PartView = ({
  part,
  badge,
  chalk,
  draw,
  erase,
  index,
  count,
  id,
}: {
  part: BadgePart;
  badge: ConnectorBadge;
  chalk: ConnectorChalk;
  draw: number;
  erase: number;
  index: number;
  count: number;
  id: string;
}) => {
  const shown = within(draw, part.at);
  const gone = within(erase, part.out ?? eraseWindow(count - 1 - index, count));
  const color = toneColor(badge, part.tone);
  if (part.kind === 'stroke') {
    return (
      <g transform={part.rotate === undefined ? undefined : `rotate(${part.rotate})`}>
        <BrushStroke
          points={part.points}
          width={chalk.width * part.width}
          color={color}
          head={write(shown)}
          tail={glide(gone)}
          seed={index * 37 + 11}
        />
      </g>
    );
  }
  if (part.kind === 'shape') {
    return <ShapeView part={part} id={`${id}-${index}`} color={color} shown={shown} gone={gone} />;
  }
  return null;
};

const BadgeArt = ({
  badge,
  chalk,
  draw,
  erase,
  id,
}: {
  badge: ConnectorBadge;
  chalk: ConnectorChalk;
  draw: number;
  erase: number;
  id: string;
}) => {
  const parts = BADGE_DESIGNS[badge.variant]();
  return (
    <div style={{ position: 'relative', width: BADGE_WIDTH, height: BADGE_HEIGHT }}>
      <svg
        viewBox={`${-BADGE_WIDTH / 2} ${-BADGE_HEIGHT / 2} ${BADGE_WIDTH} ${BADGE_HEIGHT}`}
        width={BADGE_WIDTH}
        height={BADGE_HEIGHT}
        style={{ position: 'absolute', inset: 0, overflow: 'visible' }}
        aria-hidden="true"
      >
        {parts.map((part, index) => (
          <PartView
            key={`${part.kind}-${index.toString()}`}
            part={part}
            badge={badge}
            chalk={chalk}
            draw={draw}
            erase={erase}
            index={index}
            count={parts.length}
            id={id}
          />
        ))}
      </svg>
      {parts.map((part, index) =>
        part.kind === 'word' ? (
          <WordView
            key={`word-${index.toString()}`}
            part={part}
            text={badge.word}
            color={toneColor(badge, part.tone)}
            shown={within(draw, part.at)}
            gone={within(erase, part.out ?? eraseWindow(parts.length - 1 - index, parts.length))}
          />
        ) : null,
      )}
    </div>
  );
};

export const chalkFilterStyle = (filterId: string, chalk: ConnectorChalk): string =>
  `url(#${filterId}) drop-shadow(0 14px 16px rgba(0, 0, 0, ${(chalk.shadow * 0.7).toFixed(3)}))`;

export const Badge = ({
  badge,
  chalk,
  draw,
  erase,
  x,
  y,
  scale,
  id,
  filterId,
}: {
  badge: ConnectorBadge;
  chalk: ConnectorChalk;
  draw: number;
  erase: number;
  x: number;
  y: number;
  scale: number;
  id: string;
  filterId: string;
}) => {
  if (draw <= 0 || erase >= 1) {
    return null;
  }
  return (
    <div
      style={{
        position: 'absolute',
        left: x - BADGE_WIDTH / 2,
        top: y - BADGE_HEIGHT / 2,
        width: BADGE_WIDTH,
        height: BADGE_HEIGHT,
        transform: `scale(${scale})`,
        filter: chalkFilterStyle(filterId, chalk),
      }}
    >
      <BadgeArt badge={badge} chalk={chalk} draw={draw} erase={erase} id={id} />
    </div>
  );
};
