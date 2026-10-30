---
id: rendercomp-free/social-reel
name: SocialReel
source: rendercomp-free
sourceCommit: f648980e528defb7594e24c6447055f0540f340b
kind: remotion-clip
runtime: direct
determinism: native
license: MIT
spdx: MIT
licenseClass: permissive
grade: B
deps: [react, remotion]
cost: 2
aspects: []
tags: [social-reel]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `SocialReel` from `src/components/social-reel/SocialReel.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- brandName: string
- handle: string
- headline: string
- headlineAccent: string
- secondCopy: string
- infoLines: string[]
- ctaText: string
- accentColor: string
- bgFrom: string
- bgVia: string
- bgTo: string
- caption: string
- likeCount: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 31-31)
- const `DURATION_FRAMES` (lines 32-32)
- const `SANS` (lines 34-34)
- type `SocialReelProps` (lines 36-50)
- const `defaultSocialReelProps` (lines 52-66)
- const `FadeIn` (lines 69-86)
- const `AccentLine` (lines 89-114)
- const `PulseButton` (lines 117-151)
- const `HeartGlyph` (lines 154-163)
- const `CommentGlyph` (lines 165-174)
- const `ShareGlyph` (lines 176-185)
- const `ActionRail` (lines 187-235)
- const `TopBar` (lines 238-296)
- const `BottomChrome` (lines 299-342)
- const `PlayerBody` (lines 345-473)
- const `SocialReel` (lines 475-503)
- default `default` (lines 505-505)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, spring, Easing, Sequence, AbsoluteFill.

## Adaptation presets



## Owner notes


