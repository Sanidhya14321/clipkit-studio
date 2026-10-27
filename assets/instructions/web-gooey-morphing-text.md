# Gooey Morphing Text

Actual Magic UI source adapted to seekable video frame time

Status: ready implementation

## When to use

- A short semantic word change or material transformation.

## Avoid

- Long copy, dense captions, or continuous morphing while a claim must be read.

## Adapt to this video prompt

1. Bind the texts array to approved words; tune morphFrames/holdFrames, size and colors. Keep each resolved word readable at phone size.
2. Retain the original paired blur and SVG threshold mechanism. Match its resolved silhouette to the next object.

Set the approved text, aspect ratio, brand palette, target frames, footage/narration and protected reading windows first. Choose this mechanism only when it expresses the shot’s purpose. Fast/slow pacing changes travel, density, accents and holds; never blindly speed up the whole scene.

## Implementation

Ready component: inspect API through library_inspect, place with library_place, or compose via @clipkit/components with item and bindPath.

Retain actual frame-native choreography; derive all interaction/scroll progress from integral project frames. No wall-clock playback.

Editable bindings: texts[0], texts[1], texts[2], color, background, size, morphFrames, holdFrames.

Prove the chosen composition in the three-second sample; component inclusion is optional.

## Source and rights

https://github.com/magicuidesign/magicui/tree/efe76fc5755eca1142dd4f74c1d6f0ace2887e3c

MIT — Magic UI

Local source: `assets/adapters/web.tsx`
