---
id: rendercomp-free/whip-pan
name: WhipPan
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
tags: [whip-pan]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `WhipPan` from `src/components/whip-pan/WhipPan.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- scenes: WhipPanScene[]
- panLeft?: boolean
- maxBlur?: number

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 15-15)
- const `DURATION_FRAMES` (lines 16-16)
- const `FONT_SANS` (lines 18-18)
- type `WhipPanScene` (lines 21-30)
- type `WhipPanProps` (lines 32-39)
- const `defaultWhipPanProps` (lines 41-49)
- const `ScenePane` (lines 52-101)
- const `WhipPan` (lines 103-184)
- default `default` (lines 186-186)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


