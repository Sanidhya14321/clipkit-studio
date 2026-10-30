---
id: rendercomp-free/lower-third-glass-card
name: LowerThirdGlassCard
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
tags: [lower-third-glass-card]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `LowerThirdGlassCard` from `src/components/lower-third-glass-card/LowerThirdGlassCard.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- accentColor: string
- textColor: string
- nameText: string
- titleText: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 12-12)
- const `DURATION_FRAMES` (lines 13-13)
- type `LowerThirdGlassCardProps` (lines 15-21)
- const `defaultLowerThirdGlassCardProps` (lines 23-29)
- const `FONT` (lines 31-31)
- const `LowerThirdGlassCard` (lines 33-208)
- default `default` (lines 210-210)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, spring, AbsoluteFill.

## Adaptation presets



## Owner notes


