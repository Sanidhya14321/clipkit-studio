---
id: rendercomp-free/ink-spread-transition
name: InkSpreadTransition
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
tags: [ink-spread-transition]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `InkSpreadTransition` from `src/components/ink-spread-transition/InkSpreadTransition.tsx` (no doc comment).

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

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 23-23)
- const `DURATION_FRAMES` (lines 24-24)
- const `INK_START` (lines 32-32)
- const `INK_END` (lines 33-33)
- const `LABEL_IN` (lines 34-34)
- const `LABEL_FULL` (lines 35-35)
- const `DROP_COUNT` (lines 36-36)
- const `SPECK_COUNT` (lines 37-37)
- const `FONT` (lines 39-39)
- const `easeInOut` (lines 43-43)
- const `seededRandom` (lines 46-49)
- type `InkSpreadTransitionProps` (lines 51-57)
- const `defaultInkSpreadTransitionProps` (lines 59-65)
- const `PageLayer` (lines 68-94)
- type `Drop` (lines 96-96)
- const `buildDrops` (lines 100-128)
- const `buildSpecks` (lines 131-151)
- const `InkSpreadTransition` (lines 153-344)
- default `default` (lines 346-346)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


