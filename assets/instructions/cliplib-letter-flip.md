# Letter Flip

Each letter swings in from its mirror image and lands in turn.

Status: ready implementation

## When to use

- A short transformation, reveal or change of meaning.

## Avoid

- A long paragraph or constant flipping under narration.

## Adapt to this video prompt

1. Preserve the exact string; tune stagger, flipFrames, depth and typography for a legible resolved pose. Hold before exiting.
2. Match a rotating character plane to the incoming surface; preserve its direction across the cut.

Set the approved text, aspect ratio, brand palette, target frames, footage/narration and protected reading windows first. Choose this mechanism only when it expresses the shot’s purpose. Fast/slow pacing changes travel, density, accents and holds; never blindly speed up the whole scene.

## Implementation

Ready component: inspect API through library_inspect, place with library_place, or compose via @clipkit/components with item and bindPath.

Retain actual frame-native choreography; derive all interaction/scroll progress from integral project frames. No wall-clock playback.

Editable bindings: text, size, weight, tracking, startAngle, slide, delay, stagger, flipFrames, depth, highlight, glowOut, placement.horizontal, placement.vertical, placement.insetX, placement.insetY, placement.width, look.showBackdrop.

Prove the chosen composition in the three-second sample; component inclusion is optional.

## Source and rights

https://github.com/meetportal/cliplibrary/blob/c4585582f712294b82faccd4c9ed233308925f9b/src/compositions/scenes/text-reveals/letter-flip.tsx

MIT — Clip Library / clawwde

Local source: `assets/sources/cliplibrary/src/compositions/scenes/text-reveals/letter-flip.tsx`
