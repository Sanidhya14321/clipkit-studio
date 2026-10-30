---
id: rendercomp-free/card-flip-transition
name: CardFlipTransition
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
tags: [card-flip-transition]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `CardFlipTransition` from `src/components/card-flip-transition/CardFlipTransition.tsx` (no doc comment).

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

- const `FPS` (lines 10-10)
- const `DURATION_FRAMES` (lines 11-11)
- const `FONT_EN` (lines 13-13)
- type `CardFlipTransitionProps` (lines 15-26)
- const `DEFAULT_BACKDROP` (lines 28-28)
- const `defaultCardFlipTransitionProps` (lines 30-37)
- const `Face` (lines 39-70)
- const `CardFlipTransition` (lines 72-155)
- default `default` (lines 157-157)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, interpolate, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


