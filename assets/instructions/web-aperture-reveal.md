# Aperture Reveal

Actual Animata source adapted to seekable video frame time

Status: ready implementation

## When to use

- Revealing real footage, a texture, screenshot or evidence through a composed aperture.

## Avoid

- Invented product evidence or a stock-photo hero unrelated to the brief.

## Adapt to this video prompt

1. Use a registered imageSourceId for real media; bind lead/trail/caption, delays, expansion and type size. Preserve the source media.
2. Translate scroll progression into frame progress; the aperture edge becomes a next-shot mask or frame.

Set the approved text, aspect ratio, brand palette, target frames, footage/narration and protected reading windows first. Choose this mechanism only when it expresses the shot’s purpose. Fast/slow pacing changes travel, density, accents and holds; never blindly speed up the whole scene.

## Implementation

Ready component: inspect API through library_inspect, place with library_place, or compose via @clipkit/components with item and bindPath.

Retain actual frame-native choreography; derive all interaction/scroll progress from integral project frames. No wall-clock playback.

Editable bindings: lead, trail, caption, size, captionSize, background, color, imageSourceId, delayFrames, expandFrames.

Prove the chosen composition in the three-second sample; component inclusion is optional.

## Source and rights

https://github.com/codse/animata/tree/25c27e5a874c60a56dcc1609df3b2c4f5f8456f8

MIT — Animata

Local source: `assets/adapters/web.tsx`
