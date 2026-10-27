# Chunky Type Stack

Chunky hand-cut letters pop in one by one into a staggered headline stack, then a giant glyph punches through to an end tag.

Status: ready implementation

## When to use

- An intentionally tactile typography object or bold editorial stack.

## Avoid

- A default opening for every brief; a small centered poster with unused space.

## Adapt to this video prompt

1. Bind lines[i].text/size/offset and look palette to the approved hierarchy. Choose crop, scale and camera placement before animating letters.
2. Author a parent camera, hinge or mask around the actual stack. A zoom ending on the same poster is not a complete creative direction.

Set the approved text, aspect ratio, brand palette, target frames, footage/narration and protected reading windows first. Choose this mechanism only when it expresses the shot’s purpose. Fast/slow pacing changes travel, density, accents and holds; never blindly speed up the whole scene.

## Implementation

Ready component: inspect API through library_inspect, place with library_place, or compose via @clipkit/components with item and bindPath.

Retain actual frame-native choreography; derive all interaction/scroll progress from integral project frames. No wall-clock playback.

Editable bindings: lines[0].text, lines[0].size, lines[0].offset, lines[1].text, lines[1].size, lines[1].offset, lines[2].text, lines[2].size, lines[2].offset, lines[3].text, lines[3].size, lines[3].offset, color, font, lineGap, jitter, seed, delay.

Prove the chosen composition in the three-second sample; component inclusion is optional.

## Source and rights

https://github.com/meetportal/cliplibrary/blob/c4585582f712294b82faccd4c9ed233308925f9b/src/compositions/scenes/kinetic-type/chunky-type-stack.tsx

MIT — Clip Library / clawwde

Local source: `assets/sources/cliplibrary/src/compositions/scenes/kinetic-type/chunky-type-stack.tsx`
