# Ribbon Words

A bold white ribbon draws a big S down a black frame, carrying a short phrase that settles mid-ribbon and drifts, then rides off as the ribbon retracts.

Status: ready implementation

## When to use

- A directed phrase, path metaphor, connective motion or editorial flourish.

## Avoid

- Dense captions or a decorative ribbon with no narrative purpose.

## Adapt to this video prompt

1. Bind text, words.size, ribbon.width/color, path geometry and timing to the brief. Disable the second phrase unless it serves the story.
2. Inherit an existing line or mask as the ribbon; flatten its tangent into the next shot edge. Preserve continuous source phase.

Set the approved text, aspect ratio, brand palette, target frames, footage/narration and protected reading windows first. Choose this mechanism only when it expresses the shot’s purpose. Fast/slow pacing changes travel, density, accents and holds; never blindly speed up the whole scene.

## Implementation

Ready component: inspect API through library_inspect, place with library_place, or compose via @clipkit/components with item and bindPath.

Retain actual frame-native choreography; derive all interaction/scroll progress from integral project frames. No wall-clock playback.

Editable bindings: text, highlight, words.size, words.weight, words.tracking, words.centering, ribbon.color, ribbon.width, ribbon.style, ribbon.outline, second.enabled, second.text, second.offsetY, second.shift, second.delay, inverted, path.shape, path.centerY.

Prove the chosen composition in the three-second sample; component inclusion is optional.

## Source and rights

https://github.com/meetportal/cliplibrary/blob/c4585582f712294b82faccd4c9ed233308925f9b/src/compositions/scenes/kinetic-type/ribbon-words.tsx

MIT — Clip Library / clawwde

Local source: `assets/sources/cliplibrary/src/compositions/scenes/kinetic-type/ribbon-words.tsx`
