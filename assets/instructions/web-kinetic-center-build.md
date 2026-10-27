# Kinetic Center Build

Actual Animata source adapted to seekable video frame time

Status: ready implementation

## When to use

- Words joining a growing idea while previous words reflow.

## Avoid

- A long sentence that exceeds the canvas or identical arrivals for every shot.

## Adapt to this video prompt

1. Bind words to the sentence, measure widths with the actual font, tune size/gap/pushFrames. Recompose into lines when needed rather than shrinking to unreadability.
2. Keep earlier words present as the composition re-centers; choose a meaningful final alignment to hand off.

Set the approved text, aspect ratio, brand palette, target frames, footage/narration and protected reading windows first. Choose this mechanism only when it expresses the shot’s purpose. Fast/slow pacing changes travel, density, accents and holds; never blindly speed up the whole scene.

## Implementation

Ready component: inspect API through library_inspect, place with library_place, or compose via @clipkit/components with item and bindPath.

Retain actual frame-native choreography; derive all interaction/scroll progress from integral project frames. No wall-clock playback.

Editable bindings: words[0], words[1], words[2], background, color, size, gap, pushFrames.

Prove the chosen composition in the three-second sample; component inclusion is optional.

## Source and rights

https://github.com/codse/animata/tree/25c27e5a874c60a56dcc1609df3b2c4f5f8456f8

MIT — Animata

Local source: `assets/adapters/web.tsx`
