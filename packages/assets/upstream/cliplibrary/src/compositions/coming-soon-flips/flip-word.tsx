import { AbsoluteFill, useVideoConfig } from 'remotion';
import type { ComingSoonFlipsProps, FlipBeat } from '@/schemas';
import { FLIP_FACES } from './fonts';

type Words = ComingSoonFlipsProps['words'];

const faceFor = (beat: FlipBeat, words: Words, local: number) => {
  if (!beat.cycleFonts || words.cycle.length === 0) {
    return { ...FLIP_FACES.condensed, weight: words.weight };
  }
  const step = Math.floor(local / Math.max(1, words.cycleHold)) % words.cycle.length;
  const font = words.cycle[step] ?? 'condensed';
  return font === 'condensed'
    ? { ...FLIP_FACES.condensed, weight: words.weight }
    : FLIP_FACES[font];
};

export const FlipWord = ({
  beat,
  words,
  local,
  color,
  barColor,
}: {
  beat: FlipBeat;
  words: Words;
  local: number;
  color: string;
  barColor: string;
}) => {
  const { height } = useVideoConfig();
  const face = faceFor(beat, words, local);
  const { bar } = words;
  const grown =
    bar.startWidth + (1 - bar.startWidth) * Math.min(1, local / Math.max(1, beat.frames - 1));
  return (
    <AbsoluteFill>
      {bar.show && beat.text.trim() !== '' ? (
        <div
          style={{
            position: 'absolute',
            left: '50%',
            top: words.centerY * height + words.size / 2 + bar.gap,
            width: bar.length,
            height: bar.thickness,
            marginLeft: -bar.length / 2,
            background: barColor,
            transform: `scaleX(${grown})`,
          }}
        />
      ) : null}
      <div
        style={{
          position: 'absolute',
          left: 0,
          right: 0,
          top: words.centerY * height,
          transform: 'translateY(-50%)',
          textAlign: 'center',
          fontFamily: face.family,
          fontWeight: face.weight,
          fontSize: words.size * face.scale,
          letterSpacing: `${words.tracking}em`,
          lineHeight: 1,
          whiteSpace: 'pre',
          color,
        }}
      >
        {beat.text}
      </div>
    </AbsoluteFill>
  );
};
