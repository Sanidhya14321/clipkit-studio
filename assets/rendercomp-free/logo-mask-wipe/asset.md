---
id: rendercomp-free/logo-mask-wipe
name: LogoMaskWipe
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
tags: [logo-mask-wipe]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `LogoMaskWipe` from `src/components/logo-mask-wipe/LogoMaskWipe.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- accentColor: string
- textColor: string
- monogram: string
- brandText: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 12-12)
- const `DURATION_FRAMES` (lines 13-13)
- type `LogoMaskWipeProps` (lines 15-21)
- const `defaultLogoMaskWipeProps` (lines 23-29)
- const `FONT` (lines 31-31)
- const `LogoMaskWipe` (lines 33-210)
- default `default` (lines 212-212)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, spring, AbsoluteFill.

## Adaptation presets



## Owner notes


