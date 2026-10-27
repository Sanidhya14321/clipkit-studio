# Scene Transition

A solid colour covers the frame and opens again, to bridge two clips on the timeline.

Status: ready implementation

## When to use

- A motivated overlapping handoff between two already composed shots.

## Avoid

- Hiding unrelated layouts with repeated full-screen wipes.

## Adapt to this video prompt

1. Choose kind, center, easing, color and holdFrames from incoming/outgoing geometry; budget overlap separately from copy.
2. Use as the transition mechanism, not the entire shot. Maintain object position and velocity on both sides.

Set the approved text, aspect ratio, brand palette, target frames, footage/narration and protected reading windows first. Choose this mechanism only when it expresses the shot’s purpose. Fast/slow pacing changes travel, density, accents and holds; never blindly speed up the whole scene.

## Implementation

Ready component: inspect API through library_inspect, place with library_place, or compose via @clipkit/components with item and bindPath.

Retain actual frame-native choreography; derive all interaction/scroll progress from integral project frames. No wall-clock playback.

Editable bindings: kind, color, easing, holdFrames, centerX, centerY.

Prove the chosen composition in the three-second sample; component inclusion is optional.

## Source and rights

https://github.com/meetportal/cliplibrary/blob/c4585582f712294b82faccd4c9ed233308925f9b/src/compositions/scenes/transitions/scene-transition.tsx

MIT — Clip Library / clawwde

Local source: `assets/sources/cliplibrary/src/compositions/scenes/transitions/scene-transition.tsx`
