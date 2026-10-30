---
id: rendercomp-free/flip-page-transition
name: FlipPageTransition
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
tags: [flip-page-transition]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `FlipPageTransition` from `src/components/flip-page-transition/FlipPageTransition.tsx` (no doc comment).

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

- const `FPS` (lines 19-19)
- const `DURATION_FRAMES` (lines 20-20)
- const `FONT` (lines 22-22)
- const `SCENE_EASING` (lines 26-26)
- const `FLIP_DELAY` (lines 30-30)
- const `FLIP_DURATION` (lines 31-31)
- const `PRE_LIFT_FRAMES` (lines 34-34)
- type `FlipPageTransitionProps` (lines 36-42)
- const `defaultFlipPageTransitionProps` (lines 44-50)
- const `PageLayer` (lines 53-79)
- const `flipProgress` (lines 84-94)
- const `FlipPage` (lines 104-179)
- const `FlipPageTransition` (lines 181-258)
- default `default` (lines 260-260)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, interpolate, Easing, Sequence, AbsoluteFill.

## Adaptation presets



## Owner notes


