---
id: rendercomp-free/slide-wipe
name: SlideWipe
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
tags: [slide-wipe]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `SlideWipe` from `src/components/slide-wipe/SlideWipe.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- colorBefore: string
- colorAfter: string
- labelBefore: string
- labelAfter: string
- textColor: string
- backdropColor?: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 12-12)
- const `DURATION_FRAMES` (lines 13-13)
- type `SlideWipeProps` (lines 15-23)
- const `DEFAULT_BACKDROP` (lines 25-25)
- const `defaultSlideWipeProps` (lines 27-34)
- const `FONT` (lines 36-36)
- const `DELAY` (lines 40-40)
- const `TRANSITION` (lines 41-41)
- const `EASE_IN_OUT` (lines 44-44)
- const `PageLayer` (lines 48-77)
- const `SlideWipe` (lines 82-122)
- default `default` (lines 124-124)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


