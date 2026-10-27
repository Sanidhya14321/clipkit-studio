import { Composition, useCurrentFrame } from 'remotion';
import { type WordPunchProps, wordPunchDefaults, wordPunchMeta, wordPunchSchema } from '@/schemas';
import { displayType, KineticLayer } from '../../promo/kinetic';
import { land, launch, tween } from '../../promo/timing';
import { highlightSet, normalize, wordsOf } from '../../promo/words';
import type { SceneModule } from '../../scene-module';

const PUNCTUATION = /[.,!?;:]+$/;

const TYPE_GAP = 2;

const markOf = (word: string): string => word.match(PUNCTUATION)?.[0] ?? '';

const JUSTIFY = { left: 'flex-start', center: 'center', right: 'flex-end' } as const;

const TypedWord = ({
  word,
  frame,
  start,
  typeFrames,
  leaveAt,
  isLast,
  color,
  accentMark,
}: {
  word: string;
  frame: number;
  start: number;
  typeFrames: number;
  leaveAt: number;
  isLast: boolean;
  color: string;
  accentMark: string | null;
}) => {
  const mark = markOf(word);
  const letters = Array.from(word.slice(0, word.length - mark.length));
  const typed = Math.min(letters.length, Math.max(0, (frame - start) / typeFrames));
  const deleted = isLast
    ? 0
    : Math.min(letters.length, Math.max(0, (frame - leaveAt) / typeFrames));
  const shown = letters.slice(Math.floor(deleted), Math.ceil(typed));
  const anchorRight = deleted > 0;
  const markOpacity =
    deleted >= letters.length ? 1 - Math.min(1, (deleted - letters.length) * 2) : 1;
  return (
    <span style={{ position: 'relative', display: 'inline-block' }}>
      <span style={{ visibility: 'hidden' }}>{word}</span>
      <span
        style={{
          position: 'absolute',
          top: 0,
          [anchorRight ? 'right' : 'left']: 0,
          color,
          whiteSpace: 'pre',
        }}
      >
        {shown.map((letter, index) => {
          const at = Math.floor(deleted) + index;
          const fadeIn = Math.min(1, typed - at);
          const fadeOut = Math.min(1, Math.max(0, deleted - at));
          return (
            <span key={`${at}:${letter}`} style={{ opacity: Math.max(0, fadeIn) * (1 - fadeOut) }}>
              {letter}
            </span>
          );
        })}
        <span style={{ opacity: markOpacity, color: accentMark ?? color }}>{mark}</span>
      </span>
    </span>
  );
};

const PunchedWord = ({ word, accentMark }: { word: string; accentMark: string | null }) => {
  const mark = markOf(word);
  if (accentMark === null || mark === '') {
    return word;
  }
  return (
    <>
      {word.slice(0, word.length - mark.length)}
      <span style={{ color: accentMark }}>{mark}</span>
    </>
  );
};

const punchMotion = ({
  frame,
  start,
  leaveAt,
  isLast,
  swapFrames,
  punch,
  rise,
  wordSize,
}: {
  frame: number;
  start: number;
  leaveAt: number;
  isLast: boolean;
  swapFrames: number;
  punch: number;
  rise: number;
  wordSize: number;
}) => {
  const arrive = tween(frame, [start, start + swapFrames], [0, 1], (t) => t);
  const scale = land(frame, [start, start + swapFrames + 8], [1 + punch, 1], 1 / wordSize);
  const leave = isLast ? 0 : tween(frame, [leaveAt, leaveAt + swapFrames], [0, 1], (t) => t);
  const settle = land(frame, [start, start + swapFrames + 4], [wordSize * rise, 0]);
  const lift = isLast ? 0 : launch(frame, [leaveAt, leaveAt + swapFrames], [0, -wordSize * rise]);
  return {
    opacity: arrive * (1 - leave),
    transform: `translateY(${(settle + lift).toFixed(2)}px) scale(${(scale * (1 - leave * 0.06)).toFixed(5)})`,
  };
};

export const WordPunch = ({
  text,
  highlight,
  style,
  size,
  shrink,
  weight,
  tracking,
  accentPunctuation,
  delay,
  wordFrames,
  swapFrames,
  typeFrames,
  punch,
  rise,
  placement,
  look,
  motion,
}: WordPunchProps) => {
  const frame = useCurrentFrame() * motion.speed;
  const words = wordsOf(text);
  const accented = highlightSet(highlight);
  const accentMark = accentPunctuation ? look.accent : null;
  const cycle = (word: string) =>
    style === 'type' ? wordFrames + word.length * typeFrames + TYPE_GAP : wordFrames;
  const starts = words.map((_, index) =>
    words.slice(0, index).reduce((at, word) => at + cycle(word), delay),
  );
  return (
    <KineticLayer look={look} motion={motion} placement={placement}>
      <div style={{ position: 'relative', height: size, width: '100%' }}>
        {words.map((word, index) => {
          const start = starts[index] ?? delay;
          const leaveAt = start + wordFrames;
          const isLast = index === words.length - 1;
          const wordSize = size * (1 - shrink) ** index;
          const color = accented.has(normalize(word)) ? look.accent : look.ink;
          const frameStyle = {
            ...displayType(wordSize, weight, tracking),
            position: 'absolute' as const,
            inset: 0,
            display: 'flex',
            justifyContent: JUSTIFY[placement.horizontal],
            alignItems: 'center',
          };
          if (style === 'type') {
            const lettersOut = start + wordFrames + word.length * typeFrames;
            if (frame < start - 2 || (!isLast && frame > lettersOut)) {
              return null;
            }
            return (
              <div key={`${index}:${word}`} style={frameStyle}>
                <TypedWord
                  word={word}
                  frame={frame}
                  start={start}
                  typeFrames={typeFrames}
                  leaveAt={leaveAt}
                  isLast={isLast}
                  color={color}
                  accentMark={accentMark}
                />
              </div>
            );
          }
          if (frame < start || (!isLast && frame > leaveAt + swapFrames)) {
            return null;
          }
          return (
            <div
              key={`${index}:${word}`}
              style={{
                ...frameStyle,
                color,
                ...punchMotion({
                  frame,
                  start,
                  leaveAt,
                  isLast,
                  swapFrames,
                  punch,
                  rise,
                  wordSize,
                }),
                willChange: 'transform',
              }}
            >
              <PunchedWord word={word} accentMark={accentMark} />
            </div>
          );
        })}
      </div>
    </KineticLayer>
  );
};

export const wordPunchScene: SceneModule = {
  id: 'word-punch',
  component: WordPunch,
  render: (key) => (
    <Composition
      key={key}
      id="word-punch"
      component={WordPunch}
      schema={wordPunchSchema}
      defaultProps={wordPunchDefaults}
      durationInFrames={wordPunchMeta.durationInFrames}
      fps={wordPunchMeta.fps}
      width={wordPunchMeta.width}
      height={wordPunchMeta.height}
    />
  ),
};
