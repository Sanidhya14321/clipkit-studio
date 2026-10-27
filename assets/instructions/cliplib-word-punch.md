# Word Punch

One huge word at a time types on behind its full stop and deletes from the front, or punches in.

Status: ready implementation

## When to use

- A sharp claim, launch hook or single decisive word.

## Avoid

- Long explanation, measured dialogue or every phrase in a film.

## Adapt to this video prompt

1. Set text from the exact approved copy; adjust size/tracking to aspect ratio, wordFrames to reading time, and look palette to brand. Choose type or punch by the intention.
2. Carry the outgoing word edge/period into the next shot instead of fading to another centered title.

Set the approved text, aspect ratio, brand palette, target frames, footage/narration and protected reading windows first. Choose this mechanism only when it expresses the shot’s purpose. Fast/slow pacing changes travel, density, accents and holds; never blindly speed up the whole scene.

## Implementation

Ready component: inspect API through library_inspect, place with library_place, or compose via @clipkit/components with item and bindPath.

Retain actual frame-native choreography; derive all interaction/scroll progress from integral project frames. No wall-clock playback.

Editable bindings: text, highlight, style, size, shrink, weight, tracking, accentPunctuation, delay, wordFrames, swapFrames, typeFrames, punch, rise, placement.horizontal, placement.vertical, placement.insetX, placement.insetY.

Prove the chosen composition in the three-second sample; component inclusion is optional.

## Source and rights

https://github.com/meetportal/cliplibrary/blob/c4585582f712294b82faccd4c9ed233308925f9b/src/compositions/scenes/kinetic-type/word-punch.tsx

MIT — Clip Library / clawwde

Local source: `assets/sources/cliplibrary/src/compositions/scenes/kinetic-type/word-punch.tsx`
