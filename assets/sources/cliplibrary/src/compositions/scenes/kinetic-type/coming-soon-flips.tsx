import { AbsoluteFill, Composition, useCurrentFrame, useVideoConfig } from 'remotion';
import {
  type ComingSoonFlipsProps,
  comingSoonFlipsDefaults,
  comingSoonFlipsMeta,
  comingSoonFlipsSchema,
  type FlipTone,
} from '@/schemas';
import { FlipGrain } from '../../coming-soon-flips/flip-grain';
import { FlipWord } from '../../coming-soon-flips/flip-word';
import { LitWall } from '../../coming-soon-flips/light-streak';
import { beamCenter, flipCue } from '../../coming-soon-flips/timeline';
import { TrackedTitle } from '../../coming-soon-flips/tracked-title';
import { TransitionFrame } from '../../promo/transition-frame';
import type { SceneModule } from '../../scene-module';

const CueLayer = ({ props, frame }: { props: ComingSoonFlipsProps; frame: number }) => {
  const { durationInFrames } = useVideoConfig();
  const { look } = props;
  const cue = flipCue(props, frame, durationInFrames);
  if (cue.kind === 'word') {
    const light = cue.beat.tone === 'light';
    return (
      <>
        {look.showBackdrop ? (
          <AbsoluteFill style={{ background: light ? look.light : look.dark }} />
        ) : null}
        {cue.blank ? null : (
          <FlipWord
            beat={cue.beat}
            words={props.words}
            local={cue.local}
            color={light ? look.onLight : look.onDark}
            barColor={light ? props.words.bar.onLight : props.words.bar.onDark}
          />
        )}
      </>
    );
  }
  if (cue.kind === 'flash') {
    return <LitWall props={props} center={null} />;
  }
  return (
    <>
      {look.showBackdrop ? <AbsoluteFill style={{ background: look.dark }} /> : null}
      <LitWall props={props} center={beamCenter(props.beam, cue.local, cue.length - 1)} />
      <TrackedTitle title={props.title} local={cue.local} />
    </>
  );
};

const toneAt = (props: ComingSoonFlipsProps, frame: number, durationInFrames: number): FlipTone => {
  const cue = flipCue(props, frame, durationInFrames);
  if (cue.kind === 'word') {
    return cue.beat.tone;
  }
  return cue.kind === 'flash' ? 'light' : 'dark';
};

const ComingSoonFlips = (props: ComingSoonFlipsProps) => {
  const frame = Math.floor(useCurrentFrame() * props.motion.speed);
  const { durationInFrames } = useVideoConfig();
  return (
    <TransitionFrame motion={props.motion}>
      <CueLayer props={props} frame={frame} />
      <FlipGrain
        amount={props.look.grain}
        live={props.look.liveGrain}
        frame={frame}
        tone={toneAt(props, frame, durationInFrames)}
      />
    </TransitionFrame>
  );
};

export const comingSoonFlipsScene: SceneModule = {
  id: 'coming-soon-flips',
  component: ComingSoonFlips,
  render: (key) => (
    <Composition
      key={key}
      id="coming-soon-flips"
      component={ComingSoonFlips}
      schema={comingSoonFlipsSchema}
      defaultProps={comingSoonFlipsDefaults}
      durationInFrames={comingSoonFlipsMeta.durationInFrames}
      fps={comingSoonFlipsMeta.fps}
      width={comingSoonFlipsMeta.width}
      height={comingSoonFlipsMeta.height}
    />
  ),
};
