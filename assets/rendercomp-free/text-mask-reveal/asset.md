---
id: rendercomp-free/text-mask-reveal
name: TextMaskReveal
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
tags: [text-mask-reveal]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `TextMaskReveal` from `src/components/text-mask-reveal/TextMaskReveal.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- accentColor: string
- textColor: string
- lines: string[]

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 11-11)
- const `DURATION_FRAMES` (lines 12-12)
- type `TextMaskRevealProps` (lines 14-19)
- const `defaultTextMaskRevealProps` (lines 21-26)
- const `FONT` (lines 28-28)
- const `easeInOut` (lines 31-32)
- const `TextMaskReveal` (lines 34-198)
- default `default` (lines 200-200)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, AbsoluteFill.

## Adaptation presets



## Owner notes


