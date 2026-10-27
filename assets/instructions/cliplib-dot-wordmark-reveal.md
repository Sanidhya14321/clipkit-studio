# Dot Wordmark Reveal

A jade dot hops across a cream frame dropping a tagline word by word, swells into a full jade field where a rounded wordmark writes itself stroke by stroke, then the field morphs into a rounded tile badge that tumbles the lockup away.

Status: ready implementation

## When to use

- A brief brand resolve with a recognizable mark or dot motif.

## Avoid

- Using an arbitrary dot as decoration or withholding the brand too long.

## Adapt to this video prompt

1. Use real copy and brand palette; tune dot/type/roll/timing for exact final hold. Keep last readable frame intentional.
2. The dot can inherit a prior period, cursor or accent that already has meaning; resolve into a stable signature.

Set the approved text, aspect ratio, brand palette, target frames, footage/narration and protected reading windows first. Choose this mechanism only when it expresses the shot’s purpose. Fast/slow pacing changes travel, density, accents and holds; never blindly speed up the whole scene.

## Implementation

Ready component: inspect API through library_inspect, place with library_place, or compose via @clipkit/components with item and bindPath.

Retain actual frame-native choreography; derive all interaction/scroll progress from integral project frames. No wall-clock playback.

Editable bindings: copy.tagline, copy.wordmark, copy.monogram, copy.accentLetter, look.showBackdrop, look.paper, look.brand, look.flash, look.onBrand, look.mark, look.accent, look.monogramInk, look.streak, look.sheen, look.badgeCorner, look.glow, look.pulse, look.monogramDot.

Prove the chosen composition in the three-second sample; component inclusion is optional.

## Source and rights

https://github.com/meetportal/cliplibrary/blob/c4585582f712294b82faccd4c9ed233308925f9b/src/compositions/scenes/brand/dot-wordmark-reveal.tsx

MIT — Clip Library / clawwde

Local source: `assets/sources/cliplibrary/src/compositions/scenes/brand/dot-wordmark-reveal.tsx`
