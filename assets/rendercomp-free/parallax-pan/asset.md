---
id: rendercomp-free/parallax-pan
name: ParallaxPan
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
tags: [parallax-pan]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `ParallaxPan` from `src/components/parallax-pan/ParallaxPan.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- title: string
- subtitle?: string
- skyTop: string
- skyBottom: string
- mountainColor: string
- hillColor: string
- groundColor: string
- textColor: string
- panDistance?: number

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 14-14)
- const `DURATION_FRAMES` (lines 15-15)
- const `FONT` (lines 17-17)
- type `ParallaxPanProps` (lines 19-38)
- const `defaultParallaxPanProps` (lines 40-50)
- const `ParallaxPan` (lines 52-206)
- const `ParallaxLayer` (lines 208-223)
- const `seededHeight` (lines 226-230)
- const `MountainRange` (lines 232-258)
- const `Trees` (lines 260-288)
- default `default` (lines 290-290)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


